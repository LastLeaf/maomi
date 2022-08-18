use enum_dispatch::enum_dispatch;
use event::DomListeners;
use maomi::{
    backend::{tree::*, *},
    error::Error,
};
use wasm_bindgen::{JsCast, JsValue};

pub mod base_element;
use base_element::DomElement;
pub mod element;
pub mod virtual_element;
use virtual_element::DomVirtualElement;
pub mod text_node;
use text_node::DomTextNode;
pub mod class_list;
pub mod event;
mod composing;

pub mod prelude {
    pub use crate::DomBackend;
    pub use maomi_dom_macro::dom_css;
}

thread_local! {
    pub(crate) static WINDOW: web_sys::Window = web_sys::window().expect("Cannot init DOM backend outside web page environment");
    pub(crate) static DOCUMENT: web_sys::Document = {
        WINDOW.with(|window| {
            window.document().expect("Cannot init DOM backend when document is not ready")
        })
    };
}

fn log_js_error(err: &JsValue) {
    if let Some(err) = err.dyn_ref::<js_sys::Error>() {
        log::error!("{}", err.message());
    } else {
        log::error!("(JavaScript Error)");
    }
}

/// A common async runner for DOM environment
pub fn async_task(fut: impl 'static + std::future::Future<Output = ()>) {
    wasm_bindgen_futures::spawn_local(fut);
}

/// A DOM backend
pub struct DomBackend {
    tree: tree::ForestNodeRc<DomGeneralElement>,
    #[allow(dead_code)]
    listeners: DomListeners,
}

impl DomBackend {
    pub fn new_with_element(dom_elem: web_sys::Element) -> Result<Self, Error> {
        Ok(Self::wrap_root_element(dom_elem)?)
    }

    pub fn new_with_element_id(id: &str) -> Result<Self, Error> {
        let dom_elem = DOCUMENT
            .with(|document| document.get_element_by_id(id))
            .ok_or_else(|| Error::BackendError {
                msg: format!("Cannot find the element {:?}", id),
                err: None,
            })?;
        Ok(Self::wrap_root_element(dom_elem)?)
    }

    pub fn new_with_document_body() -> Result<Self, Error> {
        let dom_elem =
            DOCUMENT
                .with(|document| document.body())
                .ok_or_else(|| Error::BackendError {
                    msg: "Cannot find the <body> element".into(),
                    err: None,
                })?;
        Ok(Self::wrap_root_element(dom_elem.into())?)
    }

    fn wrap_root_element(dom_elem: web_sys::Element) -> Result<Self, Error> {
        let listeners = event::DomListeners::new(&dom_elem)
            .map_err(|_| Error::BackendError { msg: "Cannot bind event".to_string(), err: None })?;
        let tree_root = {
            let ret = tree::ForestNodeRc::new_forest(DomGeneralElement::DomElement(unsafe {
                DomElement::new(dom_elem)
            }));
            let token = ret.token();
            if let DomGeneralElement::DomElement(x) = &mut *ret.borrow_mut() {
                x.init(token);
            } else {
                unreachable!()
            }
            ret
        };
        Ok(Self {
            tree: tree_root,
            listeners,
        })
    }
}

impl Backend for DomBackend {
    type GeneralElement = DomGeneralElement;
    type VirtualElement = DomVirtualElement;
    type TextNode = DomTextNode;

    fn root(&self) -> ForestNode<Self::GeneralElement> {
        self.tree.borrow()
    }

    fn root_mut(&mut self) -> ForestNodeMut<Self::GeneralElement> {
        self.tree.borrow_mut()
    }
}

#[doc(hidden)]
#[enum_dispatch]
pub trait DomGeneralElementTrait {}

#[doc(hidden)]
#[enum_dispatch(DomGeneralElementTrait)]
pub enum DomGeneralElement {
    VirtualElement(DomVirtualElement),
    DomText(DomTextNode),
    DomElement(DomElement),
}

impl std::fmt::Debug for DomGeneralElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VirtualElement(_) => write!(f, "[Virtual {:p}]", self),
            Self::DomText(x) => write!(f, "{:?}", x.dom().text_content().unwrap_or_default()),
            Self::DomElement(x) => write!(f, "{:?}", x),
        }
    }
}

impl DomGeneralElement {
    fn create_dom_element<'b>(
        this: &'b mut ForestNodeMut<Self>,
        elem: &'b web_sys::Element,
    ) -> ForestNodeRc<DomGeneralElement> {
        let ret = this.new_tree(DomGeneralElement::DomElement(unsafe {
            DomElement::new(elem.clone())
        }));
        let token = ret.token();
        if let DomGeneralElement::DomElement(x) = &mut *this.borrow_mut(&ret) {
            x.init(token);
        } else {
            unreachable!()
        }
        ret
    }

    pub fn as_dom_element_mut<'b>(
        this: &'b mut ForestNodeMut<Self>,
    ) -> Option<ForestValueMut<'b, DomElement>> {
        if let DomGeneralElement::DomElement(_) = &mut **this {
            Some(this.map(|g| {
                if let DomGeneralElement::DomElement(e) = g {
                    e
                } else {
                    unreachable!()
                }
            }))
        } else {
            None
        }
    }

    pub fn inner_html(this: &ForestNode<Self>) -> String {
        match &**this {
            DomGeneralElement::DomText(x) => {
                return x.inner_html();
            }
            DomGeneralElement::DomElement(x) => {
                return x.inner_html();
            }
            DomGeneralElement::VirtualElement(_) => {}
        }
        let mut ret = String::new();
        let mut cur = this.first_child();
        while let Some(c) = &cur {
            ret += Self::outer_html(&c).as_str();
            cur = c.next_sibling();
        }
        ret
    }

    pub fn outer_html(this: &ForestNode<Self>) -> String {
        match &**this {
            DomGeneralElement::DomText(x) => {
                return x.inner_html();
            }
            DomGeneralElement::DomElement(x) => {
                return x.outer_html();
            }
            DomGeneralElement::VirtualElement(_) => {}
        }
        let mut ret = String::new();
        let mut cur = this.first_child();
        while let Some(c) = &cur {
            ret += Self::outer_html(&c).as_str();
            cur = c.next_sibling();
        }
        ret
    }
}

impl BackendGeneralElement for DomGeneralElement {
    type BaseBackend = DomBackend;

    fn as_virtual_element_mut<'b>(
        this: &'b mut ForestNodeMut<Self>,
    ) -> Option<
        ForestValueMut<
            'b,
            <<Self as BackendGeneralElement>::BaseBackend as Backend>::VirtualElement,
        >,
    >
    where
        Self: Sized,
    {
        if let DomGeneralElement::VirtualElement(_) = &mut **this {
            Some(this.map(|g| {
                if let DomGeneralElement::VirtualElement(e) = g {
                    e
                } else {
                    unreachable!()
                }
            }))
        } else {
            None
        }
    }

    fn as_text_node_mut<'b>(
        this: &'b mut ForestNodeMut<Self>,
    ) -> Option<
        ForestValueMut<'b, <<Self as BackendGeneralElement>::BaseBackend as Backend>::TextNode>,
    >
    where
        Self: Sized,
    {
        if let DomGeneralElement::DomText(_) = &mut **this {
            Some(this.map(|g| {
                if let DomGeneralElement::DomText(e) = g {
                    e
                } else {
                    unreachable!()
                }
            }))
        } else {
            None
        }
    }

    fn create_virtual_element<'b>(
        this: &'b mut ForestNodeMut<Self>,
    ) -> Result<ForestNodeRc<<Self::BaseBackend as Backend>::GeneralElement>, Error>
    where
        Self: Sized,
    {
        let elem = DomVirtualElement::new();
        let child = this.new_tree(DomGeneralElement::VirtualElement(elem));
        Ok(child)
    }

    fn create_text_node(
        this: &mut ForestNodeMut<Self>,
        content: &str,
    ) -> Result<ForestNodeRc<<Self::BaseBackend as Backend>::GeneralElement>, Error>
    where
        Self: Sized,
    {
        let child = this.new_tree(DomGeneralElement::DomText(DomTextNode::new(content)));
        Ok(child)
    }

    fn append<'b>(
        this: &'b mut ForestNodeMut<Self>,
        child: &'b ForestNodeRc<
            <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    ) where
        Self: Sized,
    {
        this.append(&child);
        let this = this.as_ref();
        if let Some(parent) = composing::find_nearest_dom_ancestor(this.clone()) {
            let child = this.last_child_rc().unwrap();
            let child = this.borrow(&child);
            let before = composing::find_next_dom_sibling(child.clone());
            let child_frag = composing::collect_child_frag(child);
            if let Some(child_frag) = child_frag.dom() {
                parent.insert_before(child_frag, before.as_ref()).unwrap();
            }
        }
    }

    fn insert<'b>(
        this: &'b mut ForestNodeMut<Self>,
        target: &'b ForestNodeRc<
            <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    ) where
        Self: Sized,
    {
        this.insert(&target);
        let target = this.as_ref().borrow(&target);
        if let Some(parent) = composing::find_nearest_dom_ancestor(target.clone()) {
            let before = composing::find_next_dom_sibling(target.clone());
            let child_frag = composing::collect_child_frag(target);
            if let Some(child_frag) = child_frag.dom() {
                parent.insert_before(child_frag, before.as_ref()).unwrap();
            }
        }
    }

    fn temp_detach(
        this: ForestNodeMut<Self>,
    ) -> ForestNodeRc<<<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement>
    where
        Self: Sized,
    {
        this.detach()
    }

    fn detach(
        this: ForestNodeMut<Self>,
    ) -> ForestNodeRc<<<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement>
    where
        Self: Sized,
    {
        {
            let this = this.as_ref();
            if let Some(parent) = composing::find_nearest_dom_ancestor(this.clone()) {
                composing::remove_all_children(&parent, this);
            }
        }
        let ret = this.detach();
        ret
    }
}
