use std::{
    cell::{Cell, RefCell},
    marker::PhantomData,
    rc::{Rc, Weak},
};
use async_trait::async_trait;

use crate::{
    backend::{
        context::AsyncCallback, tree::*, Backend, BackendComponent, BackendGeneralElement,
        SupportBackend,
    },
    error::Error,
    node::{OwnerWeak, SlotChange},
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
                Box::new(move |c| {
                    let r = f(c);
                    ret2.set(Some(r));
                    true
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
    /// If the template is needed to be updated, `ComponentMutCtx::need_update` should be called during `f` execution.
    pub async fn get_mut<R: 'static>(
        &self,
        f: impl 'static + FnOnce(&mut C, &mut ComponentMutCtx) -> R,
    ) -> Result<R, Error> {
        let ret = Rc::new(Cell::<Option<R>>::new(None));
        let ret2 = ret.clone();
        self.inner
            .enter_mut(
                Box::new(move |c| {
                    let mut ctx = ComponentMutCtx { need_update: false };
                    let r = f(c, &mut ctx);
                    ret2.set(Some(r));
                    ctx.need_update
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

/// A helper for `ComponentRc::get_mut`
pub struct ComponentMutCtx {
    need_update: bool,
}

impl ComponentMutCtx {
    /// Request an update when the `ComponentRc::get_mut` call ends
    pub fn need_update(&mut self) {
        self.need_update = true;
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
    fn before_template_apply(&mut self) {}
}

/// Some component helper functions
///
/// This trait is auto-implemented by `#[component]` .
#[async_trait(?Send)]
pub trait ComponentExt<B: Backend, C> {
    type TemplateStructure;

    /// Get a template structure
    ///
    /// The components in the template can be visited within the structure.
    /// If the component has not been fully created yet, `None` is returned.
    fn template_structure(&self) -> Option<&Self::TemplateStructure>;

    /// Manually trigger an update for the template
    async fn apply_updates(&mut self) -> Result<(), Error>
    where
        C: 'static,
        Self: 'static;

    /// Get a mutable reference of the component and then do updates
    ///
    /// It is a short cut for `.rc().update()`
    async fn update<R>(&self, f: impl 'static + for<'r> FnOnce(&'r mut Self) -> R) -> Result<R, Error>
    where
        R: 'static,
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

#[async_trait(?Send)]
impl<B: Backend, T: ComponentTemplate<B>> ComponentExt<B, Self> for T {
    type TemplateStructure = T::TemplateStructure;

    #[inline]
    fn template_structure(&self) -> Option<&Self::TemplateStructure> {
        <Self as ComponentTemplate<B>>::template(self).structure()
    }

    #[inline]
    async fn apply_updates(&mut self) -> Result<(), Error>
    where
        T: 'static,
    {
        <Self as ComponentTemplate<B>>::template_mut(self).mark_dirty();
        self.rc().update(|_| {}).await
    }

    #[inline]
    async fn update<R: 'static>(&self, f: impl 'static + for<'r> FnOnce(&'r mut Self) -> R) -> Result<R, Error>
    where
        T: 'static,
    {
        <Self as ComponentExt<B, Self>>::rc(self).update(f).await
    }

    #[inline]
    fn rc(&self) -> ComponentRc<Self>
    where
        T: 'static,
    {
        <Self as ComponentTemplate<B>>::template(self)
            .component_rc()
            .expect("Cannot get `ComponentRc` before initialization")
    }
}

pub(crate) trait UpdateScheduler: 'static {
    type EnterType;
    fn clone_scheduler(&self) -> Option<Box<dyn UpdateScheduler<EnterType = Self::EnterType>>>;
    fn enter(&self, f: Box<dyn FnOnce(&Self::EnterType)>) -> AsyncCallback<()>;
    fn enter_mut(
        &self,
        f: Box<dyn FnOnce(&mut Self::EnterType) -> bool>,
    ) -> AsyncCallback<Result<(), Error>>;
    fn sync_update(&self) -> Result<(), Error>;
}

pub(crate) trait UpdateSchedulerWeak: 'static {
    type EnterType;
    fn upgrade_scheduler(&self) -> Option<Box<dyn UpdateScheduler<EnterType = Self::EnterType>>>;
    fn clone_scheduler(&self) -> Box<dyn UpdateSchedulerWeak<EnterType = Self::EnterType>>;
    fn to_owner_weak(&self) -> Box<dyn OwnerWeak>;
}

/// A node that wraps a component instance
pub struct ComponentNode<B: Backend, C: ComponentTemplate<B> + Component> {
    inner: Rc<(
        RefCell<C>,
        BackendContext<B>,
        ForestNodeRc<B::GeneralElement>,
        Box<dyn OwnerWeak>,
    )>,
}

impl<B: Backend, C: ComponentTemplate<B> + Component> Clone for ComponentNode<B, C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<B: Backend, C: ComponentTemplate<B> + Component> ComponentNode<B, C> {
    pub(crate) fn component(&self) -> &RefCell<C> {
        &self.inner.0
    }

    fn backend_context(&self) -> &BackendContext<B> {
        &self.inner.1
    }

    fn backend_element(&self) -> &ForestNodeRc<B::GeneralElement> {
        &self.inner.2
    }

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
            inner: Rc::downgrade(&self.inner),
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
        let inner = self.inner.clone();
        self.backend_context().enter(move |_| f(&inner.0.borrow()))
    }

    #[inline]
    fn enter_mut(
        &self,
        f: Box<dyn FnOnce(&mut Self::EnterType) -> bool>,
    ) -> AsyncCallback<Result<(), Error>> {
        let inner = self.inner.clone();
        self.backend_context()
            .enter::<Result<(), Error>, _>(move |_| {
                let has_slot_changes = {
                    let mut comp = inner.0.borrow_mut();
                    let force_schdule_update = f(&mut comp);
                    if <C as ComponentTemplate<B>>::template_mut(&mut comp).clear_dirty()
                        || force_schdule_update
                    {
                        let mut backend_element = inner.2.borrow_mut();
                        <C as Component>::before_template_apply(&mut comp);
                        let has_slot_changes =
                            <C as ComponentTemplate<B>>::template_update_store_slot_changes(
                                &mut comp,
                                &inner.1,
                                &mut backend_element,
                            )?;
                        has_slot_changes
                    } else {
                        false
                    }
                };
                if has_slot_changes {
                    inner.3.apply_updates()?;
                }
                Ok(())
            })
    }

    #[inline]
    fn sync_update(&self) -> Result<(), Error> {
        let inner = &self.inner;
        let has_slot_changes = {
            let mut comp = inner.0.borrow_mut();
            <C as ComponentTemplate<B>>::template_mut(&mut comp).clear_dirty();
            let mut backend_element = inner.2.borrow_mut();
            <C as Component>::before_template_apply(&mut comp);
            let has_slot_changes = <C as ComponentTemplate<B>>::template_update_store_slot_changes(
                &mut comp,
                &inner.1,
                &mut backend_element,
            )?;
            has_slot_changes
        };
        if has_slot_changes {
            inner.3.apply_updates()?;
        }
        Ok(())
    }
}

/// A node that wraps a component instance
pub struct ComponentNodeWeak<B: Backend, C: ComponentTemplate<B> + Component> {
    inner: Weak<(
        RefCell<C>,
        BackendContext<B>,
        ForestNodeRc<B::GeneralElement>,
        Box<dyn OwnerWeak>,
    )>,
}

impl<B: Backend, C: ComponentTemplate<B> + Component> ComponentNodeWeak<B, C> {
    /// Upgrade to a strong reference
    #[inline]
    pub fn upgrade(&self) -> Option<ComponentNode<B, C>> {
        if let Some(inner) = self.inner.upgrade() {
            Some(ComponentNode { inner })
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

    #[inline]
    fn clone_scheduler(&self) -> Box<dyn UpdateSchedulerWeak<EnterType = Self::EnterType>> {
        Box::new(Self {
            inner: self.inner.clone(),
        })
    }

    #[inline]
    fn to_owner_weak(&self) -> Box<dyn OwnerWeak> {
        Box::new(Self {
            inner: self.inner.clone(),
        })
    }
}

impl<B: Backend, C: ComponentTemplate<B> + Component> OwnerWeak for ComponentNodeWeak<B, C> {
    fn apply_updates(&self) -> Result<(), Error> {
        if let Some(x) = self.upgrade_scheduler() {
            x.sync_update()?;
        }
        Ok(())
    }

    fn clone_owner_weak(&self) -> Box<dyn OwnerWeak> {
        Box::new(Self {
            inner: self.inner.clone(),
        })
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
        owner_weak: &Box<dyn OwnerWeak>,
    ) -> Result<(Self, ForestNodeRc<B::GeneralElement>), Error>
    where
        Self: Sized,
    {
        let backend_element = B::GeneralElement::create_virtual_element(owner)?;
        let this = ComponentNode {
            inner: Rc::new((
                RefCell::new(<C as Component>::new()),
                backend_context.clone(),
                backend_element.clone(),
                owner_weak.clone_owner_weak(),
            )),
        };
        let init = TemplateInit {
            updater: Box::new(this.weak_ref()),
        };
        {
            let mut comp = this.component().borrow_mut();
            <C as ComponentTemplate<B>>::template_init(&mut comp, init);
        }
        Ok((this, backend_element))
    }

    #[inline]
    fn create<'b>(
        &'b mut self,
        backend_context: &'b BackendContext<B>,
        owner: &'b mut ForestNodeMut<<B as Backend>::GeneralElement>,
        update_fn: impl FnOnce(&mut C, &mut bool),
        slot_fn: impl FnMut(
            &mut ForestNodeMut<B::GeneralElement>,
            &ForestToken,
            &Self::SlotData,
        ) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if let Ok(mut comp) = self.component().try_borrow_mut() {
            let mut backend_element = owner.borrow_mut(&self.backend_element());
            let mut force_dirty = false;
            update_fn(&mut comp, &mut force_dirty);
            <C as Component>::before_template_apply(&mut comp);
            <C as ComponentTemplate<B>>::template_create(
                &mut comp,
                backend_context,
                &mut backend_element,
                slot_fn,
            )?;
            <C as Component>::created(&mut comp);
            Ok(())
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
        mut slot_fn: impl FnMut(
            SlotChange<&mut ForestNodeMut<B::GeneralElement>, &ForestToken, &Self::SlotData>,
        ) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if let Ok(mut comp) = self.component().try_borrow_mut() {
            let mut force_dirty = false;
            update_fn(&mut comp, &mut force_dirty);
            if <C as ComponentTemplate<B>>::template_mut(&mut comp).clear_dirty() || force_dirty {
                // if any data changed, do updates
                let mut backend_element = owner.borrow_mut(&self.backend_element());
                <C as Component>::before_template_apply(&mut comp);
                <C as ComponentTemplate<B>>::template_update(
                    &mut comp,
                    backend_context,
                    &mut backend_element,
                    slot_fn,
                )
            } else {
                let changes = <C as ComponentTemplate<B>>::template_mut(&mut comp).pending_slot_changes(Vec::with_capacity(0));
                if changes.len() > 0 {
                    // if there is pending slot changes, use it
                    for slot_change in changes {
                        match slot_change {
                            SlotChange::Unchanged(..) => {}
                            SlotChange::DataChanged(_, t, _) => {
                                let addr = t.stable_addr();
                                slot_fn(SlotChange::Unchanged(
                                    owner
                                        .borrow_mut_token(&t)
                                        .as_mut()
                                        .ok_or(Error::ListChangeWrong)?,
                                    &t,
                                    &<C as ComponentTemplate<B>>::template_mut(&mut comp)
                                        .slot_scopes()
                                        .get(addr)?
                                        .1,
                                ))?;
                            }
                            SlotChange::Added(_, t, _) => {
                                let addr = t.stable_addr();
                                slot_fn(SlotChange::Added(
                                    owner
                                        .borrow_mut_token(&t)
                                        .as_mut()
                                        .ok_or(Error::ListChangeWrong)?,
                                    &t,
                                    &<C as ComponentTemplate<B>>::template_mut(&mut comp)
                                        .slot_scopes()
                                        .get(addr)?
                                        .1,
                                ))?;
                            }
                            SlotChange::Removed(t) => {
                                slot_fn(SlotChange::Removed(&t))?;
                            }
                        }
                    }
                    Ok(())
                } else {
                    // if nothing changed, just return the slots
                    let mut backend_element = owner.borrow_mut(&self.backend_element());
                    <C as ComponentTemplate<B>>::for_each_slot_scope(
                        &mut comp,
                        &mut backend_element,
                        slot_fn,
                    )
                }
            }
        } else {
            Err(Error::RecursiveUpdate)
        }
    }
}
