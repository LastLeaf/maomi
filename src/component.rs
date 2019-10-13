use super::backend::Backend;
use super::node::*;

pub trait Component: ComponentTemplate {
    fn new() -> Self where Self: Sized;
    fn update<B: Backend>(c: &mut ComponentNodeRefMut<B>) where Self: Sized {
        <Self as ComponentTemplate>::template(c, true);
    }
    fn update_now<B: Backend>(c: &mut ComponentNodeRefMut<B>) where Self: Sized {
        <Self as ComponentTemplate>::template(c, true);
    }
    fn created<B: Backend>(_: &mut ComponentNodeRefMut<B>) where Self: Sized {

    }
    fn attached<B: Backend>(_: &mut ComponentNodeRefMut<B>) where Self: Sized {

    }
    fn ready<B: Backend>(_: &mut ComponentNodeRefMut<B>) where Self: Sized {

    }
    fn moved<B: Backend>(_: &mut ComponentNodeRefMut<B>) where Self: Sized {

    }
    fn detached<B: Backend>(_: &mut ComponentNodeRefMut<B>) where Self: Sized {

    }
}
pub trait ComponentTemplate {
    fn template<B: Backend>(component: &mut ComponentNodeRefMut<B>, is_update: bool) -> Option<Vec<NodeRc<B>>> where Self: Sized {
        if is_update {
            return None
        }
        let mut f = || {
            vec![component.new_virtual_node("slot", VirtualNodeProperty::Slot(""), vec![]).into()]
        };
        Some(f())
    }
}

pub struct DefaultComponent {
    // empty
}
impl Component for DefaultComponent {
    fn new() -> Self {
        Self {

        }
    }
}
impl ComponentTemplate for DefaultComponent {
    // empty
}
