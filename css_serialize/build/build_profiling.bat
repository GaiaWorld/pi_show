cd ../
set RUST_LOG=info
wasm-pack build --profiling  --target web --out-dir pkg_profiling --out-name css_serialize
"C:\\Users\\chuanyan\\AppData\\Local\\.wasm-pack\\wasm-bindgen-2b8061563077bfb8\\wasm-bindgen.exe" "D:\\work\\pi_show_wasm_bindgen\\pi_show_wasm_bindgen\\css_serialize\\target\\wasm32-unknown-unknown\\release\\css_serialize.wasm" "--out-dir" "D:\\work\\pi_show_wasm_bindgen\\pi_show_wasm_bindgen\\css_serialize\\pkg_profiling" "--typescript" "--target" "web" "--out-name" "css_serialize"
node build/build_wasm.js pkg_profiling css_serialize
pause;
