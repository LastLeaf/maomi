use maomi::backend::{tree::*, *};

use crate::DomGeneralElement;

pub struct DomComponent {
    shadow_root: ForestTree<DomGeneralElement>,
}

impl DomComponent {
    pub(crate) fn new(parent: &mut ForestNodeMut<DomGeneralElement>) -> Self {
        Self {
            shadow_root: parent.new_tree(DomGeneralElement::ShadowRoot(crate::DomShadowRoot::new()))
        }
    }
}

impl BackendComponent for DomComponent {
    type BaseBackend = crate::DomBackend;

    fn shadow_root_mut(&mut self) -> ForestNodeMut<crate::DomGeneralElement> {
        self.shadow_root.as_node_mut()
    }
}
