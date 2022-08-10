use maomi::backend::tree::*;

use crate::DomGeneralElement;

pub(crate) enum ChildFrag {
    None,
    Single(web_sys::Node),
    Multi(web_sys::DocumentFragment),
}

impl ChildFrag {
    fn new() -> Self {
        Self::None
    }

    fn add(&mut self, n: &web_sys::Node) {
        match self {
            Self::None => {
                *self = Self::Single(n.clone());
            }
            Self::Single(prev) => {
                let frag = crate::DOCUMENT.with(|document| document.create_document_fragment());
                frag.append_child(prev).map(|_| ()).unwrap_or_else(|x| {
                    crate::log_js_error(&x);
                });
                frag.append_child(&n).map(|_| ()).unwrap_or_else(|x| {
                    crate::log_js_error(&x);
                });
                *self = Self::Multi(frag);
            }
            Self::Multi(frag) => {
                frag.append_child(&n).map(|_| ()).unwrap_or_else(|x| {
                    crate::log_js_error(&x);
                });
            }
        }
    }

    pub(crate) fn dom(&self) -> Option<&web_sys::Node> {
        match self {
            Self::None => None,
            Self::Single(x) => Some(x),
            Self::Multi(x) => Some(&x),
        }
    }
}

pub(crate) fn remove_all_children<'a>(
    parent: &web_sys::Node,
    n: ForestNode<'a, DomGeneralElement>,
) {
    fn rec<'a>(parent: &web_sys::Node, n: &ForestNode<'a, DomGeneralElement>) {
        match &**n {
            DomGeneralElement::DomElement(x) => {
                parent
                    .remove_child(x.dom())
                    .map(|_| ())
                    .unwrap_or_else(|x| {
                        crate::log_js_error(&x);
                    });
            }
            DomGeneralElement::DomText(x) => {
                parent
                    .remove_child(x.dom())
                    .map(|_| ())
                    .unwrap_or_else(|x| {
                        crate::log_js_error(&x);
                    });
            }
            DomGeneralElement::VirtualElement(_) => {
                let mut cur_option = n.first_child();
                while let Some(cur) = cur_option {
                    rec(parent, &cur);
                    cur_option = cur.next_sibling();
                }
            }
        }
    }
    rec(parent, &n);
}

pub(crate) fn collect_child_frag<'a>(n: ForestNode<'a, DomGeneralElement>) -> ChildFrag {
    fn rec<'a>(n: ForestNode<'a, DomGeneralElement>, ret: &mut ChildFrag) {
        match &*n {
            DomGeneralElement::DomElement(x) => {
                return ret.add(&x.dom());
            }
            DomGeneralElement::DomText(x) => {
                return ret.add(&x.dom());
            }
            DomGeneralElement::VirtualElement(_) => {
                let mut cur_option = n.first_child();
                while let Some(cur) = cur_option {
                    rec(cur.clone(), ret);
                    cur_option = cur.next_sibling();
                }
            }
        }
    }
    let mut ret = ChildFrag::new();
    rec(n, &mut ret);
    ret
}

pub(crate) fn find_nearest_dom_ancestor<'a>(
    n: ForestNode<'a, DomGeneralElement>,
) -> Option<web_sys::Node> {
    let mut cur_rc = n.rc();
    loop {
        let next = {
            let cur = n.borrow(&cur_rc);
            match &*cur {
                DomGeneralElement::DomElement(x) => {
                    return Some(x.dom().clone());
                }
                DomGeneralElement::DomText(x) => {
                    return Some(x.dom().clone());
                }
                DomGeneralElement::VirtualElement(_) => {
                    if let Some(x) = cur.parent_rc() {
                        x
                    } else {
                        break;
                    }
                }
            }
        };
        cur_rc = next;
    }
    return None;
}

fn find_first_dom_child<'a>(parent: ForestNode<'a, DomGeneralElement>) -> Option<web_sys::Node> {
    match &*parent {
        DomGeneralElement::DomElement(x) => {
            return Some(x.dom().clone());
        }
        DomGeneralElement::DomText(x) => {
            return Some(x.dom().clone());
        }
        DomGeneralElement::VirtualElement(_) => {
            let mut cur_option = parent.first_child();
            while let Some(cur) = cur_option {
                if let Some(x) = find_first_dom_child(cur.clone()) {
                    return Some(x);
                }
                cur_option = cur.next_sibling();
            }
        }
    }
    return None;
}

pub(crate) fn find_next_dom_sibling<'a>(
    n: ForestNode<'a, DomGeneralElement>,
) -> Option<web_sys::Node> {
    let mut cur_rc = n.rc();
    loop {
        let next = {
            let cur = n.borrow(&cur_rc);
            if let Some(next) = cur.next_sibling_rc() {
                if let Some(x) = find_first_dom_child(n.borrow(&next)) {
                    return Some(x);
                }
                next
            } else if let Some(parent) = cur.parent_rc() {
                match &*cur {
                    DomGeneralElement::DomElement(_) | DomGeneralElement::DomText(_) => {
                        break;
                    }
                    DomGeneralElement::VirtualElement(_) => parent,
                }
            } else {
                break;
            }
        };
        cur_rc = next;
    }
    return None;
}
