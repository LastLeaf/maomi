use std::pin::Pin;
use std::marker::PhantomData;
use std::fmt;
use std::rc::Rc;
use std::cell::{RefCell};
use std::ops::{Deref, DerefMut};
use futures::Future;
use me_cell::{MeRefHandle, MeRefMutHandle};

use super::context::Scheduler;
use super::node::*;
use super::backend::{Backend};

pub trait Component<B: Backend>: ComponentTemplate<B> + downcast_rs::Downcast {
    fn new(_ctx: ComponentContext<B, Self>) -> Self where Self: Sized;
    fn created(&mut self) {

    }
    fn attached(&mut self) {

    }
    fn moved(&mut self) {

    }
    fn detached(&mut self) {

    }
}
downcast_rs::impl_downcast!(Component<B> where B: Backend);

pub trait PrerenderableComponent<B: Backend>: Component<B> {
    type Args;
    type PrerenderedData: 'static + serde::Serialize + for<'de> serde::Deserialize<'de>;
    type MetaData;
    fn get_prerendered_data(&self, args: Self::Args) -> Pin<Box<dyn Future<Output = (Self::PrerenderedData, Self::MetaData)>>>;
    fn apply_prerendered_data(&mut self, _data: &Self::PrerenderedData);
}

pub enum ComponentTemplateOperation {
    Init,
    Update,
}

pub trait ComponentTemplate<B: Backend>: 'static {
    fn template(component: &mut ComponentNodeRefMut<B>, operation: ComponentTemplateOperation) -> Option<Vec<NodeRc<B>>> where Self: Sized {
        if let ComponentTemplateOperation::Update = operation {
            return None
        }
        let mut f = || {
            vec![component.new_virtual_node("slot", VirtualNodeProperty::Slot("", vec![]), vec![]).into()]
        };
        Some(f())
    }
    fn template_skin() -> &'static str where Self: Sized {
        ""
    }
}

#[derive(Clone)]
pub struct ComponentContext<B: Backend, C: Component<B>> {
    node_weak: ComponentWeak<B, C>,
    need_update: Rc<RefCell<Vec<Box<dyn 'static + FnOnce(&mut ComponentNodeRefMut<B>)>>>>,
    scheduler: Rc<Scheduler>,
    phantom_data: PhantomData<C>,
}
impl<B: Backend, C: Component<B>> ComponentContext<B, C> {
    pub(crate) fn new(node_weak: ComponentWeak<B, C>, need_update: Rc<RefCell<Vec<Box<dyn 'static + FnOnce(&mut ComponentNodeRefMut<B>)>>>>, scheduler: Rc<Scheduler>) -> Self {
        Self {
            node_weak,
            need_update,
            scheduler,
            phantom_data: PhantomData,
        }
    }
    fn add_updater(v: &mut Vec<Box<dyn 'static + FnOnce(&mut ComponentNodeRefMut<B>)>>) {
        v.push(Box::new(|c: &mut ComponentNodeRefMut<B>| {
            <C as ComponentTemplate<B>>::template(c, ComponentTemplateOperation::Update);
        }));
    }
    pub fn update(&self) {
        let mut update_callbacks = self.need_update.borrow_mut();
        if update_callbacks.len() == 0 {
            Self::add_updater(&mut update_callbacks);
        }
    }
    pub fn update_then<F: 'static + FnOnce(&mut ComponentRefMut<B, C>)>(&self, callback: F) {
        let mut update_callbacks = self.need_update.borrow_mut();
        if update_callbacks.len() == 0 {
            Self::add_updater(&mut update_callbacks);
        }
        update_callbacks.push(Box::new(|x| {
            callback(&mut x.duplicate().with_type::<C>());
        }));
    }
    pub fn tick<F: 'static + FnOnce(&mut ComponentRefMut<B, C>)>(&self, f: F) {
        let rc = self.node_weak.upgrade().unwrap();
        self.scheduler.add_task(move || {
            f(&mut rc.borrow_mut());
        });
    }
    pub fn tick_with_component_rc<F: 'static + FnOnce(ComponentRc<B, C>)>(&self, f: F) {
        let rc = self.node_weak.upgrade().unwrap();
        self.scheduler.add_task(move || {
            f(rc);
        });
    }
}

#[derive(Clone)]
pub struct ComponentRc<B: Backend, C: Component<B>> {
    n: ComponentNodeRc<B>,
    phantom_data: std::marker::PhantomData<C>,
}
impl<B: Backend, C: Component<B>> ComponentRc<B, C> {
    pub fn into_node(self) -> ComponentNodeRc<B> {
        self.n
    }
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
impl<B: Backend, C: Component<B>> From<ComponentNodeRc<B>> for ComponentRc<B, C> {
    fn from(n: ComponentNodeRc<B>) -> Self {
        Self {
            n,
            phantom_data: std::marker::PhantomData
        }
    }
}

#[derive(Clone)]
pub struct ComponentWeak<B: Backend, C: Component<B>> {
    n: ComponentNodeWeak<B>,
    phantom_data: std::marker::PhantomData<C>,
}
impl<B: Backend, C: Component<B>> ComponentWeak<B, C> {
    pub fn into_node(self) -> ComponentNodeWeak<B> {
        self.n
    }
    pub fn as_node(&self) -> &ComponentNodeWeak<B> {
        &self.n
    }
    pub fn upgrade(&self) -> Option<ComponentRc<B, C>> {
        self.n.upgrade().map(|x| {
            x.with_type::<C>()
        })
    }
}
impl<B: Backend, C: Component<B>> From<ComponentNodeWeak<B>> for ComponentWeak<B, C> {
    fn from(n: ComponentNodeWeak<B>) -> Self {
        Self {
            n,
            phantom_data: std::marker::PhantomData
        }
    }
}

pub struct ComponentRef<'a, B: Backend, C: Component<B>> {
    n: ComponentNodeRef<'a, B>,
    phantom_data: std::marker::PhantomData<C>,
}
impl<'a, B: Backend, C: Component<B>> ComponentRef<'a, B, C> {
    pub fn duplicate<'b>(&'b mut self) -> ComponentRef<'b, B, C> {
        ComponentRef {
            n: self.n.duplicate(),
            phantom_data: PhantomData,
        }
    }
    pub fn check_type(&self) -> bool {
        self.n.is_type::<C>()
    }
    pub fn into_node(self) -> ComponentNodeRef<'a, B> {
        self.n
    }
    pub fn as_node(&self) -> &ComponentNodeRef<'a, B> {
        &self.n
    }
    pub fn backend_element(&self) -> &B::BackendElement {
        self.n.backend_element()
    }
    pub fn owner(&self) -> Option<ComponentNodeRc<B>> {
        self.n.owner()
    }
    pub fn marked(&self, r: &str) -> Option<NodeRc<B>> {
        self.n.marked(r)
    }
    pub fn marked_native_node(&self, r: &str) -> Option<NativeNodeRc<B>> {
        self.n.marked_native_node(r)
    }
    pub fn marked_component_node(&self, r: &str) -> Option<ComponentNodeRc<B>> {
        self.n.marked_component_node(r)
    }
    pub fn marked_component<D: Component<B>>(&self, r: &str) -> Option<ComponentRc<B, D>> {
        self.n.marked_component(r)
    }
    pub fn to_html<T: std::io::Write>(&self, s: &mut T) -> std::io::Result<()> {
        self.n.to_html(s)
    }
}
impl<'a, B: Backend, C: Component<B>> Deref for ComponentRef<'a, B, C> {
    type Target = C;
    fn deref(&self) -> &C {
        self.n.as_component()
    }
}
impl<'a, B: Backend, C: Component<B>> ElementRef<'a, B> for ComponentRef<'a, B, C> {
    fn backend(&self) -> &Rc<B> {
        self.n.backend()
    }
    fn as_me_ref_handle(&self) -> &MeRefHandle<'a> {
        self.n.as_me_ref_handle()
    }
}
impl<'a, B: Backend, C: Component<B>> From<ComponentNodeRef<'a, B>> for ComponentRef<'a, B, C> {
    fn from(n: ComponentNodeRef<'a, B>) -> Self {
        Self {
            n,
            phantom_data: std::marker::PhantomData
        }
    }
}
impl<'a, B: Backend, C: Component<B>> fmt::Debug for ComponentRef<'a, B, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.n)
    }
}

pub struct ComponentRefMut<'a, B: Backend, C: Component<B>> {
    n: ComponentNodeRefMut<'a, B>,
    phantom_data: std::marker::PhantomData<C>,
}
impl<'a, B: Backend, C: Component<B>> ComponentRefMut<'a, B, C> {
    pub fn duplicate<'b>(&'b mut self) -> ComponentRefMut<'b, B, C> {
        ComponentRefMut {
            n: self.n.duplicate(),
            phantom_data: PhantomData,
        }
    }
    pub fn check_type(&self) -> bool {
        self.n.is_type::<C>()
    }
    pub fn as_node(&mut self) -> &mut ComponentNodeRefMut<'a, B> {
        &mut self.n
    }
    pub fn to_ref<'b>(&'b self) -> ComponentRef<'b, B, C> where 'a: 'b {
        self.n.to_ref().into()
    }
    pub fn backend_element(&self) -> &B::BackendElement {
        self.n.backend_element()
    }
    pub fn owner(&self) -> Option<ComponentNodeRc<B>> {
        self.n.owner()
    }
    pub fn to_html<T: std::io::Write>(&self, s: &mut T) -> std::io::Result<()> {
        self.n.to_html(s)
    }
    pub fn apply_updates(&mut self) {
        self.n.apply_updates();
    }
    pub fn force_apply_updates(&mut self) {
        self.n.force_apply_updates::<C>();
    }
    pub fn marked(&self, r: &str) -> Option<NodeRc<B>> {
        self.n.marked(r)
    }
    pub fn marked_native_node(&self, r: &str) -> Option<NativeNodeRc<B>> {
        self.n.marked_native_node(r)
    }
    pub fn marked_component_node(&self, r: &str) -> Option<ComponentNodeRc<B>> {
        self.n.marked_component_node(r)
    }
    pub fn marked_component<D: Component<B>>(&self, r: &str) -> Option<ComponentRc<B, D>> {
        self.n.marked_component(r)
    }
}
impl<'a, B: Backend, C: Component<B>> Drop for ComponentRefMut<'a, B, C> {
    fn drop(&mut self) {
        self.apply_updates();
    }
}
impl<'a, B: Backend, C: Component<B>> Deref for ComponentRefMut<'a, B, C> {
    type Target = C;
    fn deref(&self) -> &C {
        self.n.as_component()
    }
}
impl<'a, B: Backend, C: Component<B>> DerefMut for ComponentRefMut<'a, B, C> {
    fn deref_mut(&mut self) -> &mut C {
        self.n.as_component_mut()
    }
}
impl<'a, B: Backend, C: Component<B>> ElementRefMut<'a, B> for ComponentRefMut<'a, B, C> {
    fn backend(&self) -> &Rc<B> {
        self.n.backend()
    }
    fn as_me_ref_mut_handle<'b>(&'b mut self) -> &'b mut MeRefMutHandle<'a> where 'a: 'b {
        self.n.as_me_ref_mut_handle()
    }
}
impl<'a, B: Backend, C: Component<B>> From<ComponentNodeRefMut<'a, B>> for ComponentRefMut<'a, B, C> {
    fn from(n: ComponentNodeRefMut<'a, B>) -> Self {
        Self {
            n,
            phantom_data: std::marker::PhantomData
        }
    }
}
impl<'a, B: Backend, C: Component<B>> fmt::Debug for ComponentRefMut<'a, B, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.n)
    }
}

pub struct EmptyComponent { }
impl<B: Backend> Component<B> for EmptyComponent {
    fn new(_ctx: ComponentContext<B, Self>) -> Self {
        Self { }
    }
}
impl<B: Backend> ComponentTemplate<B> for EmptyComponent {
    // empty
}
