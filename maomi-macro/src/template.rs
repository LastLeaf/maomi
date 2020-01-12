use proc_macro2::TokenStream as TokenStream2;
use syn::*;
use quote::*;

fn is_expr_dynamic(expr: &Expr) -> bool {
    if let Expr::Lit(_) = expr {
        false
    } else {
        true
    }
}

#[derive(Clone)]
pub(crate) struct TemplateValue {
    pub(crate) is_dynamic: bool,
    pub(crate) expr: Expr,
}
impl ToTokens for TemplateValue {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let expr = &self.expr;
        tokens.append_all(quote! {
            #expr
        })
    }
}
impl From<Expr> for TemplateValue {
    fn from(expr: Expr) -> Self {
        Self {
            is_dynamic: is_expr_dynamic(&expr),
            expr,
        }
    }
}

#[derive(Clone)]
pub(crate) enum Attribute {
    Mark { value: TemplateValue },
    ClassProp { value: TemplateValue },
    Common { name: LitStr, value: TemplateValue },
    Prop { name: Ident, value: TemplateValue },
    SystemEv { name: Ident, value: TemplateValue },
    Ev { name: Ident, value: TemplateValue },
}
impl Attribute {
    fn is_dynamic(&self) -> bool {
        match self {
            Attribute::Mark { value, .. } => value.is_dynamic,
            Attribute::ClassProp { value, .. } => value.is_dynamic,
            Attribute::Common { value, .. } => value.is_dynamic,
            Attribute::Prop { value, .. } => value.is_dynamic,
            Attribute::SystemEv { value, .. } => value.is_dynamic,
            Attribute::Ev { value, .. } => value.is_dynamic,
        }
    }
}

#[derive(Clone)]
pub(crate) struct TemplateNativeNode {
    pub(crate) tag_name: LitStr,
    pub(crate) attributes: Vec<Attribute>,
    pub(crate) children: Vec<TemplateNode>
}

#[derive(Clone)]
pub(crate) enum TemplateVirtualNode {
    Slot { name: Option<LitStr> },
    InSlot { name: LitStr, children: Vec<TemplateNode> },
    If { branches: Vec<(Option<Expr>, Vec<TemplateNode>)> },
    For { list: Expr, index: Ident, item: Ident, key: Option<(Ident, Path)>, children: Vec<TemplateNode> },
}

#[derive(Clone)]
pub(crate) struct TemplateComponent {
    pub(crate) tag_name: LitStr,
    pub(crate) component: Path,
    pub(crate) property_values: Vec<Attribute>,
    pub(crate) children: Vec<TemplateNode>
}

#[derive(Clone)]
pub(crate) enum TemplateNode {
    NativeNode(TemplateNativeNode),
    VirtualNode(TemplateVirtualNode),
    Component(TemplateComponent),
    TextNode(TemplateValue),
}
impl ToTokens for TemplateNode {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let node = match self {

            TemplateNode::NativeNode(x) => {
                // native node logic
                let TemplateNativeNode { tag_name, attributes, children } = x;
                let indexes: Vec<usize> = (0..children.len()).into_iter().map(|x| x).collect();
                let update_attributes: Vec<TokenStream2> = attributes.iter().map(|attribute| {
                    let content = match attribute {
                        Attribute::Mark { value } => quote! {
                            node.set_mark(#value);
                        },
                        Attribute::ClassProp { value } => quote! {
                            node.set_attribute("class", __prepend_class_prefix(#value, __template_skin_prefix))
                        },
                        Attribute::Common { name, value } => quote! {
                            node.set_attribute(#name, #value)
                        },
                        Attribute::SystemEv { name, value } => quote! {
                            node.global_events_mut().#name.set_handler(Box::new(|self_ref_mut, e| {
                                let f: Box<dyn Fn(ComponentRefMut<B, Self>, _)> = Box::new(#value);
                                f(self_ref_mut.duplicate().with_type::<Self>(), e)
                            }));
                        },
                        _ => {
                            unreachable!()
                        },
                    };
                    if attribute.is_dynamic() {
                        quote! { #content }
                    } else {
                        quote! { if is_init { #content } }
                    }
                }).collect();
                quote! {
                    |__owner: &mut ComponentNodeRefMut<_>, __update_to: Option<&NodeRc<_>>| {
                        let __node_rc = __update_to.map(|node_rc| if let NodeRc::NativeNode(node_rc) = node_rc { node_rc } else { unreachable!() });
                        let __node = __node_rc.as_ref().map(|node_rc| unsafe { node_rc.borrow_mut_unsafe_with(__owner) });
                        let __children = __node.as_ref().map(|node| { node.children() });
                        let ret_children: Vec<NodeRc<_>> = vec![#(
                            (#children)(__owner, if let Some(children) = __children { Some(&children[#indexes]) } else { None })
                        ),*];
                        let is_init = __node_rc.is_none();
                        let node_rc = match __node_rc {
                            None => __owner.new_native_node(#tag_name, vec![], ret_children),
                            Some(node_rc) => node_rc.clone(),
                        };
                        {
                            let mut node = node_rc.borrow_mut_with(__owner);
                            #(#update_attributes)*
                        }
                        node_rc.into()
                    }
                }
            },

            TemplateNode::VirtualNode(x) => {
                match x {

                    TemplateVirtualNode::Slot { name } => {
                        // slot node logic
                        let slot_name = match name {
                            None => LitStr::new("", proc_macro2::Span::call_site()),
                            Some(x) => x.clone(),
                        };
                        quote! {
                            |__owner: &mut ComponentNodeRefMut<_>, __update_to: Option<&NodeRc<_>>| {
                                match __update_to {
                                    None => {
                                        __owner.new_virtual_node("slot", VirtualNodeProperty::Slot(#slot_name, vec![]), vec![]).into()
                                    },
                                    Some(node_rc) => {
                                        node_rc.clone()
                                    },
                                }
                            }
                        }
                    },

                    TemplateVirtualNode::InSlot { name, children } => {
                        // in-slot node logic
                        let indexes: Vec<usize> = (0..children.len()).into_iter().map(|x| x).collect();
                        quote! {
                            |__owner: &mut ComponentNodeRefMut<_>, __update_to: Option<&NodeRc<_>>| {
                                match __update_to {
                                    None => {
                                        let __node_rc = __update_to.map(|node_rc| if let NodeRc::NativeNode(node_rc) = node_rc { node_rc } else { unreachable!() });
                                        let __node = __node_rc.as_ref().map(|node_rc| unsafe { node_rc.borrow_mut_unsafe_with(__owner) });
                                        let __children = __node.as_ref().map(|node| { node.children() });
                                        let ret_children: Vec<NodeRc<_>> = vec![#(
                                            (#children)(__owner, if let Some(children) = __children { Some(&children[#indexes]) } else { None })
                                        ),*];
                                        __owner.new_virtual_node("in", VirtualNodeProperty::InSlot(#name, ret_children), vec![]).into()
                                    },
                                    Some(node_rc) => {
                                        node_rc.clone()
                                    },
                                }
                            }
                        }
                    },

                    TemplateVirtualNode::If { branches } => {
                        // if node logic
                        let children_branches: Vec<_> = branches.iter().enumerate().map(|(key, (cond, children))| {
                            let indexes: Vec<usize> = (0..children.len()).into_iter().map(|x| x).collect();
                            let content = quote! {
                                {
                                    const KEY: usize = #key;
                                    let __equal = if let Some(old_key) = __old_key { *old_key == KEY } else { false };
                                    let children: Vec<NodeRc<_>> = vec![#(
                                        (#children)(__owner, if let Some(children) = __children {
                                            if __equal { Some(&children[#indexes]) } else { None }
                                        } else { None })
                                    ),*];
                                    if __equal {
                                        __node_rc.unwrap().clone().into()
                                    } else {
                                        match __node_rc {
                                            Some(node_rc) => {
                                                let mut node = node_rc.borrow_mut_with(__owner);
                                                node.replace_children_list(children);
                                                *node.property_mut() = VirtualNodeProperty::Branch(KEY);
                                                node_rc.clone().into()
                                            },
                                            None => {
                                                __owner.new_virtual_node("if", VirtualNodeProperty::Branch(KEY), children).into()
                                            }
                                        }
                                    }
                                }
                            };
                            match cond {
                                Some(cond) => quote! {
                                    if #cond #content
                                },
                                None => quote! {
                                    #content
                                }
                            }
                        }).collect();
                        quote! {

                            // if node logic
                            |__owner: &mut ComponentNodeRefMut<_>, __update_to: Option<&NodeRc<_>>| {
                                let __node_rc = __update_to.map(|node_rc| if let NodeRc::VirtualNode(node_rc) = node_rc { node_rc } else { unreachable!() });
                                let __node = __node_rc.as_ref().map(|node_rc| unsafe { node_rc.borrow_mut_unsafe_with(__owner) });
                                let __old_key = match &__node {
                                    Some(x) => {
                                        let index = if let VirtualNodeProperty::Branch(b) = x.property() { b } else { unreachable!() };
                                        Some(index)
                                    },
                                    None => None,
                                };
                                let __children = __node.as_ref().map(|node| { node.children() });
                                #(#children_branches)else*
                            }
                        }
                    },

                    TemplateVirtualNode::For { list, index, item, key, children } => {
                        // for node logic
                        let indexes: Vec<usize> = (0..children.len()).into_iter().map(|x| x).collect();
                        let key_list = match key {
                            Some((key_name, key_ty)) => quote! {
                                {
                                    let keys: Box<VirtualKeyList<#key_ty>> = {
                                        let v: Vec<Option<#key_ty>> = (#list).into_iter().map(|x| {
                                            Some(x.#key_name.clone())
                                        }).collect();
                                        let v = VirtualKeyList::new(v);
                                        let keys = Box::new(v);
                                        keys
                                    };
                                    let reordered_list: VirtualKeyChanges<_> = match __update_to.as_ref() {
                                        Some(node_rc) => {
                                            let node_rc = if let NodeRc::VirtualNode(node_rc) = node_rc { node_rc } else { unreachable!() };
                                            let node = unsafe { node_rc.borrow_mut_unsafe_with(__owner) };
                                            let mut node2 = node_rc.borrow_mut_with(__owner);
                                            let old_keys: &VirtualKeyList<#key_ty> = if let VirtualNodeProperty::List(list) = node.property() {
                                                list.downcast_ref::<VirtualKeyList<#key_ty>>().unwrap()
                                            } else { unreachable!() };
                                            keys.list_reorder(old_keys, &mut node2)
                                        },
                                        None => {
                                            VirtualKeyChanges::new_empty(keys.len())
                                        },
                                    };
                                    (keys, reordered_list)
                                }
                            },
                            None => quote! {
                                {
                                    let keys: Box<VirtualKeyList<()>> = {
                                        let v: Vec<Option<()>> = (#list).into_iter().map(|_| None).collect();
                                        let v = VirtualKeyList::new(v);
                                        let keys = Box::new(v);
                                        keys
                                    };
                                    let reordered_list: VirtualKeyChanges<_> = match __update_to.as_ref() {
                                        Some(node_rc) => {
                                            let node_rc = if let NodeRc::VirtualNode(node_rc) = node_rc { node_rc } else { unreachable!() };
                                            let node = unsafe { node_rc.borrow_mut_unsafe_with(__owner) };
                                            let mut node2 = node_rc.borrow_mut_with(__owner);
                                            let old_keys: &VirtualKeyList<()> = if let VirtualNodeProperty::List(list) = node.property() {
                                                list.downcast_ref::<VirtualKeyList<()>>().unwrap()
                                            } else { unreachable!() };
                                            keys.list_reorder(old_keys, &mut node2)
                                        },
                                        None => {
                                            VirtualKeyChanges::new_empty(keys.len())
                                        },
                                    };
                                    (keys, reordered_list)
                                }
                            },
                        };
                        quote! {
                            |__owner: &mut ComponentNodeRefMut<_>, __update_to: Option<&NodeRc<_>>| -> NodeRc<_> {
                                let (__keys, mut __reordered_list) = #key_list;

                                let children: Vec<_> = (#list).into_iter().enumerate().map(|(#index, #item)| -> NodeRc<_> {
                                    let __node_rc = __reordered_list.nodes_mut()[#index].as_ref().map(|node_rc| if let NodeRc::VirtualNode(node_rc) = node_rc { node_rc } else { unreachable!() });
                                    let __node = __node_rc.as_ref().map(|node_rc| unsafe { node_rc.borrow_mut_unsafe_with(__owner) });
                                    let __children = __node.as_ref().map(|node| { node.children() });
                                    let children: Vec<NodeRc<_>> = vec![#(
                                        (#children)(__owner, if let Some(children) = __children {
                                            Some(&children[#indexes])
                                        } else { None })
                                    ),*];
                                    match __node_rc {
                                        None => __owner.new_virtual_node("for-item", VirtualNodeProperty::None, children).into(),
                                        Some(node_rc) => node_rc.clone().into(),
                                    }
                                }).collect();

                                match __update_to.as_ref() {
                                    None => __owner.new_virtual_node("for-list", VirtualNodeProperty::List(__keys), children).into(),
                                    Some(node_rc) => {
                                        let node_rc = if let NodeRc::VirtualNode(node_rc) = node_rc { node_rc } else { unreachable!() };
                                        let mut node = node_rc.borrow_mut_with(__owner);
                                        __reordered_list.apply(&mut node, children);
                                        *node.property_mut() = VirtualNodeProperty::List(__keys);
                                        node_rc.clone().into()
                                    }
                                }
                            }
                        }
                    },
                }
            },

            TemplateNode::Component(x) => {
                let TemplateComponent { tag_name, component, property_values, children } = x;
                let indexes: Vec<usize> = (0..children.len()).into_iter().map(|x| x).collect();
                let property_apply: Vec<TokenStream2> = property_values.iter().map(|attribute| {
                    let content = match attribute {
                        Attribute::Mark { value } => quote! {
                            node.as_node().set_mark(#value);
                        },
                        Attribute::ClassProp { value } => quote! {
                            node.as_node().set_attribute("class", __prepend_class_prefix(#value, __template_skin_prefix));
                        },
                        Attribute::Common { name, value } => quote! {
                            node.as_node().set_attribute(#name, #value);
                        },
                        Attribute::Prop { name, value } => quote! {
                            if Property::update_from(&mut node.#name, #value) { changed = true };
                        },
                        Attribute::SystemEv { name, value } => quote! {
                            node.as_node().global_events_mut().#name.set_handler(Box::new(|self_ref_mut, e| {
                                let f: Box<dyn Fn(ComponentRefMut<B, Self>, _)> = Box::new(#value);
                                f(self_ref_mut.duplicate().with_type::<Self>(), e)
                            }));
                        },
                        Attribute::Ev { name, value } => quote! {
                            node.#name.set_handler(Box::new(|self_ref_mut, e| {
                                let f: Box<dyn Fn(ComponentRefMut<B, Self>, _)> = Box::new(#value);
                                f(self_ref_mut.duplicate().with_type::<Self>(), e)
                            }));
                        },
                    };
                    if attribute.is_dynamic() {
                        quote! { #content }
                    } else {
                        quote! { if is_init { #content } }
                    }
                }).collect();
                quote! {
                    |__owner: &mut ComponentNodeRefMut<_>, __update_to: Option<&NodeRc<_>>| {
                        let __node_rc = __update_to.map(|node_rc| if let NodeRc::ComponentNode(node_rc) = node_rc { node_rc } else { unreachable!() });
                        let __node = __node_rc.as_ref().map(|node_rc| unsafe { node_rc.borrow_mut_unsafe_with(__owner) });
                        let __children = __node.as_ref().map(|node| { node.children() });
                        let ret_children: Vec<NodeRc<_>> = vec![#(
                            (#children)(__owner, if let Some(children) = __children { Some(&children[#indexes]) } else { None })
                        ),*];
                        let is_init = __node_rc.is_none();
                        let node_rc = match __node_rc {
                            None => __owner.new_component_node::<#component>(#tag_name, ret_children),
                            Some(node_rc) => node_rc.clone(),
                        };
                        {
                            let mut changed = false;
                            let mut node = node_rc.borrow_mut_with(__owner).with_type::<#component>();
                            {
                                #(#property_apply)*
                            }
                            if changed { node.force_apply_updates() };
                        }
                        node_rc.into()
                    }
                }
            },

            TemplateNode::TextNode(x) => {
                // text node logic
                let update = {
                    if x.is_dynamic {
                        quote! {
                            let node_rc = if let NodeRc::TextNode(node_rc) = node_rc { node_rc } else { unreachable!() };
                            node_rc.borrow_mut_with(__owner).set_text_content(#x);
                            node_rc.clone().into()
                        }
                    } else {
                        quote! { node_rc.clone() }
                    }
                };
                quote! {
                    |__owner: &mut ComponentNodeRefMut<_>, __update_to: Option<&NodeRc<_>>| {
                        match __update_to {
                            None => {
                                __owner.new_text_node(#x).into()
                            },
                            Some(node_rc) => {
                                #update
                            },
                        }
                    }
                }
            },
        };
        tokens.append_all(node);
    }
}

#[derive(Clone)]
pub(crate) struct TemplateShadowRoot {
    pub(crate) children: Vec<TemplateNode>
}
impl ToTokens for TemplateShadowRoot {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let TemplateShadowRoot { children } = self;
        let indexes: Vec<usize> = (0..children.len()).into_iter().map(|x| x).collect();
        tokens.append_all(quote! {
            |__owner: &mut ComponentNodeRefMut<_>, __update_to: Option<&VirtualNodeRc<_>>, __template_skin_prefix: &'static str| {
                // shadow root node logic
                let __node = __update_to.map(|node_rc| unsafe { node_rc.borrow_mut_unsafe_with(__owner) });
                let __children = __node.as_ref().map(|node| { node.children() });
                vec![#(
                    (#children)(__owner, if let Some(children) = __children { Some(&children[#indexes]) } else { None })
                ),*]
            }
        });
    }
}
