
cd ../
set RUST_LOG=info
wasm-pack build --profiling  --target web --out-dir pkg_profiling --out-name gui

"C:\\Users\\chuanyan\\AppData\\Local\\.wasm-pack\\wasm-bindgen-35e10c997186a1d2\\wasm-bindgen.exe" "D:\\work\\pi_show_wasm_bindgen\\pi_show_wasm_bindgen\\gui_web\\target\\wasm32-unknown-unknown\\release\\gui_web.wasm" "--out-dir" "D:\\work\\pi_show_wasm_bindgen\\pi_show_wasm_bindgen\\gui_web\\pkg_profiling" "--typescript" "--target" "web" "--out-name" "gui"

node build/build_wasm.js pkg_profiling gui

pause;
