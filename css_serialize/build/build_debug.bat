cd ../
wasm-pack build --debug  --target web --out-dir pkg_debug --out-name css_serialize
node build/build_wasm.js pkg_debug css_serialize
pause;