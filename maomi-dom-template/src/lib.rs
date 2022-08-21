// import WASM support
use wasm_bindgen::prelude::*;
// import maomi core module
use maomi::{prelude::*, BackendContext};
// using DOM backend
use maomi_dom::{async_task, element::*, event::*, prelude::dom_css, DomBackend};

// write limited CSS
dom_css!(
    // only single class selectors are allowed
    .warn {
        color: orange;
        font-size: 16px;
    }
);

// declare a component
#[component(Backend = DomBackend)]
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
        // bind event with `@xxx()`
        if !self.r {
            <div long_tap=@handle_tap()> "Click me!" </div>
        }
    },
    hello: String,
    r: bool,
}

// implement basic component interfaces
impl Component for HelloWorld {
    fn new() -> Self {
        Self {
            template: Default::default(),
            hello: "Hello world again!".to_string(),
            r: false,
        }
    }
}

impl HelloWorld {
    // an event handler
    fn handle_tap(this: ComponentRc<Self>, _detail: &mut TapEvent) {
        log::info!("Clicked!");
        async_task(async move {
            this.update(|this| this.r = true).await.unwrap();
        });
    }
}

#[wasm_bindgen(start)]
pub fn wasm_main() {
    // init logger
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Trace).unwrap();

    // init a backend context
    let dom_backend = DomBackend::new_with_document_body().unwrap();
    let backend_context = BackendContext::new(dom_backend);

    // create a mount point
    backend_context
        .enter_sync(move |ctx| {
            let mount_point = ctx.append_attach(|_: &mut HelloWorld| {}).unwrap();
            // leak the mount point, so that event callbacks still work
            Box::leak(Box::new(mount_point));
        })
        .map_err(|_| "Cannot init mount point")
        .unwrap();

    // leak the backend context, so that event callbacks still work
    Box::leak(Box::new(backend_context));
}
