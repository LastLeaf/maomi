use maomi::backend::*;

pub struct DomTextNode {
    // TODO
}

impl DomTextNode {
    fn new() -> Self {
        Self {}
    }
}

impl BackendTextNode for DomTextNode {
    type BaseBackend = crate::DomBackend;

    fn set_text(&mut self, content: &str) {
        todo!()
    }

    fn into_general_element(
        self,
    ) -> <<Self as BackendTextNode>::BaseBackend as Backend>::GeneralElement
    where
        Self: Sized,
    {
        self.into()
    }
}
