use maomi::{
    prelude::*,
    backend::{tree::*, Backend, BackendGeneralElement, SupportBackend},
    component::{Component, Node},
    text_node::TextNode,
};
use maomi_backend_dom::{element::*, DomBackend};
use wasm_bindgen::prelude::*;

template!(for DomBackend {
    <div />
    <div>{{ self.hello_text }}</div>
});

#[component]
struct HelloWorld {
    template_structure: (Node<div, (Node<div, (TextNode, ())>, ())>, ()),
    need_update: bool,
    #[property]
    hello_text: String,
    #[event]
    tap_hello: (),
}

#[wasm_bindgen(start)]
pub fn wasm_main() {
    let mut dom_backend = DomBackend::new();
    let mut parent = dom_backend.root_mut();
    let (mut hello_world, elem) =
        <HelloWorld as SupportBackend<DomBackend>>::create(&mut parent).unwrap();
    <DomBackend as Backend>::GeneralElement::append(&mut parent, elem);
    hello_world.set_property_hello_text("Hello again!");
    <HelloWorld as SupportBackend<DomBackend>>::apply_updates(
        &mut hello_world,
        &mut parent.first_child_mut().unwrap(),
    )
    .unwrap();
}
