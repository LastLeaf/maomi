use wasm_bindgen_test::*;

use maomi::prelude::*;
use maomi_dom::{element::*, prelude::*};

use super::*;

#[cfg(feature = "prerendering")]
#[wasm_bindgen_test]
async fn generate_prerendering_html() {
    #[component(Backend = DomBackend)]
    struct Child {
        template: template! {
            <div title={ self.title.as_str() } />
            { &self.text } <slot />
        },
        text: Prop<String>,
        title: Prop<String>,
    }

    impl Component for Child {
        fn new() -> Self {
            Self {
                template: Default::default(),
                text: Prop::new("".into()),
                title: Prop::new("".into()),
            }
        }
    }

    dom_css! {
        .abc {}
        .def {}
    }

    #[component(Backend = DomBackend)]
    struct Parent {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div class:abc class:def>
                <Child title={ &self.child_title } text=&{ self.child_text }>
                    { "123" }
                </Child>
            </div>
        },
        child_text: String,
        child_title: String,
    }

    impl Component for Parent {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
                child_text: "456".into(),
                child_title: "".into(),
            }
        }

        fn created(&self) {
            panic!("The created function should not be called in prerendering stage")
        }
    }

    #[async_trait]
    impl PrerenderableComponent for Parent {
        type QueryData = &'static str;
        type PrerenderingData = String;

        async fn prerendering_data(query_data: &Self::QueryData) -> Self::PrerenderingData {
            query_data.to_string()
        }

        fn apply_prerendering_data(&mut self, data: Self::PrerenderingData) {
            self.child_title = data;
        }
    }

    impl ComponentTest for Parent {
        fn set_callback(&mut self, callback: ComponentTestCb) {
            self.callback = Some(callback);
        }
    }

    assert_eq!(
        &test_component_prerendering::<Parent>(&"789").await,
        r#"<div class="abc def"><div title="789"/>456<!---->123</div>"#,
    );
}
