use crate::{
    backend::{tree, Backend, SupportBackend, BackendGeneralElement},
    error::Error,
};

/// A helper type to represent a node with child nodes
#[derive(Debug, Clone, PartialEq)]
pub struct Node<N, C> {
    pub node: N,
    pub child_nodes: C,
}

/// Some helper functions for the template type
pub trait TemplateHelper: Default {
    fn backend_element_mut<B: BackendGeneralElement>(&mut self) -> Result<&tree::ForestNodeRc<B>, Error>;
    fn backend_element_token(&self) -> Result<&tree::ForestToken, Error>;
    fn mark_dirty(&mut self); // TODO update queue when mark dirty
}

/// The template type
pub enum Template<S> {
    Uninitialized,
    Structure {
        dirty: bool,
        backend_element: Box<dyn std::any::Any>,
        backend_element_token: tree::ForestToken,
        child_nodes: S,
    },
}

impl<S> Default for Template<S> {
    fn default() -> Self {
        Self::Uninitialized
    }
}

impl<S> TemplateHelper for Template<S> {
    fn backend_element_mut<B: BackendGeneralElement>(&mut self) -> Result<&tree::ForestNodeRc<B>, Error> {
        match self {
            Self::Uninitialized => Err(Error::TreeNotCreated),
            Self::Structure { backend_element, .. } => {
                backend_element.downcast_ref::<tree::ForestNodeRc<B>>().ok_or(Error::TreeNodeTypeWrong)
            }
        }
    }

    fn backend_element_token(&self) -> Result<&tree::ForestToken, Error> {
        match self {
            Self::Uninitialized => Err(Error::TreeNotCreated),
            Self::Structure { backend_element_token, .. } => {
                Ok(backend_element_token)
            }
        }
    }

    fn mark_dirty(&mut self) {
        match self {
            Self::Uninitialized => {}
            Self::Structure { dirty, .. } => {
                *dirty = true;
            }
        }
    }
}

/// A component
///
/// This trait must be implemented by components.
/// It contains some lifetime callbacks.
pub trait Component {
    /// Called when a new instance requested
    fn new() -> Self;

    /// Called after fully created
    fn created(&mut self) {}

    /// Called after attached to a root component
    fn attached(&mut self) {} // TODO manage attach status

    /// Called after detached from a root component
    fn detached(&mut self) {}
}

/// Some component helper functions
///
/// This trait is auto-implemented by `#[component]` .
pub trait ComponentExt<B: Backend> {
    type TemplateField;

    /// Get a reference of the template field of the component
    fn template(&self) -> &Self::TemplateField;

    /// Get a mutable reference of the template field of the component
    fn template_mut(&mut self) -> &mut Self::TemplateField;
}

impl<B: Backend, T: ComponentTemplate<B>> ComponentExt<B> for T {
    type TemplateField = T::TemplateField;

    fn template(&self) -> &Self::TemplateField {
        <Self as ComponentTemplate<B>>::template(self)
    }

    fn template_mut(&mut self) -> &mut Self::TemplateField {
        <Self as ComponentTemplate<B>>::template_mut(self)
    }
}

pub(crate) trait StaticComponent<B: Backend> {
    fn apply_template_updates(&mut self) -> Result<(), Error>;
}

impl<B: Backend, T: ComponentTemplate<B> + 'static> StaticComponent<B> for T {
    fn apply_template_updates(&mut self) -> Result<(), Error> {
        let backend_element = <Self as ComponentTemplate<B>>::template_mut(self).backend_element_mut()?.clone();
        let mut backend_element = backend_element.borrow_mut();
        ComponentTemplate::<B>::apply_updates(self, &mut backend_element)?;
        Ok(())
    }
}

/// A component template
///
/// Normally it is auto-implemented by `#[component]` .
pub trait ComponentTemplate<B: Backend> {
    type TemplateField: TemplateHelper;

    /// Get a reference of the template field of the component
    fn template(&self) -> &Self::TemplateField;

    /// Get a mutable reference of the template field of the component
    fn template_mut(&mut self) -> &mut Self::TemplateField;

    /// Create a component within the specified shadow root
    fn create(
        &mut self,
        parent: &mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<tree::ForestNodeRc<B::GeneralElement>, Error>
    where
        Self: Sized;

    /// Indicate that the pending updates should be applied
    fn apply_updates(
        &mut self,
        parent: &mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<(), Error>;
}

impl<B: Backend, T: ComponentTemplate<B> + Component> SupportBackend<B> for T {
    fn create(
        owner: &mut tree::ForestNodeMut<B::GeneralElement>,
        init: impl FnOnce(&mut Self) -> Result<(), Error>,
    ) -> Result<(Self, tree::ForestNodeRc<B::GeneralElement>), Error>
    where
        Self: Sized,
    {
        let mut component = <Self as Component>::new();
        init(&mut component)?;
        let backend_element = <Self as ComponentTemplate<B>>::create(&mut component, owner)?;
        <Self as Component>::created(&mut component);
        <Self as ComponentTemplate<B>>::apply_updates(&mut component, owner)?;
        Ok((component, backend_element))
    }

    fn apply_updates(
        &mut self,
        owner: &mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<(), Error> {
        <Self as ComponentTemplate<B>>::apply_updates(self, owner)
    }

    fn backend_element_mut<'b>(
        &'b mut self,
        owner: &'b mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<tree::ForestNodeMut<B::GeneralElement>, Error> {
        let token = <Self as ComponentTemplate<B>>::template(self).backend_element_token()?;
        Ok(owner.borrow_mut_token(&token))
    }

    fn backend_element_rc<'b>(
        &'b mut self,
        owner: &'b mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<tree::ForestNodeRc<B::GeneralElement>, Error> {
        let token = <Self as ComponentTemplate<B>>::template(self).backend_element_token()?;
        Ok(owner.resolve_token(&token))
    }
}
