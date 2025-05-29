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
                if [[ "$*" == *"--release"* ]]; then
                  profile="release"
                elif [[ "$*" == *"--profile="* ]]; then
                  profile=$(echo "$*" | sed -n 's/.*--profile=\([^[:space:]]*\).*/\1/p')
                else
                  profile="debug"
                fi
                binary_path="target/$profile/vylfs"
                echo "[build] Patching dynamic linker of ELF binary..."
                patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 "$binary_path"
                echo "[build] Done."
              '')
              (mkScript "lint" ''
                echo "[lint] Running cargo fmt..."
                cargo fmt -- --check
                echo "[lint] Running cargo clippy..."
                cargo clippy --all-targets --all-features -- -D warnings
                echo "[lint] Done."
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
                cargo-udeps
                rust-analyzer
                fuse3
                patchelf
                mold
                clang
              ]
              ++ scripts;

            env = {
              RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
              MOLD_PATH = "${pkgs.mold}/bin/mold";
              FUSE3_LIB_PATH = "${pkgs.fuse3.out}/lib";
            };
          };
        }
      );
    };
}
