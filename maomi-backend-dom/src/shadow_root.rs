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

    fn create<T>(&mut self) -> T
    where
        T: Into<<<Self as BackendShadowRoot>::BaseBackend as Backend>::GeneralElement> {
        todo!()
    }
}
