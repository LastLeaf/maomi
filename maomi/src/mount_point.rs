use crate::backend::{tree, Backend, BackendComponent, BackendGeneralElement};
use crate::component::{Component, ComponentNode};
use crate::error::Error;
use crate::node::OwnerWeak;
use crate::template::ComponentTemplate;
use crate::BackendContext;

struct DanglingOwner();

impl OwnerWeak for DanglingOwner {
    fn apply_updates(&self) -> Result<(), Error> {
        Ok(())
    }

    fn clone_owner_weak(&self) -> Box<dyn OwnerWeak> {
        Box::new(DanglingOwner())
    }
}

/// A mount point which can generate a "root" component and mounted to the whole page
///
/// A mount point can be created in a `BackendContext` .
pub struct MountPoint<B: Backend, C: Component + ComponentTemplate<B> + 'static> {
    component_node: ComponentNode<B, C>,
    backend_element: tree::ForestNodeRc<B::GeneralElement>,
}

impl<B: Backend, C: Component + ComponentTemplate<B>> MountPoint<B, C> {
    pub(crate) fn attach(
        backend_context: &BackendContext<B>,
        parent: &mut tree::ForestNodeMut<B::GeneralElement>,
        init: impl FnOnce(&mut C),
    ) -> Result<Self, Error> {
        let owner_weak: Box<dyn OwnerWeak> = Box::new(DanglingOwner());
        let (mut component_node, backend_element) =
            <ComponentNode<B, C> as BackendComponent<B>>::init(
                backend_context,
                parent,
                &owner_weak,
            )?;
        <ComponentNode<B, C> as BackendComponent<B>>::create(
            &mut component_node,
            backend_context,
            parent,
            |comp, _| init(comp),
            |_, _, _| Ok(()),
        )?;
        let this = Self {
            component_node,
            backend_element,
        };
        <B::GeneralElement as BackendGeneralElement>::append(parent, &this.backend_element);
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
