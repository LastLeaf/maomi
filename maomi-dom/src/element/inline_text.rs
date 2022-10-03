use web_sys::HtmlAnchorElement;

use super::*;

#[dom_element_definition]
pub struct a {
    pub download: attribute!(&str in HtmlAnchorElement),
    pub href: attribute!(&str in HtmlAnchorElement),
    pub hreflang: attribute!(&str in HtmlAnchorElement),
    pub ping: attribute!(&str in HtmlAnchorElement),
    pub referrer_policy: attribute!(&str in HtmlAnchorElement),
    pub rel: attribute!(&str in HtmlAnchorElement),
    pub target: attribute!(&str in HtmlAnchorElement),
    pub r#type: attribute!(&str in HtmlAnchorElement),
}

#[dom_element_definition]
pub struct abbr {}

#[dom_element_definition]
pub struct b {}

#[dom_element_definition]
pub struct bdi {}

#[dom_element_definition]
pub struct bdo {}

#[dom_element_definition]
pub struct br {}

#[dom_element_definition]
pub struct site {}

#[dom_element_definition]
pub struct code {}

#[dom_element_definition]
pub struct data {
    pub value: attribute!(&str in web_sys::HtmlDataElement),
}

#[dom_element_definition]
pub struct em {}

#[dom_element_definition]
pub struct i {}

#[dom_element_definition]
pub struct kbd {}

#[dom_element_definition]
pub struct mark {}

#[dom_element_definition]
pub struct q {
    pub cite: attribute!(&str in web_sys::HtmlQuoteElement),
}

#[dom_element_definition]
pub struct rp {}

#[dom_element_definition]
pub struct rt {}

#[dom_element_definition]
pub struct ruby {}

#[dom_element_definition]
pub struct s {}

#[dom_element_definition]
pub struct samp {}

#[dom_element_definition]
pub struct small {}

#[dom_element_definition]
pub struct span {}

#[dom_element_definition]
pub struct strong {}

#[dom_element_definition]
pub struct sub {}

#[dom_element_definition]
pub struct sup {}

#[dom_element_definition]
pub struct time {
    pub date_time: attribute!(&str in web_sys::HtmlTimeElement),
}

#[dom_element_definition]
pub struct u {}

#[dom_element_definition]
pub struct var {}

#[dom_element_definition]
pub struct wbr {}
