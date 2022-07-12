pub mod backend;
pub mod component;
pub mod error;
pub mod text_node;
pub mod mount_point;

pub mod prelude {
    pub use super::component::{Component, ComponentExt, TemplateHelper};
    pub use maomi_macro::*;
}
