//! The element definition
//!
//! The element list is found in [MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element) .

use maomi::{
    backend::{BackendComponent, SupportBackend},
    error::Error,
    node::{OwnerWeak, SlotChange},
    BackendContext,
};
use wasm_bindgen::JsCast;

use crate::{
    base_element::*,
    class_list::DomClassList,
    event,
    event::DomEvent,
    tree::*,
    DomBackend,
    DomGeneralElement,
    DomState,
};

// FIXME add common elements and attributes

pub mod content_sectioning;
pub use content_sectioning::*;
pub mod inline_text;
pub use inline_text::*;
