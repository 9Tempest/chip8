web:
	cd wasm && \
	wasm-pack build --target web && \
	mv pkg/wasm_bg.wasm ../web && \
	mv pkg/wasm.js ../web && \
	cd ../web && \
	python3 -m http.server

desktop:
	cd desktop && cargo run ../tests/UFO

.PHONY: web desktop