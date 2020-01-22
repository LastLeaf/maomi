# maomi: A rust wasm framework for building pages with components

`maomi` is a MVVM-like framework for web development. Write your code in rust and compile to WebAssembly!

**Still in early development. Do not use it in productional stage.**

## Features

* Targeting WebAssembly, with powerful rust static checking!
* Declarative UI updates (MVVM-like)
* Optional server-side rendering

## Guide & Sample Code

### Declare a Component

```rust
template!(xml for HelloWorld {
    <div>
        {&self.a}
    </div>
});
struct HelloWorld {
    a: String,
}
impl<B: Backend> Component<B> for HelloWorld {
    fn new(_ctx: ComponentContext<B, Self>) -> Self {
        Self {
            a: "Hello world!".into()
        }
    }
}
```

### Make a Placeholder in HTML Page

```html
<div id="placeholder"> THE COMPONENT WILL BE PLACED HERE </div>
```

### Bootstrap a context

```rust
let mut context = maomi::Context::new(maomi::backend::Dom::new("placeholder"));
let root_component = context.new_root_component::<HelloWorld>();
context.set_root_component(root_component);
```

### Compile it

Make sure your code is also WebAssembly-ready, and compile it with `wasm-pack build` , then `webpack` .

**More full examples at [examples](./examples)**
