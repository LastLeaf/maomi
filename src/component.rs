use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};
use me_cell::MeRefHandle;

use super::context::Scheduler;
use super::node::*;
use super::backend::{Backend, BackendNode};

pub trait Component: ComponentTemplate + downcast_rs::Downcast {
    fn new(c: Rc<ComponentContext>) -> Self where Self: Sized;
    fn created<B: Backend>(&mut self, _weak: ComponentWeak<B, Self>) where Self: Sized {

    }
    fn attached(&mut self) {

    }
    fn moved(&mut self) {

    }
    fn detached(&mut self) {

    }
}
downcast_rs::impl_downcast!(Component);

pub trait ComponentTemplate: 'static {
    fn template<B: Backend>(component: &mut ComponentNodeRefMut<B>, is_update: bool) -> Option<Vec<NodeRc<B>>> where Self: Sized {
        if is_update {
            return None
        }
        let mut f = || {
            vec![component.new_virtual_node("slot", VirtualNodeProperty::Slot("", vec![]), vec![]).into()]
        };
        Some(f())
    }
}

pub struct ComponentContext {
    need_update: Cell<bool>,
    scheduler: Rc<Scheduler>,
}
impl ComponentContext {
    pub(crate) fn new(scheduler: Rc<Scheduler>) -> Self {
        Self {
            need_update: Cell::new(false),
            scheduler,
        }
    }
    pub(crate) fn scheduler(&self) -> &Rc<Scheduler> {
        &self.scheduler
    }
    fn clear_update(&self) -> bool {
        self.need_update.replace(false)
    }
    pub fn update(&self) {
        self.need_update.set(true);
    }
    pub fn update_then<B: Backend, C: Component, F: 'static + FnOnce(&mut ComponentRefMut<B, C>)>(&self, callback: F) {
        self.need_update.set(true);
        self.scheduler.add_task(|| {
            // TODO
        });
    }
    pub fn tick<B: Backend, F: 'static + FnOnce(&mut ComponentRefMut<B, Self>)>(&self, f: F) {
        // TODO
    }
    pub fn next_tick<B: Backend, F: 'static + FnOnce(&mut ComponentRefMut<B, Self>)>(&self, f: F) {
        // TODO
    }
}

pub struct ComponentRc<B: Backend, C: Component> {
    n: ComponentNodeRc<B>,
    phantom_data: std::marker::PhantomData<C>,
}
impl<B: Backend, C: Component> ComponentRc<B, C> {
    pub fn as_node(&self) -> &ComponentNodeRc<B> {
        &self.n
    }
    pub fn borrow<'a>(&'a self) -> ComponentRef<'a, B, C> {
        self.n.borrow().with_type::<C>()
    }
    pub fn borrow_mut<'a>(&'a self) -> ComponentRefMut<'a, B, C> {
        self.n.borrow_mut().with_type::<C>()
    }
    pub fn borrow_with<'a: 'b, 'b, U>(&'b self, source: &'b U) -> ComponentRef<'b, B, C> where U: ElementRef<'a, B> {
        self.n.borrow_with(source).with_type::<C>()
    }
    pub fn borrow_mut_with<'a: 'b, 'b, U>(&'b self, source: &'b mut U) -> ComponentRefMut<'b, B, C> where U: ElementRefMut<'a, B> {
        self.n.borrow_mut_with(source).with_type::<C>()
    }
    pub unsafe fn borrow_mut_unsafe_with<'a: 'b, 'b, 'c, U>(&'c self, source: &'b mut U) -> ComponentRefMut<'c, B, C> where U: ElementRefMut<'a, B> {
        self.n.borrow_mut_unsafe_with(source).with_type::<C>()
    }
    pub fn downgrade(&self) -> ComponentWeak<B, C> {
        self.n.downgrade().with_type::<C>()
    }
}
impl<B: Backend, C: Component> From<ComponentNodeRc<B>> for ComponentRc<B, C> {
    fn from(n: ComponentNodeRc<B>) -> Self {
        Self {
            n,
            phantom_data: std::marker::PhantomData
        }
    }
}

pub struct ComponentWeak<B: Backend, C: Component> {
    n: ComponentNodeWeak<B>,
    phantom_data: std::marker::PhantomData<C>,
}
impl<B: Backend, C: Component> ComponentWeak<B, C> {
    pub fn as_node(&self) -> &ComponentNodeWeak<B> {
        &self.n
    }
    pub fn upgrade(&self) -> Option<ComponentRc<B, C>> {
        self.n.upgrade().map(|x| {
            x.with_type::<C>()
        })
    }
}
impl<B: Backend, C: Component> From<ComponentNodeWeak<B>> for ComponentWeak<B, C> {
    fn from(n: ComponentNodeWeak<B>) -> Self {
        Self {
            n,
            phantom_data: std::marker::PhantomData
        }
    }
}

pub struct ComponentRef<'a, B: Backend, C: Component> {
    n: ComponentNodeRef<'a, B>,
    phantom_data: std::marker::PhantomData<C>,
}
impl<'a, B: Backend, C: Component> ComponentRef<'a, B, C> {
    pub fn as_node(&self) -> &ComponentNodeRef<'a, B> {
        &self.n
    }
    pub fn backend_element(&self) -> &<<B as Backend>::BackendNode as BackendNode>::BackendElement {
        self.n.backend_element()
    }
}
impl<'a, B: Backend, C: Component> Deref for ComponentRef<'a, B, C> {
    type Target = C;
    fn deref(&self) -> &C {
        self.n.as_component()
    }
}
impl<'a, B: Backend, C: Component> ElementRef<'a, B> for ComponentRef<'a, B, C> {
    fn backend(&self) -> &Rc<B> {
        self.n.backend()
    }
    fn as_me_ref_handle(&self) -> &MeRefHandle<'a> {
        self.n.as_me_ref_handle()
    }
    fn as_node_ref<'b>(self) -> NodeRef<'b, B> where 'a: 'b {
        self.n.as_node_ref()
    }
}
impl<'a, B: Backend, C: Component> From<ComponentNodeRef<'a, B>> for ComponentRef<'a, B, C> {
    fn from(n: ComponentNodeRef<'a, B>) -> Self {
        Self {
            n,
            phantom_data: std::marker::PhantomData
        }
    }
}

pub struct ComponentRefMut<'a, B: Backend, C: Component> {
    n: ComponentNodeRefMut<'a, B>,
    phantom_data: std::marker::PhantomData<C>,
}
impl<'a, B: Backend, C: Component> ComponentRefMut<'a, B, C> {
    pub fn as_node(&mut self) -> &mut ComponentNodeRefMut<'a, B> {
        &mut self.n
    }
    pub fn to_ref<'b>(&'b self) -> ComponentRef<'b, B, C> where 'a: 'b {
        self.n.to_ref().into()
    }
    pub fn backend_element(&self) -> &<<B as Backend>::BackendNode as BackendNode>::BackendElement {
        self.n.backend_element()
    }
    pub fn apply_updates(&mut self) {
        if self.n.ctx.clear_update() {
            self.n.apply_updates::<C>();
        }
    }
    pub fn force_apply_updates(&mut self) {
        self.n.ctx.clear_update();
        self.n.apply_updates::<C>();
    }
    pub fn owner(&self) -> Option<ComponentNodeRc<B>> {
        self.n.owner()
    }
    pub fn sub_node(&self) -> Option<NodeRc<B>> {
        unimplemented!()
    }
}
impl<'a, B: Backend, C: Component> Drop for ComponentRefMut<'a, B, C> {
    fn drop(&mut self) {
        self.apply_updates();
    }
}
impl<'a, B: Backend, C: Component> Deref for ComponentRefMut<'a, B, C> {
    type Target = C;
    fn deref(&self) -> &C {
        self.n.as_component()
    }
}
impl<'a, B: Backend, C: Component> DerefMut for ComponentRefMut<'a, B, C> {
    fn deref_mut(&mut self) -> &mut C {
        self.n.as_component_mut()
    }
}
impl<'a, B: Backend, C: Component> From<ComponentNodeRefMut<'a, B>> for ComponentRefMut<'a, B, C> {
    fn from(n: ComponentNodeRefMut<'a, B>) -> Self {
        Self {
            n,
            phantom_data: std::marker::PhantomData
        }
    }
}

pub struct EmptyComponent {
    ctx: Rc<ComponentContext>
}
impl Component for EmptyComponent {
    fn new(ctx: Rc<ComponentContext>) -> Self {
        Self {
            ctx
        }
    }
}
impl ComponentTemplate for EmptyComponent {
    // empty
}
