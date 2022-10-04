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

// TODO add embedded content, svg, MathML support

pub mod content_sectioning;
pub use content_sectioning::*;
pub mod text_content;
pub use text_content::*;
pub mod inline_text;
pub use inline_text::*;
pub mod multimedia;
pub use multimedia::*;
pub mod demarcating_edits;
pub use demarcating_edits::*;
pub mod table_content;
pub use table_content::*;
pub mod forms;
pub use forms::*;
