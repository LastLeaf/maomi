use super::*;

fn set_a_download(elem: &web_sys::HtmlElement, s: &str) {
    web_sys::HtmlAnchorElement::set_download(elem.unchecked_ref(), s)
}

fn set_a_href(elem: &web_sys::HtmlElement, s: &str) {
    web_sys::HtmlAnchorElement::set_href(elem.unchecked_ref(), s)
}

define_element_with_shared_props!(a, {
    download: DomStrAttr: set_a_download,
    href: DomStrAttr: set_a_href,
}, {});

define_element_with_shared_props!(abbr, {}, {});
