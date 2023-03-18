![maomi](icon_160.png)

# maomi

Strict and Performant Web Application Programming

![crates.io](https://img.shields.io/crates/v/maomi?style=flat-square) ![docs.rs](https://img.shields.io/docsrs/maomi?style=flat-square)

```rust
#[component]
struct HelloWorld {
    template: template! {
        "Hello world!"
    }
}
```

## Key Features

* Write rust code, compile to WebAssembly, and run in browser.
* Great overall performance and no common performance pitfalls.
* Reports mistakes while compilation.
* With rust-analyzer installed, easier to investigate elements, properties, and even style classes.
* Based on templates and data bindings.
* Limited stylesheet syntax, easier to investigate.
* High performance server side rendering.
* I18n in the core design.

Checkout the [website](http://lastleaf.cn/maomi/en_US) for details.

去 [中文版站点](http://lastleaf.cn/maomi/zh_CN) 了解详情。

## Examples

See [dom-template](./maomi-dom-template/) for the basic example. Compile with:

```sh
wasm-pack build maomi-dom-template --target no-modules
```

## Run Tests

General rust tests and wasm-pack tests are both needed.

```sh
cargo test
wasm-pack test --firefox maomi-dom # or --chrome
```
