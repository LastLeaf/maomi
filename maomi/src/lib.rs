pub mod backend;
pub mod component;
pub mod diff;
pub mod error;
pub mod mount_point;
pub mod node;
pub mod prop;
pub mod template;
pub mod text_node;
pub use backend::context::{AsyncCallback, BackendContext};

pub mod prelude {
    pub use super::component::{Component, ComponentExt};
    pub use super::diff::key::AsListKey;
    pub use super::prop::Prop;
    pub use maomi_macro::*;
}
