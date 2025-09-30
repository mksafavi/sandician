export CARGO_TERM_COLOR := 'always'

test:
   cargo test

test-log:
   cargo test -- --nocapture

build BIN:
    cargo build --bin {{BIN}}

build-wasm BIN:
    RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --release --target wasm32-unknown-unknown --bin {{BIN}}
    wasm-bindgen --target web --no-typescript --out-dir ./target/wasm-bind ./target/wasm32-unknown-unknown/release/{{BIN}}.wasm

build-pages BIN : (build-wasm BIN)
    rm -fr ./target/pages
    cp -r ./pages ./target/pages
    cp ./target/wasm-bind/* ./target/pages
    sed --in-place 's/binary.js/{{BIN}}.js/' ./target/pages/index.html

ci-test:
   RUSTFLAGS='-Dwarnings' cargo test

ci-build BIN:
    RUSTFLAGS='-Dwarnings' cargo build --release --bin {{BIN}}

ci-build-wasm BIN:
    RUSTFLAGS='-Dwarnings --cfg getrandom_backend="wasm_js"' cargo build --release --target wasm32-unknown-unknown --bin {{BIN}}
    wasm-bindgen --target web --no-typescript --out-dir ./target/wasm-bind ./target/wasm32-unknown-unknown/release/{{BIN}}.wasm
    wasm-opt -Oz ./target/wasm-bind/{{BIN}}_bg.wasm -o ./target/wasm-bind/{{BIN}}_bg.wasm

ci-build-pages BIN : (ci-build-wasm BIN)
    rm -fr ./target/pages
    cp -r ./pages ./target/pages
    cp ./target/wasm-bind/* ./target/pages
    sed --in-place 's/binary.js/{{BIN}}.js/' ./target/pages/index.html

clippy:
    RUSTFLAGS='-Dwarnings' cargo clippy --release --all-targets --all-features
