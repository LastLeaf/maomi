pub use maomi_macro::*;
pub use super::{Component, ComponentTemplate, backend::Backend, node::*, virtual_key::list_diff};

pub fn __if_fn<B: Backend, F, G, H>(f: F) -> (impl Fn(&mut ComponentNodeRefMut<B>) -> NodeRc<B>, impl Fn(&mut ComponentNodeRefMut<B>, &NodeRc<B>))
where
    F: Fn() -> (usize, Vec<(G, H)>),
    G: Fn(&mut ComponentNodeRefMut<B>) -> NodeRc<B>,
    H: Fn(&mut ComponentNodeRefMut<B>, &NodeRc<B>)
{
    (|owner: &mut ComponentNodeRefMut<B>| {
        let (key, children) = f();
        let children: Vec<NodeRc<B>> = children.iter().map(|(init_fn, _)| {
            init_fn(owner)
        }).collect();
        let node_rc = owner.new_virtual_node("if", Some(Box::new(key)), children);
        node_rc.into()
    }, |owner: &mut ComponentNodeRefMut<B>, node_rc: &NodeRc<B>| {
        let (key, children) = f();
        if let NodeRc::VirtualNode(node_rc) = node_rc {
            let mut node = unsafe { node_rc.borrow_mut_unsafe_with(owner) };
            if *node.key().as_ref().unwrap().downcast_ref::<usize>().unwrap() != key {
                node.set_key(Some(Box::new(key)));
                node.set_children(children.iter().map(|(init_fn, _)| {
                    init_fn(owner)
                }).collect());
            } else {
                let node = unsafe { node_rc.borrow_mut_unsafe_with(owner) };
                let children_nodes = node.children();
                for (index, (_, update_fn)) in children.iter().enumerate() {
                    update_fn(owner, &children_nodes[index])
                }
            }
        } else {
            unreachable!()
        }
    })
}
