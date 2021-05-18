cd ../
wasm-pack build --release  --target nodejs --out-dir pkg --out-name css_serialize
node build/build_wasm.js pkg css_serialize
pause;