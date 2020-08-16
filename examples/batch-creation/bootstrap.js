import("./pkg/maomi_example_batch_creation").catch(e => console.error(e)).then(wasm => {
  console.info(Date.now())
  wasm.create()
  console.info(Date.now())
});
