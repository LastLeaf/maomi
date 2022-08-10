use maomi::backend::*;

#[doc(hidden)]
pub struct DomVirtualElement {
    // empty
}

impl DomVirtualElement {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl BackendVirtualElement for DomVirtualElement {
    type BaseBackend = crate::DomBackend;
}
