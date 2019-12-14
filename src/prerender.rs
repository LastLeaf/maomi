use std::rc::Rc;

use super::node::*;
use super::backend::{Backend, BackendElement, BackendTextNode};

pub(crate) struct PrerenderWriter {
    buf: Vec<u8>,
}

impl PrerenderWriter {
    pub(crate) fn new() -> Self {
        Self {
            buf: vec![],
        }
    }
    pub(crate) fn append(&mut self, mut data: Vec<u8>) {
        let buf = &mut self.buf;
        let mut len = data.len();
        while len > 0 {
            let low = len % 256;
            buf.push(low as u8);
            len = len >> 8;
        }
        buf.push(0);
        buf.append(&mut data);
    }
    pub(crate) fn end(self) -> Vec<u8> {
        self.buf
    }
}

pub struct PrerenderReader {
    buf: Box<[u8]>,
    pos: usize,
}

impl PrerenderReader {
    pub(crate) fn new(buf: Box<[u8]>) -> Self {
        Self {
            buf,
            pos: 0,
        }
    }
    pub(crate) fn next<'b>(&'b mut self) -> &'b [u8] {
        let buf = &self.buf;
        let mut pos = self.pos;
        let mut len = 0usize;
        let mut ml = 0;
        loop {
            let b = buf[pos] as usize;
            pos += 1;
            if b == 0 {
                break;
            }
            len += b << ml;
            ml += 8;
        }
        let ret = &buf[pos..(pos + len)];
        self.pos += pos + len;
        ret
    }
}

pub(crate) fn match_prerendered_tree<B: Backend>(mut root: ComponentNodeRefMut<B>, backend: &Rc<B>) {
    backend.match_prerendered_root_element(&mut root.backend_element);
    struct ChildIterState<B: Backend> {
        prev: NodeRc<B>,
        next_is_child: bool,
    }
    fn match_prerendered_children<B: Backend>(rm: &mut NodeRefMut<B>, children: Vec<NodeRc<B>>, child_iter_state: &mut ChildIterState<B>) {
        for child_rc in children {
            {
                let mut child = unsafe { child_rc.borrow_mut_unsafe_with(rm) };
                {
                    if let NodeRefMut::VirtualNode(n) = child {
                        let c = n.composed_children();
                        match_prerendered_children(rm, c, child_iter_state);
                        continue
                    }
                }
                match &child_iter_state.prev {
                    NodeRc::NativeNode(n) => {
                        let mut n = unsafe { n.borrow_mut_unsafe_with(rm) };
                        if child_iter_state.next_is_child {
                            n.backend_element.match_prerendered_first_child(child.backend_node_mut().unwrap());
                        } else {
                            n.backend_element.match_prerendered_next_sibling(child.backend_node_mut().unwrap());
                        }
                        let c = child.to_ref().composed_children();
                        match_prerendered_children(rm, c, &mut ChildIterState { prev: child_rc.clone(), next_is_child: true });
                    },
                    NodeRc::VirtualNode(_) => {
                        unreachable!()
                    },
                    NodeRc::ComponentNode(n) => {
                        let mut n = unsafe { n.borrow_mut_unsafe_with(rm) };
                        if child_iter_state.next_is_child {
                            n.backend_element.match_prerendered_first_child(child.backend_node_mut().unwrap());
                        } else {
                            n.backend_element.match_prerendered_next_sibling(child.backend_node_mut().unwrap());
                        }
                        let c = child.to_ref().composed_children();
                        match_prerendered_children(rm, c, &mut ChildIterState { prev: child_rc.clone(), next_is_child: true });
                    },
                    NodeRc::TextNode(n) => {
                        let mut n = unsafe { n.borrow_mut_unsafe_with(rm) };
                        if child_iter_state.next_is_child {
                            unreachable!();
                        } else {
                            n.backend_element.match_prerendered_next_sibling(child.backend_node_mut().unwrap());
                        }
                    },
                };
            }
            child_iter_state.prev = child_rc;
            child_iter_state.next_is_child = false;
        }
    }
    let c = root.composed_children();
    let root_rc = root.rc().into();
    match_prerendered_children(&mut root.into(), c, &mut ChildIterState { prev: root_rc, next_is_child: true });
}
