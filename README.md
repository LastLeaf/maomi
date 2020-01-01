# maomi: A rust wasm framework for building pages with components

`maomi` is a MVVM-like framework for web development. Write your code in rust and compile to WebAssembly!

**Still in very early development. Do not use it in productional stage.**

## Features

* Targeting WebAssembly, with rust static checking!
* Declarative UI updates (MVVM-like)
* Server-side rendering (if needed)

## Sample Code

### Declare a Component

```rust
template!(xml for HelloWorld {
    <div style="display: inline">
        {&self.a}
        <slot />
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

### Create a DOM Context

```html
<div id="placeholder"> THE COMPONENT WILL BE PLACED HERE </div>
```

```rust
let context = maomi::Context::new(maomi::backend::Dom::new("placeholder"));
```

### Compile it

Make sure your code is also WebAssembly-ready, and compile it with `wasm-pack build` .
