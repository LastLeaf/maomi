use std::{
    cell::{Cell, RefCell},
    marker::PhantomData,
    rc::{Rc, Weak},
};

use crate::{
    backend::{
        context::AsyncCallback, tree::*, Backend, BackendComponent, BackendGeneralElement,
        SupportBackend,
    },
    diff::ListItemChange,
    error::Error,
    node::SlotChildren,
    template::*,
    BackendContext,
};

/// A ref-counted token of a component
pub struct ComponentRc<C: 'static> {
    inner: Box<dyn UpdateScheduler<EnterType = C>>,
    _phantom: PhantomData<C>,
}

impl<C: 'static> ComponentRc<C> {
    pub(crate) fn new(inner: Box<dyn UpdateScheduler<EnterType = C>>) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<C: 'static> Clone for ComponentRc<C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone_scheduler().unwrap(),
            _phantom: PhantomData,
        }
    }
}

impl<C: 'static> ComponentRc<C> {
    /// Schedule an update task, getting the component mutable reference
    ///
    /// The `f` may not be called immediately.
    /// If any other component is still in a visiting or updating task, the `f` will be delayed.
    /// The template is always updated after `f` being called.
    pub async fn update<R: 'static>(
        &self,
        f: impl 'static + FnOnce(&mut C) -> R,
    ) -> Result<R, Error> {
        let ret = Rc::new(Cell::<Option<R>>::new(None));
        let ret2 = ret.clone();
        self.inner
            .enter_mut(
                true,
                Box::new(move |c| {
                    let r = f(c);
                    ret2.set(Some(r));
                }),
            )
            .await?;
        Ok(Rc::try_unwrap(ret)
            .map_err(|_| "Enter callback failed")
            .unwrap()
            .into_inner()
            .unwrap())
    }

    /// Schedule a visiting task, getting the component reference
    ///
    /// The `f` may not be called immediately.
    /// If any other component is still in a visiting or updating task, the `f` will be delayed.
    pub async fn get<R: 'static>(&self, f: impl 'static + FnOnce(&C) -> R) -> R {
        let ret = Rc::new(Cell::<Option<R>>::new(None));
        let ret2 = ret.clone();
        self.inner
            .enter(Box::new(move |c| {
                let r = f(c);
                ret2.set(Some(r));
            }))
            .await;
        Rc::try_unwrap(ret)
            .map_err(|_| "Enter callback failed")
            .unwrap()
            .into_inner()
            .unwrap()
    }

    /// Schedule a visiting task, getting the component mutable reference
    ///
    /// The `f` may not be called immediately.
    /// If any other component is still in a visiting or updating task, the `f` will be delayed.
    /// If the template is needed to be updated, `schedule_update` should be called during `f` execution.
    pub async fn get_mut<R: 'static>(
        &self,
        f: impl 'static + FnOnce(&mut C) -> R,
    ) -> Result<R, Error> {
        let ret = Rc::new(Cell::<Option<R>>::new(None));
        let ret2 = ret.clone();
        self.inner
            .enter_mut(
                false,
                Box::new(move |c| {
                    let r = f(c);
                    ret2.set(Some(r));
                }),
            )
            .await?;
        Ok(Rc::try_unwrap(ret)
            .map_err(|_| "Enter callback failed")
            .unwrap()
            .into_inner()
            .unwrap())
    }
}

/// A component
///
/// This trait must be implemented by components.
/// It contains some lifetime callbacks.
pub trait Component: 'static {
    /// Called when a new instance requested
    fn new() -> Self;

    /// Called after fully created
    fn created(&self) {}

    /// Called before every template updates
    fn before_update(&mut self) {}
}

/// Some component helper functions
///
/// This trait is auto-implemented by `#[component]` .
pub trait ComponentExt<B: Backend, C> {
    type TemplateStructure;

    /// Get a template structure
    ///
    /// The components in the template can be visited within the structure.
    /// If the component has not been fully created yet, `None` is returned.
    fn template_structure(&self) -> Option<&Self::TemplateStructure>;

    /// Manually trigger an update for the template
    fn schedule_update(&mut self)
    where
        C: 'static,
        Self: 'static;

    /// Get a `ComponentRc` for the component
    ///
    /// The `ComponentRc` can move across async steps.
    /// It is useful for doing updates after async steps such as network requests.
    fn rc(&self) -> ComponentRc<C>
    where
        C: 'static,
        Self: 'static;
}

impl<B: Backend, T: ComponentTemplate<B>> ComponentExt<B, Self> for T {
    type TemplateStructure = T::TemplateStructure;

    #[inline]
    fn template_structure(&self) -> Option<&Self::TemplateStructure> {
        <Self as ComponentTemplate<B>>::template(self).structure()
    }

    #[inline]
    fn schedule_update(&mut self)
    where
        T: 'static,
    {
        <Self as ComponentTemplate<B>>::template_mut(self).mark_dirty();
    }

    #[inline]
    fn rc(&self) -> ComponentRc<Self>
    where
        T: 'static,
    {
        <Self as ComponentTemplate<B>>::template(self)
            .component_rc()
            .unwrap()
    }
}

pub(crate) trait UpdateScheduler: 'static {
    type EnterType;
    fn clone_scheduler(&self) -> Option<Box<dyn UpdateScheduler<EnterType = Self::EnterType>>>;
    fn enter(&self, f: Box<dyn FnOnce(&Self::EnterType)>) -> AsyncCallback<()>;
    fn enter_mut(
        &self,
        force_schdule_update: bool,
        f: Box<dyn FnOnce(&mut Self::EnterType)>,
    ) -> AsyncCallback<Result<(), Error>>;
}

pub(crate) trait UpdateSchedulerWeak: 'static {
    type EnterType;
    fn upgrade_scheduler(&self) -> Option<Box<dyn UpdateScheduler<EnterType = Self::EnterType>>>;
}

/// A node that wraps a component instance
pub struct ComponentNode<B: Backend, C: ComponentTemplate<B> + Component> {
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

impl<B: Backend, C: ComponentTemplate<B> + Component> ComponentNode<B, C> {
    /// Get a `ComponentRc` for the component
    ///
    /// The `ComponentRc` can move across async steps.
    /// It is useful for doing updates after async steps such as network requests.
    #[inline]
    pub fn rc(&self) -> ComponentRc<C> {
        let component = Box::new(self.clone());
        ComponentRc::new(component)
    }

    /// Get a weak reference
    #[inline]
    pub fn weak_ref(&self) -> ComponentNodeWeak<B, C> {
        ComponentNodeWeak {
            component: Rc::downgrade(&self.component),
            backend_context: self.backend_context.clone(),
            backend_element: self.backend_element.clone(),
        }
    }
}

impl<B: Backend, C: ComponentTemplate<B> + Component> UpdateScheduler for ComponentNode<B, C> {
    type EnterType = C;

    #[inline]
    fn clone_scheduler(&self) -> Option<Box<dyn UpdateScheduler<EnterType = Self::EnterType>>> {
        Some(Box::new(self.clone()))
    }

    #[inline]
    fn enter(&self, f: Box<dyn FnOnce(&Self::EnterType)>) -> AsyncCallback<()> {
        let component = self.component.clone();
        self.backend_context.enter(move |_| f(&component.borrow()))
    }

    #[inline]
    fn enter_mut(
        &self,
        force_schdule_update: bool,
        f: Box<dyn FnOnce(&mut Self::EnterType)>,
    ) -> AsyncCallback<Result<(), Error>> {
        let backend_element = self.backend_element.clone();
        let backend_context = self.backend_context.clone();
        let component = self.component.clone();
        self.backend_context
            .enter::<Result<(), Error>, _>(move |_| {
                let mut comp = component.borrow_mut();
                f(&mut comp);
                if <C as ComponentTemplate<B>>::template_mut(&mut comp).clear_dirty()
                    || force_schdule_update
                {
                    let mut backend_element = backend_element.borrow_mut();
                    <C as Component>::before_update(&mut comp);
                    <C as ComponentTemplate<B>>::template_update(
                        &mut comp,
                        &backend_context,
                        &mut backend_element,
                        |_| {
                            // TODO notify slot changes to the owner component
                            Ok(())
                        },
                    )?;
                }
                Ok(())
            })
    }
}

/// A node that wraps a component instance
pub struct ComponentNodeWeak<B: Backend, C: ComponentTemplate<B> + Component> {
    component: Weak<RefCell<C>>,
    backend_context: BackendContext<B>,
    backend_element: ForestNodeRc<B::GeneralElement>,
}

impl<B: Backend, C: ComponentTemplate<B> + Component> ComponentNodeWeak<B, C> {
    /// Upgrade to a strong reference
    #[inline]
    pub fn upgrade(&self) -> Option<ComponentNode<B, C>> {
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

impl<B: Backend, C: ComponentTemplate<B> + Component> UpdateSchedulerWeak
    for ComponentNodeWeak<B, C>
{
    type EnterType = C;

    #[inline]
    fn upgrade_scheduler(&self) -> Option<Box<dyn UpdateScheduler<EnterType = Self::EnterType>>> {
        if let Some(this) = self.upgrade() {
            Some(Box::new(this))
        } else {
            None
        }
    }
}

impl<B: Backend, C: ComponentTemplate<B> + Component> SupportBackend<B> for C {
    type Target = ComponentNode<B, C>;
}

impl<B: Backend, C: ComponentTemplate<B> + Component> BackendComponent<B> for ComponentNode<B, C> {
    type SlotData = <C as ComponentTemplate<B>>::SlotData;
    type UpdateTarget = C;
    type UpdateContext = bool;

    #[inline]
    fn init<'b>(
        backend_context: &'b BackendContext<B>,
        owner: &'b mut ForestNodeMut<B::GeneralElement>,
    ) -> Result<(Self, ForestNodeRc<B::GeneralElement>), Error>
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
        owner: &'b mut ForestNodeMut<<B as Backend>::GeneralElement>,
        update_fn: impl FnOnce(&mut C, &mut bool),
        slot_fn: impl FnMut(&mut ForestNodeMut<B::GeneralElement>, &Self::SlotData) -> Result<R, Error>,
    ) -> Result<SlotChildren<R>, Error> {
        if let Ok(mut comp) = self.component.try_borrow_mut() {
            let mut backend_element = owner.borrow_mut(&self.backend_element);
            let mut force_dirty = false;
            update_fn(&mut comp, &mut force_dirty);
            let ret = <C as ComponentTemplate<B>>::template_create(
                &mut comp,
                backend_context,
                &mut backend_element,
                slot_fn,
            )?;
            <C as Component>::created(&mut comp);
            Ok(ret)
        } else {
            Err(Error::RecursiveUpdate)
        }
    }

    #[inline]
    fn apply_updates<'b>(
        &'b mut self,
        backend_context: &'b BackendContext<B>,
        owner: &'b mut ForestNodeMut<B::GeneralElement>,
        update_fn: impl FnOnce(&mut C, &mut bool),
        slot_fn: impl FnMut(
            ListItemChange<&mut ForestNodeMut<B::GeneralElement>, &Self::SlotData>,
        ) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if let Ok(mut comp) = self.component.try_borrow_mut() {
            let mut backend_element = owner.borrow_mut(&self.backend_element);
            let mut force_dirty = false;
            update_fn(&mut comp, &mut force_dirty);
            if <C as ComponentTemplate<B>>::template_mut(&mut comp).clear_dirty() || force_dirty {
                <C as Component>::before_update(&mut comp);
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
