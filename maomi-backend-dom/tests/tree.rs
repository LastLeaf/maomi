use wasm_bindgen_test::*;

use maomi::{
    backend::{tree::ForestNodeMut, Backend, BackendGeneralElement, SupportBackend},
    component::{ComponentTemplate, Node},
    text_node::TextNode,
};
use maomi_backend_dom::{element::*, DomBackend, DomGeneralElement};

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn prepare_env(f: impl FnOnce(&mut ForestNodeMut<DomGeneralElement>)) {
    let mut dom_backend = DomBackend::new();
    let mut parent = dom_backend.root_mut();
    let (_, wrapper_elem) = div::create(&mut parent).unwrap();
    <DomBackend as Backend>::GeneralElement::append(&mut parent, wrapper_elem);
    f(&mut parent.first_child_mut().unwrap())
}

fn dom_html(e: &mut ForestNodeMut<DomGeneralElement>) -> String {
    <DomBackend as Backend>::GeneralElement::inner_html(&e.as_ref())
}

#[wasm_bindgen_test]
fn manual_tree_building() {
    struct HelloWorld {
        template_structure: (Node<div, (Node<div, ()>, TextNode, ())>, ()),
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

    impl ComponentTemplate<DomBackend> for HelloWorld {
        fn create(
            backend_element: &mut ForestNodeMut<'_, <DomBackend as Backend>::GeneralElement>,
        ) -> Result<Self, maomi::error::Error>
        where
            Self: Sized,
        {
            const HELLO_TEXT: &'static str = "Hello world!";
            let mut parent_elem = backend_element;
            let this = Self {
                template_structure: (
                    {
                        let (node, mut elem) =
                            <div as SupportBackend<DomBackend>>::create(&mut parent_elem)?;
                        let children = (
                            {
                                let mut parent_elem = elem.as_node_mut();
                                let (node, elem) =
                                    <div as SupportBackend<DomBackend>>::create(&mut parent_elem)?;
                                let children = ();
                                <DomBackend as Backend>::GeneralElement::append(
                                    &mut parent_elem,
                                    elem,
                                );
                                Node { node, children }
                            },
                            {
                                let mut parent_elem = elem.as_node_mut();
                                let (node, elem) = TextNode::create::<DomBackend>(
                                    &mut parent_elem,
                                    HELLO_TEXT.into(),
                                )?;
                                <DomBackend as Backend>::GeneralElement::append(
                                    &mut parent_elem,
                                    elem,
                                );
                                node
                            },
                            (),
                        );
                        <DomBackend as Backend>::GeneralElement::append(&mut parent_elem, elem);
                        Node { node, children }
                    },
                    (),
                ),
                need_update: false,
                hello_text: HELLO_TEXT.into(),
            };
            Ok(this)
        }

        fn apply_updates(
            &mut self,
            backend_element: &mut ForestNodeMut<'_, <DomBackend as Backend>::GeneralElement>,
        ) -> Result<(), maomi::error::Error> {
            if !self.need_update {
                return Ok(());
            }
            let children = &mut self.template_structure;
            let elem = backend_element;
            let next_child_elem = elem.first_child_mut();
            {
                let Node { node: _, children } = &mut children.0;
                let mut elem = next_child_elem.ok_or(maomi::error::Error::TreeNotMatchedError)?;
                {
                    let next_child_elem = elem.first_child_mut();
                    {
                        let Node {
                            node: _self_node,
                            children: _self_children,
                        } = &mut children.0;
                        let mut elem =
                            next_child_elem.ok_or(maomi::error::Error::TreeNotMatchedError)?;
                        // {
                        //     let next_child_elem = elem.first_child_mut();
                        //     {}
                        // }
                        let next_child_elem = elem.next_sibling_mut();

                        let node = &mut children.1;
                        let elem =
                            next_child_elem.ok_or(maomi::error::Error::TreeNotMatchedError)?;
                        {
                            node.set_text(&self.hello_text);
                            node.apply_updates::<DomBackend>(elem)?;
                        }
                        // let next_child_elem = elem.next_sibling_mut();
                    }
                }
            }
            Ok(())
        }
    }

    prepare_env(|mut wrapper| {
        let (mut hello_world, elem) =
            <HelloWorld as SupportBackend<DomBackend>>::create(&mut wrapper).unwrap();
        <DomBackend as Backend>::GeneralElement::append(&mut wrapper, elem);
        hello_world.set_property_hello_text("Hello again!");
        assert_eq!(dom_html(wrapper), "<div><div></div>Hello world!</div>");
        <HelloWorld as SupportBackend<DomBackend>>::apply_updates(
            &mut hello_world,
            &mut wrapper.first_child_mut().unwrap(),
        )
        .unwrap();
        assert_eq!(dom_html(wrapper), "<div><div></div>Hello again!</div>");
    });
}
