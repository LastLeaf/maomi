use maomi::backend::*;

use crate::{DomGeneralElement, DomState};

#[doc(hidden)]
pub struct DomTextNode {
    dom_elem: dom_state_ty!(web_sys::Text, ()),
    content: String,
}

impl DomTextNode {
    pub(crate) fn composing_dom(&self) -> &web_sys::Node {
        match &self.dom_elem {
            DomState::Normal(x) => &x,
            #[cfg(feature = "prerendering")]
            DomState::Prerendering(_) => unreachable!(),
            #[cfg(feature = "prerendering-apply")]
            DomState::PrerenderingApply => unreachable!(),
        }
    }

    pub(crate) fn new(this: &mut tree::ForestNodeMut<DomGeneralElement>, content: &str) -> Self {
        let dom_elem = match this.is_prerendering() {
            DomState::Normal(_) => DomState::Normal(
                crate::DOCUMENT.with(|document| document.create_text_node(content)),
            ),
            #[cfg(feature = "prerendering")]
            DomState::Prerendering(_) => DomState::Prerendering(()),
            #[cfg(feature = "prerendering-apply")]
            DomState::PrerenderingApply => DomState::PrerenderingApply,
        };
        Self {
            dom_elem,
            content: content.to_string(),
        }
    }

    pub(crate) fn is_prerendering(&self) -> dom_state_ty!((), ()) {
        match &self.dom_elem {
            DomState::Normal(_) => DomState::Normal(()),
            #[cfg(feature = "prerendering")]
            DomState::Prerendering(_) => DomState::Prerendering(()),
            #[cfg(feature = "prerendering-apply")]
            DomState::PrerenderingApply => DomState::PrerenderingApply,
        }
    }

    pub(crate) fn write_inner_html(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        match &self.dom_elem {
            DomState::Normal(x) => {
                let s = x.text_content().unwrap_or_default();
                write!(w, "{}", s)?;
            }
            #[cfg(feature = "prerendering")]
            DomState::Prerendering(_) => {
                html_escape::encode_text_minimal_to_writer(&self.content, w)?;
            }
            #[cfg(feature = "prerendering-apply")]
            DomState::PrerenderingApply => {}
        }
        Ok(())
    }
}

impl BackendTextNode for DomTextNode {
    type BaseBackend = crate::DomBackend;

    #[inline]
    fn set_text(&mut self, content: &str) {
        if self.content.as_str() != content {
            self.content = content.to_string();
            match &self.dom_elem {
                DomState::Normal(x) => x.set_text_content(Some(content)),
                #[cfg(feature = "prerendering")]
                DomState::Prerendering(_) => {}
                #[cfg(feature = "prerendering-apply")]
                DomState::PrerenderingApply => {}
            }
        }
    }
}
