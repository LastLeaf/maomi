pub trait Backend {
    type BackendElement;
    fn root_node(&self) -> &Self::BackendElement;
}
