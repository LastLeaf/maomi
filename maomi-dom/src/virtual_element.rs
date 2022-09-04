use maomi::backend::*;

use crate::{DomGeneralElement, DomState};

#[doc(hidden)]
pub struct DomVirtualElement {
    dom_elem: dom_state_ty!((), (), ()),
}

impl DomVirtualElement {
    #[inline]
    pub(crate) fn new(this: &mut tree::ForestNodeMut<DomGeneralElement>) -> Self {
        let dom_elem = match this.is_prerendering() {
            DomState::Normal(_) => DomState::Normal(()),
            #[cfg(feature = "prerendering")]
            DomState::Prerendering(_) => DomState::Prerendering(()),
            #[cfg(feature = "prerendering-apply")]
            DomState::PrerenderingApply(_) => DomState::PrerenderingApply(()),
        };
        Self { dom_elem }
    }

    pub(crate) fn is_prerendering(&self) -> dom_state_ty!((), (), ()) {
        match &self.dom_elem {
            DomState::Normal(_) => DomState::Normal(()),
            #[cfg(feature = "prerendering")]
            DomState::Prerendering(_) => DomState::Prerendering(()),
            #[cfg(feature = "prerendering-apply")]
            DomState::PrerenderingApply(_) => DomState::PrerenderingApply(()),
        }
    }
}

impl BackendVirtualElement for DomVirtualElement {
    type BaseBackend = crate::DomBackend;
}
