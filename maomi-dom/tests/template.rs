use wasm_bindgen_test::*;

use maomi::prelude::*;
use maomi_dom::{async_task, element::*, prelude::*};

mod env;
use env::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn template_if_else() {
    #[component(for DomBackend)]
    struct Parent {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div>
                if self.text.len() > 10 {
                    <div> "(too long)" </div>
                } else if self.text.len() == 0 {
                    <div> "(empty)" </div>
                } else {
                    <div> { &self.text } </div>
                }
            </div>
        },
        text: String,
    }

    impl Component for Parent {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
                text: "".into(),
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
                        r#"<div>(empty)</div>"#,
                    );
                    this.text = "hello".into();
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
                        r#"<div>hello</div>"#,
                    );
                    this.text = "long........".into();
                    this.schedule_update();
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
                        r#"<div>(too long)</div>"#,
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
async fn template_lonely_if() {
    #[component(for DomBackend)]
    struct Parent {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div>
                if self.text.len() > 0 {
                    <div> { &self.text } </div>
                }
            </div>
        },
        text: String,
    }

    impl Component for Parent {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
                text: "".into(),
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
                    this.text = "hello".into();
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
                        r#"<div>hello</div>"#,
                    );
                    this.text = "".into();
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
                        r#""#,
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
async fn template_match() {
    #[component(for DomBackend)]
    struct Parent {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div>
                match self.text.len() {
                    11.. => {
                        <div> "(too long)" </div>
                    },
                    0 => {
                        <div> "(empty)" </div>
                    }
                    _ => {
                        <div> { &self.text } </div>
                    }
                }
            </div>
        },
        text: String,
    }

    impl Component for Parent {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
                text: "".into(),
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
                        r#"<div>(empty)</div>"#,
                    );
                    this.text = "hello".into();
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
                        r#"<div>hello</div>"#,
                    );
                    this.text = "long........".into();
                    this.schedule_update();
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
                        r#"<div>(too long)</div>"#,
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
async fn template_for_keyless() {
    #[component(for DomBackend)]
    struct Parent {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div>
                for (index, _) in self.list.iter().enumerate() {
                    <div> { &index.to_string() } </div>
                }
                for item in self.list.iter() {
                    <div> { &item.to_string() } </div>
                }
            </div>
        },
        list: Vec<usize>,
    }

    impl Component for Parent {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
                list: vec![123, 456],
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
                        r#"<div>0</div><div>1</div><div>123</div><div>456</div>"#,
                    );
                    this.list.push(789);
                })
                .await
                .unwrap();
                this.update(|this| {
                    assert_eq!(
                        this.template_structure().unwrap().0.tag.dom_element().inner_html(),
                        r#"<div>0</div><div>1</div><div>2</div><div>123</div><div>456</div><div>789</div>"#,
                    );
                    this.list.pop();
                    this.list.pop();
                }).await.unwrap();
                this.update(|this| {
                    assert_eq!(
                        this.template_structure()
                            .unwrap()
                            .0
                            .tag
                            .dom_element()
                            .inner_html(),
                        r#"<div>0</div><div>123</div>"#,
                    );
                    this.list.pop();
                    this.list.pop();
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
                        r#""#,
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
async fn template_for() {
    use std::cell::RefCell;

    struct MyList(usize);

    impl AsListKey for MyList {
        type ListKey = usize;

        fn as_list_key(&self) -> &usize {
            &self.0
        }
    }

    thread_local! {
        static EV_LIST: RefCell<Vec<usize>> = RefCell::new(vec![]);
    }

    #[component(for DomBackend)]
    struct Child {
        template: template! {
            { &self.num.to_string() }
        },
        num: Prop<usize>,
    }

    impl Component for Child {
        fn new() -> Self {
            Self {
                template: Default::default(),
                num: Prop::new(0),
            }
        }

        fn created(&self) {
            EV_LIST.with(|ev_list| ev_list.borrow_mut().push(*self.num));
        }
    }

    #[component(for DomBackend)]
    struct Parent {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div>
                for (index, item) in self.list.iter().enumerate() use (item) usize {
                    <div> { &index.to_string() } </div>
                }
                for item in self.list.iter() use usize {
                    <Child num={ &item.0 } />
                }
            </div>
        },
        list: Vec<MyList>,
    }

    impl Component for Parent {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
                list: vec![MyList(12), MyList(34)],
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
                        r#"<div>0</div><div>1</div>1234"#,
                    );
                    assert_eq!(
                        EV_LIST.with(|ev_list| ev_list.borrow_mut().drain(..).collect::<Vec<_>>()),
                        vec![12, 34],
                    );
                    this.list = vec![MyList(78), MyList(12), MyList(56), MyList(34), MyList(90)];
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
                        r#"<div>0</div><div>1</div><div>2</div><div>3</div><div>4</div>7812563490"#,
                    );
                    assert_eq!(
                        EV_LIST.with(|ev_list| ev_list.borrow_mut().drain(..).collect::<Vec<_>>()),
                        vec![78, 56, 90],
                    );
                    this.list = vec![MyList(12), MyList(90), MyList(56), MyList(78), MyList(34)];
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
                        r#"<div>0</div><div>1</div><div>2</div><div>3</div><div>4</div>1290567834"#,
                    );
                    assert_eq!(
                        EV_LIST.with(|ev_list| ev_list.borrow_mut().drain(..).collect::<Vec<_>>()),
                        vec![],
                    );
                    this.list = vec![MyList(78), MyList(12), MyList(56), MyList(34), MyList(90)];
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
                        r#"<div>0</div><div>1</div><div>2</div><div>3</div><div>4</div>7812563490"#,
                    );
                    assert_eq!(
                        EV_LIST.with(|ev_list| ev_list.borrow_mut().drain(..).collect::<Vec<_>>()),
                        vec![],
                    );
                    this.list = vec![MyList(12), MyList(67), MyList(34)];
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
                        r#"<div>0</div><div>1</div><div>2</div>126734"#,
                    );
                    assert_eq!(
                        EV_LIST.with(|ev_list| ev_list.borrow_mut().drain(..).collect::<Vec<_>>()),
                        vec![67],
                    );
                    this.list = vec![];
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
                        r#""#,
                    );
                    assert_eq!(
                        EV_LIST.with(|ev_list| ev_list.borrow_mut().drain(..).collect::<Vec<_>>()),
                        vec![],
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

// #[wasm_bindgen_test]
// async fn class_attr() {
//     dom_css! {
//         .static-class {
//             color: red;
//         }
//         .dyn_class {
//             color: blue;
//         }
//     }

//     #[component(for DomBackend)]
//     struct Parent {
//         callback: Option<ComponentTestCb>,
//         template: template! {
//             <div class:static_class class:dyn_class={&self.v} />
//         },
//         v: bool,
//     }

//     impl Component for Parent {
//         fn new() -> Self {
//             Self {
//                 callback: None,
//                 template: Default::default(),
//                 v: false,
//             }
//         }

//         fn created(&self) {
//             let this = self.rc();
//             async_task(async move {
//                 this.update(|this| {
//                     assert_eq!(
//                         this.template_structure()
//                             .unwrap()
//                             .0
//                             .tag
//                             .dom_element()
//                             .outer_html(),
//                         r#"<div class="static-class"></div>"#,
//                     );
//                     this.v = true;
//                 })
//                 .await
//                 .unwrap();
//                 this.update(|this| {
//                     assert_eq!(
//                         this.template_structure()
//                             .unwrap()
//                             .0
//                             .tag
//                             .dom_element()
//                             .outer_html(),
//                         r#"<div class="static-class dyn_class"></div>"#,
//                     );
//                     this.v = false;
//                 })
//                 .await
//                 .unwrap();
//                 this.get_mut(|this| {
//                     assert_eq!(
//                         this.template_structure()
//                             .unwrap()
//                             .0
//                             .tag
//                             .dom_element()
//                             .outer_html(),
//                         r#"<div class="static-class"></div>"#,
//                     );
//                     (this.callback.take().unwrap())();
//                 })
//                 .await
//                 .unwrap();
//             });
//         }
//     }

//     impl ComponentTest for Parent {
//         fn set_callback(&mut self, callback: ComponentTestCb) {
//             self.callback = Some(callback);
//         }
//     }

//     test_component::<Parent>().await;
// }

#[wasm_bindgen_test]
async fn style_attr() {
    #[component(for DomBackend)]
    struct Parent {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div style={&format!("color: {};", self.color)} />
        },
        color: String,
    }

    impl Component for Parent {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
                color: "red".into(),
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
                        r#"<div style="color: red;"></div>"#,
                    );
                    this.color = "blue".into();
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
                        r#"<div style="color: blue;"></div>"#,
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
