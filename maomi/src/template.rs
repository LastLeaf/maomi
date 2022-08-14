use crate::{
    backend::{tree::*, Backend},
    component::*,
    error::Error,
    node::{SlotChildren, SlotChange, OwnerWeak},
    BackendContext,
};

/// An init object for the template
///
/// This struct is auto-managed by `#[component]` .
pub struct TemplateInit<C: ?Sized> {
    pub(crate) updater: Box<dyn UpdateSchedulerWeak<EnterType = C>>,
}

/// Some helper functions for the template type
/// 
/// This struct is auto-managed by `#[component]` and should not be called directly.
pub trait TemplateHelper<C: ?Sized, S, D>: Default {
    fn mark_dirty(&mut self)
    where
        C: 'static;
    fn clear_dirty(&mut self) -> bool
    where
        C: 'static;
    fn is_initialized(&self) -> bool;
    fn structure(&self) -> Option<&S>;
    fn component_rc(&self) -> Result<ComponentRc<C>, Error>
    where
        C: 'static + Sized;
    fn slot_scopes(&self) -> &SlotChildren<ForestTokenAddr, (ForestToken, D)>;
    fn pending_slot_changes(&mut self) -> Option<Vec<SlotChange<ForestToken, ()>>>;
    fn self_owner_weak(&self) -> &Box<dyn OwnerWeak>;
}

/// The template type
///
/// This struct is auto-managed by `#[component]` .
pub struct Template<C, S, D> {
    #[doc(hidden)]
    pub __m_self_owner_weak: Option<Box<dyn OwnerWeak>>,
    updater: Option<Box<dyn UpdateSchedulerWeak<EnterType = C>>>,
    dirty: bool,
    #[doc(hidden)]
    pub __m_structure: Option<S>,
    #[doc(hidden)]
    pub __m_slot_scopes: SlotChildren<ForestTokenAddr, (ForestToken, D)>,
    #[doc(hidden)]
    pub __m_pending_slot_changes: Option<Vec<SlotChange<ForestToken, ()>>>,
}

impl<C, S, D> Default for Template<C, S, D> {
    fn default() -> Self {
        Self {
            __m_self_owner_weak: None,
            updater: None,
            dirty: false,
            __m_structure: None,
            __m_slot_scopes: SlotChildren::None,
            __m_pending_slot_changes: None,
        }
    }
}

impl<C: 'static, S, D> Template<C, S, D> {
    #[inline]
    pub fn init(&mut self, init: TemplateInit<C>) {
        self.__m_self_owner_weak = Some(init.updater.to_owner_weak());
        self.updater = Some(init.updater);
    }
}

impl<C, S, D> TemplateHelper<C, S, D> for Template<C, S, D> {
    #[inline]
    fn mark_dirty(&mut self)
    where
        C: 'static,
    {
        if self.__m_structure.is_some() && !self.dirty {
            self.dirty = true;
        }
    }

    #[inline]
    fn clear_dirty(&mut self) -> bool
    where
        C: 'static,
    {
        if !self.dirty {
            return false;
        }
        self.dirty = false;
        true
    }

    #[inline]
    fn is_initialized(&self) -> bool {
        self.__m_structure.is_some()
    }

    #[inline]
    fn structure(&self) -> Option<&S> {
        self.__m_structure.as_ref()
    }

    #[inline]
    fn component_rc(&self) -> Result<ComponentRc<C>, Error>
    where
        C: 'static,
    {
        self.updater
            .as_ref()
            .and_then(|x| x.upgrade_scheduler())
            .map(|x| ComponentRc::new(x))
            .ok_or(Error::TreeNotCreated)
    }

    #[inline]
    fn slot_scopes(&self) -> &SlotChildren<ForestTokenAddr, (ForestToken, D)> {
        &self.__m_slot_scopes
    }

    #[inline]
    fn pending_slot_changes(&mut self) -> Option<Vec<SlotChange<ForestToken, ()>>> {
        self.__m_pending_slot_changes.take()
    }

    #[inline]
    fn self_owner_weak(&self) -> &Box<dyn OwnerWeak> {
        self.__m_self_owner_weak.as_ref().unwrap()
    }
}

/// A component template
///
/// It is auto-implemented by `#[component]` .
pub trait ComponentTemplate<B: Backend> {
    type TemplateField: TemplateHelper<Self, Self::TemplateStructure, Self::SlotData>;
    type TemplateStructure;
    type SlotData: 'static;

    /// Get a reference of the template field of the component
    fn template(&self) -> &Self::TemplateField;

    /// Get a mutable reference of the template field of the component
    fn template_mut(&mut self) -> &mut Self::TemplateField;

    /// Init a template
    fn template_init(&mut self, init: TemplateInit<Self>);

    /// Create a component within the specified shadow root
    fn template_create<'b>(
        &'b mut self,
        backend_context: &'b BackendContext<B>,
        backend_element: &'b mut ForestNodeMut<B::GeneralElement>,
        slot_fn: impl FnMut(&mut ForestNodeMut<B::GeneralElement>, &Self::SlotData) -> Result<(), Error>,
    ) -> Result<(), Error>
    where
        Self: Sized;

    /// Update a component
    fn template_update<'b>(
        &'b mut self,
        backend_context: &'b BackendContext<B>,
        backend_element: &'b mut ForestNodeMut<B::GeneralElement>,
        slot_fn: impl FnMut(
            SlotChange<&mut ForestNodeMut<B::GeneralElement>, &Self::SlotData>,
        ) -> Result<(), Error>,
    ) -> Result<(), Error>
    where
        Self: Sized;

    /// Update a component and store the slot changes
    #[inline]
    fn template_update_store_slot_changes<'b>(
        &'b mut self,
        backend_context: &'b BackendContext<B>,
        backend_element: &'b mut ForestNodeMut<B::GeneralElement>,
    ) -> Result<bool, Error>
    where
        Self: Sized
    {
        let mut slot_changes = Vec::with_capacity(0);
        self.template_update(backend_context, backend_element, |slot_change| {
            match slot_change {
                SlotChange::Unchanged(n, _) => slot_changes.push(SlotChange::Unchanged(n.rc().token(), ())),
                SlotChange::Added(n, _) => slot_changes.push(SlotChange::Added(n.rc().token(), ())),
                SlotChange::Removed(n) => slot_changes.push(SlotChange::Removed(n.rc().token()))
            }
            Ok(())
        })?;
        Ok(slot_changes.len() > 0)
    }

    /// Iterate slots
    #[inline]
    fn for_each_slot_scope<'b>(
        &'b mut self,
        backend_element: &'b mut ForestNodeMut<B::GeneralElement>,
        mut slot_fn: impl FnMut(
            SlotChange<&mut ForestNodeMut<B::GeneralElement>, &Self::SlotData>,
        ) -> Result<(), Error>,
    ) -> Result<(), Error> {
        for (_, (t, d)) in self.template_mut().slot_scopes().iter() {
            slot_fn(SlotChange::Unchanged(
                &mut backend_element.borrow_mut_token(t).ok_or(Error::TreeNodeReleased)?,
                d,
            ))?;
        }
        Ok(())
    }
}
