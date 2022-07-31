use crate::{error::Error, backend::{tree, SupportBackend, Backend}};

/// A helper type for a node with child nodes
#[derive(Debug)]
pub struct Node<B: Backend, N: SupportBackend<B>, C> {
    pub tag: N::Target,
    pub child_nodes: SlotChildren<C>,
}

/// A helper type for control flow node such as "if" node
#[derive(Debug)]
pub struct ControlNode<C> {
    pub forest_token: tree::ForestToken,
    pub content: C,
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

/// A helper type for "for" node
#[derive(Debug)]
pub struct Loop<A, C> {
    pub list_diff_algo: A,
    pub items: Vec<C>,
}

/// A helper type for store slot children
// Since rust GAT is not stable yet, we cannot make it a trait - use enum instead
#[derive(Debug)]
pub enum SlotChildren<C> {
    None,
    Single(C),
    Multiple(Vec<C>),
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
