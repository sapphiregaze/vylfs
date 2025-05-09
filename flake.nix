{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1.*.tar.gz";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forEachSupportedSystem =
        f:
        nixpkgs.lib.genAttrs supportedSystems (
          system:
          let
            pkgs = import nixpkgs {
              inherit system;
              overlays = [
                rust-overlay.overlays.default
                self.overlays.default
              ];
            };

            mkScript = name: text: pkgs.writeShellScriptBin name text;

            scripts = [
              (mkScript "build" ''
                echo "[build] Running cargo build..."
                cargo build "$@"
              '')
              (mkScript "lint" ''
                echo "[lint] Running cargo fmt and clippy..."
                cargo fmt -- --check
                cargo clippy --all-targets --all-features -- -D warnings
              '')
            ];
          in
          f { inherit pkgs scripts; }
        );
    in
    {
      overlays.default = final: prev: {
        rustToolchain =
          let
            rust = prev.rust-bin;
          in
          if builtins.pathExists ./rust-toolchain.toml then
            rust.fromRustupToolchainFile ./rust-toolchain.toml
          else if builtins.pathExists ./rust-toolchain then
            rust.fromRustupToolchainFile ./rust-toolchain
          else
            rust.stable.latest.default.override {
              extensions = [
                "rust-src"
                "rustfmt"
              ];
            };
      };

      devShells = forEachSupportedSystem (
        { pkgs, scripts }:
        {
          default = pkgs.mkShell {
            packages =
              with pkgs;
              [
                rustToolchain
                openssl
                pkg-config
                cargo-deny
                cargo-edit
                cargo-watch
                rust-analyzer
                fuse3
              ]
              ++ scripts;

            env = {
              RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
            };
          };
        }
      );
    };
}
