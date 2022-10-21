//! The DOM elements about multimedia.

use super::*;

#[dom_element_definition]
pub struct canvas {
    pub width: attribute!(u32 in web_sys::HtmlInputElement),
    pub height: attribute!(u32 in web_sys::HtmlInputElement),
}

#[dom_element_definition]
pub struct img {
    pub alt: attribute!(&str in web_sys::HtmlImageElement),
    pub decoding: attribute!(&str in web_sys::HtmlImageElement),
    pub height: attribute!(u32 in web_sys::HtmlImageElement),
    pub is_map: attribute!(bool in web_sys::HtmlImageElement),
    pub referrer_policy: attribute!(&str in web_sys::HtmlImageElement),
    pub sizes: attribute!(&str in web_sys::HtmlImageElement),
    pub src: attribute!(&str in web_sys::HtmlImageElement),
    pub srcset: attribute!(&str in web_sys::HtmlImageElement),
    pub width: attribute!(u32 in web_sys::HtmlImageElement),
    pub use_map: attribute!(&str in web_sys::HtmlImageElement),
}

#[dom_element_definition]
pub struct audio {
    pub autoplay: attribute!(bool in web_sys::HtmlMediaElement),
    pub controls: attribute!(bool in web_sys::HtmlMediaElement),
    pub r#loop: attribute!(bool in web_sys::HtmlMediaElement),
    pub muted: attribute!(bool in web_sys::HtmlMediaElement),
    pub preload: attribute!(&str in web_sys::HtmlMediaElement),
    pub src: attribute!(&str in web_sys::HtmlMediaElement),
}

#[dom_element_definition]
pub struct video {
    pub autoplay: attribute!(bool in web_sys::HtmlMediaElement),
    pub controls: attribute!(bool in web_sys::HtmlMediaElement),
    pub height: attribute!(u32 in web_sys::HtmlVideoElement),
    pub r#loop: attribute!(bool in web_sys::HtmlMediaElement),
    pub muted: attribute!(bool in web_sys::HtmlMediaElement),
    pub poster: attribute!(&str in web_sys::HtmlVideoElement),
    pub preload: attribute!(&str in web_sys::HtmlMediaElement),
    pub src: attribute!(&str in web_sys::HtmlMediaElement),
    pub width: attribute!(u32 in web_sys::HtmlVideoElement),
}

#[dom_element_definition]
pub struct track {
    pub default: attribute!(bool in web_sys::HtmlTrackElement),
    pub kind: attribute!(&str in web_sys::HtmlTrackElement),
    pub label: attribute!(&str in web_sys::HtmlTrackElement),
    pub src: attribute!(&str in web_sys::HtmlTrackElement),
    pub srclang: attribute!(&str in web_sys::HtmlTrackElement),
}

#[dom_element_definition]
pub struct map {
    pub name: attribute!(&str in web_sys::HtmlInputElement),
}

#[dom_element_definition]
pub struct area {
    pub name: attribute!(&str in web_sys::HtmlInputElement),
    pub alt: attribute!(&str in web_sys::HtmlAreaElement),
    pub coords: attribute!(&str in web_sys::HtmlAreaElement),
    pub download: attribute!(&str in web_sys::HtmlAreaElement),
    pub href: attribute!(&str in web_sys::HtmlAreaElement),
    pub hreflang: attribute!(&str in web_sys::HtmlAnchorElement),
    pub ping: attribute!(&str in web_sys::HtmlAreaElement),
    pub rel: attribute!(&str in web_sys::HtmlAreaElement),
    pub shape: attribute!(&str in web_sys::HtmlAreaElement),
    pub target: attribute!(&str in web_sys::HtmlAreaElement),
}
