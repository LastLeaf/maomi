use maomi::backend::*;

pub struct DomVirtualElement {
    // TODO
}

impl DomVirtualElement {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl BackendVirtualElement for DomVirtualElement {
    type BaseBackend = crate::DomBackend;

    fn into_general_element(
        self,
    ) -> <<Self as BackendVirtualElement>::BaseBackend as Backend>::GeneralElement
    where
        Self: Sized,
    {
        self.into()
    }
}
