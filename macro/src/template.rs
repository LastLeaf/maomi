use proc_macro2::TokenStream as TokenStream2;
use syn::*;
use quote::*;

fn is_expr_dynamic(expr: &Expr) -> bool {
    if let Expr::Lit(_) = expr {
        true
    } else {
        false
    }
}

#[derive(Clone)]
pub(crate) struct TemplateValue {
    pub(crate) is_dynamic: bool,
    pub(crate) expr: syn::Expr,
}
impl ToTokens for TemplateValue {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let expr = &self.expr;
        tokens.append_all(quote! {
            #expr
        })
    }
}
impl From<syn::Expr> for TemplateValue {
    fn from(expr: Expr) -> Self {
        Self {
            is_dynamic: is_expr_dynamic(&expr),
            expr,
        }
    }
}

#[derive(Clone)]
pub(crate) struct TemplateNativeNode {
    pub(crate) tag_name: syn::LitStr,
    pub(crate) attributes: Vec<(syn::LitStr, TemplateValue)>,
    pub(crate) children: Vec<TemplateNode>
}

#[derive(Clone)]
pub(crate) enum TemplateVirtualNode {
    Slot { name: Option<Expr> },
    If { branches: Vec<(Option<Expr>, Vec<TemplateNode>)> },
    For { list: Expr, index: Ident, item: Ident, key: Option<(Ident, Path)>, children: Vec<TemplateNode> },
}

#[derive(Clone)]
pub(crate) struct TemplateComponent {
    pub(crate) tag_name: syn::Ident,
    pub(crate) property_values: Vec<(syn::Ident, TemplateValue)>,
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
                let TemplateNativeNode { tag_name, attributes, children } = x;
                let init_attributes: Vec<TokenStream2> = attributes.iter().map(|(name, value)| {
                    quote! {
                        (#name, (#value).into())
                    }
                }).collect();
                let update_attributes: Vec<TokenStream2> = attributes.iter().map(|(name, value)| {
                    if value.is_dynamic {
                        quote! {
                            (#name, (#value).into())
                        }
                    } else {
                        quote! { }
                    }
                }).collect();
                quote! {
                    {
                        let children = [#(#children),*];
                        (|owner: ComponentRefMut<B>| {
                            let children: Vec<NodeRc<B>> = children.iter().map(|(init_fn, _)| {
                                init_fn(owner)
                            }).collect();
                            let native_node_rc = owner.new_native_node(#tag_name, vec![#(#init_attributes),*], children);
                            native_node_rc.into()
                        }, |owner: ComponentRefMut<B>, node_rc: &NodeRc<B>| {
                            let children_nodes: &Vec<NodeRc<B>> = unsafe { parent_rc.borrow_mut_unsafe_with(owner).children() };
                            for (index, (_, update_fn)) in children.iter().enumerate() {
                                update_fn(owner, children_nodes[index])
                            }
                            if let NodeRc::NativeNodeRc(native_node_rc) = node_rc {
                                let mut native_node = native_node_rc.borrow_mut_with(owner);
                                native_node.update_ordered_attributes(vec![#(#update_attributes),*]);
                            } else {
                                unreachable!()
                            }
                        })
                    }
                }
            },
            TemplateNode::VirtualNode(x) => {
                match x {
                    TemplateVirtualNode::Slot { name } => {
                        let slot_name = match name {
                            None => quote! { None },
                            Some(x) => quote! { Some(#x) },
                        };
                        quote! {
                            {
                                (|owner: ComponentRefMut<B>| {
                                    owner.new_virtual_node("slot", #slot_name, vec![])
                                }, |_owner: ComponentRefMut<B>, _node_rc: &NodeRc<B>| {
                                    // empty
                                })
                            }
                        }
                    },
                    TemplateVirtualNode::If { branches } => {
                        let children_branches: Vec<_> = branches.iter().enumerate().map(|(key, (cond, children))| {
                            match cond {
                                Some(cond) => quote! {
                                    if #cond { (#key, vec![#(#children),*]) }
                                },
                                None => quote! {
                                    { (#key, vec![#(#children),*]) }
                                }
                            }
                        }).collect();
                        quote! {
                            let (key, children) = #(#children_branches)else*;
                            __if_fn(key, children)
                        }
                    },
                    TemplateVirtualNode::For { list, index, item, key, children } => {
                        let key_list = match key {
                            None => quote! { None },
                            Some((field, ty)) => {
                                quote! {
                                    {
                                        let v: Vec<#ty> = (#list).iter().map(|x| {
                                            let key: #ty = x.#field.clone();
                                            key
                                        }).collect();
                                        Some(Box::new(v))
                                    }
                                }
                            }
                        };
                        let old_key_list = match key {
                            None => quote! { None },
                            Some((_, ty)) => {
                                quote! {
                                    {
                                        let v: Vec<#ty> = node.key().unwrap().downcast_ref();
                                        Some(Box::new(v))
                                    }
                                }
                            }
                        };
                        quote! {
                            let children: Vec<Node> = vec![#(#children),*];
                            let key_list = #key_list;
                            (|owner: ComponentRefMut<B>| {
                                let items: Vec<Node> = (#list).into_iter().enumerate().map(|(#index, #item)| {
                                    owner.new_virtual_node("for-item", None, children) // TODO use key
                                }).collect();
                                let node_rc = owner.new_virtual_node("if", key_list, children);
                                node_rc.into()
                            }, |owner: ComponentRefMut<B>, node_rc: &NodeRc<B>| {
                                if let NodeRc::VirtualNodeRc(node_rc) = node_rc {
                                    let mut node = node_rc.borrow_mut_with(owner);
                                    let old_key_list = #old_key_list;
                                    list_diff(&mut node, &key_list, old_key_list, children);


                                    if node.key() !== key {
                                        node.set_key(Some(key));
                                        node.set_children(children.iter().map(|(init_fn, _)| {
                                            init_fn(owner)
                                        }).collect());
                                    } else {
                                        let children_nodes: &Vec<NodeRc<B>> = unsafe { parent_rc.borrow_mut_unsafe_with(owner).children() };
                                        for (index, (_, update_fn)) in children.iter().enumerate() {
                                            update_fn(owner, children_nodes[index])
                                        }
                                    }
                                } else {
                                    unreachable!()
                                }
                            })
                        }
                    },
                }
            },
            TemplateNode::Component(x) => {
                let TemplateComponent { tag_name, property_values, children } = x;
                let property_apply: Vec<TokenStream2> = property_values.iter().map(|(name, value)| {
                    let name = format_ident!("set_property_{}", name);
                    quote! {
                        component.#name(#value);
                    }
                }).collect();
                let dynamic_property_apply: Vec<TokenStream2> = property_values.iter().map(|(name, value)| {
                    let name = format_ident!("set_property_{}", name);
                    if value.is_dynamic {
                        quote! {
                            component.#name(#value);
                        }
                    } else {
                        quote! { }
                    }
                }).collect();
                quote! {
                    {
                        let children = [#(#children),*];
                        (|owner: ComponentRefMut<B>| {
                            let children: Vec<NodeRc<B>> = children.iter().map(|(init_fn, _)| {
                                init_fn(owner)
                            }).collect();
                            let mut component = Box::new(#tag_name::new());
                            #(#property_apply)*
                            let component_node_rc = owner.new_component_node(component, children);
                            component_node_rc.into()
                        }, |owner: ComponentRefMut<B>, node_rc: &NodeRc<B>| {
                            let children_nodes: &Vec<NodeRc<B>> = unsafe { parent_rc.borrow_mut_unsafe_with(owner).children() };
                            for (index, (_, update_fn)) in children.iter().enumerate() {
                                update_fn(owner, children_nodes[index])
                            }
                            if let NodeRc::ComponentNodeRc(component_node_rc) = node_rc {
                                let mut component_node = component_node_rc.borrow_mut_with(owner);
                                {
                                    let mut component = component.as_component_mut::<#tag_name>();
                                    #(#dynamic_property_apply)*
                                }
                                $tag_name::update_now(&mut component_node);
                            } else {
                                unreachable!()
                            }
                        })
                    }
                }
            },
            TemplateNode::TextNode(x) => {
                quote! {
                    || {
                        Node::TextNode((#x).into())
                    }
                }
            },
        };
        tokens.append_all(quote! {
            (#node)()
        });
    }
}

#[derive(Clone)]
pub(crate) struct TemplateShadowRoot {
    pub(crate) children: Vec<TemplateNode>
}
impl ToTokens for TemplateShadowRoot {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let TemplateShadowRoot { children } = self;
        tokens.append_all(quote! {
            {
                let children = [#(#children),*];
                (|owner: ComponentRefMut<B>| {
                    let children: Vec<NodeRc<B>> = children.iter().map(|(init_fn, _)| {
                        init_fn(owner)
                    }).collect();
                    children
                }, |owner: ComponentRefMut<B>, parent_rc: &VirtualNodeRc<B>| {
                    let children_nodes: &Vec<NodeRc<B>> = unsafe { parent_rc.borrow_mut_unsafe_with(owner).children() };
                    for (index, (_, update_fn)) in children.iter().enumerate() {
                        update_fn(owner, children_nodes[index])
                    }
                })
            }
        });
    }
}
