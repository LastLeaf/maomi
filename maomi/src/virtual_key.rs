use std::ops::Range;

use super::node::*;
use super::backend::Backend;

/// VirtualNode key management
/// **Should be done through template engine!**
#[doc(hidden)]
pub struct VirtualKeyList<T: PartialEq> {
    keys: Vec<Option<T>>,
}

impl<T: PartialEq> VirtualKeyList<T> {
    /// Create a new key list
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub fn new(keys: Vec<Option<T>>) -> Self {
        Self {
            keys: keys
        }
    }

    /// Get the count of keys
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Reorder keys
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub fn list_reorder<B: Backend>(&self, old: &Self, node: &mut VirtualNode<B>) -> VirtualKeyChanges<B> {
        let old_keys = &old.keys;
        let new_keys = &self.keys;
        let mut old_i = 0;
        let mut new_i = 0;
        let old_len = old_keys.len();
        let new_len = new_keys.len();
        let mut index_map: Vec<Option<usize>> = (0..new_len).map(|_| None).collect();
        let mut removes: Vec<(usize, Box<[bool]>)> = vec![];
        let mut inserts: Vec<Range<usize>> = vec![];
        let mut remove_and_insert = |old_i, old_i_end, new_i, new_i_end| {
            if old_i < old_i_end {
                let reusable: Box<[bool]> = (old_i..old_i_end).map(|_| false).collect();
                removes.push((old_i, reusable));
            }
            if new_i < new_i_end {
                inserts.push(new_i..new_i_end);
            }
        };
        // find a long common sub sequence
        loop {
            if old_i == old_len || new_i == new_len {
                remove_and_insert(old_i, old_len, new_i, new_len);
                break
            }
            if old_keys[old_i] == new_keys[new_i] {
                index_map[new_i] = Some(old_i);
                old_i += 1;
                new_i += 1;
            } else {
                let mut c = 1;
                let mut d = 0;
                loop {
                    if old_i + d < old_len && new_i + c < new_len && old_keys[old_i + d] == new_keys[new_i + c] {
                        remove_and_insert(old_i, old_i + d, new_i, new_i + c);
                        old_i += d;
                        new_i += c;
                        break
                    }
                    if old_i + c < old_len && new_i + d < new_len && old_keys[old_i + c] == new_keys[new_i + d] {
                        remove_and_insert(old_i, old_i + c, new_i, new_i + d);
                        old_i += c;
                        new_i += d;
                        break
                    }
                    if old_i + c >= old_len && new_i + c >= new_len {
                        d += 1;
                        if d == c {
                            remove_and_insert(old_i, old_len, new_i, new_len);
                            break
                        }
                        c = d
                    }
                    c += 1;
                }
            }
        }
        // try to reuse removed items
        for new_range in inserts.iter() {
            for new_i in new_range.clone() {
                for (start, reusable) in removes.iter_mut() {
                    for i in 0..reusable.len() {
                        let old_i = i + *start;
                        if old_keys[old_i] == new_keys[new_i] {
                            index_map[new_i] = Some(old_i);
                            reusable[i] = true;
                        }
                    }
                }
            }
        }
        VirtualKeyChanges {
            removes,
            inserts,
            nodes: index_map.into_iter().map(|x| {
                match x {
                    None => None,
                    Some(x) => Some(node.children_rc()[x].clone())
                }
            }).collect()
        }
    }
}

/// Diff between two key lists
/// **Should be done through template engine!**
#[doc(hidden)]
pub struct VirtualKeyChanges<B: Backend> {
    removes: Vec<(usize, Box<[bool]>)>,
    inserts: Vec<Range<usize>>,
    nodes: Vec<Option<NodeRc<B>>>,
}

impl<B: Backend> VirtualKeyChanges<B> {
    /// Generate an empty diff
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub fn new_empty(len: usize) -> Self {
        Self {
            removes: Vec::with_capacity(0),
            inserts: vec![0..len],
            nodes: (0..len).map(|_| None).collect(),
        }
    }

    /// Get the nodes
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub fn nodes_mut(&mut self) -> &mut Vec<Option<NodeRc<B>>> {
        &mut self.nodes
    }

    /// Apply it
    /// **Should be done through template engine!**
    #[doc(hidden)]
    pub unsafe fn apply(self, node: &mut VirtualNode<B>, children: Vec<NodeRc<B>>) {
        let Self {inserts, removes, nodes: _} = self;
        let mut d = 0;
        for (start, reusable) in removes {
            node.remove_with_reuse(start - d, &reusable);
            d += reusable.len();
        }
        for new_range in inserts {
            node.insert_list(new_range.start, children[new_range].to_vec());
        }
    }
}
