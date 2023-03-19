use maomi::prelude::*;
use maomi_dom::class_list::DomExternalClasses;
use maomi_dom::{async_task, element::*, event::*, DomBackend};

#[component(Backend = DomBackend)]
pub(crate) struct Div {
    template: template! {
        <div
            class:DomExternalClasses={&self.class}
        ><slot /></div>
    },
    pub(crate) class: DomExternalClasses,
}

impl Component for Div {
    fn new() -> Self {
        Self {
            template: Default::default(),
            class: DomExternalClasses::new(),
        }
    }
}

#[component(Backend = DomBackend)]
pub(crate) struct Span {
    template: template! {
        <span
            class:DomExternalClasses={&self.class}
            aria_hidden={&*self.aria_hidden}
            tap=@click()
        ><slot /></span>
    },
    pub(crate) class: DomExternalClasses,
    pub(crate) aria_hidden: Prop<String>,
    pub(crate) click: Event<TapEvent>,
}

impl Component for Span {
    fn new() -> Self {
        Self {
            template: Default::default(),
            class: DomExternalClasses::new(),
            aria_hidden: Prop::new(String::new()),
            click: Default::default(),
        }
    }
}

impl Span {
    fn click(this: ComponentRc<Self>, detail: &mut TapEvent) {
        let mut detail = detail.clone();
        async_task(async move {
            this.get(move |this| {
                this.click.trigger(&mut detail);
            })
            .await;
        });
    }
}

#[component(Backend = DomBackend)]
pub(crate) struct H1 {
    template: template! {
        <h1
            class:DomExternalClasses={&self.class}
        ><slot /></h1>
    },
    pub(crate) class: DomExternalClasses,
}

impl Component for H1 {
    fn new() -> Self {
        Self {
            template: Default::default(),
            class: DomExternalClasses::new(),
        }
    }
}

#[component(Backend = DomBackend)]
pub(crate) struct Button {
    template: template! {
        <button
            class:DomExternalClasses={&self.class}
            r#type={&*self.r#type}
            id={&*self.id}
            click=@tap()
        ><slot /></button>
    },
    pub(crate) class: DomExternalClasses,
    pub(crate) r#type: Prop<String>,
    pub(crate) id: Prop<String>,
    pub(crate) tap: Event<MouseEvent>,
}

impl Component for Button {
    fn new() -> Self {
        Self {
            template: Default::default(),
            class: DomExternalClasses::new(),
            r#type: Prop::new(String::new()),
            id: Prop::new(String::new()),
            tap: Default::default(),
        }
    }
}

impl Button {
    fn tap(this: ComponentRc<Self>, detail: &mut MouseEvent) {
        let mut detail = detail.clone();
        async_task(async move {
            this.get(move |this| {
                this.tap.trigger(&mut detail);
            })
            .await;
        });
    }
}

#[component(Backend = DomBackend)]
pub(crate) struct A {
    template: template! {
        <a
            class:DomExternalClasses={&self.class}
            click=@tap()
        ><slot /></a>
    },
    pub(crate) class: DomExternalClasses,
    pub(crate) tap: Event<MouseEvent>,
}

impl Component for A {
    fn new() -> Self {
        Self {
            template: Default::default(),
            class: DomExternalClasses::new(),
            tap: Default::default(),
        }
    }
}

impl A {
    fn tap(this: ComponentRc<Self>, detail: &mut MouseEvent) {
        let mut detail = detail.clone();
        async_task(async move {
            this.get(move |this| {
                this.tap.trigger(&mut detail);
            })
            .await;
        });
    }
}

#[component(Backend = DomBackend)]
pub(crate) struct Table {
    template: template! {
        <table
            class:DomExternalClasses={&self.class}
        ><slot /></table>
    },
    pub(crate) class: DomExternalClasses,
}

impl Component for Table {
    fn new() -> Self {
        Self {
            template: Default::default(),
            class: DomExternalClasses::new(),
        }
    }
}

#[component(Backend = DomBackend)]
pub(crate) struct Tbody {
    template: template! {
        <tbody
            class:DomExternalClasses={&self.class}
        ><slot /></tbody>
    },
    pub(crate) class: DomExternalClasses,
}

impl Component for Tbody {
    fn new() -> Self {
        Self {
            template: Default::default(),
            class: DomExternalClasses::new(),
        }
    }
}

#[component(Backend = DomBackend)]
pub(crate) struct Tr {
    template: template! {
        <tr
            class:DomExternalClasses={&self.class}
        ><slot /></tr>
    },
    pub(crate) class: DomExternalClasses,
}

impl Component for Tr {
    fn new() -> Self {
        Self {
            template: Default::default(),
            class: DomExternalClasses::new(),
        }
    }
}

#[component(Backend = DomBackend)]
pub(crate) struct Td {
    template: template! {
        <td
            class:DomExternalClasses={&self.class}
        ><slot /></td>
    },
    pub(crate) class: DomExternalClasses,
}

impl Component for Td {
    fn new() -> Self {
        Self {
            template: Default::default(),
            class: DomExternalClasses::new(),
        }
    }
}
