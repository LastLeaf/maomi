use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    future::Future,
    pin::Pin,
    rc::{Rc, Weak},
    task::{Context, Poll, Waker},
};

use super::{tree, Backend, BackendStage};
use crate::component::Component;
#[cfg(any(feature = "prerendering", feature = "prerendering-apply"))]
use crate::component::PrerenderableComponent;
use crate::error::Error;
use crate::mount_point::MountPoint;
use crate::template::ComponentTemplate;

/// A future that can be resolved with a callback function
#[must_use]
pub struct AsyncCallback<R: 'static> {
    done: Rc<Cell<Option<R>>>,
    waker: Rc<Cell<Option<Waker>>>,
}

impl<R: 'static> Future for AsyncCallback<R> {
    type Output = R;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if let Some(ret) = self.done.take() {
            Poll::Ready(ret)
        } else {
            self.waker.set(Some(cx.waker().clone()));
            Poll::Pending
        }
    }
}

impl<R: 'static> AsyncCallback<R> {
    /// Create with a function which can resolve the future later
    pub fn new() -> (Self, impl 'static + FnOnce(R)) {
        let done = Rc::new(Cell::new(None));
        let done2 = done.clone();
        let waker = Rc::new(Cell::new(None));
        let waker2 = waker.clone();
        let callback = move |ret| {
            done2.set(Some(ret));
            match waker2.take() {
                Some(waker) => {
                    let waker: Waker = waker;
                    waker.wake();
                }
                None => {}
            }
        };
        (Self { done, waker }, callback)
    }
}

pub(crate) enum BackendContextEvent<B: Backend> {
    General(Box<dyn FnOnce(&mut EnteredBackendContext<B>)>),
}

/// A backend context for better backend management
pub struct BackendContext<B: Backend> {
    inner: Rc<BackendContextInner<B>>,
}

struct BackendContextInner<B: Backend> {
    initial_backend_stage: Cell<BackendStage>,
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

// TODO impl better entering with backend-provided async task

impl<B: Backend> BackendContext<B> {
    /// Create a new backend context
    pub fn new(backend: B) -> Self {
        let initial_backend_stage = Cell::new(backend.backend_stage());
        let entered = RefCell::new(EnteredBackendContext { backend, ctx: None });
        let inner = Rc::new(BackendContextInner {
            initial_backend_stage,
            entered,
            event_queue: Default::default(),
        });
        let w = Rc::downgrade(&inner);
        inner.entered.borrow_mut().ctx = Some(w);
        Self { inner }
    }

    /// Get the current backend stage
    pub fn initial_backend_stage(&self) -> BackendStage {
        self.inner.initial_backend_stage.get()
    }

    fn exec_queue(&self, entered: &mut EnteredBackendContext<B>) {
        while let Some(ev) = self.inner.event_queue.borrow_mut().pop_front() {
            match ev {
                BackendContextEvent::General(f) => {
                    f(entered);
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
    pub fn enter<T: 'static, F>(&self, f: F) -> AsyncCallback<T>
    where
        F: 'static + FnOnce(&mut EnteredBackendContext<B>) -> T,
    {
        let (fut, cb) = AsyncCallback::new();
        if let Ok(mut entered) = self.inner.entered.try_borrow_mut() {
            let ret = f(&mut entered);
            self.exec_queue(&mut entered);
            cb(ret);
        } else {
            self.inner
                .event_queue
                .borrow_mut()
                .push_back(BackendContextEvent::General(Box::new(move |x| {
                    cb(f(x));
                })));
        }
        fut
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

    /// Get the prerendering data of a component
    ///
    /// The `QueryData` should be provided to the `PrerenderableComponent` .
    #[cfg(any(feature = "prerendering", feature = "prerendering-apply"))]
    pub async fn prerendering_data<C: PrerenderableComponent>(
        &self,
        query_data: &C::QueryData,
    ) -> PrerenderingData<C> {
        PrerenderingData::new(C::prerendering_data(query_data).await)
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
    pub fn attach<C: Component + ComponentTemplate<B> + 'static>(
        &mut self,
        init: impl FnOnce(&mut C),
    ) -> Result<MountPoint<B, C>, Error> {
        let mut root = self.backend.root_mut();
        MountPoint::attach(
            &BackendContext {
                inner: self.ctx.as_ref().unwrap().upgrade().unwrap(),
            },
            &mut root,
            init,
        )
    }

    /// Create a mount point and apply the prerendering data
    #[cfg(any(feature = "prerendering", feature = "prerendering-apply"))]
    pub fn prerendering_attach<C: PrerenderableComponent + ComponentTemplate<B> + 'static>(
        &mut self,
        prerendering_data: PrerenderingData<C>,
    ) -> Result<MountPoint<B, C>, Error> {
        let mut root = self.backend.root_mut();
        MountPoint::attach(
            &BackendContext {
                inner: self.ctx.as_ref().unwrap().upgrade().unwrap(),
            },
            &mut root,
            |comp| {
                PrerenderableComponent::apply_prerendering_data(comp, prerendering_data.data);
            },
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

impl<B: Backend> std::ops::Deref for EnteredBackendContext<B> {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        &self.backend
    }
}

impl<B: Backend> std::ops::DerefMut for EnteredBackendContext<B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.backend
    }
}

/// A helper for the prerendering data
#[cfg(any(feature = "prerendering", feature = "prerendering-apply"))]
pub struct PrerenderingData<C: PrerenderableComponent> {
    data: C::PrerenderingData,
}

#[cfg(any(feature = "prerendering", feature = "prerendering-apply"))]
impl<C: PrerenderableComponent> PrerenderingData<C> {
    /// Wrap the prerendering data
    pub fn new(data: C::PrerenderingData) -> Self {
        Self { data }
    }

    /// Get the underlying prerendering data
    pub fn get(&self) -> &C::PrerenderingData {
        &self.data
    }
}
