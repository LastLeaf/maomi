use maomi::backend::Empty;
use maomi::context::Context;
use maomi::prelude::*;

skin!(pub(self) SKIN_STATIC = r#"
    .a {
        color: red;
        transform: translateZ(0);
    }
"#);
template!(xml for SkinStatic ~SKIN_STATIC {
    <div class="a ~b"></div>
});
struct SkinStatic {}
impl<B: Backend> Component<B> for SkinStatic {
    fn new(_ctx: ComponentContext<B, Self>) -> Self {
        Self {}
    }
}
#[test]
fn component_class_prefix() {
    assert_eq!(
        <SkinStatic as ComponentTemplate<Empty>>::template_skin(),
        r#".SKIN_STATIC-a{color:red;transform:translateZ(0)}"#
    );
    let mut context = Context::new(Empty::new());
    let root_component = context.new_root_component::<SkinStatic>();
    context.set_root_component(root_component);
    let root_component = context.root_component::<SkinStatic>().unwrap();
    let mut root_component = root_component.borrow_mut();
    let mut html: Vec<u8> = vec![];
    root_component.as_mut().to_html(&mut html).unwrap();
    assert_eq!(
        std::str::from_utf8(&html).unwrap(),
        r#"<maomi><div class="SKIN_STATIC-a b"></div></maomi>"#
    );
}

skin!(
    SKIN_SET = r#"
    @set my-size: 0px 1px;
    @set my-style {
        padding: my-size;
        color: red;
    }
    .a {
        border: my-size;
        my-style;
    }
"#
);
#[test]
fn skin_set() {
    assert_eq!(
        SKIN_SET,
        r#".SKIN_SET-a{border:0px 1px;padding:0px 1px;color:red}"#
    );
}

skin!(
    SKIN_IMPORT = r#"
    @import "/../tests/global.skin";
    .a {
        color: red;
        margin: global-size;
    }
"#
);
#[test]
fn skin_import() {
    assert_eq!(
        SKIN_IMPORT,
        r#".SKIN_IMPORT-global{color:blue}.SKIN_IMPORT-a{color:red;margin:1px 0px}"#
    );
}
