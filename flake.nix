{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };

  outputs =
    inputs@{
      self,
      flake-parts,
      rust-overlay,
      crane,
      nixpkgs,
      ...
    }:
    let
      GIT_HASH = self.shortRev or self.dirtyShortRev;
    in
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      perSystem =
        { pkgs, system, ... }:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };

          commonArgs = {
            src = pkgs.lib.cleanSourceWith {
              src = craneLib.path ./.;
              filter =
                path: type:
                (craneLib.filterCargoSources path type)
                || (pkgs.lib.hasSuffix ".frag" path)
                || (pkgs.lib.hasSuffix ".rgba" path);
            };

            buildInputs = with pkgs; [
              mold
              clang
              libgbm
              libinput
              libxkbcommon
              pkg-config
              seatd
              wayland
              xorg.libX11
              xorg.libXrandr
              xorg.libXinerama
            ];

            inherit GIT_HASH;
          }
          // (craneLib.crateNameFromCargoToml { cargoToml = ./leftwm/Cargo.toml; });

          craneLib = (crane.mkLib pkgs).overrideToolchain pkgs.rust-bin.stable.latest.minimal;

          cargoArtifacts = craneLib.buildDepsOnly (
            commonArgs
            // {
              panme = "leftwm-deps";
            }
          );

          leftwm = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;

              postFixup = ''
                for p in $out/bin/left*; do
                  patchelf --set-rpath "${pkgs.lib.makeLibraryPath commonArgs.buildInputs}" $p
                done
              '';

              NIX_CFLAGS_LINK = "-fuse-ld=mold";
            }
          );
        in
        {

          # `nix build`
          packages = {
            inherit leftwm;
            default = leftwm;
          };

          # `nix develop`
          devShells.default = pkgs.mkShell {
            NIX_CFLAGS_LINK = "-fuse-ld=mold";

            buildInputs =
              with pkgs;
              [
                mold
                clang
                pkg-config
                systemd
              ]
              ++ commonArgs.buildInputs;
            nativeBuildInputs = with pkgs; [
              gnumake
              (rust-bin.stable.latest.default.override {
                extensions = [
                  "cargo"
                  "clippy"
                  "rust-src"
                  "rust-analyzer"
                  "rustc"
                  "rustfmt"
                ];
              })
              virt-viewer
            ];

            shellHook = ''
              source ./.nixos-vm/vm.sh
            '';

            inherit GIT_HASH;
          };
        };

      flake = {
        formatter.x86_64-linux = nixpkgs.legacyPackages.x86_64-linux.nixpkgs-fmt;
        overlays.default = final: prev: {
          leftwm = self.packages.${final.system}.leftwm;
        };

        # nixos development vm
        nixosConfigurations.leftwm = nixpkgs.lib.nixosSystem {
          system = "x86_64-linux";
          modules = [
            {
              nixpkgs.overlays = [
                self.overlays.default
              ];
            }
            "${nixpkgs}/nixos/modules/virtualisation/qemu-vm.nix"
            ./.nixos-vm/configuration.nix
          ];
        };
      };
    };
}
