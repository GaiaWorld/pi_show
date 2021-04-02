### release
* wasm-pack build --release --target web -d --out-name gui
* ..\wasm2js --emscripten  pkg/gui_bg.wasm -o pkg/gui_bg_asm.js
* ..\wasm2js --emscripten  pkg/gui.wasm -o pkg/gui_asm.js

### release
* wasm-pack build --profiling  --target web -d --out-name gui
* ..\wasm2js --emscripten  pkg/gui_bg.wasm -o pkg/gui_bg_asm.js
* ..\wasm2js --emscripten  pkg/gui.wasm -o pkg/gui_asm.js

### debug
* wasm-pack build --debug --target web --out-dir pkg_debug --out-name gui
* ..\wasm2js --emscripten  pkg_debug/gui_bg.wasm -o pkg_debug/gui_bg_asm.js
*  ..\wasm2js --emscripten  pkg_debug/gui.wasm -o pkg_debug/gui_asm.js



## set RUST_LOG=info 
##  
## wasm-pack build --target -d --out-name gui

## 打包为js
## wasm2js pkg/res_mgr_bg.wasm -o pkg/res_mgr_bg.js

## sed -i 's/res_mg_bg.wasm/res_mgr_bg.js/' pkg/res_mgr.js


## emcc工具转wasm为asm
## ..\wasm2js --emscripten  pkg/gui_bg.wasm -o pkg/gui_bg.js


## node_modules\.bin\webpack --config webpack.config.js