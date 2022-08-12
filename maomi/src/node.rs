use std::{rc::Rc, cell::{Cell, RefCell}, collections::HashMap};

use crate::{
    backend::{tree, Backend, SupportBackend},
    error::Error,
};

/// A subtree relation manager
#[derive(Debug, Clone)]
pub struct SubtreeStatus {
    inner: Rc<SubtreeStatusInner>,
}

#[derive(Debug)]
struct SubtreeStatusInner {
    parent: Option<Rc<SubtreeStatusInner>>,
    notifier: RefCell<Option<Box<dyn Fn()>>>,
    slot_content_dirty: Cell<bool>,
}

impl SubtreeStatus {
    /// Create a root manager
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: Rc::new(SubtreeStatusInner {
                parent: None,
                notifier: RefCell::new(None),
                slot_content_dirty: Cell::new(false),
            }),
        }
    }

    /// Create a child manager for current root
    #[inline]
    pub fn new_child(&self) -> Self {
        Self {
            inner: Rc::new(SubtreeStatusInner {
                parent: Some(self.inner.clone()),
                notifier: RefCell::new(None),
                slot_content_dirty: Cell::new(false),
            }),
        }
    }

    pub(crate) fn attach_notifier(&mut self, f: Box<dyn Fn()>) {
        *self.inner.notifier.borrow_mut() = Some(f);
    }

    pub(crate) fn mark_slot_content_dirty(&self) -> bool {
        let mut inner = &self.inner;
        if !inner.slot_content_dirty.replace(true) {
            loop {
                if let Some(p) = inner.parent.as_ref() {
                    if p.slot_content_dirty.replace(true) {
                        break;
                    }
                    inner = p;
                } else {
                    if let Some(f) = inner.notifier.borrow().as_ref() {
                        f();
                    }
                    break;
                }
            }
            true
        } else {
            false
        }
    }

    /// Clear the dirty bit
    /// 
    /// Returns `false` if it is not dirty.
    pub fn clear_slot_content_dirty(&self) -> bool {
        self.inner.slot_content_dirty.replace(false)
    }
}

/// A helper type for a node with child nodes
#[derive(Debug)]
pub struct Node<B: Backend, N: SupportBackend<B>, C> {
    pub tag: N::Target,
    pub child_nodes: SlotChildren<C>,
}

impl<B: Backend, N: SupportBackend<B>, C> Node<B, N, C> {
    #[inline(always)]
    pub fn new(
        tag: N::Target,
        child_nodes: SlotChildren<C>,
    ) -> Self {
        Self {
            tag,
            child_nodes,
        }
    }
}

/// A helper type for control flow node such as "if" node
#[derive(Debug)]
pub struct ControlNode<C> {
    pub forest_token: tree::ForestToken,
    pub content: C,
    pub subtree_status: SubtreeStatus,
}

impl<C> ControlNode<C> {
    #[inline(always)]
    pub fn new(
        forest_token: tree::ForestToken,
        content: C,
        subtree_status: SubtreeStatus,
    ) -> Self {
        Self {
            forest_token,
            content,
            subtree_status,
        }
    }
}

macro_rules! gen_branch_node {
    ($t: ident, $($n: ident),*) => {
        /// A helper type for "if" and "match" node
        #[derive(Debug, Clone, PartialEq)]
        pub enum $t<$($n),*> {
            $($n($n),)*
        }
    };
}
gen_branch_node!(Branch1, B0);
gen_branch_node!(Branch2, B0, B1);
gen_branch_node!(Branch3, B0, B1, B2);
gen_branch_node!(Branch4, B0, B1, B2, B3);
gen_branch_node!(Branch5, B0, B1, B2, B3, B4);
gen_branch_node!(Branch6, B0, B1, B2, B3, B4, B5);
gen_branch_node!(Branch7, B0, B1, B2, B3, B4, B5, B6);
gen_branch_node!(Branch8, B0, B1, B2, B3, B4, B5, B6, B7);
gen_branch_node!(Branch9, B0, B1, B2, B3, B4, B5, B6, B7, B8);
gen_branch_node!(Branch10, B0, B1, B2, B3, B4, B5, B6, B7, B8, B9);
gen_branch_node!(Branch11, B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10);
gen_branch_node!(Branch12, B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11);
gen_branch_node!(Branch13, B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12);
gen_branch_node!(Branch14, B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13);
gen_branch_node!(Branch15, B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14);
gen_branch_node!(Branch16, B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12, B13, B14, B15);

/// A helper type for store slot children
// Since rust GAT is not stable yet, we cannot make it a trait - use enum instead
#[derive(Debug)]
pub enum SlotChildren<C> {
    None,
    Single(C),
    Multiple(HashMap<SlotId, C>),
}

impl<C> SlotChildren<C> {
    pub fn append(&mut self, v: C) {
        todo!()
    }

    pub fn add(&mut self, index: usize, v: C) -> Result<(), Error> {
        if let Self::Single(_) = self {
            if let Self::Single(x) = std::mem::replace(self, Self::None) {
                *self = match index {
                    0 => Self::Multiple(vec![v, x]),
                    1 => Self::Multiple(vec![x, v]),
                    _ => Err(Error::ListChangeWrong)?,
                }
            } else {
                unreachable!();
            }
        } else if let Self::Multiple(vec) = self {
            if index > vec.len() {
                Err(Error::ListChangeWrong)?;
            }
            vec.insert(index, v);
        } else {
            if index > 0 {
                Err(Error::ListChangeWrong)?;
            }
            *self = Self::Single(v);
        }
        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> Result<C, Error> {
        todo!()
    }

    pub fn get_mut(&mut self, index: usize) -> Result<&mut C, Error> {
        if let Self::Single(x) = self {
            match index {
                0 => Ok(x),
                _ => Err(Error::ListChangeWrong),
            }
        } else if let Self::Multiple(vec) = self {
            vec.get_mut(index).ok_or(Error::ListChangeWrong)
        } else {
            Err(Error::ListChangeWrong)
        }
    }

    pub fn iter(&self) -> SlotChildrenIter<C> {
        (&self).into_iter()
    }
}

impl<'a, C> IntoIterator for &'a SlotChildren<C> {
    type Item = &'a C;
    type IntoIter = SlotChildrenIter<'a, C>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            SlotChildren::None => SlotChildrenIter::None,
            SlotChildren::Single(x) => SlotChildrenIter::Single(x),
            SlotChildren::Multiple(x) => SlotChildrenIter::Multiple(x.iter()),
        }
    }
}

pub enum SlotChildrenIter<'a, C> {
    None,
    Single(&'a C),
    Multiple(std::slice::Iter<'a, C>),
}

impl<'a, C> Iterator for SlotChildrenIter<'a, C> {
    type Item = &'a C;

    fn next(&mut self) -> Option<Self::Item> {
        if let Self::Single(x) = self {
            let x = *x;
            *self = Self::None;
            Some(x)
        } else if let Self::Multiple(x) = self {
            x.next()
        } else {
            None
        }
    }
}

/// A helper type for slot changes
// Since rust GAT is not stable yet, we cannot make it a trait - use enum instead
#[derive(Debug, Clone, PartialEq)]
pub enum SlotChange<N, T> {
    Unchanged(N, T),
    Added(N, T),
    Removed(N),
}
