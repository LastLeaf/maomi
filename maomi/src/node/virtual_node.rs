use std::rc::{Rc, Weak};
use std::cell::{Ref};
use std::ops::{Deref, DerefMut};
use std::fmt;
use std::any::Any;
use std::mem::ManuallyDrop;
use std::borrow::Cow;

use super::*;
use crate::backend::*;
use crate::context::Scheduler;

/// The property it contains.
/// `property` decides the basic functionality of a virtual node.
/// **Should be done through template engine!**
#[doc(hidden)]
pub enum VirtualNodeProperty<B: Backend> {
    /// A pure node without any functionality
    None,
    /// A root of a shadow tree
    ShadowRoot,
    /// A slot node
    Slot(&'static str, Vec<NodeRc<B>>),
    /// A node inserted to a slot
    InSlot(&'static str),
    /// A node which contains several conditional branches
    Branch(usize),
    /// A node which contains a repeatable node list
    List(Box<dyn Any>),
}

/// A virtual node which has no corresponding backend node
pub struct VirtualNode<B: Backend> {
    pub(crate) backend: Rc<B>,
    pub(crate) scheduler: Rc<Scheduler>,
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

    /// Get the children nodes list in composed tree
    pub fn composed_children_rc(&self) -> Cow<'_, Vec<NodeRc<B>>> {
        match &self.property {
            VirtualNodeProperty::Slot(_, children) => Cow::Borrowed(children),
            _ => Cow::Borrowed(&self.children),
        }
    }

    pub(super) fn collect_backend_nodes<'b>(&'b self, v: &'b mut Vec<NodeRc<B>>) {
        for child in self.composed_children() {
            child.collect_backend_nodes(v);
        }
    }

    pub(crate) fn new_empty(backend: Rc<B>, scheduler: Rc<Scheduler>) -> Self {
        Self::new_with_children(backend, scheduler, "", VirtualNodeProperty::None, vec![], None)
    }

    pub(super) fn new_with_children(backend: Rc<B>, scheduler: Rc<Scheduler>, tag_name: &'static str, property: VirtualNodeProperty<B>, children: Vec<NodeRc<B>>, owner: Option<ComponentNodeWeak<B>>) -> Self {
        if let VirtualNodeProperty::Slot(_, c) = &property {
            if children.len() > 0 || c.len() > 0 {
                panic!("Slot cannot contain any child")
            }
        }
        VirtualNode {
            backend,
            scheduler,
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

    pub(super) unsafe fn initialize(&mut self, self_weak: VirtualNodeWeak<B>) {
        // set chilren's parent
        self.self_weak = Some(self_weak.clone());
        let self_weak: NodeWeak<B> = self_weak.into();
        for child in self.children.clone() {
            let mut child = child.deref_mut_unsafe();
            child.set_parent(Some(self_weak.clone()));
            child.set_composed_parent(Some(self_weak.clone()));
        }
    }

    /// Get the tag name
    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }

    /// Get the property
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub fn property(&self) -> &VirtualNodeProperty<B> {
        &self.property
    }

    /// Get the property
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub fn property_mut(&mut self) -> &mut VirtualNodeProperty<B> {
        &mut self.property
    }

    /// Set the property
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub fn set_property(&mut self, property: VirtualNodeProperty<B>) {
        self.property = property;
    }

    pub(super) fn find_backend_parent<'b>(&'b self) -> Option<NodeRc<B>> {
        match self.composed_parent_rc() {
            None => None,
            Some(composed_parent) => match composed_parent {
                NodeRc::NativeNode(x) => Some(x.clone().into()),
                NodeRc::VirtualNode(x) => unsafe { x.deref_unsafe() }.find_backend_parent(),
                NodeRc::ComponentNode(x) => Some(x.clone().into()),
                _ => unreachable!()
            }
        }
    }

    /// Convert to HTML
    pub fn to_html<T: std::io::Write>(&self, s: &mut T) -> std::io::Result<()> {
        for child in self.composed_children() {
            child.to_html(s)?;
        }
        Ok(())
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

    /// Set the content of a shadow root.
    /// It is safe only when updating the shadow tree.
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub unsafe fn set_shadow_root_content(&mut self, mut list: Vec<NodeRc<B>>) {
        self.owner().for_each(|x| x.marks_cache_dirty.set(true));
        if let VirtualNodeProperty::ShadowRoot = self.property {
            if self.children.len() > 0 {
                panic!("Cannot reset shadow root content")
            }
            // set new children's parent
            let self_weak: NodeWeak<B> = self.self_weak.clone().unwrap().into();
            for child in list.iter() {
                let mut child = child.deref_mut_unsafe();
                child.set_parent(Some(self_weak.clone()));
                child.set_composed_parent(Some(self_weak.clone()));
            }
            std::mem::swap(&mut self.children, &mut list);
            // insert new backend children
            let mut backend_children = vec![];
            for n in self.children() {
                n.collect_backend_nodes(&mut backend_children);
            }
            let before = self.find_next_sibling(false);
            let before = before.as_ref().map(|x| x.deref_unsafe());
            let backend_children: Vec<_> = backend_children.iter().map(|x| x.deref_unsafe()).collect();
            let backend_children: Vec<_> = backend_children.iter().map(|x| {
                x.backend_node().unwrap()
            }).collect();
            self.owner().next().unwrap().backend_element.insert_list_before(backend_children, before.as_ref().map(|x| x.backend_node().unwrap()));
        } else {
            panic!("Cannot set shadow root content on non-shadowRoot node")
        }
    }

    /// Replace children.
    /// It is safe only when updating the shadow tree.
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub unsafe fn replace_children_list(&mut self, mut list: Vec<NodeRc<B>>) {
        self.owner().for_each(|x| x.marks_cache_dirty.set(true));
        // set new children's parent
        let self_weak: NodeWeak<B> = self.self_weak.clone().unwrap().into();
        for child in list.iter() {
            let mut child = child.deref_mut_unsafe();
            child.set_parent(Some(self_weak.clone()));
            child.set_composed_parent(Some(self_weak.clone()));
        }
        std::mem::swap(&mut self.children, &mut list);
        // remove old children's parent
        for child in list.iter_mut() {
            let mut child = child.deref_mut_unsafe();
            child.set_parent(None);
            child.set_composed_parent(None);
        }
        {
            if let Some(p) = self.find_backend_parent() {
                // remove old backend children
                let mut backend_children = vec![];
                for n in list.iter() {
                    unsafe { n.deref_unsafe() }.collect_backend_nodes(&mut backend_children);
                }
                let backend_children: Vec<_> = backend_children.iter().map(|x| x.deref_unsafe()).collect();
                let backend_children: Vec<_> = backend_children.iter().map(|x| {
                    x.backend_node().unwrap()
                }).collect();
                unsafe { p.deref_unsafe() }.backend_element().unwrap().remove_list(backend_children);
                // insert new backend children
                let mut backend_children = vec![];
                for n in self.children.iter() {
                    unsafe { n.deref_unsafe() }.collect_backend_nodes(&mut backend_children);
                }
                let before = self.find_next_sibling(false);
                let before = before.as_ref().map(|x| unsafe { x.deref_unsafe() });
                let backend_children: Vec<_> = backend_children.iter().map(|x| x.deref_unsafe()).collect();
                let backend_children: Vec<_> = backend_children.iter().map(|x| {
                    x.backend_node().unwrap()
                }).collect();
                unsafe { p.deref_unsafe() }.backend_element().unwrap().insert_list_before(backend_children, before.as_ref().map(|x| x.backend_node().unwrap()));
            }
        }
        // call detached and attached
        if self.attached {
            for child in list.iter_mut() {
                let mut child = unsafe { child.deref_mut_unsafe() };
                child.set_detached();
            }
            for child in self.children.clone() {
                let mut child = unsafe { child.deref_mut_unsafe() };
                child.set_attached();
            }
        }
    }

    /// Remove children and reuse some of them.
    /// It is safe only when updating the shadow tree.
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub unsafe fn remove_with_reuse(&mut self, start: usize, reusable: &Box<[bool]>) {
        self.owner().for_each(|x| x.marks_cache_dirty.set(true));
        // set children
        let r = start..(start + reusable.len());
        let removed: Vec<NodeRc<B>> = self.children.splice(r, vec![]).collect();
        // remove old children's parent
        for child in removed.iter() {
            let mut child = unsafe { child.deref_mut_unsafe() };
            child.set_parent(None);
            child.set_composed_parent(None);
        }
        // remove in backend
        {
            if let Some(p) = self.find_backend_parent() {
                let mut backend_children = vec![];
                for (i, n) in removed.iter().enumerate() {
                    if !reusable[i] {
                        unsafe { n.deref_unsafe() }.collect_backend_nodes(&mut backend_children);
                    }
                }
                let backend_children: Vec<_> = backend_children.iter().map(|x| x.deref_unsafe()).collect();
                let backend_children: Vec<_> = backend_children.iter().map(|x| {
                   x.backend_node().unwrap()
                }).collect();
                unsafe { p.deref_unsafe() }.backend_element().unwrap().remove_list(backend_children);
            }
        }
        // call detached if it really needs to be detached
        if self.attached {
            for (i, n) in removed.iter().enumerate() {
                if !reusable[i] {
                    unsafe { n.deref_mut_unsafe() }.set_detached()
                }
            }
        }
    }

    /// Insert children.
    /// It is safe only when updating the shadow tree.
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub unsafe fn insert_list(&mut self, pos: usize, list: Vec<NodeRc<B>>) {
        self.owner().for_each(|x| x.marks_cache_dirty.set(true));
        // set new children's parent
        let self_weak: NodeWeak<B> = self.self_weak.clone().unwrap().into();
        for child in list.iter() {
            let mut child = unsafe { child.deref_mut_unsafe() };
            child.set_parent(Some(self_weak.clone()));
            child.set_composed_parent(Some(self_weak.clone()));
        }
        {
            // insert in backend
            if let Some(b) = self.find_backend_parent() {
                let mut backend_children = vec![];
                for n in list.iter() {
                    unsafe { n.deref_unsafe() }.collect_backend_nodes(&mut backend_children);
                }
                let before = match self.children.get(pos) {
                    Some(x) => {
                        unsafe { x.deref_unsafe() }.find_next_sibling(true)
                    },
                    None => None,
                };
                let before = before.as_ref().map(|x| unsafe { x.deref_unsafe() });
                let backend_children: Vec<_> = backend_children.iter().map(|x| x.deref_unsafe()).collect();
                let backend_children: Vec<_> = backend_children.iter().map(|x| {
                    x.backend_node().unwrap()
                }).collect();
                unsafe { b.deref_unsafe() }.backend_element().unwrap().insert_list_before(backend_children, before.as_ref().map(|x| x.backend_node().unwrap()));
            }
        }
        // set children
        let _: Vec<_> = self.children.splice(pos..pos, list.clone()).collect();
        // call attached
        if self.attached {
            for child in list {
                let mut child = unsafe { child.deref_mut_unsafe() };
                child.set_attached();
            }
        }
    }

    pub(crate) fn set_attached(&mut self) {
        let mut children = self.composed_children_mut();
        while let Some(mut child) = children.next() {
            child.set_attached();
        }
    }

    pub(crate) fn set_detached(&mut self) {
        let mut children = self.composed_children_mut();
        while let Some(mut child) = children.next() {
            child.set_detached();
        }
    }
}

impl<'a, B: Backend> VirtualNodeRef<'a, B> {
    define_tree_getter!(ref);
}

impl<'a, B: Backend> VirtualNodeRefMut<'a, B> {
    define_tree_getter!(ref mut);
}

impl<B: Backend> fmt::Debug for VirtualNode<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug_fmt(f, 0)
    }
}

some_node_def!(VirtualNode, VirtualNodeRc, VirtualNodeWeak, VirtualNodeRef, VirtualNodeRefMut);
