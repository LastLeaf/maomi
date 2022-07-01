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

    fn shadow_root() -> <<Self as BackendComponent>::BaseBackend as Backend>::ShadowRoot {
        todo!()
    }
}
