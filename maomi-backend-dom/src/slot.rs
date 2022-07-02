use maomi::backend::*;

pub struct DomSlot {
    // TODO
}

impl DomSlot {
    fn new() -> Self {
        Self {}
    }
}

impl BackendSlot for DomSlot {
    type BaseBackend = crate::DomBackend;

    fn associate_element(
        &mut self,
        content_element: <<Self as BackendSlot>::BaseBackend as Backend>::GeneralElement,
    ) {
        todo!()
    }
}
