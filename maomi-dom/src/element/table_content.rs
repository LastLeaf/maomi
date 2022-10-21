//! The DOM elements about table contents.

use super::*;

#[dom_element_definition]
pub struct caption {}

#[dom_element_definition]
pub struct col {
    pub span: attribute!(u32 in web_sys::HtmlTableColElement),
}

#[dom_element_definition]
pub struct colgroup {
    pub span: attribute!(u32 in web_sys::HtmlTableColElement),
}

#[dom_element_definition]
pub struct table {}

#[dom_element_definition]
pub struct tbody {}

#[dom_element_definition]
pub struct td {
    pub col_span: attribute!(u32 in web_sys::HtmlTableCellElement),
    pub row_span: attribute!(u32 in web_sys::HtmlTableCellElement),
    pub headers: attribute!(&str in web_sys::HtmlTableCellElement),
}

#[dom_element_definition]
pub struct tfoot {}

#[dom_element_definition]
pub struct th {
    pub col_span: attribute!(u32 in web_sys::HtmlTableCellElement),
    pub row_span: attribute!(u32 in web_sys::HtmlTableCellElement),
    pub headers: attribute!(&str in web_sys::HtmlTableCellElement),
}

#[dom_element_definition]
pub struct thead {}

#[dom_element_definition]
pub struct tr {}
