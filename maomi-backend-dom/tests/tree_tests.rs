wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

mod tree;

fn prepare_env(
    f: impl FnOnce(&mut maomi::backend::context::EnteredBackendContext<maomi_backend_dom::DomBackend>),
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
        .enter_sync(move |ctx| f(ctx))
        .map_err(|_| "Cannot init mount point")
        .unwrap();
}
