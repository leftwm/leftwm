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
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ self, flake-parts, rust-overlay, crane, nixpkgs, ... }: 
    flake-parts.lib.mkFlake {inherit inputs;} {
        systems = [
          "x86_64-linux"
          "aarch64-linux"
        ];

        perSystem = { pkgs, system, ... }: 
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };

          commonArgs = {
            src = craneLib.cleanCargoSource (craneLib.path ./.);

            buildInputs = with pkgs; [
              git
              xorg.libX11
              xorg.libXinerama
            ];
          } // (craneLib.crateNameFromCargoToml { cargoToml = ./leftwm/Cargo.toml; });
          
          rustToolchain = pkgs.rust-bin.stable.latest.default;
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

          cargoArtifacts = craneLib.buildDepsOnly (commonArgs);

          leftwm = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
            
            postFixup = ''
              for p in $out/bin/left*; do
                patchelf --set-rpath "${pkgs.lib.makeLibraryPath commonArgs.buildInputs}" $p
              done
            '';
            GIT_HASH = self.shortRev or "dirty";
          });
        in {

          # `nix build`
          packages = {
            inherit leftwm;
            default = leftwm;
          };

          # `nix develop`
          devShells.default = pkgs.mkShell
            {
              buildInputs = commonArgs.buildInputs ++ [ pkgs.pkg-config pkgs.systemd ];
              nativeBuildInputs = with pkgs; [
                gnumake
                (rustToolchain.override { extensions = [
                  "cargo"
                  "clippy"
                  "rust-src"
                  "rust-analyzer"
                  "rustc"
                  "rustfmt"
                ];})
                virt-viewer
              ];

              shellHook = ''
                source './.nixos-vm/vm.sh'                
              '';
            };
        };

        flake = {
          overlays.default = final: prev: {
            leftwm = self.packages.${final.system}.leftwm;
          };

          # nixos development vm
          nixosConfigurations.leftwm = nixpkgs.lib.nixosSystem 
          {
            system = "x86_64-linux";
            modules = [
                {nixpkgs.overlays = [
                  self.overlays.default
                ];}
               ./.nixos-vm/configuration.nix
            ]; 
          };
        };
    };
}
