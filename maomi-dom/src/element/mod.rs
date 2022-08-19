use maomi::{
    backend::{BackendComponent, SupportBackend},
    error::Error,
    node::{OwnerWeak, SlotChange},
    BackendContext,
};

use crate::{
    base_element::*,
    event::*,
    class_list::DomClassList,
    tree::*,
    DomBackend,
    DomGeneralElement,
};

fn set_style(elem: &web_sys::HtmlElement, s: &str) {
    elem.style().set_css_text(s)
}

macro_rules! define_element {
    ($tag_name:ident, { $($prop:ident: $prop_type:ident: $f:expr,)* }, { $($event:ident: $event_type:ty,)* }) => {
        #[allow(non_camel_case_types)]
        pub struct $tag_name {
            backend_element_token: ForestToken,
            pub class: DomClassList,
            pub style: DomStrAttr,
            $(
                pub $prop: $prop_type,
            )*
            $(
                pub $event: $event_type,
            )*
            elem: web_sys::Element,
        }

        impl $tag_name {
            /// Get the underlying DOM element
            pub fn dom_element(&self) -> &web_sys::Element {
                &self.elem
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
                let elem = crate::DOCUMENT.with(|document| document.create_element(std::stringify!($tag_name)).unwrap());
                let backend_element =
                    crate::DomGeneralElement::create_dom_element(owner, &elem);
                let this = Self {
                    backend_element_token: backend_element.token(),
                    class: DomClassList::new(elem.class_list()),
                    style: DomStrAttr {
                        inner: String::new(),
                        f: set_style,
                    },
                    $(
                        $prop: $prop_type {
                            inner: Default::default(),
                            f: $f,
                        },
                    )*
                    $(
                        $event: Default::default(),
                    )*
                    elem,
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

macro_rules! define_element_with_shared_props {
    ($tag_name:ident, { $($prop:ident: $prop_type:ident: $f:expr,)* }, { $($event:ident: $event_type:ty,)* }) => {
        define_element!($tag_name, {
            title: DomStrAttr: web_sys::HtmlElement::set_title,
            hidden: DomBoolAttr: web_sys::HtmlElement::set_hidden,
            $($prop: $prop_type: $f,)*
        }, {
            touch_start: DomEvent<crate::event::touch::TouchStart>,
            touch_move: DomEvent<crate::event::touch::TouchMove>,
            touch_end: DomEvent<crate::event::touch::TouchEnd>,
            touch_cancel: DomEvent<crate::event::touch::TouchCancel>,
            tap: DomEvent<crate::event::tap::Tap>,
            long_tap: DomEvent<crate::event::tap::LongTap>,
            cancel_tap: DomEvent<crate::event::tap::CancelTap>,
        });
    };
}

define_element_with_shared_props!(div, {}, {});
define_element_with_shared_props!(span, {}, {});
