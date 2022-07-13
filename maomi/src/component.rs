use std::{cell::RefCell, rc::{Rc, Weak}};

use tree::ForestNodeRc;

use crate::{
    backend::{tree, Backend, SupportBackend, BackendGeneralElement},
    error::Error, BackendContext,
};

/// Some helper functions for the template type
pub trait TemplateHelper: Default {
    fn mark_dirty(&mut self); // TODO update queue when mark dirty
}

/// The template type
pub enum Template<S> {
    Uninitialized,
    Structure {
        dirty: bool,
        component: Weak<dyn std::any::Any>,
        child_nodes: S,
    },
}

impl<S> Default for Template<S> {
    fn default() -> Self {
        Self::Uninitialized
    }
}

impl<S> TemplateHelper for Template<S> {
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
        backend_element: &tree::ForestNodeRc<B::GeneralElement>,
        update_scheduler: Box<dyn UpdateScheduler>,
    ) -> Result<(), Error>
    where
        Self: Sized;

    /// Indicate that the pending updates should be applied
    fn apply_updates(
        &mut self,
        parent: &mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<(), Error>;
}

/// Represent a component that can update independently
///
/// Normally it is auto-implemented by `#[component]` .
pub trait UpdateScheduler: 'static {
    fn schedule_update(&mut self);
}

/// A node that wraps a component instance
pub struct ComponentNode<B: Backend, C: ComponentTemplate<B> + Component + 'static> {
    component: Rc<RefCell<C>>,
    backend_context: BackendContext<B>,
    backend_element: ForestNodeRc<B::GeneralElement>,
}

impl<B: Backend, C: ComponentTemplate<B> + Component> Clone for ComponentNode<B, C> {
    fn clone(&self) -> Self {
        Self {
            component: self.component.clone(),
            backend_context: self.backend_context.clone(),
            backend_element: self.backend_element.clone(),
        }
    }
}

impl<B: Backend, C: ComponentTemplate<B> + Component + 'static> ComponentNode<B, C> {
    /// Enter the component, getting a mutable reference of it
    ///
    /// If any component has already entered,
    /// it will wait until exits,
    /// so the `f` is required to be `'static` .
    #[inline]
    pub fn enter(&self, f: impl 'static + FnOnce(&mut C) -> Result<(), Error>) {
        let component = self.component.clone();
        self.backend_context.enter(move |_| {
            f(&mut component.borrow_mut())
        });
    }

    /// Try enter the component sync, getting a mutable reference of it
    ///
    /// If any component has already entered, an `Err` is returned.
    #[inline]
    pub fn enter_sync<T>(&self, f: impl FnOnce(&mut C) -> T) -> Result<T, Error> {
        match self.backend_context.enter_sync(|_| {
            f(&mut self.component.borrow_mut())
        }) {
            Ok(ret) => Ok(ret),
            Err(_) => Err(Error::AlreadyEntered),
        }
    }
}

impl<B: Backend, C: ComponentTemplate<B> + Component + 'static> UpdateScheduler for ComponentNode<B, C> {
    #[inline]
    fn schedule_update(&mut self) {
        let backend_element = self.backend_element.clone();
        self.enter(move |comp| {
            let mut backend_element = backend_element.borrow_mut();
            <C as ComponentTemplate<B>>::apply_updates(comp, &mut backend_element)
        });
    }
}

impl<B: Backend, C: ComponentTemplate<B> + Component + 'static> SupportBackend<B> for ComponentNode<B, C> {
    #[inline]
    fn create(
        backend_context: &BackendContext<B>,
        owner: &mut tree::ForestNodeMut<B::GeneralElement>,
        init: impl FnOnce(&mut Self) -> Result<(), Error>,
    ) -> Result<(Self, tree::ForestNodeRc<B::GeneralElement>), Error>
    where
        Self: Sized,
    {
        let mut backend_element = B::GeneralElement::create_virtual_element(owner)?;
        let this = ComponentNode {
            component: Rc::new(RefCell::new(<C as Component>::new())),
            backend_context: backend_context.clone(),
            backend_element: backend_element.clone(),
        };
        {
            let mut component = this.component.borrow_mut();
            // TODO init(&mut component)?;
            <C as ComponentTemplate<B>>::create(
                &mut component,
                owner,
                &mut backend_element,
                Box::new(this.clone()),
            )?;
            <C as Component>::created(&mut component);
            <C as ComponentTemplate<B>>::apply_updates(&mut component, owner)?;
        }
        Ok((this, backend_element))
    }

    #[inline]
    fn apply_updates(
        &mut self,
        owner: &mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<(), Error> {
        if let Ok(mut comp) = self.component.try_borrow_mut() {
            <C as ComponentTemplate<B>>::apply_updates(&mut comp, owner)
        } else {
            Err(Error::RecursiveUpdate)
        }
    }

    #[inline]
    fn backend_element_mut<'b>(
        &'b mut self,
        owner: &'b mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<tree::ForestNodeMut<B::GeneralElement>, Error> {
        Ok(owner.borrow_mut(&self.backend_element))
    }

    #[inline]
    fn backend_element_rc<'b>(
        &'b mut self,
        _owner: &'b mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<tree::ForestNodeRc<B::GeneralElement>, Error> {
        Ok(self.backend_element.clone())
    }
}
