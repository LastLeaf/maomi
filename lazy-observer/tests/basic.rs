use std::rc::Rc;
use std::cell::RefCell;
use lazy_observer::*;

#[test]
fn lazy() {
    let call_count = Rc::new(RefCell::new(0));
    let base = Rc::new(RefCell::new(Observable::new(String::new())));
    let computed: Lazy<String> = {
        let call_count = call_count.clone();
        let base = base.clone();
        Lazy::new(move || {
            *call_count.borrow_mut() += 1;
            (**base.borrow()).clone()
        })
    };
    assert_eq!(*call_count.borrow(), 0);
    **base.borrow_mut() = String::from("new");
    assert_eq!(*call_count.borrow(), 0);
    assert_eq!(computed.get(), "new");
    assert_eq!(*call_count.borrow(), 1);
    assert_eq!(*computed.get_ref(), "new");
    assert_eq!(*call_count.borrow(), 1);
    **base.borrow_mut() = String::from("new 2");
    assert_eq!(*call_count.borrow(), 1);
    assert_eq!(*computed.get_ref(), "new 2");
    assert_eq!(*call_count.borrow(), 2);
}

#[test]
fn lazy_with_if() {
    let call_count = Rc::new(RefCell::new(0));
    let base = Rc::new(RefCell::new(Observable::new(String::from("old"))));
    let cond = Rc::new(RefCell::new(Observable::new(false)));
    let computed = {
        let call_count = call_count.clone();
        let base = base.clone();
        let cond = cond.clone();
        Lazy::new(move || {
            *call_count.borrow_mut() += 1;
            if **cond.clone().borrow() {
                (**base.borrow()).clone()
            } else {
                "".into()
            }
        })
    };
    assert_eq!(*call_count.borrow(), 0);
    assert_eq!(computed.get(), "");
    assert_eq!(*call_count.borrow(), 1);
    **base.borrow_mut() = String::from("new");
    assert_eq!(computed.get(), "");
    assert_eq!(*call_count.borrow(), 1);
    **cond.borrow_mut() = true;
    assert_eq!(computed.get(), "new");
    assert_eq!(*call_count.borrow(), 2);
    **base.borrow_mut() = String::from("new 2");
    assert_eq!(computed.get(), "new 2");
    assert_eq!(*call_count.borrow(), 3);
    **cond.borrow_mut() = false;
    assert_eq!(computed.get(), "");
    assert_eq!(*call_count.borrow(), 4);
    **base.borrow_mut() = String::from("new 3");
    assert_eq!(computed.get(), "");
    assert_eq!(*call_count.borrow(), 4);
}

#[test]
fn lazy_field() {
    let call_count = Rc::new(RefCell::new(0));
    struct Obj {
        base: Observable<String>,
        computed: LazyField<Self, usize>,
    }
    let mut obj = {
        let call_count = call_count.clone();
        Obj {
            base: Observable::new(String::new()),
            computed: LazyField::new(move |s: &Obj| {
                *call_count.borrow_mut() += 1;
                s.base.len()
            }),
        }
    };
    assert_eq!(*call_count.borrow(), 0);
    assert_eq!(obj.computed.get(&obj), 0);
    assert_eq!(*call_count.borrow(), 1);
    *obj.base = String::from("new");
    assert_eq!(*call_count.borrow(), 1);
    assert_eq!(obj.computed.get(&obj), 3);
    assert_eq!(*call_count.borrow(), 2);
}
