
use crate::BackendContext;
use crate::error::Error;
use crate::component::{Component, ComponentTemplate, ComponentNode};
use crate::backend::{Backend, SupportBackend, tree, BackendGeneralElement};

/// A mount point which can generate a "root" component and mounted to the whole page
/// 
/// A mount point can be created in a `BackendContext` .
pub struct MountPoint<
    B: Backend,
    C: Component + ComponentTemplate<B, C> + 'static,
> {
    component_node: ComponentNode<B, C>,
    backend_element: tree::ForestNodeRc<B::GeneralElement>,
}

impl<
    B: Backend,
    C: Component + ComponentTemplate<B, C>,
> MountPoint<B, C> {
    pub(crate) fn new_in_backend(
        backend_context: &BackendContext<B>,
        owner: &mut tree::ForestNodeMut<B::GeneralElement>,
        init: impl FnOnce(&mut C) -> Result<(), Error>,
    ) -> Result<Self, Error> {
        let mut component_node = <ComponentNode<B, C> as SupportBackend<B>>::init(backend_context, owner)?;
        let backend_element = component_node.backend_element_rc(owner);
        {
            let mut comp = component_node.component.borrow_mut();
            init(&mut comp)?;
        }
        <ComponentNode<B, C> as SupportBackend<B>>::create_or_update(&mut component_node, backend_context, owner, |_, _| Ok(()))?;
        Ok(Self {
            component_node,
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

    /// Get the root component
    pub fn root_component(&self) -> &ComponentNode<B, C> {
        &self.component_node
    }
}
