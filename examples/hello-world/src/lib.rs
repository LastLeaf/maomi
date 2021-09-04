#[macro_use]
extern crate log;
use maomi::prelude::*;
use maomi::{backend::Dom, Context};
use wasm_bindgen::prelude::*;

// declare the template for a component (XML-like syntax)
template!(xml for HelloWorld {
    // comments is C-like syntax
    <div style={ format!("color: {}", self.color) }> // {...} accepts a rust expression
        "Hello world!" // unlike XML, plain text should be wrapped in quotes!
    </div>
});

// list the data used in this component
struct HelloWorld {
    color: &'static str,
}

// provide life-time methods for this component
impl<B: Backend> Component<B> for HelloWorld {
    // create a new component instance (required)
    fn new(_ctx: ComponentContext<B, Self>) -> Self {
        Self { color: "red" }
    }
}

#[wasm_bindgen(start)]
pub fn wasm_main() {
    init_panic_hook();
    console_log::init_with_level(log::Level::Debug).unwrap();
    // init in page
    thread_local! {
        static CONTEXT: Context<Dom> = {
            let mut context = Context::new(Dom::new("placeholder"));
            let root_component = context.new_root_component::<HelloWorld>();
            context.set_root_component(root_component);
            context
        };
    }
    CONTEXT.with(|_| {
        info!("Hello world!");
    });
}

fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
