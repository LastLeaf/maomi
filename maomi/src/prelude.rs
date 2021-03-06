pub use maomi_macro::*;
pub use maomi_skin::*;
pub use super::{Component, PrerenderableComponent, ComponentTemplate, ComponentTemplateOperation, EmptyComponent, ComponentContext, ComponentRef, ComponentRefMut, Property, Ev, backend::Backend, node::*, virtual_key::*};

pub fn __prepend_class_prefix<T: ToString>(s: T, prefix: &'static str) -> String {
    use std::fmt::Write;
    let s = s.to_string();
    let slices = s.split_whitespace();
    let mut ret = String::new();
    for slice in slices {
        if ret.len() > 0 {
            ret += " ";
        }
        if &slice[0..1] == "~" {
            ret.write_fmt(format_args!("{}", &slice[1..])).unwrap();
        } else {
            ret.write_fmt(format_args!("{}{}", prefix, slice)).unwrap();
        }
    }
    ret
}

unsafe fn __shadow_root_sample<B: Backend>(__owner: &mut ComponentNodeRefMut<B>, __update_to: Option<&VirtualNodeRc<B>>) -> Vec<NodeRc<B>> {
    struct SampleData {
        list: Vec<ForSampleData>
    }
    struct ForSampleData {
        id: i32,
        value: String,
    }
    let data = SampleData {
        list: vec![ForSampleData {
            id: 1,
            value: "hello".into(),
        }, ForSampleData {
            id: 2,
            value: "world".into(),
        }, ForSampleData {
            id: 3,
            value: "".into(),
        }]
    };

    // for node logic
    let for_node_sample = |__owner: &mut ComponentNodeRefMut<_>, __update_to: Option<&NodeRc<_>>| -> NodeRc<_> {
        let (__keys, mut __reordered_list) = {
            let keys: Box<VirtualKeyList<i32>> = {
                let v: Vec<Option<i32>> = data.list.iter().map(|x| {
                    Some(x.id.clone())
                }).collect();
                let v = VirtualKeyList::new(v);
                let keys = Box::new(v);
                keys
            };
            let reordered_list: VirtualKeyChanges<_> = match __update_to.as_ref() {
                Some(node_rc) => {
                    let node_rc = if let NodeRc::VirtualNode(node_rc) = node_rc { node_rc } else { unreachable!() };
                    let node = unsafe { node_rc.deref_mut_unsafe() };
                    let mut node2 = unsafe { node_rc.deref_mut_unsafe() };
                    let old_keys: &VirtualKeyList<i32> = if let VirtualNodeProperty::List(list) = node.property() {
                        list.downcast_ref::<VirtualKeyList<i32>>().unwrap()
                    } else { unreachable!() };
                    keys.list_reorder(old_keys, &mut node2)
                },
                None => {
                    VirtualKeyChanges::new_empty(keys.len())
                },
            };
            (keys, reordered_list)
        };

        let children: Vec<_> = data.list.iter().enumerate().map(|(index, item)| -> NodeRc<_> {

            // if node logic
            let if_node_sample = |__owner: &mut ComponentNodeRefMut<_>, __update_to: Option<&NodeRc<_>>| -> NodeRc<_> {

                // native node logic
                let native_node_sample = |__owner: &mut ComponentNodeRefMut<_>, __update_to: Option<&NodeRc<_>>| -> NodeRc<_> {

                    // text node logic
                    let text_node_sample = |__owner: &mut ComponentNodeRefMut<_>, __update_to: Option<&NodeRc<_>>| -> NodeRc<_> {

                        match __update_to {
                            None => {
                                __owner.new_text_node(item.value.clone()).into()
                            },
                            Some(node_rc) => {
                                let node_rc = if let NodeRc::TextNode(node_rc) = node_rc { node_rc } else { unreachable!() };
                                unsafe { node_rc.deref_mut_unsafe() }.set_text_content(item.value.clone());
                                node_rc.clone().into()
                            },
                        }
                    };

                    let node_rc = __update_to.map(|node_rc| if let NodeRc::NativeNode(node_rc) = node_rc { node_rc } else { unreachable!() });
                    let node = node_rc.as_ref().map(|node_rc| unsafe { node_rc.deref_mut_unsafe() });
                    let children = node.as_ref().map(|node| { node.children_rc() });
                    let ret_children: Vec<NodeRc<_>> = vec![text_node_sample(__owner, if let Some(children) = children { Some(&children[0]) } else { None })];
                    let node_rc = match node_rc {
                        None => __owner.new_native_node("div", vec![], ret_children),
                        Some(node_rc) => node_rc.clone(),
                    };
                    {
                        // let mut node = unsafe { node_rc.borrow_mut_unsafe() };
                        // node.set_attribute("data-xxx", "xxx");
                        // node.global_events_mut().click.set_handler(Box::new(|self_ref_mut, e| {
                        //     // (|mut self_ref_mut: ComponentRefMut<B, EmptyComponent>, e| {
                        //     //     self_ref_mut.apply_updates()
                        //     // })(self_ref_mut.with_type::<EmptyComponent>(), e)
                        // }));
                    }
                    node_rc.into()
                };

                // component node logic
                let component_node_sample = |__owner: &mut ComponentNodeRefMut<_>, __update_to: Option<&NodeRc<_>>| -> NodeRc<_> {

                    // text node logic
                    let slot_node_sample = |__owner: &mut ComponentNodeRefMut<_>, __update_to: Option<&NodeRc<_>>| -> NodeRc<_> {

                        match __update_to {
                            None => {
                                __owner.new_virtual_node("slot", VirtualNodeProperty::Slot("", vec![]), vec![]).into()
                            },
                            Some(node_rc) => {
                                node_rc.clone()
                            },
                        }
                    };

                    let node_rc = __update_to.map(|node_rc| if let NodeRc::ComponentNode(node_rc) = node_rc { node_rc } else { unreachable!() });
                    let node = node_rc.as_ref().map(|node_rc| unsafe { node_rc.deref_mut_unsafe() });
                    let children = node.as_ref().map(|node| { node.children_rc() });
                    let ret_children: Vec<NodeRc<_>> = vec![slot_node_sample(__owner, if let Some(children) = children { Some(&children[0]) } else { None })];
                    let node_rc = match node_rc {
                        None => __owner.new_component_node::<EmptyComponent>("maomi-default-component", ret_children),
                        Some(node_rc) => node_rc.clone(),
                    };
                    {
                        // let mut changed = false;
                        // let mut node = node_rc.borrow_mut_with(__owner);
                        // {
                        //     let node = node.as_component_mut::<EmptyComponent<_>>();
                        //     if Property::update_from(&mut node.todo, "xxx") { changed = true };
                        // }
                        // if changed { node.apply_updates() };
                    }
                    node_rc.into()
                };

                let node_rc = __update_to.map(|node_rc| if let NodeRc::VirtualNode(node_rc) = node_rc { node_rc } else { unreachable!() });
                let node = node_rc.as_ref().map(|node_rc| unsafe { node_rc.deref_mut_unsafe() });
                let old_key = match &node {
                    Some(x) => {
                        let index = if let VirtualNodeProperty::Branch(b) = x.property() { b } else { unreachable!() };
                        Some(index)
                    },
                    None => None,
                };
                let children = node.as_ref().map(|node| { node.children_rc() });
                if item.value.len() > 0 {
                    const KEY: usize = 0;
                    let __equal = if let Some(old_key) = old_key { *old_key == KEY } else { false };
                    let children: Vec<NodeRc<_>> = vec![native_node_sample(__owner, if let Some(children) = children {
                        if __equal { Some(&children[0]) } else { None }
                    } else { None })];
                    if __equal {
                        node_rc.unwrap().clone().into()
                    } else {
                        match node_rc {
                            Some(node_rc) => {
                                let mut node = unsafe { node_rc.deref_mut_unsafe() };
                                node.replace_children_list(children);
                                *node.property_mut() = VirtualNodeProperty::Branch(KEY);
                                node_rc.clone().into()
                            },
                            None => {
                                __owner.new_virtual_node("if", VirtualNodeProperty::Branch(KEY), children).into()
                            }
                        }
                    }
                } else {
                    const KEY: usize = 1;
                    let __equal = if let Some(old_key) = old_key { *old_key == KEY } else { false };
                    let children: Vec<NodeRc<_>> = vec![component_node_sample(__owner, if let Some(children) = children {
                        if __equal { Some(&children[0]) } else { None }
                    } else { None })];
                    if __equal {
                        node_rc.unwrap().clone().into()
                    } else {
                        match node_rc {
                            Some(node_rc) => {
                                let mut node = unsafe { node_rc.deref_mut_unsafe() };
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

            let node_rc = __reordered_list.nodes_mut()[index].as_ref().map(|node_rc| if let NodeRc::VirtualNode(node_rc) = node_rc { node_rc } else { unreachable!() });
            let node = node_rc.as_ref().map(|node_rc| unsafe { node_rc.deref_mut_unsafe() });
            let children = node.as_ref().map(|node| { node.children_rc() });
            let children: Vec<NodeRc<_>> = vec![if_node_sample(__owner, if let Some(children) = children {
                Some(&children[0])
            } else { None })];
            match node_rc {
                None => __owner.new_virtual_node("for-item", VirtualNodeProperty::None, children).into(),
                Some(node_rc) => node_rc.clone().into(),
            }
        }).collect();

        match __update_to.as_ref() {
            None => __owner.new_virtual_node("for-list", VirtualNodeProperty::List(__keys), children).into(),
            Some(node_rc) => {
                let node_rc = if let NodeRc::VirtualNode(node_rc) = node_rc { node_rc } else { unreachable!() };
                let mut node = unsafe { node_rc.deref_mut_unsafe() };
                __reordered_list.apply(&mut node, children);
                *node.property_mut() = VirtualNodeProperty::List(__keys);
                node_rc.clone().into()
            }
        }
    };

    // shadow root node logic
    let node = __update_to.map(|node_rc| unsafe { node_rc.deref_mut_unsafe() });
    let children = node.as_ref().map(|node| { node.children_rc() });
    vec![for_node_sample(__owner, if let Some(children) = children { Some(&children[0]) } else { None })]
}

pub fn __template_sample<B: Backend>(__owner: &mut ComponentNodeRefMut<B>, __operation: ComponentTemplateOperation) -> Option<Vec<NodeRc<B>>> {
    let sr = __owner.shadow_root_rc().clone();
    let __is_update = if let ComponentTemplateOperation::Update = __operation { true } else { false };
    let __update_to = if __is_update { Some(&sr) } else { None };
    let ret = unsafe { __shadow_root_sample(__owner, __update_to) };
    if __is_update { None } else { Some(ret) }
}
