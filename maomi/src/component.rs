use crate::{
    backend::{tree, Backend, BackendGeneralElement, SupportBackend},
    error::Error,
};

/// A helper type to represent a node with child nodes
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Node<N, C> {
    pub node: N,
    pub children: C,
}

/// The template type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Template<S> {
    Uninitialized,
    Structure {
        child_nodes: S,
        backend_element: (), // TODO
    },
}

impl<S> Default for Template<S> {
    fn default() -> Self {
        Self::Uninitialized
    }
}

/// A component
///
/// This trait must be implemented by components.
pub trait Component<B: Backend>: ComponentTemplate<B> {
    /// Called when a new instance requested
    fn new() -> Self;

    /// Called after fully created
    fn created(&mut self);

    /// Called after attached to a root component
    fn attached(&mut self); // TODO manage attach status

    /// Called after detached from a root component
    fn detached(&mut self);

    /// Apply updates to templates
    fn update(&mut self) -> Result<(), Error> {
        <Self as ComponentTemplate>::apply_updates(self)
    }
}

/// A component template
/// 
/// Normally it is auto-implemented by `#[component]` .
pub trait ComponentTemplate<B: Backend> {
    /// Create a component within the specified shadow root
    fn create(ctx: &mut tree::ForestCtx<B::GeneralElement>) -> Result<Self, Error>
    where
        Self: Sized;

    /// Indicate that the pending updates should be applied
    fn apply_updates(
        &mut self,
        ctx: &mut tree::ForestCtx<B::GeneralElement>,
    ) -> Result<(), Error>;
}

/// Indicate `T` is renderable in backend `B`
impl<B: Backend, T: ComponentTemplate<B>> SupportBackend<B> for T {
    /// Called when a new node requested
    fn create(
        ctx: &mut tree::ForestCtx<B::GeneralElement>,
        init: FnOnce(&mut T),
    ) -> Result<(Self, tree::ForestTree<B::GeneralElement>), Error>
    where
        Self: Sized, 
    {
        let mut backend_element = B::GeneralElement::create_virtual_element(parent)?;
        let this = <Self as ComponentTemplate<B>>::create(&mut backend_element.as_node_mut())?;
        Ok((this, backend_element))
    }

    /// Called when updates needed to be applied
    fn apply_updates(
        &mut self,
        ctx: &mut tree::ForestCtx<B::GeneralElement>,
    ) -> Result<(), Error> {
        <Self as ComponentTemplate<B>>::apply_updates(self)?;
        Ok(())
    }
}
