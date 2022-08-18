use super::{ColdEventItem, DomEventRegister};

pub struct AnimationEvent {
    dom_event: web_sys::AnimationEvent,
}

impl AnimationEvent {
    pub fn animation_name(&self) -> String {
        self.dom_event.animation_name()
    }

    pub fn elapsed_time(&self) -> f32 {
        self.dom_event.elapsed_time()
    }
}

macro_rules! cold_event {
    ($t:ident, $arm:ident, $detail:ty) => {
        pub struct $t {}

        impl DomEventRegister for $t {
            type Detail = $detail;
        
            fn bind(target: &mut crate::base_element::DomElement, f: Box<dyn 'static + Fn(&mut Self::Detail)>) {
                let list = target.cold_event_list_mut();
                let found = list.iter_mut()
                    .find_map(|x| if let ColdEventItem::$arm(x) = x {
                        Some(x)
                    } else {
                        None
                    });
                match found {
                    Some(x) => {
                        *x = f;
                    }
                    None => {
                        list.push(ColdEventItem::$arm(f));
                        // TODO bind
                    }
                }
            }
        
            fn trigger(target: &mut crate::base_element::DomElement, detail: &mut Self::Detail) {
                let list = target.cold_event_list_mut();
                let f = list.iter_mut()
                    .find_map(|x| if let ColdEventItem::$arm(x) = x {
                        Some(x)
                    } else {
                        None
                    });
                if let Some(f) = f {
                    f(detail);
                }
            }
        }
    };
}

cold_event!(AnimationStart, AnimationStart, AnimationEvent);
cold_event!(AnimationIteration, AnimationIteration, AnimationEvent);
cold_event!(AnimationEnd, AnimationEnd, AnimationEvent);
cold_event!(AnimationCancel, AnimationCancel, AnimationEvent);
