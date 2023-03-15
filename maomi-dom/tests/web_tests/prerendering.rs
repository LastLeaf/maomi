#![cfg(all(feature = "prerendering", feature = "prerendering-apply"))]

use wasm_bindgen_test::*;

use maomi::prelude::*;
use maomi_dom::{async_task, element::*, event::*, prelude::*};

use super::*;

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

    stylesheet! {
        #[css_name("abc")]
        class abc {}
        #[css_name("def")]
        class def {}
        style g(v: f32) {
            opacity = v;
        }
        style h(v: f32) {
            height = Px(v);
        }
    }

    #[component(Backend = DomBackend)]
    struct Parent {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div class:abc class:def={ &self.def_class } style:g={ &self.g_style } style:h=50>
                <Child title={ &self.child_title } text=&{ self.child_text }>
                    { &self.text }
                </Child>
            </div>
        },
        def_class: bool,
        g_style: f32,
        child_text: String,
        child_title: String,
        text: String,
    }

    impl Component for Parent {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
                def_class: true,
                g_style: 0.5,
                child_text: "456<".into(),
                child_title: "".into(),
                text: "123".into(),
            }
        }

        fn created(&self) {
            let this = self.rc();
            async_task(async move {
                this.update(|this| {
                    assert_eq!(
                        this.template_structure()
                            .unwrap()[0]
                            .downcast_ref::<maomi::node::Node<div>>()
                            .unwrap()
                            .tag
                            .dom_element()
                            .inner_html(),
                        r#"<div title="789&quot;"></div>456&lt;<!---->123"#,
                    );
                    this.def_class = false;
                    this.g_style = 1.;
                    this.child_text = "456".into();
                    this.child_title = "789".into();
                    this.text = "+123".into();
                })
                .await
                .unwrap();
                async_task(async move {
                    this.update_with(|this, _| {
                        assert_eq!(
                            this.template_structure()
                                .unwrap()[0]
                                .downcast_ref::<maomi::node::Node<div>>()
                                .unwrap()
                                .tag
                                .dom_element()
                                .outer_html(),
                            r#"<div class="abc" style="opacity: 1; height: 50px;"><div title="789"></div>456<!---->+123</div>"#,
                        );
                        (this.callback.take().unwrap())();
                    })
                    .await
                    .unwrap();
                })
            })
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

    let (html, prerendering_data) = test_component_prerendering::<Parent>(&"789\"").await;
    assert_eq!(
        &html,
        r#"<div class="abc def" style="opacity:0.5;height:50px"><div title="789&quot;"></div>456&lt;<!---->123</div>"#,
    );

    test_component_prerendering_apply::<Parent>(&html, prerendering_data).await;
}

#[wasm_bindgen_test]
async fn cold_event_in_prerendered() {
    #[component(Backend = DomBackend)]
    struct MyComp {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div scroll=@scroll_fn()></div>
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
            this.task_with(|this, _| {
                let dom_elem = this.template_structure()
                    .unwrap()[0]
                    .downcast_ref::<maomi::node::Node<div>>()
                    .unwrap()
                    .tag
                    .dom_element()
                    .clone();
                simulate_event(&dom_elem, "scroll", false, []);
            });
        }
    }

    impl MyComp {
        fn scroll_fn(this: ComponentRc<Self>, _ev: &mut ScrollEvent) {
            async_task(async move {
                this.update_with(|this, _| {
                    (this.callback.take().unwrap())();
                })
                .await
                .unwrap();
            });
        }
    }

    #[async_trait]
    impl PrerenderableComponent for MyComp {
        type QueryData = ();
        type PrerenderingData = ();

        async fn prerendering_data(_query_data: &Self::QueryData) -> Self::PrerenderingData {
            ()
        }

        fn apply_prerendering_data(&mut self, _data: Self::PrerenderingData) {
            // empty
        }
    }

    impl ComponentTest for MyComp {
        fn set_callback(&mut self, callback: ComponentTestCb) {
            self.callback = Some(callback);
        }
    }

    let (html, prerendering_data) = test_component_prerendering::<MyComp>(&()).await;
    test_component_prerendering_apply::<MyComp>(&html, prerendering_data).await;
}

#[wasm_bindgen_test]
async fn hot_event_in_prerendered() {
    #[component(Backend = DomBackend)]
    struct MyComp {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div touch_start=@handler()></div>
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
                this.get(|this| {
                    let dom_elem = this.template_structure()
                        .unwrap()[0]
                        .downcast_ref::<maomi::node::Node<div>>()
                        .unwrap()
                        .tag
                        .dom_element()
                        .clone();
                    simulate_event(
                        &dom_elem,
                        "touchstart",
                        true,
                        [("changedTouches", generate_fake_touch(&dom_elem, 1, 12, 34))],
                    );
                })
                .await;
            });
        }
    }

    impl MyComp {
        fn handler(this: ComponentRc<Self>, ev: &mut TouchEvent) {
            assert_eq!(ev.client_x(), 12);
            assert_eq!(ev.client_y(), 34);
            this.task_with(|this, _| {
                (this.callback.take().unwrap())();
            });
        }
    }

    #[async_trait]
    impl PrerenderableComponent for MyComp {
        type QueryData = ();
        type PrerenderingData = ();

        async fn prerendering_data(_query_data: &Self::QueryData) -> Self::PrerenderingData {
            ()
        }

        fn apply_prerendering_data(&mut self, _data: Self::PrerenderingData) {
            // empty
        }
    }

    impl ComponentTest for MyComp {
        fn set_callback(&mut self, callback: ComponentTestCb) {
            self.callback = Some(callback);
        }
    }

    let (html, prerendering_data) = test_component_prerendering::<MyComp>(&()).await;
    test_component_prerendering_apply::<MyComp>(&html, prerendering_data).await;
}
