use maomi::{prelude::*, BackendContext};
use maomi_dom::{async_task, event::*, prelude::*, DomBackend};
use wasm_bindgen::prelude::*;

mod comp;
mod data;
use comp::*;

stylesheet! {
  #[css_name("jumbotron")]
  class jumbotron {}
  #[css_name("row")]
  class row {}
  #[css_name("col-md-1")]
  class col_md_1 {}
  #[css_name("col-md-4")]
  class col_md_4 {}
  #[css_name("col-md-6")]
  class col_md_6 {}
  #[css_name("col-sm-6")]
  class col_sm_6 {}
  #[css_name("smallpad")]
  class smallpad {}
  #[css_name("btn")]
  class btn {}
  #[css_name("btn-primary")]
  class btn_primary {}
  #[css_name("btn-block")]
  class btn_block {}
  #[css_name("table")]
  class table {}
  #[css_name("table-hover")]
  class table_hover {}
  #[css_name("table-striped")]
  class table_striped {}
  #[css_name("test-data")]
  class test_data {}
  #[css_name("danger")]
  class danger {}
  #[css_name("glyphicon")]
  class glyphicon {}
  #[css_name("glyphicon-remove")]
  class glyphicon_remove {}
  #[css_name("preloadicon")]
  class preloadicon {}
}

#[component(Backend = DomBackend)]
struct HelloWorld {
    template: template! {
        <Div class:jumbotron>
        <Div class:row>
          <Div class:col_md_6>
            <H1> "maomi (keyed, wrapped)" </H1>
          </Div>
          <Div class:col_md_6>
            <Div class:row>
              <Div class:col_sm_6 class:smallpad>
                <Button
                  r#type="button"
                  class:btn
                  class:btn_primary
                  class:btn_block
                  id="run"
                  tap=@run()
                >
                  "Create 1,000 rows"
                </Button>
              </Div>
              <Div class:col_sm_6 class:smallpad>
                <Button
                  r#type="button"
                  class:btn
                  class:btn_primary
                  class:btn_block
                  id="runlots"
                  tap=@run_lots()
                >
                  "Create 10,000 rows"
                </Button>
              </Div>
              <Div class:col_sm_6 class:smallpad>
                <Button
                  r#type="button"
                  class:btn
                  class:btn_primary
                  class:btn_block
                  id="add"
                  tap=@add()
                >
                  "Append 1,000 rows"
                </Button>
              </Div>
              <Div class:col_sm_6 class:smallpad>
                <Button
                  r#type="button"
                  class:btn
                  class:btn_primary
                  class:btn_block
                  id="update"
                  tap=@update()
                >
                  "Update every 10th row"
                </Button>
              </Div>
              <Div class:col_sm_6 class:smallpad>
                <Button
                  r#type="button"
                  class:btn
                  class:btn_primary
                  class:btn_block
                  id="clear"
                  tap=@clear()
                >
                  "Clear"
                </Button>
              </Div>
              <Div class:col_sm_6 class:smallpad>
                <Button
                  r#type="button"
                  class:btn
                  class:btn_primary
                  class:btn_block
                  id="swaprows"
                  tap=@swap_rows()
                >
                  "Swap Rows"
                </Button>
              </Div>
            </Div>
          </Div>
        </Div>
      </Div>
      <Table class:table class:table_hover class:table_striped class:test_data>
        <Tbody>
          for item in self.rows.iter() use usize {
            <Tr
                class:danger=&{ item.id == self.selected }
            >
                <Td class:col_md_1>{ &format!("{}", item.id) }</Td>
                <Td class:col_md_4>
                    <A tap=@select(&item.id)>{ &item.label }</A>
                </Td>
                <Td class:col_md_1>
                <A>
                    <Span class:glyphicon class:glyphicon_remove aria_hidden="true" click=@remove(&item.id)></Span>
                </A>
                </Td>
                <Td class:col_md_6></Td>
            </Tr>
          }
        </Tbody>
      </Table>
      <Span
        class:preloadicon class:glyphicon class:glyphicon_remove
        aria_hidden="true"
      ></Span>
    },
    rows: Vec<TableRow>,
    selected: usize,
}

#[derive(Debug, Clone)]
struct TableRow {
    id: usize,
    label: String,
}

impl AsListKey for TableRow {
    type ListKey = usize;

    fn as_list_key(&self) -> &Self::ListKey {
        &self.id
    }
}

// implement basic component interfaces
impl Component for HelloWorld {
    fn new() -> Self {
        Self {
            template: Default::default(),
            rows: vec![],
            selected: std::usize::MAX,
        }
    }
}

impl HelloWorld {
    fn add(this: ComponentRc<Self>, _detail: &mut MouseEvent) {
        async_task(async move {
            this.update(|this| {
                this.rows.append(&mut data::build(1000));
            })
            .await
            .unwrap();
        });
    }

    fn remove(this: ComponentRc<Self>, _detail: &mut TapEvent, id: &usize) {
        let id = *id;
        async_task(async move {
            this.update(move |this| {
                let index = this.rows.iter().position(|x| x.id == id).unwrap();
                this.rows.remove(index);
            })
            .await
            .unwrap();
        });
    }

    fn select(this: ComponentRc<Self>, _detail: &mut MouseEvent, id: &usize) {
        let id = *id;
        async_task(async move {
            this.update(move |this| {
                this.selected = id;
            })
            .await
            .unwrap();
        });
    }

    fn run(this: ComponentRc<Self>, _detail: &mut MouseEvent) {
        async_task(async move {
            this.update(|this| {
                this.rows = data::build(1000);
                this.selected = usize::MAX;
            })
            .await
            .unwrap();
        });
    }

    fn update(this: ComponentRc<Self>, _detail: &mut MouseEvent) {
        async_task(async move {
            this.update(|this| {
                let mut i = 0;
                while i < this.rows.len() {
                    this.rows[i].label += " !!!";
                    i += 10;
                }
            })
            .await
            .unwrap();
        });
    }

    fn run_lots(this: ComponentRc<Self>, _detail: &mut MouseEvent) {
        async_task(async move {
            this.update(|this| {
                this.rows = data::build(10000);
                this.selected = usize::MAX;
            })
            .await
            .unwrap();
        });
    }

    fn clear(this: ComponentRc<Self>, _detail: &mut MouseEvent) {
        async_task(async move {
            this.update(|this| {
                this.rows = Vec::with_capacity(0);
                this.selected = usize::MAX;
            })
            .await
            .unwrap();
        });
    }

    fn swap_rows(this: ComponentRc<Self>, _detail: &mut MouseEvent) {
        async_task(async move {
            this.update(|this| {
                let rows = &mut this.rows;
                if rows.len() > 998 {
                    let r998 = rows[998].clone();
                    rows[998] = std::mem::replace(&mut rows[1], r998);
                }
            })
            .await
            .unwrap();
        });
    }
}

#[wasm_bindgen(start)]
pub fn wasm_main() {
    // init logger
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Warn).unwrap();

    // init a backend context
    let dom_backend = DomBackend::new_with_element_id("main").unwrap();
    let backend_context = BackendContext::new(dom_backend);

    // create a mount point
    backend_context
        .enter_sync(move |ctx| {
            let mount_point = ctx.attach(|_: &mut HelloWorld| {}).unwrap();
            // leak the mount point, so that event callbacks still work
            Box::leak(Box::new(mount_point));
        })
        .map_err(|_| "Cannot init mount point")
        .unwrap();

    // leak the backend context, so that event callbacks still work
    Box::leak(Box::new(backend_context));
}
