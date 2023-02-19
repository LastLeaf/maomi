use wasm_bindgen_test::*;

use maomi::prelude::*;
use maomi_dom::{async_task, element::*, prelude::*};

use super::*;

#[wasm_bindgen_test]
async fn template_if_else() {
    #[component(Backend = DomBackend)]
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
                this.update_with(|this, ctx| {
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
                    ctx.need_update();
                })
                .await
                .unwrap();
                this.update_with(|this, _| {
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
    #[component(Backend = DomBackend)]
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
                this.update_with(|this, _| {
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
    #[component(Backend = DomBackend)]
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
                this.update_with(|this, ctx| {
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
                    ctx.need_update();
                })
                .await
                .unwrap();
                this.update_with(|this, _| {
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
    #[component(Backend = DomBackend)]
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
                this.update_with(|this, _| {
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

    #[component(Backend = DomBackend)]
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

    #[component(Backend = DomBackend)]
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
                        Vec::<usize>::new(),
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
                        Vec::<usize>::new(),
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
                this.update_with(|this, _| {
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
                        Vec::<usize>::new(),
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
async fn class_attr() {
    stylesheet! {
        #[css_name("static-class")]
        class static_class {
            color = red;
        }
        #[css_name("dyn-class")]
        class dyn_class {
            color = blue;
        }
    }

    #[component(Backend = DomBackend)]
    struct Parent {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div class:static_class class:dyn_class={&self.v} />
        },
        v: bool,
    }

    impl Component for Parent {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
                v: false,
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
                        r#"<div class="static-class"></div>"#,
                    );
                    this.v = true;
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
                            .outer_html(),
                        r#"<div class="static-class dyn-class"></div>"#,
                    );
                    this.v = false;
                })
                .await
                .unwrap();
                this.update_with(|this, _| {
                    assert_eq!(
                        this.template_structure()
                            .unwrap()
                            .0
                            .tag
                            .dom_element()
                            .outer_html(),
                        r#"<div class="static-class"></div>"#,
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
async fn style_attr() {
    stylesheet! {
        style color(rgb: f32) {
            color = rgb(rgb, rgb, rgb);
        }
    }

    #[component(Backend = DomBackend)]
    struct Parent {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div style:color=&{ self.color } />
        },
        color: i32,
    }

    impl Component for Parent {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
                color: 64,
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
                        r#"<div style="color: rgb(64, 64, 64);"></div>"#,
                    );
                    this.color = 128;
                })
                .await
                .unwrap();
                this.update_with(|this, _| {
                    assert_eq!(
                        this.template_structure()
                            .unwrap()
                            .0
                            .tag
                            .dom_element()
                            .outer_html(),
                        r#"<div style="color: rgb(128, 128, 128);"></div>"#,
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
async fn event_handler() {
    struct MyEventDetail {
        num: Option<u32>,
    }

    #[component(Backend = DomBackend)]
    struct Child {
        template: template! {
            { &self.my_prop }
        },
        my_prop: Prop<String>,
        my_event: Event<MyEventDetail>,
    }

    impl Component for Child {
        fn new() -> Self {
            Self {
                template: Default::default(),
                my_prop: Prop::new("".into()),
                my_event: Event::new(),
            }
        }

        fn before_template_apply(&mut self) {
            self.my_event.trigger(&mut MyEventDetail {
                num: self.my_prop.parse().ok(),
            });
        }
    }

    #[component(Backend = DomBackend)]
    struct Parent {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div>
                for item in &self.list {
                    <Child my_prop={item} my_event=@my_event_handler(item)>
                        <slot />
                    </Child>
                }
            </div>
        },
        list: Vec<String>,
    }

    impl Parent {
        fn my_event_handler(this: ComponentRc<Self>, e: &mut MyEventDetail, item: &str) {
            let num = e.num.unwrap_or(0);
            assert_eq!(num.to_string().as_str(), item);
            async_task(async move {
                this.update(move |this| {
                    if num <= 300 {
                        this.list = vec![(num + 100).to_string()];
                    } else {
                        assert_eq!(
                            this.template_structure()
                                .unwrap()
                                .0
                                .tag
                                .dom_element()
                                .outer_html(),
                            r#"<div>400</div>"#,
                        );
                        (this.callback.take().unwrap())();
                    }
                })
                .await
                .unwrap();
            });
        }
    }

    impl Component for Parent {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
                list: vec![100.to_string()],
            }
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
async fn binding_prop() {
    use maomi::prop::{BindingProp, BindingValue};
    use maomi_dom::event::*;

    #[component(Backend = DomBackend)]
    struct Child {
        template: template! {
            <input value={ &self.input_value } change=@input_change() />
        },
        input_value: BindingValue<String>,
        has_input_value: BindingProp<bool>,
        change: Event<()>,
    }

    impl Component for Child {
        fn new() -> Self {
            Self {
                template: Default::default(),
                input_value: BindingValue::new(String::with_capacity(0)),
                has_input_value: BindingProp::new(false),
                change: Event::new(),
            }
        }

        fn before_template_apply(&mut self) {
            self.update_has_input_value();
        }

        fn created(&self) {
            let this = self.rc();
            this.task_with(|this, _| {
                let dom_elem = this.template_structure().unwrap().0.tag.dom_element();
                dom_elem.dyn_ref::<web_sys::HtmlInputElement>().unwrap().set_value("abc");
                simulate_event(
                    dom_elem,
                    "input",
                    false,
                    [],
                );
                simulate_event(
                    dom_elem,
                    "change",
                    false,
                    [],
                );
            });
        }
    }

    impl Child {
        fn update_has_input_value(&mut self) {
            self.has_input_value.set(self.input_value.get().len() > 0);
        }

        fn input_change(this: ComponentRc<Self>, _: &mut ChangeEvent) {
            this.task(|this| {
                this.change.trigger(&mut ());
            });
        }
    }

    #[component(Backend = DomBackend)]
    struct Parent {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div>
                <Child has_input_value={ &self.has_input_value } change=@child_change() />
            </div>
        },
        has_input_value: BindingValue<bool>,
    }

    impl Component for Parent {
        fn new() -> Self {
            Self {
                callback: None,
                template: Default::default(),
                has_input_value: BindingValue::new(false),
            }
        }
    }

    impl Parent {
        fn child_change(this: ComponentRc<Self>, _: &mut ()) {
            this.task_with(|this, _| {
                assert_eq!(this.has_input_value.get(), true);
                (this.callback.take().unwrap())();
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
async fn list_prop() {
    use maomi::prop::ListProp;

    #[component(Backend = DomBackend)]
    struct Child {
        template: template! {
            for item in &self.my_list_prop {
                <div> { item } </div>
            }
        },
        my_list_prop: ListProp<String>,
    }

    impl Component for Child {
        fn new() -> Self {
            Self {
                template: Default::default(),
                my_list_prop: ListProp::new(),
            }
        }
    }

    #[component(Backend = DomBackend)]
    struct Parent {
        callback: Option<ComponentTestCb>,
        template: template! {
            <div>
                <Child my_list_prop:String="abc" my_list_prop:String="def" />
                <Child my_list_prop=&{ ["ghi".to_string()] } />
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
            this.task_with(|this, _| {
                assert_eq!(
                    this.template_structure()
                        .unwrap()
                        .0
                        .tag
                        .dom_element()
                        .inner_html(),
                    r#"<div>abc</div><div>def</div><div>ghi</div>"#,
                );
                (this.callback.take().unwrap())();
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
