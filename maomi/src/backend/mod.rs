//! The backend-related interface.
//! 
//! This module contains some basic types that a backend implementor will use.

pub use maomi_tree as tree;
use tree::*;

use crate::{
    error::Error,
    node::{OwnerWeak, SlotChange, SlotKindTrait, DynNodeList},
};
pub mod context;
use context::BackendContext;

/// The backend stage.
/// 
/// This is meaningful only when prerendering is used.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackendStage {
    /// The normal backend stage.
    Normal,
    /// The backend is in prerendering stage.
    #[cfg(feature = "prerendering")]
    Prerendering,
    /// The backend is applying prerendering result.
    #[cfg(feature = "prerendering-apply")]
    PrerenderingApply,
}

/// The interface that a backend should implement.
/// 
/// This is used by the backend implementor.
/// *In most cases, it should not be used in component implementors.*
pub trait Backend: 'static {
    /// The general type for a backend element.
    type GeneralElement: BackendGeneralElement<BaseBackend = Self>;

    /// The type for a virtual element.
    type VirtualElement: BackendVirtualElement<BaseBackend = Self>;

    /// The type for a text node.
    type TextNode: BackendTextNode<BaseBackend = Self>;

    /// Generate an async task.
    fn async_task(fut: impl 'static + std::future::Future<Output = ()>) where Self: Sized;

    /// Whether the backend is in prerendering stage.
    fn backend_stage(&self) -> BackendStage;

    /// Get the root element.
    fn root(&self) -> ForestNode<Self::GeneralElement>;

    /// Get the root element.
    fn root_mut(&mut self) -> ForestNodeMut<Self::GeneralElement>;
}

/// The general type of the elements.
/// 
/// This is used by the backend implementor.
/// *In most cases, it should not be used in component implementors.*
///
/// The backend can contain several types of elements.
/// * A `VirtualElement` is an element which should not layout in backend.
/// * A `TextNode` is a text node.
/// * The backend can define other types of elements.
pub trait BackendGeneralElement: 'static {
    /// The related backend type.
    type BaseBackend: Backend<GeneralElement = Self>;

    /// Cast to a virtual element.
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

    /// Cast to a text node.
    fn as_text_node_mut<'b>(
        this: &'b mut ForestNodeMut<Self>,
    ) -> Option<
        ForestValueMut<'b, <<Self as BackendGeneralElement>::BaseBackend as Backend>::TextNode>,
    >
    where
        Self: Sized;

    /// Create a virtual element.
    fn create_virtual_element<'b>(
        this: &'b mut ForestNodeMut<Self>,
    ) -> Result<ForestNodeRc<<Self::BaseBackend as Backend>::GeneralElement>, Error>
    where
        Self: Sized;

    /// Create a text node.
    fn create_text_node(
        this: &mut ForestNodeMut<Self>,
        content: &str,
    ) -> Result<ForestNodeRc<<Self::BaseBackend as Backend>::GeneralElement>, Error>
    where
        Self: Sized;

    /// Append a child element.
    fn append<'b>(
        this: &'b mut ForestNodeMut<Self>,
        child: &'b ForestNodeRc<
            <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    ) where
        Self: Sized;

    /// Insert an element before this element.
    fn insert<'b>(
        this: &'b mut ForestNodeMut<Self>,
        target: &'b ForestNodeRc<
            <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    ) where
        Self: Sized;

    /// Detach this element temporarily.
    fn temp_detach(
        this: ForestNodeMut<Self>,
    ) -> ForestNodeRc<<<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement>
    where
        Self: Sized;

    /// Remove this element.
    fn detach(
        this: ForestNodeMut<Self>,
    ) -> ForestNodeRc<<<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement>
    where
        Self: Sized;

    /// Replace an element.
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

/// The virtual element in the backend.
/// 
/// This is used by the backend implementor.
/// *In most cases, it should not be used in component implementors.*
pub trait BackendVirtualElement {
    /// The related backend type.
    type BaseBackend: Backend;
}

/// The text node in the backend.
/// 
/// This is used by the backend implementor.
/// *In most cases, it should not be used in component implementors.*
pub trait BackendTextNode {
    /// The related backend type.
    type BaseBackend: Backend;

    /// Set the text content.
    fn set_text(&mut self, content: &str);
}

/// A trait that indicates a component or a backend-implemented element for the backend.
/// 
/// This is used by the backend implementor.
/// *In most cases, it should not be used in component implementors.*
pub trait BackendComponent<B: Backend> {
    /// The slot data type.
    type SlotData;
    /// The type of the updated comopnent or element.
    type UpdateTarget;
    /// The update-related data of the component or element.
    /// 
    /// Should be `bool` for components.
    type UpdateContext;

    /// Create with a backend element.
    fn init<'b>(
        backend_context: &'b BackendContext<B>,
        owner: &'b mut ForestNodeMut<B::GeneralElement>,
        owner_weak: &Box<dyn OwnerWeak>,
    ) -> Result<(Self, ForestNodeRc<B::GeneralElement>), Error>
    where
        Self: Sized;

    /// Indicate that the create process should be finished.
    fn create<'b>(
        &'b mut self,
        backend_context: &'b BackendContext<B>,
        owner: &'b mut ForestNodeMut<B::GeneralElement>,
        update_fn: Box<dyn 'b + FnOnce(&mut Self::UpdateTarget, &mut Self::UpdateContext)>,
        slot_fn: &mut dyn FnMut(
            &mut tree::ForestNodeMut<B::GeneralElement>,
            &ForestToken,
            &Self::SlotData,
        ) -> Result<(), Error>,
    ) -> Result<(), Error>;

    /// Indicate that the pending updates should be applied.
    fn apply_updates<'b>(
        &'b mut self,
        backend_context: &'b BackendContext<B>,
        owner: &'b mut ForestNodeMut<B::GeneralElement>,
        update_fn: Box<dyn 'b + FnOnce(&mut Self::UpdateTarget, &mut Self::UpdateContext)>,
        slot_fn: &mut dyn FnMut(
            SlotChange<&mut tree::ForestNodeMut<B::GeneralElement>, &ForestToken, &Self::SlotData>,
        ) -> Result<(), Error>,
    ) -> Result<(), Error>;
}

/// A trait that indicates a component that can be converted into a `BackendComponent` .
/// 
/// This is used by the backend implementor.
/// *In most cases, it should not be used in component implementors.*
pub trait SupportBackend {
    /// The converted `BackendComponent` type.
    type Target: 'static;
    /// The slot list type.
    type SlotChildren: SlotKindTrait<ForestTokenAddr, DynNodeList>;
}
