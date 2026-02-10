RFLAGS="-C link-arg=-s"

build-staker:
	rustup override	set 1.84.0
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p injective-staker

test-staker: build-staker build-optimized
	RUSTFLAGS=$(RFLAGS) RUST_TEST_THREADS=1 cargo test -p injective-staker --features test

build: build-staker

build-debug:
	cargo wasm-debug

build-wasm:
	cd ./contracts/injective-staker && cargo wasm && cd ..

build-optimized:
	mkdir -p ./contracts/injective-staker/tests/test_artifacts
	docker run --platform linux/amd64 --rm -v ./:/code \
  --mount type=volume,source=src_cache,target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.16.0
	cp artifacts/injective_staker.wasm ./contracts/injective-staker/tests/test_artifacts/

test: test-staker

schema:
	cd contracts/injective-staker && cargo schema

validate: build-optimized
	cosmwasm-check artifacts/injective_staker.wasm

check-format:
	cargo fmt --check
	cargo clippy --all-features --workspace --tests -- --warn clippy::all --warn clippy::nursery

check-coverage: test
	cargo install cargo-tarpaulin --version 0.31.4
	DYLD_LIBRARY_PATH="`pwd`/target/debug/deps" cargo tarpaulin --all-features --skip-clean --out Html --output-dir coverage-report

clean:
	cargo clean
	rm -rf target/
	rm -rf artifacts/
	rm -rf schema/
	rm -rf ./contracts/injective-staker/tests/test_artifacts/
