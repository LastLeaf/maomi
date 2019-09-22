use std::rc::Rc;

use super::Component;
use super::backend::Backend;
use super::node::*;

pub struct Context<B: Backend> {
    backend: Rc<B>,
    root: NativeNodeRc<B>,
}

impl<B: Backend> Context<B> {
    pub fn new(backend: B) -> Context<B> {
        let backend = Rc::new(backend);
        let ret = Self {
            root: NativeNodeRc::new_with_me_cell_group(
                NativeNode {
                    backend: backend.clone(),
                    tag_name: "body".into(),
                    attributes: vec![].into(),
                    children: vec![],
                }
            ),
            backend,
        };
        ret
    }
    pub fn set_root_component<C: 'static + Component<B>>(&mut self, component: Box<C>) {
        let component_node = create_component(&mut self.root.borrow_mut(), component, vec![]);
        unimplemented!()
    }
}
