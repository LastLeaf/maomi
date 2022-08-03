use std::{hash::Hash, collections::HashMap, marker::PhantomData};

use super::*;

/// A key type that can be used for key-diff algorithm
pub struct ListKey<K: Eq + Hash> {
    key: K,
}

struct ListKeyRef<'a, R: Eq + Hash + ToOwned + ?Sized> {
    key: &'a R,
}

impl<'a, R: Eq + Hash + ToOwned + ?Sized> ListDiffRef<'a> for ListKeyRef<'a, R>
where
    <R as ToOwned>::Owned: Eq + Hash
{
    type Owned = ListKey<<R as ToOwned>::Owned>;

    fn to_owned_list_diff(&self) -> Self::Owned {
        ListKey {
            key: self.key.to_owned(),
        }
    }
}

pub trait AsListKey<R: Eq + Hash + ToOwned + ?Sized> {
    fn as_list_key(&self) -> &R;
}

impl<'a, R: Eq + Hash + ToOwned + ?Sized, T: AsListKey<R>> AsListDiff<'a, ListKeyRef<'a, R>> for T
where
    <R as ToOwned>::Owned: Eq + Hash
{
    fn as_list_diff(&'a self) -> ListKeyRef<'a, R> {
        ListKeyRef {
            key: self.as_list_key(),
        }
    }
}

impl<K: Eq + Hash> ListDiff for ListKey<K> {}

pub struct ListKeyAlgo<K: Eq + Hash> {
    map: HashMap<K, usize>,
}

impl<K: Eq + Hash> ListKeyAlgo<K> {
    pub fn list_diff_new() -> Self {
        todo!()
    }

    pub fn list_diff_update<'a, B: Backend, C>(
        &'a mut self,
        items: &mut Vec<C>,
        parent: &mut ForestNodeMut<B::GeneralElement>,
    ) -> ListKeyAlgoUpdate<'a, K, B, C> {
        todo!()
    }
}

pub struct ListKeyAlgoUpdate<
    'a,
    K: Eq + Hash,
    B: Backend,
    C,
> {
    map: &'a mut HashMap<K, usize>,
    items: &'a mut Vec<C>,
    _phantom: PhantomData<B>,
}

impl<
    'a,
    K: Eq + Hash,
    B: Backend,
    C,
> ListKeyAlgoUpdate<'a, K, B, C> {
    pub fn next<'b>(
        &mut self,
        list_diff: impl ListDiffRef<'b, Owned = ListKey<K>>,
        create_fn: impl FnOnce(&mut ForestNodeMut<B::GeneralElement>) -> Result<C, Error>,
        update_fn: impl FnOnce(&mut C, &mut ForestNodeMut<B::GeneralElement>) -> Result<(), Error>,
    ) -> Result<(), Error> {
        todo!()
    }

    pub fn end(self) -> Result<(), Error> {
        todo!()
    }
}
