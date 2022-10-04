use web_sys::{HtmlInputElement, HtmlFormElement};
use super::*;

// TODO implement two-way binding for values

#[dom_element_definition]
pub struct button {
    pub autocomplete: attribute!(&str in HtmlInputElement),
    pub disabled: attribute!(bool in HtmlInputElement),
    pub form_action: attribute!(&str in HtmlInputElement),
    pub form_enctype: attribute!(&str in HtmlInputElement),
    pub form_method: attribute!(&str in HtmlInputElement),
    pub form_no_validate: attribute!(bool in HtmlInputElement),
    pub form_target: attribute!(&str in HtmlInputElement),
    pub name: attribute!(&str in HtmlInputElement),
    pub r#type: attribute!(&str in HtmlInputElement),
    pub value: attribute!(&str in HtmlInputElement),
}

#[dom_element_definition]
pub struct datalist {}

#[dom_element_definition]
pub struct fieldset {
    pub disabled: attribute!(bool in HtmlInputElement),
    pub name: attribute!(&str in HtmlInputElement),
}

#[dom_element_definition]
pub struct form {
    pub autocomplete: attribute!(&str in HtmlFormElement),
    pub name: attribute!(&str in HtmlFormElement),
    pub rel: attribute!(&str in web_sys::HtmlAnchorElement),
    pub action: attribute!(&str in HtmlFormElement),
    pub enctype: attribute!(&str in HtmlFormElement),
    pub method: attribute!(&str in HtmlFormElement),
    pub no_validate: attribute!(bool in HtmlFormElement),
    pub target: attribute!(&str in HtmlFormElement),
    pub submit: event!(crate::event::form::Submit),
}

#[dom_element_definition]
pub struct input {
    pub accept: attribute!(&str in HtmlInputElement),
    pub alt: attribute!(&str in HtmlInputElement),
    pub autocomplete: attribute!(&str in HtmlInputElement),
    pub checked: attribute!(bool in HtmlInputElement),
    pub disabled: attribute!(bool in HtmlInputElement),
    pub form_action: attribute!(&str in HtmlInputElement),
    pub form_enctype: attribute!(&str in HtmlInputElement),
    pub form_method: attribute!(&str in HtmlInputElement),
    pub form_no_validate: attribute!(bool in HtmlInputElement),
    pub form_target: attribute!(&str in HtmlInputElement),
    pub height: attribute!(u32 in HtmlInputElement),
    pub max: attribute!(&str in HtmlInputElement),
    pub max_length: attribute!(i32 in HtmlInputElement),
    pub min: attribute!(&str in HtmlInputElement),
    pub min_length: attribute!(i32 in HtmlInputElement),
    pub multiple: attribute!(bool in HtmlInputElement),
    pub name: attribute!(&str in HtmlInputElement),
    pub pattern: attribute!(&str in HtmlInputElement),
    pub placeholder: attribute!(&str in HtmlInputElement),
    pub read_only: attribute!(bool in HtmlInputElement),
    pub required: attribute!(bool in HtmlInputElement),
    pub size: attribute!(u32 in HtmlInputElement),
    pub src: attribute!(&str in HtmlInputElement),
    pub step: attribute!(&str in HtmlInputElement),
    pub r#type: attribute!(&str in HtmlInputElement),
    pub spellcheck: attribute!(bool in web_sys::HtmlElement),
    pub value: attribute!(&str in HtmlInputElement),
    pub width: attribute!(u32 in HtmlInputElement),
    pub change: event!(crate::event::form::Change),
}

#[dom_element_definition]
pub struct label {
    pub r#for: attribute!(&str),
}

#[dom_element_definition]
pub struct legend {}

#[dom_element_definition]
pub struct meter {
    pub value: attribute!(f64 in web_sys::HtmlMeterElement),
    pub min: attribute!(f64 in web_sys::HtmlMeterElement),
    pub max: attribute!(f64 in web_sys::HtmlMeterElement),
    pub low: attribute!(f64 in web_sys::HtmlMeterElement),
    pub high: attribute!(f64 in web_sys::HtmlMeterElement),
    pub optimum: attribute!(f64 in web_sys::HtmlMeterElement),
    pub change: event!(crate::event::form::Change),
}

#[dom_element_definition]
pub struct optgroup {
    pub disabled: attribute!(bool in web_sys::HtmlOptionElement),
    pub label: attribute!(&str in web_sys::HtmlOptionElement),
}

#[dom_element_definition]
pub struct option {
    pub disabled: attribute!(bool in web_sys::HtmlOptionElement),
    pub label: attribute!(&str in web_sys::HtmlOptionElement),
    pub selected: attribute!(bool in web_sys::HtmlOptionElement),
    pub value: attribute!(&str in web_sys::HtmlOptionElement),
    pub change: event!(crate::event::form::Change),
}

#[dom_element_definition]
pub struct output {
    pub r#for: attribute!(&str),
    pub name: attribute!(&str in web_sys::HtmlInputElement),
}

#[dom_element_definition]
pub struct progress {
    pub max: attribute!(f64 in web_sys::HtmlMeterElement),
    pub value: attribute!(&str in web_sys::HtmlInputElement),
    pub change: event!(crate::event::form::Change),
}

#[dom_element_definition]
pub struct select {
    pub autocomplete: attribute!(&str in HtmlInputElement),
    pub disabled: attribute!(bool in HtmlInputElement),
    pub multiple: attribute!(bool in HtmlInputElement),
    pub name: attribute!(&str in HtmlInputElement),
    pub required: attribute!(bool in HtmlInputElement),
    pub size: attribute!(u32 in HtmlInputElement),
}

#[dom_element_definition]
pub struct textarea {
    pub autocomplete: attribute!(&str in web_sys::HtmlTextAreaElement),
    pub cols: attribute!(u32 in web_sys::HtmlTextAreaElement),
    pub disabled: attribute!(bool in web_sys::HtmlTextAreaElement),
    pub max_length: attribute!(i32 in web_sys::HtmlTextAreaElement),
    pub min_length: attribute!(i32 in web_sys::HtmlTextAreaElement),
    pub name: attribute!(&str in web_sys::HtmlTextAreaElement),
    pub placeholder: attribute!(&str in web_sys::HtmlTextAreaElement),
    pub read_only: attribute!(bool in web_sys::HtmlTextAreaElement),
    pub required: attribute!(bool in web_sys::HtmlTextAreaElement),
    pub rows: attribute!(u32 in web_sys::HtmlTextAreaElement),
    pub spellcheck: attribute!(bool in web_sys::HtmlElement),
    pub wrap: attribute!(&str in web_sys::HtmlTextAreaElement),
}
