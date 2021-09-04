use std::rc::Rc;

use super::backend::{Backend, BackendElement, BackendTextNode};
use super::node::*;

pub(crate) fn match_prerendered_tree<B: Backend>(
    root: &mut ComponentNodeRefMut<B>,
    backend: &Rc<B>,
) {
    backend.match_prerendered_root_element(&mut root.backend_element);

    struct ChildIterState<B: Backend> {
        prev: NodeRc<B>,
        next_is_child: bool,
    }

    fn match_prerendered_children<B: Backend>(
        children: &mut ChildIterMut<B>,
        child_iter_state: &mut ChildIterState<B>,
    ) {
        while let Some(mut child) = children.next() {
            {
                {
                    if let NodeMut::VirtualNode(n) = child {
                        let mut c = n.composed_children_mut();
                        match_prerendered_children(&mut c, child_iter_state);
                        continue;
                    }
                }
                match &child_iter_state.prev {
                    NodeRc::NativeNode(n) => {
                        let n = unsafe { n.deref_mut_unsafe() };
                        if child_iter_state.next_is_child {
                            n.backend_element
                                .match_prerendered_first_child(child.backend_node_mut().unwrap());
                        } else {
                            n.backend_element
                                .match_prerendered_next_sibling(child.backend_node_mut().unwrap());
                        }
                        let prev = child.rc();
                        let mut c = child.composed_children_mut();
                        match_prerendered_children(
                            &mut c,
                            &mut ChildIterState {
                                prev,
                                next_is_child: true,
                            },
                        );
                    }
                    NodeRc::VirtualNode(_) => {
                        unreachable!()
                    }
                    NodeRc::ComponentNode(n) => {
                        let n = unsafe { n.deref_mut_unsafe() };
                        if child_iter_state.next_is_child {
                            n.backend_element
                                .match_prerendered_first_child(child.backend_node_mut().unwrap());
                        } else {
                            n.backend_element
                                .match_prerendered_next_sibling(child.backend_node_mut().unwrap());
                        }
                        let prev = child.rc();
                        let mut c = child.composed_children_mut();
                        match_prerendered_children(
                            &mut c,
                            &mut ChildIterState {
                                prev,
                                next_is_child: true,
                            },
                        );
                    }
                    NodeRc::TextNode(n) => {
                        let n = unsafe { n.deref_mut_unsafe() };
                        if child_iter_state.next_is_child {
                            unreachable!();
                        } else {
                            n.backend_element
                                .match_prerendered_next_sibling(child.backend_node_mut().unwrap());
                        }
                    }
                };
            }
            child_iter_state.prev = child.rc();
            child_iter_state.next_is_child = false;
        }
    }

    let root_rc = root.rc().into();
    let mut c = root.composed_children_mut();
    match_prerendered_children(
        &mut c,
        &mut ChildIterState {
            prev: root_rc,
            next_is_child: true,
        },
    );
}
