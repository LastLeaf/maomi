// import WASM support
use wasm_bindgen::prelude::*;
// import maomi core module
use maomi::{prelude::*, BackendContext, locale_string::{LocaleString, ToLocaleStr}};
// using DOM backend
use maomi_dom::{element::*, event::*, prelude::*, DomBackend};

stylesheet! {
    use crate::*;

    // declare a class
    class warn {
        color = orange;
    }

    // declare a dynamic style
    style opacity(alpha: f32) {
        opacity = alpha;
    }
}

stylesheet! {
    class error {}
}

#[component(Backend = DomBackend)]
struct MyComponent {
    template: template! {
        <a href="/"></a>
        <div class:error></div>
    },
}


// declare a component
#[component(Backend = DomBackend)]
struct HelloWorld {
    // a component should have a template field
    template: template! {
        // the template is XML-like
        <div title="Hello!">
            // text in the template must be quoted
            "Hello world!"
        </div>
        // use { ... } bindings in the template
        <div title={ self.hello.to_locale_str() }>
            { &self.hello }
        </div>
        // use classes in `class:xxx` form
        <div class:warn> "WARN" </div>
        // use dynamic style in `style:xxx` form
        <div style:opacity=0.5> "transparent text" </div>
        // bind event with `@xxx()`
        if !self.clicked {
            <div tap=@handle_tap()> "Click me!" </div>
        }
    },
    hello: LocaleString,
    clicked: bool,
}

// implement basic component interfaces
impl Component for HelloWorld {
    fn new() -> Self {
        Self {
            template: Default::default(),
            hello: i18n!("Hello world again!").to_locale_string(),
            clicked: false,
        }
    }
}

impl HelloWorld {
    // an event handler
    fn handle_tap(this: ComponentRc<Self>, _detail: &mut TapEvent) {
        log::info!("Clicked!");
        this.task(|this| {
            this.clicked = true;
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
    let mount_point = backend_context
        .enter_sync(move |ctx| {
            ctx.attach(|_: &mut HelloWorld| {})
        })
        .map_err(|_| "Cannot init mount point")
        .unwrap();

    // leak the backend context, so that event callbacks still work
    std::mem::forget(mount_point);
    std::mem::forget(backend_context);
}
