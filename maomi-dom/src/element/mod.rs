//! The element definition
//!
//! The element list is found in [MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element) .

use maomi::{
    backend::{BackendComponent, SupportBackend},
    error::Error,
    node::{OwnerWeak, SlotChange},
    BackendContext,
};
use maomi_dom_macro::dom_element_definition;
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
pub mod multimedia;
pub use multimedia::*;
pub mod embedded_content;
pub use embedded_content::*;
pub mod demarcating_edits;
pub use demarcating_edits::*;
pub mod table_content;
pub use table_content::*;
pub mod forms;
pub use forms::*;

// TODO

#[dom_element_definition]
pub struct div {}

#[dom_element_definition]
pub struct button {
    pub r#type: attribute!(&str in web_sys::HtmlAnchorElement),
}

#[dom_element_definition]
pub struct table {}

#[dom_element_definition]
pub struct thead {}

#[dom_element_definition]
pub struct tbody {}

#[dom_element_definition]
pub struct tfoot {}

#[dom_element_definition]
pub struct th {}

#[dom_element_definition]
pub struct tr {}

#[dom_element_definition]
pub struct td {}
