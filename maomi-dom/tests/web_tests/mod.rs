use std::sync::Once;

use maomi::{prelude::*, template::ComponentTemplate, AsyncCallback};
use maomi_dom::prelude::*;

pub mod component;
pub mod event;
pub mod prerendering;
pub mod template;

static INIT: Once = Once::new();

fn init() {
    INIT.call_once(|| {
        console_error_panic_hook::set_once();
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
    // web_sys::window()
    //     .unwrap()
    //     .document()
    //     .unwrap()
    //     .document_element()
    //     .unwrap()
    //     .append_child(&elem)
    //     .unwrap();
    let dom_backend = DomBackend::new_with_element(elem).unwrap();
    let backend_context = maomi::BackendContext::new(dom_backend);
    let (fut, _mount_point) = backend_context
        .enter_sync(move |ctx| {
            let (fut, cb) = AsyncCallback::new();
            let mount_point = ctx
                .attach(move |comp: &mut T| {
                    comp.set_callback(Box::new(|| cb(())));
                })
                .unwrap();
            (fut, mount_point)
        })
        .map_err(|_| "Cannot init mount point")
        .unwrap();
    fut.await
}

#[cfg(feature = "prerendering")]
pub async fn test_component_prerendering<
    T: PrerenderableComponent + ComponentTemplate<DomBackend> + ComponentTest,
>(
    query_data: &T::QueryData,
) -> String {
    init();
    let dom_backend = DomBackend::prerendering();
    let backend_context = maomi::BackendContext::new(dom_backend);
    let prerendering_data = backend_context.prerendering_data::<T>(query_data).await;
    let (_mount_point, ret) = backend_context
        .enter_sync(move |ctx| {
            let mount_point = ctx.prerendering_attach(prerendering_data).unwrap();
            let mut ret = vec![];
            ctx.write_prerendering_html(&mut ret).unwrap();
            (mount_point, ret)
        })
        .map_err(|_| "Cannot init mount point")
        .unwrap();
    String::from_utf8(ret).unwrap()
}
