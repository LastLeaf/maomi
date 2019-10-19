use super::backend::Backend;
use super::node::*;

pub trait Component: ComponentTemplate + 'static {
    fn new() -> Self where Self: Sized;
    fn update<B: Backend>(c: &mut ComponentNodeRefMut<B>) where Self: Sized {
        Self::update_now(c);
    }
    fn update_now<B: Backend>(c: &mut ComponentNodeRefMut<B>) where Self: Sized {
        <Self as ComponentTemplate>::template(c, true);
        c.update_node();
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
            vec![component.new_virtual_node("slot", VirtualNodeProperty::Slot("", vec![]), vec![]).into()]
        };
        Some(f())
    }
}

pub struct DefaultComponent {
    pub todo: super::Prop<String>
    // empty
}
impl Component for DefaultComponent {
    fn new() -> Self {
        Self {
            todo: super::Prop::new(String::new())
            // empty
        }
    }
}
impl ComponentTemplate for DefaultComponent {
    // empty
}
