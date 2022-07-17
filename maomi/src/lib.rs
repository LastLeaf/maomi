pub mod error;
pub mod backend;
pub mod text_node;
pub mod component;
pub mod diff;
pub mod node;
pub mod mount_point;
pub use backend::context::BackendContext;

pub mod prelude {
    pub use super::component::{Component, ComponentExt, TemplateHelper};
    pub use maomi_macro::*;
}
