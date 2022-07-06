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

    fn add(&mut self, n: web_sys::Node) {
        match self {
            Self::None => {
                *self = Self::Single(n);
            }
            Self::Single(prev) => {
                let frag = crate::DOCUMENT.with(|document| document.create_document_fragment());
                frag.append_child(prev).unwrap();
                frag.append_child(&n).unwrap();
                *self = Self::Multi(frag);
            }
            Self::Multi(frag) => {
                frag.append_child(&n).unwrap();
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

pub(crate) fn collect_child_frag<'a>(n: ForestNode<'a, DomGeneralElement>) -> ChildFrag {
    fn rec<'a>(n: ForestNode<'a, DomGeneralElement>, ret: &mut ChildFrag) {
        match &*n {
            DomGeneralElement::DomElement(x) => {
                return ret.add(x.dom().clone());
            }
            DomGeneralElement::DomText(x) => {
                return ret.add(x.dom().clone());
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
    let mut cur = n;
    loop {
        match &*cur {
            DomGeneralElement::DomElement(x) => {
                return Some(x.dom().clone());
            }
            DomGeneralElement::DomText(x) => {
                return Some(x.dom().clone());
            }
            DomGeneralElement::VirtualElement(_) => {
                if let Some(x) = cur.parent() {
                    cur = x;
                } else {
                    break;
                }
            }
        }
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
    let mut cur = n;
    loop {
        if let Some(next) = cur.next_sibling() {
            if let Some(x) = find_first_dom_child(next.clone()) {
                return Some(x);
            }
            cur = next;
        } else if let Some(parent) = cur.parent() {
            match &*cur {
                DomGeneralElement::DomElement(_) | DomGeneralElement::DomText(_) => {
                    break;
                }
                DomGeneralElement::VirtualElement(_) => {
                    cur = parent;
                    continue;
                }
            }
        } else {
            break;
        }
    }
    return None;
}
