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
            cargo
            rustfmt
            rust-analyzer
            rustPackages.clippy
            rustup
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
    };
}
