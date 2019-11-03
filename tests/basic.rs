//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

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

template!(tmpl for HelloWorld {
    div {
        style = "display: inline";
        (&self.a);
        slot;
    }
});
struct HelloWorld {
    a: String,
}
impl<B: Backend> Component<B> for HelloWorld {
    fn new(_ctx: ComponentContext<B, Self>) -> Self {
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

template!(tmpl for ParentComponent {
    span {
        HelloWorld {
            style = "display: block";
            a = "Hello world";
            " | ";
            HelloWorld {
                a = &self.s;
            }
        }
    }
});
struct ParentComponent {
    pub s: String,
}
impl<B: Backend> Component<B> for ParentComponent {
    fn new(_ctx: ComponentContext<B, Self>) -> Self {
        Self {
            s: "from parent!".into()
        }
    }
}
#[wasm_bindgen_test]
fn parent_component() {
    let mut context = create_dom_context();
    let root_component = context.new_root_component::<ParentComponent>();
    context.set_root_component(&root_component);
    let mut root_component = root_component.borrow_mut();
    assert_eq!(root_component.backend_element().inner_html(), r#"<span><maomi-hello-world style="display: block"><div style="display: inline">Hello world | <maomi-hello-world><div style="display: inline">from parent!</div></maomi-hello-world></div></maomi-hello-world></span>"#);
    root_component.s = "from parent again!".into();
    root_component.force_apply_updates();
    assert_eq!(root_component.backend_element().inner_html(), r#"<span><maomi-hello-world style="display: block"><div style="display: inline">Hello world | <maomi-hello-world><div style="display: inline">from parent again!</div></maomi-hello-world></div></maomi-hello-world></span>"#);
    root_component.s = "from parent again and again!".into();
    root_component.force_apply_updates();
    assert_eq!(root_component.backend_element().inner_html(), r#"<span><maomi-hello-world style="display: block"><div style="display: inline">Hello world | <maomi-hello-world><div style="display: inline">from parent again and again!</div></maomi-hello-world></div></maomi-hello-world></span>"#);
}

template!(tmpl<D: Backend> for<D> TemplateIf<D> {
    div {
        if self.a == 0 {
            "branch 0";
        } else if self.a == 1 {
            "branch 1";
        } else {
            "other branches";
        }
    }
});
struct TemplateIf<D: Backend> {
    _ctx: ComponentContext<D, Self>,
    pub a: i32,
}
impl<D: Backend> Component<D> for TemplateIf<D> {
    fn new(_ctx: ComponentContext<D, Self>) -> Self {
        Self {
            _ctx,
            a: 0
        }
    }
}
#[wasm_bindgen_test]
fn template_if() {
    let mut context = create_dom_context();
    let root_component = context.new_root_component::<TemplateIf<_>>();
    context.set_root_component(&root_component);
    let mut root_component = root_component.borrow_mut();
    assert_eq!(root_component.backend_element().inner_html(), "<div>branch 0</div>");
    root_component.a = -1;
    root_component.force_apply_updates();
    assert_eq!(root_component.backend_element().inner_html(), "<div>other branches</div>");
    root_component.a = 1;
    root_component.force_apply_updates();
    assert_eq!(root_component.backend_element().inner_html(), "<div>branch 1</div>");
}

template!(tmpl for TemplateFor {
    for item in &self.list {
        div {
            (item);
        }
    }
});
struct TemplateFor {
    list: Vec<String>,
}
impl<B: Backend> Component<B> for TemplateFor {
    fn new(_ctx: ComponentContext<B, Self>) -> Self {
        Self {
            list: vec!["Aa".into(), "Bb".into(), "Cc".into()]
        }
    }
}
#[wasm_bindgen_test]
fn template_for() {
    let mut context = create_dom_context();
    let root_component = context.new_root_component::<TemplateFor>();
    context.set_root_component(&root_component);
    let mut root_component = root_component.borrow_mut();
    assert_eq!(root_component.backend_element().inner_html(), "<div>Aa</div><div>Bb</div><div>Cc</div>");
    root_component.list[1] = "Dd".into();
    root_component.force_apply_updates();
    assert_eq!(root_component.backend_element().inner_html(), "<div>Aa</div><div>Dd</div><div>Cc</div>");
}
