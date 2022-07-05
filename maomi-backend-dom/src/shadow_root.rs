use maomi::backend::*;

pub struct DomShadowRoot {
    // TODO
}

impl DomShadowRoot {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl BackendShadowRoot for DomShadowRoot {
    type BaseBackend = crate::DomBackend;
}
