use maomi::backend::*;

pub struct DomShadowRoot {
    // TODO
}

impl DomShadowRoot {
    fn new() -> Self {
        Self {}
    }
}

impl BackendShadowRoot for DomShadowRoot {
    type BaseBackend = crate::DomBackend;
}
