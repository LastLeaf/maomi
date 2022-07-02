use element::DomElement;
use enum_dispatch::enum_dispatch;
use maomi::backend::*;

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

thread_local! {
    pub(crate) static WINDOW: web_sys::Window = web_sys::window().expect("Cannot init DOM backend outside web page environment");
    pub(crate) static DOCUMENT: web_sys::Document = {
        WINDOW.with(|window| {
            window.document().expect("Cannot init DOM backend when document is not ready")
        })
    };
}

pub struct DomBackend {
    root: DomComponent,
}

impl DomBackend {
    pub fn new() -> Self {
        Self {
            root: DomComponent::new(),
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

    /// Get the root element
    fn root_mut(&mut self) -> &mut Self::Component {
        &mut self.root
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

impl DomGeneralElement {
    fn as_dom_element_mut(&mut self) -> Option<&mut DomElement> {
        todo!()
    }
}

impl BackendGeneralElement for DomGeneralElement {
    type BaseBackend = DomBackend;

    fn append_children(
        &mut self,
        children: impl IntoIterator<
            Item = <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    ) {
        todo!()
    }

    fn splice_children(
        &mut self,
        range: impl std::ops::RangeBounds<usize>,
        children: impl IntoIterator<
            Item = <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
        >,
    ) {
        todo!()
    }

    fn child_mut(
        &mut self,
        index: usize,
    ) -> &mut <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement {
        todo!()
    }

    fn children_mut<'a, T>(&'a mut self) -> T
    where
        T: Iterator<
            Item = &'a mut <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement,
    >{
        todo!()
    }

    fn as_component_mut(
        &mut self,
    ) -> Option<&mut <<Self as BackendGeneralElement>::BaseBackend as Backend>::Component> {
        todo!()
    }

    fn as_slot_mut(
        &mut self,
    ) -> Option<&mut <<Self as BackendGeneralElement>::BaseBackend as Backend>::Slot> {
        todo!()
    }

    fn as_text_node_mut(
        &mut self,
    ) -> Option<&mut <<Self as BackendGeneralElement>::BaseBackend as Backend>::TextNode> {
        todo!()
    }

    fn create_component(
        &mut self,
    ) -> <<Self as BackendGeneralElement>::BaseBackend as Backend>::Component {
        todo!()
    }

    fn create_slot(&mut self) -> <<Self as BackendGeneralElement>::BaseBackend as Backend>::Slot {
        todo!()
    }

    fn create_virtual_element(
        &mut self,
    ) -> <<Self as BackendGeneralElement>::BaseBackend as Backend>::VirtualElement {
        todo!()
    }

    fn create_text_node(
        &mut self,
        content: &str,
    ) -> <<Self as BackendGeneralElement>::BaseBackend as Backend>::TextNode {
        todo!()
    }
}
