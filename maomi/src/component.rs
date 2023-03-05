//! The component interface.
//! 
//! Pages are composed by components.
//! This module contains basic types about components.
//! 
//! A component should implement two traits:
//! * the `ComponentTemplate` trait is usually auto-implemented by `#[component]` (no need to implement manually);
//! * the `Component` trait should be implemented manually.
//! 
//! When a component should be created,
//! the framework calls the `Component::new` function and owns the created component.
//! It is not possible to get the ownership of it,
//! but a `ComponentRc` can be obtained.
//! 
//! `ComponentRc` is a ref-counted token of the component.
//! The component can be visited through functions like `ComponentRc::task` and `ComponentRc::update` .
//! 

use async_trait::async_trait;
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
    error::Error,
    node::{OwnerWeak, SlotChange, SlotKindTrait},
    template::*,
    BackendContext,
};

/// A ref-counted token of a component.
pub struct ComponentRc<C: 'static> {
    inner: Rc<dyn UpdateScheduler<EnterType = C>>,
    _phantom: PhantomData<C>,
}

impl<C: 'static> Clone for ComponentRc<C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<C: 'static> ComponentRc<C> {
    pub(crate) fn new(inner: Rc<dyn UpdateScheduler<EnterType = C>>) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    fn downgrade(&self) -> ComponentWeak<C> {
        ComponentWeak {
            inner: Rc::downgrade(&self.inner),
            _phantom: PhantomData,
        }
    }

    /// Schedule an update in another task, getting the component mutable reference.
    ///
    /// The `f` will be called asynchronously.
    /// The template is always updated after `f` being called.
    /// Panics if any error occurred during update.
    pub fn task<R: 'static>(
        &self,
        f: impl 'static + FnOnce(&mut C) -> R,
    ) {
        self.inner
            .clone()
            .enter_mut_detached(Box::new(move |c| {
                f(c);
                true
            }));
    }

    /// Schedule an update in another task, getting the component mutable reference.
    ///
    /// The `f` will be called asynchronously.
    /// If the template is needed to be updated, `ComponentMutCtx::need_update` should be called during `f` execution.
    /// Panics if any error occurred during update.
    pub fn task_with<R: 'static>(
        &self,
        f: impl 'static + FnOnce(&mut C, &mut ComponentMutCtx) -> R,
    ) {
        self.inner
            .clone()
            .enter_mut_detached(Box::new(move |c| {
                let mut ctx = ComponentMutCtx { need_update: false };
                f(c, &mut ctx);
                ctx.need_update
            }));
    }

    /// Schedule an update, getting the component mutable reference.
    ///
    /// The `f` will be called asynchronously.
    /// The template is always updated after `f` being called.
    pub async fn update<R: 'static>(
        &self,
        f: impl 'static + FnOnce(&mut C) -> R,
    ) -> Result<R, Error> {
        let ret = Rc::new(Cell::<Option<R>>::new(None));
        let ret2 = ret.clone();
        self.inner
            .clone()
            .enter_mut(Box::new(move |c| {
                let r = f(c);
                ret2.set(Some(r));
                true
            }))
            .await?;
        Ok(Rc::try_unwrap(ret)
            .map_err(|_| "Enter callback failed")
            .unwrap()
            .into_inner()
            .unwrap())
    }

    /// Schedule a visiting task, getting the component mutable reference.
    ///
    /// The `f` will be called asynchronously.
    /// If the template is needed to be updated, `ComponentMutCtx::need_update` should be called during `f` execution.
    pub async fn update_with<R: 'static>(
        &self,
        f: impl 'static + FnOnce(&mut C, &mut ComponentMutCtx) -> R,
    ) -> Result<R, Error> {
        let ret = Rc::new(Cell::<Option<R>>::new(None));
        let ret2 = ret.clone();
        self.inner
            .clone()
            .enter_mut(Box::new(move |c| {
                let mut ctx = ComponentMutCtx { need_update: false };
                let r = f(c, &mut ctx);
                ret2.set(Some(r));
                ctx.need_update
            }))
            .await?;
        Ok(Rc::try_unwrap(ret)
            .map_err(|_| "Enter callback failed")
            .unwrap()
            .into_inner()
            .unwrap())
    }

    /// Schedule a visiting task, getting the component reference.
    ///
    /// The `f` will be called asynchronously.
    pub async fn get<R: 'static>(&self, f: impl 'static + FnOnce(&C) -> R) -> R {
        let ret = Rc::new(Cell::<Option<R>>::new(None));
        let ret2 = ret.clone();
        self.inner
            .clone()
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
}

/// A weak ref-counted token of a component.
/// 
/// This is the weak version of `ComponentRc` ,
/// which does not prevent the component from dropped.
pub struct ComponentWeak<C> {
    inner: Weak<dyn UpdateScheduler<EnterType = C>>,
    _phantom: PhantomData<C>,
}

impl<C> Clone for ComponentWeak<C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<C: 'static> ComponentWeak<C> {
    pub(crate) fn to_owner_weak(&self) -> Box<dyn OwnerWeak> {
        self.clone_owner_weak()
    }

    /// Upgrade to a `ComponentRc`
    pub fn upgrade(&self) -> Option<ComponentRc<C>> {
        let inner = self.inner.upgrade()?;
        Some(ComponentRc {
            inner,
            _phantom: PhantomData,
        })
    }
}

impl<C: 'static> OwnerWeak for ComponentWeak<C> {
    fn apply_updates(&self) -> Result<(), Error> {
        if let Some(x) = self.inner.upgrade() {
            x.sync_update()?;
        }
        Ok(())
    }

    fn clone_owner_weak(&self) -> Box<dyn OwnerWeak> {
        Box::new(Self {
            inner: self.inner.clone(),
            _phantom: PhantomData
        })
    }
}

/// A helper for `ComponentRc::task_with` or `ComponentRc::update_with` .
pub struct ComponentMutCtx {
    need_update: bool,
}

impl ComponentMutCtx {
    /// Schedule an update when the `ComponentRc::task_with` or `ComponentRc::update_with` call ends
    pub fn need_update(&mut self) {
        self.need_update = true;
    }
}

/// A component.
///
/// This trait must be implemented by components.
/// It contains some lifetime callbacks.
pub trait Component: 'static {
    /// Called when a new component need to be created.
    /// 
    /// This function will be called once when a new instance is needed.
    fn new() -> Self;

    /// Called after the component is fully created.
    /// 
    /// This function can be used to do some async startup tasks,
    /// such as network requests.
    fn created(&self) {}

    /// Called before every template updates.
    /// 
    /// This function can be used to update some cache that used in the template.
    fn before_template_apply(&mut self) {}
}

/// Some component utility functions.
///
/// This trait is auto-implemented by `#[component]` .
#[async_trait(?Send)]
pub trait ComponentExt<B: Backend, C> {
    /// The type of template structure.
    type TemplateStructure;

    /// Get the template structure.
    ///
    /// The components in the template can be visited within the structure.
    /// If the component has not been fully created yet, `None` is returned.
    fn template_structure(&self) -> Option<&Self::TemplateStructure>;

    /// Manually trigger an update for the template.
    /// 
    /// In most cases, you should not call this function manually.
    /// Use `ComponentRc::task` or `ComponentRc::update` instead.
    async fn apply_updates(&mut self) -> Result<(), Error>
    where
        C: 'static,
        Self: 'static;

    /// Get a mutable reference of the component and then do updates.
    ///
    /// It is a short cut for `.rc().update()`
    /// In most cases, you should not call this function manually.
    /// Use `ComponentRc::task` or `ComponentRc::update` instead.
    async fn update<R>(
        &self,
        f: impl 'static + for<'r> FnOnce(&'r mut Self) -> R,
    ) -> Result<R, Error>
    where
        R: 'static,
        C: 'static,
        Self: 'static;

    /// Get a ref-counted token `ComponentRc` for the component.
    ///
    /// The `ComponentRc` can move across async steps.
    /// It is useful for doing updates after async steps such as network requests.
    fn rc(&self) -> ComponentRc<C>
    where
        C: 'static,
        Self: 'static;

    /// Get a ref-counted token `ComponentWeak` for the component.
    ///
    /// Similar to `ComponentRc` , the `ComponentWeak` can move across async steps.
    /// It is a weak ref which does not prevent dropping the component.
    fn weak(&self) -> ComponentWeak<C>
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
    async fn update<R: 'static>(
        &self,
        f: impl 'static + for<'r> FnOnce(&'r mut Self) -> R,
    ) -> Result<R, Error>
    where
        T: 'static,
    {
        <Self as ComponentExt<B, Self>>::rc(self).update(f).await
    }

    // TODO should improve interface (currently this requires B to be specific)
    #[inline]
    fn rc(&self) -> ComponentRc<Self>
    where
        T: 'static,
    {
        <Self as ComponentTemplate<B>>::template(self)
            .component_rc()
            .expect("Cannot get `ComponentRc` before initialization")
    }

    #[inline]
    fn weak(&self) -> ComponentWeak<Self>
    where
        T: 'static,
    {
        <Self as ComponentTemplate<B>>::template(self)
            .component_weak()
            .expect("Cannot get `ComponentWeak` before initialization")
    }
}

/// A component that can be used as mount point when prerendering
///
/// In prerendering stage,
/// components can do some async tasks (network request, etc.)
/// before actually doing the build of the component.
#[async_trait]
pub trait PrerenderableComponent: Component {
    /// The type of the query data.
    type QueryData;
    /// The type of the prerendering generated data.
    type PrerenderingData;

    /// Generate the prerendering data.
    ///
    /// This function accepts `QueryData` which represents some startup state,
    /// i.e. the URL params or the POST data.
    /// The generated `PrerenderingData` will be used in `apply_prerendering_data` .
    /// This function will be called either in prerendering process (server-side)
    /// or in prerendering-apply process (client-side).
    /// This also requires the `PrerenderingData` to be transferable.
    async fn prerendering_data(query_data: &Self::QueryData) -> Self::PrerenderingData;

    /// Apply the prerendering data.
    ///
    /// This function will be called
    /// both in prerendering process (server-side) and in prerendering-apply process (client-side).
    /// The result **must** be the same in two processes.
    fn apply_prerendering_data(&mut self, data: Self::PrerenderingData);
}

pub(crate) trait UpdateScheduler: 'static {
    type EnterType;
    fn enter(self: Rc<Self>, f: Box<dyn FnOnce(&Self::EnterType)>) -> AsyncCallback<()>;
    fn enter_mut(
        self: Rc<Self>,
        f: Box<dyn FnOnce(&mut Self::EnterType) -> bool>,
    ) -> AsyncCallback<Result<(), Error>>;
    fn enter_mut_detached(
        self: Rc<Self>,
        f: Box<dyn FnOnce(&mut Self::EnterType) -> bool>,
    );
    fn sync_update(&self) -> Result<(), Error>;
}

pub(crate) trait UpdateSchedulerWeak: 'static {
    type EnterType;
    fn upgrade_scheduler(&self) -> Option<Rc<dyn UpdateScheduler<EnterType = Self::EnterType>>>;
    fn to_owner_weak(&self) -> Box<dyn OwnerWeak>;
}

/// A node that wraps a component instance.
#[derive(Clone)]
pub struct ComponentNode<C: Component> {
    inner: Rc<RefCell<C>>,
    backend_element_token: ForestToken,
    rc: ComponentRc<C>,
}

impl<C: Component> ComponentNode<C> {
    fn new<B: Backend>(
        c: C,
        backend_context: BackendContext<B>,
        forest_node_rc: ForestNodeRc<<B as Backend>::GeneralElement>,
        owner_weak: Box<dyn OwnerWeak>,
    ) -> Self
    where
        C: ComponentTemplate<B>,
    {
        let inner = Rc::new(RefCell::new(c));
        let backend_element_token = forest_node_rc.token();
        let rc: Rc<dyn UpdateScheduler<EnterType = C>> = Rc::new(ComponentNodeInBackend {
            inner: inner.clone(),
            backend_context,
            forest_node_rc,
            owner_weak,
        });
        let rc = ComponentRc::new(rc);
        Self {
            inner,
            backend_element_token,
            rc,
        }
    }

    pub(crate) fn component(&self) -> &RefCell<C> {
        &self.inner
    }

    /// Get a ref-counted token `ComponentRc` for the component.
    ///
    /// The `ComponentRc` can move across async steps.
    /// It is useful for doing updates after async steps such as network requests.
    #[inline]
    pub fn rc(&self) -> ComponentRc<C> {
        self.rc.clone()
    }

    /// Get a ref-counted token `ComponentWeak` for the component.
    ///
    /// Similar to `ComponentRc` , the `ComponentWeak` can move across async steps.
    /// It is a weak ref which does not prevent dropping the component.
    #[inline]
    pub fn weak(&self) -> ComponentWeak<C> {
        self.rc.downgrade()
    }
}

struct ComponentNodeInBackend<B: Backend, C: ComponentTemplate<B> + Component> {
    inner: Rc<RefCell<C>>,
    backend_context: BackendContext<B>,
    forest_node_rc: ForestNodeRc<B::GeneralElement>,
    owner_weak: Box<dyn OwnerWeak>,
}

impl<B: Backend, C: ComponentTemplate<B> + Component> ComponentNodeInBackend<B, C> {
    fn prepare_inner_changes(
        this: &Rc<Self>,
        f: Box<dyn FnOnce(&mut C) -> bool>,
    ) -> Result<(), Error> {
        let has_slot_changes = {
            let mut comp = this.inner.borrow_mut();
            let force_schedule_update = f(&mut comp);
            if <C as ComponentTemplate<B>>::template_mut(&mut comp).clear_dirty()
                || force_schedule_update
            {
                let mut backend_element = this.forest_node_rc.borrow_mut();
                <C as Component>::before_template_apply(&mut comp);
                let has_slot_changes =
                    <C as ComponentTemplate<B>>::template_update_store_slot_changes(
                        &mut comp,
                        &this.backend_context,
                        &mut backend_element,
                    )?;
                has_slot_changes
            } else {
                false
            }
        };
        if has_slot_changes {
            this.owner_weak.apply_updates()?;
        }
        Ok(())
    }
}

impl<B: Backend, C: ComponentTemplate<B> + Component> UpdateScheduler for ComponentNodeInBackend<B, C> {
    type EnterType = C;

    #[inline]
    fn enter(self: Rc<Self>, f: Box<dyn FnOnce(&Self::EnterType)>) -> AsyncCallback<()> {
        self.backend_context.clone().enter(move |_| f(&self.inner.borrow()))
    }

    #[inline]
    fn enter_mut(
        self: Rc<Self>,
        f: Box<dyn FnOnce(&mut Self::EnterType) -> bool>,
    ) -> AsyncCallback<Result<(), Error>> {
        self.backend_context.clone().enter::<Result<(), Error>, _>(move |_| {
            Self::prepare_inner_changes(&self, f)
        })
    }

    #[inline]
    fn enter_mut_detached(
        self: Rc<Self>,
        f: Box<dyn FnOnce(&mut Self::EnterType) -> bool>,
    ) {
        // the sync part of `f` is always executed, so it does not require to poll
        let _ = self.backend_context.clone().enter::<(), _>(move |_| {
            if let Err(err) = Self::prepare_inner_changes(&self, f) {
                panic!("{}", err);
            }
        });
    }

    #[inline]
    fn sync_update(&self) -> Result<(), Error> {
        let has_slot_changes = {
            let mut comp = self.inner.borrow_mut();
            <C as ComponentTemplate<B>>::template_mut(&mut comp).clear_dirty();
            let mut backend_element = self.forest_node_rc.borrow_mut();
            <C as Component>::before_template_apply(&mut comp);
            let has_slot_changes = <C as ComponentTemplate<B>>::template_update_store_slot_changes(
                &mut comp,
                &self.backend_context,
                &mut backend_element,
            )?;
            has_slot_changes
        };
        if has_slot_changes {
            self.owner_weak.apply_updates()?;
        }
        Ok(())
    }
}

impl<C: Component + ComponentSlotKind> SupportBackend for C {
    type Target = ComponentNode<C>;
    type SlotChildren<SlotContent> = <C as ComponentSlotKind>::SlotChildren<SlotContent>;
}

impl<B: Backend, C: ComponentTemplate<B> + Component> BackendComponent<B> for ComponentNode<C> {
    type SlotData = <C as ComponentSlotKind>::SlotData;
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
        let this = ComponentNode::new(
            <C as Component>::new(),
            backend_context.clone(),
            backend_element.clone(),
            owner_weak.clone_owner_weak(),
        );
        let init = TemplateInit {
            updater: this.rc.downgrade(),
        };
        {
            let mut comp = this.component().borrow_mut();
            <C as ComponentTemplate<B>>::template_init(&mut comp, init);
        }
        Ok((this, backend_element))
    }

    #[inline(never)]
    fn create<'b>(
        &'b mut self,
        backend_context: &'b BackendContext<B>,
        owner: &'b mut ForestNodeMut<<B as Backend>::GeneralElement>,
        update_fn: Box<dyn 'b + FnOnce(&mut C, &mut bool)>,
        slot_fn: &mut dyn FnMut(
            &mut ForestNodeMut<B::GeneralElement>,
            &ForestToken,
            &Self::SlotData,
        ) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if let Ok(mut comp) = self.component().try_borrow_mut() {
            let mut backend_element = owner.borrow_mut_token(&self.backend_element_token).unwrap();
            let mut force_dirty = false;
            update_fn(&mut comp, &mut force_dirty);
            <C as Component>::before_template_apply(&mut comp);
            <C as ComponentTemplate<B>>::template_create_or_update(
                &mut comp,
                backend_context,
                &mut backend_element,
                &mut |slot_change| {
                    match slot_change {
                        SlotChange::Added(n, t, d) => slot_fn(n, t, d),
                        _ => Err(Error::TreeNotCreated),
                    }
                },
            )?;
            #[cfg(not(feature = "prerendering"))]
            <C as Component>::created(&comp);
            #[cfg(feature = "prerendering")]
            if backend_context.initial_backend_stage() != crate::backend::BackendStage::Prerendering
            {
                <C as Component>::created(&comp);
            }
            Ok(())
        } else {
            Err(Error::RecursiveUpdate)
        }
    }

    #[inline(never)]
    fn apply_updates<'b>(
        &'b mut self,
        backend_context: &'b BackendContext<B>,
        owner: &'b mut ForestNodeMut<B::GeneralElement>,
        update_fn: Box<dyn 'b + FnOnce(&mut C, &mut bool)>,
        mut slot_fn: &mut dyn FnMut(
            SlotChange<&mut ForestNodeMut<B::GeneralElement>, &ForestToken, &Self::SlotData>,
        ) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if let Ok(mut comp) = self.component().try_borrow_mut() {
            let mut force_dirty = false;
            update_fn(&mut comp, &mut force_dirty);
            if <C as ComponentTemplate<B>>::template_mut(&mut comp).clear_dirty() || force_dirty {
                // if any data changed, do updates
                let mut backend_element = owner.borrow_mut_token(&self.backend_element_token).unwrap();
                <C as Component>::before_template_apply(&mut comp);
                <C as ComponentTemplate<B>>::template_create_or_update(
                    &mut comp,
                    backend_context,
                    &mut backend_element,
                    &mut slot_fn,
                )
            } else {
                let changes = <C as ComponentTemplate<B>>::template_mut(&mut comp)
                    .pending_slot_changes(Vec::with_capacity(0));
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
                    let mut backend_element = owner.borrow_mut_token(&self.backend_element_token).unwrap();
                    <C as ComponentTemplate<B>>::for_each_slot_scope(
                        &mut comp,
                        &mut backend_element,
                        &mut slot_fn,
                    )
                }
            }
        } else {
            Err(Error::RecursiveUpdate)
        }
    }
}
