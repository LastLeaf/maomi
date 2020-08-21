use std::pin::Pin;
use std::marker::PhantomData;
use std::fmt;
use std::rc::Rc;
use std::cell::{RefCell};
use std::ops::{Deref, DerefMut};
use futures::Future;

use super::*;
use crate::context::Scheduler;
use crate::backend::{Backend};

/// A component should implement this trait.
/// The `new` method is called whenever an instance is creating, with a `ComponentContext` for manipulation.
pub trait Component<B: Backend>: ComponentTemplate<B> + downcast_rs::Downcast {
    fn new(_ctx: ComponentContext<B, Self>) -> Self where Self: Sized;
    /// Called when the component creation process is finished.
    fn created(&mut self) {

    }
    /// Called when the component is changed to `attached` state.
    fn attached(&mut self) {

    }
    // TODO implement moved
    /// Called when the component is changed to `attached` state.
    fn moved(&mut self) {

    }
    /// Called when the component is changed off `attached` state.
    fn detached(&mut self) {

    }
}
downcast_rs::impl_downcast!(Component<B> where B: Backend);

/// A component which supports prerendering (server side rendering).
///
/// Prerendering should be done with `Context::prerender` . It internally does following steps.
/// 1. Create a new component.
/// 1. Get the data used in prerendering through `get_prerendered_data` .
/// 1. Apply the data to the component with `apply_prerendered_data` .
/// 1. Call `to_html` method to get the prerendered html.
/// 1. The data and the html can be used as prerendering result.
///
/// Using the prerendering result should be done with `Context::new_prerendered` . It internally does following steps.
/// 1. Create a new component.
/// 1. Apply the data to the component with `apply_prerendered_data` .
/// 1. Match the prerendered html with the component.
///
pub trait PrerenderableComponent<B: Backend>: Component<B> {
    type Args;
    type PrerenderedData: 'static + serde::Serialize + for<'de> serde::Deserialize<'de>;
    type MetaData;
    fn get_prerendered_data(&self, args: Self::Args) -> Pin<Box<dyn Future<Output = (Self::PrerenderedData, Self::MetaData)>>>;
    fn apply_prerendered_data(&mut self, _data: &Self::PrerenderedData);
}

/// Template operation
/// **Should be done through template engine!**
#[doc(hidden)]
pub enum ComponentTemplateOperation {
    Init,
    Update,
}

/// The component template trait
/// **Should be done through template engine!**
#[doc(hidden)]
pub trait ComponentTemplate<B: Backend>: 'static {
    fn template(component: &mut ComponentNode<B>, operation: ComponentTemplateOperation) -> Option<Vec<NodeRc<B>>> where Self: Sized {
        if let ComponentTemplateOperation::Update = operation {
            return None
        }
        let mut f = || {
            vec![unsafe { component.new_virtual_node("slot", VirtualNodeProperty::Slot("", vec![]), vec![]).into() }]
        };
        Some(f())
    }
    fn template_skin() -> &'static str where Self: Sized {
        ""
    }
}

/// Contains some component manipulation methods.
/// Every component instance has a `ComponentContext` for it.
#[derive(Clone)]
pub struct ComponentContext<B: Backend, C: Component<B>> {
    node_weak: ComponentWeak<B, C>,
    need_update: Rc<RefCell<Vec<Box<dyn 'static + FnOnce(&mut ComponentNode<B>)>>>>,
    scheduler: Rc<Scheduler>,
    phantom_data: PhantomData<C>,
}

impl<B: Backend, C: Component<B>> ComponentContext<B, C> {
    pub(crate) fn new(node_weak: ComponentWeak<B, C>, need_update: Rc<RefCell<Vec<Box<dyn 'static + FnOnce(&mut ComponentNode<B>)>>>>, scheduler: Rc<Scheduler>) -> Self {
        Self {
            node_weak,
            need_update,
            scheduler,
            phantom_data: PhantomData,
        }
    }

    fn add_updater_if_needed(&self, manually_apply: bool) {
        let mut update_callbacks = self.need_update.borrow_mut();
        if update_callbacks.len() == 0 {
            update_callbacks.push(Box::new(|c: &mut ComponentNode<B>| {
                <C as ComponentTemplate<B>>::template(c, ComponentTemplateOperation::Update);
            }));
            if !manually_apply {
                self.tick_with_component_rc(|this| {
                    this.as_node().borrow_mut().apply_updates();
                })
            }
        }
    }

    /// Schedule a shadow tree update.
    /// The update process cannot start until other tree manipulation process (including other updates) is finished.
    pub fn update(&self) {
        self.add_updater_if_needed(false);
    }

    /// Schedule a shadow tree update, and execute a callback after it is done.
    /// The update process cannot start until other scheduled tree manipulation process (including other updates) is finished.
    pub fn update_then<F: 'static + FnOnce(&mut C)>(&self, callback: F) {
        self.add_updater_if_needed(false);
        self.need_update.borrow_mut().push(Box::new(|x| {
            callback(&mut x.as_component_mut::<C>());
        }));
    }

    /// Schedule a callback when other scheduled tree manipulation process is finished.
    pub fn tick<F: 'static + FnOnce(&mut ComponentRefMut<B, C>)>(&self, f: F) {
        let rc = self.node_weak.upgrade().unwrap();
        self.scheduler.add_task(move || {
            f(&mut rc.borrow_mut());
        });
    }

    /// Schedule a callback when other scheduled tree manipulation process is finished.
    pub fn tick_with_component_rc<F: 'static + FnOnce(ComponentRc<B, C>)>(&self, f: F) {
        let rc = self.node_weak.upgrade().unwrap();
        self.scheduler.add_task(move || {
            f(rc);
        });
    }
}

/// A typed component `NodeRc`
#[derive(Clone)]
pub struct ComponentRc<B: Backend, C: Component<B>> {
    n: ComponentNodeRc<B>,
    phantom_data: std::marker::PhantomData<C>,
}

impl<B: Backend, C: Component<B>> ComponentRc<B, C> {
    /// Untyping the node, making it a general component node
    pub fn into_node(self) -> ComponentNodeRc<B> {
        self.n
    }

    /// Get an untyped node reference
    pub fn as_node(&self) -> &ComponentNodeRc<B> {
        &self.n
    }

    /// Borrow the node.
    /// Panics if any node has been mutably borrowed.
    pub fn borrow<'a>(&'a self) -> ComponentRef<'a, B, C> {
        self.n.borrow().with_type::<C>()
    }

    /// Borrow the node.
    /// Return `Err` if any node has been mutably borrowed.
    pub fn try_borrow<'a>(&'a self) -> Result<ComponentRef<'a, B, C>, NodeBorrowError> {
        self.n.try_borrow().map(|x| x.with_type::<C>())
    }

    /// Borrow the node mutably.
    /// Panics if any node has been borrowed.
    pub fn borrow_mut<'a>(&'a self) -> ComponentRefMut<'a, B, C> {
        self.n.borrow_mut().with_type::<C>()
    }

    /// Borrow the node mutably.
    /// Return `Err` if any node has been borrowed.
    pub fn try_borrow_mut<'a>(&'a self) -> Result<ComponentRefMut<'a, B, C>, NodeBorrowError> {
        self.n.try_borrow_mut().map(|x| x.with_type::<C>())
    }

    /// Get a `NodeWeak` with node type
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

/// A typed component `NodeWeak`
#[derive(Clone)]
pub struct ComponentWeak<B: Backend, C: Component<B>> {
    n: ComponentNodeWeak<B>,
    phantom_data: std::marker::PhantomData<C>,
}

impl<B: Backend, C: Component<B>> ComponentWeak<B, C> {
    /// Untyping the node, making it a general component node
    pub fn into_node(self) -> ComponentNodeWeak<B> {
        self.n
    }

    /// Get an untyped node reference
    pub fn as_node(&self) -> &ComponentNodeWeak<B> {
        &self.n
    }

    /// Get a `NodeRc` with node type
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

/// A `NodeRef` with node type, representing a borrowed ref of a ref-counted node.
/// No other node can be mutably borrowed until this object is dropped.
pub struct ComponentRef<'a, B: Backend, C: Component<B>> {
    n: ComponentNodeRef<'a, B>,
    phantom_data: std::marker::PhantomData<C>,
}

impl<'a, B: Backend, C: Component<B>> ComponentRef<'a, B, C> {
    /// Get another borrowed ref of the same node
    pub fn clone<'b>(&'b mut self) -> ComponentRef<'b, B, C> {
        ComponentRef {
            n: ComponentNodeRef::clone(&self.n),
            phantom_data: PhantomData,
        }
    }

    /// Check the type of the component
    pub fn is_type(&self) -> bool {
        self.n.is_type::<C>()
    }

    /// Untyping the node, making it a general component node
    pub fn into_node(self) -> ComponentNodeRef<'a, B> {
        self.n
    }

    /// Get an untyped node reference
    pub fn as_node(&self) -> &ComponentNodeRef<'a, B> {
        &self.n
    }

    /// Get the backend node
    pub fn backend_element(&self) -> &B::BackendElement {
        self.n.backend_element()
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_rc(&self, r: &str) -> Option<NodeRc<B>> {
        self.n.marked_rc(r)
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_native_node_rc(&self, r: &str) -> Option<NativeNodeRc<B>> {
        self.n.marked_native_node_rc(r)
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_component_node_rc(&self, r: &str) -> Option<ComponentNodeRc<B>> {
        self.n.marked_component_node_rc(r)
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_component_rc<D: Component<B>>(&self, r: &str) -> Option<ComponentRc<B, D>> {
        self.n.marked_component_rc::<D>(r)
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked(&self, r: &str) -> Option<Node<B>> {
        self.n.marked(r)
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_native_node(&self, r: &str) -> Option<&NativeNode<B>> {
        self.n.marked_native_node(r)
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_component_node(&self, r: &str) -> Option<&ComponentNode<B>> {
        self.n.marked_component_node(r)
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_component<D: Component<B>>(&self, r: &str) -> Option<&D> {
        self.n.marked_component::<D>(r)
    }

    /// Convert to HTML
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
        write!(f, "{:?}", &*self.n)
    }
}

/// A mutably borrowed ref of a ref-counted node.
/// No other node can be borrowed until this object is dropped.
pub struct ComponentRefMut<'a, B: Backend, C: Component<B>> {
    n: ComponentNodeRefMut<'a, B>,
    phantom_data: std::marker::PhantomData<C>,
}

impl<'a, B: Backend, C: Component<B>> ComponentRefMut<'a, B, C> {
    /// Get another mutable reference `NodeMut`, borrowing out the current one
    pub fn as_mut<'b>(&'b mut self) -> ComponentRefMut<'b, B, C> {
        ComponentRefMut {
            n: self.n.as_mut(),
            phantom_data: PhantomData,
        }
    }

    /// Check the type of the component
    pub fn is_type(&self) -> bool {
        self.n.is_type::<C>()
    }

    /// Get an untyped node reference
    pub fn as_node(&mut self) -> &mut ComponentNodeRefMut<'a, B> {
        &mut self.n
    }

    /// Get the backend node
    pub fn backend_element(&self) -> &B::BackendElement {
        self.n.backend_element()
    }

    /// Apply updates immediately.
    /// If update is not needed, i.e. `ComponentContext::update` has not been called, it does not update the shadow tree.
    pub fn apply_updates(&mut self) {
        self.n.apply_updates();
    }

    /// Apply updates immediately.
    /// It forces updating the shadow tree.
    pub fn force_apply_updates(&mut self) {
        self.n.force_apply_updates::<C>();
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_rc(&self, r: &str) -> Option<NodeRc<B>> {
        self.n.marked_rc(r)
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_native_node_rc(&self, r: &str) -> Option<NativeNodeRc<B>> {
        self.n.marked_native_node_rc(r)
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_component_node_rc(&self, r: &str) -> Option<ComponentNodeRc<B>> {
        self.n.marked_component_node_rc(r)
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_component_rc<D: Component<B>>(&self, r: &str) -> Option<ComponentRc<B, D>> {
        self.n.marked_component_rc::<D>(r)
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked(&self, r: &str) -> Option<Node<B>> {
        self.n.marked(r)
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_native_node(&self, r: &str) -> Option<&NativeNode<B>> {
        self.n.marked_native_node(r)
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_component_node(&self, r: &str) -> Option<&ComponentNode<B>> {
        self.n.marked_component_node(r)
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_component<D: Component<B>>(&self, r: &str) -> Option<&D> {
        self.n.marked_component::<D>(r)
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_mut(&mut self, r: &str) -> Option<NodeMut<B>> {
        self.n.marked_mut(r)
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_native_node_mut(&mut self, r: &str) -> Option<&mut NativeNode<B>> {
        self.n.marked_native_node_mut(r)
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_component_node_mut(&mut self, r: &str) -> Option<&mut ComponentNode<B>> {
        self.n.marked_component_node_mut(r)
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_component_mut<D: Component<B>>(&mut self, r: &str) -> Option<&mut D> {
        self.n.marked_component_mut::<D>(r)
    }

    /// Convert to HTML
    pub fn to_html<T: std::io::Write>(&self, s: &mut T) -> std::io::Result<()> {
        self.n.to_html(s)
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
        write!(f, "{:?}", &*self.n)
    }
}

/// The default empty component
pub struct EmptyComponent { }

impl<B: Backend> Component<B> for EmptyComponent {
    fn new(_ctx: ComponentContext<B, Self>) -> Self {
        Self { }
    }
}

impl<B: Backend> ComponentTemplate<B> for EmptyComponent {
    // empty
}
