use std::rc::Rc;

use super::{Component, ComponentRc};
use super::backend::*;
use super::node::*;

pub struct Context<B: Backend> {
    group_holder: VirtualNodeRc<B>,
    root: Option<ComponentNodeRc<B>>,
    backend: Rc<B>,
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
        };
        ret
    }
    pub fn root_component<C: 'static + Component>(&self) -> Option<ComponentRc<B, C>> {
        self.root.clone().map(|x| {
            x.with_type::<C>()
        })
    }
    pub fn set_root_component<C: 'static + Component>(&mut self, component: Box<C>) {
        let component_node = create_component(&mut self.group_holder.borrow_mut(), "maomi", component, "".into(), vec![], None);
        self.root = Some(component_node);
        self.backend.set_root_node(&self.root.as_ref().unwrap().borrow_mut().backend_element);
    }
}
