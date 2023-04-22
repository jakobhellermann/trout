build: solver_wasm
	npm run build

.PHONY: solver_wasm
solver_wasm:
	cd solver_wasm && make --always-make

clean:
	cd solver_wasm && make clean
	rm -rf .parcel-cache dist
