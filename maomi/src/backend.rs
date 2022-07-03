/// A backend
pub trait Backend {
    type GeneralElement: BackendGeneralElement<BaseBackend = Self>;
    type Component: BackendComponent<BaseBackend = Self>;
    type ShadowRoot: BackendShadowRoot<BaseBackend = Self>;
    type Slot: BackendSlot<BaseBackend = Self>;
    type VirtualElement: BackendVirtualElement<BaseBackend = Self>;
    type TextNode: BackendTextNode<BaseBackend = Self>;

    /// Get the root element
    fn root_mut(&mut self) -> &mut Self::GeneralElement;
}

/// The general type of the elements of the backend
///
/// The backend can contain several types of elements.
///
/// Some special kinds of elements should not be treated as normal elements.
/// * A `Component` represents a component which has a `ShadowRoot` element attached to it.
/// * A `Slot` is a slot for its owner component.
/// * A `VirtualElement` is an element which has no special meaning.
///
/// A `TextNode` is a text node.
/// The backend can define other types of elements.
pub trait BackendGeneralElement:
    Sized
    + TryInto<<<Self as BackendGeneralElement>::BaseBackend as Backend>::Component>
    + TryInto<<<Self as BackendGeneralElement>::BaseBackend as Backend>::ShadowRoot>
    + TryInto<<<Self as BackendGeneralElement>::BaseBackend as Backend>::Slot>
    + TryInto<<<Self as BackendGeneralElement>::BaseBackend as Backend>::VirtualElement>
    + TryInto<<<Self as BackendGeneralElement>::BaseBackend as Backend>::TextNode>
{
    type BaseBackend: Backend;

    /// Append some child elements
    fn append_children(
        &mut self,
        children: impl IntoIterator<
            Item = <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    );

    /// Splice some child elements
    fn splice_children(
        &mut self,
        range: impl std::ops::RangeBounds<usize>,
        children: impl IntoIterator<
            Item = <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    );

    /// Get the next sibling
    ///
    /// NOTE
    /// We can simply use iterators but rust GAT is not stable yet,
    /// and this impl requires the backend being linked-list based.
    fn next_sibling_mut<'a>(
        &'a mut self,
    ) -> Option<&mut <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement>;

    /// Get the first child
    fn first_child_mut<'a>(
        &'a mut self,
    ) -> Option<&mut <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement>;

    /// Try casting to component
    fn as_component_mut(
        &mut self,
    ) -> Option<&mut <<Self as BackendGeneralElement>::BaseBackend as Backend>::Component>;

    /// Try casting to slot
    fn as_slot_mut(
        &mut self,
    ) -> Option<&mut <<Self as BackendGeneralElement>::BaseBackend as Backend>::Slot>;

    /// Try casting to slot
    fn as_text_node_mut(
        &mut self,
    ) -> Option<&mut <<Self as BackendGeneralElement>::BaseBackend as Backend>::TextNode>;

    /// Create a component in the shadow tree
    fn create_component(
        &mut self,
    ) -> <<Self as BackendGeneralElement>::BaseBackend as Backend>::Component;

    /// Create a slot in the shadow tree
    fn create_slot(&mut self) -> <<Self as BackendGeneralElement>::BaseBackend as Backend>::Slot;

    /// Create a virtual element in the shadow tree
    fn create_virtual_element(
        &mut self,
    ) -> <<Self as BackendGeneralElement>::BaseBackend as Backend>::VirtualElement;

    /// Create a text node in the shadow tree
    fn create_text_node(
        &mut self,
        content: &str,
    ) -> <<Self as BackendGeneralElement>::BaseBackend as Backend>::TextNode;
}

/// A component in the backend
pub trait BackendComponent {
    type BaseBackend: Backend;

    /// Get the shadow root element
    fn shadow_root_mut(
        &mut self,
    ) -> &mut <<Self as BackendComponent>::BaseBackend as Backend>::GeneralElement;

    /// Wrap the element as a general element
    fn into_general_element(
        self,
    ) -> <<Self as BackendComponent>::BaseBackend as Backend>::GeneralElement
    where
        Self: Sized;
}

/// A shadow root in the backend
pub trait BackendShadowRoot {
    type BaseBackend: Backend;
}

/// A slot in the backend
pub trait BackendSlot {
    type BaseBackend: Backend;

    /// Create a virtual element in the shadow tree
    fn associate_element(
        &mut self,
        content_element: <<Self as BackendSlot>::BaseBackend as Backend>::GeneralElement,
    );

    /// Wrap the element as a general element
    fn into_general_element(
        self,
    ) -> <<Self as BackendSlot>::BaseBackend as Backend>::GeneralElement
    where
        Self: Sized;
}

/// A virtual element in the backend
pub trait BackendVirtualElement {
    type BaseBackend: Backend;

    /// Wrap the element as a general element
    fn into_general_element(
        self,
    ) -> <<Self as BackendVirtualElement>::BaseBackend as Backend>::GeneralElement
    where
        Self: Sized;
}

/// A text node in the backend
pub trait BackendTextNode {
    type BaseBackend: Backend;

    /// Set the text content
    fn set_text(&mut self, content: &str);

    /// Wrap the element as a general element
    fn into_general_element(
        self,
    ) -> <<Self as BackendTextNode>::BaseBackend as Backend>::GeneralElement
    where
        Self: Sized;
}

/// A trait that indicates a component or a backend implemented element for the backend
pub trait SupportBackend<B: Backend> {
    /// Create with a backend element
    fn create(
        parent: &mut B::GeneralElement,
    ) -> Result<(Self, B::GeneralElement), crate::error::Error>
    where
        Self: Sized;

    /// Indicate that the pending updates should be applied
    fn apply_updates(
        &mut self,
        backend_element: &mut B::GeneralElement,
    ) -> Result<(), crate::error::Error>;
}
