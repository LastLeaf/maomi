use wasm_bindgen_test::*;

use maomi::prelude::*;

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
    let mut parent = dom_backend.root_mut();
    let (_, mut elem) =
        <T as maomi::backend::SupportBackend<maomi_backend_dom::DomBackend>>::create(&mut parent)
            .unwrap();
    assert_eq!(dom_html(&mut elem.as_node_mut()), expected_html);
}

#[wasm_bindgen_test]
fn basic() {
    #[component(maomi_backend_dom::DomBackend)]
    struct HelloWorld {
        template: template! {
            <div />
            <div>{{ self.hello_text }}</div>
        },
        need_update: bool,
        hello_text: String,
        tap_hello: (),
    }

    test_component::<HelloWorld>("<div><div></div>Hello world!</div>");
}
