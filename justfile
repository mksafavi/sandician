export CARGO_TERM_COLOR := 'always'

test:
   cargo test

test-log:
   cargo test -- --nocapture

build BIN:
    cargo build --release --bin {{BIN}}

build-wasm BIN:
    RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --release --target wasm32-unknown-unknown --bin {{BIN}}
    wasm-bindgen --target web --no-typescript --out-dir ./target/wasm-bind ./target/wasm32-unknown-unknown/release/{{BIN}}.wasm

ci-test:
   RUSTFLAGS='-Dwarnings' cargo test --release

ci-build BIN:
    RUSTFLAGS='-Dwarnings' cargo build --release --bin {{BIN}}

ci-build-wasm BIN:
    RUSTFLAGS='-Dwarnings --cfg getrandom_backend="wasm_js"' cargo build --release --target wasm32-unknown-unknown --bin {{BIN}}
    wasm-bindgen --target web --no-typescript --out-dir ./target/wasm-bind ./target/wasm32-unknown-unknown/release/{{BIN}}.wasm
    echo before wasm-opt: $(du target/wasm-bind/main_bg.wasm)
    wasm-opt -Oz ./target/wasm-bind/{{BIN}}_bg.wasm -o ./target/wasm-bind/{{BIN}}_bg.wasm
    echo after wasm-opt: $(du target/wasm-bind/main_bg.wasm)

ci-build-pages BIN: 
    rm -fr ./target/pages
    cp -r ./pages ./target/pages
    cp -r assets/ ./target/pages/
    cp ./target/wasm-bind/* ./target/pages
    sed --in-place 's/binary.js/{{BIN}}.js/' ./target/pages/index.html
    magick ./target/pages/favicon.jpeg -resize 192x192 ./target/pages/favicon_192.jpeg
    magick ./target/pages/favicon.jpeg -resize 512x512 ./target/pages/favicon_512.jpeg

lint:
    RUSTFLAGS='-Dwarnings' cargo fmt --check
    RUSTFLAGS='-Dwarnings' cargo clippy --release --all-targets --all-features
