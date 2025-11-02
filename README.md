# Sandician

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![](https://github.com/mksafavi/sandician/actions/workflows/build.yml/badge.svg)](https://github.com/mksafavi/sandician/actions/workflows/build.yml)

A small sand simulation built with the Bevy game engine in Rust.

## Build

### Clone the repository:
``` shell
git clone https://github.com/mksafavi/sandician.git
cd sandician
```

### Enter the development shell using Nix(Optional):
This sets up the Rust toolchain and the required dependencies. You can also manually install them on your system without Nix.

``` shell
nix develop
```

### Build the desktop application:


``` shell
just build "main"
```

This should compile the project and place the binary in `./target/release`:

``` shell
./target/release/main
```

### WebAssembly (WASM) Build:

``` shell
just build-wasm "main"
just build-pages "main"
```

## License
This project is licensed under the Apache License 2.0. See the LICENSE file for details.

## Attributions

"Adventurer" font by Brian J. Smith is licensed under Creative Commons Attribution. Source: https://www.pentacom.jp/pentacom/bitfontmaker2/gallery/?id=195.
