
class env{
    constructor(){
        function rust_wasm_syscall (index, data) {
            console.log("rust_wasm_syscall", index, data);
            // See https://github.com/rust-lang/rust/blob/master/src/libstd/sys/wasm/mod.rs
            switch (index) {
                case 6:
                return 1;
                default:
                return 0;
            }
        }
    }
    
}
export default env;