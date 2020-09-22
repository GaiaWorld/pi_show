### release
* wasm-pack build --target web
* ..\wasm2js --emscripten  pkg/res_mgr_bg.wasm -o pkg/res_mgr_bg.asm.js

### debug
* wasm-pack build --debug --target web
* ..\wasm2js --emscripten  pkg_debug/res_mgr_bg.wasm -o pkg_debug/res_mgr_bg.asm.js

## wasm-pack build --target -d --out-name gui

## 打包为js
## wasm2js pkg/res_mgr_bg.wasm -o pkg/res_mgr_bg.js

## sed -i 's/res_mg_bg.wasm/res_mgr_bg.js/' pkg/res_mgr.js


## emcc工具转wasm为asm
## ..\wasm2js --emscripten  pkg/gui_bg.wasm -o pkg/gui_bg.js


## node_modules\.bin\webpack --config webpack.config.js