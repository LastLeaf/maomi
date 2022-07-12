use wasm_bindgen_test::*;

use maomi_backend_dom::element::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn prepare_env(
    f: impl FnOnce(&mut maomi::backend::tree::ForestNodeMut<maomi_backend_dom::DomGeneralElement>),
) {
    use maomi::backend::Backend;
    let mut dom_backend = maomi_backend_dom::DomBackend::new_with_document_body().unwrap();
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
    macro_rules! backend {
        () => { maomi_backend_dom::DomBackend };
    }

    struct HelloWorld {
        template_field: maomi::component::Template<(
            maomi::component::Node<div, (maomi::text_node::TextNode,)>,
        )>,
        hello_text: String,
    }

    impl HelloWorld {
        pub fn set_property_hello_text(&mut self, content: &str) {
            use maomi::component::TemplateHelper;
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

    impl maomi::component::ComponentTemplate<backend!()> for HelloWorld {
        type TemplateField = maomi::component::Template<(
            maomi::component::Node<div, (maomi::text_node::TextNode,)>,
        )>;

        #[inline]
        fn template(&self) -> &Self::TemplateField {
            &self.template_field
        }

        #[inline]
        fn template_mut(&mut self) -> &mut Self::TemplateField {
            &mut self.template_field
        }

        fn create(
            &mut self,
            __parent_element: &mut maomi::backend::tree::ForestNodeMut<<backend!() as maomi::backend::Backend>::GeneralElement>,
        ) -> Result<maomi::backend::tree::ForestNodeRc<<backend!() as maomi::backend::Backend>::GeneralElement>, maomi::error::Error>
        where
            Self: Sized {
            use maomi::backend::BackendGeneralElement;
            let __backend_element = <backend!() as maomi::backend::Backend>::GeneralElement::create_virtual_element(__parent_element)?;
            let __child_nodes = {
                let mut __parent_element = __parent_element.borrow_mut(&__backend_element);
                (
                    {
                        let (__node, __backend_element) =
                            <div as maomi::backend::SupportBackend<backend!()>>::create(&mut __parent_element, |__node| {
                                __node.set_property_hidden(false);
                                Ok(())
                            })?;
                        let __child_nodes = {
                            let mut __parent_element = __parent_element.borrow_mut(&__backend_element);
                            (
                                {
                                    let (__node, __backend_element) = maomi::text_node::TextNode::create::<backend!()>(
                                        &mut __parent_element,
                                        &self.hello_text,
                                    )?;
                                    <backend!() as maomi::backend::Backend>::GeneralElement::append(&mut __parent_element, __backend_element);
                                    __node
                                },
                            )
                        };
                        <backend!() as maomi::backend::Backend>::GeneralElement::append(&mut __parent_element, __backend_element);
                        maomi::component::Node { node: __node, child_nodes: __child_nodes }
                    },
                )
            };
            self.template_field = maomi::component::Template::Structure {
                dirty: false,
                backend_element_token: __backend_element.token(),
                backend_element: Box::new(__backend_element.clone()),
                child_nodes: __child_nodes,
            };
            Ok(__backend_element)
        }

        fn apply_updates(
            &mut self,
            __backend_element: &mut maomi::backend::tree::ForestNodeMut<<backend!() as maomi::backend::Backend>::GeneralElement>,
        ) -> Result<(), maomi::error::Error> {
            match self.template_field {
                maomi::component::Template::Uninitialized => {
                    Ok(())
                }
                maomi::component::Template::Structure {
                    dirty: ref mut __dirty,
                    child_nodes: ref mut __child_nodes,
                    backend_element_token: ref __backend_element_token,
                    ..
                } => {
                    if *__dirty {
                        *__dirty = false;
                        let mut __backend_element = __backend_element.borrow_mut_token(__backend_element_token);
                        {
                            let maomi::component::Node { node: ref mut __node, child_nodes: ref mut __child_nodes } = __child_nodes.0;
                            {
                                __node.set_property_hidden(false);
                            }
                            <div as maomi::backend::SupportBackend<backend!()>>::apply_updates(__node, &mut __backend_element)?;
                            {
                                let __node = &mut __child_nodes.0;
                                __node.set_text(&self.hello_text);
                                __node.apply_updates::<backend!()>(&mut __backend_element)?;
                            }
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
