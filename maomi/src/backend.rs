pub trait Backend {
    type GeneralElement: BackendGeneralElement<BaseBackend = Self>;
    type VirtualElement: BackendVirtualElement<BaseBackend = Self>;
    type ShadowRoot: BackendShadowRoot<BaseBackend = Self>;
    type Slot: BackendSlot<BaseBackend = Self>;
    type Component: BackendComponent<BaseBackend = Self>;

    /// Get the root element
    fn root_mut(&mut self) -> &mut Self::Component;
}

pub trait BackendGeneralElement: Sized
    + TryInto<<<Self as BackendGeneralElement>::BaseBackend as Backend>::VirtualElement>
    + TryInto<<<Self as BackendGeneralElement>::BaseBackend as Backend>::ShadowRoot>
    + TryInto<<<Self as BackendGeneralElement>::BaseBackend as Backend>::Slot>
    + TryInto<<<Self as BackendGeneralElement>::BaseBackend as Backend>::Component>
{
    type BaseBackend: Backend;

    /// Append some child elements
    fn append_children(
        &mut self,
        children: impl IntoIterator<Item = <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement>,
    );

    /// Splice some child elements
    fn splice_children(
        &mut self,
        range: impl std::ops::RangeBounds<usize>,
        children: impl IntoIterator<Item = <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement>,
    );

    /// Get a child element
    fn child(
        &self,
        index: usize,
    );

    /// Iterate all children
    fn children(
        &self,
    );
}

pub trait BackendVirtualElement: Into<<<Self as BackendVirtualElement>::BaseBackend as Backend>::GeneralElement> {
    type BaseBackend: Backend;
}

pub trait BackendSlot: Into<<<Self as BackendSlot>::BaseBackend as Backend>::GeneralElement> {
    type BaseBackend: Backend;
}

pub trait BackendShadowRoot: Into<<<Self as BackendShadowRoot>::BaseBackend as Backend>::GeneralElement> {
    type BaseBackend: Backend;

    /// Create an element in the shadow tree
    fn create<T>(&mut self) -> T
    where
        T: Into<<<Self as BackendShadowRoot>::BaseBackend as Backend>::GeneralElement>;
}

pub trait BackendComponent: Into<<<Self as BackendComponent>::BaseBackend as Backend>::GeneralElement> {
    type BaseBackend: Backend;

    /// Get the shadow root element
    fn shadow_root() -> <<Self as BackendComponent>::BaseBackend as Backend>::ShadowRoot;
}

pub trait BackendElement: Into<<<Self as BackendElement>::BaseBackend as Backend>::GeneralElement> {
    type BaseBackend: Backend;
}

pub trait SupportBackend<B: Backend> {
    fn create(&mut self) -> Result<B::GeneralElement, crate::error::Error>;
    fn update(&mut self, backend_element: B::GeneralElement) -> Result<(), crate::error::Error>;
}
