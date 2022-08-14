use wasm_bindgen_test::*;

use maomi::prelude::*;
use maomi_dom::{async_task, element::*, prelude::*};

mod env;
use env::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn single_static_slot() {
    #[component(for DomBackend)]
    struct Child {
        template: template! {
            <div title={ &*self.title }> { &self.text } </div>
            <slot />
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

    #[component(for DomBackend)]
    struct Parent {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div>
                <Child title={ &self.hello_title } text=&{ self.hello_text }>
                    { &self.slot_text }
                    <slot />
                </Child>
            </div>
        },
        hello_text: String,
        hello_title: String,
        slot_text: String,
    }

    impl Component for Parent {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
                hello_text: "".into(),
                hello_title: "Again".into(),
                slot_text: "Hello".into(),
            }
        }

        fn created(&self) {
            let this = self.rc();
            async_task(async move {
                this.update(|this| {
                    assert_eq!(
                        this.template_structure()
                            .unwrap()
                            .0
                            .tag
                            .dom_element()
                            .outer_html(),
                        r#"<div><div title="Again"></div>Hello</div>"#,
                    );
                    this.hello_text = "Hello world again!".into();
                    this.slot_text = "".into();
                })
                .await
                .unwrap();
                this.get_mut(|this| {
                    assert_eq!(
                        this.template_structure()
                            .unwrap()
                            .0
                            .tag
                            .dom_element()
                            .outer_html(),
                        r#"<div><div title="Again">Hello world again!</div></div>"#,
                    );
                    (this.callback.take().unwrap())();
                })
                .await
                .unwrap();
            });
        }
    }

    impl ComponentTest for Parent {
        fn set_callback(&mut self, callback: ComponentTestCb) {
            self.callback = Some(callback);
        }
    }

    test_component::<Parent>().await;
}

#[wasm_bindgen_test]
async fn single_dynamic_slot() {
    #[component(for DomBackend)]
    struct Child {
        template: template! {
            <div>
                if let Some(text) = self.text.as_ref() {
                    match text.as_str() {
                        "" => {
                            <slot />
                        }
                        _ => {
                            { text }
                            <slot />
                        }
                    }
                }
            </div>
        },
        text: Prop<Option<String>>,
    }

    impl Component for Child {
        fn new() -> Self {
            Self {
                template: Default::default(),
                text: Prop::new(None),
            }
        }
    }

    #[component(for DomBackend)]
    struct Parent {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div>
                <Child text={ &self.hello_text }>
                    <span />
                </_>
            </div>
        },
        hello_text: Option<String>,
    }

    impl Component for Parent {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
                hello_text: None,
            }
        }

        fn created(&self) {
            let this = self.rc();
            async_task(async move {
                this.update(|this| {
                    assert_eq!(
                        this.template_structure()
                            .unwrap()
                            .0
                            .tag
                            .dom_element()
                            .inner_html(),
                        r#"<div></div>"#,
                    );
                    assert!(this
                        .template_structure()
                        .unwrap()
                        .0
                        .single_slot()
                        .unwrap()
                        .0
                        .single_slot()
                        .is_none());
                    this.hello_text = Some("".into());
                })
                .await
                .unwrap();
                this.update(|this| {
                    assert_eq!(
                        this.template_structure()
                            .unwrap()
                            .0
                            .tag
                            .dom_element()
                            .inner_html(),
                        r#"<div><span></span></div>"#,
                    );
                    assert!(this
                        .template_structure()
                        .unwrap()
                        .0
                        .single_slot()
                        .unwrap()
                        .0
                        .single_slot()
                        .is_some());
                    this.hello_text = Some("text".into());
                })
                .await
                .unwrap();
                this.update(|this| {
                    assert_eq!(
                        this.template_structure()
                            .unwrap()
                            .0
                            .tag
                            .dom_element()
                            .inner_html(),
                        r#"<div>text<span></span></div>"#,
                    );
                    assert!(this
                        .template_structure()
                        .unwrap()
                        .0
                        .single_slot()
                        .unwrap()
                        .0
                        .single_slot()
                        .is_some());
                    this.hello_text = None;
                })
                .await
                .unwrap();
                this.get_mut(|this| {
                    assert_eq!(
                        this.template_structure()
                            .unwrap()
                            .0
                            .tag
                            .dom_element()
                            .inner_html(),
                        r#"<div></div>"#,
                    );
                    assert!(this
                        .template_structure()
                        .unwrap()
                        .0
                        .single_slot()
                        .unwrap()
                        .0
                        .single_slot()
                        .is_none());
                    (this.callback.take().unwrap())();
                })
                .await
                .unwrap();
            });
        }
    }

    impl ComponentTest for Parent {
        fn set_callback(&mut self, callback: ComponentTestCb) {
            self.callback = Some(callback);
        }
    }

    test_component::<Parent>().await;
}

#[wasm_bindgen_test]
async fn multiple_slots() {
    #[component(for DomBackend)]
    struct Child {
        template: template! {
            for n in &*self.list {
                { &n.to_string() }
                <slot />
            }
        },
        list: Prop<Vec<u32>>,
    }

    impl Component for Child {
        fn new() -> Self {
            Self {
                template: Default::default(),
                list: Prop::new(vec![]),
            }
        }
    }

    #[component(for DomBackend)]
    struct Parent {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div>
                <Child list={ &self.list }>
                    "A"
                </_>
            </div>
        },
        list: Vec<u32>,
    }

    impl Component for Parent {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
                list: vec![12],
            }
        }

        fn created(&self) {
            let this = self.rc();
            async_task(async move {
                this.update(|this| {
                    assert_eq!(
                        this.template_structure()
                            .unwrap()
                            .0
                            .tag
                            .dom_element()
                            .inner_html(),
                        r#"12A"#,
                    );
                    assert!(this
                        .template_structure()
                        .unwrap()
                        .0
                        .single_slot()
                        .unwrap()
                        .0
                        .single_slot()
                        .is_some());
                    this.list.push(34);
                })
                .await
                .unwrap();
                this.update(|this| {
                    assert_eq!(
                        this.template_structure()
                            .unwrap()
                            .0
                            .tag
                            .dom_element()
                            .inner_html(),
                        r#"12A34A"#,
                    );
                    assert!(this
                        .template_structure()
                        .unwrap()
                        .0
                        .single_slot()
                        .unwrap()
                        .0
                        .single_slot()
                        .is_none());
                    this.list = vec![];
                })
                .await
                .unwrap();
                this.update(|this| {
                    assert_eq!(
                        this.template_structure()
                            .unwrap()
                            .0
                            .tag
                            .dom_element()
                            .inner_html(),
                        r#""#,
                    );
                    assert!(this
                        .template_structure()
                        .unwrap()
                        .0
                        .single_slot()
                        .unwrap()
                        .0
                        .single_slot()
                        .is_none());
                    this.list = vec![12, 34, 56];
                })
                .await
                .unwrap();
                this.get_mut(|this| {
                    assert_eq!(
                        this.template_structure()
                            .unwrap()
                            .0
                            .tag
                            .dom_element()
                            .inner_html(),
                        r#"12A34A56A"#,
                    );
                    assert!(this
                        .template_structure()
                        .unwrap()
                        .0
                        .single_slot()
                        .unwrap()
                        .0
                        .single_slot()
                        .is_none());
                    (this.callback.take().unwrap())();
                })
                .await
                .unwrap();
            });
        }
    }

    impl ComponentTest for Parent {
        fn set_callback(&mut self, callback: ComponentTestCb) {
            self.callback = Some(callback);
        }
    }

    test_component::<Parent>().await;
}
