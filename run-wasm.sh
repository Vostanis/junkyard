cd crates/junk-client
wasm-pack build --target web --out-dir ../junk-server/static/pkg
cp index.html ../junk-server/static/

cd ../junk-server
cargo run

