
import wasi_unstable from "./dist/polyfill.js";
const ws = require("ws");
import init from "./dist/native"
// FIXME dirty hack for wasm_syscall
for (var p in Module){
  console.log("p",p);
}
const env = {
  rust_wasm_syscall: (index, data) => {
    console.log("rust_wasm_syscall", index, data);
    // See https://github.com/rust-lang/rust/blob/master/src/libstd/sys/wasm/mod.rs
    switch (index) {
      case 6:
        return 1;
      default:
        return 0;
    }
  }
};

const instantiateStreaming = WebAssembly.instantiateStreaming;
WebAssembly.instantiateStreaming = (source, importObject) =>
  instantiateStreaming(source, {
    ...importObject,
    env,ws
  });
const instantiate = WebAssembly.instantiate;
WebAssembly.instantiate = (bufferSource, importObject) =>
  instantiate(bufferSource, {
    ...importObject,
    env,ws
  });
 // init("./dist/native_bg.wasm")



// vim: set ts=2 sw=2 et:
