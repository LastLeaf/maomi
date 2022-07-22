use element::DomElement;
use enum_dispatch::enum_dispatch;
use maomi::{
    backend::{tree::*, *},
    error::Error,
};

pub mod element;
pub mod virtual_element;
pub use virtual_element::DomVirtualElement;
pub mod text_node;
pub use text_node::DomTextNode;
mod composing;

thread_local! {
    pub(crate) static WINDOW: web_sys::Window = web_sys::window().expect("Cannot init DOM backend outside web page environment");
    pub(crate) static DOCUMENT: web_sys::Document = {
        WINDOW.with(|window| {
            window.document().expect("Cannot init DOM backend when document is not ready")
        })
    };
}

pub struct DomBackend {
    tree: tree::ForestNodeRc<DomGeneralElement>,
}

impl DomBackend {
    pub fn new_with_element_id(id: &str) -> Result<Self, Error> {
        let dom_elem = DOCUMENT.with(|document| {
            document.get_element_by_id(id)
        }).ok_or_else(|| {
            Error::BackendError {
                msg: format!("Cannot find the element {:?}", id),
                err: None,
            }
        })?;
        Ok(Self::wrap_root_element(dom_elem))
    }

    pub fn new_with_document_body() -> Result<Self, Error> {
        let dom_elem = DOCUMENT.with(|document| {
            document.body()
        }).ok_or_else(|| {
            Error::BackendError {
                msg: "Cannot find the <body> element".into(),
                err: None,
            }
        })?;
        Ok(Self::wrap_root_element(dom_elem.into()))
    }

    fn wrap_root_element(dom_elem: web_sys::Element) -> Self {
        let root_elem = element::DomElement(dom_elem);
        Self {
            tree: tree::ForestNodeRc::new_forest(root_elem.into()),
        }
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

#[enum_dispatch]
pub trait DomGeneralElementTrait {}

#[enum_dispatch(DomGeneralElementTrait)]
pub enum DomGeneralElement {
    VirtualElement(DomVirtualElement),
    DomText(DomTextNode),
    DomElement(element::DomElement),
}

impl std::fmt::Debug for DomGeneralElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VirtualElement(_) => write!(f, "[Virtual]"),
            Self::DomText(x) => write!(f, "{:?}", x.dom().text_content().unwrap_or_default()),
            Self::DomElement(x) => write!(f, "{:?}", x),
        }
    }
}

impl DomGeneralElement {
    fn create_dom_element<'b>(
        this: &'b mut ForestNodeMut<Self>,
        elem: element::DomElement,
    ) -> ForestNodeRc<DomGeneralElement> {
        this.new_tree(DomGeneralElement::DomElement(elem))
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
        child: ForestNodeRc<
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
        sibling: ForestNodeRc<
            <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    ) where
        Self: Sized,
    {
        this.insert(&sibling);
        todo!();
    }

    fn detach(
        this: ForestNodeMut<Self>,
    ) -> ForestNodeRc<<<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement>
    where
        Self: Sized,
    {
        this.detach();
        todo!();
    }
}
