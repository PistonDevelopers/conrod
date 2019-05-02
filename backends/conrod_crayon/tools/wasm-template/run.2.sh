SCRIPT_DIR=$(dirname "$0")
BASE_DIR=$SCRIPT_DIR/../../../../

set -e
xargo build --example $1 --target wasm32-unknown-unknown
rm -rf $SCRIPT_DIR/dist
mkdir $SCRIPT_DIR/dist
mkdir $SCRIPT_DIR/dist/intermediate
cp $BASE_DIR/target/wasm32-unknown-unknown/debug/examples/$1.wasm $SCRIPT_DIR/dist/intermediate/native.wasm
wasm-bindgen --target web $SCRIPT_DIR/dist/intermediate/native.wasm --out-dir $SCRIPT_DIR/dist --no-typescript
cd $SCRIPT_DIR