use maomi::backend::*;

pub struct DomVirtualElement {
    // TODO
}

impl DomVirtualElement {
    fn new() -> Self {
        Self {}
    }
}

impl BackendVirtualElement for DomVirtualElement {
    type BaseBackend = crate::DomBackend;
}
