//! The mount point utilities.

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

/// A mount point which contains a root component.
///
/// A mount point can be created through `BackendContext::attach` .
pub struct MountPoint<B: Backend, C: Component + ComponentTemplate<B> + 'static> {
    component_node: ComponentNode<C>,
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
            <ComponentNode<C> as BackendComponent<B>>::init(
                backend_context,
                parent,
                &owner_weak,
            )?;
        <ComponentNode<C> as BackendComponent<B>>::create(
            &mut component_node,
            backend_context,
            parent,
            Box::new(|comp, _| init(comp)),
            &mut |_, _, _| Ok(()),
        )?;
        let this = Self {
            component_node,
            backend_element,
        };
        <B::GeneralElement as BackendGeneralElement>::append(parent, &this.backend_element);
        Ok(this)
    }

    pub(crate) fn detach(&mut self, parent: &mut tree::ForestNodeMut<B::GeneralElement>) {
        let elem = parent.borrow_mut(&self.backend_element);
        <B::GeneralElement as BackendGeneralElement>::detach(elem);
    }

    /// Get the root component node.
    pub fn root_component(&self) -> &ComponentNode<C> {
        &self.component_node
    }

    /// Get the `dyn` form of the mount point
    ///
    /// This is useful for storing a mount point without its exact component type.
    pub fn into_dyn(self) -> DynMountPoint<B> {
        DynMountPoint {
            _component_node: Box::new(self.component_node),
            backend_element: self.backend_element,
        }
    }
}

/// The `dyn` form of the mount point.
/// 
/// This form does not contain the root component type.
pub struct DynMountPoint<B: Backend> {
    _component_node: Box<dyn std::any::Any>,
    backend_element: tree::ForestNodeRc<B::GeneralElement>,
}

impl<B: Backend> DynMountPoint<B> {
    pub(crate) fn detach(&mut self, parent: &mut tree::ForestNodeMut<B::GeneralElement>) {
        let elem = parent.borrow_mut(&self.backend_element);
        <B::GeneralElement as BackendGeneralElement>::detach(elem);
    }
}
