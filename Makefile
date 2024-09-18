.ONESHELL:
    
build-release:
	cargo build --release
    
build-wasm:
	cd ./wasm-binding && wasm-pack build --target nodejs