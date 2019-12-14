use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use super::{Component, ComponentRc};
use super::backend::*;
use super::node::*;
use super::prerender::{PrerenderReader, match_prerendered_tree};

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
    pub fn new_prerendered<C: 'static + Component<B>>(backend: B, prerendered_data: Box<[u8]>) -> Context<B> {
        let backend = Rc::new(backend);
        if !backend.is_prerendering() {
            panic!("the backend is not in prerendering progress");
        }
        let group_holder = VirtualNodeRc::new_with_me_cell_group(
            VirtualNode::new_empty(backend.clone())
        );
        let scheduler = Rc::new(Scheduler::new());
        let mut prerendered_data = PrerenderReader::new(prerendered_data);
        let prerendered_root = create_component::<_, _, C>(&mut group_holder.borrow_mut(), scheduler.clone(), "maomi", vec![], None, Some(&mut prerendered_data)).with_type::<C>().into_node();
        match_prerendered_tree(prerendered_root.borrow_mut(), &backend);
        let ret = Self {
            group_holder,
            root: Some(prerendered_root),
            backend,
            scheduler,
        };
        ret
    }
    pub fn backend(&self) -> &B {
        &self.backend
    }
    pub fn get_prerendered_data(&self) -> Vec<u8> {
        let mut s = super::prerender::PrerenderWriter::new();
        self.root.as_ref().unwrap().borrow().serialize_component_tree_data(&mut s);
        s.end()
    }
    pub fn root_component<C: 'static + Component<B>>(&self) -> Option<ComponentRc<B, C>> {
        self.root.clone().map(|x| {
            x.with_type::<C>()
        })
    }
    pub fn new_root_component<C: 'static + Component<B>>(&mut self) -> ComponentRc<B, C> {
        let ret = create_component::<_, _, C>(&mut self.group_holder.borrow_mut(), self.scheduler.clone(), "maomi", vec![], None, None).with_type::<C>();
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
