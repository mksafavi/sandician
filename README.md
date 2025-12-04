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

## Bibliography

  - Bittker, M. (2019). *Making Sandspiel*. maxbittker.com. https://maxbittker.com/making-sandspiel/
  - Devlin, J. & Schuster, M. D. (2020). *Probabilistic Cellular Automata for Granular Media in Video Games*. arXiv:2008.06341. https://arxiv.org/abs/2008.06341
  - Elsts, J. (2009). *Simple Fluid Simulation With Cellular Automata*. w-shadow.com. https://w-shadow.com/blog/2009/09/01/simple-fluid-simulation/
  - Elsts, J. (2009). *How To Make a 'Falling Sand' Style Water Simulation*. w-shadow.com. https://w-shadow.com/blog/2009/09/29/falling-sand-style-water-simulation/
  - Loginova, D. (2019). *Noita: a Game Based on Falling Sand Simulation*. 80.lv. https://80.lv/articles/noita-a-game-based-on-falling-sand-simulation
  - McGhee, J. (2022). *Making a falling sand simulator*. jason.today. https://jason.today/falling-sand
  - McGhee, J. (2022). *Improved Falling Sand*. jason.today. https://jason.today/falling-improved
  - Purho, P. (2019). *Exploring the Tech and Design of Noita* [Video]. GDC Festival of Gaming, YouTube. https://www.youtube.com/watch?v=prXuyMCgbTc
  - The Coding Train. (2024). *Coding Challenge 180: Falling Sand* [Video]. YouTube. https://www.youtube.com/watch?v=L4u7Zy_b868
  - The Powder Toy contributors. (n.d.). *The Powder Toy* [Software]. GitHub. https://github.com/The-Powder-Toy/The-Powder-Toy
  - TileStats. (2022). *Cellular automata tutorial - the basics* [Video]. YouTube. https://www.youtube.com/watch?v=2nE0z_9-fCY
  - TileStats. (2022). *Cellular automata tutorial - applications (epidemic and movements)* [Video]. YouTube. https://www.youtube.com/watch?v=ANAZIEFXKck
  - TileStats. (2022). *Cellular automata tutorial - how to implement a CA in R* [Video]. YouTube. https://www.youtube.com/watch?v=6byzbkDe-RI
  - TodePond. (2021). *TOP 9 WAYS TO MAKE: Water* [Video]. YouTube. https://www.youtube.com/watch?v=2qfjJ-0ZeVM
  - TodePond. (2021). *TOP 9 WAYS TO MAKE: Big Sand* [Video]. YouTube. https://www.youtube.com/watch?v=9mbs0sx3z2A

