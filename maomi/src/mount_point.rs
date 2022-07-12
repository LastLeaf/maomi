
use crate::error::Error;
use crate::component::{Component, ComponentTemplate};
use crate::backend::{Backend, SupportBackend, tree, BackendGeneralElement};

/// A mount point which can generate a "root" component and mounted to the whole page
/// 
/// A mount point can be created in a `BackendContext` .
pub struct MountPoint<
    B: Backend,
    C: Component + SupportBackend<B>,
> {
    component: C,
    backend_element: tree::ForestNodeRc<B::GeneralElement>,
}

impl<
    B: Backend,
    C: Component + ComponentTemplate<B>,
> MountPoint<B, C> {
    pub(crate) fn new_in_backend(
        owner: &mut tree::ForestNodeMut<B::GeneralElement>,
        init: impl FnOnce(&mut C) -> Result<(), Error>,
    ) -> Result<Self, Error> {
        let mut component = C::new();
        init(&mut component)?;
        let backend_element = <C as ComponentTemplate<B>>::create(&mut component, owner)?;
        <C as Component>::created(&mut component);
        <C as ComponentTemplate<B>>::apply_updates(&mut component, owner)?;
        Ok(Self {
            component,
            backend_element,
        })
    }

    /// Attach to a parent as the last child of it
    pub fn append_attach(
        &mut self,
        parent: &mut tree::ForestNodeMut<B::GeneralElement>,
    ) {
        <B::GeneralElement as BackendGeneralElement>::append(parent, self.backend_element.clone())
    }

    /// Detach the mount point
    pub fn detach(
        &mut self,
        owner: &mut tree::ForestNodeMut<B::GeneralElement>,
    ) {
        let elem = owner.borrow_mut(&self.backend_element);
        <B::GeneralElement as BackendGeneralElement>::detach(elem);
    }

    /// Get the underlying root component
    pub fn root_component(&self) -> &C {
        &self.component
    }

    /// Get the underlying root component
    pub fn root_component_mut(&mut self) -> &mut C {
        &mut self.component
    }
}
