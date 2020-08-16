use std::rc::{Rc, Weak};
use std::cell::{Ref};
use std::ops::{Deref, DerefMut};
use std::fmt;
use std::mem::ManuallyDrop;

use super::*;
use crate::backend::*;
use crate::context::Scheduler;
use crate::escape;

/// A text node
pub struct TextNode<B: Backend> {
    pub(crate) backend: Rc<B>,
    pub(crate) scheduler: Rc<Scheduler>,
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

    pub(super) fn collect_backend_nodes<'b>(&'b self, v: &'b mut Vec<NodeRc<B>>) {
        v.push(self.rc().into());
    }
    
    pub(crate) fn new_with_content(backend: Rc<B>, scheduler: Rc<Scheduler>, text_content: String, owner: Option<ComponentNodeWeak<B>>) -> Self {
        let backend_element = backend.create_text_node(text_content.as_ref());
        TextNode { backend, scheduler, backend_element, attached: false, self_weak: None, text_content, owner, parent: None, composed_parent: None }
    }

    pub(super) fn initialize(&mut self, self_weak: TextNodeWeak<B>) {
        self.self_weak = Some(self_weak);
    }

    /// Get the text content
    pub fn text_content(&self) -> &str {
        &self.text_content
    }

    /// Set the text content.
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub fn set_text_content<T: ToString>(&mut self, c: T) {
        self.text_content = c.to_string();
        self.backend_element.set_text_content(&self.text_content);
    }

    /// Get the backend text node
    pub fn backend_element(&self) -> &B::BackendTextNode {
        &self.backend_element
    }

    /// Convert to HTML
    pub fn to_html<T: std::io::Write>(&self, s: &mut T) -> std::io::Result<()> {
        let html = escape::escape_html(&self.text_content);
        if html == "" {
            write!(s, "<!---->")
        } else {
            write!(s, "{}", html)
        }
    }

    pub(super) fn debug_fmt(&self, f: &mut fmt::Formatter<'_>, level: u32) -> fmt::Result {
        for _ in 0..level {
            write!(f, "  ")?;
        }
        writeln!(f, "{:?}", self.text_content)?;
        Ok(())
    }
}

impl<'a, B: Backend> TextNodeRefMut<'a, B> {
    define_tree_getter!(ref mut);
}

impl<B: Backend> fmt::Debug for TextNode<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug_fmt(f, 0)
    }
}

some_node_def!(TextNode, TextNodeRc, TextNodeWeak, TextNodeRef, TextNodeRefMut);
