use wasm_bindgen_test::*;

use maomi::prelude::*;
use maomi_dom::{async_task, element::*, prelude::*};

use super::*;

#[wasm_bindgen_test]
async fn single_static_slot() {
    #[component(Backend = DomBackend)]
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

    #[component(Backend = DomBackend)]
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
                this.get_mut(|this, _| {
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
    #[component(Backend = DomBackend)]
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

    #[component(Backend = DomBackend)]
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
                this.get_mut(|this, _| {
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
    #[component(Backend = DomBackend)]
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

    #[component(Backend = DomBackend)]
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
                this.get_mut(|this, _| {
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

#[wasm_bindgen_test]
async fn multiple_slots_with_data() {
    #[derive(PartialEq, Clone)]
    enum ChildSlot {
        Even(u32),
        Odd(u32),
    }

    #[component(Backend = DomBackend, SlotData = ChildSlot)]
    struct Child {
        template: template! {
            for n in &*self.list {
                { &n.to_string() }
                <slot data=&{
                    match n % 2 {
                        0 => ChildSlot::Even(*n),
                        1 => ChildSlot::Odd(*n),
                        _ => unreachable!(),
                    }
                } />
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

    #[component(Backend = DomBackend)]
    struct Parent {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div>
                <Child list={ &self.list } slot:data>
                    match data {
                        ChildSlot::Even(_) => { "A" }
                        ChildSlot::Odd(_) => { "B" }
                    }
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
                list: vec![],
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
                    this.list.push(1);
                    this.list.push(2);
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
                        r#"1B2A"#,
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
                    this.list = vec![6, 7, 8];
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
                        r#"6A7B8A"#,
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
                this.get_mut(|this, _| {
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
async fn self_update_slot_data() {
    #[component(Backend = DomBackend, SlotData = String)]
    struct Child {
        template: template! {
            if self.slot_data.len() == 0 {
                <slot data={ "(empty)" } />
            }
            <slot data={ self.slot_data.as_str() } />
        },
        slot_data: String,
    }

    impl Component for Child {
        fn new() -> Self {
            Self {
                template: Default::default(),
                slot_data: "".into(),
            }
        }

        fn created(&self) {
            let this = self.rc();
            async_task(async move {
                this.update(|this| {
                    this.slot_data = "abc".into();
                })
                .await
                .unwrap();
            });
        }
    }

    impl Child {
        fn update_data(&self) -> AsyncCallback<()> {
            let this = self.rc();
            let (fut, cb) = AsyncCallback::new();
            async_task(async move {
                this.update(|this| {
                    this.slot_data = "".into();
                })
                .await
                .unwrap();
                cb(());
            });
            fut
        }
    }

    #[component(Backend = DomBackend)]
    struct Parent {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div>
                <Child slot:data>
                    "|" { data }
                </_>
            </div>
        },
    }

    impl Component for Parent {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
            }
        }

        fn created(&self) {
            let this = self.rc();
            async_task(async move {
                this.get(|this| {
                    assert_eq!(
                        this.template_structure()
                            .unwrap()
                            .0
                            .tag
                            .dom_element()
                            .inner_html(),
                        r#"|abc"#,
                    );
                })
                .await;
                let child = this
                    .get(|this| {
                        this.template_structure()
                            .unwrap()
                            .0
                            .single_slot()
                            .unwrap()
                            .0
                            .tag
                            .rc()
                    })
                    .await;
                child.get(|c| c.update_data()).await.await;
                this.get_mut(|this, _| {
                    assert_eq!(
                        this.template_structure()
                            .unwrap()
                            .0
                            .tag
                            .dom_element()
                            .inner_html(),
                        r#"|(empty)|"#,
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
