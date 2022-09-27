//! The backend protocol that should be implemented by backends

pub use maomi_tree as tree;
use tree::*;

use crate::{
    error::Error,
    node::{OwnerWeak, SlotChange},
};
pub mod context;
use context::BackendContext;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackendStage {
    Normal,
    #[cfg(feature = "prerendering")]
    Prerendering,
    #[cfg(feature = "prerendering-apply")]
    PrerenderingApply,
}

/// A backend
pub trait Backend: 'static {
    type GeneralElement: BackendGeneralElement<BaseBackend = Self>;
    type VirtualElement: BackendVirtualElement<BaseBackend = Self>;
    type TextNode: BackendTextNode<BaseBackend = Self>;

    /// Whether the backend is in prerendering stage
    fn backend_stage(&self) -> BackendStage;

    /// Get the root element
    fn root(&self) -> ForestNode<Self::GeneralElement>;

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
    type BaseBackend: Backend<GeneralElement = Self>;

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
        child: &'b ForestNodeRc<
            <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    ) where
        Self: Sized;

    /// Insert an element before this element
    fn insert<'b>(
        this: &'b mut ForestNodeMut<Self>,
        target: &'b ForestNodeRc<
            <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    ) where
        Self: Sized;

    /// Detach this element temporarily
    fn temp_detach(
        this: ForestNodeMut<Self>,
    ) -> ForestNodeRc<<<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement>
    where
        Self: Sized;

    /// Remove this element
    fn detach(
        this: ForestNodeMut<Self>,
    ) -> ForestNodeRc<<<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement>
    where
        Self: Sized;

    /// Replace an element before this element
    fn replace_with(
        mut this: ForestNodeMut<Self>,
        replacer: ForestNodeRc<
            <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    ) -> ForestNodeRc<<<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement>
    where
        Self: Sized,
    {
        Self::insert(&mut this, &replacer);
        Self::detach(this)
    }
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

// FIXME consider using `dyn` to control the generated bin size

/// A trait that indicates a component or a backend-implemented element for the backend
pub trait BackendComponent<B: Backend> {
    type SlotData;
    type UpdateTarget;
    type UpdateContext;

    /// Create with a backend element
    fn init<'b>(
        backend_context: &'b BackendContext<B>,
        owner: &'b mut ForestNodeMut<B::GeneralElement>,
        owner_weak: &Box<dyn OwnerWeak>,
    ) -> Result<(Self, ForestNodeRc<B::GeneralElement>), Error>
    where
        Self: Sized;

    /// Indicate that the create process should be finished
    fn create<'b>(
        &'b mut self,
        backend_context: &'b BackendContext<B>,
        owner: &'b mut ForestNodeMut<B::GeneralElement>,
        update_fn: impl FnOnce(&mut Self::UpdateTarget, &mut Self::UpdateContext),
        slot_fn: impl FnMut(
            &mut tree::ForestNodeMut<B::GeneralElement>,
            &ForestToken,
            &Self::SlotData,
        ) -> Result<(), Error>,
    ) -> Result<(), Error>;

    /// Indicate that the pending updates should be applied
    fn apply_updates<'b>(
        &'b mut self,
        backend_context: &'b BackendContext<B>,
        owner: &'b mut ForestNodeMut<B::GeneralElement>,
        update_fn: impl FnOnce(&mut Self::UpdateTarget, &mut Self::UpdateContext),
        slot_fn: impl FnMut(
            SlotChange<&mut tree::ForestNodeMut<B::GeneralElement>, &ForestToken, &Self::SlotData>,
        ) -> Result<(), Error>,
    ) -> Result<(), Error>;
}

/// A trait that indicates a component that can be converted into a `SupportBackend`
pub trait SupportBackend<B: Backend> {
    type Target: BackendComponent<B>;
}
