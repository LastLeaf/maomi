use std::{
    cell::RefCell,
    marker::PhantomData,
    rc::{Rc, Weak},
};

use crate::{
    backend::{tree, Backend, BackendGeneralElement, SupportBackend},
    diff::ListItemChange,
    error::Error,
    node::SlotChildren,
    BackendContext,
};
use tree::ForestNodeRc;

/// An init object for the template
///
/// This struct is auto-managed by `#[component]` .
pub struct TemplateInit<C: ?Sized> {
    updater: Box<dyn UpdateScheduler<EnterType = C>>,
}

/// Some helper functions for the template type
pub trait TemplateHelper<C: ?Sized, D>: Default {
    fn mark_dirty(&mut self)
    where
        C: 'static;
    fn clear_dirty(&mut self) -> bool
    where
        C: 'static;
    fn is_initialized(&self) -> bool;
    fn component_rc(&self) -> Result<ComponentRc<C>, Error>
    where
        C: 'static + Sized;
    fn slot_scopes(&self) -> &SlotChildren<(tree::ForestToken, D)>;
}

/// The template type
///
/// This struct is auto-managed by `#[component]` .
pub struct Template<C, S, D> {
    updater: Option<Box<dyn UpdateScheduler<EnterType = C>>>,
    dirty: bool,
    /// The template node tree structure
    ///
    /// Caution: do not modify anything inside node tree unless you really understand how templates works.
    pub structure: Option<S>,
    /// The slot scope data
    /// 
    /// Caution: do not modify anything inside node tree unless you really understand how templates works.
    pub slot_scopes: SlotChildren<(tree::ForestToken, D)>,
}

impl<C, S, D> Default for Template<C, S, D> {
    fn default() -> Self {
        Self {
            updater: None,
            dirty: false,
            structure: None,
            slot_scopes: SlotChildren::None,
        }
    }
}

impl<C, S, D> Template<C, S, D> {
    #[inline]
    pub fn init(&mut self, init: TemplateInit<C>) {
        self.updater = Some(init.updater);
    }
}

impl<C, S, D> TemplateHelper<C, D> for Template<C, S, D> {
    #[inline]
    fn mark_dirty(&mut self)
    where
        C: 'static,
    {
        if self.structure.is_some() && !self.dirty {
            if let Some(updater) = self.updater.as_ref() {
                self.dirty = true;
                updater.schedule_update();
            }
        }
    }

    #[inline]
    fn clear_dirty(&mut self) -> bool
    where
        C: 'static,
    {
        let dirty = self.dirty;
        self.dirty = false;
        dirty
    }

    #[inline]
    fn is_initialized(&self) -> bool {
        self.structure.is_some()
    }

    #[inline]
    fn component_rc(&self) -> Result<ComponentRc<C>, Error>
    where
        C: 'static,
    {
        self.updater
            .as_ref()
            .and_then(|x| x.upgrade_scheduler())
            .map(|x| ComponentRc {
                inner: x,
                _phantom: PhantomData,
            })
            .ok_or(Error::TreeNotCreated)
    }

    fn slot_scopes(&self) -> &SlotChildren<(tree::ForestToken, D)> {
        &self.slot_scopes
    }
}

/// A ref-counted token of a component
pub struct ComponentRc<C: 'static> {
    inner: Box<dyn UpdateScheduler<EnterType = C>>,
    _phantom: PhantomData<C>,
}

impl<C: 'static> Clone for ComponentRc<C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.upgrade_scheduler().unwrap(),
            _phantom: PhantomData,
        }
    }
}

impl<C: 'static> ComponentRc<C> {
    /// Get the component mutable reference to update it
    pub fn update(&self, f: impl 'static + FnOnce(&mut C) -> Result<(), Error>) {
        self.inner.enter(Box::new(f));
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
}

/// Some component helper functions
///
/// This trait is auto-implemented by `#[component]` .
pub trait ComponentExt<B: Backend, C> {
    type TemplateField;

    /// Get a reference of the template field of the component
    fn template(&self) -> &Self::TemplateField;

    /// Manually trigger an update for the template
    fn mark_dirty(&mut self)
    where
        C: 'static,
        Self: 'static;

    /// Get a `ComponentRc` for the component
    fn component_rc(&self) -> Result<ComponentRc<C>, Error>
    where
        C: 'static,
        Self: 'static;
}

impl<B: Backend, T: ComponentTemplate<B>> ComponentExt<B, Self> for T {
    type TemplateField = T::TemplateField;

    fn template(&self) -> &Self::TemplateField {
        <Self as ComponentTemplate<B>>::template(self)
    }

    fn mark_dirty(&mut self)
    where
        T: 'static,
    {
        <Self as ComponentTemplate<B>>::template_mut(self).mark_dirty();
    }

    fn component_rc(&self) -> Result<ComponentRc<Self>, Error>
    where
        T: 'static,
    {
        <Self as ComponentTemplate<B>>::template(self).component_rc()
    }
}

/// A component template
///
/// It is auto-implemented by `#[component]` .
pub trait ComponentTemplate<B: Backend> {
    type TemplateField: TemplateHelper<Self, Self::SlotData>;
    type SlotData;

    /// Get a reference of the template field of the component
    fn template(&self) -> &Self::TemplateField;

    /// Get a mutable reference of the template field of the component
    fn template_mut(&mut self) -> &mut Self::TemplateField;

    /// Init a template
    fn template_init(&mut self, init: TemplateInit<Self>);

    /// Create a component within the specified shadow root
    fn template_create<'b, R>(
        &'b mut self,
        backend_context: &'b BackendContext<B>,
        backend_element: &'b mut tree::ForestNodeMut<B::GeneralElement>,
        slot_fn: impl FnMut(
            &mut tree::ForestNodeMut<B::GeneralElement>,
            &Self::SlotData,
        ) -> Result<R, Error>,
    ) -> Result<SlotChildren<R>, Error>
    where
        Self: Sized;

    /// Create a component within the specified shadow root
    fn template_update<'b>(
        &'b mut self,
        backend_context: &'b BackendContext<B>,
        backend_element: &'b mut tree::ForestNodeMut<B::GeneralElement>,
        slot_fn: impl FnMut(
            ListItemChange<&mut tree::ForestNodeMut<B::GeneralElement>, &Self::SlotData>,
        ) -> Result<(), Error>,
    ) -> Result<(), Error>
    where
        Self: Sized;

    /// Iterate slots
    fn for_each_slot_scope<'b>(
        &'b mut self,
        backend_element: &'b mut tree::ForestNodeMut<B::GeneralElement>,
        mut slot_fn: impl FnMut(
            ListItemChange<&mut tree::ForestNodeMut<B::GeneralElement>, &Self::SlotData>,
        ) -> Result<(), Error>,
    ) -> Result<(), Error> {
        for (t, d) in self.template_mut().slot_scopes() {
            slot_fn(ListItemChange::Unchanged(&mut backend_element.borrow_mut_token(t), d))?;
        }
        Ok(())
    }
}

/// Represent a component that can update independently
///
/// Normally it is auto-implemented by `#[component]` .
trait UpdateScheduler: 'static {
    type EnterType;
    fn schedule_update(&self);
    fn upgrade_scheduler(&self) -> Option<Box<dyn UpdateScheduler<EnterType = Self::EnterType>>>;
    fn enter(&self, f: Box<dyn FnOnce(&mut Self::EnterType) -> Result<(), Error>>);
}

/// A node that wraps a component instance
pub struct ComponentNode<B: Backend, C: ComponentTemplate<B> + Component + 'static> {
    pub(crate) component: Rc<RefCell<C>>,
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
    /// Get a weak reference
    #[inline]
    fn weak_ref(&self) -> ComponentNodeWeak<B, C> {
        ComponentNodeWeak {
            component: Rc::downgrade(&self.component),
            backend_context: self.backend_context.clone(),
            backend_element: self.backend_element.clone(),
        }
    }
}

impl<B: Backend, C: ComponentTemplate<B> + Component + 'static> UpdateScheduler
    for ComponentNode<B, C>
{
    type EnterType = C;

    #[inline]
    fn schedule_update(&self) {
        let backend_element = self.backend_element.clone();
        let backend_context = self.backend_context.clone();
        let component = self.component.clone();
        self.backend_context.enter(move |_| {
            let mut backend_element = backend_element.borrow_mut();
            let mut comp = component.borrow_mut();
            <C as ComponentTemplate<B>>::template_update(
                &mut comp,
                &backend_context,
                &mut backend_element,
                |_| {
                    // TODO notify slot changes to the owner component
                    Ok(())
                },
            )?;
            Ok(())
        });
    }

    #[inline]
    fn upgrade_scheduler(&self) -> Option<Box<dyn UpdateScheduler<EnterType = Self::EnterType>>> {
        Some(Box::new(self.clone()))
    }

    #[inline]
    fn enter(&self, f: Box<dyn FnOnce(&mut Self::EnterType) -> Result<(), Error>>) {
        let component = self.component.clone();
        self.backend_context
            .enter(move |_| f(&mut component.borrow_mut()));
    }
}

/// A node that wraps a component instance
struct ComponentNodeWeak<B: Backend, C: ComponentTemplate<B> + Component + 'static> {
    component: Weak<RefCell<C>>,
    backend_context: BackendContext<B>,
    backend_element: ForestNodeRc<B::GeneralElement>,
}

impl<B: Backend, C: ComponentTemplate<B> + Component + 'static> ComponentNodeWeak<B, C> {
    fn upgrade(&self) -> Option<ComponentNode<B, C>> {
        if let Some(component) = self.component.upgrade() {
            Some(ComponentNode {
                component,
                backend_context: self.backend_context.clone(),
                backend_element: self.backend_element.clone(),
            })
        } else {
            None
        }
    }
}

impl<B: Backend, C: ComponentTemplate<B> + Component + 'static> UpdateScheduler
    for ComponentNodeWeak<B, C>
{
    type EnterType = C;

    #[inline]
    fn schedule_update(&self) {
        if let Some(this) = self.upgrade() {
            this.schedule_update()
        }
    }

    #[inline]
    fn upgrade_scheduler(&self) -> Option<Box<dyn UpdateScheduler<EnterType = Self::EnterType>>> {
        if let Some(this) = self.upgrade() {
            Some(Box::new(this))
        } else {
            None
        }
    }

    #[inline]
    fn enter(&self, f: Box<dyn FnOnce(&mut Self::EnterType) -> Result<(), Error>>) {
        if let Some(this) = self.upgrade() {
            this.enter(f)
        }
    }
}

impl<B: Backend, C: ComponentTemplate<B> + Component + 'static> SupportBackend<B>
    for ComponentNode<B, C>
{
    type SlotData = <C as ComponentTemplate<B>>::SlotData;

    #[inline]
    fn init<'b>(
        backend_context: &'b BackendContext<B>,
        owner: &'b mut tree::ForestNodeMut<B::GeneralElement>,
    ) -> Result<(Self, tree::ForestNodeRc<B::GeneralElement>), Error>
    where
        Self: Sized,
    {
        let backend_element = B::GeneralElement::create_virtual_element(owner)?;
        let this = ComponentNode {
            component: Rc::new(RefCell::new(<C as Component>::new())),
            backend_context: backend_context.clone(),
            backend_element: backend_element.clone(),
        };
        let init = TemplateInit {
            updater: Box::new(this.weak_ref()),
        };
        {
            let mut comp = this.component.borrow_mut();
            <C as ComponentTemplate<B>>::template_init(&mut comp, init);
        }
        Ok((this, backend_element))
    }

    #[inline]
    fn create<'b, R>(
        &'b mut self,
        backend_context: &'b BackendContext<B>,
        owner: &'b mut tree::ForestNodeMut<<B as Backend>::GeneralElement>,
        slot_fn: impl FnMut(
            &mut tree::ForestNodeMut<B::GeneralElement>,
            &Self::SlotData,
        ) -> Result<R, Error>,
    ) -> Result<SlotChildren<R>, Error> {
        if let Ok(mut comp) = self.component.try_borrow_mut() {
            let mut backend_element = owner.borrow_mut(&self.backend_element);
            let ret = <C as ComponentTemplate<B>>::template_create(
                &mut comp,
                backend_context,
                &mut backend_element,
                slot_fn,
            )?;
            <C as Component>::created(&mut comp);
            if <C as ComponentTemplate<B>>::template_mut(&mut comp).clear_dirty() {
                <C as ComponentTemplate<B>>::template_update(
                    &mut comp,
                    backend_context,
                    &mut backend_element,
                    |_| Ok(()), // TODO handling slot upper update
                )?;
            }
            Ok(ret)
        } else {
            Err(Error::RecursiveUpdate)
        }
    }

    #[inline]
    fn apply_updates<'b>(
        &'b mut self,
        backend_context: &'b BackendContext<B>,
        owner: &'b mut tree::ForestNodeMut<B::GeneralElement>,
        force_dirty: bool,
        slot_fn: impl FnMut(
            ListItemChange<&mut tree::ForestNodeMut<B::GeneralElement>, &Self::SlotData>,
        ) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if let Ok(mut comp) = self.component.try_borrow_mut() {
            let mut backend_element = owner.borrow_mut(&self.backend_element);
            if <C as ComponentTemplate<B>>::template_mut(&mut comp).clear_dirty() || force_dirty {
                <C as ComponentTemplate<B>>::template_update(
                    &mut comp,
                    backend_context,
                    &mut backend_element,
                    slot_fn,
                )
            } else {
                <C as ComponentTemplate<B>>::for_each_slot_scope(
                    &mut comp,
                    &mut backend_element,
                    slot_fn,
                )
            }
        } else {
            Err(Error::RecursiveUpdate)
        }
    }
}
