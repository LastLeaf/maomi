use wasm_bindgen_test::*;

use crate::*;
use maomi_backend_dom::element::*;

#[wasm_bindgen_test]
fn single_slot() {
    macro_rules! todo_backend {
        () => {
            maomi_backend_dom::DomBackend
        };
    }

    struct TodoComp {
        // template structure
        todo_template: maomi::template::Template<
            TodoComp,
            (maomi::node::Node<todo_backend!(), div, (maomi::text_node::TextNode,)>,),
            (),
        >,
        todo_hello_text: String,
        todo_hello_title: String,
    }

    impl maomi::component::Component for TodoComp {
        fn new() -> Self {
            Self {
                todo_template: Default::default(),
                todo_hello_text: "Hello world!".into(),
                todo_hello_title: "Hello world".into(),
            }
        }
    }

    // main impl
    impl maomi::template::ComponentTemplate<todo_backend!()> for TodoComp {
        type TemplateField = maomi::template::Template<
            Self,
            Self::TemplateStructure,
            Self::SlotData,
        >;
        type TemplateStructure = (maomi::node::Node<todo_backend!(), div, (maomi::text_node::TextNode,)>,);
        type SlotData = ();

        #[inline]
        fn template(&self) -> &Self::TemplateField {
            &self.todo_template
        }

        #[inline]
        fn template_mut(&mut self) -> &mut Self::TemplateField {
            &mut self.todo_template
        }

        #[inline]
        fn template_init(&mut self, __m_init: maomi::template::TemplateInit<TodoComp>) {
            self.todo_template.init(__m_init);
        }

        #[inline]
        fn template_create<'__m_b, __MSlot>(
            &'__m_b mut self,
            __m_backend_context: &'__m_b maomi::BackendContext<todo_backend!()>,
            __m_backend_element: &'__m_b mut maomi::backend::tree::ForestNodeMut<
                <todo_backend!() as maomi::backend::Backend>::GeneralElement,
            >,
            __m_slot_fn: impl FnMut(
                &mut maomi::backend::tree::ForestNodeMut<
                    <todo_backend!() as maomi::backend::Backend>::GeneralElement,
                >,
                &Self::SlotData,
            ) -> Result<__MSlot, maomi::error::Error>,
        ) -> Result<maomi::node::SlotChildren<__MSlot>, maomi::error::Error>
        where
            Self: Sized,
        {
            // create initial tree
            let mut __m_slot: maomi::node::SlotChildren<__MSlot> = maomi::node::SlotChildren::None;
            let mut __m_parent_element = __m_backend_element;
            self.todo_template.__m_structure = Some({
                // create children
                ({
                    let (mut __m_child, __m_backend_element) =
                        <<div as maomi::backend::SupportBackend<todo_backend!()>>::Target as maomi::backend::BackendComponent<todo_backend!()>>::init(
                            __m_backend_context,
                            __m_parent_element,
                        )?;
                    let __m_slot_children = <<div as maomi::backend::SupportBackend<todo_backend!()>>::Target as maomi::backend::BackendComponent<todo_backend!()>>::create(
                        &mut __m_child,
                        __m_backend_context,
                        __m_parent_element,
                        |__m_child, __m_update_ctx| {
                            maomi::prop::PropertyUpdate::compare_and_set_ref(
                                &mut __m_child.title,
                                &self.todo_hello_title,
                                __m_update_ctx,
                            );
                            maomi::prop::PropertyUpdate::compare_and_set_ref(
                                &mut __m_child.hidden,
                                &false,
                                __m_update_ctx,
                            );
                        },
                        |__m_parent_element, __m_scope| {
                            Ok({
                                // create children
                                ({
                                    let (__m_child, __m_backend_element) =
                                        maomi::text_node::TextNode::create::<todo_backend!()>(
                                            __m_parent_element,
                                            &self.todo_hello_text,
                                        )?;
                                    <<todo_backend!() as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::append(__m_parent_element, __m_backend_element);
                                    __m_child
                                },)
                            })
                        },
                    )?;
                    <<todo_backend!() as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::append(&mut __m_parent_element, __m_backend_element);
                    maomi::node::Node {
                        tag: __m_child,
                        child_nodes: __m_slot_children,
                    }
                },)
            });
            Ok(__m_slot)
        }

        #[inline]
        fn template_update<'__m_b>(
            &'__m_b mut self,
            __m_backend_context: &'__m_b maomi::BackendContext<todo_backend!()>,
            __m_backend_element: &'__m_b mut maomi::backend::tree::ForestNodeMut<
                <todo_backend!() as maomi::backend::Backend>::GeneralElement,
            >,
            __m_slot_fn: impl FnMut(
                maomi::diff::ListItemChange<
                    &mut maomi::backend::tree::ForestNodeMut<
                        <todo_backend!() as maomi::backend::Backend>::GeneralElement,
                    >,
                    &Self::SlotData,
                >,
            ) -> Result<(), maomi::error::Error>,
        ) -> Result<(), maomi::error::Error>
        where
            Self: Sized,
        {
            // update tree
            let mut __m_parent_element = __m_backend_element;
            let __m_children = self
                .todo_template
                .__m_structure
                .as_mut()
                .ok_or(maomi::error::Error::TreeNotCreated)?;
            {
                // update children
                let maomi::node::Node {
                    tag: ref mut __m_child,
                    child_nodes: ref mut __m_slot_children,
                } = __m_children.0;
                let mut __m_children_i = 0usize;
                <<div as maomi::backend::SupportBackend<todo_backend!()>>::Target as maomi::backend::BackendComponent<todo_backend!()>>::apply_updates(
                    __m_child,
                    __m_backend_context,
                    __m_parent_element,
                    |__m_child, __m_update_ctx| {
                        maomi::prop::PropertyUpdate::compare_and_set_ref(
                            &mut __m_child.title,
                            &self.todo_hello_title,
                            __m_update_ctx,
                        );        
                    },
                    |__m_slot_change| {
                        Ok({
                            match __m_slot_change {
                                maomi::diff::ListItemChange::Added(
                                    __m_parent_element,
                                    __m_scope,
                                ) => {
                                    // create children
                                    let __m_children = ({
                                        let (__m_child, __m_backend_element) =
                                            maomi::text_node::TextNode::create::<todo_backend!()>(
                                                __m_parent_element,
                                                &self.todo_hello_text,
                                            )?;
                                        <<todo_backend!() as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::append(__m_parent_element, __m_backend_element);
                                        __m_child
                                    },);
                                    __m_slot_children.add(__m_children_i, __m_children)?;
                                    __m_children_i += 1;
                                }
                                maomi::diff::ListItemChange::Unchanged(
                                    __m_parent_element,
                                    __m_scope,
                                ) => {
                                    let __m_children = __m_slot_children.get_mut(__m_children_i)?;
                                    {
                                        // update children
                                        let __m_child = &mut __m_children.0;
                                        __m_child.set_text::<todo_backend!()>(
                                            __m_parent_element,
                                            &self.todo_hello_text,
                                        )?;
                                    }
                                    __m_children_i += 1;
                                }
                                maomi::diff::ListItemChange::Removed(__m_parent_element) => {
                                    __m_slot_children.remove(__m_children_i)?;
                                }
                            }
                        })
                    },
                )?;
            }
            Ok(())
        }
    }

    prepare_env(|ctx| {
        let _mount_point = ctx.append_attach(|_: &mut TodoComp| {}).unwrap();
        let html = maomi_backend_dom::DomGeneralElement::inner_html(&ctx.root());
        assert_eq!(html, "<div>Hello world!</div>");
    });
}
