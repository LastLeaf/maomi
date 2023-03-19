//! The diff algorithm utilities.
//! 
//! When applying list update,
//! the framework tries to figure out which items should be added, removed, or moved.
//! For example, in the following component:
//! 
//! ```rust
//! use maomi::prelude::*;
//! 
//! #[component]
//! struct MyComponent {
//!     template: template! {
//!         for item in self.list.iter() {
//!             /* ... */
//!         }
//!     },
//!     list: Vec<usize>,
//! }
//! ```
//! 
//! This requires an algorithm to compare the current `list` and the `list` that used to do previous update,
//! and decide which items should be added, removed, or moved.
//! 
//! By default, the [keyless](./keyless) algorithm is used.
//! This algorithm compares items one by one,
//! and adds or removes items at the ends of the list.
//! For example:
//! * if the `list` in the previous update is `[30, 40, 50]` while the current `list` is `[30, 40, 50, 60]` ,
//!   then the forth item with item data `60` is added;
//! * if the `list` in the previous update is `[30, 40, 50]` while the current `list` is `[30, 50]` ,
//!   then the second item is updated with item data `50` , and the third item is removed.
//! This algorithm is very fast if the items at the start and the middle of the list will not be removed or inserted,
//! but it is pretty slow if that happens.
//! 
//! For lists that often randomly changes, the [key](./key) algorithm is a better option.
//! To use this algorithm, the `AsListKey` trait must be implemented for the item data,
//! and the `use` instruction should be added in the template `for` expression.
//! The example code above should be changed:
//! 
//! ```rust
//! use maomi::prelude::*;
//! 
//! struct ListData {
//!     id: usize,
//! }
//! 
//! impl AsListKey for ListData {
//!     type ListKey = usize;
//! 
//!     fn as_list_key(&self) -> &usize {
//!         &self.id
//!     }
//! }
//! 
//! #[component]
//! struct MyComponent {
//!     template: template! {
//!         // add a `use` list key
//!         for item in self.list.iter() use usize {
//!             /* ... */
//!         }
//!     },
//!     list: Vec<ListData>,
//! }
//! ```
//! 
//! The `ListKey` is used for list comparison.
//! * if the `ListKey` list in the previous update is `[30, 40, 50]` while the current is `[30, 50]` ,
//!   then the second item is removed.
//! * if the `ListKey` list in the previous update is `[30, 40, 50]` while the current is `[30, 40, 60, 50]` ,
//!   then the third item with item data 60 is inserted;
//! This algorithm has a balanced performance on lists that dynamically changes,
//! while it has a small overhead no matter the list is changed or not.

use crate::{
    backend::{tree::*, Backend},
    error::Error,
};
pub mod key;
pub mod keyless;

#[doc(hidden)]
pub enum ListAlgo<N, U> {
    New(N),
    Update(U),
}

impl<N, U> ListAlgo<N, U> {
    #[doc(hidden)]
    #[inline]
    pub fn as_new(&mut self) -> &mut N {
        match self {
            Self::New(x) => x,
            Self::Update(_) => panic!("illegal list update"),
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn as_update(&mut self) -> &mut U {
        match self {
            Self::Update(x) => x,
            Self::New(_) => panic!("illegal list update"),
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn into_new(self) -> N {
        match self {
            Self::New(x) => x,
            Self::Update(_) => panic!("illegal list update"),
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn into_update(self) -> U {
        match self {
            Self::Update(x) => x,
            Self::New(_) => panic!("illegal list update"),
        }
    }
}
