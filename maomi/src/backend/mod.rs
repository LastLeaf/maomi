use crate::error::Error;
pub use maomi_tree as tree;
use tree::*;

pub mod context;

/// A backend
pub trait Backend: 'static {
    type GeneralElement: BackendGeneralElement<BaseBackend = Self>;
    type VirtualElement: BackendVirtualElement<BaseBackend = Self>;
    type TextNode: BackendTextNode<BaseBackend = Self>;

    /// Get the root element
    fn root_mut(&mut self) -> ForestNodeMut<Self::GeneralElement>;
}

/// The general type of the elements of the backend
///
/// The backend can contain several types of elements.
/// * A `VirtualElement` is an element which should not layout in backend.
/// * A `TextNode` is a text node.
/// * The backend can define other types of elements.
pub trait BackendGeneralElement: 'static {
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
    ) -> Result<ForestNodeRc<<Self::BaseBackend as Backend>::GeneralElement>, Error>
    where
        Self: Sized;

    /// Create a text node in the shadow tree
    fn create_text_node(
        this: &mut ForestNodeMut<Self>,
        content: &str,
    ) -> Result<ForestNodeRc<<Self::BaseBackend as Backend>::GeneralElement>, Error>
    where
        Self: Sized;

    /// Append a child element
    fn append<'b>(
        this: &'b mut ForestNodeMut<Self>,
        child: ForestNodeRc<
            <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    ) where
        Self: Sized;

    /// Insert an element before this element
    fn insert<'b>(
        this: &'b mut ForestNodeMut<Self>,
        sibling: ForestNodeRc<
            <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    ) where
        Self: Sized;

    /// Detach this element
    fn detach(
        this: ForestNodeMut<Self>,
    ) -> ForestNodeRc<<<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement>
    where
        Self: Sized;
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

/// A trait that indicates a component or a backend-implemented element for the backend
pub trait SupportBackend<B: Backend> {
    /// Create with a backend element
    fn create<'b>(
        owner: &'b mut ForestNodeMut<B::GeneralElement>,
        init: impl FnOnce(&mut Self) -> Result<(), Error>,
    ) -> Result<(Self, ForestNodeRc<B::GeneralElement>), Error>
    where
        Self: Sized;

    /// Indicate that the pending updates should be applied
    fn apply_updates<'b>(
        &'b mut self,
        owner: &'b mut ForestNodeMut<B::GeneralElement>,
    ) -> Result<(), Error>;

    /// Get the backend element
    fn backend_element_mut<'b>(
        &'b mut self,
        owner: &'b mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<ForestNodeMut<B::GeneralElement>, Error>;

    /// Get the backend element
    fn backend_element_rc<'b>(
        &'b mut self,
        owner: &'b mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<ForestNodeRc<B::GeneralElement>, Error>;
}
