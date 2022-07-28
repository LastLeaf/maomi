use crate::{
    component::*,
    backend::{tree::*, Backend},
    diff::ListItemChange,
    error::Error,
    node::SlotChildren,
    BackendContext,
};

/// An init object for the template
///
/// This struct is auto-managed by `#[component]` .
pub struct TemplateInit<C: ?Sized> {
    pub(crate) updater: Box<dyn UpdateSchedulerWeak<EnterType = C>>,
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
    fn slot_scopes(&self) -> &SlotChildren<(ForestToken, D)>;
}

/// The template type
///
/// This struct is auto-managed by `#[component]` .
pub struct Template<C, S, D> {
    updater: Option<Box<dyn UpdateSchedulerWeak<EnterType = C>>>,
    dirty: bool,
    /// The template node tree structure
    ///
    /// Caution: do not modify anything inside node tree unless you really understand how templates works.
    pub structure: Option<S>,
    /// The slot scope data
    ///
    /// Caution: do not modify anything inside node tree unless you really understand how templates works.
    pub slot_scopes: SlotChildren<(ForestToken, D)>,
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
            self.dirty = true;
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
            .map(|x| ComponentRc::new(x))
            .ok_or(Error::TreeNotCreated)
    }

    fn slot_scopes(&self) -> &SlotChildren<(ForestToken, D)> {
        &self.slot_scopes
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
        backend_element: &'b mut ForestNodeMut<B::GeneralElement>,
        slot_fn: impl FnMut(
            &mut ForestNodeMut<B::GeneralElement>,
            &Self::SlotData,
        ) -> Result<R, Error>,
    ) -> Result<SlotChildren<R>, Error>
    where
        Self: Sized;

    /// Create a component within the specified shadow root
    fn template_update<'b>(
        &'b mut self,
        backend_context: &'b BackendContext<B>,
        backend_element: &'b mut ForestNodeMut<B::GeneralElement>,
        slot_fn: impl FnMut(
            ListItemChange<&mut ForestNodeMut<B::GeneralElement>, &Self::SlotData>,
        ) -> Result<(), Error>,
    ) -> Result<(), Error>
    where
        Self: Sized;

    /// Iterate slots
    fn for_each_slot_scope<'b>(
        &'b mut self,
        backend_element: &'b mut ForestNodeMut<B::GeneralElement>,
        mut slot_fn: impl FnMut(
            ListItemChange<&mut ForestNodeMut<B::GeneralElement>, &Self::SlotData>,
        ) -> Result<(), Error>,
    ) -> Result<(), Error> {
        for (t, d) in self.template_mut().slot_scopes() {
            slot_fn(ListItemChange::Unchanged(
                &mut backend_element.borrow_mut_token(t),
                d,
            ))?;
        }
        Ok(())
    }
}
