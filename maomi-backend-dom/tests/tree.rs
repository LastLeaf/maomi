use maomi::{diff::ListItemChange, prelude::TemplateHelper};
use wasm_bindgen_test::*;

use maomi_backend_dom::element::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn prepare_env(
    f: impl FnOnce(&mut maomi::backend::context::EnteredBackendContext<maomi_backend_dom::DomBackend>),
) {
    let elem = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("div")
        .unwrap();
    let dom_backend = maomi_backend_dom::DomBackend::new_with_element(elem).unwrap();
    let backend_context = maomi::BackendContext::new(dom_backend);
    backend_context
        .enter_sync(move |ctx| f(ctx))
        .map_err(|_| "Cannot init mount point")
        .unwrap();
}

#[wasm_bindgen_test]
fn manual_tree_building() {
    macro_rules! todo_backend {
        () => {
            maomi_backend_dom::DomBackend
        };
    }

    struct TodoComp {
        // template structure
        todo_template: maomi::component::Template<
            TodoComp,
            (maomi::node::Node<div, (maomi::text_node::TextNode,)>,),
        >,
        todo_hello_text: String,
    }

    impl maomi::component::Component for TodoComp {
        fn new() -> Self {
            Self {
                todo_template: Default::default(),
                todo_hello_text: "Hello world!".into(),
            }
        }
    }

    // main impl
    impl maomi::component::ComponentTemplate<todo_backend!(), TodoComp> for TodoComp {
        type TemplateField = maomi::component::Template<
            TodoComp,
            (maomi::node::Node<div, (maomi::text_node::TextNode,)>,),
        >;
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
        fn template_init(&mut self, __m_init: maomi::component::TemplateInit<TodoComp>) {
            self.todo_template.init(__m_init);
        }

        #[inline]
        fn template_create<'b, __MSlot>(
            &'b mut self,
            __m_backend_context: &'b maomi::BackendContext<todo_backend!()>,
            __m_backend_element: &'b mut maomi::backend::tree::ForestNodeMut<
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
            self.todo_template.structure = Some({
                // create children
                ({
                    let (mut __m_child, __m_backend_element) =
                        <div as maomi::backend::SupportBackend<todo_backend!()>>::init(
                            __m_backend_context,
                            __m_parent_element,
                        )?;
                    let __m_slot_children = <div as maomi::backend::SupportBackend<
                        todo_backend!(),
                    >>::create(
                        &mut __m_child,
                        __m_backend_context,
                        __m_parent_element,
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
                        node: __m_child,
                        child_nodes: __m_slot_children,
                    }
                },)
            });
            Ok(__m_slot)
        }

        #[inline]
        fn template_update<'b, __MSlot>(
            &'b mut self,
            __m_backend_context: &'b maomi::BackendContext<todo_backend!()>,
            __m_backend_element: &'b mut maomi::backend::tree::ForestNodeMut<
                <todo_backend!() as maomi::backend::Backend>::GeneralElement,
            >,
            __m_slot_fn: impl FnMut(
                ListItemChange<
                    &mut maomi::backend::tree::ForestNodeMut<
                        <todo_backend!() as maomi::backend::Backend>::GeneralElement,
                    >,
                    &Self::SlotData,
                >,
            ) -> Result<__MSlot, maomi::error::Error>,
        ) -> Result<(), maomi::error::Error>
        where
            Self: Sized,
        {
            // update tree
            if self.todo_template.clear_dirty() {
                // update tree
                let mut __m_slot: maomi::node::SlotChildren<__MSlot> =
                    maomi::node::SlotChildren::None;
                let mut __m_parent_element = __m_backend_element;
                let __m_children = self
                    .todo_template
                    .structure
                    .as_mut()
                    .ok_or(maomi::error::Error::TreeNotCreated)?;
                {
                    // update children
                    let maomi::node::Node {
                        node: ref mut __m_child,
                        child_nodes: ref mut __m_slot_children,
                    } = __m_children.0;
                    let mut __m_children_i = 0usize;
                    <div as maomi::backend::SupportBackend<todo_backend!()>>::apply_updates(
                        __m_child,
                        __m_backend_context,
                        __m_parent_element,
                        |__m_slot_change| {
                            Ok({
                                match __m_slot_change {
                                    ListItemChange::Added(__m_parent_element, __m_scope) => {
                                        // create children
                                        let __m_children = ({
                                            let (__m_child, __m_backend_element) =
                                                maomi::text_node::TextNode::create::<
                                                    todo_backend!(),
                                                >(
                                                    __m_parent_element, &self.todo_hello_text
                                                )?;
                                            <<todo_backend!() as maomi::backend::Backend>::GeneralElement as maomi::backend::BackendGeneralElement>::append(__m_parent_element, __m_backend_element);
                                            __m_child
                                        },);
                                        __m_slot_children.add(__m_children_i, __m_children)?;
                                        __m_children_i += 1;
                                    }
                                    ListItemChange::Unchanged(__m_parent_element, __m_scope) => {
                                        let __m_children =
                                            __m_slot_children.get_mut(__m_children_i)?;
                                        {
                                            // update children
                                            let __m_child = &mut __m_children.0;
                                            __m_child.set_text(&self.todo_hello_text);
                                        }
                                        __m_children_i += 1;
                                    }
                                    ListItemChange::Removed(__m_parent_element) => {
                                        __m_slot_children.remove(__m_children_i)?;
                                    }
                                }
                            })
                        },
                    )?;
                }
            } else {
                // TODO no updates, recurse into slots only
                todo!()
            }
            Ok(())
        }
    }

    prepare_env(|ctx| {
        let mut mount_point = ctx.new_mount_point(|_: &mut TodoComp| Ok(())).unwrap();
        mount_point.append_attach(&mut ctx.root_mut());
        let html = maomi_backend_dom::DomGeneralElement::inner_html(&ctx.root());
        assert_eq!(html, "<div>Hello world!</div>");
    });
}
