use std::{hash::{Hash, BuildHasher}, collections::{hash_map::RandomState, HashMap}, marker::PhantomData};

use super::*;

/// A key type that can be used for key-diff algorithm
pub struct ListKey<K: Eq + Hash, S: BuildHasher = RandomState> {
    key: K,
    _phantom: PhantomData<S>,
}

struct ListKeyRef<'a, R: Eq + Hash + ToOwned + ?Sized, S: BuildHasher = RandomState> {
    key: &'a R,
    _phantom: PhantomData<S>,
}

impl<'a, R: Eq + Hash + ToOwned + ?Sized, S: BuildHasher> ListDiffRef<'a> for ListKeyRef<'a, R, S>
where
    <R as ToOwned>::Owned: Eq + Hash
{
    type Owned = ListKey<<R as ToOwned>::Owned, S>;

    fn to_owned_list_diff(&self) -> Self::Owned {
        ListKey {
            key: self.key.to_owned(),
            _phantom: PhantomData,
        }
    }
}

pub trait AsListKey<R: Eq + Hash + ToOwned + ?Sized, S: BuildHasher = RandomState> {
    fn as_list_key(&self) -> &R;
}

impl<'a, R: Eq + Hash + ToOwned + ?Sized, S: BuildHasher, T: AsListKey<R, S>> AsListDiff<'a, ListKeyRef<'a, R, S>> for T
where
    <R as ToOwned>::Owned: Eq + Hash
{
    fn as_list_diff(&'a self) -> ListKeyRef<'a, R, S> {
        ListKeyRef {
            key: self.as_list_key(),
            _phantom: PhantomData,
        }
    }
}

impl<K: Eq + Hash, S: BuildHasher> ListDiff for ListKey<K, S> {}

pub struct ListKeyAlgo<K: Eq + Hash, S: BuildHasher> {
    map: HashMap<K, usize, S>,
}

impl<K: Eq + Hash, S: BuildHasher> ListKeyAlgo<K, S> {
    fn list_diff_new() -> Self {
        todo!()
    }

    fn list_diff_update<
        'a,
        B: Backend,
        C,
        D,
        N: 'a + FnMut(&mut ForestNodeMut<B::GeneralElement>, &D) -> Result<C, Error>,
        U: 'a + FnMut(&mut C, &mut ForestNodeMut<B::GeneralElement>, &D) -> Result<(), Error>,
    >(
        &'a mut self,
        items: &mut Vec<C>,
        new_child_fn: N,
        update_child_fn: U,
    ) -> ListKeyAlgoUpdate<'a, K, S, B, C, D, N, U> {
        todo!()
    }
}

pub struct ListKeyAlgoUpdate<
    'a,
    K: Eq + Hash,
    S: BuildHasher,
    B: Backend,
    C,
    D,
    N: 'a + FnMut(&mut ForestNodeMut<B::GeneralElement>, &D) -> Result<C, Error>,
    U: 'a + FnMut(&mut C, &mut ForestNodeMut<B::GeneralElement>, &D) -> Result<(), Error>,
> {
    map: &'a mut HashMap<K, usize, S>,
    items: &'a mut Vec<C>,
    new_child_fn: N,
    update_child_fn: U,
    _phantom: PhantomData<(B, D)>,
}

impl<
    'a,
    K: Eq + Hash,
    S: BuildHasher,
    B: Backend,
    C,
    D,
    N: 'a + FnMut(&mut ForestNodeMut<B::GeneralElement>, &D) -> Result<C, Error>,
    U: 'a + FnMut(&mut C, &mut ForestNodeMut<B::GeneralElement>, &D) -> Result<(), Error>,
> ListKeyAlgoUpdate<'a, K, S, B, C, D, N, U> {
    fn next<'b>(
        &mut self,
        list_diff: impl ListDiffRef<'b, Owned = ListKey<K, S>>,
        data: &D,
    ) -> Result<(), Error> {
        todo!()
    }

    fn end(self) -> Result<(), Error> {
        todo!()
    }
}
