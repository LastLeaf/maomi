use maomi::{
    backend::{Backend, BackendComponent, BackendGeneralElement, SupportBackend},
    component::{Component, Node},
    text_node::TextNode,
};
use maomi_backend_dom::{element::*, DomBackend};
use wasm_bindgen::prelude::*;

struct HelloWorld {
    template_structure: (Node<div, (Node<div, (TextNode, ())>, ())>, ()),
    need_update: bool,
    hello_text: String,
}

impl HelloWorld {
    pub fn set_property_hello_text(&mut self, content: &str) {
        if self.hello_text.as_str() != content {
            self.hello_text = content.into();
            self.need_update = true;
        }
    }
}

impl Component<DomBackend> for HelloWorld {
    fn create(
        parent: &mut <DomBackend as Backend>::GeneralElement,
    ) -> Result<(Self, <DomBackend as Backend>::Component), maomi::error::Error>
    where
        Self: Sized,
    {
        const HELLO_TEXT: &'static str = "Hello world!";
        let mut elem = parent.create_component();
        let parent_elem = elem.shadow_root_mut();
        let this = Self {
            template_structure: (
                {
                    let (node, mut elem) =
                        <div as SupportBackend<DomBackend>>::create(parent_elem)?;
                    let children = (
                        {
                            let parent_elem = &mut elem;
                            let (node, mut elem) =
                                <div as SupportBackend<DomBackend>>::create(parent_elem)?;
                            let children = (
                                {
                                    let parent_elem = &mut elem;
                                    let (node, elem) = TextNode::create::<DomBackend>(
                                        parent_elem,
                                        HELLO_TEXT.into(),
                                    )?;
                                    parent_elem.append_children(Some(elem));
                                    node
                                },
                                (),
                            );
                            parent_elem.append_children(Some(elem));
                            Node { node, children }
                        },
                        (),
                    );
                    Node { node, children }
                },
                (),
            ),
            need_update: false,
            hello_text: HELLO_TEXT.into(),
        };
        Ok((this, elem.into()))
    }

    fn apply_updates(
        &mut self,
        backend_element: &mut <DomBackend as Backend>::Component,
    ) -> Result<(), maomi::error::Error> {
        if !self.need_update {
            return Ok(());
        }
        let children = &mut self.template_structure;
        {
            let Node { node: _, children } = &mut children.0;
            let elem = backend_element.shadow_root_mut();
            let mut next_child_elem = elem.first_child_mut();
            {
                let Node { node: _, children } = &mut children.0;
                let elem = next_child_elem.ok_or(maomi::error::Error::TreeNotMatchedError)?;
                {
                    let node = &mut children.0;
                    let mut next_child_elem = elem.first_child_mut();
                    {
                        let elem =
                            next_child_elem.ok_or(maomi::error::Error::TreeNotMatchedError)?;
                        node.set_text(&self.hello_text);
                        node.apply_updates::<DomBackend>(elem)?;
                    }
                }
                next_child_elem = elem.next_sibling_mut();
            }
        }
        Ok(())
    }
}

#[wasm_bindgen(start)]
pub fn wasm_main() {
    let mut dom_backend = DomBackend::new();
    let parent = dom_backend.root_mut();
    let (mut hello_world, elem) =
        <HelloWorld as SupportBackend<DomBackend>>::create(parent).unwrap();
    parent.append_children(Some(elem.into()));
    hello_world.set_property_hello_text("Hello again!");
    <HelloWorld as SupportBackend<DomBackend>>::apply_updates(
        &mut hello_world,
        parent.first_child_mut().unwrap(),
    )
    .unwrap();
}
