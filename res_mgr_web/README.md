## wasm-pack build --target no-modules

## 打包为js
## wasm2js pkg/res_mgr_bg.wasm -o pkg/res_mgr_bg.js

## sed -i 's/res_mg_bg.wasm/res_mgr_bg.js/' pkg/res_mgr.js


## emcc工具转wasm为asm
## .\wasm2js --emscripten  pkg/res_mgr_bg.wasm -o pkg/res_mgr_bg1.js