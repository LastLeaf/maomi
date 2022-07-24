pub mod backend;
pub mod component;
pub mod diff;
pub mod error;
pub mod mount_point;
pub mod node;
pub mod text_node;
pub use backend::context::BackendContext;

pub mod prelude {
    pub use super::component::{Component, ComponentExt};
    pub use maomi_macro::*;
}
