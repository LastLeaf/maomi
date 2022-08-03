pub mod backend;
pub mod component;
pub mod template;
pub mod diff;
pub mod error;
pub mod mount_point;
pub mod node;
pub mod prop;
pub mod text_node;
pub use backend::context::{BackendContext, AsyncCallback};

pub mod prelude {
    pub use super::component::{Component, ComponentExt};
    pub use super::prop::Prop;
    pub use super::diff::key::AsListKey;
    pub use maomi_macro::*;
}
