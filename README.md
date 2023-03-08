# maomi: a rust framework for building pages with components

`maomi` is a framework for building (web) application user interface. Write your code in rust and compile to WebAssembly!

Key features:

* better performance than hand-written vanilla JavaScript;
* strict compile-time check like rust;
* highlighted mistakes in IDE with rust-analyzer;
* limited CSS usage;
* performant server side rendering;
* integrated i18n support.

See [dom-template](./maomi-dom-template/) for examples.
Compile with `wasm-pack build maomi-dom-template --target no-modules` .

## Run Tests

General rust tests and wasm-pack tests are both needed.

```sh
cargo test
wasm-pack test --firefox maomi-dom # or --chrome
```
