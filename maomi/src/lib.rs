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
#[cfg(any(feature = "prerendering", feature = "prerendering-apply"))]
pub use backend::context::PrerenderingData;
pub use backend::context::{AsyncCallback, BackendContext};

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
