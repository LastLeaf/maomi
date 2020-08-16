#[macro_use] extern crate log;
use wasm_bindgen::prelude::*;
use maomi::prelude::*;
use maomi::{Context, backend::Dom};

template!(xml for View {
    <slot />
});

struct View {
    // empty
}

impl<B: Backend> Component<B> for View {
    fn new(_ctx: ComponentContext<B, Self>) -> Self {
        Self { }
    }
}

template!(xml for BatchCreation {
    <for i in { 0..self.count }>
        <View>{i}</View>
    </for>
});

struct BatchCreation {
    count: u32
}

impl<B: Backend> Component<B> for BatchCreation {
    fn new(_ctx: ComponentContext<B, Self>) -> Self {
        Self {
            count: 1000
        }
    }
}

#[wasm_bindgen]
pub fn create() {
    use std::cell::RefCell;
    thread_local! {
        static CONTEXT: RefCell<Context<Dom>> = {
            let context = Context::new(Dom::new("placeholder"));
            RefCell::new(context)
        };
    }
    CONTEXT.with(|context| {
        let mut context = context.borrow_mut();
        let root_component = context.new_root_component::<BatchCreation>();
        context.set_root_component(root_component);
    });
}

#[wasm_bindgen(start)]
pub fn wasm_main() {
    init_panic_hook();
    console_log::init_with_level(log::Level::Debug).unwrap();
}

fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
