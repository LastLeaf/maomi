//! maomi: a rust framework for building pages with components
//! 
//! `maomi` is a framework for building (web) application user interface.
//! It has strict compile-time check and generates fast code.
//! 
//! This is the *core* module of the framework.
//! In browsers, the `maomi-dom` crate is also needed.
//! See the [`maomi_dom`](../maomi_dom) crate document for the quick start.

#![warn(missing_docs)]

pub mod backend;
pub mod component;
pub mod diff;
pub mod error;
pub mod event;
pub mod mount_point;
pub mod node;
pub mod prop;
pub mod template;
pub mod text_node;
pub mod locale_string;
#[cfg(any(feature = "prerendering", feature = "prerendering-apply"))]
pub use backend::context::PrerenderingData;
pub use backend::context::{AsyncCallback, BackendContext};

/// The types that should usually be imported.
/// 
/// Usually, `use maomi::prelude::*;` should be added in component files for convinience.
pub mod prelude {
    #[cfg(any(feature = "prerendering", feature = "prerendering-apply"))]
    pub use super::component::PrerenderableComponent;
    pub use super::component::{Component, ComponentExt, ComponentRc};
    pub use super::diff::key::AsListKey;
    pub use super::event::Event;
    pub use super::prop::Prop;
    pub use async_trait::async_trait;
    pub use maomi_macro::*;
}
