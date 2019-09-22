//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use maomi::prelude::*;

wasm_bindgen_test_configure!(run_in_browser);

template!(tmpl TestComponent {
    div {
        style = "display: inline";
        (&self.a);
    }
});

#[component]
struct TestComponent {
    #[property]
    a: String,
}

#[component]
impl TestComponent {
    fn new() -> Self {
        Self {
            a: "Hello world!".into()
        }
    }
    fn attached() {

    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen_test]
fn create_new_component() {
    let test_component = maomi::render(Box::new(TestComponent::new()));
    console_log!("{:?}", test_component);
    // TestComponent::dom();
}
