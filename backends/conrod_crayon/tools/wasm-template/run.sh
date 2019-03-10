SCRIPT_DIR=$(dirname "$0")
BASE_DIR=$SCRIPT_DIR/../../../../

set -e
cargo build --example $1 --target wasm32-unknown-unknown
cp $BASE_DIR/target/wasm32-unknown-unknown/debug/examples/$1.wasm $SCRIPT_DIR/dist/intermediate/native.wasm
wasm-bindgen $SCRIPT_DIR/dist/intermediate/native.wasm --out-dir $SCRIPT_DIR/dist

cd $SCRIPT_DIR
npm run serve