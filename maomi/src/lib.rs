//! maomi: a rust framework for building pages with components
//! 
//! `maomi` is a framework for building (web) application user interface.
//! It has strict compile-time check and generates fast code.
//! 
//! This is the *core* module of the framework.
//! In browsers, the `maomi-dom` crate is also needed.
//! See the [`maomi_dom`](../maomi-dom) crate document for quick start.
//! 

#![warn(missing_docs)]

pub mod backend;
/// The component interface for building pages.
pub mod component;
/// The diff algorithm utilities.
pub mod diff;
/// The utilities for error handling.
pub mod error;
/// The events that can be triggered by components.
pub mod event;
/// The mount point containing the root of the page.
pub mod mount_point;
/// Helper types for node trees.
pub mod node;
/// The properties that can be received by components.
pub mod prop;
/// The utilities for template handling.
pub mod template;
/// Helper types for text nodes.
pub mod text_node;
/// The translated string types, used in i18n.
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
