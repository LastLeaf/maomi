use wasm_bindgen_test::*;

use maomi::{prelude::*, diff::key::AsListKey};
use maomi_backend_dom::{element::*, DomBackend, async_task};

mod env;
use env::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn child_component() {
    #[component(for DomBackend)]
    struct Child {
        template: template! {
            <div title={ &*self.title }> { &self.text } </div>
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
                <Child title={ &self.hello_title } text=&{ self.hello_text } />
            </div>
        },
        hello_text: String,
        hello_title: String,
    }

    impl Component for Parent {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
                hello_text: "".into(),
                hello_title: "Again".into(),
            }
        }

        fn created(&self) {
            let this = self.rc();
            async_task(async move {
                this.update(|this| {
                    assert_eq!(
                        this.template_structure().unwrap().0.tag.dom_element().outer_html(),
                        r#"<div><div title="Again"></div></div>"#,
                    );
                    this.hello_text = "Hello world again!".into();
                }).await.unwrap();
                this.get_mut(|this| {
                    assert_eq!(
                        this.template_structure().unwrap().0.tag.dom_element().outer_html(),
                        r#"<div><div title="Again">Hello world again!</div></div>"#,
                    );
                    (this.callback.take().unwrap())();
                }).await.unwrap();
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
                        this.template_structure().unwrap().0.tag.dom_element().inner_html(),
                        r#"<div>(empty)</div>"#,
                    );
                    this.text = "hello".into();
                }).await.unwrap();
                this.get_mut(|this| {
                    assert_eq!(
                        this.template_structure().unwrap().0.tag.dom_element().inner_html(),
                        r#"<div>hello</div>"#,
                    );
                    this.text = "long........".into();
                    this.schedule_update();
                }).await.unwrap();
                this.get_mut(|this| {
                    assert_eq!(
                        this.template_structure().unwrap().0.tag.dom_element().inner_html(),
                        r#"<div>(too long)</div>"#,
                    );
                    (this.callback.take().unwrap())();
                }).await.unwrap();
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
                        this.template_structure().unwrap().0.tag.dom_element().inner_html(),
                        r#""#,
                    );
                    this.text = "hello".into();
                }).await.unwrap();
                this.update(|this| {
                    assert_eq!(
                        this.template_structure().unwrap().0.tag.dom_element().inner_html(),
                        r#"<div>hello</div>"#,
                    );
                    this.text = "".into();
                }).await.unwrap();
                this.get_mut(|this| {
                    assert_eq!(
                        this.template_structure().unwrap().0.tag.dom_element().inner_html(),
                        r#""#,
                    );
                    (this.callback.take().unwrap())();
                }).await.unwrap();
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
                        this.template_structure().unwrap().0.tag.dom_element().inner_html(),
                        r#"<div>(empty)</div>"#,
                    );
                    this.text = "hello".into();
                }).await.unwrap();
                this.get_mut(|this| {
                    assert_eq!(
                        this.template_structure().unwrap().0.tag.dom_element().inner_html(),
                        r#"<div>hello</div>"#,
                    );
                    this.text = "long........".into();
                    this.schedule_update();
                }).await.unwrap();
                this.get_mut(|this| {
                    assert_eq!(
                        this.template_structure().unwrap().0.tag.dom_element().inner_html(),
                        r#"<div>(too long)</div>"#,
                    );
                    (this.callback.take().unwrap())();
                }).await.unwrap();
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
    struct MyList(usize);

    impl AsListKey for MyList {
        type ListKey = str;

        fn as_list_key(&self) -> &str {
            "test"
        }
    }

    #[component(for DomBackend)]
    struct Parent {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div>
                for (index, item) in self.list.iter().enumerate() use(item) String {
                    <div> { &index.to_string() } </div>
                }
                for item in self.list.iter() use usize {
                    <div> { &item.0.to_string() } </div>
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
                list: vec![
                    MyList(123),
                    MyList(456),
                ],
            }
        }

        fn created(&self) {
            let this = self.rc();
            async_task(async move {
                this.update(|this| {
                    assert_eq!(
                        this.template_structure().unwrap().0.tag.dom_element().inner_html(),
                        r#"<div title="0">123</div><div title="1">456</div>"#,
                    );
                    this.list.push(MyList(789));
                }).await.unwrap();
                this.update(|this| {
                    assert_eq!(
                        this.template_structure().unwrap().0.tag.dom_element().inner_html(),
                        r#"<div title="0">123</div><div title="1">456</div><div title="2">789</div>"#,
                    );
                    this.list.pop();
                    this.list.pop();
                }).await.unwrap();
                this.update(|this| {
                    assert_eq!(
                        this.template_structure().unwrap().0.tag.dom_element().inner_html(),
                        r#"<div title="0">123</div><div title="1">456</div><div title="2">789</div>"#,
                    );
                    this.list.pop();
                    this.list.pop();
                }).await.unwrap();
                this.get_mut(|this| {
                    assert_eq!(
                        this.template_structure().unwrap().0.tag.dom_element().inner_html(),
                        r#""#,
                    );
                    (this.callback.take().unwrap())();
                }).await.unwrap();
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
