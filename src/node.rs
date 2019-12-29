use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};
use std::collections::HashMap;
use std::fmt;
use std::any::Any;
use std::mem::ManuallyDrop;
use me_cell::*;

use super::{Component, ComponentTemplate, ComponentTemplateOperation, ComponentContext, ComponentRef, ComponentRefMut, ComponentRc, ComponentWeak};
use super::backend::*;
use super::context::Scheduler;
use super::escape;
use super::global_events::GlobalEvents;

fn dfs_shadow_tree<'a, B: Backend, T: ElementRef<'a, B>, F: FnMut(&NodeRc<B>)>(n: &T, children: &Vec<NodeRc<B>>, f: &mut F) {
    for child in children.iter() {
        f(child);
        match child {
            NodeRc::NativeNode(child) => dfs_shadow_tree(n, &child.borrow_with(n).children, f),
            NodeRc::VirtualNode(child) => dfs_shadow_tree(n, &child.borrow_with(n).children, f),
            NodeRc::ComponentNode(child) => dfs_shadow_tree(n, &child.borrow_with(n).children, f),
            NodeRc::TextNode(_) => { },
        }
    }
}

pub(crate) fn create_component<'a, B: Backend, T: ElementRefMut<'a, B>, C: 'static + Component<B>>(
    n: &mut T, scheduler: Rc<Scheduler>,
    tag_name: &'static str,
    children: Vec<NodeRc<B>>,
    owner: Option<ComponentNodeWeak<B>>,
) -> ComponentNodeRc<B> {
    let backend = n.backend().clone();
    let shadow_root = VirtualNodeRc {
        c: Rc::new(n.as_me_ref_mut_handle().entrance(VirtualNode::new_with_children(backend, "shadow-root", VirtualNodeProperty::ShadowRoot, vec![], owner.clone())))
    };
    shadow_root.borrow_mut_with(n).initialize(shadow_root.downgrade());
    let backend = n.backend().clone();
    let ret = ComponentNodeRc {
        c: Rc::new(n.as_me_ref_mut_handle().entrance(ComponentNode::new_with_children(backend, scheduler, tag_name, shadow_root.clone(), children, owner)))
    };
    ret.borrow_mut_with(n).initialize::<C>(ret.downgrade());
    ret
}

macro_rules! define_tree_getter {
    (text) => {
        pub fn owner(&self) -> Option<ComponentNodeRc<B>> {
            match self.owner.as_ref() {
                Some(x) => x.upgrade(),
                None => None,
            }
        }
        pub fn parent(&self) -> Option<NodeRc<B>> {
            match self.parent.as_ref() {
                Some(x) => x.upgrade(),
                None => None,
            }
        }
        fn set_parent(&mut self, p: Option<NodeWeak<B>>) {
            self.parent = p;
        }
        pub fn composed_parent(&self) -> Option<NodeRc<B>> {
            match self.composed_parent.as_ref() {
                Some(x) => x.upgrade(),
                None => None,
            }
        }
        fn set_composed_parent(&mut self, p: Option<NodeWeak<B>>) {
            self.composed_parent = p;
        }
        pub fn is_attached(&self) -> bool {
            self.attached
        }
    };
    (node) => {
        define_tree_getter!(text);
        pub fn children(&self) -> &Vec<NodeRc<B>> {
            &self.children
        }
    };
    (ref) => {
        fn find_next_sibling<'b>(&'b self, include_self: bool) -> Option<NodeRc<B>> {
            match self.composed_parent() {
                None => None,
                Some(composed_parent) => {
                    let next_child = {
                        let composed_parent = composed_parent.borrow_with(self);
                        let index = composed_parent.composed_children().iter().position(|x| *x == NodeRc::from(self.rc())).unwrap();
                        composed_parent.find_next_backend_child(index + if include_self { 0 } else { 1 })
                    };
                    match next_child {
                        Some(x) => Some(x),
                        None => match composed_parent {
                            NodeRc::NativeNode(_) => None,
                            NodeRc::VirtualNode(x) => x.borrow_with(self).find_next_sibling(false),
                            NodeRc::ComponentNode(_) => None,
                            _ => unreachable!()
                        }
                    }
                }
            }
        }
    };
    (ref mut) => {
        pub fn to_html<T: std::io::Write>(&self, s: &mut T) -> std::io::Result<()> {
            self.to_ref().to_html(s)
        }
    };
}

pub struct NativeNode<B: Backend> {
    pub(crate) backend: Rc<B>,
    pub(crate) backend_element: B::BackendElement,
    pub(crate) attached: bool,
    pub(crate) self_weak: Option<NativeNodeWeak<B>>,
    pub(crate) mark: Cow<'static, str>,
    pub(crate) tag_name: &'static str,
    pub(crate) attributes: Vec<(&'static str, String)>,
    pub(crate) children: Vec<NodeRc<B>>,
    pub(crate) owner: Option<ComponentNodeWeak<B>>,
    pub(crate) parent: Option<NodeWeak<B>>,
    pub(crate) composed_parent: Option<NodeWeak<B>>,
    pub(crate) global_events: GlobalEvents<B>,
}
impl<B: Backend> NativeNode<B> {
    define_tree_getter!(node);
    pub fn composed_children(&self) -> Vec<NodeRc<B>> {
        self.children.clone()
    }
    pub(crate) fn new_with_children(backend: Rc<B>, tag_name: &'static str, attributes: Vec<(&'static str, String)>, children: Vec<NodeRc<B>>, owner: Option<ComponentNodeWeak<B>>) -> Self {
        let backend_element = backend.create_element(tag_name);
        NativeNode {
            backend,
            backend_element,
            attached: false,
            self_weak: None,
            mark: "".into(),
            tag_name,
            attributes,
            children,
            owner,
            parent: None,
            composed_parent: None,
            global_events: GlobalEvents::new(),
        }
    }
    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
    pub fn get_attribute(&self, name: &'static str) -> Option<&str> {
        self.attributes.iter().find(|x| x.0 == name).map(|x| x.1.as_str())
    }
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
    pub fn global_events(&self) -> &GlobalEvents<B> {
        &self.global_events
    }
    pub fn global_events_mut(&mut self) -> &mut GlobalEvents<B> {
        &mut self.global_events
    }
}
impl<'a, B: Backend> NativeNodeRef<'a, B> {
    define_tree_getter!(ref);
    fn collect_backend_nodes<'b>(&'b self, v: &'b mut Vec<NodeRc<B>>) {
        v.push(self.rc().into())
    }
    pub fn backend_element(&self) -> &B::BackendElement {
        &self.backend_element
    }
    pub fn to_html<T: std::io::Write>(&self, s: &mut T) -> std::io::Result<()> {
        write!(s, "<{}", self.tag_name)?;
        for (name, value) in self.attributes.iter() {
            write!(s, r#" {}="{}""#, name, escape::escape_html(value))?;
        }
        write!(s, ">")?;
        for child in self.children.iter() {
            child.borrow_with(self).to_html(s)?;
        }
        write!(s, "</{}>", self.tag_name)
    }
    fn debug_fmt(&self, f: &mut fmt::Formatter<'_>, level: u32) -> fmt::Result {
        for _ in 0..level {
            write!(f, "  ")?;
        }
        let n: &NativeNode<B> = &**self;
        writeln!(f, "{:?}", n)?;
        for child in self.children.iter() {
            child.borrow_with(self).debug_fmt(f, level + 1)?;
        }
        Ok(())
    }
}
impl<'a, B: Backend> NativeNodeRefMut<'a, B> {
    define_tree_getter!(ref mut);
    pub fn backend_element(&self) -> &B::BackendElement {
        &self.backend_element
    }
    fn initialize(&mut self, self_weak: NativeNodeWeak<B>) {
        // bind backend element
        self.backend_element.bind_node_weak(self_weak.clone().into());
        // set chilren's parent
        self.self_weak = Some(self_weak.clone());
        let self_weak: NodeWeak<B> = self_weak.into();
        for child in self.children.clone() {
            let mut child = child.borrow_mut_with(self);
            child.set_parent(Some(self_weak.clone()));
            child.set_composed_parent(Some(self_weak.clone()));
        }
        // insert in backend
        let mut backend_children = vec![];
        let self_ref = self.to_ref();
        for child in self.children.iter() {
            child.borrow_with(&self_ref).collect_backend_nodes(&mut backend_children);
        }
        let backend_children: Vec<_> = backend_children.iter().map(|x| x.borrow_with(&self_ref)).collect();
        let backend_children: Vec<_> = backend_children.iter().map(|x| x.backend_node().unwrap()).collect();
        self.backend_element.append_list(backend_children);
    }
    pub fn set_mark<T: Into<Cow<'static, str>>>(&mut self, r: T) {
        let r = r.into();
        if self.mark == r {
            return;
        }
        self.mark = r;
        if let Some(c) = self.owner() {
            c.borrow_mut_with(self).marks_cache_dirty.set(true);
        }
    }
    fn set_attached(&mut self) {
        if self.attached { return };
        self.attached = true;
        for child in self.children.clone() {
            child.borrow_mut_with(self).set_attached();
        }
    }
    fn set_detached(&mut self) {
        if !self.attached { return };
        self.attached = false;
        for child in self.children.clone() {
            child.borrow_mut_with(self).set_detached();
        }
    }
}
impl<'a, B: Backend> Drop for NativeNodeRefMut<'a, B> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.c);
        }
    }
}
impl<B: Backend> fmt::Debug for NativeNode<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}", self.tag_name)?;
        for (name, value) in self.attributes.iter() {
            write!(f, r#" {}="{}""#, name, value)?;
        }
        write!(f, ">")?;
        Ok(())
    }
}

pub enum VirtualNodeProperty<B: Backend> {
    None,
    ShadowRoot,
    Slot(&'static str, Vec<NodeRc<B>>),
    InSlot(&'static str),
    Branch(usize),
    List(Box<dyn Any>),
}

pub struct VirtualNode<B: Backend> {
    pub(crate) backend: Rc<B>,
    pub(crate) attached: bool,
    pub(crate) self_weak: Option<VirtualNodeWeak<B>>,
    pub(crate) tag_name: &'static str,
    pub(crate) property: VirtualNodeProperty<B>,
    pub(crate) children: Vec<NodeRc<B>>, // for slot node, children is always empty
    pub(crate) owner: Option<ComponentNodeWeak<B>>,
    pub(crate) parent: Option<NodeWeak<B>>,
    pub(crate) composed_parent: Option<NodeWeak<B>>,
}
impl<B: Backend> VirtualNode<B> {
    define_tree_getter!(node);
    pub fn composed_children(&self) -> Vec<NodeRc<B>> {
        match &self.property {
            VirtualNodeProperty::Slot(_, children) => children.clone(),
            _ => self.children.clone(),
        }
    }
    pub(crate) fn new_empty(backend: Rc<B>) -> Self {
        Self::new_with_children(backend, "", VirtualNodeProperty::None, vec![], None)
    }
    fn new_with_children(backend: Rc<B>, tag_name: &'static str, property: VirtualNodeProperty<B>, children: Vec<NodeRc<B>>, owner: Option<ComponentNodeWeak<B>>) -> Self {
        if let VirtualNodeProperty::Slot(_, c) = &property {
            if children.len() > 0 || c.len() > 0 {
                panic!("Slot cannot contain any child")
            }
        }
        VirtualNode {
            backend,
            attached: false,
            self_weak: None,
            tag_name,
            property,
            children,
            owner,
            parent: None,
            composed_parent: None,
        }
    }
    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
    pub fn property(&self) -> &VirtualNodeProperty<B> {
        &self.property
    }
    pub fn set_property(&mut self, property: VirtualNodeProperty<B>) {
        self.property = property;
    }
}
impl<'a, B: Backend> VirtualNodeRef<'a, B> {
    define_tree_getter!(ref);
    fn collect_backend_nodes<'b>(&'b self, v: &'b mut Vec<NodeRc<B>>) {
        for child in self.composed_children().iter() {
            child.borrow_with(self).collect_backend_nodes(v);
        }
    }
    fn find_backend_parent<'b>(&'b self) -> Option<NodeRc<B>> {
        match self.composed_parent() {
            None => None,
            Some(composed_parent) => match composed_parent {
                NodeRc::NativeNode(x) => Some(x.clone().into()),
                NodeRc::VirtualNode(x) => x.borrow_with(self).find_backend_parent(),
                NodeRc::ComponentNode(x) => Some(x.clone().into()),
                _ => unreachable!()
            }
        }
    }
    pub fn to_html<T: std::io::Write>(&self, s: &mut T) -> std::io::Result<()> {
        match &self.property {
            VirtualNodeProperty::Slot(_, children) => {
                for child in children.iter() {
                    child.borrow_with(self).to_html(s)?;
                }
            },
            _ => {
                for child in self.children.iter() {
                    child.borrow_with(self).to_html(s)?;
                }
            },
        }
        Ok(())
    }
    fn debug_fmt(&self, f: &mut fmt::Formatter<'_>, level: u32) -> fmt::Result {
        for _ in 0..level {
            write!(f, "  ")?;
        }
        let n: &VirtualNode<B> = &**self;
        writeln!(f, "{:?}", n)?;
        for child in self.children.iter() {
            child.borrow_with(self).debug_fmt(f, level + 1)?;
        }
        Ok(())
    }
}
impl<'a, B: Backend> VirtualNodeRefMut<'a, B> {
    define_tree_getter!(ref mut);
    pub fn property_mut(&mut self) -> &mut VirtualNodeProperty<B> {
        &mut self.property
    }
    pub fn set_shadow_root_content(&mut self, mut list: Vec<NodeRc<B>>) {
        self.owner().unwrap().borrow_mut_with(self).marks_cache_dirty.set(true);
        if let VirtualNodeProperty::ShadowRoot = self.property {
            if self.children.len() > 0 {
                panic!("Cannot reset shadow root content")
            }
            // set new children's parent
            let self_weak: NodeWeak<B> = self.self_weak.clone().unwrap().into();
            for child in list.iter() {
                let mut child = child.borrow_mut_with(self);
                child.set_parent(Some(self_weak.clone()));
                child.set_composed_parent(Some(self_weak.clone()));
            }
            std::mem::swap(&mut self.children, &mut list);
            // insert new backend children
            let self_ref = self.to_ref();
            let mut backend_children = vec![];
            for n in self.children.iter() {
                n.borrow_with(&self_ref).collect_backend_nodes(&mut backend_children);
            }
            let before = self_ref.find_next_sibling(false);
            let before = before.as_ref().map(|x| x.borrow_with(&self_ref));
            let backend_children: Vec<_> = backend_children.iter().map(|x| x.borrow_with(&self_ref)).collect();
            let backend_children: Vec<_> = backend_children.iter().map(|x| x.backend_node().unwrap()).collect();
            self.owner().unwrap().borrow_with(&self_ref).backend_element.insert_list_before(backend_children, before.as_ref().map(|x| x.backend_node().unwrap()));
        } else {
            panic!("Cannot set shadow root content on non-shadowRoot node")
        }
    }
    pub fn replace_children_list(&mut self, mut list: Vec<NodeRc<B>>) {
        if let Some(c) = self.owner() {
            c.borrow_mut_with(self).marks_cache_dirty.set(true);
        }
        // set new children's parent
        let self_weak: NodeWeak<B> = self.self_weak.clone().unwrap().into();
        for child in list.iter() {
            let mut child = child.borrow_mut_with(self);
            child.set_parent(Some(self_weak.clone()));
            child.set_composed_parent(Some(self_weak.clone()));
        }
        std::mem::swap(&mut self.children, &mut list);
        // remove old children's parent
        for child in list.iter_mut() {
            let mut child = child.borrow_mut_with(self);
            child.set_parent(None);
            child.set_composed_parent(None);
        }
        {
            let self_ref = self.to_ref();
            if let Some(p) = self_ref.find_backend_parent() {
                // remove old backend children
                let mut backend_children = vec![];
                for n in list.iter() {
                    n.borrow_with(&self_ref).collect_backend_nodes(&mut backend_children);
                }
                let backend_children: Vec<_> = backend_children.iter().map(|x| x.borrow_with(&self_ref)).collect();
                let backend_children: Vec<_> = backend_children.iter().map(|x| x.backend_node().unwrap()).collect();
                p.borrow_with(&self_ref).backend_element().unwrap().remove_list(backend_children);
                // insert new backend children
                let mut backend_children = vec![];
                for n in self.children.iter() {
                    n.borrow_with(&self_ref).collect_backend_nodes(&mut backend_children);
                }
                let before = self_ref.find_next_sibling(false);
                let before = before.as_ref().map(|x| x.borrow_with(&self_ref));
                let backend_children: Vec<_> = backend_children.iter().map(|x| x.borrow_with(&self_ref)).collect();
                let backend_children: Vec<_> = backend_children.iter().map(|x| x.backend_node().unwrap()).collect();
                p.borrow_with(&self_ref).backend_element().unwrap().insert_list_before(backend_children, before.as_ref().map(|x| x.backend_node().unwrap()));
            }
        }
        // call detached and attached
        if self.attached {
            for child in list.iter_mut() {
                let mut child = child.borrow_mut_with(self);
                child.set_detached();
            }
            for child in self.children.clone() {
                let mut child = child.borrow_mut_with(self);
                child.set_attached();
            }
        }
    }
    pub fn remove_with_reuse(&mut self, start: usize, reusable: &Box<[bool]>) {
        if let Some(c) = self.owner() {
            c.borrow_mut_with(self).marks_cache_dirty.set(true);
        }
        // set children
        let r = start..(start + reusable.len());
        let removed: Vec<NodeRc<B>> = self.children.splice(r, vec![]).collect();
        // remove old children's parent
        for child in removed.iter() {
            let mut child = child.borrow_mut_with(self);
            child.set_parent(None);
            child.set_composed_parent(None);
        }
        // remove in backend
        {
            let self_ref = self.to_ref();
            if let Some(p) = self_ref.find_backend_parent() {
                let mut backend_children = vec![];
                for (i, n) in removed.iter().enumerate() {
                    if !reusable[i] {
                        n.borrow_with(&self_ref).collect_backend_nodes(&mut backend_children);
                    }
                }
                let backend_children: Vec<_> = backend_children.iter().map(|x| x.borrow_with(&self_ref)).collect();
                let backend_children: Vec<_> = backend_children.iter().map(|x| x.backend_node().unwrap()).collect();
                p.borrow_with(&self_ref).backend_element().unwrap().remove_list(backend_children);
            }
        }
        // call detached if it really needs to be detached
        if self.attached {
            for (i, n) in removed.iter().enumerate() {
                if !reusable[i] {
                    n.borrow_mut_with(self).set_detached()
                }
            }
        }
    }
    pub fn insert_list(&mut self, pos: usize, list: Vec<NodeRc<B>>) {
        if let Some(c) = self.owner() {
            c.borrow_mut_with(self).marks_cache_dirty.set(true);
        }
        // set new children's parent
        let self_weak: NodeWeak<B> = self.self_weak.clone().unwrap().into();
        for child in list.iter() {
            let mut child = child.borrow_mut_with(self);
            child.set_parent(Some(self_weak.clone()));
            child.set_composed_parent(Some(self_weak.clone()));
        }
        {
            // insert in backend
            let self_ref = self.to_ref();
            if let Some(b) = self_ref.find_backend_parent() {
                let mut backend_children = vec![];
                for n in list.iter() {
                    n.borrow_with(&self_ref).collect_backend_nodes(&mut backend_children);
                }
                let before = match self_ref.children.get(pos) {
                    Some(x) => {
                        x.borrow_with(&self_ref).find_next_sibling(true)
                    },
                    None => None,
                };
                let before = before.as_ref().map(|x| x.borrow_with(&self_ref));
                let backend_children: Vec<_> = backend_children.iter().map(|x| x.borrow_with(&self_ref)).collect();
                let backend_children: Vec<_> = backend_children.iter().map(|x| x.backend_node().unwrap()).collect();
                b.borrow_with(&self_ref).backend_element().unwrap().insert_list_before(backend_children, before.as_ref().map(|x| x.backend_node().unwrap()));
            }
        }
        // set children
        let _: Vec<_> = self.children.splice(pos..pos, list.clone()).collect();
        // call attached
        if self.attached {
            for child in list {
                let mut child = child.borrow_mut_with(self);
                child.set_attached();
            }
        }
    }
    fn initialize(&mut self, self_weak: VirtualNodeWeak<B>) {
        // set chilren's parent
        self.self_weak = Some(self_weak.clone());
        let self_weak: NodeWeak<B> = self_weak.into();
        for child in self.children.clone() {
            let mut child = child.borrow_mut_with(self);
            child.set_parent(Some(self_weak.clone()));
            child.set_composed_parent(Some(self_weak.clone()));
        }
    }
    fn set_attached(&mut self) {
        if self.attached { return };
        self.attached = true;
        for child in self.children.clone() {
            child.borrow_mut_with(self).set_attached();
        }
    }
    fn set_detached(&mut self) {
        if !self.attached { return };
        self.attached = false;
        for child in self.children.clone() {
            child.borrow_mut_with(self).set_detached();
        }
    }
}
impl<'a, B: Backend> Drop for VirtualNodeRefMut<'a, B> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.c);
        }
    }
}
impl<B: Backend> fmt::Debug for VirtualNode<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.tag_name)
    }
}

pub struct ComponentNode<B: Backend> {
    pub(crate) backend: Rc<B>,
    pub(crate) backend_element: B::BackendElement,
    pub(crate) need_update: Rc<RefCell<Vec<Box<dyn 'static + FnOnce(&mut ComponentNodeRefMut<B>)>>>>,
    pub(crate) scheduler: Rc<Scheduler>,
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
    pub fn with_type<C: Component<B>>(self) -> ComponentRc<B, C> {
        ComponentRc::from(self)
    }
}
impl<B: 'static + Backend> ComponentNodeWeak<B> {
    pub fn with_type<C: Component<B>>(self) -> ComponentWeak<B, C> {
        ComponentWeak::from(self)
    }
}
impl<'a, B: 'static + Backend> ComponentNode<B> {
    define_tree_getter!(node);
    pub fn composed_children(&self) -> Vec<NodeRc<B>> {
        vec![self.shadow_root.clone().into()]
    }
    fn collect_backend_nodes<'b>(&'b self, v: &'b mut Vec<NodeRc<B>>) {
        v.push(self.rc().into());
    }
    fn new_with_children(backend: Rc<B>, scheduler: Rc<Scheduler>, tag_name: &'static str, shadow_root: VirtualNodeRc<B>, children: Vec<NodeRc<B>>, owner: Option<ComponentNodeWeak<B>>) -> Self {
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
    pub fn get_attribute(&self, name: &'static str) -> Option<&str> {
        self.attributes.iter().find(|x| x.0 == name).map(|x| x.1.as_str())
    }
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
    pub fn shadow_root_rc(&self) -> &VirtualNodeRc<B> {
        &self.shadow_root
    }
    pub fn as_component<C: Component<B>>(&self) -> &C {
        self.component.downcast_ref::<C>().unwrap()
    }
    pub fn as_component_mut<C: Component<B>>(&mut self) -> &mut C {
        self.component.downcast_mut::<C>().unwrap()
    }
    pub fn try_as_component<C: 'static + Component<B>>(&self) -> Option<&C> {
        let c: &dyn Any = &self.component;
        c.downcast_ref()
    }
    pub fn try_as_component_mut<C: 'static + Component<B>>(&mut self) -> Option<&mut C> {
        let c: &mut dyn Any = &mut self.component;
        c.downcast_mut()
    }
    pub fn global_events(&self) -> &GlobalEvents<B> {
        &self.global_events
    }
    pub fn global_events_mut(&mut self) -> &mut GlobalEvents<B> {
        &mut self.global_events
    }
}
impl<'a, B: Backend> ComponentNodeRef<'a, B> {
    define_tree_getter!(ref);
    pub fn backend_element(&self) -> &B::BackendElement {
        &self.backend_element
    }
    pub fn with_type<C: Component<B>>(self) -> ComponentRef<'a, B, C> {
        ComponentRef::from(self)
    }
    pub fn shadow_root_ref(&self) -> VirtualNodeRef<B> {
        self.shadow_root.borrow_with(self)
    }
    fn check_marks_cache(&self) {
        if self.marks_cache_dirty.replace(false) {
            let mut map: HashMap<_, NodeRc<B>> = HashMap::new();
            let shadow_root = self.shadow_root_ref();
            dfs_shadow_tree(&shadow_root, &shadow_root.children, &mut |node_rc| {
                let n = node_rc.borrow_with(&shadow_root);
                match n {
                    NodeRef::NativeNode(n) => {
                        if n.mark.len() > 0 && !map.contains_key(&n.mark) {
                            map.insert(n.mark.clone(), node_rc.clone());
                        }
                    },
                    NodeRef::ComponentNode(n) => {
                        if n.mark.len() > 0 && !map.contains_key(&n.mark) {
                            map.insert(n.mark.clone(), node_rc.clone());
                        }
                    },
                    _ => { }
                }
            });
            *self.marks_cache.borrow_mut() = map;
        }
    }
    pub fn marked(&self, r: &str) -> Option<NodeRc<B>> {
        self.check_marks_cache();
        self.marks_cache.borrow().get(r).cloned()
    }
    pub fn marked_native_node(&self, r: &str) -> Option<NativeNodeRc<B>> {
        match self.marked(r) {
            None => None,
            Some(x) => {
                match x {
                    NodeRc::NativeNode(x) => Some(x),
                    _ => None,
                }
            }
        }
    }
    pub fn marked_component_node(&self, r: &str) -> Option<ComponentNodeRc<B>> {
        match self.marked(r) {
            None => None,
            Some(x) => {
                match x {
                    NodeRc::ComponentNode(x) => Some(x),
                    _ => None,
                }
            }
        }
    }
    pub fn marked_component<C: Component<B>>(&self, r: &str) -> Option<ComponentRc<B, C>> {
        self.marked_component_node(r).map(|x| x.with_type::<C>())
    }
    pub fn to_html<T: std::io::Write>(&self, s: &mut T) -> std::io::Result<()> {
        write!(s, "<{}", self.tag_name)?;
        for (name, value) in self.attributes.iter() {
            write!(s, r#" {}="{}""#, name, escape::escape_html(value))?;
        }
        write!(s, ">")?;
        self.shadow_root.borrow_with(self).to_html(s)?;
        write!(s, "</{}>", self.tag_name)
    }
    fn debug_fmt(&self, f: &mut fmt::Formatter<'_>, level: u32) -> fmt::Result {
        for _ in 0..level {
            write!(f, "  ")?;
        }
        let n: &ComponentNode<B> = &**self;
        writeln!(f, "{:?}", n)?;
        n.shadow_root.borrow_with(self).debug_fmt(f, level + 1)?;
        for child in self.children.iter() {
            child.borrow_with(self).debug_fmt(f, level + 1)?;
        }
        Ok(())
    }
}
impl<'a, B: Backend> ComponentNodeRefMut<'a, B> {
    define_tree_getter!(ref mut);
    pub fn backend_element(&self) -> &B::BackendElement {
        &self.backend_element
    }
    pub fn with_type<C: Component<B>>(self) -> ComponentRefMut<'a, B, C> {
        ComponentRefMut::from(self)
    }
    pub fn apply_updates(&mut self) {
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
    pub fn force_apply_updates<C: Component<B>>(&mut self) {
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
    pub fn new_native_node(&mut self, tag_name: &'static str, attributes: Vec<(&'static str, String)>, children: Vec<NodeRc<B>>) -> NativeNodeRc<B> {
        let backend = self.backend().clone();
        let owner = self.self_weak.clone();
        let n = NativeNodeRc {
            c: Rc::new(self.as_me_ref_mut_handle().entrance(NativeNode::new_with_children(backend, tag_name, attributes, children, owner)))
        };
        n.borrow_mut_with(self).initialize(n.downgrade());
        n
    }
    pub fn new_virtual_node(&mut self, tag_name: &'static str, property: VirtualNodeProperty<B>, children: Vec<NodeRc<B>>) -> VirtualNodeRc<B> {
        let backend = self.backend().clone();
        let owner = self.self_weak.clone();
        let n = VirtualNodeRc {
            c: Rc::new(self.as_me_ref_mut_handle().entrance(VirtualNode::new_with_children(backend, tag_name, property, children, owner)))
        };
        n.borrow_mut_with(self).initialize(n.downgrade());
        n
    }
    pub fn new_component_node<C: 'static + Component<B>>(&mut self, tag_name: &'static str, children: Vec<NodeRc<B>>) -> ComponentNodeRc<B> {
        create_component::<_, _, C>(self, self.scheduler.clone(), tag_name, children, self.self_weak.clone())
    }
    pub fn new_text_node<T: ToString>(&mut self, text_content: T) -> TextNodeRc<B> {
        let backend = self.backend().clone();
        let owner = self.self_weak.clone();
        let n = TextNodeRc {
            c: Rc::new(self.as_me_ref_mut_handle().entrance(TextNode::new_with_content(backend, text_content.to_string(), owner)))
        };
        n.borrow_mut_with(self).initialize(n.downgrade());
        n
    }
    fn reassign_slots(&mut self, new_slots: HashMap<&'static str, VirtualNodeRc<B>>) {
        let main_slot_changed = self.slots.get("") != new_slots.get("");
        // clear composed children for slots
        for (name, slot) in new_slots.iter() {
            if *name == "" && !main_slot_changed {
                continue
            }
            let mut slot = slot.borrow_mut_with(self);
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
            let mut slot = slot.borrow_mut_with(self);
            if let VirtualNodeProperty::Slot(_, c) = &mut slot.property {
                c.truncate(0);
            } else {
                unreachable!()
            }
        }
        // re-insert children
        for child in self.children.clone() {
            let child_node_rc = child.clone();
            let put_child_in_slot = |name, mut child: NodeRefMut<_>| {
                match new_slots.get(name) {
                    Some(slot) => {
                        let slot_weak = slot.borrow_with(&child.to_ref()).self_weak.clone().map(|x| x.into());
                        child.set_composed_parent(slot_weak);
                        {
                            let mut slot = slot.borrow_mut_with(&mut child);
                            if let VirtualNodeProperty::Slot(_, c) = &mut slot.property {
                                c.push(child_node_rc);
                            } else {
                                unreachable!()
                            }
                        }
                        let mut nodes = vec![];
                        let child_ref = child.to_ref();
                        child_ref.collect_backend_nodes(&mut nodes);
                        let slot = slot.borrow_with(&child_ref);
                        let before = slot.find_next_sibling(false);
                        let before = before.as_ref().map(|x| x.borrow_with(&child_ref));
                        let nodes: Vec<_> = nodes.iter().map(|x| x.borrow_with(&child_ref)).collect();
                        let nodes = nodes.iter().map(|x| x.backend_node().unwrap()).collect();
                        match slot.find_backend_parent() {
                            Some(parent) => parent.borrow_with(&child_ref).backend_element().unwrap().insert_list_before(nodes, before.as_ref().map(|x| x.backend_node().unwrap())),
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
                        let child_ref = child.to_ref();
                        child_ref.collect_backend_nodes(&mut nodes);
                        let nodes: Vec<_> = nodes.iter().map(|x| x.borrow_with(&child_ref)).collect();
                        let nodes: Vec<_> = nodes.iter().map(|x| x.backend_node().unwrap()).collect();
                        for n in nodes {
                            n.remove_self();
                        }
                    }
                }
            };
            if let NodeRc::VirtualNode(child) = &child {
                let child = child.borrow_mut_with(self);
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
            let child = child.borrow_mut_with(self);
            put_child_in_slot("", child);
        }
    }
    fn check_slots_update(&mut self) {
        // collect slots
        let mut slots = HashMap::new();
        {
            let self_ref = self.to_ref();
            dfs_shadow_tree(&self_ref, &self.shadow_root.borrow_with(&self_ref).children, &mut |n| {
                if let NodeRc::VirtualNode(n) = &n {
                    if let VirtualNodeProperty::Slot(name, _) = n.borrow_with(&self_ref).property {
                        slots.insert(name, n.clone());
                    }
                }
            });
        }
        if self.slots == slots {
            return
        }
        self.reassign_slots(slots);
    }
    fn initialize<C: 'static + Component<B>>(&mut self, self_weak: ComponentNodeWeak<B>) {
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
            child.borrow_mut_with(self).set_parent(Some(self_weak.clone()));
        }
        {
            // initialize shadow root
            let shadow_root_content = <C as ComponentTemplate<B>>::template(self, ComponentTemplateOperation::Init);
            let shadow_root = self.shadow_root.clone();
            let mut shadow_root = unsafe { shadow_root.borrow_mut_unsafe_with(self) };
            shadow_root.set_composed_parent(Some(self_weak.clone()));
            if let Some(shadow_root_content) = shadow_root_content {
                shadow_root.replace_children_list(shadow_root_content);
            }
            // append shadow root
            let mut backend_children = vec![];
            let shadow_root_ref = shadow_root.to_ref();
            shadow_root_ref.collect_backend_nodes(&mut backend_children);
            let backend_children: Vec<_> = backend_children.iter().map(|x| x.borrow_with(&shadow_root_ref)).collect();
            self.backend_element.append_list(backend_children.iter().map(|x| x.backend_node().unwrap()).collect());
        }
        self.check_slots_update();
        <C as Component<B>>::created(&mut self.duplicate().with_type::<C>());
    }
    pub fn set_mark<T: Into<Cow<'static, str>>>(&mut self, r: T) {
        let r = r.into();
        if self.mark == r {
            return;
        }
        self.mark = r;
        if let Some(c) = self.owner() {
            c.borrow_mut_with(self).marks_cache_dirty.set(true);
        }
    }
    pub fn marked(&self, r: &str) -> Option<NodeRc<B>> {
        self.to_ref().marked(r)
    }
    pub fn marked_native_node(&self, r: &str) -> Option<NativeNodeRc<B>> {
        self.to_ref().marked_native_node(r)
    }
    pub fn marked_component_node(&self, r: &str) -> Option<ComponentNodeRc<B>> {
        self.to_ref().marked_component_node(r)
    }
    pub fn marked_component<C: Component<B>>(&self, r: &str) -> Option<ComponentRc<B, C>> {
        self.to_ref().marked_component::<C>(r)
    }
    pub(crate) fn set_attached(&mut self) {
        if self.attached { return };
        self.attached = true;
        self.shadow_root.clone().borrow_mut_with(self).set_attached();
        for child in self.children.clone() {
            child.borrow_mut_with(self).set_attached();
        }
        self.component.attached();
        self.apply_updates();
    }
    pub(crate) fn set_detached(&mut self) {
        if !self.attached { return };
        self.attached = false;
        self.shadow_root.clone().borrow_mut_with(self).set_attached();
        for child in self.children.clone() {
            child.borrow_mut_with(self).set_detached();
        }
        self.component.detached();
    }
    pub(crate) fn update_node(&mut self) {
        self.check_slots_update();
    }
}
impl<'a, B: Backend> Drop for ComponentNodeRefMut<'a, B> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.c);
        }
        self.scheduler.run_tasks();
    }
}
impl<B: Backend> fmt::Debug for ComponentNode<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Component<B>>")
    }
}

pub struct TextNode<B: Backend> {
    pub(crate) backend: Rc<B>,
    pub(crate) backend_element: B::BackendTextNode,
    pub(crate) attached: bool,
    pub(crate) self_weak: Option<TextNodeWeak<B>>,
    pub(crate) text_content: String,
    pub(crate) owner: Option<ComponentNodeWeak<B>>,
    pub(crate) parent: Option<NodeWeak<B>>,
    pub(crate) composed_parent: Option<NodeWeak<B>>,
}
impl<B: Backend> TextNode<B> {
    define_tree_getter!(text);
    fn collect_backend_nodes<'b>(&'b self, v: &'b mut Vec<NodeRc<B>>) {
        v.push(self.rc().into());
    }
    pub fn new_with_content(backend: Rc<B>, text_content: String, owner: Option<ComponentNodeWeak<B>>) -> Self {
        let backend_element = backend.create_text_node(text_content.as_ref());
        TextNode { backend, backend_element, attached: false, self_weak: None, text_content, owner, parent: None, composed_parent: None }
    }
    pub fn text_content(&self) -> &str {
        &self.text_content
    }
    pub fn set_text_content<T: ToString>(&mut self, c: T) {
        self.text_content = c.to_string();
        self.backend_element.set_text_content(&self.text_content);
    }
    fn set_attached(&mut self) {
        if self.attached { return };
        self.attached = true;
    }
    fn set_detached(&mut self) {
        if !self.attached { return };
        self.attached = false;
    }
}
impl<'a, B: Backend> TextNodeRef<'a, B> {
    define_tree_getter!(ref);
    pub fn backend_element(&self) -> &B::BackendTextNode {
        &self.backend_element
    }
    pub fn to_html<T: std::io::Write>(&self, s: &mut T) -> std::io::Result<()> {
        let html = escape::escape_html(&self.text_content);
        if html == "" {
            write!(s, "<!---->")
        } else {
            write!(s, "{}", html)
        }
    }
    fn debug_fmt(&self, f: &mut fmt::Formatter<'_>, level: u32) -> fmt::Result {
        for _ in 0..level {
            write!(f, "  ")?;
        }
        let n: &TextNode<B> = &**self;
        writeln!(f, "{:?}", n)?;
        Ok(())
    }
}
impl<'a, B: Backend> TextNodeRefMut<'a, B> {
    define_tree_getter!(ref mut);
    pub fn backend_element(&self) -> &B::BackendTextNode {
        &self.backend_element
    }
    fn initialize(&mut self, self_weak: TextNodeWeak<B>) {
        self.self_weak = Some(self_weak);
    }
}
impl<'a, B: Backend> Drop for TextNodeRefMut<'a, B> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.c);
        }
    }
}
impl<B: Backend> fmt::Debug for TextNode<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?}", self.text_content)
    }
}

pub enum NodeRc<B: Backend> {
    NativeNode(NativeNodeRc<B>),
    VirtualNode(VirtualNodeRc<B>),
    ComponentNode(ComponentNodeRc<B>),
    TextNode(TextNodeRc<B>),
}
impl<B: Backend> Clone for NodeRc<B> {
    fn clone(&self) -> Self {
        match self {
            NodeRc::NativeNode(x) => NodeRc::NativeNode(x.clone()),
            NodeRc::VirtualNode(x) => NodeRc::VirtualNode(x.clone()),
            NodeRc::ComponentNode(x) => NodeRc::ComponentNode(x.clone()),
            NodeRc::TextNode(x) => NodeRc::TextNode(x.clone()),
        }
    }
}
impl<B: Backend> PartialEq for NodeRc<B> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            NodeRc::NativeNode(x) => if let NodeRc::NativeNode(y) = other { x == y } else { false },
            NodeRc::VirtualNode(x) => if let NodeRc::VirtualNode(y) = other { x == y } else { false },
            NodeRc::ComponentNode(x) => if let NodeRc::ComponentNode(y) = other { x == y } else { false },
            NodeRc::TextNode(x) => if let NodeRc::TextNode(y) = other { x == y } else { false },
        }
    }
}
impl<B: Backend> NodeRc<B> {
    pub fn borrow<'a>(&'a self) -> NodeRef<'a, B> {
        match self {
            NodeRc::NativeNode(x) => NodeRef::NativeNode(x.borrow()),
            NodeRc::VirtualNode(x) => NodeRef::VirtualNode(x.borrow()),
            NodeRc::ComponentNode(x) => NodeRef::ComponentNode(x.borrow()),
            NodeRc::TextNode(x) => NodeRef::TextNode(x.borrow()),
        }
    }
    pub fn borrow_mut<'a>(&'a self) -> NodeRefMut<'a, B> {
        match self {
            NodeRc::NativeNode(x) => NodeRefMut::NativeNode(x.borrow_mut()),
            NodeRc::VirtualNode(x) => NodeRefMut::VirtualNode(x.borrow_mut()),
            NodeRc::ComponentNode(x) => NodeRefMut::ComponentNode(x.borrow_mut()),
            NodeRc::TextNode(x) => NodeRefMut::TextNode(x.borrow_mut()),
        }
    }
    pub fn borrow_with<'a: 'b, 'b, U>(&'b self, source: &'b U) -> NodeRef<'b, B> where U: ElementRef<'a, B> {
        match self {
            NodeRc::NativeNode(x) => NodeRef::NativeNode(x.borrow_with(source)),
            NodeRc::VirtualNode(x) => NodeRef::VirtualNode(x.borrow_with(source)),
            NodeRc::ComponentNode(x) => NodeRef::ComponentNode(x.borrow_with(source)),
            NodeRc::TextNode(x) => NodeRef::TextNode(x.borrow_with(source)),
        }
    }
    pub fn borrow_mut_with<'a: 'b, 'b, U>(&'b self, source: &'b mut U) -> NodeRefMut<'b, B> where U: ElementRefMut<'a, B> {
        match self {
            NodeRc::NativeNode(x) => NodeRefMut::NativeNode(x.borrow_mut_with(source)),
            NodeRc::VirtualNode(x) => NodeRefMut::VirtualNode(x.borrow_mut_with(source)),
            NodeRc::ComponentNode(x) => NodeRefMut::ComponentNode(x.borrow_mut_with(source)),
            NodeRc::TextNode(x) => NodeRefMut::TextNode(x.borrow_mut_with(source)),
        }
    }
    pub unsafe fn borrow_mut_unsafe_with<'a: 'b, 'b, 'c, U>(&'c self, source: &'b mut U) -> NodeRefMut<'c, B> where U: ElementRefMut<'a, B> {
        match self {
            NodeRc::NativeNode(x) => NodeRefMut::NativeNode(x.borrow_mut_unsafe_with(source)),
            NodeRc::VirtualNode(x) => NodeRefMut::VirtualNode(x.borrow_mut_unsafe_with(source)),
            NodeRc::ComponentNode(x) => NodeRefMut::ComponentNode(x.borrow_mut_unsafe_with(source)),
            NodeRc::TextNode(x) => NodeRefMut::TextNode(x.borrow_mut_unsafe_with(source)),
        }
    }
}

pub enum NodeWeak<B: Backend> {
    NativeNode(NativeNodeWeak<B>),
    VirtualNode(VirtualNodeWeak<B>),
    ComponentNode(ComponentNodeWeak<B>),
    TextNode(TextNodeWeak<B>),
}
impl<B: Backend> Clone for NodeWeak<B> {
    fn clone(&self) -> Self {
        match self {
            NodeWeak::NativeNode(x) => NodeWeak::NativeNode(x.clone()),
            NodeWeak::VirtualNode(x) => NodeWeak::VirtualNode(x.clone()),
            NodeWeak::ComponentNode(x) => NodeWeak::ComponentNode(x.clone()),
            NodeWeak::TextNode(x) => NodeWeak::TextNode(x.clone()),
        }
    }
}
impl<B: Backend> NodeWeak<B> {
    pub fn upgrade(&self) -> Option<NodeRc<B>> {
        match self {
            NodeWeak::NativeNode(x) => x.upgrade().map(|x| NodeRc::NativeNode(x)),
            NodeWeak::VirtualNode(x) => x.upgrade().map(|x| NodeRc::VirtualNode(x)),
            NodeWeak::ComponentNode(x) => x.upgrade().map(|x| NodeRc::ComponentNode(x)),
            NodeWeak::TextNode(x) => x.upgrade().map(|x| NodeRc::TextNode(x)),
        }
    }
}

pub enum NodeRef<'a, B: Backend> {
    NativeNode(NativeNodeRef<'a, B>),
    VirtualNode(VirtualNodeRef<'a, B>),
    ComponentNode(ComponentNodeRef<'a, B>),
    TextNode(TextNodeRef<'a, B>),
}
pub enum NodeRefMut<'a, B: Backend> {
    NativeNode(NativeNodeRefMut<'a, B>),
    VirtualNode(VirtualNodeRefMut<'a, B>),
    ComponentNode(ComponentNodeRefMut<'a, B>),
    TextNode(TextNodeRefMut<'a, B>),
}
impl<'a, B: Backend> NodeRef<'a, B> {
    pub fn backend_node<'b>(&'b self) -> Option<BackendNodeRef<'b, B>> {
        match self {
            Self::NativeNode(x) => Some(BackendNodeRef::Element(&x.backend_element)),
            Self::VirtualNode(_) => None,
            Self::ComponentNode(x) => Some(BackendNodeRef::Element(&x.backend_element)),
            Self::TextNode(x) => Some(BackendNodeRef::TextNode(&x.backend_element)),
        }
    }
    pub fn backend_element<'b>(&'b self) -> Option<&B::BackendElement> {
        match self {
            Self::NativeNode(x) => Some(&x.backend_element),
            Self::VirtualNode(_) => None,
            Self::ComponentNode(x) => Some(&x.backend_element),
            Self::TextNode(_) => None,
        }
    }
    fn collect_backend_nodes<'b>(&'b self, v: &'b mut Vec<NodeRc<B>>) {
        match self {
            Self::NativeNode(x) => x.collect_backend_nodes(v),
            Self::VirtualNode(x) => x.collect_backend_nodes(v),
            Self::ComponentNode(x) => x.collect_backend_nodes(v),
            Self::TextNode(x) => x.collect_backend_nodes(v),
        }
    }
    pub fn owner(&self) -> Option<ComponentNodeRc<B>> {
        match self {
            Self::NativeNode(x) => x.owner(),
            Self::VirtualNode(x) => x.owner(),
            Self::ComponentNode(x) => x.owner(),
            Self::TextNode(x) => x.owner(),
        }
    }
    pub fn parent(&self) -> Option<NodeRc<B>> {
        match self {
            Self::NativeNode(x) => x.parent(),
            Self::VirtualNode(x) => x.parent(),
            Self::ComponentNode(x) => x.parent(),
            Self::TextNode(x) => x.parent(),
        }
    }
    pub fn composed_parent(&self) -> Option<NodeRc<B>> {
        match self {
            Self::NativeNode(x) => x.parent(),
            Self::VirtualNode(x) => x.parent(),
            Self::ComponentNode(x) => x.parent(),
            Self::TextNode(x) => x.parent(),
        }
    }
    pub fn composed_children(&self) -> Vec<NodeRc<B>> {
        match self {
            Self::NativeNode(x) => x.composed_children(),
            Self::VirtualNode(x) => x.composed_children(),
            Self::ComponentNode(x) => x.composed_children(),
            Self::TextNode(_) => vec![],
        }
    }
    fn find_next_sibling(&self, include_self: bool) -> Option<NodeRc<B>> {
        match self {
            Self::NativeNode(x) => x.find_next_sibling(include_self),
            Self::VirtualNode(x) => x.find_next_sibling(include_self),
            Self::ComponentNode(x) => x.find_next_sibling(include_self),
            Self::TextNode(x) => x.find_next_sibling(include_self),
        }
    }
    fn find_next_backend_child(&self, from_index: usize) -> Option<NodeRc<B>> {
        let children = self.composed_children().into_iter().skip(from_index);
        for child in children {
            match child {
                NodeRc::NativeNode(_) => {
                    return Some(child)
                },
                NodeRc::VirtualNode(_) => { },
                NodeRc::ComponentNode(_) => {
                    return Some(child)
                },
                NodeRc::TextNode(_) => {
                    return Some(child)
                },
            }
            match child.borrow_with(self).find_next_backend_child(0) {
                None => { },
                Some(x) => {
                    return Some(x)
                },
            }
        }
        None
    }
    pub fn to_html<T: std::io::Write>(&self, s: &mut T) -> std::io::Result<()> {
        match self {
            Self::NativeNode(x) => x.to_html(s),
            Self::VirtualNode(x) => x.to_html(s),
            Self::ComponentNode(x) => x.to_html(s),
            Self::TextNode(x) => x.to_html(s),
        }
    }
    fn debug_fmt(&self, f: &mut fmt::Formatter<'_>, level: u32) -> fmt::Result {
        match self {
            Self::NativeNode(x) => x.debug_fmt(f, level),
            Self::VirtualNode(x) => x.debug_fmt(f, level),
            Self::ComponentNode(x) => x.debug_fmt(f, level),
            Self::TextNode(x) => x.debug_fmt(f, level),
        }
    }
}
impl<'a, B: Backend> fmt::Debug for NodeRef<'a, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug_fmt(f, 0)
    }
}
impl<'a, B: Backend> NodeRefMut<'a, B> {
    pub(crate) fn backend_node_mut<'b>(&'b mut self) -> Option<BackendNodeRefMut<'b, B>> {
        match self {
            Self::NativeNode(x) => Some(BackendNodeRefMut::Element(&mut x.backend_element)),
            Self::VirtualNode(_) => None,
            Self::ComponentNode(x) => Some(BackendNodeRefMut::Element(&mut x.backend_element)),
            Self::TextNode(x) => Some(BackendNodeRefMut::TextNode(&mut x.backend_element)),
        }
    }
    pub fn owner(&self) -> Option<ComponentNodeRc<B>> {
        match self {
            Self::NativeNode(x) => x.owner(),
            Self::VirtualNode(x) => x.owner(),
            Self::ComponentNode(x) => x.owner(),
            Self::TextNode(x) => x.owner(),
        }
    }
    fn set_parent(&mut self, p: Option<NodeWeak<B>>) {
        match self {
            NodeRefMut::NativeNode(x) => x.set_parent(p),
            NodeRefMut::VirtualNode(x) => x.set_parent(p),
            NodeRefMut::ComponentNode(x) => x.set_parent(p),
            NodeRefMut::TextNode(x) => x.set_parent(p),
        }
    }
    fn set_composed_parent(&mut self, p: Option<NodeWeak<B>>) {
        match self {
            NodeRefMut::NativeNode(x) => x.set_composed_parent(p),
            NodeRefMut::VirtualNode(x) => x.set_composed_parent(p),
            NodeRefMut::ComponentNode(x) => x.set_composed_parent(p),
            NodeRefMut::TextNode(x) => x.set_composed_parent(p),
        }
    }
    fn set_attached(&mut self) {
        match self {
            NodeRefMut::NativeNode(x) => x.set_attached(),
            NodeRefMut::VirtualNode(x) => x.set_attached(),
            NodeRefMut::ComponentNode(x) => x.set_attached(),
            NodeRefMut::TextNode(x) => x.set_attached(),
        }
    }
    fn set_detached(&mut self) {
        match self {
            NodeRefMut::NativeNode(x) => x.set_detached(),
            NodeRefMut::VirtualNode(x) => x.set_detached(),
            NodeRefMut::ComponentNode(x) => x.set_detached(),
            NodeRefMut::TextNode(x) => x.set_detached(),
        }
    }
    pub fn duplicate<'b>(&'b mut self) -> NodeRefMut<'b, B> where 'a: 'b {
        match self {
            Self::NativeNode(x) => NodeRefMut::NativeNode(x.duplicate()),
            Self::VirtualNode(x) => NodeRefMut::VirtualNode(x.duplicate()),
            Self::ComponentNode(x) => NodeRefMut::ComponentNode(x.duplicate()),
            Self::TextNode(x) => NodeRefMut::TextNode(x.duplicate()),
        }
    }
    pub fn to_ref<'b>(&'b self) -> NodeRef<'b, B> where 'a: 'b {
        match self {
            Self::NativeNode(x) => NodeRef::NativeNode(x.to_ref()),
            Self::VirtualNode(x) => NodeRef::VirtualNode(x.to_ref()),
            Self::ComponentNode(x) => NodeRef::ComponentNode(x.to_ref()),
            Self::TextNode(x) => NodeRef::TextNode(x.to_ref()),
        }
    }
    pub fn to_html<T: std::io::Write>(&self, s: &mut T) -> std::io::Result<()> {
        self.to_ref().to_html(s)
    }
    fn debug_fmt(&self, f: &mut fmt::Formatter<'_>, level: u32) -> fmt::Result {
        match self {
            Self::NativeNode(x) => x.debug_fmt(f, level),
            Self::VirtualNode(x) => x.debug_fmt(f, level),
            Self::ComponentNode(x) => x.debug_fmt(f, level),
            Self::TextNode(x) => x.debug_fmt(f, level),
        }
    }
}
impl<'a, B: Backend> fmt::Debug for NodeRefMut<'a, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug_fmt(f, 0)
    }
}

pub type TemplateNodeFn<B, T> = Box<dyn Fn(&T) -> NodeRc<B>>;

pub trait ElementRef<'a, B: Backend> {
    fn backend(&self) -> &Rc<B>;
    fn as_me_ref_handle(&self) -> &MeRefHandle<'a>;
}
pub trait ElementRefMut<'a, B: Backend> {
    fn backend(&self) -> &Rc<B>;
    fn as_me_ref_mut_handle<'b>(&'b mut self) -> &'b mut MeRefMutHandle<'a> where 'a: 'b;
}

impl<'a, B: Backend> ElementRef<'a, B> for NodeRef<'a, B> {
    fn backend(&self) -> &Rc<B> {
        match self {
            Self::NativeNode(x) => &x.backend,
            Self::VirtualNode(x) => &x.backend,
            Self::ComponentNode(x) => &x.backend,
            Self::TextNode(x) => &x.backend,
        }
    }
    fn as_me_ref_handle(&self) -> &MeRefHandle<'a> {
        match self {
            Self::NativeNode(x) => x.as_me_ref_handle(),
            Self::VirtualNode(x) => x.as_me_ref_handle(),
            Self::ComponentNode(x) => x.as_me_ref_handle(),
            Self::TextNode(x) => x.as_me_ref_handle(),
        }
    }
}
impl<'a, B: Backend> ElementRefMut<'a, B> for NodeRefMut<'a, B> {
    fn backend(&self) -> &Rc<B> {
        match self {
            Self::NativeNode(x) => &x.backend,
            Self::VirtualNode(x) => &x.backend,
            Self::ComponentNode(x) => &x.backend,
            Self::TextNode(x) => &x.backend,
        }
    }
    fn as_me_ref_mut_handle<'b>(&'b mut self) -> &'b mut MeRefMutHandle<'a> where 'a: 'b {
        match self {
            Self::NativeNode(x) => x.as_me_ref_mut_handle(),
            Self::VirtualNode(x) => x.as_me_ref_mut_handle(),
            Self::ComponentNode(x) => x.as_me_ref_mut_handle(),
            Self::TextNode(x) => x.as_me_ref_mut_handle(),
        }
    }
}

macro_rules! some_node_def {
    ($t: ident, $rc: ident, $weak: ident, $r: ident, $rm: ident) => {
        pub struct $rc<B: Backend> {
            c: Rc<MeCell<$t<B>>>
        }
        impl<B: Backend> $rc<B> {
            #[allow(dead_code)]
            pub(crate) fn new_with_me_cell_group(c: $t<B>) -> Self {
                Self {
                    c: Rc::new(MeCell::new_group(c))
                }
            }
            pub fn borrow<'a>(&'a self) -> $r<'a, B> {
                $r { c: self.c.borrow() }
            }
            pub fn borrow_mut<'a>(&'a self) -> $rm<'a, B> {
                $rm { c: ManuallyDrop::new(self.c.borrow_mut()) }
            }
            pub fn borrow_with<'a: 'b, 'b, U>(&'b self, source: &'b U) -> $r<'b, B> where U: ElementRef<'a, B> {
                $r { c: self.c.borrow_with_handle(source.as_me_ref_handle()) }
            }
            pub fn borrow_mut_with<'a: 'b, 'b, U>(&'b self, source: &'b mut U) -> $rm<'b, B> where U: ElementRefMut<'a, B> {
                $rm { c: ManuallyDrop::new(self.c.borrow_mut_with_handle(source.as_me_ref_mut_handle())) }
            }
            pub unsafe fn borrow_mut_unsafe_with<'a: 'b, 'b, 'c, U>(&'c self, source: &'b mut U) -> $rm<'c, B> where U: ElementRefMut<'a, B> {
                $rm { c: ManuallyDrop::new(self.c.borrow_mut_unsafe_with_handle(source.as_me_ref_mut_handle())) }
            }
            pub fn downgrade(&self) -> $weak<B> {
                $weak { c: Rc::downgrade(&self.c) }
            }
        }
        impl<B: Backend> Clone for $rc<B> {
            fn clone(&self) -> Self {
                Self { c: self.c.clone() }
            }
        }
        impl<B: Backend> PartialEq for $rc<B> {
            fn eq(&self, other: &Self) -> bool {
                Rc::ptr_eq(&self.c, &other.c)
            }
        }
        impl<B: Backend> From<$rc<B>> for NodeRc<B> {
            fn from(s: $rc<B>) -> Self {
                NodeRc::$t(s)
            }
        }

        pub struct $weak<B: Backend> {
            c: Weak<MeCell<$t<B>>>
        }
        impl<B: Backend> $weak<B> {
            pub fn upgrade(&self) -> Option<$rc<B>> {
                self.c.upgrade().map(|x| {
                    $rc { c: x }
                })
            }
        }
        impl<B: Backend> Clone for $weak<B> {
            fn clone(&self) -> Self {
                Self { c: self.c.clone() }
            }
        }
        impl<B: Backend> From<$weak<B>> for NodeWeak<B> {
            fn from(s: $weak<B>) -> Self {
                NodeWeak::$t(s)
            }
        }

        pub struct $r<'a, B: Backend> {
            c: MeRef<'a, $t<B>>
        }
        impl<'a, B: Backend> $r<'a, B> {
            pub fn duplicate<'b>(&'b self) -> $r<'b, B> {
                $r {
                    c: self.c.duplicate()
                }
            }
        }
        impl<'a, B: Backend> Deref for $r<'a, B> {
            type Target = $t<B>;
            fn deref(&self) -> &$t<B> {
                &*self.c
            }
        }
        impl<'a, B: Backend> fmt::Debug for $r<'a, B> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.debug_fmt(f, 0)
            }
        }
        impl<'a, B: Backend> ElementRef<'a, B> for $r<'a, B> {
            fn backend(&self) -> &Rc<B> {
                &self.backend
            }
            fn as_me_ref_handle(&self) -> &MeRefHandle<'a> {
                self.c.handle()
            }
        }
        impl<'a, B: Backend> From<$r<'a, B>> for NodeRef<'a, B> {
            fn from(s: $r<'a, B>) -> Self {
                NodeRef::$t(s)
            }
        }

        pub struct $rm<'a, B: Backend> {
            c: ManuallyDrop<MeRefMut<'a, $t<B>>>
        }
        impl<'a, B: Backend> $rm<'a, B> {
            pub fn duplicate<'b>(&'b mut self) -> $rm<'b, B> {
                $rm {
                    c: ManuallyDrop::new(self.c.duplicate())
                }
            }
            pub fn to_ref<'b>(&'b self) -> $r<'b, B> where 'a: 'b {
                $r { c: self.c.to_ref() }
            }
            fn debug_fmt(&self, f: &mut fmt::Formatter<'_>, level: u32) -> fmt::Result {
                self.to_ref().debug_fmt(f, level)
            }
        }
        impl<'a, B: Backend> Deref for $rm<'a, B> {
            type Target = $t<B>;
            fn deref(&self) -> &$t<B> {
                &*self.c
            }
        }
        impl<'a, B: Backend> DerefMut for $rm<'a, B> {
            fn deref_mut(&mut self) -> &mut $t<B> {
                &mut *self.c
            }
        }
        impl<'a, B: Backend> fmt::Debug for $rm<'a, B> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.debug_fmt(f, 0)
            }
        }
        impl<'a, B: Backend> ElementRefMut<'a, B> for $rm<'a, B> {
            fn backend(&self) -> &Rc<B> {
                &self.backend
            }
            fn as_me_ref_mut_handle<'b>(&'b mut self) -> &'b mut MeRefMutHandle<'a> where 'a: 'b {
                self.c.handle_mut()
            }
        }
        impl<'a, B: Backend> From<$rm<'a, B>> for NodeRefMut<'a, B> {
            fn from(s: $rm<'a, B>) -> Self {
                NodeRefMut::$t(s)
            }
        }

        impl<B: Backend> $t<B> {
            pub fn rc(&self) -> $rc<B> {
                match &self.self_weak {
                    None => unreachable!(),
                    Some(x) => x.upgrade().unwrap(),
                }
            }
        }
    }
}
some_node_def!(NativeNode, NativeNodeRc, NativeNodeWeak, NativeNodeRef, NativeNodeRefMut);
some_node_def!(VirtualNode, VirtualNodeRc, VirtualNodeWeak, VirtualNodeRef, VirtualNodeRefMut);
some_node_def!(ComponentNode, ComponentNodeRc, ComponentNodeWeak, ComponentNodeRef, ComponentNodeRefMut);
some_node_def!(TextNode, TextNodeRc, TextNodeWeak, TextNodeRef, TextNodeRefMut);
