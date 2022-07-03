use crate::{
    backend::{Backend, BackendComponent, BackendGeneralElement, SupportBackend},
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
    fn create(parent: &mut B::GeneralElement) -> Result<(Self, B::Component), Error>
    where
        Self: Sized;

    /// Indicate that the pending updates should be applied
    fn apply_updates(&mut self, backend_element: &mut B::Component) -> Result<(), Error>;
}

impl<B: Backend, T: Component<B>> SupportBackend<B> for T {
    fn create(
        parent: &mut B::GeneralElement,
    ) -> Result<(Self, <B as Backend>::GeneralElement), Error>
    where
        Self: Sized,
    {
        let (this, comp) = <Self as Component<B>>::create(parent)?;
        Ok((this, comp.into_general_element()))
    }

    fn apply_updates(
        &mut self,
        backend_element: &mut <B as Backend>::GeneralElement,
    ) -> Result<(), Error> {
        let comp: &mut B::Component = backend_element
            .as_component_mut()
            .ok_or(Error::TreeNotMatchedError)?;
        <Self as Component<B>>::apply_updates(self, comp)?;
        Ok(())
    }
}
