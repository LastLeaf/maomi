use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};
use std::collections::HashMap;
use std::fmt;
use std::any::Any;
use std::ops::Range;
use me_cell::*;

use super::{Component, ComponentTemplate};
use super::backend::*;

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

pub(crate) fn create_component<'a, B: Backend, T: ElementRefMut<'a, B>, C: 'static + Component>(n: &mut T, tag_name: &'static str, component: Box<C>, slot: String, children: Vec<NodeRc<B>>, owner: Option<ComponentNodeWeak<B>>) -> ComponentNodeRc<B> {
    let backend = n.backend().clone();
    let shadow_root = VirtualNodeRc {
        c: Rc::new(n.as_me_ref_mut_handle().entrance(VirtualNode::new_with_children(backend, "shadow-root", VirtualNodeProperty::ShadowRoot, vec![], owner.clone())))
    };
    shadow_root.borrow_mut_with(n).initialize(shadow_root.downgrade());
    let backend = n.backend().clone();
    let ret = ComponentNodeRc {
        c: Rc::new(n.as_me_ref_mut_handle().entrance(ComponentNode::new_with_children(backend, tag_name, component, shadow_root.clone(), slot, children, owner)))
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
    };
    (node) => {
        define_tree_getter!(text);
        pub fn children(&self) -> &Vec<NodeRc<B>> {
            &self.children
        }
    };
    (ref) => {
        fn find_next_backend_sibling(&self, include_self: bool) -> Option<<B as Backend>::BackendNode> {
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
                            NodeRc::VirtualNode(x) => x.borrow_with(self).find_next_backend_sibling(false),
                            NodeRc::ComponentNode(_) => None,
                            _ => unreachable!()
                        }
                    }
                }
            }
        }
        fn find_backend_parent(&self) -> Option<<<B as Backend>::BackendNode as BackendNode>::BackendElement> {
            match self.composed_parent() {
                None => None,
                Some(composed_parent) => match composed_parent {
                    NodeRc::NativeNode(x) => Some(x.borrow_with(self).backend_element.ref_clone()),
                    NodeRc::VirtualNode(x) => x.borrow_with(self).find_backend_parent(),
                    NodeRc::ComponentNode(x) => Some(x.borrow_with(self).backend_element.ref_clone()),
                    _ => unreachable!()
                }
            }
        }
    };
    (ref mut) => {
    };
}

pub struct NativeNode<B: Backend> {
    pub(crate) backend: Rc<B>,
    pub(crate) backend_element: <<B as Backend>::BackendNode as BackendNode>::BackendElement,
    pub(crate) self_weak: Option<NativeNodeWeak<B>>,
    pub(crate) tag_name: &'static str,
    pub(crate) attributes: Vec<(&'static str, String)>,
    pub(crate) children: Vec<NodeRc<B>>,
    pub(crate) slot: String,
    pub(crate) owner: Option<ComponentNodeWeak<B>>,
    pub(crate) parent: Option<NodeWeak<B>>,
    pub(crate) composed_parent: Option<NodeWeak<B>>,
}
impl<B: Backend> NativeNode<B> {
    define_tree_getter!(node);
    pub fn composed_children(&self) -> Vec<NodeRc<B>> {
        self.children.clone()
    }
    pub(crate) fn new_with_children(backend: Rc<B>, tag_name: &'static str, attributes: Vec<(&'static str, String)>, slot: String, children: Vec<NodeRc<B>>, owner: Option<ComponentNodeWeak<B>>) -> Self {
        let backend_element = backend.create_element(tag_name);
        NativeNode { backend, backend_element, self_weak: None, tag_name, attributes, slot, children, owner, parent: None, composed_parent: None }
    }
    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
    pub fn get_attribute(&self, name: &'static str) -> Option<&str> {
        self.attributes.iter().find(|x| x.0 == name).map(|x| x.1.as_str())
    }
    pub fn set_attribute<T: ToString>(&mut self, name: &'static str, value: T) {
        match self.attributes.iter_mut().find(|x| x.0 == name) {
            Some(x) => {
                x.1 = value.to_string();
                return
            },
            None => { }
        }
        self.attributes.push((name, value.to_string()))
    }
}
impl<'a, B: Backend> NativeNodeRef<'a, B> {
    define_tree_getter!(ref);
    fn collect_backend_nodes<'b>(&'b self, v: &'b mut Vec<<B as Backend>::BackendNode>) {
        v.push(self.backend_element.ref_clone().into_node())
    }
    pub fn backend_element(&self) -> &<<B as Backend>::BackendNode as BackendNode>::BackendElement {
        &self.backend_element
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
    pub fn backend_element(&self) -> &<<B as Backend>::BackendNode as BackendNode>::BackendElement {
        &self.backend_element
    }
    fn initialize(&mut self, self_weak: NativeNodeWeak<B>) {
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
        self.backend_element.append_list(backend_children);
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
    Branch(usize),
    List(Box<dyn Any>),
}

pub struct VirtualNode<B: Backend> {
    pub(crate) backend: Rc<B>,
    pub(crate) self_weak: Option<VirtualNodeWeak<B>>,
    pub(crate) tag_name: &'static str,
    pub(crate) property: VirtualNodeProperty<B>,
    pub(crate) children: Vec<NodeRc<B>>, // for slot node, children means the children
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
        VirtualNode { backend, self_weak: None, tag_name, property, children, owner, parent: None, composed_parent: None }
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
    fn collect_backend_nodes<'b>(&'b self, v: &'b mut Vec<<B as Backend>::BackendNode>) {
        for child in self.composed_children().iter() {
            child.borrow_with(self).collect_backend_nodes(v);
        }
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
    pub fn replace_children_list(&mut self, mut list: Vec<NodeRc<B>>) {
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
        let self_ref = self.to_ref();
        if let Some(b) = self_ref.find_backend_parent() {
            // remove old backend children
            let mut backend_children = vec![];
            for n in list {
                n.borrow_with(&self_ref).collect_backend_nodes(&mut backend_children);
            }
            if backend_children.len() > 0 {
                b.remove_list(backend_children);
            }
            // insert new backend children
            let mut backend_children = vec![];
            for n in self.children.iter() {
                n.borrow_with(&self_ref).collect_backend_nodes(&mut backend_children);
            }
            let before = self_ref.find_next_backend_sibling(false);
            b.insert_list_before(backend_children, before);
        }
    }
    pub fn remove_range(&mut self, r: Range<usize>) -> Vec<NodeRc<B>> {
        // set children
        let removed: Vec<NodeRc<B>> = self.children.splice(r, vec![]).collect();
        // remove old children's parent
        for child in removed.iter() {
            let mut child = child.borrow_mut_with(self);
            child.set_parent(None);
            child.set_composed_parent(None);
        }
        // remove in backend
        let self_ref = self.to_ref();
        if let Some(b) = self_ref.find_backend_parent() {
            let mut backend_children = vec![];
            for n in removed.iter() {
                n.borrow_with(&self_ref).collect_backend_nodes(&mut backend_children);
            }
            b.remove_list(backend_children);
        }
        removed
    }
    pub fn insert_list(&mut self, pos: usize, list: Vec<NodeRc<B>>) {
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
                    Some(x) => x.borrow_with(&self_ref).find_next_backend_sibling(true),
                    None => None,
                };
                b.insert_list_before(backend_children, before);
            }
        }
        // set children
        let _: Vec<_> = self.children.splice(pos..pos, list).collect();
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
}
impl<B: Backend> fmt::Debug for VirtualNode<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.tag_name)
    }
}

pub struct ComponentNode<B: Backend> {
    pub(crate) backend: Rc<B>,
    pub(crate) backend_element: <<B as Backend>::BackendNode as BackendNode>::BackendElement,
    pub(crate) component: Box<dyn Any>,
    pub(crate) self_weak: Option<ComponentNodeWeak<B>>,
    pub(crate) shadow_root: VirtualNodeRc<B>,
    pub(crate) children: Vec<NodeRc<B>>,
    pub(crate) slot: String,
    pub(crate) slots: HashMap<&'static str, VirtualNodeRc<B>>,
    pub(crate) owner: Option<ComponentNodeWeak<B>>,
    pub(crate) parent: Option<NodeWeak<B>>,
    pub(crate) composed_parent: Option<NodeWeak<B>>,
}
impl<'a, B: 'static + Backend> ComponentNode<B> {
    define_tree_getter!(node);
    pub fn composed_children(&self) -> Vec<NodeRc<B>> {
        vec![self.shadow_root.clone().into()]
    }
    fn collect_backend_nodes<'b>(&'b self, v: &'b mut Vec<<B as Backend>::BackendNode>) {
        v.push(self.backend_element.ref_clone().into_node());
    }
    fn new_with_children(backend: Rc<B>, tag_name: &'static str, component: Box<dyn Any>, shadow_root: VirtualNodeRc<B>, slot: String, children: Vec<NodeRc<B>>, owner: Option<ComponentNodeWeak<B>>) -> Self {
        let backend_element = backend.create_element(tag_name);
        ComponentNode { backend, backend_element, component, self_weak: None, shadow_root, slot, children, slots: HashMap::new(), owner, parent: None, composed_parent: None }
    }
    pub fn shadow_root_rc(&self) -> &VirtualNodeRc<B> {
        &self.shadow_root
    }
    pub fn as_component<C: Component>(&self) -> &C {
        self.component.downcast_ref().unwrap()
    }
    pub fn as_component_mut<C: Component>(&mut self) -> &mut C {
        self.component.downcast_mut().unwrap()
    }
    pub fn try_as_component<C: 'static + Component>(&self) -> Option<&C> {
        let c: &dyn Any = &self.component;
        c.downcast_ref()
    }
    pub fn try_as_component_mut<C: 'static + Component>(&mut self) -> Option<&mut C> {
        let c: &mut dyn Any = &mut self.component;
        c.downcast_mut()
    }
}
impl<'a, B: Backend> ComponentNodeRef<'a, B> {
    define_tree_getter!(ref);
    pub fn backend_element(&self) -> &<<B as Backend>::BackendNode as BackendNode>::BackendElement {
        &self.backend_element
    }
    pub fn shadow_root(&self) -> VirtualNodeRef<B> {
        self.shadow_root.borrow_with(self)
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
    pub fn backend_element(&self) -> &<<B as Backend>::BackendNode as BackendNode>::BackendElement {
        &self.backend_element
    }
    pub fn new_native_node(&mut self, tag_name: &'static str, attributes: Vec<(&'static str, String)>, slot: String, children: Vec<NodeRc<B>>) -> NativeNodeRc<B> {
        let backend = self.backend().clone();
        let owner = self.self_weak.clone();
        let n = NativeNodeRc {
            c: Rc::new(self.as_me_ref_mut_handle().entrance(NativeNode::new_with_children(backend, tag_name, attributes, slot, children, owner)))
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
    pub fn new_component_node<C: 'static + Component>(&mut self, tag_name: &'static str, component: Box<C>, slot: String, children: Vec<NodeRc<B>>) -> ComponentNodeRc<B> {
        create_component(self, tag_name, component, slot, children, self.self_weak.clone())
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
    fn reassign_slots(&mut self) {
        let mut slots: HashMap<&'static str, Vec<NodeRc<B>>> = self.slots.keys().map(|name| {
            (*name, vec![])
        }).collect();
        macro_rules! collect_into_slots_def {
            ($r: ident, $n: ident) => {
                fn $n<B: Backend>(s: &$r<B>, slots: &mut HashMap<&'static str, Vec<NodeRc<B>>>) {
                    for child in s.children.iter() {
                        let slot = match child.borrow_with(s) {
                            NodeRef::NativeNode(x) => slots.get_mut(x.slot.as_str()),
                            NodeRef::VirtualNode(_) => slots.get_mut(""),
                            NodeRef::ComponentNode(x) => slots.get_mut(x.slot.as_str()),
                            NodeRef::TextNode(_) => slots.get_mut(""),
                        };
                        match slot {
                            None => { },
                            Some(slot) => {
                                slot.push(child.clone());
                            },
                        }
                        if let NodeRef::VirtualNode(x) = child.borrow_with(s) {
                            collect_into_slots_virtual(&x, slots);
                        }
                    }
                }
            }
        }
        collect_into_slots_def!(ComponentNodeRef, collect_into_slots);
        collect_into_slots_def!(VirtualNodeRef, collect_into_slots_virtual);
        collect_into_slots(&self.to_ref().into(), &mut slots);
        for (name, mut list) in slots {
            let slot_node = self.slots[name].clone();
            let self_weak = slot_node.borrow_with(&self.to_ref()).self_weak.clone().map(|x| x.into());
            for child in list.iter() {
                child.borrow_mut_with(self).set_composed_parent(self_weak.clone());
            }
            let mut slot_node = slot_node.borrow_mut_with(self);
            let before = slot_node.to_ref().find_next_backend_sibling(false);
            match &mut slot_node.property {
                VirtualNodeProperty::Slot(_, children) => {
                    std::mem::swap(children, &mut list);
                },
                _ => unreachable!()
            }
            let slot_node_ref = slot_node.to_ref();
            match slot_node_ref.find_backend_parent() {
                None => { },
                Some(x) => {
                    let mut n = vec![];
                    slot_node_ref.collect_backend_nodes(&mut n);
                    x.insert_list_before(n, before);
                }
            }
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
        std::mem::swap(&mut self.slots, &mut slots);
        self.reassign_slots();
    }
    fn initialize<C: 'static + Component>(&mut self, self_weak: ComponentNodeWeak<B>) {
        // set chilren's parent
        self.self_weak = Some(self_weak.clone());
        let self_weak: NodeWeak<B> = self_weak.into();
        for child in self.children.clone() {
            child.borrow_mut_with(self).set_parent(Some(self_weak.clone()));
        }
        {
            // initialize shadow root
            let shadow_root_content = <C as ComponentTemplate>::template(self, false);
            let shadow_root = self.shadow_root.clone();
            let mut shadow_root = unsafe { shadow_root.borrow_mut_unsafe_with(self) };
            shadow_root.set_composed_parent(Some(self_weak.clone()));
            if let Some(shadow_root_content) = shadow_root_content {
                shadow_root.replace_children_list(shadow_root_content);
            }
            // append shadow root
            let mut backend_children = vec![];
            shadow_root.to_ref().collect_backend_nodes(&mut backend_children);
            self.backend_element.append_list(backend_children);
        }
        self.check_slots_update();
    }
    pub(crate) fn update_node(&mut self) {
        self.check_slots_update();
    }
}
impl<B: Backend> fmt::Debug for ComponentNode<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Component>")
    }
}

pub struct TextNode<B: Backend> {
    pub(crate) backend: Rc<B>,
    pub(crate) backend_element: <<B as Backend>::BackendNode as BackendNode>::BackendTextNode,
    pub(crate) self_weak: Option<TextNodeWeak<B>>,
    pub(crate) text_content: String,
    pub(crate) owner: Option<ComponentNodeWeak<B>>,
    pub(crate) parent: Option<NodeWeak<B>>,
    pub(crate) composed_parent: Option<NodeWeak<B>>,
}
impl<B: Backend> TextNode<B> {
    define_tree_getter!(text);
    fn collect_backend_nodes<'b>(&'b self, v: &'b mut Vec<<B as Backend>::BackendNode>) {
        v.push(self.backend_element.ref_clone().into_node());
    }
    pub fn new_with_content(backend: Rc<B>, text_content: String, owner: Option<ComponentNodeWeak<B>>) -> Self {
        let backend_element = backend.create_text_node(text_content.as_ref());
        TextNode { backend, backend_element, self_weak: None, text_content, owner, parent: None, composed_parent: None }
    }
    pub fn text_content(&self) -> &str {
        &self.text_content
    }
    pub fn set_text_content<T: Into<String>>(&mut self, c: T) {
        self.text_content = c.into();
    }
}
impl<'a, B: Backend> TextNodeRef<'a, B> {
    define_tree_getter!(ref);
    pub fn backend_element(&self) -> &<<B as Backend>::BackendNode as BackendNode>::BackendTextNode {
        &self.backend_element
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
    pub fn backend_element(&self) -> &<<B as Backend>::BackendNode as BackendNode>::BackendTextNode {
        &self.backend_element
    }
    fn initialize(&mut self, self_weak: TextNodeWeak<B>) {
        self.self_weak = Some(self_weak);
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
    pub unsafe fn borrow_mut_unsafe_with<'a: 'b, 'b, U>(&'b self, source: &'b mut U) -> NodeRefMut<'b, B> where U: ElementRefMut<'a, B> {
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
    fn collect_backend_nodes<'b>(&'b self, v: &'b mut Vec<<B as Backend>::BackendNode>) {
        match self {
            Self::NativeNode(x) => x.collect_backend_nodes(v),
            Self::VirtualNode(x) => x.collect_backend_nodes(v),
            Self::ComponentNode(x) => x.collect_backend_nodes(v),
            Self::TextNode(x) => x.collect_backend_nodes(v),
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
    pub fn find_next_backend_sibling(&self, include_self: bool) -> Option<<B as Backend>::BackendNode> {
        match self {
            Self::NativeNode(x) => x.find_next_backend_sibling(include_self),
            Self::VirtualNode(x) => x.find_next_backend_sibling(include_self),
            Self::ComponentNode(x) => x.find_next_backend_sibling(include_self),
            Self::TextNode(x) => x.find_next_backend_sibling(include_self),
        }
    }
    fn find_next_backend_child(&self, from_index: usize) -> Option<<B as Backend>::BackendNode> {
        let children = self.composed_children().into_iter().skip(from_index);
        for child in children {
            match child {
                NodeRc::NativeNode(x) => {
                    return Some(x.borrow_with(self).backend_element.ref_clone().into_node())
                },
                NodeRc::VirtualNode(_) => { },
                NodeRc::ComponentNode(x) => {
                    return Some(x.borrow_with(self).backend_element.ref_clone().into_node())
                },
                NodeRc::TextNode(x) => {
                    return Some(x.borrow_with(self).backend_element.ref_clone().into_node())
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
    pub fn to_ref<'b>(&'b self) -> NodeRef<'b, B> where 'a: 'b {
        match self {
            Self::NativeNode(x) => NodeRef::NativeNode(x.to_ref()),
            Self::VirtualNode(x) => NodeRef::VirtualNode(x.to_ref()),
            Self::ComponentNode(x) => NodeRef::ComponentNode(x.to_ref()),
            Self::TextNode(x) => NodeRef::TextNode(x.to_ref()),
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
impl<'a, B: Backend> fmt::Debug for NodeRefMut<'a, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug_fmt(f, 0)
    }
}

pub type TemplateNodeFn<B, T> = Box<dyn Fn(&T) -> NodeRc<B>>;

pub trait ElementRef<'a, B: Backend> {
    fn backend(&self) -> &Rc<B>;
    fn as_me_ref_handle(&self) -> &MeRefHandle<'a>;
    fn as_node_ref<'b>(self) -> NodeRef<'b, B> where 'a: 'b;
}
pub trait ElementRefMut<'a, B: Backend> {
    fn backend(&self) -> &Rc<B>;
    fn as_me_ref_mut_handle<'b>(&'b mut self) -> &'b mut MeRefMutHandle<'a> where 'a: 'b;
    fn as_node_ref_mut<'b>(self) -> NodeRefMut<'b, B> where 'a: 'b;
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
    fn as_node_ref<'b>(self) -> NodeRef<'b, B> where 'a: 'b {
        match self {
            Self::NativeNode(x) => x.as_node_ref(),
            Self::VirtualNode(x) => x.as_node_ref(),
            Self::ComponentNode(x) => x.as_node_ref(),
            Self::TextNode(x) => x.as_node_ref(),
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
    fn as_node_ref_mut<'b>(self) -> NodeRefMut<'b, B> where 'a: 'b {
        match self {
            Self::NativeNode(x) => x.as_node_ref_mut(),
            Self::VirtualNode(x) => x.as_node_ref_mut(),
            Self::ComponentNode(x) => x.as_node_ref_mut(),
            Self::TextNode(x) => x.as_node_ref_mut(),
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
                $rm { c: self.c.borrow_mut() }
            }
            pub fn borrow_with<'a: 'b, 'b, U>(&'b self, source: &'b U) -> $r<'b, B> where U: ElementRef<'a, B> {
                $r { c: self.c.borrow_with_handle(source.as_me_ref_handle()) }
            }
            pub fn borrow_mut_with<'a: 'b, 'b, U>(&'b self, source: &'b mut U) -> $rm<'b, B> where U: ElementRefMut<'a, B> {
                $rm { c: self.c.borrow_mut_with_handle(source.as_me_ref_mut_handle()) }
            }
            pub unsafe fn borrow_mut_unsafe_with<'a: 'b, 'b, 'c, U>(&'c self, source: &'b mut U) -> $rm<'c, B> where U: ElementRefMut<'a, B> {
                $rm { c: self.c.borrow_mut_unsafe_with_handle(source.as_me_ref_mut_handle()) }
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
        impl<'a, B: Backend> $r<'a, B> { }
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
            fn as_node_ref<'b>(self) -> NodeRef<'b, B> where 'a: 'b {
                NodeRef::$t($r { c: self.c.map(|x| x) })
            }
        }
        impl<'a, B: Backend> From<$r<'a, B>> for NodeRef<'a, B> {
            fn from(s: $r<'a, B>) -> Self {
                NodeRef::$t(s)
            }
        }

        pub struct $rm<'a, B: Backend> {
            c: MeRefMut<'a, $t<B>>
        }
        impl<'a, B: Backend> $rm<'a, B> {
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
            fn as_node_ref_mut<'b>(self) -> NodeRefMut<'b, B> where 'a: 'b {
                NodeRefMut::$t($rm { c: self.c.map(|x| x) })
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
