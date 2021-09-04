use maomi::prelude::*;
use maomi::{backend::Dom, Context};
use wasm_bindgen::prelude::*;

// declare the template for a component (XML-like syntax)
template!(xml<B: Backend> for<B> EventHandling<B> {
    <input
        r#type="button"
        value={ self.title }
        @click={ |s, _| s.tap() }
    ></input>
});

// list the data used in this component
struct EventHandling<B: Backend> {
    ctx: ComponentContext<B, Self>,
    title: &'static str,
}

// provide some methods
impl<B: Backend> EventHandling<B> {
    fn tap(&mut self) {
        self.title = "You got it!";
        self.ctx.update();
    }
}

// provide life-time methods for this component
impl<B: Backend> Component<B> for EventHandling<B> {
    // create a new component instance (required)
    fn new(ctx: ComponentContext<B, Self>) -> Self {
        Self {
            ctx,
            title: "Click me!",
        }
    }
}

#[wasm_bindgen(start)]
pub fn wasm_main() {
    init_panic_hook();
    // init in page
    thread_local! {
        static CONTEXT: Context<Dom> = {
            let mut context = Context::new(Dom::new("placeholder"));
            let root_component = context.new_root_component::<EventHandling<_>>();
            context.set_root_component(root_component);
            context
        };
    }
    CONTEXT.with(|_| {
        // empty
    });
}

fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
