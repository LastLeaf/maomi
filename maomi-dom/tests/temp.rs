use maomi::prelude::*;

#[component]
struct MyComponent {
    template: template! {
        /* ... */
    },
    my_event: Event<usize>,
}

impl Component for MyComponent {
    fn new() -> Self {
        Self {
            template: Default::default(),
            my_event: Event::new(),
        }
    }
}

#[component]
struct MyComponentUser {
    template: template! {
        <MyComponent my_event=@my_ev() />
    },
}

impl MyComponentUser {
    fn my_ev(this: ComponentRc<Self>, detail: &mut usize) { /* ... */ }
}
