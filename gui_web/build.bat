REM  start D:emsdk\emsdk_env.bat
REM  ..\..\..\emsdk\emsdk_env.bat

REM python3.7 -m http.server 8000
python2.7 -m SimpleHTTPServer 8000

REM debug
REM 	cargo web build --target=asmjs-unknown-emscripten
REM release
REM 	cargo web build --release --target=asmjs-unknown-emscripten
		node build_asm.js
REM wasm
REM 	cargo web build --target=wasm32-unknown-emscripten
REM wasm release
REM	 	cargo web build --release --target=wasm32-unknown-emscripten
        node build_wasm.js
REM 	cargo web build --release --target=wasm32-unknown-unknown

REM wasm build again
REM 	node build_wasm.js