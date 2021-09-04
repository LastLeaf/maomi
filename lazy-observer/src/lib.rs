use std::cell::RefCell;
use std::collections::HashSet;
use std::pin::Pin;
use std::rc::Rc;

mod observable;
pub use observable::Observable;
mod lazy;
pub use lazy::Lazy;
mod lazy_field;
pub use lazy_field::LazyField;
mod dirty_marker;
use dirty_marker::DirtyMarker;

thread_local! {
    static DIRTY_REF_STACK: RefCell<Vec<HashSet<Pin<Rc<DirtyMarker>>>>> = RefCell::new(vec![]);
}

fn notify_updater(dirty: &Pin<Rc<DirtyMarker>>) {
    DIRTY_REF_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        if stack.is_empty() {
            return;
        }
        let pos = stack.len() - 1;
        let hs = &mut stack[pos];
        hs.insert(dirty.clone());
    })
}

fn exec_updater<T>(dirty: &Pin<Rc<DirtyMarker>>, f: &Rc<dyn Fn() -> T>) -> T {
    DIRTY_REF_STACK.with(|stack| {
        stack.borrow_mut().push(HashSet::new());
        let ret = f();
        let targets = stack.borrow_mut().pop().unwrap();
        DirtyMarker::update_listen_targets(dirty, targets);
        ret
    })
}

fn exec_field_updater<S, T>(
    dirty: &Pin<Rc<DirtyMarker>>,
    arg: &S,
    f: &Rc<dyn for<'r> Fn(&'r S) -> T>,
) -> T {
    DIRTY_REF_STACK.with(|stack| {
        stack.borrow_mut().push(HashSet::new());
        let ret = f(arg);
        let targets = stack.borrow_mut().pop().unwrap();
        DirtyMarker::update_listen_targets(dirty, targets);
        ret
    })
}
