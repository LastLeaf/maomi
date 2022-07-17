
/// A helper type for list changes
// Since rust GAT is not stable yet, we cannot make it a trait - use enum instead
#[derive(Debug, Clone, PartialEq)]
pub enum ListItemChange<N, T> {
    Unchanged(N, T),
    Added(N, T),
    Removed(N, T),
}
