//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys;
use maomi::prelude::*;

wasm_bindgen_test_configure!(run_in_browser);

fn append_placeholder() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let placeholder = document.create_element("div").unwrap();
    placeholder.set_id("placeholder");
    document.body().unwrap().append_child(&placeholder).unwrap();
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

template!(tmpl HelloWorld {
    div {
        style = "display: inline";
        (&self.a);
    }
});
#[component]
struct HelloWorld {
    #[property]
    a: String,
}
#[component]
impl HelloWorld {
    fn new() -> Self {
        Self {
            a: "Hello world!".into()
        }
    }
}
#[wasm_bindgen_test]
fn create_new_component() {
    append_placeholder();
    let mut context = maomi::Context::new(maomi::backend::Dom::new("placeholder"));
    context.set_root_component(Box::new(HelloWorld::new()));
    let root_component = context.root_component().as_ref().unwrap().borrow();
    console_log!("{:?}", root_component.backend_element().outer_html());
}
//
// template!(tmpl TemplateIf {
//     div {
//         if self.a == 0 {
//             "branch 0";
//         } else if self.a == 1 {
//             "branch 1";
//         } else {
//             "other branches";
//         }
//     }
// });
// #[component]
// struct TemplateIf<B: Backend> {
//     a: u32,
// }
// #[component]
// impl<B: Backend> TemplateIf<B> {
//     fn new() -> Self {
//         Self {
//             a: 1
//         }
//     }
// }
// #[wasm_bindgen_test]
// fn template_if() {
//     let test_component = maomi::render(Box::new(TemplateIf::new()));
//     console_log!("{:?}", test_component);
// }
//
// template!(tmpl TemplateFor {
//     for item in &self.list {
//         div {
//             (item);
//         }
//     }
// });
// #[component]
// struct TemplateFor<B: Backend> {
//     list: Vec<String>,
// }
// #[component]
// impl<B: Backend> TemplateFor<B> {
//     fn new() -> Self {
//         Self {
//             list: vec!["Aa".into(), "Bb".into(), "Cc".into()]
//         }
//     }
// }
// #[wasm_bindgen_test]
// fn template_for() {
//     let test_component = maomi::render(Box::new(TemplateFor::new()));
//     console_log!("{:?}", test_component);
// }
