use wasm_bindgen_test::*;

use maomi::prelude::*;
use maomi_backend_dom::element::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn dom_html(
    e: &mut maomi::backend::tree::ForestNodeMut<maomi_backend_dom::DomGeneralElement>,
) -> String {
    <maomi_backend_dom::DomBackend as maomi::backend::Backend>::GeneralElement::inner_html(
        &e.as_ref(),
    )
}

fn test_component<T: maomi::backend::SupportBackend<maomi_backend_dom::DomBackend>>(
    expected_html: &str,
) {
    use maomi::backend::Backend;
    let mut dom_backend = maomi_backend_dom::DomBackend::new();
    let elem = {
        let mut parent = dom_backend.root_mut();
        let (_, elem) =
            <T as maomi::backend::SupportBackend<maomi_backend_dom::DomBackend>>::create(&mut parent, |_| Ok(()))
                .unwrap();
        elem
    };
    assert_eq!(dom_html(&mut elem.borrow_mut()), expected_html);
}

#[wasm_bindgen_test]
fn basic() {
    #[component(for maomi_backend_dom::DomBackend)]
    struct HelloWorld {
        template: template! {
            <div>"Hello world!"</div>
            <div>{ &self.hello_text }</div>
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
            self.hello_text = "Hello world!".into();
            self.template.mark_dirty();
        }
    }

    test_component::<HelloWorld>("<div>Hello world!</div><div>Hello world!</div>");
}
