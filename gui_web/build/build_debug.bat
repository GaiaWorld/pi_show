cd ../
wasm-pack build --release  --target web --out-dir pkg_debug --out-name gui
node build/build_wasm.js pkg_debug gui
pause;