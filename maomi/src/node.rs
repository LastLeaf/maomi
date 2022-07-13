
/// A helper type to represent a node with child nodes
#[derive(Debug, Clone, PartialEq)]
pub struct Node<N, C> {
    pub node: N,
    pub child_nodes: C,
}
