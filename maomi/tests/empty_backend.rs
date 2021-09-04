use maomi::backend::Empty;
use maomi::context::Context;
use maomi::prelude::*;

template!(xml for HelloWorld {
    <div style="display: inline">
        {&self.a}
        <slot />
    </div>
});
struct HelloWorld {
    pub a: String,
}
impl<B: Backend> Component<B> for HelloWorld {
    fn new(_ctx: ComponentContext<B, Self>) -> Self {
        Self {
            a: "Hello world!".into(),
        }
    }
}
#[test]
fn create_new_component() {
    let mut context = Context::new(Empty::new());
    let root_component = context.new_root_component::<HelloWorld>();
    context.set_root_component(root_component);
    let root_component = context.root_component::<HelloWorld>().unwrap();
    let mut root_component = root_component.borrow_mut();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div style="display: inline">Hello world!</div></maomi>"#
    );
    root_component.a = "Hello world again!".into();
    root_component.force_apply_updates();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div style="display: inline">Hello world again!</div></maomi>"#
    );
    root_component.a = "Hello world again and again!".into();
    root_component.force_apply_updates();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div style="display: inline">Hello world again and again!</div></maomi>"#
    );
}

template!(xml<B: Backend> for<B> ParentComponent<B> {
    <span>
        <HelloWorld class="hello" a="Hello world">
            " | "
            <HelloWorld a={&self.s} />
        </HelloWorld>
    </span>
});
struct ParentComponent<B: Backend> {
    ctx: ComponentContext<B, Self>,
    pub s: String,
}
impl<B: Backend> Component<B> for ParentComponent<B> {
    fn new(ctx: ComponentContext<B, Self>) -> Self {
        Self {
            ctx,
            s: "from parent!".into(),
        }
    }
    fn attached(&mut self) {
        self.ctx.tick(|_| {})
    }
}
#[test]
fn parent_component() {
    let mut context = Context::new(Empty::new());
    let root_component = context.new_root_component::<ParentComponent<_>>();
    context.set_root_component(root_component);
    let root_component = context.root_component::<ParentComponent<_>>().unwrap();
    let mut root_component = root_component.borrow_mut();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><span><maomi-hello-world class="hello"><div style="display: inline">Hello world | <maomi-hello-world><div style="display: inline">from parent!</div></maomi-hello-world></div></maomi-hello-world></span></maomi>"#
    );
    root_component.s = "from parent again!".into();
    root_component.force_apply_updates();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><span><maomi-hello-world class="hello"><div style="display: inline">Hello world | <maomi-hello-world><div style="display: inline">from parent again!</div></maomi-hello-world></div></maomi-hello-world></span></maomi>"#
    );
    root_component.s = "from parent again and again!".into();
    root_component.force_apply_updates();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><span><maomi-hello-world class="hello"><div style="display: inline">Hello world | <maomi-hello-world><div style="display: inline">from parent again and again!</div></maomi-hello-world></div></maomi-hello-world></span></maomi>"#
    );
}

template!(xml<D: Backend> for<D> TemplateIf<D> {
    <div>
        <if { self.a == 0 }>
            "branch 0"
        </if>
        <else if { self.a == 1 }>
            "branch 1"
        </else>
        <else>
            "other branches"
        </else>
    </div>
});
struct TemplateIf<D: Backend> {
    _ctx: ComponentContext<D, Self>,
    pub a: i32,
}
impl<D: Backend> Component<D> for TemplateIf<D> {
    fn new(_ctx: ComponentContext<D, Self>) -> Self {
        Self { _ctx, a: 0 }
    }
}
#[test]
fn template_if() {
    let mut context = Context::new(Empty::new());
    let root_component = context.new_root_component::<TemplateIf<_>>();
    context.set_root_component(root_component);
    let root_component = context.root_component::<TemplateIf<_>>().unwrap();
    let mut root_component = root_component.borrow_mut();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div>branch 0</div></maomi>"#
    );
    root_component.a = -1;
    root_component.force_apply_updates();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div>other branches</div></maomi>"#
    );
    root_component.a = 1;
    root_component.force_apply_updates();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div>branch 1</div></maomi>"#
    );
}

template!(xml for TemplateFor {
    <for item in { &self.list }>
        <div>
            { item }
        </div>
    </for>
});
struct TemplateFor {
    list: Vec<String>,
}
impl<B: Backend> Component<B> for TemplateFor {
    fn new(_ctx: ComponentContext<B, Self>) -> Self {
        Self {
            list: vec!["Aa".into(), "Bb".into(), "Cc".into()],
        }
    }
}
#[test]
fn template_for() {
    let mut context = Context::new(Empty::new());
    let root_component = context.new_root_component::<TemplateFor>();
    context.set_root_component(root_component);
    let root_component = context.root_component::<TemplateFor>().unwrap();
    let mut root_component = root_component.borrow_mut();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div>Aa</div><div>Bb</div><div>Cc</div></maomi>"#
    );
    // modify
    root_component.list[1] = "Dd".into();
    root_component.force_apply_updates();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div>Aa</div><div>Dd</div><div>Cc</div></maomi>"#
    );
    // append
    root_component.list.push("Ee".into());
    root_component.list.push("Ff".into());
    root_component.force_apply_updates();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div>Aa</div><div>Dd</div><div>Cc</div><div>Ee</div><div>Ff</div></maomi>"#
    );
    // insert
    root_component.list.insert(1, "Gg".into());
    root_component.list.insert(2, "Hh".into());
    root_component.force_apply_updates();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div>Aa</div><div>Gg</div><div>Hh</div><div>Dd</div><div>Cc</div><div>Ee</div><div>Ff</div></maomi>"#
    );
    // remove
    root_component.list.remove(3);
    root_component.list.remove(3);
    root_component.force_apply_updates();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div>Aa</div><div>Gg</div><div>Hh</div><div>Ee</div><div>Ff</div></maomi>"#
    );
    // multi-insert
    root_component.list.insert(0, "Ii".into());
    root_component.list.insert(3, "Jj".into());
    root_component.list.push("Kk".into());
    root_component.force_apply_updates();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div>Ii</div><div>Aa</div><div>Gg</div><div>Jj</div><div>Hh</div><div>Ee</div><div>Ff</div><div>Kk</div></maomi>"#
    );
    // multi-remove
    root_component.list.remove(0);
    root_component.list.remove(1);
    root_component.list.remove(5);
    root_component.force_apply_updates();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div>Aa</div><div>Jj</div><div>Hh</div><div>Ee</div><div>Ff</div></maomi>"#
    );
}

template!(xml for TemplateForKey {
    <for item in { &self.list } use k: i32>
        <div>
            { &item.v }
        </div>
    </for>
});
struct TemplateForKeyItem {
    k: i32,
    v: String,
}
struct TemplateForKey {
    list: Vec<TemplateForKeyItem>,
}
impl<B: Backend> Component<B> for TemplateForKey {
    fn new(_ctx: ComponentContext<B, Self>) -> Self {
        Self {
            list: vec![
                TemplateForKeyItem {
                    k: 1,
                    v: "1".into(),
                },
                TemplateForKeyItem {
                    k: 2,
                    v: "2".into(),
                },
            ],
        }
    }
}
#[test]
fn template_for_key() {
    let mut context = Context::new(Empty::new());
    let root_component = context.new_root_component::<TemplateForKey>();
    context.set_root_component(root_component);
    let root_component = context.root_component::<TemplateForKey>().unwrap();
    let mut root_component = root_component.borrow_mut();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div>1</div><div>2</div></maomi>"#
    );
    // modify
    root_component.list[1] = TemplateForKeyItem {
        k: 2,
        v: "22".into(),
    };
    root_component.force_apply_updates();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div>1</div><div>22</div></maomi>"#
    );
    // append
    root_component.list.push(TemplateForKeyItem {
        k: 3,
        v: "3".into(),
    });
    root_component.list.push(TemplateForKeyItem {
        k: 4,
        v: "4".into(),
    });
    root_component.force_apply_updates();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div>1</div><div>22</div><div>3</div><div>4</div></maomi>"#
    );
    // insert
    root_component.list.insert(
        1,
        TemplateForKeyItem {
            k: 5,
            v: "5".into(),
        },
    );
    root_component.list.insert(
        2,
        TemplateForKeyItem {
            k: 6,
            v: "6".into(),
        },
    );
    root_component.force_apply_updates();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div>1</div><div>5</div><div>6</div><div>22</div><div>3</div><div>4</div></maomi>"#
    );
    // remove
    root_component.list.remove(3);
    root_component.list.remove(3);
    root_component.force_apply_updates();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div>1</div><div>5</div><div>6</div><div>4</div></maomi>"#
    );
    // multi-insert
    root_component.list.insert(
        0,
        TemplateForKeyItem {
            k: 7,
            v: "7".into(),
        },
    );
    root_component.list.insert(
        3,
        TemplateForKeyItem {
            k: 8,
            v: "8".into(),
        },
    );
    root_component.list.push(TemplateForKeyItem {
        k: 9,
        v: "9".into(),
    });
    root_component.force_apply_updates();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div>7</div><div>1</div><div>5</div><div>8</div><div>6</div><div>4</div><div>9</div></maomi>"#
    );
    // multi-remove
    root_component.list.remove(0);
    root_component.list.remove(1);
    root_component.list.remove(4);
    root_component.force_apply_updates();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div>1</div><div>8</div><div>6</div><div>4</div></maomi>"#
    );
}

template!(xml<B: Backend> for<B> EventParentComponent<B> {
    { &self.event_detail }
    <EventChildComponent<B>
        mark = "c"
        @custom_event = {
            |this, data: &String| {
                this.event_detail = data.clone();
                this.ctx.update();
            }
        }
    >
    </EventChildComponent>
});
struct EventParentComponent<B: Backend> {
    ctx: ComponentContext<B, Self>,
    event_detail: String,
}
impl<B: Backend> Component<B> for EventParentComponent<B> {
    fn new(ctx: ComponentContext<B, Self>) -> Self {
        Self {
            ctx,
            event_detail: "".into(),
        }
    }
}
template!(xml<B: Backend> for<B> EventChildComponent<B> { });
struct EventChildComponent<B: Backend> {
    custom_event: Ev<B, String>,
}
impl<B: Backend> Component<B> for EventChildComponent<B> {
    fn new(_ctx: ComponentContext<B, Self>) -> Self {
        Self {
            custom_event: Ev::new(),
        }
    }
}
#[test]
fn custom_event() {
    let mut context = Context::new(Empty::new());
    let root_component = context.new_root_component::<EventParentComponent<Empty>>();
    context.set_root_component(root_component);
    let root_component = context
        .root_component::<EventParentComponent<Empty>>()
        .unwrap();
    {
        let mut root_component = root_component.borrow_mut();
        let mut html: Vec<u8> = vec![];
        root_component.to_html(&mut html).unwrap();
        assert_eq!(
            std::str::from_utf8(&html).unwrap(),
            r#"<maomi><!----><maomi-event-child-component></maomi-event-child-component></maomi>"#
        );
        {
            let child = root_component.marked_component_node_mut("c").unwrap();
            child
                .as_component::<EventChildComponent<_>>()
                .custom_event
                .new_event()
                .trigger(child, &"fromEv".to_string());
        }
    }
    let root_component = root_component.borrow_mut();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi>fromEv<maomi-event-child-component></maomi-event-child-component></maomi>"#
    );
}

template!(xml<B: Backend> for<B> LifetimeParentComponent<B> {
    { &self.order.join(",") }
    <for _item in { &self.list }>
        <LifetimeChildComponent<B> />
    </for>
});
struct LifetimeParentComponent<B: Backend> {
    list: Vec<usize>,
    order: Vec<String>,
    ctx: ComponentContext<B, Self>,
}
impl<B: Backend> Component<B> for LifetimeParentComponent<B> {
    fn new(ctx: ComponentContext<B, Self>) -> Self {
        Self {
            list: vec![0, 1],
            order: vec![],
            ctx,
        }
    }
    fn attached(&mut self) {
        self.order.push("attached".into());
        self.ctx.update();
    }
    fn detached(&mut self) {
        self.order.push("detached".into());
        self.ctx.update();
    }
}
template!(xml<B: Backend> for<B> LifetimeChildComponent<B> {
    { &self.order.join(",") }
});
struct LifetimeChildComponent<B: Backend> {
    order: Vec<String>,
    ctx: ComponentContext<B, Self>,
}
impl<B: Backend> Component<B> for LifetimeChildComponent<B> {
    fn new(ctx: ComponentContext<B, Self>) -> Self {
        Self { order: vec![], ctx }
    }
    fn attached(&mut self) {
        self.order.push("attached".into());
        self.ctx.update();
    }
    fn moved(&mut self) {
        self.order.push("moved".into());
        self.ctx.update();
    }
    fn detached(&mut self) {
        self.order.push("detached".into());
        self.ctx.update();
    }
}
#[test]
fn component_lifetimes() {
    let mut context = Context::new(Empty::new());
    let root_component = context.new_root_component::<LifetimeParentComponent<Empty>>();
    context.set_root_component(root_component);
    let root_component = context
        .root_component::<LifetimeParentComponent<Empty>>()
        .unwrap();
    {
        let root_component = root_component.borrow_mut();
        let mut html: Vec<u8> = vec![];
        root_component.to_html(&mut html).unwrap();
        assert_eq!(
            std::str::from_utf8(&html).unwrap(),
            r#"<maomi>attached<maomi-lifetime-child-component>attached</maomi-lifetime-child-component><maomi-lifetime-child-component>attached</maomi-lifetime-child-component></maomi>"#
        );
    }
    let root_component = root_component.borrow_mut();
    let mut html: Vec<u8> = vec![];
    root_component.to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi>attached<maomi-lifetime-child-component>attached</maomi-lifetime-child-component><maomi-lifetime-child-component>attached</maomi-lifetime-child-component></maomi>"#
    );
}
