use wasm_bindgen_test::*;

use maomi::prelude::*;
use maomi_dom::{element::*, prelude::*};

use super::*;

#[wasm_bindgen_test]
async fn skin_class() {
    stylesheet! {
        #[css_name("a-class")]
        class a_class {}
    }

    #[component(Backend = DomBackend)]
    struct MyComp {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div class:a_class></div>
        },
    }

    impl Component for MyComp {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
            }
        }

        fn created(&self) {
            self.rc().task_with(|this, _| {
                assert_eq!(
                    this.template_structure()
                        .unwrap()
                        .0
                        .tag
                        .dom_element()
                        .outer_html(),
                    r#"<div class="a-class"></div>"#,
                );
                (this.callback.take().unwrap())();
            });
        }
    }

    impl ComponentTest for MyComp {
        fn set_callback(&mut self, callback: ComponentTestCb) {
            self.callback = Some(callback);
        }
    }

    test_component::<MyComp>().await;
}

#[wasm_bindgen_test]
async fn skin_style() {
    stylesheet! {
        style opacity(v: f32) {
            opacity = v;
        }
        style text_color(v: &str) {
            color = Color(v);
        }
        style url(v: &str) {
            background_image = url(v);
        }
    }

    #[component(Backend = DomBackend)]
    struct MyComp {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div style:opacity=0 style:text_color="abc" style:url="a.png"></div>
        },
    }

    impl Component for MyComp {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
            }
        }

        fn created(&self) {
            self.rc().task_with(|this, _| {
                let style = this.template_structure()
                    .unwrap()
                    .0
                    .tag
                    .dom_element()
                    .dyn_ref::<web_sys::HtmlElement>()
                    .unwrap()
                    .style();
                assert_eq!(
                    style.get_property_value("opacity"),
                    Ok("0".into()),
                );
                assert_eq!(
                    style.get_property_value("color"),
                    Ok("rgb(170, 187, 204)".into()),
                );
                assert_eq!(
                    style.get_property_value("background-image"),
                    Ok(r#"url("a.png")"#.into()),
                );
                (this.callback.take().unwrap())();
            });
        }
    }

    impl ComponentTest for MyComp {
        fn set_callback(&mut self, callback: ComponentTestCb) {
            self.callback = Some(callback);
        }
    }

    test_component::<MyComp>().await;
}
