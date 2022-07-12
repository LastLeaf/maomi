use std::{collections::VecDeque, rc::{Rc, Weak}, cell::RefCell};

use super::{Backend, tree};
use crate::component::{StaticComponent, Component, ComponentTemplate};
use crate::mount_point::MountPoint;
use crate::error::Error;

pub(crate) enum BackendContextEvent<B: Backend> {
    General(Box<dyn FnOnce(&mut EnteredBackendContext<B>)>),
    Update(Rc<dyn StaticComponent<B>>),
}

/// A backend context for better backend management
pub struct BackendContext<B: Backend> {
    inner: Rc<BackendContextInner<B>>,
}

struct BackendContextInner<B: Backend> {
    entered: RefCell<EnteredBackendContext<B>>,
    event_queue: RefCell<VecDeque<BackendContextEvent<B>>>,
}

impl<B: Backend> BackendContext<B> {
    /// Create a new backend context
    pub fn new(backend: B) -> Self {
        let entered = RefCell::new(EnteredBackendContext {
            backend,
            ctx: None,
        });
        let inner = Rc::new(BackendContextInner {
            entered,
            event_queue: Default::default(),
        });
        let w = Rc::downgrade(&inner);
        inner.entered.borrow_mut().ctx = Some(w);
        Self { inner }
    }

    /// Enter the backend context
    ///
    /// If the backend context has already entered,
    /// it will wait until exits,
    /// so the `f` is required to be `'static` .
    pub fn enter(&self, f: impl 'static + FnOnce(&mut EnteredBackendContext<B>)) {
        if let Ok(mut entered) = self.inner.entered.try_borrow_mut() {
            f(&mut entered);
        } else {
            self.inner.event_queue.borrow_mut().push_back(BackendContextEvent::General(Box::new(f)));
        }
    }

    /// Try enter the backend context sync
    ///
    /// If the backend context has already entered, an `Err` is returned.
    pub fn enter_sync<T, F>(&self, f: F) -> Result<T, F>
    where
        F: FnOnce(&mut EnteredBackendContext<B>) -> T {
        if let Ok(mut entered) = self.inner.entered.try_borrow_mut() {
            Ok(f(&mut entered))
        } else {
            Err(f)
        }
    }
}

/// An entered backend context
pub struct EnteredBackendContext<B: Backend> {
    backend: B,
    ctx: Option<Weak<BackendContextInner<B>>>,
}

impl<B: Backend> EnteredBackendContext<B> {
    /// Create a mount point
    ///
    /// The `init` provides a way to do some updates before the component `created` lifetime.
    /// It is encouraged to change template data bindings in `init` .
    pub fn new_mount_point<C: Component + ComponentTemplate<B>>(
        &mut self,
        init: impl FnOnce(&mut C) -> Result<(), Error>,
    ) -> Result<MountPoint<B, C>, Error> {
        let mut owner = self.backend.root_mut();
        MountPoint::new_in_backend(&mut owner, init)
    }

    /// Get the root backend element
    pub fn root_mut(&mut self) -> tree::ForestNodeMut<B::GeneralElement> {
        self.backend.root_mut()
    }
}
