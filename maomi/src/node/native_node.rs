use std::rc::{Rc, Weak};
use std::cell::{Ref};
use std::ops::{Deref, DerefMut};
use std::fmt;
use std::borrow::Cow;
use std::mem::ManuallyDrop;

use super::*;
use crate::backend::*;
use crate::context::Scheduler;
use crate::escape;
use crate::global_events::GlobalEvents;

/// A native node which has a single corresponding backend node
pub struct NativeNode<B: Backend> {
    pub(crate) backend: Rc<B>,
    pub(crate) scheduler: Rc<Scheduler>,
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

    /// Get the backend element
    pub fn backend_element(&self) -> &B::BackendElement {
        &self.backend_element
    }

    /// Get the children nodes list in composed tree
    pub fn composed_children_rc(&self) -> Cow<'_, Vec<NodeRc<B>>> {
        Cow::Borrowed(&self.children)
    }

    pub(super) fn collect_backend_nodes<'b>(&'b self, v: &'b mut Vec<NodeRc<B>>) {
        v.push(self.rc().into())
    }

    pub(crate) fn new_with_children(backend: Rc<B>, scheduler: Rc<Scheduler>, tag_name: &'static str, attributes: Vec<(&'static str, String)>, children: Vec<NodeRc<B>>, owner: Option<ComponentNodeWeak<B>>) -> Self {
        let backend_element = backend.create_element(tag_name);
        NativeNode {
            backend,
            scheduler,
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

    pub(super) unsafe fn initialize(&mut self, self_weak: NativeNodeWeak<B>) {
        // bind backend element
        self.backend_element.bind_node_weak(self_weak.clone().into());
        // set chilren's parent
        self.self_weak = Some(self_weak.clone());
        let self_weak: NodeWeak<B> = self_weak.into();
        for child in self.children.clone() {
            let mut child = child.deref_mut_unsafe();
            child.set_parent(Some(self_weak.clone()));
            child.set_composed_parent(Some(self_weak.clone()));
        }
        // insert in backend
        let mut backend_children = vec![];
        for child in self.children.iter() {
            child.deref_unsafe().collect_backend_nodes(&mut backend_children);
        }
        let backend_children: Vec<_> = backend_children.iter().map(|x| x.deref_unsafe()).collect();
        let backend_children: Vec<_> = backend_children.iter().map(|x| x.backend_node().unwrap()).collect();
        self.backend_element.append_list(backend_children);
    }

    /// Get the tag name
    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }

    /// Get an attribute value
    pub fn get_attribute(&self, name: &'static str) -> Option<&str> {
        self.attributes.iter().find(|x| x.0 == name).map(|x| x.1.as_str())
    }

    /// Set an attribute.
    /// **Should be done through template engine!**
    #[doc(hidden)]
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

    /// Get binded events list.
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub fn global_events(&self) -> &GlobalEvents<B> {
        &self.global_events
    }

    /// Get binded events list mutably.
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub fn global_events_mut(&mut self) -> &mut GlobalEvents<B> {
        &mut self.global_events
    }

    /// Convert to HTML
    pub fn to_html<T: std::io::Write>(&self, s: &mut T) -> std::io::Result<()> {
        write!(s, "<{}", self.tag_name)?;
        for (name, value) in self.attributes.iter() {
            write!(s, r#" {}="{}""#, name, escape::escape_html(value))?;
        }
        write!(s, ">")?;
        for child in self.children.iter() {
            // it is safe because a ref for this node is provided
            unsafe { child.deref_unsafe() }.to_html(s)?;
        }
        write!(s, "</{}>", self.tag_name)
    }

    pub(super) fn debug_fmt(&self, f: &mut fmt::Formatter<'_>, level: u32) -> fmt::Result {
        for _ in 0..level {
            write!(f, "  ")?;
        }
        write!(f, "<{}", self.tag_name)?;
        for (name, value) in self.attributes.iter() {
            write!(f, r#" {}="{}""#, name, value)?;
        }
        write!(f, ">")?;
        for child in self.children() {
            child.debug_fmt(f, level + 1)?;
        }
        Ok(())
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

impl<'a, B: Backend> NativeNodeRef<'a, B> {
    define_tree_getter!(ref);
}

impl<'a, B: Backend> NativeNodeRefMut<'a, B> {
    define_tree_getter!(ref mut);
}

impl<B: Backend> fmt::Debug for NativeNode<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug_fmt(f, 0)
    }
}

some_node_def!(NativeNode, NativeNodeRc, NativeNodeWeak, NativeNodeRef, NativeNodeRefMut);
