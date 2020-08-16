use super::*;
use crate::backend::*;

/// The range of traversal (in shadow tree or composed tree)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TraversalRange {
    /// Traverse in shadow tree
    Shadow,
    /// Traverse in composed tree
    Composed,
}

/// The item order while iteration (parent to child or child to parent)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TraversalOrder {
    /// Returns parent before its child
    ParentFirst,
    /// Returns parent after its child
    ParentLast,
}

pub struct SingleNodeIter<'b, B: Backend, N: 'b> {
    source: Node<'b, B>,
    node: Option<NodeRc<B>>,
    node_ref: Option<N>,
}

impl<'b, B: Backend, N: 'b> SingleNodeIter<'b, B, N> {
    pub(super) fn parent(source: Node<'b, B>, range: TraversalRange) -> SingleNodeIter<'b, B, Node<'b, B>> {
        let node = match range {
            TraversalRange::Shadow => source.parent_rc(),
            TraversalRange::Composed => source.composed_parent_rc(),
        };
        SingleNodeIter {
            source,
            node_ref: node.as_ref().map(|x| unsafe { x.deref_unsafe_with_lifetime() }),
            node,
        }
    }

    pub(super) fn owner(source: Node<'b, B>) -> SingleNodeIter<'b, B, &'b ComponentNode<B>> {
        let node = source.owner_rc();
        SingleNodeIter {
            source,
            node_ref: node.as_ref().map(|x| unsafe { x.deref_unsafe_with_lifetime() }),
            node: node.map(|x| x.into()),
        }
    }
}

impl<'b, B: Backend, N: 'b> Iterator for SingleNodeIter<'b, B, N> {
    type Item = N;
    fn next(&mut self) -> Option<Self::Item> {
        self.node_ref.take()
    }
}

pub struct SingleNodeIterMut<'b, B: Backend, N: 'b> {
    source: NodeMut<'b, B>,
    node: Option<NodeRc<B>>,
    node_ref: Option<N>,
}

impl<'b, B: Backend, N: 'b> SingleNodeIterMut<'b, B, N> {
    pub(super) fn parent(source: NodeMut<'b, B>, range: TraversalRange) -> SingleNodeIterMut<'b, B, NodeMut<'b, B>> {
        let mut node = match range {
            TraversalRange::Shadow => source.as_ref().parent_rc(),
            TraversalRange::Composed => source.as_ref().composed_parent_rc(),
        };
        SingleNodeIterMut {
            source,
            node_ref: node.as_mut().map(|x| unsafe { x.deref_mut_unsafe_with_lifetime() }),
            node,
        }
    }

    pub(super) fn owner(source: NodeMut<'b, B>) -> SingleNodeIterMut<'b, B, &'b mut ComponentNode<B>> {
        let mut node = source.as_ref().owner_rc();
        SingleNodeIterMut {
            source,
            node_ref: node.as_mut().map(|x| unsafe { x.deref_mut_unsafe_with_lifetime() }),
            node: node.map(|x| x.into()),
        }
    }
}

impl<'b, B: Backend, N: 'b> Iterator for SingleNodeIterMut<'b, B, N> {
    type Item = N;
    fn next(&mut self) -> Option<Self::Item> {
        self.node_ref.take()
    }
}

pub struct AncestorIter<'b, B: Backend> {
    source: Node<'b, B>,
    nodes: Vec<NodeRc<B>>,
    index: usize,
    range: TraversalRange,
}

impl<'b, B: Backend> AncestorIter<'b, B> {
    pub(super) fn new(source: Node<'b, B>, range: TraversalRange, order: TraversalOrder) -> Self {
        let mut nodes = vec![];
        let mut cur: Node<B> = Node::clone(&source);
        let mut cur_node;
        loop {
            let next = match range {
                TraversalRange::Shadow => cur.parent_rc(),
                TraversalRange::Composed => cur.composed_parent_rc(),
            };
            match next {
                Some(n) => {
                    cur_node = n.clone();
                    nodes.push(n);
                    cur = unsafe { cur_node.deref_unsafe() };
                },
                None => break
            }
        }
        if order == TraversalOrder::ParentFirst {
            nodes.reverse();
        }
        Self {
            source,
            nodes,
            index: 0,
            range,
        }
    }
}

impl<'b, B: Backend> Iterator for AncestorIter<'b, B> {
    type Item = Node<'b, B>;
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        let ret = if let Some(n) = self.nodes.get(index) {
            self.index += 1;
            Some(unsafe { n.deref_unsafe_with_lifetime() })
        } else {
            None
        };
        return ret
    }
}

pub struct AncestorIterMut<'b, B: Backend> {
    source: NodeMut<'b, B>,
    nodes: Vec<NodeRc<B>>,
    index: usize,
    range: TraversalRange,
}

impl<'b, B: Backend> AncestorIterMut<'b, B> {
    pub(super) fn new(mut source: NodeMut<'b, B>, range: TraversalRange, order: TraversalOrder) -> Self {
        let mut nodes = vec![];
        {
            let mut cur = source.as_mut();
            let mut cur = cur.as_mut();
            let mut cur_node;
            loop {
                let next = match range {
                    TraversalRange::Shadow => cur.as_ref().parent_rc(),
                    TraversalRange::Composed => cur.as_ref().composed_parent_rc(),
                };
                match next {
                    Some(n) => {
                        cur_node = n.clone();
                        nodes.push(n);
                        cur = unsafe { cur_node.deref_mut_unsafe() };
                    },
                    None => break
                }
            }
        }
        if order == TraversalOrder::ParentFirst {
            nodes.reverse();
        }
        Self {
            source,
            nodes,
            index: 0,
            range,
        }
    }
}

impl<'b, 'c, B: Backend> MutIterator<'c> for AncestorIterMut<'b, B> {
    type Item = NodeMut<'c, B>;
    fn next(&'c mut self) -> Option<Self::Item> {
        let index = self.index;
        let ret = if let Some(n) = self.nodes.get(index) {
            self.index += 1;
            Some(unsafe { n.deref_mut_unsafe_with_lifetime() })
        } else {
            None
        };
        return ret
    }
}

pub struct ChildIter<'b, B: Backend> {
    source: Node<'b, B>,
    index: usize,
    nodes: Vec<NodeRc<B>>,
}

impl<'b, B: Backend> ChildIter<'b, B> {
    pub(super) fn new(source: Node<'b, B>, range: TraversalRange) -> Self {
        let nodes = match range {
            TraversalRange::Shadow => source.children_rc().into_owned(),
            TraversalRange::Composed => source.composed_children_rc().into_owned(),
        };
        Self {
            source,
            index: 0,
            nodes,
        }
    }
}

impl<'b, B: Backend> Iterator for ChildIter<'b, B> {
    type Item = Node<'b, B>;
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        if let Some(n) = self.nodes.get(index) {
            self.index += 1;
            Some(unsafe { n.deref_unsafe_with_lifetime() })
        } else {
            None
        }
    }
}

pub struct ChildIterMut<'b, B: Backend> {
    source: NodeMut<'b, B>,
    index: usize,
    nodes: Vec<NodeRc<B>>,
}

impl<'b, B: Backend> ChildIterMut<'b, B> {
    pub(super) fn new(source: NodeMut<'b, B>, range: TraversalRange) -> Self {
        let nodes = match range {
            TraversalRange::Shadow => source.as_ref().children_rc().into_owned(),
            TraversalRange::Composed => source.as_ref().composed_children_rc().into_owned(),
        };
        Self {
            source,
            index: 0,
            nodes,
        }
    }
}

impl<'b, 'c, B: Backend> MutIterator<'c> for ChildIterMut<'b, B> {
    type Item = NodeMut<'c, B>;
    fn next(&'c mut self) -> Option<Self::Item> {
        let index = self.index;
        if let Some(n) = self.nodes.get(index) {
            self.index += 1;
            Some(unsafe { n.deref_mut_unsafe_with_lifetime() })
        } else {
            None
        }
    }
}

pub struct DfsIter<'b, B: Backend> {
    source: Node<'b, B>,
    cur: Vec<(Vec<NodeRc<B>>, usize)>,
    range: TraversalRange,
    order: TraversalOrder,
}

impl<'b, B: Backend> DfsIter<'b, B> {
    pub(super) fn new(source: Node<'b, B>, range: TraversalRange, order: TraversalOrder) -> Self {
        let nodes = match range {
            TraversalRange::Shadow => source.children_rc().into_owned(),
            TraversalRange::Composed => source.composed_children_rc().into_owned(),
        };
        Self {
            source,
            cur: vec![(nodes, 0)],
            range,
            order,
        }
    }
}

impl<'b, B: Backend> Iterator for DfsIter<'b, B> {
    type Item = Node<'b, B>;
    fn next(&mut self) -> Option<Self::Item> {
        let Self { ref mut cur, range, order, .. } = self;
        let ret = {
            if let Some((nodes, index)) = cur.pop() {
                match nodes.get(index).cloned() {
                    Some(x) => {
                        cur.push((nodes, index));
                        let nodes = {
                            let x = unsafe { x.deref_unsafe_with_lifetime() };
                            match range {
                                TraversalRange::Shadow => x.children_rc().into_owned(),
                                TraversalRange::Composed => x.composed_children_rc().into_owned(),
                            }
                        };
                        cur.push((nodes, 0));
                        match order {
                            TraversalOrder::ParentFirst => x,
                            TraversalOrder::ParentLast => return self.next(),
                        }
                    },
                    None => {
                        if let Some((x, ref mut index)) = cur.last_mut() {
                            let i = *index;
                            *index += 1;
                            match order {
                                TraversalOrder::ParentFirst => return self.next(),
                                TraversalOrder::ParentLast => x.get(i).unwrap().clone(),
                            }
                        } else {
                            return None;
                        }
                    },
                }
            } else {
                return None;
            }
        };
        Some(unsafe { ret.deref_unsafe_with_lifetime() })
    }
}

pub struct DfsIterMut<'b, B: Backend> {
    source: NodeMut<'b, B>,
    cur: Vec<(Vec<NodeRc<B>>, usize)>,
    range: TraversalRange,
    order: TraversalOrder,
}

impl<'b, B: Backend> DfsIterMut<'b, B> {
    pub(super) fn new(source: NodeMut<'b, B>, range: TraversalRange, order: TraversalOrder) -> Self {
        let nodes = match range {
            TraversalRange::Shadow => source.as_ref().children_rc().into_owned(),
            TraversalRange::Composed => source.as_ref().composed_children_rc().into_owned(),
        };
        Self {
            source,
            cur: vec![(nodes, 0)],
            range,
            order,
        }
    }
}

impl<'b, 'c, B: Backend> MutIterator<'c> for DfsIterMut<'b, B> {
    type Item = NodeMut<'c, B>;
    fn next(&'c mut self) -> Option<Self::Item> {
        let Self { ref mut cur, range, order, .. } = self;
        let ret = {
            if let Some((nodes, index)) = cur.pop() {
                match nodes.get(index).cloned() {
                    Some(x) => {
                        cur.push((nodes, index));
                        let nodes = {
                            let x = unsafe { x.deref_unsafe_with_lifetime() };
                            match range {
                                TraversalRange::Shadow => x.children_rc().into_owned(),
                                TraversalRange::Composed => x.composed_children_rc().into_owned(),
                            }
                        };
                        cur.push((nodes, 0));
                        match order {
                            TraversalOrder::ParentFirst => x,
                            TraversalOrder::ParentLast => return self.next(),
                        }
                    },
                    None => {
                        cur.pop();
                        if let Some((x, ref mut index)) = cur.last_mut() {
                            let i = *index;
                            *index += 1;
                            match order {
                                TraversalOrder::ParentFirst => return self.next(),
                                TraversalOrder::ParentLast => x.get(i).unwrap().clone(),
                            }
                        } else {
                            return None;
                        }
                    },
                }
            } else {
                return None;
            }
        };
        Some(unsafe { ret.deref_mut_unsafe_with_lifetime() })
    }
}
