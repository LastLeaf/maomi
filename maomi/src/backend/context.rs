//! Utility types for backends.

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
use crate::mount_point::{MountPoint, DynMountPoint};
use crate::template::ComponentTemplate;

/// A future that can be resolved with a callback function.
///
/// This type implements `Future` .
/// It can convert a callback-style interface into a `Future` .
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
    /// Create with a function which can resolve the future later.
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

/// A wrapper type for a backend.
/// 
/// The wrapped backend cannot be visited directly.
/// This is because the backend might be visited in multiple async tasks.
/// When a task want to visit the backend,
/// `BackendContext::enter` or `BackendContext::enter_sync` should be used.
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

impl<B: Backend> BackendContext<B> {
    /// Create a new backend context.
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

    /// Get the current backend stage.
    /// 
    /// This is meaningful only when prerendering is used.
    pub fn initial_backend_stage(&self) -> BackendStage {
        self.inner.initial_backend_stage.get()
    }

    fn generate_async_task(&self) {
        let inner = self.inner.clone();
        B::async_task(async move {
            let entered = &mut inner.entered.borrow_mut();
            loop {
                let ev = inner.event_queue.borrow_mut().pop_front();
                if let Some(ev) = ev {
                    match ev {
                        BackendContextEvent::General(f) => {
                            f(entered);
                        }
                    }
                } else {
                    break;
                }
            }
        });
    }

    /// Get the underlying backend synchronously.
    ///
    /// If the backend context is visited by other async tasks,
    /// it will wait until available.
    /// 
    /// The backend is always be visited asynchronously,
    /// so the `f` is required to be `'static` .
    #[inline]
    pub fn enter<T: 'static, F>(&self, f: F) -> AsyncCallback<T>
    where
        F: 'static + FnOnce(&mut EnteredBackendContext<B>) -> T,
    {
        let (fut, cb) = AsyncCallback::new();
        let need_task = {
            let event_queue = &mut self.inner.event_queue.borrow_mut();
            event_queue.push_back(BackendContextEvent::General(Box::new(move |x| {
                cb(f(x));
            })));
            event_queue.len() == 1
        };
        if need_task && self.inner.entered.try_borrow_mut().is_ok() {
            self.generate_async_task();
        }
        fut
    }

    /// Get the underlying backend asynchronously.
    ///
    /// If the backend context is still being visited, an `Err` is returned.
    #[inline]
    pub fn enter_sync<T, F>(&self, f: F) -> Result<T, F>
    where
        F: FnOnce(&mut EnteredBackendContext<B>) -> T,
    {
        if let Ok(mut entered) = self.inner.entered.try_borrow_mut() {
            let need_task = self.inner.event_queue.borrow().is_empty();
            let ret = f(&mut entered);
            if need_task && !self.inner.event_queue.borrow().is_empty() {
                self.generate_async_task();
            }
            Ok(ret)
        } else {
            Err(f)
        }
    }

    /// Get the prerendering data of a prerenderable component.
    ///
    /// The `QueryData` should be provided to the `PrerenderableComponent` .
    #[cfg(any(feature = "prerendering", feature = "prerendering-apply"))]
    pub async fn prerendering_data<C: PrerenderableComponent>(
        query_data: &C::QueryData,
    ) -> PrerenderingData<C> {
        PrerenderingData::new(C::prerendering_data(query_data).await)
    }
}

/// A mutable reference to a backend context.
pub struct EnteredBackendContext<B: Backend> {
    backend: B,
    ctx: Option<Weak<BackendContextInner<B>>>,
}

impl<B: Backend> EnteredBackendContext<B> {
    /// Create a mount point.
    ///
    /// A component should be specified as the root component.
    /// The `init` provides a way to do some updates before the component `created` lifetime.
    pub fn attach<C: Component + ComponentTemplate<B>>(
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

    /// Create a mount point and apply the prerendering data.
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

    /// Detach a mount point.
    pub fn detach<C: Component + ComponentTemplate<B>>(
        &mut self,
        mount_point: &mut MountPoint<B, C>,
    ) {
        let mut root = self.backend.root_mut();
        mount_point.detach(&mut root);
    }

    /// Detach a mount point with its `dyn` form.
    pub fn detach_dyn(
        &mut self,
        mount_point: &mut DynMountPoint<B>
    ) {
        let mut root = self.backend.root_mut();
        mount_point.detach(&mut root);
    }

    /// Get the root component of a mount point.
    #[inline]
    pub fn root_component_with<C: Component + ComponentTemplate<B>, R>(
        &mut self,
        mount_point: &MountPoint<B, C>,
        f: impl FnOnce(&mut C) -> R,
    ) -> R {
        let n = mount_point.root_component();
        f(&mut n.component().borrow_mut())
    }

    /// Get the root backend element.
    #[inline]
    pub fn root(&self) -> tree::ForestNode<B::GeneralElement> {
        self.backend.root()
    }

    /// Get the root backend element.
    #[inline]
    pub fn root_mut(&mut self) -> tree::ForestNodeMut<B::GeneralElement> {
        self.backend.root_mut()
    }
}

impl<B: Backend> std::ops::Deref for EnteredBackendContext<B> {
    type Target = B;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.backend
    }
}

impl<B: Backend> std::ops::DerefMut for EnteredBackendContext<B> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.backend
    }
}

/// A helper for the prerendering data.
#[cfg(any(feature = "prerendering", feature = "prerendering-apply"))]
pub struct PrerenderingData<C: PrerenderableComponent> {
    data: C::PrerenderingData,
}

#[cfg(any(feature = "prerendering", feature = "prerendering-apply"))]
impl<C: PrerenderableComponent> PrerenderingData<C> {
    /// Wrap the prerendering data.
    pub fn new(data: C::PrerenderingData) -> Self {
        Self { data }
    }

    /// Get the underlying prerendering data.
    pub fn get(&self) -> &C::PrerenderingData {
        &self.data
    }

    /// Unwrap the data.
    pub fn unwrap(self) -> C::PrerenderingData {
        self.data
    }
}
