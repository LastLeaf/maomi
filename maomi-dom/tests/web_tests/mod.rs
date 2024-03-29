use js_sys::Reflect;
use std::sync::Once;
use wasm_bindgen::{JsCast, JsValue};

use maomi::{prelude::*, template::ComponentTemplate, AsyncCallback};
use maomi_dom::{async_task, prelude::*};

macro_rules! first_dom {
    ($this:expr, $elem:ty) => {
        (
            $this.template_structure()
                .unwrap()[0]
                .as_ref::<maomi::node::Node<$elem>>()
                .tag
                .dom_element()
        )
    };
}

pub mod component;
pub mod event;
pub mod prerendering;
pub mod skin;
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
) -> (String, T::PrerenderingData)
where
    T::PrerenderingData: Clone,
{
    init();
    let dom_backend = DomBackend::prerendering();
    let backend_context = maomi::BackendContext::new(dom_backend);
    let prerendering_data = maomi::BackendContext::<DomBackend>::prerendering_data::<T>(query_data).await;
    let prerendering_data_cloned = prerendering_data.get().clone();
    let (_mount_point, ret) = backend_context
        .enter_sync(move |ctx| {
            let mount_point = ctx.prerendering_attach(prerendering_data).unwrap();
            let mut ret = vec![];
            ctx.write_prerendering_html(&mut ret).unwrap();
            (mount_point, ret)
        })
        .map_err(|_| "Cannot init mount point")
        .unwrap();
    (String::from_utf8(ret).unwrap(), prerendering_data_cloned)
}

#[cfg(feature = "prerendering-apply")]
pub async fn test_component_prerendering_apply<
    T: PrerenderableComponent + ComponentTemplate<DomBackend> + ComponentTest,
>(
    html: &str,
    prerendering_data: T::PrerenderingData,
) {
    init();
    let prerendering_data = maomi::PrerenderingData::<T>::new(prerendering_data);
    let elem = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("div")
        .unwrap();
    elem.set_inner_html(html);
    let dom_backend = DomBackend::new_prerendered();
    let backend_context = maomi::BackendContext::new(dom_backend);
    let (fut, _mount_point) = backend_context
        .enter_sync(move |ctx| {
            let (fut, cb) = AsyncCallback::new();
            let mount_point = ctx.prerendering_attach(prerendering_data).unwrap();
            ctx.apply_prerendered_element(elem).unwrap();
            let root_rc = mount_point.root_component().rc();
            maomi_dom::async_task(async move {
                root_rc
                    .update(|comp| {
                        comp.set_callback(Box::new(|| cb(())));
                    })
                    .await
                    .unwrap();
            });
            (fut, mount_point)
        })
        .map_err(|_| "Cannot init mount point")
        .unwrap();
    fut.await
}

fn simulate_event(
    target: &web_sys::EventTarget,
    ty: &str,
    bubbles: bool,
    props: impl IntoIterator<Item = (&'static str, JsValue)>,
) {
    let mut event_init = web_sys::EventInit::new();
    event_init.bubbles(bubbles);
    let ev = web_sys::Event::new_with_event_init_dict(ty, &event_init).unwrap();
    for (k, v) in props {
        Reflect::set(&ev, &JsValue::from_str(k), &v).unwrap();
    }
    let target = target.clone();
    async_task(async move {
        target.dispatch_event(&ev).unwrap();
    });
}

fn generate_fake_touch(
    target: &web_sys::Element,
    identifier: u32,
    client_x: i32,
    client_y: i32,
) -> JsValue {
    let v = js_sys::Object::new();
    Reflect::set(&v, &JsValue::from_str("identifier"), &JsValue::from_f64(identifier as f64)).unwrap();
    Reflect::set(&v, &JsValue::from_str("clientX"), &JsValue::from_f64(client_x as f64)).unwrap();
    Reflect::set(&v, &JsValue::from_str("clientY"), &JsValue::from_f64(client_y as f64)).unwrap();
    Reflect::set(&v, &JsValue::from_str("target"), target).unwrap();
    let arr = js_sys::Array::new();
    arr.push(&v);
    arr.dyn_into().unwrap()
}
