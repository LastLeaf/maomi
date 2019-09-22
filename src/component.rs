use super::backend::Backend;
use super::node::*;

pub trait Component<B: Backend>: ComponentTemplate {
    fn new() -> Self where Self: Sized;
    fn update(c: &mut ComponentNodeRefMut<B>) where Self: Sized {
        <Self as ComponentTemplate>::template(c, true);
    }
    fn update_now(c: &mut ComponentNodeRefMut<B>) where Self: Sized {
        <Self as ComponentTemplate>::template(c, true);
    }
    fn created(_: &mut ComponentNodeRefMut<B>) where Self: Sized {

    }
    fn attached(_: &mut ComponentNodeRefMut<B>) where Self: Sized {

    }
    fn ready(_: &mut ComponentNodeRefMut<B>) where Self: Sized {

    }
    fn moved(_: &mut ComponentNodeRefMut<B>) where Self: Sized {

    }
    fn detached(_: &mut ComponentNodeRefMut<B>) where Self: Sized {

    }
}
pub trait ComponentTemplate {
    fn template<B: Backend>(component: &mut ComponentNodeRefMut<B>, is_update: bool) -> Option<Vec<NodeRc<B>>> where Self: Sized {
        if is_update {
            return None
        }
        let mut f = || {
            vec![component.new_virtual_node("slot", None, vec![]).into()]
        };
        Some(f())
    }
}
