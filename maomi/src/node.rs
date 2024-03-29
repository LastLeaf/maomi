//! Helper types for node trees.

use std::{collections::HashMap, hash::Hash, marker::PhantomData, any::Any};

use crate::{
    backend::{tree, SupportBackend},
    error::Error,
};
use tree::ForestTokenAddr;

/// An unsafe option as a union used to reduce some checking overhead.
pub union UnionOption<T> {
    none: (),
    some: std::mem::ManuallyDrop<T>,
}

impl<T> UnionOption<T> {
    /// Create a none value.
    #[inline(always)]
    pub fn none() -> Self {
        Self { none: () }
    }

    /// Create a none value.
    #[inline(always)]
    pub fn some(inner: T) -> Self {
        Self { some: std::mem::ManuallyDrop::new(inner) }
    }

    /// Assume it is not none and get the contained value.
    #[inline(always)]
    pub unsafe fn unwrap_unchecked(self) -> T {
        std::mem::ManuallyDrop::into_inner(self.some)
    }

    /// Assume it is not none and get the reference.
    #[inline(always)]
    pub unsafe fn as_ref_unchecked(&self) -> &T {
        &self.some
    }
}

/// A weak ref to the owner.
/// 
/// This is used by the backend implementor.
/// *In most cases, it should not be used in component implementors.*
pub trait OwnerWeak {
    /// Schedule an update on the owner.
    fn apply_updates(&self) -> Result<(), Error>;
    /// Clone the owner itself.
    fn clone_owner_weak(&self) -> Box<dyn OwnerWeak>;
}

/// A general node type.
pub struct DynNode {
    inner: Box<dyn Any>,
}

impl DynNode {
    /// Build from a node.
    #[inline(always)]
    pub fn new<N: 'static>(n: N) -> Self {
        Self { inner: Box::new(n) }
    }

    /// Cast into a node of specified type.
    #[inline(always)]
    pub unsafe fn node_unchecked<N: 'static>(&mut self) -> &mut N {
        &mut *(&mut *self.inner as *mut dyn Any as *mut N)
    }

    /// Cast into a node of specified type.
    #[inline(always)]
    pub fn as_mut<N: 'static>(&mut self) -> &mut N {
        self.inner.downcast_mut().unwrap()
    }

    /// Cast into a node of specified type.
    #[inline(always)]
    pub fn as_ref<N: 'static>(&self) -> &N {
        self.inner.downcast_ref().unwrap()
    }
}

/// A general node list.
pub type DynNodeList = Box<[DynNode]>;

/// A helper type for a node with child nodes.
#[derive(Debug)]
pub struct Node<N: SupportBackend> {
    /// The node itself.
    pub tag: N::Target,
    /// The child nodes of the node.
    pub child_nodes: N::SlotChildren,
}

impl<N: SupportBackend> Node<N> {
    /// Create a node with specified children.
    #[inline(always)]
    pub fn new(tag: N::Target, child_nodes: N::SlotChildren) -> Self {
        Self { tag, child_nodes }
    }

    /// Iterator over slots of the node.
    #[inline]
    pub fn iter_slots(&self) -> <N::SlotChildren as SlotKindTrait<ForestTokenAddr, DynNodeList>>::Iter<'_> {
        self.child_nodes.iter()
    }

    /// If the node has only one slot, returns it.
    #[inline]
    pub fn single_slot(&self) -> Option<&DynNodeList> {
        self.child_nodes.single_slot()
    }
}

/// A helper type for control flow node such as "if" node and "for" node.
#[derive(Debug)]
pub struct ControlNode<C> {
    /// The backend node token
    /// 
    /// It is auto-managed by the `#[component]` .
    /// Do not touch unless you know how it works exactly.
    pub forest_token: tree::ForestToken,
    /// The content nodes of the control node.
    pub content: C,
}

impl<C> ControlNode<C> {
    /// Create a control node.
    #[inline(always)]
    pub fn new(forest_token: tree::ForestToken, content: C) -> Self {
        Self {
            forest_token,
            content,
        }
    }
}

/// A helper type for "if" and "match" node.
pub struct Branch {
    /// The current branch index.
    pub cur: usize,
    /// Child node list in "if...else" or "match" node.
    pub children: DynNodeList,
}

/// A helper trait for managing slot list and slot content.
/// 
/// It is auto-managed by the `#[component]` .
/// Do not touch unless you know how it works exactly.
pub trait SlotKindTrait<K, C>: Default {
    /// The updater type.
    type Update<'a>: SlotKindUpdateTrait<'a, K, C>
    where
        Self: 'a,
        K: 'a,
        C: 'a;

    /// The iterator type.
    type Iter<'a>: Iterator<Item = &'a C>
    where
        Self: 'a,
        C: 'a;

    /// Whether the slot may update after created
    fn may_update(&self) -> bool;

    /// Add a slot with the slot content.
    #[doc(hidden)]
    fn add(&mut self, k: K, c: C) -> Result<(), Error>;

    /// Remove a slot and return the slot content.
    #[doc(hidden)]
    fn remove(&mut self, k: K) -> Result<C, Error>;

    /// Get a reference of the slot content.
    #[doc(hidden)]
    fn get(&self, k: K) -> Result<&C, Error>;

    /// Get a mutable reference of the slot content.
    #[doc(hidden)]
    fn get_mut(&mut self, k: K) -> Result<&mut C, Error>;
    
    /// Start an update for all slots.
    #[doc(hidden)]
    fn update<'a>(&'a mut self) -> Self::Update<'a>;

    /// Iterator over all slots.
    fn iter<'a>(&'a self) -> Self::Iter<'a>;

    /// If there is only one slot, returns it.
    fn single_slot(&self) -> Option<&C>;
}

/// A helper trait for a group of slot list updates.
pub trait SlotKindUpdateTrait<'a, K: 'a, C: 'a> {
    /// Add a slot with the slot content.
    #[doc(hidden)]
    fn add(&mut self, k: K, c: C) -> Result<(), Error>;

    /// Reuse a slot, returning it.
    #[doc(hidden)]
    fn reuse(&mut self, k: K) -> Result<&mut C, Error>;

    /// Finish update, handling unused items
    #[doc(hidden)]
    fn finish(self, remove_item_fn: impl FnMut(C) -> Result<(), Error>) -> Result<(), Error>;
}

/// A slot list that is always empty.
#[derive(Debug)]
pub struct NoneSlot<K, C> {
    phantom: PhantomData<(K, C)>,
}

impl<K, C> Default for NoneSlot<K, C> {
    #[inline]
    fn default() -> Self {
        Self { phantom: PhantomData }
    }
}

impl<K, C> SlotKindTrait<K, C> for NoneSlot<K, C> {
    type Update<'a> = NoneSlotUpdate<'a, K, C> where K: 'a, C: 'a;
    type Iter<'a> = std::iter::Empty<&'a C> where K: 'a, C: 'a;

    #[inline(always)]
    fn may_update(&self) -> bool {
        false
    }

    #[inline]
    fn add(&mut self, _: K, _: C) -> Result<(), Error> {
        Err(Error::ListChangeWrong)
    }

    #[inline]
    fn remove(&mut self, _: K) -> Result<C, Error> {
        Err(Error::ListChangeWrong)
    }

    #[inline]
    fn get(&self, _: K) -> Result<&C, Error> {
        Err(Error::ListChangeWrong)
    }

    #[inline]
    fn get_mut(&mut self, _: K) -> Result<&mut C, Error> {
        Err(Error::ListChangeWrong)
    }

    #[inline]
    fn update<'a>(&'a mut self) -> Self::Update<'a> {
        NoneSlotUpdate {
            phantom: PhantomData,
        }
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        std::iter::empty()
    }

    #[inline]
    fn single_slot(&self) -> Option<&C> {
        None
    }
}

#[doc(hidden)]
pub struct NoneSlotUpdate<'a, K, C> {
    phantom: PhantomData<&'a (K, C)>,
}

impl<'a, K: 'a, C: 'a> SlotKindUpdateTrait<'a, K, C> for NoneSlotUpdate<'a, K, C> {
    #[inline]
    fn add(&mut self, _: K, _: C) -> Result<(), Error> {
        Err(Error::ListChangeWrong)
    }

    #[inline]
    fn reuse(&mut self, _: K) -> Result<&mut C, Error> {
        Err(Error::ListChangeWrong)
    }

    #[inline]
    fn finish(self, _: impl FnMut(C) -> Result<(), Error>) -> Result<(), Error> {
        Ok(())
    }
}

/// A slot list that always contains a single slot.
/// 
/// It is auto-managed by the `#[component]` .
/// Do not touch unless you know how it works exactly.
#[derive(Debug)]
pub struct StaticSingleSlot<K, C> {
    kc: Option<C>,
    phantom: PhantomData<K>,
}

impl<K, C> Default for StaticSingleSlot<K, C> {
    #[inline]
    fn default() -> Self where Self: Sized {
        Self { kc: None, phantom: PhantomData }
    }
}

impl<K, C> SlotKindTrait<K, C> for StaticSingleSlot<K, C> {
    type Update<'a> = StaticSingleSlotUpdate<'a, K, C> where K: 'a, C: 'a;
    type Iter<'a> = std::option::IntoIter<&'a C> where K: 'a, C: 'a;

    #[inline(always)]
    fn may_update(&self) -> bool {
        false
    }

    #[inline(always)]
    fn add(&mut self, _: K, c: C) -> Result<(), Error> {
        if self.kc.is_some() {
            return Err(Error::ListChangeWrong);
        }
        self.kc = Some(c);
        Ok(())
    }

    #[inline(always)]
    fn remove(&mut self, _: K) -> Result<C, Error> {
        if self.kc.is_none() {
            return Err(Error::ListChangeWrong);
        }
        match self.kc.take() {
            Some(c) => Ok(c),
            None => Err(Error::ListChangeWrong),
        }
    }

    #[inline]
    fn get(&self, _: K) -> Result<&C, Error> {
        self.kc.as_ref().ok_or(Error::ListChangeWrong)
    }

    #[inline]
    fn get_mut(&mut self, _: K) -> Result<&mut C, Error> {
        self.kc.as_mut().ok_or(Error::ListChangeWrong)
    }

    #[inline]
    fn update<'a>(&'a mut self) -> Self::Update<'a> {
        StaticSingleSlotUpdate { s: self, visited: false }
    }

    #[inline]
    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        self.kc.as_ref().into_iter()
    }

    #[inline]
    fn single_slot(&self) -> Option<&C> {
        self.kc.as_ref()
    }
}

#[doc(hidden)]
pub struct StaticSingleSlotUpdate<'a, K, C> {
    s: &'a mut StaticSingleSlot<K, C>,
    visited: bool,
}

impl<'a, K, C> SlotKindUpdateTrait<'a, K, C> for StaticSingleSlotUpdate<'a, K, C> {
    #[inline]
    fn add(&mut self, k: K, c: C) -> Result<(), Error> {
        let ret = self.s.add(k, c);
        if ret.is_ok() {
            self.visited = true;
        }
        ret
    }

    #[inline]
    fn reuse(&mut self, k: K) -> Result<&mut C, Error> {
        let ret = self.s.get_mut(k);
        if ret.is_ok() {
            self.visited = true;
        }
        ret
    }

    #[inline]
    fn finish(self, mut remove_item_fn: impl FnMut(C) -> Result<(), Error>) -> Result<(), Error> {
        if !self.visited {
            if let Some(c) = self.s.kc.take() {
                return remove_item_fn(c);
            }
        }
        Ok(())
    }
}

/// A slot list that can contain any number of slots.
/// 
/// It is auto-managed by the `#[component]` .
/// Do not touch unless you know how it works exactly.
#[derive(Debug)]
pub struct DynamicSlot<K, C> {
    slots: HashMap<K, C>,
}

impl<K, C> Default for DynamicSlot<K, C> {
    #[inline]
    fn default() -> Self where Self: Sized {
        Self { slots: HashMap::new() }
    }
}

impl<K: Hash + Eq, C> SlotKindTrait<K, C> for DynamicSlot<K, C> {
    type Update<'a> = DynamicSlotUpdate<'a, K, C> where K: 'a, C: 'a;
    type Iter<'a> = std::collections::hash_map::Values<'a, K, C> where K: 'a, C: 'a;

    #[inline(always)]
    fn may_update(&self) -> bool {
        true
    }

    #[inline]
    fn add(&mut self, k: K, v: C) -> Result<(), Error> {
        self.slots.insert(k, v);
        Ok(())
    }

    #[inline]
    fn remove(&mut self, k: K) -> Result<C, Error> {
        self.slots.remove(&k).ok_or(Error::ListChangeWrong)
    }

    #[inline]
    fn get(&self, k: K) -> Result<&C, Error> {
        self.slots.get(&k).ok_or(Error::ListChangeWrong)
    }

    #[inline]
    fn get_mut(&mut self, k: K) -> Result<&mut C, Error> {
        self.slots.get_mut(&k).ok_or(Error::ListChangeWrong)
    }

    #[inline]
    fn update(&mut self) -> Self::Update<'_> {
        DynamicSlotUpdate {
            cur_map: HashMap::with_capacity(self.slots.len()),
            old: self,
        }
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        self.slots.values()
    }

    #[inline]
    fn single_slot(&self) -> Option<&C> {
        match self.slots.len() {
            1 => self.slots.values().next(),
            _ => None,
        }
    }
}

#[doc(hidden)]
pub struct DynamicSlotUpdate<'a, K, C> {
    cur_map: HashMap<K, C>,
    old: &'a mut DynamicSlot<K, C>,
}

impl<'a, K: Hash + Eq, C> SlotKindUpdateTrait<'a, K, C> for DynamicSlotUpdate<'a, K, C> {
    #[inline]
    fn add(&mut self, k: K, v: C) -> Result<(), Error> {
        self.cur_map.insert(k, v);
        Ok(())
    }

    #[inline]
    fn reuse(&mut self, k: K) -> Result<&mut C, Error> {
        let c = self.old.slots.remove(&k).ok_or(Error::ListChangeWrong)?;
        let ret = self.cur_map.entry(k).or_insert(c);
        Ok(ret)
    }

    #[inline]
    fn finish(self, mut item_fn: impl FnMut(C) -> Result<(), Error>) -> Result<(), Error> {
        let r = std::mem::replace(&mut self.old.slots, self.cur_map);
        for (_, c) in r {
            item_fn(c)?;
        }
        Ok(())
    }
}

/// A helper type for slot changes
/// 
/// It is auto-managed by the `#[component]` .
/// Do not touch unless you know how it works exactly.
#[derive(Debug, Clone, PartialEq)]
pub enum SlotChange<N, M, T> {
    /// The slot is not changed.
    Unchanged(N, M, T),
    /// The data of the slot may have changed.
    DataChanged(N, M, T),
    /// The slot is added.
    Added(N, M, T),
    /// The slot is removed.
    Removed(M),
}
