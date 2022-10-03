use maomi_dom_macro::dom_element_definition;
use web_sys::HtmlAnchorElement;

use super::*;

#[dom_element_definition]
pub struct a {
    pub href: attribute!(&str in HtmlAnchorElement),
}

// TODO

#[dom_element_definition]
pub struct div {}

#[dom_element_definition]
pub struct span {}

#[dom_element_definition]
pub struct button {
    pub r#type: attribute!(&str in HtmlAnchorElement),
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
