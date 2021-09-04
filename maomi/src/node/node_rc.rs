use std::borrow::Cow;
use std::fmt;
use std::rc::Rc;

use super::*;
use crate::backend::*;
use crate::context::Scheduler;

pub(crate) fn create_component<'a, B: Backend, C: 'static + Component<B>>(
    n: &mut NodeRefMut<'a, B>,
    scheduler: Rc<Scheduler>,
    tag_name: &'static str,
    children: Vec<NodeRc<B>>,
    owner: Option<ComponentNodeWeak<B>>,
) -> ComponentNodeRc<B> {
    unsafe {
        ComponentNode::new_free_component_node::<C>(
            n.as_mut(),
            scheduler,
            tag_name,
            children,
            owner,
        )
    }
}

macro_rules! define_tree_getter {
    (text) => {
        pub(super) fn backend(&self) -> &Rc<B> {
            &self.backend
        }

        /// Get the owner component node
        pub fn owner_rc(&self) -> Option<ComponentNodeRc<B>> {
            match self.owner.as_ref() {
                Some(x) => x.upgrade(),
                None => None,
            }
        }

        /// Get the owner component node
        /// Returns an iterator containing the owner node itself.
        pub fn owner<'c>(&'c self) -> SingleNodeIter<'c, B, &'c ComponentNode<B>> {
            SingleNodeIter::<B, &'c ComponentNode<B>>::owner(self.into())
        }

        /// Get the owner component node
        /// Returns an iterator containing the owner node itself.
        pub fn owner_mut<'c>(&'c mut self) -> SingleNodeIterMut<'c, B, &'c mut ComponentNode<B>> {
            SingleNodeIterMut::<B, &'c ComponentNode<B>>::owner(self.into())
        }

        /// Get the parent `NodeRc` in shadow tree
        pub fn parent_rc(&self) -> Option<NodeRc<B>> {
            match self.parent.as_ref() {
                Some(x) => x.upgrade(),
                None => None,
            }
        }

        /// Get the parent node in shadow tree.
        /// Returns an iterator containing the parent node itself.
        pub fn parent<'c>(&'c self) -> SingleNodeIter<'c, B, Node<'c, B>> {
            SingleNodeIter::<B, Node<B>>::parent(self.into(), TraversalRange::Shadow)
        }

        /// Get the parent node in shadow tree.
        /// Returns an iterator containing the parent node itself.
        pub fn parent_mut<'c>(&'c mut self) -> SingleNodeIterMut<'c, B, NodeMut<'c, B>> {
            SingleNodeIterMut::<B, Node<B>>::parent(self.into(), TraversalRange::Shadow)
        }

        pub(super) fn set_parent(&mut self, p: Option<NodeWeak<B>>) {
            self.parent = p;
        }

        /// Get the parent `NodeRc` in composed tree
        pub fn composed_parent_rc(&self) -> Option<NodeRc<B>> {
            match self.composed_parent.as_ref() {
                Some(x) => x.upgrade(),
                None => None,
            }
        }

        /// Get the parent node in composed tree.
        /// Returns an iterator containing the parent node itself.
        pub fn composed_parent<'c>(&'c self) -> SingleNodeIter<'c, B, Node<'c, B>> {
            SingleNodeIter::<B, Node<B>>::parent(self.into(), TraversalRange::Composed)
        }

        /// Get the parent node in composed tree.
        /// Returns an iterator containing the parent node itself.
        pub fn composed_parent_mut<'c>(&'c mut self) -> SingleNodeIterMut<'c, B, NodeMut<'c, B>> {
            SingleNodeIterMut::<B, Node<B>>::parent(self.into(), TraversalRange::Composed)
        }

        pub(super) fn set_composed_parent(&mut self, p: Option<NodeWeak<B>>) {
            self.composed_parent = p;
        }

        /// Iterate ancestors in shadow tree
        pub fn ancestors<'c>(&'c self, order: TraversalOrder) -> AncestorIter<'c, B> {
            AncestorIter::new(self.into(), TraversalRange::Shadow, order)
        }

        /// Iterate ancestors in shadow tree
        pub fn ancestors_mut<'c>(&'c mut self, order: TraversalOrder) -> AncestorIterMut<'c, B> {
            AncestorIterMut::new(self.into(), TraversalRange::Shadow, order)
        }

        /// Iterate ancestors in composed tree
        pub fn composed_ancestors<'c>(&'c self, order: TraversalOrder) -> AncestorIter<'c, B> {
            AncestorIter::new(self.into(), TraversalRange::Composed, order)
        }

        /// Iterate ancestors in composed tree
        pub fn composed_ancestors_mut<'c>(
            &'c mut self,
            order: TraversalOrder,
        ) -> AncestorIterMut<'c, B> {
            AncestorIterMut::new(self.into(), TraversalRange::Composed, order)
        }

        /// Iterate descendants
        pub fn dfs<'c>(&'c self, range: TraversalRange, order: TraversalOrder) -> DfsIter<'c, B> {
            DfsIter::new(self.into(), range, order)
        }

        /// Iterate descendants
        pub fn dfs_mut(&mut self, range: TraversalRange, order: TraversalOrder) -> DfsIterMut<B> {
            DfsIterMut::new(self.into(), range, order)
        }

        /// Get the attach status. A node is attached if its top most ancestor is the root component node.
        pub fn is_attached(&self) -> bool {
            self.attached
        }
    };

    (node) => {
        define_tree_getter!(text);

        /// Get the children node list in shadow tree
        pub fn children_rc(&self) -> &Vec<NodeRc<B>> {
            &self.children
        }

        /// Get a child node
        pub fn child<'c>(&'c self, index: usize) -> Option<Node<'c, B>> {
            self.children
                .get(index)
                .map(|x| unsafe { x.deref_unsafe() })
        }

        /// Get a child node
        pub fn child_mut<'c>(&'c mut self, index: usize) -> Option<NodeMut<'c, B>> {
            self.children
                .get(index)
                .map(|x| unsafe { x.deref_mut_unsafe() })
        }

        /// Iterate child nodes
        pub fn children(&self) -> ChildIter<B> {
            ChildIter::new(self.into(), TraversalRange::Shadow)
        }

        /// Iterate child nodes
        pub fn children_mut(&mut self) -> ChildIterMut<B> {
            ChildIterMut::new(self.into(), TraversalRange::Shadow)
        }

        /// Iterate child nodes
        pub fn composed_children(&self) -> ChildIter<B> {
            ChildIter::new(self.into(), TraversalRange::Composed)
        }

        /// Iterate child nodes
        pub fn composed_children_mut(&mut self) -> ChildIterMut<B> {
            ChildIterMut::new(self.into(), TraversalRange::Composed)
        }

        pub(super) unsafe fn find_next_backend_child(
            &self,
            from_index: usize,
        ) -> Option<NodeRc<B>> {
            let children = self.composed_children().skip(from_index);
            for child in children {
                match child {
                    Node::NativeNode(_) => return Some(child.rc()),
                    Node::VirtualNode(_) => {}
                    Node::ComponentNode(_) => return Some(child.rc()),
                    Node::TextNode(_) => return Some(child.rc()),
                }
                match child.find_next_backend_child(0) {
                    None => {}
                    Some(x) => return Some(x),
                }
            }
            None
        }

        pub(super) unsafe fn find_next_sibling<'b>(
            &'b self,
            include_self: bool,
        ) -> Option<NodeRc<B>> {
            match self.composed_parent().next() {
                None => None,
                Some(composed_parent) => {
                    let next_child = {
                        let index = composed_parent
                            .composed_children_rc()
                            .iter()
                            .position(|x| *x == NodeRc::from(self.rc()))
                            .unwrap();
                        composed_parent
                            .find_next_backend_child(index + if include_self { 0 } else { 1 })
                    };
                    match next_child {
                        Some(x) => Some(x),
                        None => match composed_parent {
                            Node::NativeNode(_) => None,
                            Node::VirtualNode(x) => x.find_next_sibling(false),
                            Node::ComponentNode(_) => None,
                            _ => unreachable!(),
                        },
                    }
                }
            }
        }
    };

    (ref) => {};

    (ref mut) => {};
}

/// A ref-counted node
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
            NodeRc::NativeNode(x) => {
                if let NodeRc::NativeNode(y) = other {
                    x == y
                } else {
                    false
                }
            }
            NodeRc::VirtualNode(x) => {
                if let NodeRc::VirtualNode(y) = other {
                    x == y
                } else {
                    false
                }
            }
            NodeRc::ComponentNode(x) => {
                if let NodeRc::ComponentNode(y) = other {
                    x == y
                } else {
                    false
                }
            }
            NodeRc::TextNode(x) => {
                if let NodeRc::TextNode(y) = other {
                    x == y
                } else {
                    false
                }
            }
        }
    }
}

impl<B: Backend> NodeRc<B> {
    pub(super) fn another_me_cell(&self) -> MeCell<()> {
        match self {
            NodeRc::NativeNode(x) => x.another_me_cell(),
            NodeRc::VirtualNode(x) => x.another_me_cell(),
            NodeRc::ComponentNode(x) => x.another_me_cell(),
            NodeRc::TextNode(x) => x.another_me_cell(),
        }
    }

    /// Borrow the node.
    /// Panics if any node has been mutably borrowed.
    pub fn borrow<'a>(&'a self) -> NodeRef<'a, B> {
        match self {
            NodeRc::NativeNode(x) => NodeRef::NativeNode(x.borrow()),
            NodeRc::VirtualNode(x) => NodeRef::VirtualNode(x.borrow()),
            NodeRc::ComponentNode(x) => NodeRef::ComponentNode(x.borrow()),
            NodeRc::TextNode(x) => NodeRef::TextNode(x.borrow()),
        }
    }

    /// Borrow the node.
    /// Return `Err` if any node has been mutably borrowed.
    pub fn try_borrow<'a>(&'a self) -> Result<NodeRef<'a, B>, NodeBorrowError> {
        match self {
            NodeRc::NativeNode(x) => x.try_borrow().map(|x| NodeRef::NativeNode(x)),
            NodeRc::VirtualNode(x) => x.try_borrow().map(|x| NodeRef::VirtualNode(x)),
            NodeRc::ComponentNode(x) => x.try_borrow().map(|x| NodeRef::ComponentNode(x)),
            NodeRc::TextNode(x) => x.try_borrow().map(|x| NodeRef::TextNode(x)),
        }
    }

    /// Borrow the node mutably.
    /// Panics if any node has been borrowed.
    pub fn borrow_mut<'a>(&'a self) -> NodeRefMut<'a, B> {
        match self {
            NodeRc::NativeNode(x) => NodeRefMut::NativeNode(x.borrow_mut()),
            NodeRc::VirtualNode(x) => NodeRefMut::VirtualNode(x.borrow_mut()),
            NodeRc::ComponentNode(x) => NodeRefMut::ComponentNode(x.borrow_mut()),
            NodeRc::TextNode(x) => NodeRefMut::TextNode(x.borrow_mut()),
        }
    }

    /// Borrow the node mutably.
    /// Return `Err` if any node has been borrowed.
    pub fn try_borrow_mut<'a>(&'a self) -> Result<NodeRefMut<'a, B>, NodeBorrowError> {
        match self {
            NodeRc::NativeNode(x) => x.try_borrow_mut().map(|x| NodeRefMut::NativeNode(x)),
            NodeRc::VirtualNode(x) => x.try_borrow_mut().map(|x| NodeRefMut::VirtualNode(x)),
            NodeRc::ComponentNode(x) => x.try_borrow_mut().map(|x| NodeRefMut::ComponentNode(x)),
            NodeRc::TextNode(x) => x.try_borrow_mut().map(|x| NodeRefMut::TextNode(x)),
        }
    }

    /// Get a `NodeWeak` of the node
    pub fn downgrade(&self) -> NodeWeak<B> {
        match self {
            NodeRc::NativeNode(x) => NodeWeak::NativeNode(x.downgrade()),
            NodeRc::VirtualNode(x) => NodeWeak::VirtualNode(x.downgrade()),
            NodeRc::ComponentNode(x) => NodeWeak::ComponentNode(x.downgrade()),
            NodeRc::TextNode(x) => NodeWeak::TextNode(x.downgrade()),
        }
    }

    /// Deref the node unsafely
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub unsafe fn deref_unsafe<'b>(&'b self) -> Node<'b, B> {
        match self {
            NodeRc::NativeNode(x) => Node::NativeNode(x.deref_unsafe()),
            NodeRc::VirtualNode(x) => Node::VirtualNode(x.deref_unsafe()),
            NodeRc::ComponentNode(x) => Node::ComponentNode(x.deref_unsafe()),
            NodeRc::TextNode(x) => Node::TextNode(x.deref_unsafe()),
        }
    }

    /// Deref the node unsafely
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub unsafe fn deref_mut_unsafe<'b>(&'b self) -> NodeMut<'b, B> {
        match self {
            NodeRc::NativeNode(x) => NodeMut::NativeNode(x.deref_mut_unsafe()),
            NodeRc::VirtualNode(x) => NodeMut::VirtualNode(x.deref_mut_unsafe()),
            NodeRc::ComponentNode(x) => NodeMut::ComponentNode(x.deref_mut_unsafe()),
            NodeRc::TextNode(x) => NodeMut::TextNode(x.deref_mut_unsafe()),
        }
    }

    pub(crate) unsafe fn deref_unsafe_with_lifetime<'c, 'b>(&'c self) -> Node<'b, B> {
        match self {
            NodeRc::NativeNode(x) => Node::NativeNode(x.deref_unsafe_with_lifetime()),
            NodeRc::VirtualNode(x) => Node::VirtualNode(x.deref_unsafe_with_lifetime()),
            NodeRc::ComponentNode(x) => Node::ComponentNode(x.deref_unsafe_with_lifetime()),
            NodeRc::TextNode(x) => Node::TextNode(x.deref_unsafe_with_lifetime()),
        }
    }

    pub(crate) unsafe fn deref_mut_unsafe_with_lifetime<'c, 'b>(&'c self) -> NodeMut<'b, B> {
        match self {
            NodeRc::NativeNode(x) => NodeMut::NativeNode(x.deref_mut_unsafe_with_lifetime()),
            NodeRc::VirtualNode(x) => NodeMut::VirtualNode(x.deref_mut_unsafe_with_lifetime()),
            NodeRc::ComponentNode(x) => NodeMut::ComponentNode(x.deref_mut_unsafe_with_lifetime()),
            NodeRc::TextNode(x) => NodeMut::TextNode(x.deref_mut_unsafe_with_lifetime()),
        }
    }
}

/// A weak ref of a ref-counted node
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
    /// Get a `NodeRc` of the node
    pub fn upgrade(&self) -> Option<NodeRc<B>> {
        match self {
            NodeWeak::NativeNode(x) => x.upgrade().map(|x| NodeRc::NativeNode(x)),
            NodeWeak::VirtualNode(x) => x.upgrade().map(|x| NodeRc::VirtualNode(x)),
            NodeWeak::ComponentNode(x) => x.upgrade().map(|x| NodeRc::ComponentNode(x)),
            NodeWeak::TextNode(x) => x.upgrade().map(|x| NodeRc::TextNode(x)),
        }
    }
}

/// A borrowed ref of a ref-counted node.
/// No other node can be mutably borrowed until this object is dropped.
pub enum NodeRef<'a, B: Backend> {
    NativeNode(NativeNodeRef<'a, B>),
    VirtualNode(VirtualNodeRef<'a, B>),
    ComponentNode(ComponentNodeRef<'a, B>),
    TextNode(TextNodeRef<'a, B>),
}

impl<'a, B: Backend> NodeRef<'a, B> {
    /// Get another borrowed ref of the same node
    pub fn clone<'b>(orig: &'b NodeRef<'a, B>) -> NodeRef<'b, B> {
        match orig {
            Self::NativeNode(x) => Self::NativeNode(NativeNodeRef::clone(&x)),
            Self::VirtualNode(x) => Self::VirtualNode(VirtualNodeRef::clone(&x)),
            Self::ComponentNode(x) => Self::ComponentNode(ComponentNodeRef::clone(&x)),
            Self::TextNode(x) => Self::TextNode(TextNodeRef::clone(&x)),
        }
    }

    /// Get the backend node
    pub fn backend_node<'b>(&'b self) -> Option<BackendNodeRef<'b, B>> {
        match self {
            Self::NativeNode(x) => Some(BackendNodeRef::Element(&x.backend_element)),
            Self::VirtualNode(_) => None,
            Self::ComponentNode(x) => Some(BackendNodeRef::Element(&x.backend_element)),
            Self::TextNode(x) => Some(BackendNodeRef::TextNode(&x.backend_element)),
        }
    }

    /// Get the backend element
    pub fn backend_element<'b>(&'b self) -> Option<&B::BackendElement> {
        match self {
            Self::NativeNode(x) => Some(&x.backend_element),
            Self::VirtualNode(_) => None,
            Self::ComponentNode(x) => Some(&x.backend_element),
            Self::TextNode(_) => None,
        }
    }

    /// Get the `NodeRc` of the node
    pub fn rc(&self) -> NodeRc<B> {
        match self {
            Self::NativeNode(x) => NodeRc::NativeNode(x.rc()),
            Self::VirtualNode(x) => NodeRc::VirtualNode(x.rc()),
            Self::ComponentNode(x) => NodeRc::ComponentNode(x.rc()),
            Self::TextNode(x) => NodeRc::TextNode(x.rc()),
        }
    }

    /// Get the owner component node
    pub fn owner_rc(&self) -> Option<ComponentNodeRc<B>> {
        match self {
            Self::NativeNode(x) => x.owner_rc(),
            Self::VirtualNode(x) => x.owner_rc(),
            Self::ComponentNode(x) => x.owner_rc(),
            Self::TextNode(x) => x.owner_rc(),
        }
    }

    /// Get the owner component node
    pub fn owner<'c>(&'c self) -> SingleNodeIter<'c, B, &'c ComponentNode<B>> {
        match self {
            Self::NativeNode(x) => x.owner(),
            Self::VirtualNode(x) => x.owner(),
            Self::ComponentNode(x) => x.owner(),
            Self::TextNode(x) => x.owner(),
        }
    }

    /// Get the parent `NodeRc` in shadow tree
    pub fn parent_rc(&self) -> Option<NodeRc<B>> {
        match self {
            Self::NativeNode(x) => x.parent_rc(),
            Self::VirtualNode(x) => x.parent_rc(),
            Self::ComponentNode(x) => x.parent_rc(),
            Self::TextNode(x) => x.parent_rc(),
        }
    }

    /// Get the parent node in shadow tree.
    /// Returns an iterator containing the parent node itself.
    pub fn parent<'c>(&'c self) -> SingleNodeIter<'c, B, Node<'c, B>> {
        match self {
            Self::NativeNode(x) => x.parent(),
            Self::VirtualNode(x) => x.parent(),
            Self::ComponentNode(x) => x.parent(),
            Self::TextNode(x) => x.parent(),
        }
    }

    /// Get the parent `NodeRc` in composed tree
    pub fn composed_parent_rc(&self) -> Option<NodeRc<B>> {
        match self {
            Self::NativeNode(x) => x.composed_parent_rc(),
            Self::VirtualNode(x) => x.composed_parent_rc(),
            Self::ComponentNode(x) => x.composed_parent_rc(),
            Self::TextNode(x) => x.composed_parent_rc(),
        }
    }

    /// Get the parent node in composed tree.
    /// Returns an iterator containing the parent node itself.
    pub fn composed_parent<'c>(&'c self) -> SingleNodeIter<'c, B, Node<'c, B>> {
        match self {
            Self::NativeNode(x) => x.composed_parent(),
            Self::VirtualNode(x) => x.composed_parent(),
            Self::ComponentNode(x) => x.composed_parent(),
            Self::TextNode(x) => x.composed_parent(),
        }
    }

    /// Iterate ancestors in shadow tree
    pub fn ancestors<'c>(&'c self, order: TraversalOrder) -> AncestorIter<'c, B> {
        match self {
            Self::NativeNode(x) => x.ancestors(order),
            Self::VirtualNode(x) => x.ancestors(order),
            Self::ComponentNode(x) => x.ancestors(order),
            Self::TextNode(x) => x.ancestors(order),
        }
    }

    /// Iterate ancestors in composed tree
    pub fn composed_ancestors<'c>(&'c self, order: TraversalOrder) -> AncestorIter<'c, B> {
        match self {
            Self::NativeNode(x) => x.composed_ancestors(order),
            Self::VirtualNode(x) => x.composed_ancestors(order),
            Self::ComponentNode(x) => x.composed_ancestors(order),
            Self::TextNode(x) => x.composed_ancestors(order),
        }
    }

    /// Get the children node list in shadow tree
    pub fn children_rc(&self) -> Cow<'_, Vec<NodeRc<B>>> {
        match self {
            Self::NativeNode(x) => Cow::Borrowed(x.children_rc()),
            Self::VirtualNode(x) => Cow::Borrowed(x.children_rc()),
            Self::ComponentNode(x) => Cow::Borrowed(x.children_rc()),
            Self::TextNode(_) => Cow::Owned(vec![]),
        }
    }

    /// Get the children node list in composed tree
    pub fn composed_children_rc(&self) -> Cow<'_, Vec<NodeRc<B>>> {
        match self {
            Self::NativeNode(x) => x.composed_children_rc(),
            Self::VirtualNode(x) => x.composed_children_rc(),
            Self::ComponentNode(x) => x.composed_children_rc(),
            Self::TextNode(_) => Cow::Owned(vec![]),
        }
    }

    /// Get the children node list in composed tree
    pub fn composed_children<'c>(&'c self) -> ChildIter<'c, B> {
        match self {
            Self::NativeNode(x) => x.composed_children(),
            Self::VirtualNode(x) => x.composed_children(),
            Self::ComponentNode(x) => x.composed_children(),
            Self::TextNode(x) => ChildIter::new((&**x).into(), TraversalRange::Composed),
        }
    }

    /// Get the children node list in composed tree
    pub fn dfs<'c>(&'c self, range: TraversalRange, order: TraversalOrder) -> DfsIter<'c, B> {
        match self {
            Self::NativeNode(x) => x.dfs(range, order),
            Self::VirtualNode(x) => x.dfs(range, order),
            Self::ComponentNode(x) => x.dfs(range, order),
            Self::TextNode(x) => x.dfs(range, order),
        }
    }

    pub fn to_html<T: std::io::Write>(&self, s: &mut T) -> std::io::Result<()> {
        match self {
            Self::NativeNode(x) => x.to_html(s),
            Self::VirtualNode(x) => x.to_html(s),
            Self::ComponentNode(x) => x.to_html(s),
            Self::TextNode(x) => x.to_html(s),
        }
    }
}

/// A mutably borrowed ref of a ref-counted node.
/// No other node can be borrowed until this object is dropped.
pub enum NodeRefMut<'a, B: Backend> {
    NativeNode(NativeNodeRefMut<'a, B>),
    VirtualNode(VirtualNodeRefMut<'a, B>),
    ComponentNode(ComponentNodeRefMut<'a, B>),
    TextNode(TextNodeRefMut<'a, B>),
}

impl<'a, B: Backend> NodeRefMut<'a, B> {
    /// Get an immutable reference `Node`
    pub fn as_ref<'b>(&'b self) -> Node<'b, B> {
        match self {
            Self::NativeNode(x) => Node::NativeNode(x),
            Self::VirtualNode(x) => Node::VirtualNode(x),
            Self::ComponentNode(x) => Node::ComponentNode(x),
            Self::TextNode(x) => Node::TextNode(x),
        }
    }

    /// Get another mutable reference `NodeMut`, borrowing out the current one
    pub fn as_mut<'b>(&'b mut self) -> NodeMut<'b, B> {
        match self {
            Self::NativeNode(x) => NodeMut::NativeNode(x),
            Self::VirtualNode(x) => NodeMut::VirtualNode(x),
            Self::ComponentNode(x) => NodeMut::ComponentNode(x),
            Self::TextNode(x) => NodeMut::TextNode(x),
        }
    }

    /// Get the `NodeRc` of the node
    pub fn rc(&self) -> NodeRc<B> {
        match self {
            Self::NativeNode(x) => NodeRc::NativeNode(x.rc()),
            Self::VirtualNode(x) => NodeRc::VirtualNode(x.rc()),
            Self::ComponentNode(x) => NodeRc::ComponentNode(x.rc()),
            Self::TextNode(x) => NodeRc::TextNode(x.rc()),
        }
    }

    /// Get the owner component node
    pub fn owner_rc(&self) -> Option<ComponentNodeRc<B>> {
        match self {
            Self::NativeNode(x) => x.owner_rc(),
            Self::VirtualNode(x) => x.owner_rc(),
            Self::ComponentNode(x) => x.owner_rc(),
            Self::TextNode(x) => x.owner_rc(),
        }
    }

    /// Get the owner component node
    pub fn owner(&self) -> SingleNodeIter<B, &ComponentNode<B>> {
        match self {
            Self::NativeNode(x) => x.owner(),
            Self::VirtualNode(x) => x.owner(),
            Self::ComponentNode(x) => x.owner(),
            Self::TextNode(x) => x.owner(),
        }
    }
}

/// A reference to a node
pub enum Node<'a, B: Backend> {
    NativeNode(&'a NativeNode<B>),
    VirtualNode(&'a VirtualNode<B>),
    ComponentNode(&'a ComponentNode<B>),
    TextNode(&'a TextNode<B>),
}

impl<'a, B: Backend> Node<'a, B> {
    pub(super) fn backend(&self) -> &Rc<B> {
        match self {
            Node::NativeNode(x) => x.backend(),
            Node::VirtualNode(x) => x.backend(),
            Node::ComponentNode(x) => x.backend(),
            Node::TextNode(x) => x.backend(),
        }
    }

    /// Get another borrowed ref of the same node
    pub fn clone<'b>(orig: &'b Node<'a, B>) -> Node<'b, B> {
        match orig {
            Self::NativeNode(x) => Self::NativeNode(x),
            Self::VirtualNode(x) => Self::VirtualNode(x),
            Self::ComponentNode(x) => Self::ComponentNode(x),
            Self::TextNode(x) => Self::TextNode(x),
        }
    }

    /// Get the backend node
    pub fn backend_node<'b>(&'b self) -> Option<BackendNodeRef<'b, B>> {
        match self {
            Self::NativeNode(x) => Some(BackendNodeRef::Element(&x.backend_element)),
            Self::VirtualNode(_) => None,
            Self::ComponentNode(x) => Some(BackendNodeRef::Element(&x.backend_element)),
            Self::TextNode(x) => Some(BackendNodeRef::TextNode(&x.backend_element)),
        }
    }

    /// Get the backend element
    pub fn backend_element<'b>(&'b self) -> Option<&B::BackendElement> {
        match self {
            Self::NativeNode(x) => Some(&x.backend_element),
            Self::VirtualNode(_) => None,
            Self::ComponentNode(x) => Some(&x.backend_element),
            Self::TextNode(_) => None,
        }
    }

    /// Get the `NodeRc` of the node
    pub fn rc(&self) -> NodeRc<B> {
        match self {
            Self::NativeNode(x) => NodeRc::NativeNode(x.rc()),
            Self::VirtualNode(x) => NodeRc::VirtualNode(x.rc()),
            Self::ComponentNode(x) => NodeRc::ComponentNode(x.rc()),
            Self::TextNode(x) => NodeRc::TextNode(x.rc()),
        }
    }

    /// Get the owner component node
    pub fn owner_rc(&self) -> Option<ComponentNodeRc<B>> {
        match self {
            Self::NativeNode(x) => x.owner_rc(),
            Self::VirtualNode(x) => x.owner_rc(),
            Self::ComponentNode(x) => x.owner_rc(),
            Self::TextNode(x) => x.owner_rc(),
        }
    }

    /// Get the owner component node.
    /// Returns an iterator containing the owner node itself.
    pub fn owner<'c>(&'c self) -> SingleNodeIter<'c, B, &'c ComponentNode<B>> {
        match self {
            Self::NativeNode(x) => x.owner(),
            Self::VirtualNode(x) => x.owner(),
            Self::ComponentNode(x) => x.owner(),
            Self::TextNode(x) => x.owner(),
        }
    }

    /// Get the parent `NodeRc` in shadow tree
    pub fn parent_rc(&self) -> Option<NodeRc<B>> {
        match self {
            Self::NativeNode(x) => x.parent_rc(),
            Self::VirtualNode(x) => x.parent_rc(),
            Self::ComponentNode(x) => x.parent_rc(),
            Self::TextNode(x) => x.parent_rc(),
        }
    }

    /// Get the parent node in shadow tree.
    /// Returns an iterator containing the parent node itself.
    pub fn parent<'c>(&'c self) -> SingleNodeIter<'c, B, Node<'c, B>> {
        match self {
            Self::NativeNode(x) => x.parent(),
            Self::VirtualNode(x) => x.parent(),
            Self::ComponentNode(x) => x.parent(),
            Self::TextNode(x) => x.parent(),
        }
    }

    /// Get the parent `NodeRc` in composed tree
    pub fn composed_parent_rc(&self) -> Option<NodeRc<B>> {
        match self {
            Self::NativeNode(x) => x.composed_parent_rc(),
            Self::VirtualNode(x) => x.composed_parent_rc(),
            Self::ComponentNode(x) => x.composed_parent_rc(),
            Self::TextNode(x) => x.composed_parent_rc(),
        }
    }

    /// Get the parent node in composed tree.
    /// Returns an iterator containing the parent node itself.
    pub fn composed_parent<'c>(&'c self) -> SingleNodeIter<'c, B, Node<'c, B>> {
        match self {
            Self::NativeNode(x) => x.composed_parent(),
            Self::VirtualNode(x) => x.composed_parent(),
            Self::ComponentNode(x) => x.composed_parent(),
            Self::TextNode(x) => x.composed_parent(),
        }
    }

    /// Get the children node list in shadow tree
    pub fn children_rc(&self) -> Cow<'_, Vec<NodeRc<B>>> {
        match self {
            Self::NativeNode(x) => Cow::Borrowed(x.children_rc()),
            Self::VirtualNode(x) => Cow::Borrowed(x.children_rc()),
            Self::ComponentNode(x) => Cow::Borrowed(x.children_rc()),
            Self::TextNode(_) => Cow::Owned(vec![]),
        }
    }

    /// Get the children node list in shadow tree
    pub fn children<'c>(&'c self) -> ChildIter<'c, B> {
        match self {
            Self::NativeNode(x) => x.children(),
            Self::VirtualNode(x) => x.children(),
            Self::ComponentNode(x) => x.children(),
            Self::TextNode(x) => ChildIter::new((&**x).into(), TraversalRange::Shadow),
        }
    }

    /// Get the children node list in composed tree
    pub fn composed_children_rc(&self) -> Cow<'_, Vec<NodeRc<B>>> {
        match self {
            Self::NativeNode(x) => x.composed_children_rc(),
            Self::VirtualNode(x) => x.composed_children_rc(),
            Self::ComponentNode(x) => x.composed_children_rc(),
            Self::TextNode(_) => Cow::Owned(vec![]),
        }
    }

    /// Get the children node list in composed tree
    pub fn composed_children<'c>(&'c self) -> ChildIter<'c, B> {
        match self {
            Self::NativeNode(x) => x.composed_children(),
            Self::VirtualNode(x) => x.composed_children(),
            Self::ComponentNode(x) => x.composed_children(),
            Self::TextNode(x) => ChildIter::new((&**x).into(), TraversalRange::Composed),
        }
    }

    /// Get the children node list in composed tree
    pub fn dfs<'c>(&'c self, range: TraversalRange, order: TraversalOrder) -> DfsIter<'c, B> {
        match self {
            Self::NativeNode(x) => x.dfs(range, order),
            Self::VirtualNode(x) => x.dfs(range, order),
            Self::ComponentNode(x) => x.dfs(range, order),
            Self::TextNode(x) => x.dfs(range, order),
        }
    }

    pub fn to_html<T: std::io::Write>(&self, s: &mut T) -> std::io::Result<()> {
        match self {
            Self::NativeNode(x) => x.to_html(s),
            Self::VirtualNode(x) => x.to_html(s),
            Self::ComponentNode(x) => x.to_html(s),
            Self::TextNode(x) => x.to_html(s),
        }
    }

    pub(super) fn collect_backend_nodes<'b>(&'b self, v: &'b mut Vec<NodeRc<B>>) {
        match self {
            Self::NativeNode(x) => x.collect_backend_nodes(v),
            Self::VirtualNode(x) => x.collect_backend_nodes(v),
            Self::ComponentNode(x) => x.collect_backend_nodes(v),
            Self::TextNode(x) => x.collect_backend_nodes(v),
        }
    }

    pub(super) unsafe fn find_next_backend_child(&self, from_index: usize) -> Option<NodeRc<B>> {
        match self {
            Self::NativeNode(x) => x.find_next_backend_child(from_index),
            Self::VirtualNode(x) => x.find_next_backend_child(from_index),
            Self::ComponentNode(x) => x.find_next_backend_child(from_index),
            Self::TextNode(_) => None,
        }
    }

    pub(super) unsafe fn find_next_sibling(&self, include_self: bool) -> Option<NodeRc<B>> {
        match self {
            Self::NativeNode(x) => x.find_next_sibling(include_self),
            Self::VirtualNode(x) => x.find_next_sibling(include_self),
            Self::ComponentNode(x) => x.find_next_sibling(include_self),
            Self::TextNode(_) => None,
        }
    }

    pub(super) fn debug_fmt(&self, f: &mut fmt::Formatter<'_>, level: u32) -> fmt::Result {
        match self {
            Self::NativeNode(x) => x.debug_fmt(f, level),
            Self::VirtualNode(x) => x.debug_fmt(f, level),
            Self::ComponentNode(x) => x.debug_fmt(f, level),
            Self::TextNode(x) => x.debug_fmt(f, level),
        }
    }
}

impl<'a, B: Backend> fmt::Debug for Node<'a, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug_fmt(f, 0)
    }
}

/// A mutable reference to a node
pub enum NodeMut<'a, B: Backend> {
    NativeNode(&'a mut NativeNode<B>),
    VirtualNode(&'a mut VirtualNode<B>),
    ComponentNode(&'a mut ComponentNode<B>),
    TextNode(&'a mut TextNode<B>),
}

impl<'a, B: Backend> NodeMut<'a, B> {
    /// Get the `NodeRc` of the node
    pub fn rc(&self) -> NodeRc<B> {
        match self {
            Self::NativeNode(x) => NodeRc::NativeNode(x.rc()),
            Self::VirtualNode(x) => NodeRc::VirtualNode(x.rc()),
            Self::ComponentNode(x) => NodeRc::ComponentNode(x.rc()),
            Self::TextNode(x) => NodeRc::TextNode(x.rc()),
        }
    }

    /// Get an immutable reference `Node`
    pub fn as_ref<'b>(&'b self) -> Node<'b, B> {
        match self {
            Self::NativeNode(x) => Node::NativeNode(x),
            Self::VirtualNode(x) => Node::VirtualNode(x),
            Self::ComponentNode(x) => Node::ComponentNode(x),
            Self::TextNode(x) => Node::TextNode(x),
        }
    }

    /// Get another mutable reference `NodeMut`, borrowing out the current one
    pub fn as_mut<'b>(&'b mut self) -> NodeMut<'b, B> {
        match self {
            Self::NativeNode(x) => NodeMut::NativeNode(x),
            Self::VirtualNode(x) => NodeMut::VirtualNode(x),
            Self::ComponentNode(x) => NodeMut::ComponentNode(x),
            Self::TextNode(x) => NodeMut::TextNode(x),
        }
    }

    pub(crate) fn backend_node_mut<'b>(&'b mut self) -> Option<BackendNodeRefMut<'b, B>> {
        match self {
            Self::NativeNode(x) => Some(BackendNodeRefMut::Element(&mut x.backend_element)),
            Self::VirtualNode(_) => None,
            Self::ComponentNode(x) => Some(BackendNodeRefMut::Element(&mut x.backend_element)),
            Self::TextNode(x) => Some(BackendNodeRefMut::TextNode(&mut x.backend_element)),
        }
    }

    pub(super) fn set_parent(&mut self, p: Option<NodeWeak<B>>) {
        match self {
            Self::NativeNode(x) => x.set_parent(p),
            Self::VirtualNode(x) => x.set_parent(p),
            Self::ComponentNode(x) => x.set_parent(p),
            Self::TextNode(x) => x.set_parent(p),
        }
    }

    pub(super) fn set_composed_parent(&mut self, p: Option<NodeWeak<B>>) {
        match self {
            Self::NativeNode(x) => x.set_composed_parent(p),
            Self::VirtualNode(x) => x.set_composed_parent(p),
            Self::ComponentNode(x) => x.set_composed_parent(p),
            Self::TextNode(x) => x.set_composed_parent(p),
        }
    }

    /// Get the owner component node.
    /// Returns an iterator containing the owner node itself.
    pub fn owner_mut<'c>(&'c mut self) -> SingleNodeIterMut<'c, B, &'c mut ComponentNode<B>> {
        match self {
            Self::NativeNode(x) => x.owner_mut(),
            Self::VirtualNode(x) => x.owner_mut(),
            Self::ComponentNode(x) => x.owner_mut(),
            Self::TextNode(x) => x.owner_mut(),
        }
    }

    /// Get the parent node in shadow tree.
    /// Returns an iterator containing the parent node itself.
    pub fn parent_mut<'c>(&'c mut self) -> SingleNodeIterMut<'c, B, NodeMut<'c, B>> {
        match self {
            Self::NativeNode(x) => x.parent_mut(),
            Self::VirtualNode(x) => x.parent_mut(),
            Self::ComponentNode(x) => x.parent_mut(),
            Self::TextNode(x) => x.parent_mut(),
        }
    }

    /// Iterate ancestors in shadow tree
    pub fn ancestors_mut<'c>(&'c mut self, order: TraversalOrder) -> AncestorIterMut<'c, B> {
        match self {
            Self::NativeNode(x) => x.ancestors_mut(order),
            Self::VirtualNode(x) => x.ancestors_mut(order),
            Self::ComponentNode(x) => x.ancestors_mut(order),
            Self::TextNode(x) => x.ancestors_mut(order),
        }
    }

    /// Iterate ancestors in composed tree
    pub fn composed_ancestors_mut<'c>(
        &'c mut self,
        order: TraversalOrder,
    ) -> AncestorIterMut<'c, B> {
        match self {
            Self::NativeNode(x) => x.composed_ancestors_mut(order),
            Self::VirtualNode(x) => x.composed_ancestors_mut(order),
            Self::ComponentNode(x) => x.composed_ancestors_mut(order),
            Self::TextNode(x) => x.composed_ancestors_mut(order),
        }
    }

    /// Get the children node list in shadow tree
    pub fn children_mut(&mut self) -> ChildIterMut<B> {
        match self {
            Self::NativeNode(x) => x.children_mut(),
            Self::VirtualNode(x) => x.children_mut(),
            Self::ComponentNode(x) => x.children_mut(),
            Self::TextNode(x) => ChildIterMut::new((&mut **x).into(), TraversalRange::Shadow),
        }
    }

    /// Get the children node list in composed tree
    pub fn composed_children_mut(&mut self) -> ChildIterMut<B> {
        match self {
            Self::NativeNode(x) => x.composed_children_mut(),
            Self::VirtualNode(x) => x.composed_children_mut(),
            Self::ComponentNode(x) => x.composed_children_mut(),
            Self::TextNode(x) => ChildIterMut::new((&mut **x).into(), TraversalRange::Composed),
        }
    }

    /// Get the children node list in composed tree
    pub fn dfs_mut(&mut self, range: TraversalRange, order: TraversalOrder) -> DfsIterMut<B> {
        match self {
            Self::NativeNode(x) => x.dfs_mut(range, order),
            Self::VirtualNode(x) => x.dfs_mut(range, order),
            Self::ComponentNode(x) => x.dfs_mut(range, order),
            Self::TextNode(x) => x.dfs_mut(range, order),
        }
    }

    fn debug_fmt(&self, f: &mut fmt::Formatter<'_>, level: u32) -> fmt::Result {
        self.as_ref().debug_fmt(f, level)
    }

    pub(super) fn set_attached(&mut self) {
        match self {
            Self::NativeNode(x) => x.set_attached(),
            Self::VirtualNode(x) => x.set_attached(),
            Self::ComponentNode(x) => x.set_attached(),
            Self::TextNode(_) => {}
        }
    }

    pub(super) fn set_detached(&mut self) {
        match self {
            Self::NativeNode(x) => x.set_detached(),
            Self::VirtualNode(x) => x.set_detached(),
            Self::ComponentNode(x) => x.set_detached(),
            Self::TextNode(_) => {}
        }
    }
}

impl<'a, B: Backend> fmt::Debug for NodeMut<'a, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug_fmt(f, 0)
    }
}

pub type TemplateNodeFn<B, T> = Box<dyn Fn(&T) -> NodeRc<B>>;

pub(super) enum MeRefMutOrRaw<'a, T> {
    Me(std::cell::RefMut<'a, T>),
    Raw(&'a mut T),
}

macro_rules! some_node_def {
    ($t: ident, $rc: ident, $weak: ident, $r: ident, $rm: ident) => {
        /// A `NodeRc` with node type, representing a ref-counted node.
        pub struct $rc<B: Backend> {
            pub(super) c: Rc<MeCell<$t<B>>>,
        }

        impl<B: Backend> $rc<B> {
            #[allow(dead_code)]
            pub(crate) fn new_with_me_cell_group(c: $t<B>) -> Self {
                Self {
                    c: Rc::new(MeCell::new_group(c)),
                }
            }

            pub(super) fn another_me_cell(&self) -> MeCell<()> {
                self.c.another(())
            }

            /// Borrow the node.
            /// Panics if any node has been mutably borrowed.
            pub fn borrow<'a>(&'a self) -> $r<'a, B> {
                $r { c: self.c.borrow() }
            }

            /// Borrow the node.
            /// Return `Err` if any node has been mutably borrowed.
            pub fn try_borrow<'a>(&'a self) -> Result<$r<'a, B>, NodeBorrowError> {
                self.c.try_borrow().map(|c| $r { c })
            }

            /// Borrow the node mutably.
            /// Panics if any node has been borrowed.
            pub fn borrow_mut<'a>(&'a self) -> $rm<'a, B> {
                $rm {
                    c: ManuallyDrop::new(MeRefMutOrRaw::Me(self.c.borrow_mut())),
                }
            }

            /// Borrow the node mutably.
            /// Return `Err` if any node has been borrowed.
            pub fn try_borrow_mut<'a>(&'a self) -> Result<$rm<'a, B>, NodeBorrowError> {
                self.c.try_borrow_mut().map(|c| $rm {
                    c: ManuallyDrop::new(MeRefMutOrRaw::Me(c)),
                })
            }

            /// Get a `NodeWeak` with node type
            pub fn downgrade(&self) -> $weak<B> {
                $weak {
                    c: Rc::downgrade(&self.c),
                }
            }

            /// Deref the node unsafely
            /// **Should be done through template engine!**
            #[doc(hidden)]
            pub unsafe fn deref_unsafe(&self) -> &$t<B> {
                self.c.deref_unsafe()
            }

            /// Deref the node unsafely
            /// **Should be done through template engine!**
            #[doc(hidden)]
            pub unsafe fn deref_mut_unsafe(&self) -> &mut $t<B> {
                self.c.deref_mut_unsafe()
            }

            pub(crate) unsafe fn deref_unsafe_with_lifetime<'c, 'b>(&'c self) -> &'b $t<B> {
                self.c.deref_unsafe_with_lifetime()
            }

            pub(crate) unsafe fn deref_mut_unsafe_with_lifetime<'c, 'b>(&'c self) -> &'b mut $t<B> {
                self.c.deref_mut_unsafe_with_lifetime()
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

        /// A `NodeWeak` with node type, representing a weak ref of a ref-counted node.
        pub struct $weak<B: Backend> {
            pub(super) c: Weak<MeCell<$t<B>>>,
        }

        impl<B: Backend> $weak<B> {
            /// Get a `NodeRc` with node type
            pub fn upgrade(&self) -> Option<$rc<B>> {
                self.c.upgrade().map(|x| $rc { c: x })
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

        /// A `NodeRef` with node type, representing a borrowed ref of a ref-counted node.
        /// No other node can be mutably borrowed until this object is dropped.
        #[allow(dead_code)]
        pub struct $r<'a, B: Backend> {
            pub(super) c: Ref<'a, $t<B>>,
        }

        impl<'a, B: Backend> $r<'a, B> {
            /// Get another borrowed ref of the same node
            pub fn clone<'b>(orig: &$r<'b, B>) -> $r<'b, B> {
                $r {
                    c: Ref::clone(&orig.c),
                }
            }

            /// Convert to a `std::cell::Ref` object, keeping borrowing.
            pub fn to_ref(self) -> Ref<'a, $t<B>> {
                self.c
            }
        }

        impl<'a, B: Backend> Deref for $r<'a, B> {
            type Target = $t<B>;
            fn deref(&self) -> &$t<B> {
                &*self.c
            }
        }

        impl<'a, B: Backend> From<$r<'a, B>> for NodeRef<'a, B> {
            fn from(s: $r<'a, B>) -> Self {
                NodeRef::$t(s)
            }
        }

        /// A `NodeRefMut` with node type, representing a mutably borrowed ref of a ref-counted node.
        /// No other node can be borrowed until this object is dropped.
        #[allow(dead_code)]
        pub struct $rm<'a, B: Backend> {
            pub(super) c: ManuallyDrop<MeRefMutOrRaw<'a, $t<B>>>,
        }

        impl<'a, B: Backend> $rm<'a, B> {
            /// Get another mutable reference `NodeMut`, borrowing out the current one
            pub fn as_mut<'b>(&'b mut self) -> $rm<'b, B>
            where
                'a: 'b,
            {
                $rm {
                    c: ManuallyDrop::new(MeRefMutOrRaw::Raw(match &mut *self.c {
                        MeRefMutOrRaw::Me(x) => &mut *x,
                        MeRefMutOrRaw::Raw(x) => &mut *x,
                    })),
                }
            }
        }

        impl<'a, B: Backend> Deref for $rm<'a, B> {
            type Target = $t<B>;
            fn deref(&self) -> &$t<B> {
                match &*self.c {
                    MeRefMutOrRaw::Me(x) => &*x,
                    MeRefMutOrRaw::Raw(x) => &*x,
                }
            }
        }

        impl<'a, B: Backend> DerefMut for $rm<'a, B> {
            fn deref_mut(&mut self) -> &mut $t<B> {
                match &mut *self.c {
                    MeRefMutOrRaw::Me(x) => &mut *x,
                    MeRefMutOrRaw::Raw(x) => &mut *x,
                }
            }
        }

        impl<'a, B: Backend> From<$rm<'a, B>> for NodeRefMut<'a, B> {
            fn from(s: $rm<'a, B>) -> Self {
                NodeRefMut::$t(s)
            }
        }

        impl<'a, B: Backend> Drop for $rm<'a, B> {
            fn drop(&mut self) {
                let scheduler = self.scheduler.clone();
                unsafe {
                    ManuallyDrop::drop(&mut self.c);
                }
                scheduler.run_tasks();
            }
        }

        impl<B: Backend> $t<B> {
            /// Get the `NodeRc` of the node
            pub fn rc(&self) -> $rc<B> {
                match &self.self_weak {
                    None => unreachable!(),
                    Some(x) => x.upgrade().unwrap(),
                }
            }
        }

        impl<'a, B: Backend> From<&'a $t<B>> for Node<'a, B> {
            fn from(s: &'a $t<B>) -> Node<'a, B> {
                Node::$t(s)
            }
        }

        impl<'a, B: Backend> From<&'a mut $t<B>> for NodeMut<'a, B> {
            fn from(s: &'a mut $t<B>) -> NodeMut<'a, B> {
                NodeMut::$t(s)
            }
        }
    };
}
