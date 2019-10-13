use std::rc::Rc;

use super::Component;
use super::backend::Backend;
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
                VirtualNode {
                    backend: backend.clone(),
                    tag_name: "body".into(),
                    property: VirtualNodeProperty::None,
                    children: vec![],
                }
            ),
            root: None,
            backend,
        };
        ret
    }
    pub fn root_component(&self) -> &Option<ComponentNodeRc<B>> {
        &self.root
    }
    pub fn set_root_component<C: 'static + Component>(&mut self, component: Box<C>) {
        let component_node = create_component(&mut self.group_holder.borrow_mut(), component, vec![]);
        self.root = Some(component_node);
        self.backend.set_root_node(&self.root.as_ref().unwrap().borrow_mut().backend_element);
    }
}
