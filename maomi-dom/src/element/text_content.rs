//! The DOM elements about text content.

use super::*;

#[dom_element_definition]
pub struct blockquote {
    pub cite: attribute!(&str in web_sys::HtmlQuoteElement),
}

#[dom_element_definition]
pub struct dd {}

#[dom_element_definition]
pub struct div {}

#[dom_element_definition]
pub struct dl {}

#[dom_element_definition]
pub struct dt {}

#[dom_element_definition]
pub struct figcaption {}

#[dom_element_definition]
pub struct figure {}

#[dom_element_definition]
pub struct hr {}

#[dom_element_definition]
pub struct li {
    pub value: attribute!(&str in web_sys::HtmlDataElement),
}

#[dom_element_definition]
pub struct menu {}

#[dom_element_definition]
pub struct ol {}

#[dom_element_definition]
pub struct p {}

#[dom_element_definition]
pub struct pre {}

#[dom_element_definition]
pub struct ul {}
