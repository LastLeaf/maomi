//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

#[macro_use] extern crate log;

use std::rc::Rc;
use std::sync::Once;
use wasm_bindgen_test::*;
use web_sys;
use maomi::prelude::*;

wasm_bindgen_test_configure!(run_in_browser);

thread_local! {
    static DOCUMENT: web_sys::Document = {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        document
    };
    static WRAPPER: web_sys::Element = {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let wrapper = document.create_element("div").unwrap();
        wrapper.set_attribute("style", "height: 0; overflow: hidden").unwrap();
        document.body().unwrap().append_child(&wrapper).unwrap();
        wrapper
    };
}

fn create_dom_context() -> maomi::Context<maomi::backend::Dom> {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        console_log::init_with_level(log::Level::Debug).unwrap();
    });
    DOCUMENT.with(|document| {
        WRAPPER.with(|wrapper| {
            let placeholder = document.create_element("div").unwrap();
            placeholder.set_id("placeholder");
            wrapper.append_child(&placeholder).unwrap();
            maomi::Context::new(maomi::backend::Dom::new("placeholder"))
        })
    })
}

template!(tmpl HelloWorld {
    div {
        style = "display: inline";
        (&self.a);
    }
});
struct HelloWorld {
    a: String,
}
impl Component for HelloWorld {
    fn new(_ctx: Rc<ComponentContext>) -> Self {
        Self {
            a: "Hello world!".into()
        }
    }
}
#[wasm_bindgen_test]
fn create_new_component() {
    let mut context = create_dom_context();
    let root_component = context.new_root_component::<HelloWorld>();
    context.set_root_component(&root_component);
    let mut root_component = root_component.borrow_mut();
    assert_eq!(root_component.backend_element().inner_html(), r#"<div style="display: inline">Hello world!</div>"#);
    root_component.a = "Hello world again!".into();
    root_component.force_apply_updates();
    assert_eq!(root_component.backend_element().inner_html(), r#"<div style="display: inline">Hello world again!</div>"#);
    root_component.a = "Hello world again and again!".into();
    root_component.force_apply_updates();
    assert_eq!(root_component.backend_element().inner_html(), r#"<div style="display: inline">Hello world again and again!</div>"#);
}

// template!(tmpl<B: Backend> TemplateIf<B> {
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
// struct TemplateIf<B: Backend> {
//     ctx: Rc<ComponentContext<B>>,
//     pub a: Prop<String>,
// }
// impl<B: Backend> Component<B> for TemplateIf<B> {
//     fn new(ctx: Rc<ComponentContext<B>>) -> Self {
//         Self {
//             ctx,
//             a: "Hello world!".into()
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
