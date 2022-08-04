use std::{hash::Hash, collections::HashMap, marker::PhantomData, borrow::Borrow};

use crate::backend::BackendGeneralElement;
use super::*;

pub trait AsListKey {
    type ListKey: Eq + Hash + ToOwned + ?Sized;

    fn as_list_key(&self) -> &Self::ListKey;
}

impl<T: AsListKey> AsListKey for &'_ T {
    type ListKey = T::ListKey;

    fn as_list_key(&self) -> &Self::ListKey {
        <T as AsListKey>::as_list_key(self)
    }
}

pub struct ListKeyAlgo<B: Backend, K: Eq + Hash> {
    map: HashMap<K, (usize, ForestToken)>,
    _phantom: PhantomData<B>,
}

impl<B: Backend, K: Eq + Hash> ListKeyAlgo<B, K> {
    pub fn list_diff_new() -> Self {
        Self { map: HashMap::new(), _phantom: PhantomData }
    }

    pub fn list_diff_update<'a, 'b, C>(
        &'a mut self,
        items: &'a mut Vec<C>,
        backend_element: &'a mut ForestNodeMut<'b, B::GeneralElement>,
        size_hint: usize,
    ) -> ListKeyAlgoUpdate<'a, 'b, K, B, C> {
        ListKeyAlgoUpdate {
            old_map: &mut self.map,
            stable_pos: Vec::with_capacity(size_hint),
            items,
            backend_element,
            _phantom: PhantomData,
        }
    }
}

enum KeyChange<B: Backend, C> {
    OldPos(usize, ForestToken),
    NewChild(C, ForestNodeRc<B::GeneralElement>),
}

pub struct ListKeyAlgoUpdate<
    'a,
    'b,
    K: Eq + Hash,
    B: Backend,
    C,
> {
    old_map: &'a mut HashMap<K, (usize, ForestToken)>,
    stable_pos: Vec<KeyChange<B, C>>,
    items: &'a mut Vec<C>,
    backend_element: &'a mut ForestNodeMut<'b, B::GeneralElement>,
    _phantom: PhantomData<B>,
}

impl<
    'a,
    'b,
    K: Eq + Hash,
    B: Backend,
    C,
> ListKeyAlgoUpdate<'a, 'b, K, B, C> {
    pub fn next<R>(
        &mut self,
        list_key: impl AsListKey<ListKey = R>,
        create_fn: impl FnOnce(&mut ForestNodeMut<B::GeneralElement>) -> Result<C, Error>,
        update_fn: impl FnOnce(&mut C, &mut ForestNodeMut<B::GeneralElement>) -> Result<(), Error>,
    ) -> Result<(), Error>
    where
        R: Eq + Hash + ToOwned<Owned = K> + ?Sized,
        K: Borrow<R>,
    {
        let new_key_ref = list_key.as_list_key();
        if let Some((pos, forest_token)) = self.old_map.remove(new_key_ref) {
            update_fn(
                self.items.get_mut(pos).ok_or(Error::ListChangeWrong)?,
                &mut self.backend_element.borrow_mut_token(&forest_token),
            )?;
            self.stable_pos.push(KeyChange::OldPos(pos, forest_token));
        } else {
            let backend_element = <B::GeneralElement as BackendGeneralElement>::create_virtual_element(self.backend_element)?;
            let c = create_fn(
                &mut self.backend_element.borrow_mut(&backend_element),
            )?;
            self.stable_pos.push(KeyChange::NewChild(c, backend_element));
        }
        Ok(())
    }

    pub fn end(self) -> Result<(), Error> {
        todo!()
    }
}
