use std::rc::{Rc, Weak};
use std::cell::{Cell, RefCell, Ref};
use std::ops::{Deref, DerefMut};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::mem::ManuallyDrop;
use std::any::Any;

use super::*;
use crate::backend::*;
use crate::context::Scheduler;
use crate::global_events::GlobalEvents;
use crate::escape;

/// A component node which contains a shadow tree
pub struct ComponentNode<B: Backend> {
    pub(crate) backend: Rc<B>,
    pub(crate) scheduler: Rc<Scheduler>,
    pub(crate) backend_element: B::BackendElement,
    pub(crate) need_update: Rc<RefCell<Vec<Box<dyn 'static + FnOnce(&mut ComponentNode<B>)>>>>,
    pub(crate) mark: Cow<'static, str>,
    pub(crate) marks_cache: RefCell<HashMap<Cow<'static, str>, NodeRc<B>>>,
    pub(crate) marks_cache_dirty: Cell<bool>,
    pub(crate) tag_name: &'static str,
    pub(crate) attributes: Vec<(&'static str, String)>,
    pub(crate) component: Box<dyn Component<B>>,
    pub(crate) attached: bool,
    pub(crate) self_weak: Option<ComponentNodeWeak<B>>,
    pub(crate) shadow_root: VirtualNodeRc<B>,
    pub(crate) children: Vec<NodeRc<B>>,
    pub(crate) slots: HashMap<&'static str, VirtualNodeRc<B>>,
    pub(crate) owner: Option<ComponentNodeWeak<B>>,
    pub(crate) parent: Option<NodeWeak<B>>,
    pub(crate) composed_parent: Option<NodeWeak<B>>,
    pub(crate) global_events: GlobalEvents<B>,
}

impl<B: 'static + Backend> ComponentNodeRc<B> {
    /// Assert the component to be a specified type.
    /// Panics later if it is not this type.
    pub fn with_type<C: Component<B>>(self) -> ComponentRc<B, C> {
        ComponentRc::from(self)
    }
}

impl<B: 'static + Backend> ComponentNodeWeak<B> {
    /// Assert the component to be a specified type.
    /// Panics later if it is not this type.
    pub fn with_type<C: Component<B>>(self) -> ComponentWeak<B, C> {
        ComponentWeak::from(self)
    }
}

impl<'a, B: 'static + Backend> ComponentNode<B> {
    define_tree_getter!(node);

    pub(crate) fn inner_component_mut(&self) -> &Box<dyn Component<B>> {
        &self.component
    }

    /// Get the backend element
    pub fn backend_element(&self) -> &B::BackendElement {
        &self.backend_element
    }

    pub(crate) fn scheduler(&self) -> &Rc<Scheduler> {
        &self.scheduler
    }

    /// Get the children nodes list in composed tree
    pub fn composed_children_rc(&self) -> Cow<'_, Vec<NodeRc<B>>> {
        Cow::Owned(vec![self.shadow_root.clone().into()])
    }

    pub(super) fn collect_backend_nodes<'b>(&'b self, v: &'b mut Vec<NodeRc<B>>) {
        v.push(self.rc().into());
    }

    pub(super) fn new_with_children(backend: Rc<B>, scheduler: Rc<Scheduler>, tag_name: &'static str, shadow_root: VirtualNodeRc<B>, children: Vec<NodeRc<B>>, owner: Option<ComponentNodeWeak<B>>) -> Self {
        let backend_element = backend.create_element(tag_name);
        let need_update = Rc::new(RefCell::new(vec![]));
        let component = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        ComponentNode {
            backend,
            backend_element,
            need_update,
            scheduler,
            mark: "".into(),
            marks_cache: RefCell::new(HashMap::new()),
            marks_cache_dirty: Cell::new(false),
            tag_name,
            attributes: vec![],
            component,
            attached: false,
            self_weak: None,
            shadow_root,
            children,
            slots: HashMap::new(),
            owner,
            parent: None,
            composed_parent: None,
            global_events: GlobalEvents::new(),
        }
    }

    pub(super) unsafe fn initialize<C: 'static + Component<B>>(&mut self, self_weak: ComponentNodeWeak<B>) {
        // bind backend element
        self.backend_element.bind_node_weak(self_weak.clone().into());
        // create component
        self.self_weak = Some(self_weak.clone());
        let ctx = ComponentContext::new(self_weak.clone().into(), self.need_update.clone(), self.scheduler.clone());
        let comp = Box::new(<C as Component<B>>::new(ctx));
        let uninit = std::mem::replace(&mut self.component, comp);
        std::mem::forget(uninit);
        // set chilren's parent
        let self_weak: NodeWeak<B> = self_weak.into();
        for child in self.children.clone() {
            unsafe { child.deref_mut_unsafe() }.set_parent(Some(self_weak.clone()));
        }
        {
            // initialize shadow root
            let shadow_root_content = <C as ComponentTemplate<B>>::template(self, ComponentTemplateOperation::Init);
            let shadow_root = self.shadow_root.clone();
            let mut shadow_root = shadow_root.deref_mut_unsafe();
            shadow_root.set_composed_parent(Some(self_weak.clone()));
            if let Some(shadow_root_content) = shadow_root_content {
                shadow_root.replace_children_list(shadow_root_content);
            }
            // append shadow root
            let mut backend_children = vec![];
            shadow_root.collect_backend_nodes(&mut backend_children);
            let backend_children: Vec<_> = backend_children.iter().map(|x| x.deref_unsafe()).collect();
            self.backend_element.append_list(backend_children.iter().map(|x| x.backend_node().unwrap()).collect());
        }
        self.check_slots_update();
        self.component.created();
    }

    /// Get an attribute
    pub fn get_attribute(&self, name: &'static str) -> Option<&str> {
        self.attributes.iter().find(|x| x.0 == name).map(|x| x.1.as_str())
    }

    /// Set an attribute
    pub fn set_attribute<T: ToString>(&mut self, name: &'static str, value: T) {
        let value = value.to_string();
        self.backend_element.set_attribute(name, &value);
        match self.attributes.iter_mut().find(|x| x.0 == name) {
            Some(x) => {
                x.1 = value;
                return
            },
            None => { }
        }
        self.attributes.push((name, value))
    }

    /// Get the shadow root `NodeRc`
    pub fn shadow_root_rc(&self) -> &VirtualNodeRc<B> {
        &self.shadow_root
    }

    /// Get the shadow root node
    pub fn shadow_root(&self) -> &VirtualNode<B> {
        unsafe { self.shadow_root.deref_unsafe() }
    }

    /// Get the shadow root node
    pub fn shadow_root_mut(&self) -> &mut VirtualNode<B> {
        unsafe { self.shadow_root.deref_mut_unsafe() }
    }

    /// Check the type of the component
    pub fn is_type<C: Component<B>>(&self) -> bool {
        self.component.downcast_ref::<C>().is_some()
    }

    /// Get the inner `Component` reference
    pub fn as_component<C: Component<B>>(&self) -> &C {
        self.component.downcast_ref::<C>().unwrap()
    }

    /// Get the inner `Component` mutable reference
    pub fn as_component_mut<C: Component<B>>(&mut self) -> &mut C {
        self.component.downcast_mut::<C>().unwrap()
    }

    /// Get the inner `Component` reference
    pub fn try_as_component<C: 'static + Component<B>>(&self) -> Option<&C> {
        let c: &dyn Any = &self.component;
        c.downcast_ref()
    }

    /// Get the inner `Component` mutable reference
    pub fn try_as_component_mut<C: 'static + Component<B>>(&mut self) -> Option<&mut C> {
        let c: &mut dyn Any = &mut self.component;
        c.downcast_mut()
    }

    /// List the global events bindings
    pub fn global_events(&self) -> &GlobalEvents<B> {
        &self.global_events
    }

    /// List the global events bindings mutably
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub fn global_events_mut(&mut self) -> &mut GlobalEvents<B> {
        &mut self.global_events
    }

    fn check_marks_cache(&self) {
        if self.marks_cache_dirty.replace(false) {
            let mut map: HashMap<_, NodeRc<B>> = HashMap::new();
            let shadow_root = self.shadow_root();
            shadow_root.dfs(TraversalRange::Shadow, TraversalOrder::ParentFirst).for_each(|node| {
                match node {
                    Node::NativeNode(n) => {
                        if n.mark.len() > 0 && !map.contains_key(&n.mark) {
                            map.insert(n.mark.clone(), node.rc());
                        }
                    },
                    Node::ComponentNode(n) => {
                        if n.mark.len() > 0 && !map.contains_key(&n.mark) {
                            map.insert(n.mark.clone(), node.rc());
                        }
                    },
                    _ => { }
                }
            });
            *self.marks_cache.borrow_mut() = map;
        }
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_rc(&self, mark: &str) -> Option<NodeRc<B>> {
        self.check_marks_cache();
        self.marks_cache.borrow().get(mark).cloned()
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_native_node_rc(&self, r: &str) -> Option<NativeNodeRc<B>> {
        match self.marked_rc(r) {
            None => None,
            Some(x) => {
                match x {
                    NodeRc::NativeNode(x) => Some(x),
                    _ => None,
                }
            }
        }
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_component_node_rc(&self, r: &str) -> Option<ComponentNodeRc<B>> {
        match self.marked_rc(r) {
            None => None,
            Some(x) => {
                match x {
                    NodeRc::ComponentNode(x) => Some(x),
                    _ => None,
                }
            }
        }
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_component_rc<C: Component<B>>(&self, r: &str) -> Option<ComponentRc<B, C>> {
        self.marked_component_node_rc(r).map(|x| x.with_type::<C>())
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked<'d>(&'d self, r: &str) -> Option<Node<'d, B>> {
        self.marked_rc(r).map(|x| unsafe { x.deref_unsafe_with_lifetime() })
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_native_node<'d>(&'d self, r: &str) -> Option<&'d NativeNode<B>> {
        self.marked_native_node_rc(r).map(|x| unsafe { x.deref_unsafe_with_lifetime() })
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_component_node<'d>(&'d self, r: &str) -> Option<&'d ComponentNode<B>> {
        self.marked_component_node_rc(r).map(|x| unsafe { x.deref_unsafe_with_lifetime() })
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_component<C: Component<B>>(&self, r: &str) -> Option<&C> {
        self.marked_component_node(r).map(|x| x.as_component::<C>())
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_mut<'d>(&'d mut self, r: &str) -> Option<NodeMut<'d, B>> {
        self.marked_rc(r).map(|x| unsafe { x.deref_mut_unsafe_with_lifetime() })
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_native_node_mut<'d>(&'d mut self, r: &str) -> Option<&'d mut NativeNode<B>> {
        self.marked_native_node_rc(r).map(|x| unsafe { x.deref_mut_unsafe_with_lifetime() })
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_component_node_mut<'d>(&'d mut self, r: &str) -> Option<&'d mut ComponentNode<B>> {
        self.marked_component_node_rc(r).map(|x| unsafe { x.deref_mut_unsafe_with_lifetime() })
    }

    /// Get a node with specified mark in the shadow tree
    pub fn marked_component_mut<C: Component<B>>(&mut self, r: &str) -> Option<&mut C> {
        self.marked_component_node_mut(r).map(|x| x.as_component_mut::<C>())
    }

    /// Convert to HTML
    pub fn to_html<T: std::io::Write>(&self, s: &mut T) -> std::io::Result<()> {
        write!(s, "<{}", self.tag_name)?;
        for (name, value) in self.attributes.iter() {
            write!(s, r#" {}="{}""#, name, escape::escape_html(value))?;
        }
        write!(s, ">")?;
        self.shadow_root().to_html(s)?;
        write!(s, "</{}>", self.tag_name)
    }

    /// Convert to HTML, with an specified `id` attribute in the node
    pub fn to_html_with_id<T: std::io::Write>(&self, s: &mut T, id: &str) -> std::io::Result<()> {
        write!(s, r#"<{} id="{}""#, self.tag_name, escape::escape_html(id))?;
        for (name, value) in self.attributes.iter() {
            write!(s, r#" {}="{}""#, name, escape::escape_html(value))?;
        }
        write!(s, ">")?;
        self.shadow_root().to_html(s)?;
        write!(s, "</{}>", self.tag_name)
    }

    pub(super) fn debug_fmt(&self, f: &mut fmt::Formatter<'_>, level: u32) -> fmt::Result {
        for _ in 0..level {
            write!(f, "  ")?;
        }
        writeln!(f, "{:?}", self.tag_name)?;
        for child in self.children() {
            child.debug_fmt(f, level + 1)?;
        }
        Ok(())
    }

    /// Create a new node in the shadow tree.
    /// It is safe only when updating the shadow tree.
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub unsafe fn new_native_node(&mut self, tag_name: &'static str, attributes: Vec<(&'static str, String)>, children: Vec<NodeRc<B>>) -> NativeNodeRc<B> {
        let another_me_cell = self.rc().another_me_cell();
        let backend = self.backend.clone();
        let scheduler = self.scheduler.clone();
        let owner = self.self_weak.clone();
        let n = NativeNodeRc {
            c: Rc::new(another_me_cell.another(NativeNode::new_with_children(backend, scheduler, tag_name, attributes, children, owner)))
        };
        n.deref_mut_unsafe().initialize(n.downgrade());
        n
    }

    /// Create a new node in the shadow tree.
    /// It is safe only when updating the shadow tree.
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub unsafe fn new_virtual_node(&mut self, tag_name: &'static str, property: VirtualNodeProperty<B>, children: Vec<NodeRc<B>>) -> VirtualNodeRc<B> {
        let another_me_cell = self.rc().another_me_cell();
        let backend = self.backend.clone();
        let scheduler = self.scheduler.clone();
        let owner = self.self_weak.clone();
        let n = VirtualNodeRc {
            c: Rc::new(another_me_cell.another(VirtualNode::new_with_children(backend, scheduler, tag_name, property, children, owner)))
        };
        n.deref_mut_unsafe().initialize(n.downgrade());
        n
    }

    /// Create a new node in the shadow tree.
    /// It is safe only when updating the shadow tree.
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub unsafe fn new_component_node<C: 'static + Component<B>>(&mut self, tag_name: &'static str, children: Vec<NodeRc<B>>) -> ComponentNodeRc<B> {
        let scheduler = self.scheduler.clone();
        let self_weak = self.self_weak.clone();
        Self::new_free_component_node::<C>(self.into(), scheduler, tag_name, children, self_weak)
    }

    pub(super) unsafe fn new_free_component_node<C: 'static + Component<B>>(
        n: NodeMut<B>,
        scheduler: Rc<Scheduler>,
        tag_name: &'static str,
        children: Vec<NodeRc<B>>,
        owner: Option<ComponentNodeWeak<B>>,
    ) -> ComponentNodeRc<B> {
        let another_me_cell = n.rc().another_me_cell();
        let backend = n.as_ref().backend().clone();
        let shadow_root = VirtualNodeRc {
            c: Rc::new(another_me_cell.another(VirtualNode::new_with_children(backend, scheduler.clone(), "shadow-root", VirtualNodeProperty::ShadowRoot, vec![], owner.clone())))
        };
        unsafe { shadow_root.deref_mut_unsafe() }.initialize(shadow_root.downgrade());
        let backend = n.as_ref().backend().clone();
        let ret = ComponentNodeRc {
            c: Rc::new(another_me_cell.another(ComponentNode::new_with_children(backend, scheduler, tag_name, shadow_root.clone(), children, owner)))
        };
        unsafe { ret.deref_mut_unsafe() }.initialize::<C>(ret.downgrade());
        ret
    }

    /// Create a new node in the shadow tree.
    /// It is safe only when updating the shadow tree.
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub unsafe fn new_text_node<T: ToString>(&mut self, text_content: T) -> TextNodeRc<B> {
        let another_me_cell = self.rc().another_me_cell();
        let backend = self.backend.clone();
        let scheduler = self.scheduler.clone();
        let owner = self.self_weak.clone();
        let n = TextNodeRc {
            c: Rc::new(another_me_cell.another(TextNode::new_with_content(backend, scheduler, text_content.to_string(), owner)))
        };
        n.deref_mut_unsafe().initialize(n.downgrade());
        n
    }

    unsafe fn reassign_slots(&mut self, new_slots: HashMap<&'static str, VirtualNodeRc<B>>) {
        let main_slot_changed = self.slots.get("") != new_slots.get("");
        // clear composed children for slots
        for (name, slot) in new_slots.iter() {
            if *name == "" && !main_slot_changed {
                continue
            }
            let mut slot = slot.deref_mut_unsafe();
            if let VirtualNodeProperty::Slot(_, c) = &mut slot.property {
                c.truncate(0);
            } else {
                unreachable!()
            }
        }
        let mut old_slots = new_slots.clone();
        std::mem::swap(&mut self.slots, &mut old_slots);
        for (name, slot) in old_slots.iter() {
            if *name == "" && !main_slot_changed {
                continue
            }
            let mut slot = slot.deref_mut_unsafe();
            if let VirtualNodeProperty::Slot(_, c) = &mut slot.property {
                c.truncate(0);
            } else {
                unreachable!()
            }
        }
        // re-insert children
        for child in self.children.iter() {
            let child_node_rc = child.clone();
            let put_child_in_slot = |name, mut child: NodeMut<_>| {
                match new_slots.get(name) {
                    Some(slot) => {
                        let slot_weak = slot.deref_unsafe().self_weak.clone().map(|x| x.into());
                        child.set_composed_parent(slot_weak);
                        {
                            let mut slot = slot.deref_mut_unsafe();
                            if let VirtualNodeProperty::Slot(_, c) = &mut slot.property {
                                c.push(child_node_rc);
                            } else {
                                unreachable!()
                            }
                        }
                        let mut nodes = vec![];
                        child.as_ref().collect_backend_nodes(&mut nodes);
                        let slot = slot.deref_unsafe();
                        let before = slot.find_next_sibling(false);
                        let before = before.as_ref().map(|x| x.deref_unsafe());
                        let nodes: Vec<_> = nodes.iter().map(|x| x.deref_unsafe()).collect();
                        let nodes = nodes.iter().map(|x| x.backend_node().unwrap()).collect();
                        match slot.find_backend_parent() {
                            Some(parent) => parent.deref_unsafe().backend_element().unwrap().insert_list_before(nodes, before.as_ref().map(|x| x.backend_node().unwrap())),
                            None => {
                                for n in nodes {
                                    n.remove_self();
                                }
                            }
                        }
                    },
                    None => {
                        child.set_composed_parent(None);
                        let mut nodes = vec![];
                        child.as_ref().collect_backend_nodes(&mut nodes);
                        let nodes: Vec<_> = nodes.iter().map(|x| x.deref_unsafe()).collect();
                        let nodes: Vec<_> = nodes.iter().map(|x| x.backend_node().unwrap()).collect();
                        for n in nodes {
                            n.remove_self();
                        }
                    }
                }
            };
            if let NodeRc::VirtualNode(child) = &child {
                let child = child.deref_mut_unsafe();
                if let VirtualNodeProperty::InSlot(p) = child.property {
                    if p != "" {
                        // insert children in common slot
                        put_child_in_slot(p, child.into());
                        continue;
                    }
                }
            }
            if !main_slot_changed {
                continue;
            }
            // insert children in main slot
            let child = child.deref_mut_unsafe();
            put_child_in_slot("", child);
        }
    }

    unsafe fn check_slots_update(&mut self) {
        // collect slots
        let mut slots = HashMap::new();
        {
            self.shadow_root().dfs(TraversalRange::Shadow, TraversalOrder::ParentFirst).for_each(|n| {
                if let Node::VirtualNode(n) = &n {
                    if let VirtualNodeProperty::Slot(name, _) = n.property {
                        slots.insert(name, n.rc());
                    }
                }
            });
        }
        if self.slots == slots {
            return
        }
        self.reassign_slots(slots);
    }

    /// Set the mark.
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub fn set_mark<T: Into<Cow<'static, str>>>(&mut self, r: T) {
        let r = r.into();
        if self.mark == r {
            return;
        }
        self.mark = r;
        if let Some(c) = self.owner().next() {
            c.marks_cache_dirty.set(true);
        }
    }

    /// Apply updates immediately.
    /// If update is not needed, i.e. `ComponentContext::update` has not been called, it does not update the shadow tree.
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub unsafe fn apply_updates(&mut self) {
        let new_updates = {
            let mut updates = self.need_update.borrow_mut();
            if updates.len() == 0 {
                return;
            }
            let mut new_updates = vec![];
            std::mem::swap(&mut *updates, &mut new_updates);
            new_updates
        };
        let mut updates = new_updates.into_iter();
        let f = updates.next().unwrap();
        f(self);
        self.update_node();
        for f in updates {
            f(self);
        }
    }

    /// Apply updates immediately.
    /// It forces updating the shadow tree.
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub unsafe fn force_apply_updates<C: Component<B>>(&mut self) {
        let forced = {
            let updates = self.need_update.borrow_mut();
            updates.len() == 0
        };
        if forced {
            <C as ComponentTemplate<B>>::template(self, ComponentTemplateOperation::Update);
            self.update_node();
        } else {
            self.apply_updates();
        }
    }

    unsafe fn update_node(&mut self) {
        self.check_slots_update();
    }

    pub(crate) fn set_attached(&mut self) {
        self.component.attached();
        let mut children = self.composed_children_mut();
        while let Some(mut child) = children.next() {
            child.set_attached();
        }
    }

    pub(crate) fn set_detached(&mut self) {
        self.component.detached();
        let mut children = self.composed_children_mut();
        while let Some(mut child) = children.next() {
            child.set_detached();
        }
    }
}

impl<'a, B: Backend> ComponentNodeRef<'a, B> {
    define_tree_getter!(ref);

    /// Assert the component to be a specified type.
    /// Panics later if it is not this type.
    pub fn with_type<C: Component<B>>(self) -> ComponentRef<'a, B, C> {
        ComponentRef::from(self)
    }
}

impl<'a, B: Backend> ComponentNodeRefMut<'a, B> {
    define_tree_getter!(ref mut);

    /// Assert the component to be a specified type.
    /// Panics later if it is not this type.
    pub fn with_type<C: Component<B>>(self) -> ComponentRefMut<'a, B, C> {
        ComponentRefMut::from(self)
    }

    /// Apply updates immediately.
    /// If update is not needed, i.e. `ComponentContext::update` has not been called, it does not update the shadow tree.
    pub fn apply_updates(&mut self) {
        unsafe { (**self).apply_updates() }
    }

    /// Apply updates immediately.
    /// It forces updating the shadow tree.
    pub fn force_apply_updates<C: Component<B>>(&mut self) {
        unsafe { (**self).force_apply_updates::<C>() }
    }
}

impl<B: Backend> fmt::Debug for ComponentNode<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug_fmt(f, 0)
    }
}

some_node_def!(ComponentNode, ComponentNodeRc, ComponentNodeWeak, ComponentNodeRef, ComponentNodeRefMut);
