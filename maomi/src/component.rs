use crate::{
    backend::{tree, Backend, BackendGeneralElement, SupportBackend},
    error::Error,
};

/// A helper type to represent a node with child nodes
pub struct Node<N, C> {
    pub node: N,
    pub children: C,
}

/// A component
pub trait ComponentTemplate<B: Backend> {
    /// Create a component within the specified shadow root
    fn create(backend_element: &mut tree::ForestNodeMut<B::GeneralElement>) -> Result<Self, Error>
    where
        Self: Sized;

    /// Indicate that the pending updates should be applied
    fn apply_updates(
        &mut self,
        backend_element: &mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<(), Error>;
}

impl<B: Backend, T: ComponentTemplate<B>> SupportBackend<B> for T {
    fn create(
        parent: &mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<(Self, tree::ForestTree<B::GeneralElement>), Error>
    where
        Self: Sized,
    {
        let mut backend_element = B::GeneralElement::create_virtual_element(parent)?;
        let this = <Self as ComponentTemplate<B>>::create(&mut backend_element.as_node_mut())?;
        Ok((this, backend_element))
    }

    fn apply_updates(
        &mut self,
        mut backend_element: &mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<(), Error> {
        <Self as ComponentTemplate<B>>::apply_updates(self, &mut backend_element)?;
        Ok(())
    }
}
