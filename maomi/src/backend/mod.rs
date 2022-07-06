pub mod tree;
use crate::error::Error;
use tree::*;

/// A backend
pub trait Backend {
    type GeneralElement: BackendGeneralElement<BaseBackend = Self>;
    type VirtualElement: BackendVirtualElement<BaseBackend = Self>;
    type TextNode: BackendTextNode<BaseBackend = Self>;

    /// Get the root element
    fn root_mut(&mut self) -> ForestNodeMut<Self::GeneralElement>;
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
pub trait BackendGeneralElement {
    type BaseBackend: Backend;

    /// Try casting to slot
    fn as_virtual_element_mut<'b>(
        this: &'b mut ForestNodeMut<Self>,
    ) -> Option<
        ForestValueMut<
            'b,
            <<Self as BackendGeneralElement>::BaseBackend as Backend>::VirtualElement,
        >,
    >
    where
        Self: Sized;

    /// Try casting to slot
    fn as_text_node_mut<'b>(
        this: &'b mut ForestNodeMut<Self>,
    ) -> Option<
        ForestValueMut<'b, <<Self as BackendGeneralElement>::BaseBackend as Backend>::TextNode>,
    >
    where
        Self: Sized;

    /// Create a virtual element in the shadow tree
    fn create_virtual_element<'b>(
        this: &'b mut ForestNodeMut<Self>,
    ) -> Result<ForestTree<<Self::BaseBackend as Backend>::GeneralElement>, Error>
    where
        Self: Sized;

    /// Create a text node in the shadow tree
    fn create_text_node<'b>(
        this: &'b mut ForestNodeMut<Self>,
        content: &str,
    ) -> Result<ForestTree<<Self::BaseBackend as Backend>::GeneralElement>, Error>
    where
        Self: Sized;

    /// Append a child element
    fn append<'b>(
        this: &'b mut ForestNodeMut<Self>,
        child: ForestTree<
            <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    ) where
        Self: Sized;

    /// Insert an element before this element
    fn insert<'b>(
        this: &'b mut ForestNodeMut<Self>,
        sibling: ForestTree<
            <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    ) where
        Self: Sized;

    /// Detach this element
    fn detach<'b>(this: &'b mut ForestNodeMut<Self>)
    where
        Self: Sized;
}

/// A component in the backend
pub trait BackendComponent {
    type BaseBackend: Backend;

    /// Get the shadow root element
    fn shadow_root_mut(
        &mut self,
    ) -> ForestNodeMut<<<Self as BackendComponent>::BaseBackend as Backend>::GeneralElement>;
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
        content_element: ForestValueMut<
            <<Self as BackendSlot>::BaseBackend as Backend>::GeneralElement,
        >,
    );
}

/// A virtual element in the backend
pub trait BackendVirtualElement {
    type BaseBackend: Backend;
}

/// A text node in the backend
pub trait BackendTextNode {
    type BaseBackend: Backend;

    /// Set the text content
    fn set_text(&mut self, content: &str);
}

/// A trait that indicates a component or a backend implemented element for the backend
pub trait SupportBackend<B: Backend> {
    /// Create with a backend element
    fn create<'b>(
        parent: &'b mut ForestNodeMut<B::GeneralElement>,
    ) -> Result<(Self, ForestTree<B::GeneralElement>), crate::error::Error>
    where
        Self: Sized;

    /// Indicate that the pending updates should be applied
    fn apply_updates<'b>(
        &'b mut self,
        backend_element: &'b mut ForestNodeMut<B::GeneralElement>,
    ) -> Result<(), crate::error::Error>;
}
