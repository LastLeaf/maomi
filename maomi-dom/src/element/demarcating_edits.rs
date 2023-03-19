//! The DOM elements about demarcating edits.

use super::*;

#[dom_element_definition]
pub struct del {
    pub cite: attribute!(&str in web_sys::HtmlQuoteElement),
    pub date_time: attribute!(&str in web_sys::HtmlTimeElement),
}

#[dom_element_definition]
pub struct ins {
    pub cite: attribute!(&str in web_sys::HtmlQuoteElement),
    pub date_time: attribute!(&str in web_sys::HtmlTimeElement),
}
