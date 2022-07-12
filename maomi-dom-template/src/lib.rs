// import WASM support
use wasm_bindgen::prelude::*;
// import maomi core module
use maomi::{prelude::*, BackendContext};
// using DOM backend
use maomi_backend_dom::{DomBackend, element::*};

// declare a component
#[component(for DomBackend)]
struct HelloWorld {
    // a component should have a template field
    template: template! {
        // the template is XML-like
        <div>
            // strings in the template must be quoted
            "Hello world!"
        </div>
        <div>
            // use { ... } bindings in the template
            { &self.hello }
        </div>
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
    backend_context.enter_sync(move |ctx| {
        let mut mount_point = ctx.new_mount_point(|_: &mut HelloWorld| Ok(())).unwrap();
        mount_point.append_attach(&mut ctx.root_mut());
    }).map_err(|_| "Cannot init mount point").unwrap();
}
