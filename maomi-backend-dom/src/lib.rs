use element::DomElement;
use enum_dispatch::enum_dispatch;
use maomi::{
    backend::{tree::*, *},
    error::Error,
};

pub mod component;
pub mod element;
pub use component::DomComponent;
pub mod shadow_root;
pub use shadow_root::DomShadowRoot;
pub mod slot;
pub use slot::DomSlot;
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
    tree: tree::ForestTree<DomGeneralElement>,
}

impl DomBackend {
    pub fn new() -> Self {
        Self {
            tree: tree::ForestTree::new_forest(DomVirtualElement::new().into()),
        }
    }
}

impl Backend for DomBackend {
    type GeneralElement = DomGeneralElement;
    type VirtualElement = DomVirtualElement;
    type ShadowRoot = DomShadowRoot;
    type Slot = DomSlot;
    type Component = DomComponent;
    type TextNode = DomTextNode;

    fn root_mut(&mut self) -> ForestNodeMut<Self::GeneralElement> {
        self.tree.as_node_mut()
    }
}

#[enum_dispatch]
pub trait DomGeneralElementTrait {}

#[enum_dispatch(DomGeneralElementTrait)]
pub enum DomGeneralElement {
    Component(DomComponent),
    ShadowRoot(DomShadowRoot),
    Slot(DomSlot),
    VirtualElement(DomVirtualElement),
    DomText(DomTextNode),
    DomElement(element::DomElement),
}

impl std::fmt::Debug for DomGeneralElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Component(_) => write!(f, "[Component]"),
            Self::ShadowRoot(_) => write!(f, "[ShadowRoot]"),
            Self::Slot(_) => write!(f, "[Slot]"),
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
    ) -> ForestTree<DomGeneralElement> {
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
            DomGeneralElement::Component(_)
            | DomGeneralElement::ShadowRoot(_)
            | DomGeneralElement::Slot(_)
            | DomGeneralElement::VirtualElement(_) => {}
        }
        let mut ret = String::new();
        let mut cur = this.first_child();
        while let Some(c) = &cur {
            ret += Self::inner_html(&c).as_str();
            cur = c.next_sibling();
        }
        ret
    }
}

impl BackendGeneralElement for DomGeneralElement {
    type BaseBackend = DomBackend;

    fn as_component_mut<'b>(
        this: &'b mut ForestNodeMut<Self>,
    ) -> Option<
        ForestValueMut<'b, <<Self as BackendGeneralElement>::BaseBackend as Backend>::Component>,
    >
    where
        Self: Sized,
    {
        if let DomGeneralElement::Component(_) = &mut **this {
            Some(this.map(|g| {
                if let DomGeneralElement::Component(e) = g {
                    e
                } else {
                    unreachable!()
                }
            }))
        } else {
            None
        }
    }

    fn as_slot_mut<'b>(
        this: &'b mut ForestNodeMut<Self>,
    ) -> Option<ForestValueMut<'b, <<Self as BackendGeneralElement>::BaseBackend as Backend>::Slot>>
    where
        Self: Sized,
    {
        if let DomGeneralElement::Slot(_) = &mut **this {
            Some(this.map(|g| {
                if let DomGeneralElement::Slot(e) = g {
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

    fn create_component<'b>(
        this: &'b mut ForestNodeMut<Self>,
        f: impl FnOnce(
            &mut <<Self as BackendGeneralElement>::BaseBackend as Backend>::Component,
        ) -> Result<(), Error>,
    ) -> Result<ForestTree<<Self::BaseBackend as Backend>::GeneralElement>, Error>
    where
        Self: Sized,
    {
        let mut elem = DomComponent::new(this);
        f(&mut elem)?;
        let child = this.new_tree(DomGeneralElement::Component(elem));
        Ok(child)
    }

    fn create_slot<'b>(
        this: &'b mut ForestNodeMut<Self>,
        f: impl FnOnce(
            &mut <<Self as BackendGeneralElement>::BaseBackend as Backend>::Slot,
        ) -> Result<(), Error>,
    ) -> Result<ForestTree<<Self::BaseBackend as Backend>::GeneralElement>, Error>
    where
        Self: Sized,
    {
        let mut elem = DomSlot::new();
        f(&mut elem)?;
        let child = this.new_tree(DomGeneralElement::Slot(elem));
        Ok(child)
    }

    fn create_virtual_element<'b>(
        this: &'b mut ForestNodeMut<Self>,
        f: impl FnOnce(
            &mut <<Self as BackendGeneralElement>::BaseBackend as Backend>::VirtualElement,
        ) -> Result<(), Error>,
    ) -> Result<ForestTree<<Self::BaseBackend as Backend>::GeneralElement>, Error>
    where
        Self: Sized,
    {
        let mut elem = DomVirtualElement::new();
        f(&mut elem)?;
        let child = this.new_tree(DomGeneralElement::VirtualElement(elem));
        Ok(child)
    }

    fn create_text_node<'b>(
        this: &'b mut ForestNodeMut<Self>,
        content: &str,
    ) -> Result<ForestTree<<Self::BaseBackend as Backend>::GeneralElement>, Error>
    where
        Self: Sized,
    {
        let child = this.new_tree(DomGeneralElement::DomText(DomTextNode::new(content)));
        Ok(child)
    }

    fn append<'b>(
        this: &'b mut ForestNodeMut<Self>,
        child: ForestTree<
            <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    ) where
        Self: Sized,
    {
        this.append(child);
        let this = this.as_ref();
        if let Some(parent) = composing::find_nearest_dom_ancestor(this.clone()) {
            let child = this.last_child().unwrap();
            let before = composing::find_next_dom_sibling(child.clone());
            let child_frag = composing::collect_child_frag(child);
            if let Some(child_frag) = child_frag.dom() {
                parent.insert_before(child_frag, before.as_ref()).unwrap();
            }
        }
    }

    fn insert<'b>(
        this: &'b mut ForestNodeMut<Self>,
        sibling: ForestTree<
            <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    ) where
        Self: Sized,
    {
        this.insert(sibling);
    }

    fn detach<'b>(this: &'b mut ForestNodeMut<Self>)
    where
        Self: Sized,
    {
        this.detach();
    }
}
