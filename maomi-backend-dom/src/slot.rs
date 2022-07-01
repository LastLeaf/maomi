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
}
