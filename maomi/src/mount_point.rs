use crate::backend::{tree, Backend, BackendGeneralElement, BackendComponent};
use crate::component::{Component, ComponentNode};
use crate::template::ComponentTemplate;
use crate::error::Error;
use crate::BackendContext;

/// A mount point which can generate a "root" component and mounted to the whole page
///
/// A mount point can be created in a `BackendContext` .
pub struct MountPoint<B: Backend, C: Component + ComponentTemplate<B> + 'static> {
    component_node: ComponentNode<B, C>,
    backend_element: tree::ForestNodeRc<B::GeneralElement>,
}

impl<B: Backend, C: Component + ComponentTemplate<B>> MountPoint<B, C> {
    fn new_in_backend(
        backend_context: &BackendContext<B>,
        owner: &mut tree::ForestNodeMut<B::GeneralElement>,
        init: impl FnOnce(&mut C),
    ) -> Result<Self, Error> {
        let (mut component_node, backend_element) =
            <ComponentNode<B, C> as BackendComponent<B>>::init(backend_context, owner)?;
        <ComponentNode<B, C> as BackendComponent<B>>::create(
            &mut component_node,
            backend_context,
            owner,
            |comp, _| {
                init(comp)
            },
            |_, _| Ok(()),
        )?;
        Ok(Self {
            component_node,
            backend_element,
        })
    }

    pub(crate) fn append_attach(
        backend_context: &BackendContext<B>,
        parent: &mut tree::ForestNodeMut<B::GeneralElement>,
        init: impl FnOnce(&mut C),
    ) -> Result<Self, Error> {
        let this = Self::new_in_backend(backend_context, parent, init)?;
        <B::GeneralElement as BackendGeneralElement>::append(parent, this.backend_element.clone());
        Ok(this)
    }

    /// Detach the mount point
    pub fn detach(&mut self, owner: &mut tree::ForestNodeMut<B::GeneralElement>) {
        let elem = owner.borrow_mut(&self.backend_element);
        <B::GeneralElement as BackendGeneralElement>::detach(elem);
    }

    /// Get the root component
    pub fn root_component(&self) -> &ComponentNode<B, C> {
        &self.component_node
    }
}
