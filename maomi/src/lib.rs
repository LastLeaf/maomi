#[allow(unused_imports)] #[macro_use] extern crate log;

use wasm_bindgen::prelude::*;

pub mod prelude;
pub mod component;
pub use component::{Component, PrerenderableComponent, ComponentTemplate, ComponentTemplateOperation, ComponentContext, ComponentRc, ComponentWeak, ComponentRef, ComponentRefMut, EmptyComponent};
pub mod property;
pub use property::{Property, Prop};
pub mod event;
pub use event::{Event, Ev};
pub mod node;
pub mod context;
pub use context::{Context};
#[macro_use] pub mod global_events;
pub mod backend;
pub mod virtual_key;
mod escape;
mod prerender;

#[wasm_bindgen(start)]
pub fn framework_init() {
    console_error_panic_hook::set_once();
}
