use wasm_bindgen_test::*;

use maomi::{prelude::*, AsyncCallback};
use maomi_backend_dom::{element::*, DomBackend, async_task};

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

async fn test_component<
    T: 'static + Component + maomi::template::ComponentTemplate<maomi_backend_dom::DomBackend>,
>(
    expected_html: &'static str,
) {
    let elem = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("div")
        .unwrap();
    let dom_backend = maomi_backend_dom::DomBackend::new_with_element(elem).unwrap();
    let backend_context = maomi::BackendContext::new(dom_backend);
    backend_context
        .enter_sync(move |ctx| {
            let _mount_point = ctx.append_attach(|_: &mut T| {}).unwrap();
        })
        .map_err(|_| "Cannot init mount point")
        .unwrap();
    let (fut, cb) = AsyncCallback::new();
    async_task(async move {
        backend_context
            .enter_sync(move |ctx| {
                let html = maomi_backend_dom::DomGeneralElement::inner_html(&ctx.root());
                assert_eq!(html, expected_html);
            })
            .map_err(|_| "Cannot get mount point")
            .unwrap();
        cb(());
    });
    fut.await
}

#[wasm_bindgen_test]
async fn basic() {
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
    struct HelloWorld {
        template: template! {
            <div title="Hello"> "Hello world!" </div>
            <Child title={ &self.hello_title } text=&{ self.hello_text } />
        },
        hello_text: String,
        hello_title: String,
    }

    impl Component for HelloWorld {
        fn new() -> Self {
            Self {
                template: Default::default(),
                hello_text: "".into(),
                hello_title: "Again".into(),
            }
        }

        fn created(&self) {
            let this = self.rc();
            async_task(async move {
                this.update(|this| {
                    this.hello_text = "Hello world again!".into();
                }).await.unwrap();
            });
        }
    }

    test_component::<HelloWorld>(
        r#"<div title="Hello">Hello world!</div><div title="Again">Hello world again!</div>"#,
    ).await;
}

#[wasm_bindgen_test]
async fn template_if() {
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
        template: template! {
            if text.len() > 10 {
                <div> "(too long)" </div>
            } else if text.len() == 0 {
                <div> "(empty)" </div>
            } else {
                <div> { self.text } </div>
            }
        },
        text: String,
    }

    impl Component for Parent {
        fn new() -> Self {
            Self {
                template: Default::default(),
                text: "".into(),
            }
        }

        fn created(&self) {
            let this = self.rc();
            async_task(async move {
                this.update(|this| {
                    this.hello_text = "Hello world again!".into();
                }).await.unwrap();
            });
        }
    }

    test_component::<Parent>(
        r#"<div title="Hello">Hello world!</div><div title="Again">Hello world again!</div>"#,
    ).await;
}
