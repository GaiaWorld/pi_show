cd ../
cargo build --target=wasm32-unknown-unknown --release
"C:\\Users\\chuanyan\\AppData\\Local\\.wasm-pack\\wasm-bindgen-2b8061563077bfb8\\wasm-bindgen.exe" "D:\\work\\pi_show_wasm_bindgen\\pi_show_wasm_bindgen\\css_serialize\\target\\wasm32-unknown-unknown\\release\\css_serialize.wasm" "--out-dir" "D:\\work\\pi_show_wasm_bindgen\\pi_show_wasm_bindgen\\css_serialize\\pkg_wasm_bindgen" "--typescript" "--target" "web" "--out-name" "css_serialize"
node build/build_wasm.js pkg_wasm_bindgen css_serialize
pause;