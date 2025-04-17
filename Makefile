
build:
	wasm-pack build --target web

run: build
	python3 -m http.server 6969


