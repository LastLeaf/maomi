use std::sync::Once;

use maomi::{prelude::*, template::ComponentTemplate, AsyncCallback};
use maomi_backend_dom::DomBackend;

static INIT: Once = Once::new();

fn init() {
    INIT.call_once(|| {
        console_log::init_with_level(log::Level::Trace).unwrap();
    });
}

pub type ComponentTestCb = Box<dyn 'static + FnOnce()>;

pub trait ComponentTest {
    fn set_callback(&mut self, callback: ComponentTestCb);
}

pub async fn test_component<T: Component + ComponentTemplate<DomBackend> + ComponentTest>() {
    init();
    let elem = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("div")
        .unwrap();
    // web_sys::console::log_1(&elem);
    let dom_backend = maomi_backend_dom::DomBackend::new_with_element(elem).unwrap();
    let backend_context = maomi::BackendContext::new(dom_backend);
    let fut = backend_context
        .enter_sync(move |ctx| {
            let (fut, cb) = AsyncCallback::new();
            let _mount_point = ctx
                .append_attach(move |comp: &mut T| {
                    comp.set_callback(Box::new(|| cb(())));
                })
                .unwrap();
            fut
        })
        .map_err(|_| "Cannot init mount point")
        .unwrap();
    fut.await
}
