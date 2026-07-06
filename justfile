list:
	just --list

clean:
	cargo clean

test:
	cargo test --manifest-path tests/address-mock-test/Cargo.toml
	cargo test --manifest-path examples/gpio/Cargo.toml
	cargo test --manifest-path examples/mem_mock/Cargo.toml

build-docker target:
	docker build --build-arg LLVM_VERSION={{target}} -t llvm-container:{{target}} ./ci

docker-run target command: (build-docker target)
	docker run --rm -it -v "$PWD":/work -w /work llvm-container:{{target}}  {{command}}

ci llvm:
	wget https://img.shields.io/badge/build-failing-red?style=for-the-badge -O ./ci/llvm-{{llvm}}.svg
	just docker-run {{llvm}} "cargo clean && cargo test --manifest-path tests/address-mock-test/Cargo.toml"
	wget https://img.shields.io/badge/build-passing-brightgreen?style=for-the-badge -O ./ci/llvm-{{llvm}}.svg

ci-all:
	just ci 16
	just ci 17
	just ci 18
	just ci 19
	just ci 20
	just ci 21
	just ci 22
	just test


publish:
	cd cesty-macro && cargo publish
	cd cesty-build && cargo publish
	cargo publish
