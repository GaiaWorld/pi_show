cd ../
wasm-pack build --release  --target web --out-dir pkg --out-name gui
node build/build_wasm.js pkg gui
pause;