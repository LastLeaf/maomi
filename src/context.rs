use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use futures::executor::block_on;

use super::{Component, PrerenderableComponent, ComponentRc};
use super::backend::*;
use super::node::*;
use super::prerender::{match_prerendered_tree};

pub struct Context<B: Backend> {
    group_holder: VirtualNodeRc<B>,
    root: Option<ComponentNodeRc<B>>,
    backend: Rc<B>,
    scheduler: Rc<Scheduler>,
}

impl<B: Backend> Context<B> {
    pub fn new(backend: B) -> Context<B> {
        let backend = Rc::new(backend);
        let ret = Self {
            group_holder: VirtualNodeRc::new_with_me_cell_group(
                VirtualNode::new_empty(backend.clone())
            ),
            root: None,
            backend,
            scheduler: Rc::new(Scheduler::new()),
        };
        ret
    }
    pub fn new_prerendered<'a, C: 'static + PrerenderableComponent<'a, B>>(backend: B, prerendered_data: &'a [u8]) -> Context<B> {
        let backend = Rc::new(backend);
        if !backend.is_prerendering() {
            panic!("the backend is not in prerendering progress");
        }
        let group_holder = VirtualNodeRc::new_with_me_cell_group(
            VirtualNode::new_empty(backend.clone())
        );
        let scheduler = Rc::new(Scheduler::new());
        let prerendered_data: C::PrerenderedData = bincode::deserialize(&prerendered_data).unwrap();
        let root = create_component::<_, _, C>(&mut group_holder.borrow_mut(), scheduler.clone(), "maomi", vec![], None).with_type::<C>();
        backend.set_root_node(&root.borrow().backend_element());
        {
            let mut root = root.borrow_mut();
            <C as PrerenderableComponent<'_, B>>::apply_prerendered_data(&mut root, &prerendered_data);
            root.apply_updates();
            match_prerendered_tree(root.as_node().duplicate(), &backend);
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
    pub fn prerender<'a, C: 'static + PrerenderableComponent<'a, B>>(backend: B) -> (Context<B>, Vec<u8>) {
        let backend = Rc::new(backend);
        if backend.is_prerendering() {
            panic!("the backend is already in prerendering progress");
        }
        let group_holder = VirtualNodeRc::new_with_me_cell_group(
            VirtualNode::new_empty(backend.clone())
        );
        let scheduler = Rc::new(Scheduler::new());
        let root = create_component::<_, _, C>(&mut group_holder.borrow_mut(), scheduler.clone(), "maomi", vec![], None).with_type::<C>();
        let prerendered_data = {
            let mut root = root.borrow_mut();
            let prerendered_data = block_on(<C as PrerenderableComponent<'_, B>>::get_prerendered_data(&mut root));
            <C as PrerenderableComponent<'_, B>>::apply_prerendered_data(&mut root, &prerendered_data);
            root.apply_updates();
            prerendered_data
        };
        let root = root.into_node();
        let ret = Self {
            group_holder,
            root: Some(root),
            backend,
            scheduler,
        };
        (ret, bincode::serialize(&prerendered_data).unwrap())
    }
    pub fn backend(&self) -> &B {
        &self.backend
    }
    pub fn root_component<C: 'static + Component<B>>(&self) -> Option<ComponentRc<B, C>> {
        self.root.clone().map(|x| {
            x.with_type::<C>()
        })
    }
    pub fn new_root_component<C: 'static + Component<B>>(&mut self) -> ComponentRc<B, C> {
        let ret = create_component::<_, _, C>(&mut self.group_holder.borrow_mut(), self.scheduler.clone(), "maomi", vec![], None).with_type::<C>();
        ret
    }
    pub fn set_root_component<C: 'static + Component<B>>(&mut self, component_node: ComponentRc<B, C>) {
        if let Some(old_root) = self.root.take() {
            old_root.borrow_mut().set_detached();
        }
        self.root = Some(component_node.as_node().clone());
        self.backend.set_root_node(&self.root.as_ref().unwrap().borrow_mut().backend_element);
        component_node.as_node().borrow_mut().set_attached();
    }
}

pub(crate) struct Scheduler {
    pending_tasks: RefCell<VecDeque<Box<dyn FnOnce()>>>,
}

impl Scheduler {
    fn new() -> Self {
        Self {
            pending_tasks: RefCell::new(VecDeque::new())
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
