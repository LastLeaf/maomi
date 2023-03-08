//! Utilities for template management.
//! 
//! Most utilities in this module is used by `#[component]` .

use crate::{
    backend::{tree::*, Backend},
    component::*,
    error::Error,
    node::{OwnerWeak, SlotChange, SlotKindTrait},
    prop::Prop,
    BackendContext,
};

/// An init object for the template.
///
/// It is auto-managed by the `#[component]` .
/// Do not touch unless you know how it works exactly.
pub struct TemplateInit<C> {
    pub(crate) updater: ComponentWeak<C>,
}

/// Some helper functions for the template type.
pub trait TemplateHelper<C: ?Sized, S, L>: Default {
    /// Mark the template that update is needed.
    ///
    /// It is auto-managed by the `#[component]` .
    /// Do not touch unless you know how it works exactly.
    fn mark_dirty(&mut self)
    where
        C: 'static;

    /// Clear the mark.
    ///
    /// It is auto-managed by the `#[component]` .
    /// Do not touch unless you know how it works exactly.
    fn clear_dirty(&mut self) -> bool
    where
        C: 'static;

    /// Returns whether the template has been initialized.
    fn is_initialized(&self) -> bool;

    /// Get the template inner node tree.
    fn structure(&self) -> Option<&S>;

    /// Get the corresponding `ComponentRc` .
    fn component_rc(&self) -> Result<ComponentRc<C>, Error>
    where
        C: 'static + Sized;

    /// Get the corresponding `ComponentWeak` .
    fn component_weak(&self) -> Result<ComponentWeak<C>, Error>
    where
        C: 'static + Sized;

    #[doc(hidden)]
    fn slot_scopes(&self) -> &L;

    #[doc(hidden)]
    fn pending_slot_changes(
        &mut self,
        new_changes: Vec<SlotChange<(), ForestToken, ()>>,
    ) -> Vec<SlotChange<(), ForestToken, ()>>;

    /// Get the `OwnerWeak` of the current component.
    fn self_owner_weak(&self) -> &Box<dyn OwnerWeak>;
}

/// The template type.
///
/// It is auto-managed by the `#[component]` .
/// Do not touch unless you know how it works exactly.
pub struct Template<C, S, L: Default> {
    #[doc(hidden)]
    pub __m_self_owner_weak: Option<Box<dyn OwnerWeak>>,
    updater: Option<ComponentWeak<C>>,
    dirty: bool,
    #[doc(hidden)]
    pub __m_structure: Option<S>,
    #[doc(hidden)]
    pub __m_slot_scopes: L,
    #[doc(hidden)]
    pub __m_pending_slot_changes: Vec<SlotChange<(), ForestToken, ()>>,
}

impl<C, S, L: Default> Default for Template<C, S, L> {
    fn default() -> Self {
        Self {
            __m_self_owner_weak: None,
            updater: None,
            dirty: false,
            __m_structure: None,
            __m_slot_scopes: L::default(),
            __m_pending_slot_changes: Vec::with_capacity(0),
        }
    }
}

impl<C: 'static, S, L: Default> Template<C, S, L> {
    #[doc(hidden)]
    #[inline]
    pub fn init(&mut self, init: TemplateInit<C>) {
        self.__m_self_owner_weak = Some(init.updater.to_owner_weak());
        self.updater = Some(init.updater);
    }
}

impl<C, S, L: Default> TemplateHelper<C, S, L> for Template<C, S, L> {
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
            .and_then(|x| x.upgrade())
            .ok_or(Error::TreeNotCreated)
    }

    #[inline]
    fn component_weak(&self) -> Result<ComponentWeak<C>, Error>
    where
        C: 'static,
    {
        self.updater
            .as_ref()
            .map(|x| x.clone())
            .ok_or(Error::TreeNotCreated)
    }

    #[inline]
    fn slot_scopes(&self) -> &L {
        &self.__m_slot_scopes
    }

    #[inline]
    fn pending_slot_changes(
        &mut self,
        new_changes: Vec<SlotChange<(), ForestToken, ()>>,
    ) -> Vec<SlotChange<(), ForestToken, ()>> {
        std::mem::replace(&mut self.__m_pending_slot_changes, new_changes)
    }

    #[inline]
    fn self_owner_weak(&self) -> &Box<dyn OwnerWeak> {
        self.__m_self_owner_weak.as_ref().unwrap()
    }
}

/// The slot types which is associated with the template.
///
/// It is auto-managed by the `#[component]` .
/// Do not touch unless you know how it works exactly.
pub trait ComponentSlotKind {
    /// The slot list type.
    type SlotChildren<SlotContent>: SlotKindTrait<ForestTokenAddr, SlotContent>;

    /// The type of the slot data, specified through `#[component(SlotData = ...)]`.
    type SlotData: 'static;
}

/// A component template
///
/// It is auto-managed by the `#[component]` .
/// Do not touch unless you know how it works exactly.
pub trait ComponentTemplate<B: Backend>: ComponentSlotKind {
    /// The type of the template field.
    type TemplateField: TemplateHelper<
        Self,
        Self::TemplateStructure,
        Self::SlotChildren<(ForestToken, Prop<Self::SlotData>)>,
    >;

    /// The type of the template inner structure.
    type TemplateStructure;

    /// Get a reference of the template field of the component.
    fn template(&self) -> &Self::TemplateField;

    /// Get a mutable reference of the template field of the component.
    fn template_mut(&mut self) -> &mut Self::TemplateField;

    /// Initialize a template.
    fn template_init(&mut self, init: TemplateInit<Self>)
    where
        Self: Sized;

    /// Create a component within the specified shadow root.
    fn template_create_or_update<'b>(
        &'b mut self,
        backend_context: &'b BackendContext<B>,
        backend_element: &'b mut ForestNodeMut<B::GeneralElement>,
        slot_fn: &mut dyn FnMut(
            SlotChange<&mut ForestNodeMut<B::GeneralElement>, &ForestToken, &Self::SlotData>,
        ) -> Result<(), Error>,
    ) -> Result<(), Error>
    where
        Self: Sized;

    /// Update a component and store the slot changes.
    #[inline(never)]
    fn template_update_store_slot_changes<'b>(
        &'b mut self,
        backend_context: &'b BackendContext<B>,
        backend_element: &'b mut ForestNodeMut<B::GeneralElement>,
    ) -> Result<bool, Error>
    where
        Self: Sized,
    {
        let mut slot_changes: Vec<SlotChange<(), ForestToken, ()>> = Vec::with_capacity(0);
        self.template_create_or_update(backend_context, backend_element, &mut |slot_change| {
            match slot_change {
                SlotChange::Unchanged(..) => {}
                SlotChange::DataChanged(_, n, _) => {
                    slot_changes.push(SlotChange::DataChanged((), n.clone(), ()))
                }
                SlotChange::Added(_, n, _) => {
                    slot_changes.push(SlotChange::Added((), n.clone(), ()))
                }
                SlotChange::Removed(n) => slot_changes.push(SlotChange::Removed(n.clone())),
            }
            Ok(())
        })?;
        if slot_changes.len() > 0 {
            if self.template_mut().pending_slot_changes(slot_changes).len() > 0 {
                Err(Error::ListChangeWrong)?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Iterate over slots.
    #[inline(never)]
    fn for_each_slot_scope<'b>(
        &'b mut self,
        backend_element: &'b mut ForestNodeMut<B::GeneralElement>,
        slot_fn: &mut dyn FnMut(
            SlotChange<&mut ForestNodeMut<B::GeneralElement>, &ForestToken, &Self::SlotData>,
        ) -> Result<(), Error>,
    ) -> Result<(), Error> {
        for (t, d) in self.template_mut().slot_scopes().iter() {
            let n = &mut backend_element
                .borrow_mut_token(t)
                .ok_or(Error::TreeNodeReleased)?;
            slot_fn(SlotChange::Unchanged(n, t, d))?;
        }
        Ok(())
    }
}
