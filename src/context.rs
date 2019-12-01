use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use super::{Component, ComponentRc};
use super::backend::*;
use super::node::*;

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
