use maomi::component::{TemplateHelper, ComponentAttributeMacro};
use wasm_bindgen_test::*;

use maomi_backend_dom::element::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn prepare_env(
    f: impl FnOnce(&mut maomi::backend::tree::ForestNodeMut<maomi_backend_dom::DomGeneralElement>),
) {
    use maomi::backend::Backend;
    let mut dom_backend = maomi_backend_dom::DomBackend::new();
    let mut parent = dom_backend.root_mut();
    f(&mut parent);
}

fn dom_html(
    e: &mut maomi::backend::tree::ForestNodeMut<maomi_backend_dom::DomGeneralElement>,
) -> String {
    <maomi_backend_dom::DomBackend as maomi::backend::Backend>::GeneralElement::inner_html(
        &e.as_ref(),
    )
}

#[wasm_bindgen_test]
fn manual_tree_building() {
    struct HelloWorld {
        template_field: maomi::component::Template<(
            maomi::component::Node<div, (maomi::text_node::TextNode,)>,
        )>,
        hello_text: String,
    }

    impl HelloWorld {
        pub fn set_property_hello_text(&mut self, content: &str) {
            if self.hello_text.as_str() != content {
                self.hello_text = content.into();
                self.template_field.mark_dirty();
            }
        }
    }

    impl maomi::component::Component for HelloWorld {
        fn new() -> Self {
            Self {
                template_field: Default::default(),
                hello_text: "Hello world!".into(),
            }
        }
    }

    impl maomi::component::ComponentTemplate<maomi_backend_dom::DomBackend> for HelloWorld {
        type TemplateField = maomi::component::Template<(
            maomi::component::Node<div, (maomi::text_node::TextNode,)>,
        )>;

        fn template(&self) -> &Self::TemplateField {
            &self.template_field
        }

        fn template_mut(&mut self) -> &mut Self::TemplateField {
            &mut self.template_field
        }

        fn create(
            &mut self,
            parent_element: &mut maomi::backend::tree::ForestNodeMut<<maomi_backend_dom::DomBackend as maomi::backend::Backend>::GeneralElement>,
        ) -> Result<maomi::backend::tree::ForestNodeRc<<maomi_backend_dom::DomBackend as maomi::backend::Backend>::GeneralElement>, maomi::error::Error>
        where
            Self: Sized {
            use maomi::backend::BackendGeneralElement;
            let backend_element = <maomi_backend_dom::DomBackend as maomi::backend::Backend>::GeneralElement::create_virtual_element(parent_element)?;
            let child_nodes = {
                let mut parent_element = parent_element.borrow_mut(&backend_element);
                (
                    {
                        let (node, backend_element) =
                            <div as maomi::backend::SupportBackend<maomi_backend_dom::DomBackend>>::create(&mut parent_element, |child| {
                                child.set_property_hidden(false);
                                Ok(())
                            })?;
                        let child_nodes = {
                            let mut parent_element = parent_element.borrow_mut(&backend_element);
                            (
                                {
                                    let (text_node, backend_element) = maomi::text_node::TextNode::create::<maomi_backend_dom::DomBackend>(
                                        &mut parent_element,
                                        &self.hello_text,
                                    )?;
                                    <maomi_backend_dom::DomBackend as maomi::backend::Backend>::GeneralElement::append(&mut parent_element, backend_element);
                                    text_node
                                },
                            )
                        };
                        <maomi_backend_dom::DomBackend as maomi::backend::Backend>::GeneralElement::append(&mut parent_element, backend_element);
                        maomi::component::Node { node, child_nodes }
                    },
                )
            };
            self.template_field = maomi::component::Template::Structure {
                dirty: false,
                backend_element_token: backend_element.token(),
                backend_element: Box::new(backend_element.clone()),
                child_nodes,
            };
            Ok(backend_element)
        }

        fn apply_updates(
            &mut self,
            backend_element: &mut maomi::backend::tree::ForestNodeMut<<maomi_backend_dom::DomBackend as maomi::backend::Backend>::GeneralElement>,
        ) -> Result<(), maomi::error::Error> {
            match &mut self.template_field {
                maomi::component::Template::Uninitialized => {
                    Ok(())
                }
                maomi::component::Template::Structure { dirty, child_nodes, backend_element_token, .. } => {
                    if *dirty {
                        *dirty = false;
                        let mut backend_element = backend_element.borrow_mut_token(backend_element_token);
                        {
                            let maomi::component::Node { ref mut node, child_nodes } = &mut child_nodes.0;
                            {
                                node.set_property_hidden(false);
                            }
                            {
                                let text_node = &mut child_nodes.0;
                                text_node.set_text(&self.hello_text);
                                text_node.apply_updates::<maomi_backend_dom::DomBackend>(&mut backend_element)?;
                            }
                            <div as maomi::backend::SupportBackend<maomi_backend_dom::DomBackend>>::apply_updates(node, &mut backend_element)?;
                        }
                    }
                    Ok(())
                }
            }
        }
    }

    prepare_env(|mut wrapper| {
        use maomi::backend::{SupportBackend, BackendGeneralElement};
        use maomi_backend_dom::DomBackend;
        console_log::init().unwrap();
        let (mut hello_world, elem) =
            <HelloWorld as SupportBackend<DomBackend>>::create(&mut wrapper, |_| Ok(())).unwrap();
        <DomBackend as maomi::backend::Backend>::GeneralElement::append(&mut wrapper, elem);
        hello_world.set_property_hello_text("Hello again!");
        assert_eq!(dom_html(wrapper), "<div>Hello world!</div>");
        <HelloWorld as SupportBackend<DomBackend>>::apply_updates(
            &mut hello_world,
            &mut wrapper.first_child_mut().unwrap(),
        )
        .unwrap();
        assert_eq!(dom_html(wrapper), "<div>Hello again!</div>");
    });
}
