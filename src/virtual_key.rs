use std::ops::Range;

use super::node::*;
use super::backend::Backend;

pub struct VirtualKeyList<T: PartialEq> {
    keys: Vec<Option<T>>,
}

impl<T: PartialEq> VirtualKeyList<T> {
    pub fn new(keys: Vec<Option<T>>) -> Self {
        Self {
            keys: keys
        }
    }

    pub fn list_reorder<B: Backend>(&self, old: &Self, node: &mut VirtualNodeRefMut<B>) -> VirtualKeyChanges<B> {
        let old_keys = &old.keys;
        let new_keys = &self.keys;
        let mut old_i = 0;
        let mut new_i = 0;
        let old_len = old_keys.len();
        let new_len = new_keys.len();
        let mut index_map: Vec<Option<usize>> = (0..new_len).map(|_| None).collect();
        let mut removes: Vec<Range<usize>> = vec![];
        let mut inserts: Vec<Range<usize>> = vec![];
        let mut remove_and_insert = |old_i, old_len, new_i, new_len| {
            if old_i < old_len {
                removes.push(old_i..old_len);
            }
            if new_i < new_len {
                inserts.push(new_i..new_len);
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
                for old_range in removes.iter() {
                    for old_i in old_range.clone() {
                        if old_keys[old_i] == new_keys[new_i] {
                            index_map[new_i] = Some(old_i);
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
                    Some(x) => Some(node.children()[x].clone())
                }
            }).collect()
        }
    }
}

pub struct VirtualKeyChanges<B: Backend> {
    removes: Vec<Range<usize>>,
    inserts: Vec<Range<usize>>,
    nodes: Vec<Option<NodeRc<B>>>,
}

impl<B: Backend> VirtualKeyChanges<B> {
    pub fn new_empty(len: usize) -> Self {
        Self {
            removes: Vec::with_capacity(0),
            inserts: vec![0..len],
            nodes: (0..len).map(|_| None).collect(),
        }
    }
    pub fn nodes_mut(&mut self) -> &mut Vec<Option<NodeRc<B>>> {
        &mut self.nodes
    }
    pub fn apply(self, node: &mut VirtualNodeRefMut<B>) {
        let Self {inserts, removes, mut nodes} = self;
        let mut d = 0;
        for old_range in removes {
            node.remove_range((old_range.start - d)..(old_range.end - d));
            d += old_range.len();
        }
        for new_range in inserts {
            node.insert_list(new_range.start, nodes[new_range].iter_mut().map(|x| x.take().unwrap()).collect());
        }
    }
}
