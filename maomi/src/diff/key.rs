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

pub struct KeyList<B: Backend, K: Eq + Hash, C> {
    map: HashMap<K, (usize, C, ForestToken)>,
    _phantom: PhantomData<B>,
}

impl<B: Backend, K: Eq + Hash, C> KeyList<B, K, C> {
    pub fn list_diff_new() -> Self {
        Self {
            map: HashMap::new(),
            _phantom: PhantomData,
        }
    }

    pub fn list_diff_update<'a, 'b>(
        &'a mut self,
        backend_element: &'a mut ForestNodeMut<'b, B::GeneralElement>,
        size_hint: usize,
    ) -> ListKeyAlgoUpdate<'a, 'b, B, K, C> {
        ListKeyAlgoUpdate {
            map: &mut self.map,
            new_map: HashMap::with_capacity(size_hint),
            stable_pos: Vec::with_capacity(size_hint),
            backend_element,
            _phantom: PhantomData,
        }
    }
}

enum KeyChange<B: Backend> {
    OldPos(ForestNodeRc<B::GeneralElement>, usize),
    NewChild(ForestNodeRc<B::GeneralElement>),
}

pub struct ListKeyAlgoUpdate<
    'a,
    'b,
    B: Backend,
    K: Eq + Hash,
    C,
> {
    map: &'a mut HashMap<K, (usize, C, ForestToken)>,
    new_map: HashMap<K, (usize, C, ForestToken)>,
    stable_pos: Vec<KeyChange<B>>,
    backend_element: &'a mut ForestNodeMut<'b, B::GeneralElement>,
    _phantom: PhantomData<B>,
}

impl<
    'a,
    'b,
    B: Backend,
    K: Eq + Hash,
    C,
> ListKeyAlgoUpdate<'a, 'b, B, K, C> {
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
        let new_pos = self.stable_pos.len();
        let new_key_ref = list_key.as_list_key();
        if let Some((pos, mut c, forest_token)) = self.map.remove(new_key_ref) {
            update_fn(
                &mut c,
                &mut self.backend_element.borrow_mut_token(&forest_token),
            )?;
            let rc = self.backend_element.resolve_token(&forest_token);
            self.stable_pos.push(KeyChange::OldPos(rc, pos));
            self.new_map.insert(new_key_ref.to_owned(), (new_pos, c, forest_token));
        } else {
            let backend_element = <B::GeneralElement as BackendGeneralElement>::create_virtual_element(self.backend_element)?;
            let c = create_fn(
                &mut self.backend_element.borrow_mut(&backend_element),
            )?;
            self.new_map.insert(new_key_ref.to_owned(), (new_pos, c, backend_element.token()));
            self.stable_pos.push(KeyChange::NewChild(backend_element));
        }
        Ok(())
    }

    pub fn end(self) -> Result<(), Error> {
        let Self { map, mut stable_pos, new_map, .. } = self;

        // calc the longest increasing subsequence and use it as the unchanged items
        let mut min_index_for_seq_len = Vec::<usize>::with_capacity(stable_pos.len());
        let mut seq_back_ptr = Vec::with_capacity(stable_pos.len());
        for item in stable_pos.iter() {
            if let KeyChange::OldPos(_, pos) = item {
                let pos = *pos;
                let mut left = 0;
                let mut right = min_index_for_seq_len.len();
                while left < right {
                    let mid = (left + right) / 2;
                    if min_index_for_seq_len[mid] < pos {
                        left = mid + 1;
                    } else {
                        right = mid;
                    }
                }
                if left < min_index_for_seq_len.len() {
                    min_index_for_seq_len[left] = pos;
                } else {
                    min_index_for_seq_len.push(pos);
                }
                if left == 0 {
                    seq_back_ptr.push(usize::MAX);
                } else {
                    seq_back_ptr.push(min_index_for_seq_len[left - 1]);
                }
            } else {
                seq_back_ptr.push(usize::MAX);
            }
        }
        if let Some(mut pos) = min_index_for_seq_len.last().cloned() {
            for item in min_index_for_seq_len.iter_mut().rev() {
                *item = pos;
                pos = seq_back_ptr[pos];
            }
        }
        let seq = min_index_for_seq_len;

        // clear the old map to drop the old items
        for (_, _, forest_token) in map.values() {
            <B::GeneralElement as BackendGeneralElement>::detach(
                self.backend_element.borrow_mut_token(forest_token),
            );
        }
        *map = new_map;

        // scan the list and find out moved ones
        let mut list_iter = stable_pos.iter_mut();
        for old_pos in &seq {
            loop {
                let item = match list_iter.next() {
                    Some(x) => x,
                    None => break,
                };
                let rc = match item {
                    KeyChange::OldPos(rc, pos) => {
                        if pos == old_pos {
                            break;
                        }
                        let rc = <B::GeneralElement as BackendGeneralElement>::temp_detach(
                            self.backend_element.borrow_mut(rc),
                        );
                        rc
                    }
                    KeyChange::NewChild(_) => continue,
                };
                *item = KeyChange::NewChild(rc);
            }
        }

        // do insertion in the list
        let mut stable_pos_iter = stable_pos.iter();
        for item in stable_pos.iter() {
            match item {
                KeyChange::OldPos(rc, _) => {
                    let rel = &mut self.backend_element.borrow_mut(rc);
                    while let Some(KeyChange::NewChild(rc)) = stable_pos_iter.next() {
                        <B::GeneralElement as BackendGeneralElement>::insert(
                            rel,
                            rc,
                        );
                    }
                }
                KeyChange::NewChild(_) => continue,
            }
        }
        while let Some(KeyChange::NewChild(rc)) = stable_pos_iter.next() {
            <B::GeneralElement as BackendGeneralElement>::append(
                self.backend_element,
                rc,
            );
        }

        Ok(())
    }
}

pub struct KeyListIter<'a, C> {
    children: std::vec::IntoIter<Option<&'a C>>,
}

impl<'a, C> Iterator for KeyListIter<'a, C> {
    type Item = &'a C;

    fn next(&mut self) -> Option<Self::Item> {
        self.children.next().and_then(|x| x)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.children.size_hint()
    }
}

impl<'a, B: Backend, K: Eq + Hash, C> IntoIterator for &'a KeyList<B, K, C> {
    type Item = &'a C;
    type IntoIter = KeyListIter<'a, C>;

    fn into_iter(self) -> Self::IntoIter {
        let len = self.map.len();
        let mut arr = Vec::with_capacity(len);
        arr.resize(len, None);
        for (index, c, _) in self.map.values() {
            arr[*index] = Some(c);
        }
        KeyListIter {
            children: arr.into_iter(),
        }
    }
}
