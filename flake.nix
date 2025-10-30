{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    # Rust
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs: inputs.flake-parts.lib.mkFlake { inherit inputs; } {
    systems = import inputs.systems;

    perSystem = { config, self', inputs', pkgs, lib, system, ... }: {
      _module.args.pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [ (import inputs.rust-overlay) ];
      };

      # Rust dev environment
      devShells.default = pkgs.mkShell rec {
        rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile
          ./rust-toolchain.toml;
        shellHook = ''
          export PATH=$PWD/target/debug:$PATH
          export RUST_SRC_PATH="${rust-toolchain.availableComponents.rust-src}"
          export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
        '';

        nativeBuildInputs = with pkgs; [
          rust-toolchain
          rust-analyzer
          pnpm
          wasm-bindgen-cli
          jq
        ];

        buildInputs = with pkgs; [
          # for Wasm testing
          chromium
          chromedriver
          wasm-pack
        ];
      };
    };
  };
}
