{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
  };
  outputs =
    { self, nixpkgs }:
    let
      pkgs = import nixpkgs { system = "x86_64-linux"; };
    in
    {
      devShell.x86_64-linux =
        with pkgs;
        mkShell {
          buildInputs = [
            rustc
            wasm-bindgen-cli_0_2_100
            lld
            cargo
            rustfmt
            rust-analyzer
            rustPackages.clippy
            rustup
            pkg-config
            libGL
            libglvnd
            wayland
            alsa-lib
            udev
            libxkbcommon
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            vulkan-tools
            vulkan-headers
            vulkan-loader
            vulkan-validation-layers
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          LD_LIBRARY_PATH = "${ with pkgs; lib.makeLibraryPath [ libxkbcommon vulkan-loader xorg.libX11 xorg.libXcursor xorg.libXi ] }";
        };
    };
}
