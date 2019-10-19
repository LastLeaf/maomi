#[macro_use] extern crate log;

use wasm_bindgen::prelude::*;

pub mod prelude;
pub mod component;
pub use component::{Component, ComponentTemplate};
pub mod property;
pub use property::{Property, Prop};
pub mod node;
pub mod context;
pub use context::{Context};
pub mod backend;
pub mod virtual_key;

#[wasm_bindgen(start)]
pub fn framework_init() {
    console_error_panic_hook::set_once();
}
