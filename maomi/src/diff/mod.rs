use crate::{error::Error, backend::{tree::*, Backend}};
mod key;

/// A helper type for list changes
// Since rust GAT is not stable yet, we cannot make it a trait - use enum instead
#[derive(Debug, Clone, PartialEq)]
pub enum ListItemChange<N, T> {
    Unchanged(N, T),
    Added(N, T),
    Removed(N),
}

/// Types that can be used for list diff, a.k.a. list-diff types
pub trait ListDiff {
    // empty
    // Since rust GAT is not stable yet, we cannot find the `ListDiffAlgo` through associated types.
    // Currently we simply select the `ListDiffAlgo` manually in the macro-generated code.
}

/// A reference type for `ListDiff`
pub trait ListDiffRef<'a> {
    type Owned;

    fn to_owned_list_diff(&self) -> Self::Owned;
}

/// Types that can be converted to a list-diff type
pub trait AsListDiff<'a, L: ListDiffRef<'a>> {
    fn as_list_diff(&'a self) -> L;
}

// /// A list diff algorithm
// pub trait ListDiffAlgo<B: Backend> {
//     type ListDiff: ListDiff;
//     type ListDiffAlgoUpdate<C, D>: ListDiffAlgoUpdate<B, C, D, ListDiff = Self::ListDiff>;

//     fn list_diff_new() -> Self;

//     fn list_diff_update<'a, C, D>(
//         &'a mut self,
//         items: &mut Vec<C>,
//         new_child_fn: impl 'a + FnMut(&mut ForestNodeMut<B::GeneralElement>, &D) -> Result<C, Error>,
//         update_child_fn: impl 'a + FnMut(&mut C, &mut ForestNodeMut<B::GeneralElement>, &D) -> Result<(), Error>,
//     ) -> Self::ListDiffAlgoUpdate<C, D>;
// }

// /// An update step in the list diff algorithm
// pub trait ListDiffAlgoUpdate<B: Backend, C, D> {
//     type ListDiff;

//     fn next<'b>(
//         &mut self,
//         list_diff: impl ListDiffRef<'b, Owned = Self::ListDiff>,
//         data: &D,
//     ) -> Result<(), Error>;

//     fn end(self) -> Result<(), Error>;
// }
