use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::{Rc, Weak},
};

use super::{tree, Backend};
use crate::component::{Component, ComponentTemplate};
use crate::error::Error;
use crate::mount_point::MountPoint;

pub(crate) enum BackendContextEvent<B: Backend> {
    General(Box<dyn FnOnce(&mut EnteredBackendContext<B>) -> Result<(), Error>>),
}

/// A backend context for better backend management
pub struct BackendContext<B: Backend> {
    inner: Rc<BackendContextInner<B>>,
}

struct BackendContextInner<B: Backend> {
    entered: RefCell<EnteredBackendContext<B>>,
    event_queue: RefCell<VecDeque<BackendContextEvent<B>>>,
}

impl<B: Backend> Clone for BackendContext<B> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<B: Backend> BackendContext<B> {
    /// Create a new backend context
    pub fn new(backend: B) -> Self {
        let entered = RefCell::new(EnteredBackendContext { backend, ctx: None });
        let inner = Rc::new(BackendContextInner {
            entered,
            event_queue: Default::default(),
        });
        let w = Rc::downgrade(&inner);
        inner.entered.borrow_mut().ctx = Some(w);
        Self { inner }
    }

    fn exec_queue(&self, entered: &mut EnteredBackendContext<B>) {
        while let Some(ev) = self.inner.event_queue.borrow_mut().pop_front() {
            match ev {
                BackendContextEvent::General(f) => {
                    if let Err(err) = f(entered) {
                        log::error!("{}", err);
                    }
                }
            }
        }
    }

    /// Enter the backend context
    ///
    /// If the backend context has already entered,
    /// it will wait until exits,
    /// so the `f` is required to be `'static` .
    #[inline]
    pub fn enter(
        &self,
        f: impl 'static + FnOnce(&mut EnteredBackendContext<B>) -> Result<(), Error>,
    ) {
        if let Ok(mut entered) = self.inner.entered.try_borrow_mut() {
            if let Err(err) = f(&mut entered) {
                log::error!("{}", err);
            }
            self.exec_queue(&mut entered);
        } else {
            self.inner
                .event_queue
                .borrow_mut()
                .push_back(BackendContextEvent::General(Box::new(f)));
        }
    }

    /// Try enter the backend context sync
    ///
    /// If the backend context has already entered, an `Err` is returned.
    #[inline]
    pub fn enter_sync<T, F>(&self, f: F) -> Result<T, F>
    where
        F: FnOnce(&mut EnteredBackendContext<B>) -> T,
    {
        if let Ok(mut entered) = self.inner.entered.try_borrow_mut() {
            let ret = f(&mut entered);
            self.exec_queue(&mut entered);
            Ok(ret)
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
    pub fn new_mount_point<C: Component + ComponentTemplate<B, C> + 'static>(
        &mut self,
        init: impl FnOnce(&mut C) -> Result<(), Error>,
    ) -> Result<MountPoint<B, C>, Error> {
        let mut owner = self.backend.root_mut();
        MountPoint::new_in_backend(
            &BackendContext {
                inner: self.ctx.as_ref().unwrap().upgrade().unwrap(),
            },
            &mut owner,
            init,
        )
    }

    /// Get the root backend element
    #[inline]
    pub fn root(&self) -> tree::ForestNode<B::GeneralElement> {
        self.backend.root()
    }

    /// Get the root backend element
    #[inline]
    pub fn root_mut(&mut self) -> tree::ForestNodeMut<B::GeneralElement> {
        self.backend.root_mut()
    }
}
