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
pub trait Component<B: Backend> {
    /// Create a component within the specified shadow root
    fn create(backend_element: &mut B::Component) -> Result<Self, Error>
    where
        Self: Sized;

    /// Indicate that the pending updates should be applied
    fn apply_updates(&mut self, backend_element: &mut B::Component) -> Result<(), Error>;
}

impl<B: Backend, T: Component<B>> SupportBackend<B> for T {
    fn create(
        parent: &mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<(Self, tree::ForestTree<B::GeneralElement>), Error>
    where
        Self: Sized,
    {
        let mut this = None;
        let elem = B::GeneralElement::create_component(parent, |comp| {
            this = Some(<Self as Component<B>>::create(comp)?);
            Ok(())
        })?;
        Ok((this.unwrap(), elem))
    }

    fn apply_updates(
        &mut self,
        backend_element: &mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<(), Error> {
        let mut comp = B::GeneralElement::as_component_mut(backend_element)
            .ok_or(Error::TreeNotMatchedError)?;
        <Self as Component<B>>::apply_updates(self, &mut comp)?;
        Ok(())
    }
}
