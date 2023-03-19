use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

use maomi::prelude::*;
use maomi_dom::{async_task, element::*, event::*, prelude::*};

use super::*;

#[wasm_bindgen_test]
async fn animation_event() {
    #[component(Backend = DomBackend)]
    struct MyComp {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div
                animation_start=@ani_fn(&1)
                animation_iteration=@ani_fn(&2)
                animation_end=@ani_fn(&3)
                animation_cancel=@ani_fn(&4)
            >
                { &self.state.to_string() }
            </div>
        },
        state: u32,
    }

    impl Component for MyComp {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
                state: 0,
            }
        }

        fn created(&self) {
            let this = self.rc();
            async_task(async move {
                Self::next_step(this, 0).await;
            });
        }
    }

    impl MyComp {
        fn ani_fn(this: ComponentRc<Self>, ev: &mut AnimationEvent, kind: &u32) {
            assert_eq!(ev.elapsed_time(), 123.);
            let kind = *kind;
            async_task(async move {
                this.update(move |this| this.state = kind).await.unwrap();
                Self::next_step(this.clone(), kind).await;
            });
        }

        async fn next_step(this: ComponentRc<Self>, state: u32) {
            match state {
                0 => {
                    this.get(|this| {
                        let dom_elem = first_dom!(this, div).clone();
                        assert_eq!(dom_elem.outer_html(), r#"<div>0</div>"#,);
                        simulate_event(
                            &dom_elem,
                            "animationstart",
                            false,
                            [
                                ("animationName", JsValue::from_str("ani")),
                                ("elapsedTime", JsValue::from_f64(123.)),
                            ],
                        );
                    })
                    .await
                }
                1 => {
                    this.get(|this| {
                        let dom_elem = first_dom!(this, div).clone();
                        assert_eq!(dom_elem.outer_html(), r#"<div>1</div>"#,);
                        simulate_event(
                            &dom_elem,
                            "animationiteration",
                            false,
                            [
                                ("animationName", JsValue::from_str("ani")),
                                ("elapsedTime", JsValue::from_f64(123.)),
                            ],
                        );
                    })
                    .await
                }
                2 => {
                    this.get(|this| {
                        let dom_elem = first_dom!(this, div).clone();
                        assert_eq!(dom_elem.outer_html(), r#"<div>2</div>"#,);
                        simulate_event(
                            &dom_elem,
                            "animationend",
                            false,
                            [
                                ("animationName", JsValue::from_str("ani")),
                                ("elapsedTime", JsValue::from_f64(123.)),
                            ],
                        );
                    })
                    .await
                }
                3 => {
                    this.get(|this| {
                        let dom_elem = first_dom!(this, div).clone();
                        assert_eq!(dom_elem.outer_html(), r#"<div>3</div>"#,);
                        simulate_event(
                            &dom_elem,
                            "animationcancel",
                            false,
                            [
                                ("animationName", JsValue::from_str("ani")),
                                ("elapsedTime", JsValue::from_f64(123.)),
                            ],
                        );
                    })
                    .await
                }
                _ => this
                    .update_with(|this, _| {
                        let dom_elem = first_dom!(this, div).clone();
                        assert_eq!(dom_elem.outer_html(), r#"<div>4</div>"#,);
                        (this.callback.take().unwrap())();
                    })
                    .await
                    .unwrap(),
            }
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
async fn transition_event() {
    #[component(Backend = DomBackend)]
    struct MyComp {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div
                transition_run=@ani_fn(&1)
                transition_start=@ani_fn(&2)
                transition_end=@ani_fn(&3)
                transition_cancel=@ani_fn(&4)
            >
                { &self.state.to_string() }
            </div>
        },
        state: u32,
    }

    impl Component for MyComp {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
                state: 0,
            }
        }

        fn created(&self) {
            let this = self.rc();
            async_task(async move {
                Self::next_step(this, 0).await;
            });
        }
    }

    impl MyComp {
        fn ani_fn(this: ComponentRc<Self>, ev: &mut TransitionEvent, kind: &u32) {
            assert_eq!(ev.property_name(), "ani");
            assert_eq!(ev.elapsed_time(), 123.);
            let kind = *kind;
            async_task(async move {
                this.update(move |this| this.state = kind).await.unwrap();
                Self::next_step(this.clone(), kind).await;
            });
        }

        async fn next_step(this: ComponentRc<Self>, state: u32) {
            match state {
                0 => {
                    this.get(|this| {
                        let dom_elem = first_dom!(this, div).clone();
                        assert_eq!(dom_elem.outer_html(), r#"<div>0</div>"#,);
                        simulate_event(
                            &dom_elem,
                            "transitionrun",
                            false,
                            [
                                ("propertyName", JsValue::from_str("ani")),
                                ("elapsedTime", JsValue::from_f64(123.)),
                            ],
                        );
                    })
                    .await
                }
                1 => {
                    this.get(|this| {
                        let dom_elem = first_dom!(this, div).clone();
                        assert_eq!(dom_elem.outer_html(), r#"<div>1</div>"#,);
                        simulate_event(
                            &dom_elem,
                            "transitionstart",
                            false,
                            [
                                ("propertyName", JsValue::from_str("ani")),
                                ("elapsedTime", JsValue::from_f64(123.)),
                            ],
                        );
                    })
                    .await
                }
                2 => {
                    this.get(|this| {
                        let dom_elem = first_dom!(this, div).clone();
                        assert_eq!(dom_elem.outer_html(), r#"<div>2</div>"#,);
                        simulate_event(
                            &dom_elem,
                            "transitionend",
                            false,
                            [
                                ("propertyName", JsValue::from_str("ani")),
                                ("elapsedTime", JsValue::from_f64(123.)),
                            ],
                        );
                    })
                    .await
                }
                3 => {
                    this.get(|this| {
                        let dom_elem = first_dom!(this, div).clone();
                        assert_eq!(dom_elem.outer_html(), r#"<div>3</div>"#,);
                        simulate_event(
                            &dom_elem,
                            "transitioncancel",
                            false,
                            [
                                ("propertyName", JsValue::from_str("ani")),
                                ("elapsedTime", JsValue::from_f64(123.)),
                            ],
                        );
                    })
                    .await
                }
                _ => this
                    .update_with(|this, _| {
                        let dom_elem = first_dom!(this, div).clone();
                        assert_eq!(dom_elem.outer_html(), r#"<div>4</div>"#,);
                        (this.callback.take().unwrap())();
                    })
                    .await
                    .unwrap(),
            }
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
async fn scroll_event() {
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
            async_task(async move {
                this.get(|this| {
                    let dom_elem = first_dom!(this, div).clone();
                    simulate_event(&dom_elem, "scroll", false, []);
                })
                .await;
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

    impl ComponentTest for MyComp {
        fn set_callback(&mut self, callback: ComponentTestCb) {
            self.callback = Some(callback);
        }
    }

    test_component::<MyComp>().await;
}

macro_rules! test_touch_events {
    ($ev:ident, $ev_js_name:expr) => {
        #[wasm_bindgen_test]
        async fn $ev() {
            #[component(Backend = DomBackend)]
            struct MyComp {
                callback: Option<ComponentTestCb>,
                template: template! {
                    <div $ev=@handler()></div>
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
                            let dom_elem = first_dom!(this, div).clone();
                            simulate_event(
                                &dom_elem,
                                $ev_js_name,
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
                    async_task(async move {
                        this.update_with(|this, _| {
                            (this.callback.take().unwrap())();
                        })
                        .await
                        .unwrap();
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
    };
}

test_touch_events!(touch_start, "touchstart");
test_touch_events!(touch_move, "touchmove");
test_touch_events!(touch_end, "touchend");
test_touch_events!(touch_cancel, "touchcancel");

macro_rules! test_mouse_events {
    ($ev:ident, $ev_js_name:expr) => {
        #[wasm_bindgen_test]
        async fn $ev() {
            #[component(Backend = DomBackend)]
            struct MyComp {
                callback: Option<ComponentTestCb>,
                template: template! {
                    <div $ev=@handler()></div>
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
                            let dom_elem = first_dom!(this, div).clone();
                            simulate_event(
                                &dom_elem,
                                $ev_js_name,
                                true,
                                [
                                    ("button", JsValue::from_f64(2.)),
                                    ("clientX", JsValue::from_f64(56.)),
                                    ("clientY", JsValue::from_f64(78.)),
                                ],
                            );
                        })
                        .await;
                    });
                }
            }

            impl MyComp {
                fn handler(this: ComponentRc<Self>, ev: &mut MouseEvent) {
                    assert_eq!(ev.button(), MouseButton::Secondary);
                    assert_eq!(ev.client_x(), 56);
                    assert_eq!(ev.client_y(), 78);
                    async_task(async move {
                        this.update_with(|this, _| {
                            (this.callback.take().unwrap())();
                        })
                        .await
                        .unwrap();
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
    };
}

test_mouse_events!(mouse_down, "mousedown");
test_mouse_events!(mouse_up, "mouseup");
test_mouse_events!(mouse_move, "mousemove");
test_mouse_events!(mouse_enter, "mouseenter");
test_mouse_events!(mouse_leave, "mouseleave");

#[wasm_bindgen_test]
async fn tap() {
    #[component(Backend = DomBackend)]
    struct MyComp {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div tap=@handler()></div>
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
                    let dom_elem = first_dom!(this, div).clone();
                    simulate_event(
                        &dom_elem,
                        "mousedown",
                        true,
                        [
                            ("button", JsValue::from_f64(0.)),
                            ("clientX", JsValue::from_f64(12.)),
                            ("clientY", JsValue::from_f64(34.)),
                        ],
                    );
                    simulate_event(
                        &web_sys::window().unwrap().document().unwrap(),
                        "mouseup",
                        true,
                        [
                            ("button", JsValue::from_f64(0.)),
                            ("clientX", JsValue::from_f64(13.)),
                            ("clientY", JsValue::from_f64(35.)),
                        ],
                    );
                })
                .await;
            });
        }
    }

    impl MyComp {
        fn handler(this: ComponentRc<Self>, ev: &mut TapEvent) {
            assert_eq!(ev.client_x(), 12);
            assert_eq!(ev.client_y(), 34);
            async_task(async move {
                this.update_with(|this, _| {
                    (this.callback.take().unwrap())();
                })
                .await
                .unwrap();
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
async fn cancel_tap() {
    #[component(Backend = DomBackend)]
    struct MyComp {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div cancel_tap=@handler()></div>
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
                    let dom_elem = first_dom!(this, div).clone();
                    simulate_event(
                        &dom_elem,
                        "mousedown",
                        true,
                        [
                            ("button", JsValue::from_f64(0.)),
                            ("clientX", JsValue::from_f64(12.)),
                            ("clientY", JsValue::from_f64(34.)),
                        ],
                    );
                    simulate_event(
                        &web_sys::window().unwrap().document().unwrap(),
                        "mouseup",
                        true,
                        [
                            ("button", JsValue::from_f64(0.)),
                            ("clientX", JsValue::from_f64(23.)),
                            ("clientY", JsValue::from_f64(45.)),
                        ],
                    );
                })
                .await;
            });
        }
    }

    impl MyComp {
        fn handler(this: ComponentRc<Self>, ev: &mut TapEvent) {
            assert_eq!(ev.client_x(), 12);
            assert_eq!(ev.client_y(), 34);
            async_task(async move {
                this.update_with(|this, _| {
                    (this.callback.take().unwrap())();
                })
                .await
                .unwrap();
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
async fn long_tap() {
    #[component(Backend = DomBackend)]
    struct MyComp {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div long_tap=@handler() tap=@should_panic()></div>
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
                    let dom_elem = first_dom!(this, div).clone();
                    simulate_event(
                        &dom_elem,
                        "mousedown",
                        true,
                        [
                            ("button", JsValue::from_f64(0.)),
                            ("clientX", JsValue::from_f64(12.)),
                            ("clientY", JsValue::from_f64(34.)),
                        ],
                    );
                })
                .await;
            });
        }
    }

    impl MyComp {
        fn handler(this: ComponentRc<Self>, ev: &mut TapEvent) {
            assert_eq!(ev.client_x(), 12);
            assert_eq!(ev.client_y(), 34);
            ev.prevent_default();
            async_task(async move {
                simulate_event(
                    &web_sys::window().unwrap().document().unwrap(),
                    "mouseup",
                    true,
                    [
                        ("button", JsValue::from_f64(0.)),
                        ("clientX", JsValue::from_f64(12.)),
                        ("clientY", JsValue::from_f64(34.)),
                    ],
                );
                async_task(async move {
                    this.update_with(|this, _| {
                        (this.callback.take().unwrap())();
                    })
                    .await
                    .unwrap();
                });
            });
        }

        fn should_panic(_this: ComponentRc<Self>, _ev: &mut TapEvent) {
            panic!();
        }
    }

    impl ComponentTest for MyComp {
        fn set_callback(&mut self, callback: ComponentTestCb) {
            self.callback = Some(callback);
        }
    }

    test_component::<MyComp>().await;
}
