//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

#[allow(unused_imports)] #[macro_use] extern crate log;

use std::sync::Once;
use wasm_bindgen_test::*;
use wasm_bindgen::JsCast;
use web_sys;
use maomi::prelude::*;
use maomi::global_events::{MouseEvent, MouseButton};

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
    context.set_root_component(root_component);
    let root_component = context.root_component::<HelloWorld>().unwrap();
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
    context.set_root_component(root_component);
    let root_component = context.root_component::<ParentComponent>().unwrap();
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
    context.set_root_component(root_component);
    let root_component = context.root_component::<TemplateIf<_>>().unwrap();
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
    context.set_root_component(root_component);
    let root_component = context.root_component::<TemplateFor>().unwrap();
    let mut root_component = root_component.borrow_mut();
    assert_eq!(root_component.backend_element().inner_html(), "<div>Aa</div><div>Bb</div><div>Cc</div>");
    // modify
    root_component.list[1] = "Dd".into();
    root_component.force_apply_updates();
    assert_eq!(root_component.backend_element().inner_html(), r#"<div>Aa</div><div>Dd</div><div>Cc</div>"#);
    // append
    root_component.list.push("Ee".into());
    root_component.list.push("Ff".into());
    root_component.force_apply_updates();
    assert_eq!(root_component.backend_element().inner_html(), r#"<div>Aa</div><div>Dd</div><div>Cc</div><div>Ee</div><div>Ff</div>"#);
    // insert
    root_component.list.insert(1, "Gg".into());
    root_component.list.insert(2, "Hh".into());
    root_component.force_apply_updates();
    assert_eq!(root_component.backend_element().inner_html(), r#"<div>Aa</div><div>Gg</div><div>Hh</div><div>Dd</div><div>Cc</div><div>Ee</div><div>Ff</div>"#);
    // remove
    root_component.list.remove(3);
    root_component.list.remove(3);
    root_component.force_apply_updates();
    assert_eq!(root_component.backend_element().inner_html(), r#"<div>Aa</div><div>Gg</div><div>Hh</div><div>Ee</div><div>Ff</div>"#);
    // multi-insert
    root_component.list.insert(0, "Ii".into());
    root_component.list.insert(3, "Jj".into());
    root_component.list.push("Kk".into());
    root_component.force_apply_updates();
    assert_eq!(root_component.backend_element().inner_html(), r#"<div>Ii</div><div>Aa</div><div>Gg</div><div>Jj</div><div>Hh</div><div>Ee</div><div>Ff</div><div>Kk</div>"#);
    // multi-remove
    root_component.list.remove(0);
    root_component.list.remove(1);
    root_component.list.remove(5);
    root_component.force_apply_updates();
    assert_eq!(root_component.backend_element().inner_html(), r#"<div>Aa</div><div>Jj</div><div>Hh</div><div>Ee</div><div>Ff</div>"#);
}

template!(tmpl for TemplateForKey {
    for item in &self.list use k: i32 {
        div {
            (&item.v);
        }
    }
});
struct TemplateForKeyItem {
    k: i32,
    v: String
}
struct TemplateForKey {
    list: Vec<TemplateForKeyItem>,
}
impl<B: Backend> Component<B> for TemplateForKey {
    fn new(_ctx: ComponentContext<B, Self>) -> Self {
        Self {
            list: vec![TemplateForKeyItem {
                k: 1,
                v: "1".into(),
            }, TemplateForKeyItem {
                k: 2,
                v: "2".into(),
            }]
        }
    }
}
#[wasm_bindgen_test]
fn template_for_key() {
    let mut context = create_dom_context();
    let root_component = context.new_root_component::<TemplateForKey>();
    context.set_root_component(root_component);
    let root_component = context.root_component::<TemplateForKey>().unwrap();
    let mut root_component = root_component.borrow_mut();
    assert_eq!(root_component.backend_element().inner_html(), r#"<div>1</div><div>2</div>"#);
    // modify
    root_component.list[1] = TemplateForKeyItem {
        k: 2,
        v: "22".into(),
    };
    root_component.force_apply_updates();
    assert_eq!(root_component.backend_element().inner_html(), r#"<div>1</div><div>22</div>"#);
    // append
    root_component.list.push(TemplateForKeyItem {
        k: 3,
        v: "3".into(),
    });
    root_component.list.push(TemplateForKeyItem {
        k: 4,
        v: "4".into(),
    });
    root_component.force_apply_updates();
    assert_eq!(root_component.backend_element().inner_html(), r#"<div>1</div><div>22</div><div>3</div><div>4</div>"#);
    // insert
    root_component.list.insert(1, TemplateForKeyItem {
        k: 5,
        v: "5".into(),
    });
    root_component.list.insert(2, TemplateForKeyItem {
        k: 6,
        v: "6".into(),
    });
    root_component.force_apply_updates();
    assert_eq!(root_component.backend_element().inner_html(), r#"<div>1</div><div>5</div><div>6</div><div>22</div><div>3</div><div>4</div>"#);
    // remove
    root_component.list.remove(3);
    root_component.list.remove(3);
    root_component.force_apply_updates();
    assert_eq!(root_component.backend_element().inner_html(), r#"<div>1</div><div>5</div><div>6</div><div>4</div>"#);
    // multi-insert
    root_component.list.insert(0, TemplateForKeyItem {
        k: 7,
        v: "7".into(),
    });
    root_component.list.insert(3, TemplateForKeyItem {
        k: 8,
        v: "8".into(),
    });
    root_component.list.push(TemplateForKeyItem {
        k: 9,
        v: "9".into(),
    });
    root_component.force_apply_updates();
    assert_eq!(root_component.backend_element().inner_html(), r#"<div>7</div><div>1</div><div>5</div><div>8</div><div>6</div><div>4</div><div>9</div>"#);
    // multi-remove
    root_component.list.remove(0);
    root_component.list.remove(1);
    root_component.list.remove(4);
    root_component.force_apply_updates();
    assert_eq!(root_component.backend_element().inner_html(), r#"<div>1</div><div>8</div><div>6</div><div>4</div>"#);
}

template!(tmpl for TemplateGlobalEvent {
    div {
        @click = |mut self_ref_mut, ev: &MouseEvent| {
            self_ref_mut.triggered.push(1);
            assert_eq!(ev.button, MouseButton::Primary);
        };
        span {
            mark = "child";
            @click = |mut self_ref_mut, ev: &MouseEvent| {
                self_ref_mut.triggered.push(2);
                assert_eq!(ev.button, MouseButton::Primary);
            };
        }
    }
});
struct TemplateGlobalEvent {
    triggered: Vec<usize>
}
impl<B: Backend> Component<B> for TemplateGlobalEvent {
    fn new(_ctx: ComponentContext<B, Self>) -> Self {
        Self {
            triggered: vec![]
        }
    }
}
#[wasm_bindgen_test]
fn template_global_event() {
    let mut context = create_dom_context();
    let root_component = context.new_root_component::<TemplateGlobalEvent>();
    context.set_root_component(root_component);
    let root_component = context.root_component::<TemplateGlobalEvent>().unwrap();
    {
        DOCUMENT.with(|document| {
            let dom_ev = document.create_event("MouseEvents").unwrap().dyn_into::<web_sys::MouseEvent>().unwrap();
            dom_ev.init_mouse_event_with_can_bubble_arg("click", true);
            let child = root_component.borrow().marked("child").unwrap();
            let backend_element = child.borrow().backend_element().unwrap() as *const maomi::backend::dom::DomElement; // HACK force child exit borrowing
            let backend_element: &maomi::backend::dom::DomElement = unsafe { backend_element.as_ref() }.unwrap();
            backend_element.dispatch_event(&dom_ev).unwrap();
        });
    }
    assert_eq!(root_component.borrow().triggered, vec![2, 1]);
}
