use maomi::backend::tree::ForestNodeRc;

use super::{BubbleEvent, DomEventRegister};
use crate::DomGeneralElement;

macro_rules! hot_event {
    ($t:ident, $field:ident, $detail:ty) => {
        pub struct $t {}

        impl DomEventRegister for $t {
            type Detail = $detail;

            fn bind(
                target: &mut crate::base_element::DomElement,
                f: Box<dyn 'static + Fn(&mut Self::Detail)>,
            ) {
                let list = target.hot_event_list_mut();
                list.$field = Some(f);
            }

            fn trigger(target: &mut crate::base_element::DomElement, detail: &mut Self::Detail) {
                if let Some(list) = target.hot_event_list() {
                    if let Some(f) = &list.$field {
                        f(detail);
                    }
                }
            }
        }
    };
}

macro_rules! cold_event {
    ($arm:ident, $detail:ty, $listen:expr) => {
        pub struct $arm {}

        impl DomEventRegister for $arm {
            type Detail = $detail;

            #[inline]
            fn bind(
                target: &mut crate::base_element::DomElement,
                f: Box<dyn 'static + Fn(&mut Self::Detail)>,
            ) {
                for item in target.cold_event_list_mut() {
                    if let ColdEventItem::$arm(x, _) = item {
                        *x = f;
                        return;
                    }
                }
                #[cfg(feature = "prerendering")]
                if let crate::DomState::Prerendering(_) = &target.elem {
                    return;
                }
                let cb = $listen;
                let item = ColdEventItem::$arm(f, cb);
                match &target.elem {
                    crate::DomState::Normal(x) => {
                        item.apply(x)
                    }
                    #[cfg(feature = "prerendering")]
                    crate::DomState::Prerendering(_) => unreachable!(),
                    #[cfg(feature = "prerendering-apply")]
                    crate::DomState::PrerenderingApply(_) => {}
                }
                target.cold_event_list_mut().push(item);
            }

            #[inline]
            fn trigger(target: &mut crate::base_element::DomElement, detail: &mut Self::Detail) {
                if let Some(list) = target.cold_event_list() {
                    let f = list.iter().find_map(|x| {
                        if let ColdEventItem::$arm(x, _) = x {
                            Some(x)
                        } else {
                            None
                        }
                    });
                    if let Some(f) = f {
                        f(detail);
                    }
                }
            }
        }
    };
}

pub(super) fn bubble_event<T>(target: ForestNodeRc<DomGeneralElement>, detail: &mut T::Detail)
where
    T: DomEventRegister,
    T::Detail: BubbleEvent,
{
    let mut cur = target.clone();
    loop {
        let next = {
            let mut n = cur.borrow_mut();
            if let DomGeneralElement::Element(x) = &mut *n {
                T::trigger(x, detail);
                if detail.propagation_stopped() {
                    break;
                }
            }
            n.parent_rc()
        };
        if let Some(next) = next {
            cur = next;
        } else {
            break;
        }
    }
}
