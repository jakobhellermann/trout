out/solver_wasm.js: out/solver_wasm.wasm ../../src/lib.rs ../../src/solver.rs
	wasm-bindgen --out-dir out out/solver_wasm.wasm --target web
	wasm-opt out/solver_wasm_bg.wasm -o out/solver_wasm_bg.wasm -O2


out/solver_wasm.wasm: src/lib.rs Cargo.toml
	cargo build -Zunstable-options --out-dir out --release


.PHONY: clean
clean:
	rm -rf out
