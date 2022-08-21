use maomi::backend::tree::ForestNodeRc;

use crate::DomGeneralElement;
use super::{DomEventRegister, BubbleEvent};

macro_rules! hot_event {
    ($t:ident, $field:ident, $detail:ty) => {
        pub struct $t {}

        impl DomEventRegister for $t {
            type Detail = $detail;

            fn bind(target: &mut crate::base_element::DomElement, f: Box<dyn 'static + Fn(&mut Self::Detail)>) {
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
    ($arm:ident, $detail:ty, $ev:expr, $listen:expr) => {
        pub struct $arm {}

        impl DomEventRegister for $arm {
            type Detail = $detail;

            fn bind(target: &mut crate::base_element::DomElement, f: Box<dyn 'static + Fn(&mut Self::Detail)>) {
                for item in target.cold_event_list_mut() {
                    if let ColdEventItem::$arm(x, _) = item {
                        *x = f;
                        return;
                    }
                }
                let cb = $listen;
                // Seriously, there should be a removal on the element dropped,
                // otherwise the closure is lost and a js error is displayed in console.
                // However, most events do not trigger after element removal,
                // so here just do no removal.
                if let Err(err) = target.elem.add_event_listener_with_callback(
                    $ev,
                    cb.as_ref().unchecked_ref(),
                ) {
                    crate::log_js_error(&err);
                    log::error!("Failed adding listener for event {:?} (:?). This event will not be triggered.", $ev);
                }
                target.cold_event_list_mut().push(ColdEventItem::$arm(f, cb));
            }

            fn trigger(target: &mut crate::base_element::DomElement, detail: &mut Self::Detail) {
                if let Some(list) = target.cold_event_list() {
                    let f = list.iter()
                        .find_map(|x| if let ColdEventItem::$arm(x, _) = x {
                            Some(x)
                        } else {
                            None
                        });
                    if let Some(f) = f {
                        f(detail);
                    }
                }
            }
        }
    };
}

pub(super) fn bubble_event<T>(
    target: ForestNodeRc<DomGeneralElement>,
    detail: &mut T::Detail,
)
where
    T: DomEventRegister,
    T::Detail: BubbleEvent,
{
    let mut cur = target.clone();
    loop {
        let next = {
            let mut n = cur.borrow_mut();
            if let DomGeneralElement::DomElement(x) = &mut *n {
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
