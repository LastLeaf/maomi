use std::{collections::HashMap, hash::Hash};

use crate::{
    backend::{tree, SupportBackend},
    error::Error,
};
use tree::ForestTokenAddr;

/// A weak ref to the owner
pub trait OwnerWeak {
    fn apply_updates(&self) -> Result<(), Error>;
    fn clone_owner_weak(&self) -> Box<dyn OwnerWeak>;
}

/// A helper type for a node with child nodes
#[derive(Debug)]
pub struct Node<N: SupportBackend, C> {
    pub tag: N::Target,
    pub child_nodes: SlotChildren<ForestTokenAddr, C>,
}

impl<N: SupportBackend, C> Node<N, C> {
    #[inline(always)]
    pub fn new(tag: N::Target, child_nodes: SlotChildren<ForestTokenAddr, C>) -> Self {
        Self { tag, child_nodes }
    }

    /// Iterator over slots of the node
    #[inline]
    pub fn iter_slots(&self) -> SlotChildrenIter<ForestTokenAddr, C> {
        self.child_nodes.iter()
    }

    /// If the node has only one slot, returns it
    #[inline]
    pub fn single_slot(&self) -> Option<&C> {
        self.child_nodes.single_slot()
    }
}

/// A helper type for control flow node such as "if" node
#[derive(Debug)]
pub struct ControlNode<C> {
    pub forest_token: tree::ForestToken,
    pub content: C,
}

impl<C> ControlNode<C> {
    #[inline(always)]
    pub fn new(forest_token: tree::ForestToken, content: C) -> Self {
        Self {
            forest_token,
            content,
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
pub enum SlotChildren<K: Hash + Eq, C> {
    None,
    Single(K, C),
    Multiple(HashMap<K, C>),
}

impl<K: Hash + Eq, C> SlotChildren<K, C> {
    #[doc(hidden)]
    #[inline]
    pub fn add(&mut self, k: K, v: C) {
        if let Self::Single(..) = self {
            if let Self::Single(k2, v2) = std::mem::replace(self, Self::None) {
                *self = Self::Multiple(HashMap::from_iter([(k2, v2), (k, v)]));
            } else {
                unreachable!();
            }
        } else if let Self::Multiple(map) = self {
            map.insert(k, v);
        } else {
            *self = Self::Single(k, v);
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn remove(&mut self, k: K) -> Result<C, Error> {
        if let Self::Single(k2, _) = self {
            if *k2 == k {
                if let Self::Single(_, v2) = std::mem::replace(self, Self::None) {
                    Ok(v2)
                } else {
                    unreachable!()
                }
            } else {
                Err(Error::ListChangeWrong)
            }
        } else if let Self::Multiple(map) = self {
            map.remove(&k).ok_or(Error::ListChangeWrong)
        } else {
            Err(Error::ListChangeWrong)
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn get(&self, k: K) -> Result<&C, Error> {
        if let Self::Single(k2, v2) = self {
            if *k2 == k {
                Ok(v2)
            } else {
                Err(Error::ListChangeWrong)
            }
        } else if let Self::Multiple(vec) = self {
            vec.get(&k).ok_or(Error::ListChangeWrong)
        } else {
            Err(Error::ListChangeWrong)
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn get_mut(&mut self, k: K) -> Result<&mut C, Error> {
        if let Self::Single(k2, v2) = self {
            if *k2 == k {
                Ok(v2)
            } else {
                Err(Error::ListChangeWrong)
            }
        } else if let Self::Multiple(vec) = self {
            vec.get_mut(&k).ok_or(Error::ListChangeWrong)
        } else {
            Err(Error::ListChangeWrong)
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn update(&mut self) -> SlotChildrenUpdate<K, C> {
        SlotChildrenUpdate {
            cur_map: match self {
                Self::None => None,
                Self::Single(_, _) => None,
                Self::Multiple(map) => Some(HashMap::with_capacity(map.len())),
            },
            old: self,
            old_single_matched: false,
            removed_old_single: None,
        }
    }

    /// Iterator over slots
    #[inline]
    pub fn iter(&self) -> SlotChildrenIter<K, C> {
        (&self).into_iter()
    }

    /// If there is only one slot, returns it
    #[inline]
    pub fn single_slot(&self) -> Option<&C> {
        match self {
            Self::None => None,
            Self::Single(_, c) => Some(c),
            Self::Multiple(map) => match map.len() {
                1 => map.values().next(),
                _ => None,
            },
        }
    }
}

impl<'a, K: Hash + Eq, C> IntoIterator for &'a SlotChildren<K, C> {
    type Item = (&'a K, &'a C);
    type IntoIter = SlotChildrenIter<'a, K, C>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            SlotChildren::None => SlotChildrenIter::None,
            SlotChildren::Single(k, x) => SlotChildrenIter::Single(k, x),
            SlotChildren::Multiple(x) => SlotChildrenIter::Multiple(x.iter()),
        }
    }
}

pub enum SlotChildrenIter<'a, K: Hash + Eq, C> {
    None,
    Single(&'a K, &'a C),
    Multiple(std::collections::hash_map::Iter<'a, K, C>),
}

impl<'a, K: Hash + Eq, C> Iterator for SlotChildrenIter<'a, K, C> {
    type Item = (&'a K, &'a C);

    fn next(&mut self) -> Option<Self::Item> {
        if let Self::Single(k, x) = self {
            let x = *x;
            let k = *k;
            *self = Self::None;
            Some((k, x))
        } else if let Self::Multiple(x) = self {
            x.next()
        } else {
            None
        }
    }
}

#[doc(hidden)]
pub struct SlotChildrenUpdate<'a, K: Hash + Eq, C> {
    cur_map: Option<HashMap<K, C>>,
    old: &'a mut SlotChildren<K, C>,
    old_single_matched: bool,
    removed_old_single: Option<(K, C)>,
}

impl<'a, K: Hash + Eq, C> SlotChildrenUpdate<'a, K, C> {
    #[doc(hidden)]
    #[inline]
    pub fn add(&mut self, k: K, v: C) {
        if let Some(map) = self.cur_map.as_mut() {
            map.insert(k, v);
        } else if let SlotChildren::Single(_, _) = self.old {
            if self.old_single_matched {
                if let SlotChildren::Single(k2, v2) =
                    std::mem::replace(self.old, SlotChildren::None)
                {
                    *self.old = SlotChildren::Multiple(HashMap::from_iter([(k2, v2), (k, v)]));
                } else {
                    unreachable!();
                }
            } else {
                if let SlotChildren::Single(k2, v2) =
                    std::mem::replace(self.old, SlotChildren::Single(k, v))
                {
                    self.removed_old_single = Some((k2, v2));
                }
                self.old_single_matched = true;
            }
        } else if let SlotChildren::Multiple(map) = self.old {
            map.insert(k, v);
        } else {
            *self.old = SlotChildren::Single(k, v);
            self.old_single_matched = true;
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn reuse(&mut self, k: K) -> Result<&mut C, Error> {
        let ret = if let Some(map) = self.cur_map.as_mut() {
            let v = if let SlotChildren::Multiple(map) = self.old {
                map.remove(&k)
            } else {
                unreachable!();
            }
            .ok_or(Error::ListChangeWrong)?;
            map.entry(k).or_insert(v)
        } else if let SlotChildren::Single(k2, v) = self.old {
            if self.old_single_matched || *k2 != k {
                return Err(Error::ListChangeWrong);
            } else {
                self.old_single_matched = true;
                v
            }
        } else {
            return Err(Error::ListChangeWrong);
        };
        Ok(ret)
    }

    #[doc(hidden)]
    #[inline]
    pub fn finish(self, mut item_fn: impl FnMut(K, C) -> Result<(), Error>) -> Result<(), Error> {
        if let Some(map) = self.cur_map {
            if let SlotChildren::Multiple(map) =
                std::mem::replace(self.old, SlotChildren::Multiple(map))
            {
                for (k, c) in map {
                    item_fn(k, c)?;
                }
            } else {
                unreachable!();
            }
        } else if let Some((k, c)) = self.removed_old_single {
            item_fn(k, c)?;
        } else if !self.old_single_matched {
            if let SlotChildren::Single(k, c) = std::mem::replace(self.old, SlotChildren::None) {
                item_fn(k, c)?;
            }
        }
        Ok(())
    }
}

/// A helper type for slot changes
// Since rust GAT is not stable yet, we cannot make it a trait - use enum instead
#[derive(Debug, Clone, PartialEq)]
pub enum SlotChange<N, M, T> {
    Unchanged(N, M, T),
    DataChanged(N, M, T),
    Added(N, M, T),
    Removed(M),
}
