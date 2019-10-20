use std::rc::Rc;
use std::ops::{Deref, DerefMut};
use me_cell::MeRefHandle;

use super::backend::{Backend, BackendNode};
use super::node::*;

pub trait Component: ComponentTemplate + 'static {
    fn new() -> Self where Self: Sized;
    // TODO impl lifetimes
    fn created<B: Backend>(_: &mut ComponentRefMut<B, Self>) where Self: Sized {

    }
    fn attached<B: Backend>(_: &mut ComponentRefMut<B, Self>) where Self: Sized {

    }
    fn ready<B: Backend>(_: &mut ComponentRefMut<B, Self>) where Self: Sized {

    }
    fn moved<B: Backend>(_: &mut ComponentRefMut<B, Self>) where Self: Sized {

    }
    fn detached<B: Backend>(_: &mut ComponentRefMut<B, Self>) where Self: Sized {

    }
}
pub trait ComponentTemplate {
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

pub struct ComponentRc<B: Backend, C: Component> {
    n: ComponentNodeRc<B>,
    phantom_data: std::marker::PhantomData<C>,
}
impl<B: Backend, C: Component> ComponentRc<B, C> {
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
    pub fn backend_element(&self) -> &<<B as Backend>::BackendNode as BackendNode>::BackendElement {
        &self.n.backend_element
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
    pub fn backend_element(&self) -> &<<B as Backend>::BackendNode as BackendNode>::BackendElement {
        &self.n.backend_element
    }
    pub fn update<F: FnOnce(&mut C)>(&mut self, f: F) {
        let c = self.n.component.downcast_mut().unwrap();
        f(c);
        self.apply_updates();
    }
    pub fn apply_updates(&mut self) {
        self.n.apply_updates::<C>();
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
    // empty
}
impl Component for EmptyComponent {
    fn new() -> Self {
        Self {
            // empty
        }
    }
}
impl ComponentTemplate for EmptyComponent {
    // empty
}
