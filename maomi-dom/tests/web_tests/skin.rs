use wasm_bindgen_test::*;

use maomi::prelude::*;
use maomi_dom::{async_task, element::*, prelude::*};

use super::*;

#[wasm_bindgen_test]
async fn skin_import() {
    dom_css! {
        @import "/tests/web_tests/import.maomi.css";
        .self_class {}
        // TODO import const and macro
    }

    #[component(Backend = DomBackend)]
    struct MyComp {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div class:imported_class class:self_class></div>
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
            let this = self.rc();
            async_task(async move {
                this.get_mut(|this, _| {
                    assert_eq!(
                        this.template_structure()
                            .unwrap()
                            .0
                            .tag
                            .dom_element()
                            .outer_html(),
                        r#"<div class="imported-class self-class"></div>"#,
                    );
                    (this.callback.take().unwrap())();
                })
                .await
                .unwrap();
            })
        }
    }

    impl ComponentTest for MyComp {
        fn set_callback(&mut self, callback: ComponentTestCb) {
            self.callback = Some(callback);
        }
    }

    test_component::<MyComp>().await;
}
