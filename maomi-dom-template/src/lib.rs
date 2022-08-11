// import WASM support
use wasm_bindgen::prelude::*;
// import maomi core module
use maomi::{prelude::*, BackendContext};
// using DOM backend
use maomi_dom::{element::*, DomBackend, prelude::dom_css};

// write limited CSS
dom_css!(
    // only single class selectors are allowed
    .warn {
        color: orange;
        font-size: 16px;
    }
);

// declare a component
#[component(for DomBackend)]
struct HelloWorld {
    // a component should have a template field
    template: template! {
        // the template is XML-like
        <div title="Hello!">
            // strings in the template must be quoted
            "Hello world!"
        </div>
        // use { ... } bindings in the template
        <div title={ &self.hello }>
            { &self.hello }
        </div>
        // use classes in `class:xxx` form
        <div class:warn> "WARN" </div>
    },
    hello: String,
}

// implement basic component interfaces
impl Component for HelloWorld {
    fn new() -> Self {
        Self {
            template: Default::default(),
            hello: "Hello world again!".to_string(),
        }
    }
}

#[wasm_bindgen(start)]
pub fn wasm_main() {
    // init a backend context
    let dom_backend = DomBackend::new_with_document_body().unwrap();
    let backend_context = BackendContext::new(dom_backend);

    // create a mount point
    backend_context
        .enter_sync(move |ctx| {
            let _mount_point = ctx.append_attach(|_: &mut HelloWorld| {}).unwrap();
        })
        .map_err(|_| "Cannot init mount point")
        .unwrap();
}
