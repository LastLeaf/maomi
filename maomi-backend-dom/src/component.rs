use maomi::backend::*;

pub struct DomComponent {
    // TODO
}

impl DomComponent {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl BackendComponent for DomComponent {
    type BaseBackend = crate::DomBackend;

    fn shadow_root_mut(
        &mut self,
    ) -> &mut <<Self as BackendComponent>::BaseBackend as Backend>::GeneralElement {
        todo!()
    }

    fn into_general_element(
        self,
    ) -> <<Self as BackendComponent>::BaseBackend as Backend>::GeneralElement
    where
        Self: Sized,
    {
        self.into()
    }
}
