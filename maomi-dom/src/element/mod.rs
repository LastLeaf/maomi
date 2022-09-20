use maomi::{
    backend::{BackendComponent, SupportBackend},
    error::Error,
    node::{OwnerWeak, SlotChange},
    BackendContext,
};
use wasm_bindgen::JsCast;

use crate::{
    base_element::*, class_list::DomClassList, event::*, tree::*, DomBackend, DomGeneralElement,
    DomState,
};

fn set_style(elem: &web_sys::HtmlElement, s: &str) {
    elem.style().set_css_text(s)
}

macro_rules! define_element {
    ($tag_name:ident, { $($prop:ident: $prop_type:ident: $f:expr,)* }, { $($event:ident: $event_type:ty,)* }) => {
        /// A DOM element
        #[allow(non_camel_case_types)]
        pub struct $tag_name {
            backend_element_token: ForestToken,
            /// The classes of the element, usually for styling
            pub class: DomClassList,
            /// The CSS inline style for the element
            pub style: DomStrAttr,
            $(
                /// A property
                pub $prop: $prop_type,
            )*
            $(
                /// An event
                pub $event: $event_type,
            )*
            dom_elem_lazy: std::cell::UnsafeCell<dom_state_ty!(web_sys::Element, (), RematchedDomElem)>,
        }

        impl $tag_name {
            /// Get the underlying DOM element
            ///
            /// Panics if called during prerendering stage.
            #[inline]
            pub fn dom_element(&self) -> &web_sys::Element {
                let ptr = self.dom_elem_lazy.get();
                #[cfg(feature = "prerendering-apply")]
                if let DomState::PrerenderingApply(x) = unsafe { &*ptr } {
                    // it is safe
                    // because there cannot be another one that takes refs in DomState::PrerenderingApply state
                    if let Some(e) = x.take() {
                        unsafe { *ptr = DomState::Normal(e); }
                    }
                }
                match unsafe { &*ptr } {
                    DomState::Normal(x) => x,
                    #[cfg(feature = "prerendering")]
                    DomState::Prerendering(_) => {
                        panic!("Cannot get DOM element in prerendering stage")
                    }
                    #[cfg(feature = "prerendering-apply")]
                    DomState::PrerenderingApply(_) => {
                        panic!("Cannot get DOM element in prerendering-apply stage")
                    }
                }
            }
        }

        impl BackendComponent<DomBackend> for $tag_name {
            type SlotData = ();
            type UpdateTarget = Self;
            type UpdateContext = DomElement;

            #[inline]
            fn init<'b>(
                _backend_context: &'b BackendContext<DomBackend>,
                owner: &'b mut ForestNodeMut<DomGeneralElement>,
                _owner_weak: &'b Box<dyn OwnerWeak>,
            ) -> Result<(Self, ForestNodeRc<DomGeneralElement>), Error>
            where
                Self: Sized,
            {
                let tag_name = std::stringify!($tag_name);
                let elem = match owner.is_prerendering() {
                    DomState::Normal(_) => DomState::Normal(crate::DOCUMENT.with(|document| document.create_element(tag_name).unwrap())),
                    #[cfg(feature = "prerendering")]
                    DomState::Prerendering(_) => DomState::Prerendering(PrerenderingElement::new(tag_name)),
                    #[cfg(feature = "prerendering-apply")]
                    DomState::PrerenderingApply(_) => DomState::PrerenderingApply(RematchedDomElem::new()),
                };
                let backend_element =
                    crate::DomGeneralElement::create_dom_element(owner, &elem);
                let this = Self {
                    backend_element_token: backend_element.token(),
                    class: DomClassList::new(match &elem {
                        DomState::Normal(x) => DomState::Normal(x.class_list()),
                        #[cfg(feature = "prerendering")]
                        DomState::Prerendering(_) => DomState::Prerendering(()),
                        #[cfg(feature = "prerendering-apply")]
                        DomState::PrerenderingApply(_) => DomState::PrerenderingApply(()),
                    }),
                    style: DomStrAttr {
                        inner: String::new(),
                        f: set_style,
                        #[cfg(feature = "prerendering")]
                        attr_name: "style",
                    },
                    $(
                        $prop: $prop_type {
                            inner: Default::default(),
                            f: $f,
                            #[cfg(feature = "prerendering")]
                            attr_name: stringify!($prop),
                        },
                    )*
                    $(
                        $event: Default::default(),
                    )*
                    dom_elem_lazy: std::cell::UnsafeCell::new(match elem {
                        DomState::Normal(x) => DomState::Normal(x),
                        #[cfg(feature = "prerendering")]
                        DomState::Prerendering(_) => DomState::Prerendering(()),
                        #[cfg(feature = "prerendering-apply")]
                        DomState::PrerenderingApply(x) => DomState::PrerenderingApply(x.clone()),
                    }),
                };
                Ok((this, backend_element))
            }

            #[inline]
            fn create<'b>(
                &'b mut self,
                _backend_context: &'b BackendContext<DomBackend>,
                owner: &'b mut ForestNodeMut<DomGeneralElement>,
                update_fn: impl FnOnce(&mut Self, &mut Self::UpdateContext),
                mut slot_fn: impl FnMut(
                    &mut ForestNodeMut<DomGeneralElement>,
                    &ForestToken,
                    &Self::SlotData,
                ) -> Result<(), Error>,
            ) -> Result<(), Error> {
                let mut node = owner.borrow_mut_token(&self.backend_element_token).ok_or(Error::TreeNodeReleased)?;
                update_fn(self, &mut DomGeneralElement::as_dom_element_mut(&mut node).unwrap());
                slot_fn(&mut node, &self.backend_element_token, &())?;
                Ok(())
            }

            #[inline]
            fn apply_updates<'b>(
                &'b mut self,
                _backend_context: &'b BackendContext<DomBackend>,
                owner: &'b mut ForestNodeMut<<DomBackend as maomi::backend::Backend>::GeneralElement>,
                update_fn: impl FnOnce(&mut Self, &mut Self::UpdateContext),
                mut slot_fn: impl FnMut(
                    SlotChange<&mut ForestNodeMut<DomGeneralElement>, &ForestToken, &Self::SlotData>,
                ) -> Result<(), Error>,
            ) -> Result<(), Error> {
                let mut node = owner.borrow_mut_token(&self.backend_element_token).ok_or(Error::TreeNodeReleased)?;
                update_fn(self, &mut DomGeneralElement::as_dom_element_mut(&mut node).unwrap());
                slot_fn(SlotChange::Unchanged(&mut node, &self.backend_element_token, &()))?;
                Ok(())
            }
        }

        impl SupportBackend<DomBackend> for $tag_name {
            type Target = Self;
        }
    };
}

fn set_id(elem: &web_sys::HtmlElement, s: &str) {
    web_sys::Element::set_id(&elem, s)
}

// TODO add common elements and attributes

fn set_aria_hidden(elem: &web_sys::HtmlElement, s: &str) {
    elem.set_attribute("aria-hidden", s).ok();
}

macro_rules! define_element_with_shared_props {
    ($tag_name:ident, { $($prop:ident: $prop_type:ident: $f:expr,)* }, { $($event:ident: $event_type:ty,)* }) => {
        define_element!($tag_name, {
            id: DomStrAttr: set_id,
            title: DomStrAttr: web_sys::HtmlElement::set_title,
            hidden: DomBoolAttr: web_sys::HtmlElement::set_hidden,
            aria_hidden: DomStrAttr: set_aria_hidden,
            $($prop: $prop_type: $f,)*
        }, {
            touch_start: DomEvent<crate::event::touch::TouchStart>,
            touch_move: DomEvent<crate::event::touch::TouchMove>,
            touch_end: DomEvent<crate::event::touch::TouchEnd>,
            touch_cancel: DomEvent<crate::event::touch::TouchCancel>,
            mouse_down: DomEvent<crate::event::mouse::MouseDown>,
            mouse_up: DomEvent<crate::event::mouse::MouseUp>,
            mouse_move: DomEvent<crate::event::mouse::MouseMove>,
            mouse_enter: DomEvent<crate::event::mouse::MouseEnter>,
            mouse_leave: DomEvent<crate::event::mouse::MouseLeave>,
            click: DomEvent<crate::event::mouse::Click>,
            tap: DomEvent<crate::event::tap::Tap>,
            long_tap: DomEvent<crate::event::tap::LongTap>,
            cancel_tap: DomEvent<crate::event::tap::CancelTap>,
            scroll: DomEvent<crate::event::scroll::Scroll>,
            animation_start: DomEvent<crate::event::animation::AnimationStart>,
            animation_iteration: DomEvent<crate::event::animation::AnimationIteration>,
            animation_end: DomEvent<crate::event::animation::AnimationEnd>,
            animation_cancel: DomEvent<crate::event::animation::AnimationCancel>,
            transition_run: DomEvent<crate::event::transition::TransitionRun>,
            transition_start: DomEvent<crate::event::transition::TransitionStart>,
            transition_end: DomEvent<crate::event::transition::TransitionEnd>,
            transition_cancel: DomEvent<crate::event::transition::TransitionCancel>,
        });
    };
}

define_element_with_shared_props!(div, {}, {});
define_element_with_shared_props!(span, {}, {});
define_element_with_shared_props!(h1, {}, {});
define_element_with_shared_props!(h2, {}, {});
define_element_with_shared_props!(h3, {}, {});
define_element_with_shared_props!(h4, {}, {});
define_element_with_shared_props!(h5, {}, {});
define_element_with_shared_props!(h6, {}, {});
define_element_with_shared_props!(table, {}, {});
define_element_with_shared_props!(thead, {}, {});
define_element_with_shared_props!(tbody, {}, {});
define_element_with_shared_props!(tfoot, {}, {});
define_element_with_shared_props!(tr, {}, {});
define_element_with_shared_props!(td, {}, {});

fn set_a_href(elem: &web_sys::HtmlElement, s: &str) {
    web_sys::HtmlAnchorElement::set_href(elem.unchecked_ref(), s)
}

define_element_with_shared_props!(a, {
    href: DomStrAttr: set_a_href,
}, {});

fn set_button_type(elem: &web_sys::HtmlElement, s: &str) {
    web_sys::HtmlButtonElement::set_type(elem.unchecked_ref(), s)
}

define_element_with_shared_props!(button, {
    r#type: DomStrAttr: set_button_type,
}, {});
