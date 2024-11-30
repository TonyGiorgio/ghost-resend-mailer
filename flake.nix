{
  description = "Rust project";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        commonInputs = [
          rust
          pkgs.rust-analyzer
          pkgs.pkg-config
          pkgs.openssl
          pkgs.zlib
          pkgs.gcc
          pkgs.clang
          pkgs.jq
          pkgs.just
        ];
        linuxOnlyInputs = [
          pkgs.podman
          pkgs.conmon
          pkgs.slirp4netns
          pkgs.fuse-overlayfs
        ];
        darwinOnlyInputs = [
          pkgs.libiconv
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
        ];
        inputs = commonInputs
          ++ pkgs.lib.optionals pkgs.stdenv.isLinux linuxOnlyInputs
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin darwinOnlyInputs;

      in
      {
        defaultPackage = pkgs.rustPlatform.buildRustPackage {
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = inputs;
        };

        devShell = pkgs.mkShell {
          packages = inputs;
          shellHook = ''
            #export OPENSSL_NO_VENDOR=1

            export LIBCLANG_PATH=${pkgs.libclang.lib}/lib/
            export LD_LIBRARY_PATH=${pkgs.openssl}/lib:$LD_LIBRARY_PATH
            export CC_wasm32_unknown_unknown=${pkgs.llvmPackages_14.clang-unwrapped}/bin/clang-14
            export CFLAGS_wasm32_unknown_unknown="-I ${pkgs.llvmPackages_14.libclang.lib}/lib/clang/14.0.6/include/"
            export PKG_CONFIG_PATH=${pkgs.openssl.dev}/lib/pkgconfig

            #export OPENSSL_DIR=${pkgs.openssl.dev}
            #export OPENSSL_LIB_DIR=${pkgs.openssl.out}/lib
            #export OPENSSL_INCLUDE_DIR=${pkgs.openssl.dev}/include

            ${pkgs.lib.optionalString pkgs.stdenv.isLinux ''
              alias docker='podman'
              echo "Using 'podman' as an alias for 'docker'"
              echo "You can now use 'docker' commands, which will be executed by podman"

              # Podman configuration
              export CONTAINERS_CONF=$HOME/.config/containers/containers.conf
              export CONTAINERS_POLICY=$HOME/.config/containers/policy.json
              mkdir -p $HOME/.config/containers
              echo '{"default":[{"type":"insecureAcceptAnything"}]}' > $CONTAINERS_POLICY

              # Create a basic containers.conf if it doesn't exist
              if [ ! -f $CONTAINERS_CONF ]; then
                echo "[engine]
              cgroup_manager = \"cgroupfs\"
              events_logger = \"file\"
              runtime = \"crun\"" > $CONTAINERS_CONF
              fi

              # Ensure correct permissions
              chmod 600 $CONTAINERS_POLICY $CONTAINERS_CONF
            ''}
          '';
        };
      }
    );
}
