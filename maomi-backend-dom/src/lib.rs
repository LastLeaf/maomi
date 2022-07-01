use enum_dispatch::enum_dispatch;
use maomi::backend::*;

pub mod element;
pub mod component;
use component::DomComponent;
pub mod shadow_root;
use shadow_root::DomShadowRoot;
pub mod slot;
use slot::DomSlot;
pub mod virtual_element;
use virtual_element::DomVirtualElement;

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
    Div(element::div),
}

impl BackendGeneralElement for DomGeneralElement {
    type BaseBackend = DomBackend;

    fn append_children(
        &mut self,
        children: impl IntoIterator<Item = <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement>,
    ) {
        todo!()
    }

    fn splice_children(
        &mut self,
        range: impl std::ops::RangeBounds<usize>,
        children: impl IntoIterator<Item = <<Self as BackendGeneralElement>::BaseBackend as Backend>::GeneralElement>,
    ) {
        todo!()
    }

    fn child(
        &self,
        index: usize,
    ) {
        todo!()
    }

    fn children(
        &self,
    ) {
        todo!()
    }
}
