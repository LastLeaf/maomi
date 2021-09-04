use futures::executor::block_on;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use super::backend::*;
use super::node::*;
use super::prerender::match_prerendered_tree;
use super::{Component, ComponentRc, PrerenderableComponent};

/// A rendering area
pub struct Context<B: Backend> {
    group_holder: VirtualNodeRc<B>,
    root: Option<ComponentNodeRc<B>>,
    backend: Rc<B>,
    scheduler: Rc<Scheduler>,
}

impl<B: Backend> Context<B> {
    fn new_group_holder(backend: Rc<B>, scheduler: Rc<Scheduler>) -> VirtualNodeRc<B> {
        let group_holder = VirtualNodeRc::new_with_me_cell_group(VirtualNode::new_empty(
            backend.clone(),
            scheduler.clone(),
        ));
        unsafe {
            group_holder
                .borrow_mut()
                .initialize(group_holder.downgrade())
        };
        group_holder
    }

    /// Create a new rendering area
    pub fn new(backend: B) -> Context<B> {
        let backend = Rc::new(backend);
        let scheduler = Rc::new(Scheduler::new());
        let ret = Self {
            group_holder: Self::new_group_holder(backend.clone(), scheduler.clone()),
            root: None,
            backend,
            scheduler,
        };
        ret
    }

    /// Create a new rendering area with prerendered data
    pub fn new_prerendered<C: PrerenderableComponent<B>>(
        backend: B,
        prerendered_data: &[u8],
    ) -> Context<B> {
        let backend = Rc::new(backend);
        if !backend.is_prerendering() {
            panic!("the backend is not in prerendering progress");
        }
        let scheduler = Rc::new(Scheduler::new());
        let group_holder = Self::new_group_holder(backend.clone(), scheduler.clone());
        let prerendered_data: C::PrerenderedData =
            bincode::deserialize_from(prerendered_data).unwrap();
        let root = create_component::<_, C>(
            &mut group_holder.borrow_mut().into(),
            scheduler.clone(),
            "maomi",
            vec![],
            None,
        )
        .with_type::<C>();
        backend.set_root_node(&root.borrow().backend_element());
        {
            let mut root = root.borrow_mut();
            <C as PrerenderableComponent<B>>::apply_prerendered_data(&mut root, &prerendered_data);
            root.apply_updates();
            match_prerendered_tree(root.as_node(), &backend);
            backend.end_prerendering();
            root.as_node().set_attached();
        }
        let root = root.into_node();
        let ret = Self {
            group_holder,
            root: Some(root),
            backend,
            scheduler,
        };
        ret
    }

    /// Create a new rendering area and do the prerendering.
    /// It returns the new context (in which `to_html` is useful on the root component), prerendered data, and meta data.
    pub fn prerender<C: PrerenderableComponent<B>>(
        backend: B,
        args: <C as PrerenderableComponent<B>>::Args,
    ) -> (
        Context<B>,
        Vec<u8>,
        <C as PrerenderableComponent<B>>::MetaData,
    ) {
        let backend = Rc::new(backend);
        if backend.is_prerendering() {
            panic!("the backend is already in prerendering progress");
        }
        let scheduler = Rc::new(Scheduler::new());
        let group_holder = Self::new_group_holder(backend.clone(), scheduler.clone());
        let root = create_component::<_, C>(
            &mut group_holder.borrow_mut().into(),
            scheduler.clone(),
            "maomi",
            vec![],
            None,
        )
        .with_type::<C>();
        backend.set_root_node(&root.borrow().backend_element());
        let (prerendered_data, meta_data) = {
            let mut root = root.borrow_mut();
            let (prerendered_data, meta_data) = block_on(
                <C as PrerenderableComponent<B>>::get_prerendered_data(&mut root, args),
            );
            <C as PrerenderableComponent<B>>::apply_prerendered_data(&mut root, &prerendered_data);
            root.apply_updates();
            (prerendered_data, meta_data)
        };
        let root = root.into_node();
        let ret = Self {
            group_holder,
            root: Some(root),
            backend,
            scheduler,
        };
        (
            ret,
            bincode::serialize(&prerendered_data).unwrap(),
            meta_data,
        )
    }

    /// Get the backend
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Get the root component with specified type
    pub fn root_component<C: 'static + Component<B>>(&self) -> Option<ComponentRc<B, C>> {
        self.root.clone().map(|x| x.with_type::<C>())
    }

    /// Get the root component
    pub fn root_component_node(&self) -> Option<ComponentNodeRc<B>> {
        self.root.clone()
    }

    /// Create a new root component
    pub fn new_root_component<C: 'static + Component<B>>(&mut self) -> ComponentRc<B, C> {
        let ret = create_component::<_, C>(
            &mut self.group_holder.borrow_mut().into(),
            self.scheduler.clone(),
            "maomi",
            vec![],
            None,
        )
        .with_type::<C>();
        ret
    }

    /// Set the root component
    pub fn set_root_component<C: 'static + Component<B>>(
        &mut self,
        component_node: ComponentRc<B, C>,
    ) {
        self.set_root_component_node(component_node.as_node().clone())
    }

    /// Set the root component
    pub fn set_root_component_node(&mut self, component_node: ComponentNodeRc<B>) {
        if let Some(old_root) = self.root.take() {
            old_root.borrow_mut().set_detached();
        }
        self.root = Some(component_node.clone());
        self.backend
            .set_root_node(&self.root.as_ref().unwrap().borrow_mut().backend_element);
        component_node.borrow_mut().set_attached();
    }
}

pub(crate) struct Scheduler {
    pending_tasks: RefCell<VecDeque<Box<dyn FnOnce()>>>,
}

impl Scheduler {
    fn new() -> Self {
        Self {
            pending_tasks: RefCell::new(VecDeque::new()),
        }
    }

    pub(crate) fn add_task<F: 'static + FnOnce()>(&self, task: F) {
        self.pending_tasks.borrow_mut().push_back(Box::new(task));
    }

    pub(crate) fn run_tasks(&self) {
        loop {
            let task = self.pending_tasks.borrow_mut().pop_front();
            match task {
                None => break,
                Some(x) => x(),
            }
        }
    }
}
