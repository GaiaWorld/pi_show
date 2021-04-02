cd ../
cargo build --target=wasm32-unknown-unknown --release
"C:\\Users\\chuanyan\\AppData\\Local\\.wasm-pack\\wasm-bindgen-2b8061563077bfb8\\wasm-bindgen.exe" "D:\\work\\pi_show_wasm_bindgen\\pi_show_wasm_bindgen\\gui_web\\target\\wasm32-unknown-unknown\\release\\gui_web.wasm" "--out-dir" "D:\\work\\pi_show_wasm_bindgen\\pi_show_wasm_bindgen\\gui_web\\pkg_wasm_bindgen" "--typescript" "--target" "web" "--out-name" "gui"
node build/build_wasm.js pkg_wasm_bindgen gui
pause;