use maomi::backend::{tree::*, *};

pub struct DomSlot {
    // TODO
}

impl DomSlot {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl BackendSlot for DomSlot {
    type BaseBackend = crate::DomBackend;

    fn associate_element(&mut self, content_element: ForestValueMut<crate::DomGeneralElement>) {
        todo!()
    }
}
