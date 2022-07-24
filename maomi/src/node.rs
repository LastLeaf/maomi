use crate::error::Error;

/// A helper type for a node with child nodes
#[derive(Debug, Clone, PartialEq)]
pub struct Node<N, C> {
    pub node: N,
    pub child_nodes: SlotChildren<C>,
}

/// A helper type for store slot children
// Since rust GAT is not stable yet, we cannot make it a trait - use enum instead
#[derive(Debug, Clone, PartialEq)]
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
        todo!()
    }
}

impl<C> IntoIterator for SlotChildren<C> {
    type Item = C;
    type IntoIter = SlotChildrenIter<C>;

    fn into_iter(self) -> Self::IntoIter {
        todo!()
    }
}

pub enum SlotChildrenIter<C> {
    None,
    Single(C),
    Multiple(std::vec::IntoIter<C>),
}

impl<C> Iterator for SlotChildrenIter<C> {
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::None => None,
            Self::Multiple(x) => x.next(),
            x => {
                let r = std::mem::replace(x, SlotChildrenIter::None);
                match r {
                    Self::Single(x) => Some(x),
                    _ => unreachable!(),
                }
            }
        }
    }
}
