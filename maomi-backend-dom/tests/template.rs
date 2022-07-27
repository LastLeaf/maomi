use wasm_bindgen_test::*;

use maomi::prelude::*;
use maomi_backend_dom::element::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn test_component<T: 'static + Component + maomi::component::ComponentTemplate<maomi_backend_dom::DomBackend>>(
    expected_html: &str,
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
            let mut mount_point = ctx.new_mount_point(|_: &mut T| Ok(())).unwrap();
            mount_point.append_attach(&mut ctx.root_mut());
        })
        .map_err(|_| "Cannot init mount point")
        .unwrap();
    backend_context
        .enter_sync(move |ctx| {
            let html = maomi_backend_dom::DomGeneralElement::inner_html(&ctx.root());
            assert_eq!(html, expected_html);
        })
        .map_err(|_| "Cannot get mount point")
        .unwrap();
}

#[wasm_bindgen_test]
fn basic() {
    #[component(for maomi_backend_dom::DomBackend)]
    struct HelloWorld {
        template: template! {
            <div title="Hello">"Hello world!"</div>
            <div title="Again">{ &self.hello_text }</div>
        },
        hello_text: String,
    }

    impl Component for HelloWorld {
        fn new() -> Self {
            Self {
                template: Default::default(),
                hello_text: "".into(),
            }
        }

        fn created(&mut self) {
            self.hello_text = "Hello world again!".into();
            self.mark_dirty();
        }
    }

    test_component::<HelloWorld>("<div>Hello world!</div><div>Hello world again!</div>");
}
