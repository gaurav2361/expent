{
  description = "A Nix-flake-based Rust, Python (uv), and Node.js development environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1"; # unstable Nixpkgs
    fenix = {
      url = "https://flakehub.com/f/nix-community/fenix/0.1";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # React Native build environment (Android SDK, NDK, Emulator)
    android-nixpkgs = {
      url = "github:tadfisher/android-nixpkgs";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, ... }@inputs:

    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forEachSupportedSystem =
        f:
        inputs.nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            pkgs = import inputs.nixpkgs {
              inherit system;
              overlays = [
                inputs.self.overlays.default
              ];
            };
          }
        );
    in
    {
      overlays.default = final: prev: {
        # Rust Toolchain setup via fenix
        rustToolchain =
          with inputs.fenix.packages.${prev.stdenv.hostPlatform.system};
          combine (
            with stable;
            [
              clippy
              rustc
              cargo
              rustfmt
              rust-src
            ]
          );

        # Node.js passthrough
        nodejs = prev.nodejs;
      };

      devShells = forEachSupportedSystem (
        { pkgs }:
        let
          # Select Python Version
          python = pkgs.python313;

          # Uncomment this block to load React Native configuration
          reactNativeEnv = import ./nix/react-native.nix {
            inherit pkgs;
            system = pkgs.stdenv.hostPlatform.system;
            android-nixpkgs = inputs.android-nixpkgs;
          };
        in
        {
          default = pkgs.mkShell {
            packages =
              with pkgs;
              [
                # Rust
                rustToolchain
                openssl
                pkg-config
                rust-analyzer
                bacon

                # Node.js
                nodejs
                pnpm
                biome

                # Python
                uv
                python
                ruff
                black

                # Utilities
                just
                taplo
                tesseract
              ]
              ++ lib.optionals stdenv.isDarwin [
                libiconv
              ]
              # Uncomment the following line to append React Native packages (Android SDK, Watchman, CocoaPods, etc.)
              ++ reactNativeEnv.packages;

            env = {
              # Required by rust-analyzer
              RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";

              # Tell uv to use the specific Python version provided by Nix
              UV_PYTHON = "${python}/bin/python";

              # Tell pip (if used inside uv) not to check for updates
              PIP_DISABLE_PIP_VERSION_CHECK = "1";
            }
            # Uncomment the following line to merge React Native environment variables (ANDROID_HOME, JAVA_HOME, etc.)
            // reactNativeEnv.env;

            # Automatically creates/activates the uv venv
            shellHook = ''
              echo "Loading Hybrid Rust, Python, and Node.js Dev Environment"

              # Node Setup
              export PATH="$PWD/node_modules/.bin:$PATH"

              # uv Setup
              if [ ! -d ".venv" ]; then
                echo "Creating uv virtual environment..."
                uv venv
              fi

              # Activate venv
              source .venv/bin/activate

              # Display versions
              echo "Versions:"
              echo "  rust:   $(cargo --version)"
              echo "  python: $(python --version)"
              echo "  uv:     $(uv --version)"
              echo "  node:   $(node --version)"
              echo "  pnpm:   $(pnpm --version)"

              # Uncomment the line below to show the React Native startup message
              ${reactNativeEnv.shellHook}
            '';
          };
        }
      );
    };
}
