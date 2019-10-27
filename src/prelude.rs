pub use maomi_macro::*;
pub use super::{Component, ComponentTemplate, EmptyComponent, ComponentContext, Prop, Ev, backend::Backend, node::*, virtual_key::*};

fn __shadow_root_sample<B: Backend>(__owner: &mut ComponentNodeRefMut<B>, __update_to: Option<&VirtualNodeRc<B>>) -> Vec<NodeRc<B>> {
    struct SampleData {
        list: Vec<ForSampleData>
    }
    struct ForSampleData {
        id: i32,
        value: String
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
    let for_node_sample = |__owner: &mut ComponentNodeRefMut<B>, __update_to: Option<&NodeRc<B>>| -> NodeRc<B> {
        let (__keys, mut __reordered_list) = {
            let keys: Box<VirtualKeyList<i32>> = {
                let v: Vec<Option<i32>> = data.list.iter().map(|x| {
                    Some(x.id.clone())
                }).collect();
                let v = VirtualKeyList::new(v);
                let keys = Box::new(v);
                keys
            };
            let reordered_list: VirtualKeyChanges<B> = match __update_to.as_ref() {
                Some(node_rc) => {
                    let node_rc = if let NodeRc::VirtualNode(node_rc) = node_rc { node_rc } else { unreachable!() };
                    let node = unsafe { node_rc.borrow_mut_unsafe_with(__owner) };
                    let mut node2 = node_rc.borrow_mut_with(__owner);
                    let old_keys: &VirtualKeyList<i32> = if let VirtualNodeProperty::List(list) = node.property() {
                        list.downcast_ref::<VirtualKeyList<i32>>().unwrap()
                    } else { unreachable!() };
                    keys.list_reorder(old_keys, &mut node2)
                },
                None => {
                    VirtualKeyChanges::new_empty(data.list.len())
                },
            };
            (keys, reordered_list)
        };

        let children: Vec<_> = data.list.iter().enumerate().map(|(index, item)| -> NodeRc<B> {

            // if node logic
            let if_node_sample = |__owner: &mut ComponentNodeRefMut<B>, __update_to: Option<&NodeRc<B>>| -> NodeRc<B> {

                // native node logic
                let native_node_sample = |__owner: &mut ComponentNodeRefMut<B>, __update_to: Option<&NodeRc<B>>| -> NodeRc<B> {

                    // text node logic
                    let text_node_sample = |__owner: &mut ComponentNodeRefMut<B>, __update_to: Option<&NodeRc<B>>| -> NodeRc<B> {

                        match __update_to {
                            None => {
                                __owner.new_text_node(item.value.clone()).into()
                            },
                            Some(node_rc) => {
                                let node_rc = if let NodeRc::TextNode(node_rc) = node_rc { node_rc } else { unreachable!() };
                                node_rc.borrow_mut_with(__owner).set_text_content(item.value.clone());
                                node_rc.clone().into()
                            },
                        }
                    };

                    let node_rc = __update_to.map(|node_rc| if let NodeRc::NativeNode(node_rc) = node_rc { node_rc } else { unreachable!() });
                    let node = node_rc.as_ref().map(|node_rc| unsafe { node_rc.borrow_mut_unsafe_with(__owner) });
                    let children = node.as_ref().map(|node| { node.children() });
                    let ret_children: Vec<NodeRc<B>> = vec![text_node_sample(__owner, if let Some(children) = children { Some(&children[0]) } else { None })];
                    let node_rc = match node_rc {
                        None => __owner.new_native_node("div", vec![], "".into(), ret_children),
                        Some(node_rc) => node_rc.clone(),
                    };
                    {
                        // let mut node = node_rc.borrow_mut_with(__owner);
                        // node.set_attribute("data-xxx", "xxx");
                    }
                    node_rc.into()
                };

                // native node logic
                let component_node_sample = |__owner: &mut ComponentNodeRefMut<B>, __update_to: Option<&NodeRc<B>>| -> NodeRc<B> {

                    // text node logic
                    let slot_node_sample = |__owner: &mut ComponentNodeRefMut<B>, __update_to: Option<&NodeRc<B>>| -> NodeRc<B> {

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
                    let node = node_rc.as_ref().map(|node_rc| unsafe { node_rc.borrow_mut_unsafe_with(__owner) });
                    let children = node.as_ref().map(|node| { node.children() });
                    let ret_children: Vec<NodeRc<B>> = vec![slot_node_sample(__owner, if let Some(children) = children { Some(&children[0]) } else { None })];
                    let node_rc = match node_rc {
                        None => __owner.new_component_node::<EmptyComponent>("maomi-default-component", "".into(), ret_children),
                        Some(node_rc) => node_rc.clone(),
                    };
                    {
                        // let mut changed = false;
                        // let mut node = node_rc.borrow_mut_with(__owner);
                        // {
                        //     let node = node.as_component_mut::<EmptyComponent<B>>();
                        //     if Property::update_from(&mut node.todo, "xxx") { changed = true };
                        // }
                        // if changed { <EmptyComponent<B> as Component>::apply_updates(&mut node) };
                    }
                    node_rc.into()
                };

                let node_rc = __update_to.map(|node_rc| if let NodeRc::VirtualNode(node_rc) = node_rc { node_rc } else { unreachable!() });
                let node = node_rc.as_ref().map(|node_rc| unsafe { node_rc.borrow_mut_unsafe_with(__owner) });
                let old_key = match &node {
                    Some(x) => {
                        let index = if let VirtualNodeProperty::Branch(b) = x.property() { b } else { unreachable!() };
                        Some(index)
                    },
                    None => None,
                };
                let children = node.as_ref().map(|node| { node.children() });
                if item.value.len() > 0 {
                    const KEY: usize = 0;
                    let equal = if let Some(old_key) = old_key { *old_key == KEY } else { false };
                    let children: Vec<NodeRc<B>> = vec![native_node_sample(__owner, if let Some(children) = children {
                        if equal { Some(&children[0]) } else { None }
                    } else { None })];
                    if equal {
                        node_rc.unwrap().clone().into()
                    } else {
                        match node_rc {
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
                } else {
                    const KEY: usize = 1;
                    let equal = if let Some(old_key) = old_key { *old_key == KEY } else { false };
                    let children: Vec<NodeRc<B>> = vec![component_node_sample(__owner, if let Some(children) = children {
                        if equal { Some(&children[0]) } else { None }
                    } else { None })];
                    if equal {
                        node_rc.unwrap().clone().into()
                    } else {
                        match node_rc {
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

            let node_rc = __reordered_list.nodes_mut()[index].as_ref().map(|node_rc| if let NodeRc::VirtualNode(node_rc) = node_rc { node_rc } else { unreachable!() });
            let node = node_rc.as_ref().map(|node_rc| unsafe { node_rc.borrow_mut_unsafe_with(__owner) });
            let children = node.as_ref().map(|node| { node.children() });
            let children: Vec<NodeRc<B>> = vec![if_node_sample(__owner, if let Some(children) = children {
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
                let mut node = node_rc.borrow_mut_with(__owner);
                __reordered_list.apply(&mut node);
                *node.property_mut() = VirtualNodeProperty::List(__keys);
                node_rc.clone().into()
            }
        }
    };

    // shadow root node logic
    let node = __update_to.map(|node_rc| unsafe { node_rc.borrow_mut_unsafe_with(__owner) });
    let children = node.as_ref().map(|node| { node.children() });
    vec![for_node_sample(__owner, if let Some(children) = children { Some(&children[0]) } else { None })]
}

pub fn __template_sample<B: Backend>(__owner: &mut ComponentNodeRefMut<B>, __is_update: bool) -> Option<Vec<NodeRc<B>>> {
    let sr = __owner.shadow_root_rc().clone();
    let ret = __shadow_root_sample(__owner, if __is_update { Some(&sr) } else { None });
    if __is_update { None } else { Some(ret) }
}
