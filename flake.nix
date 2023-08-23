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

          deps = with pkgs; [
            git
            xorg.libX11
            xorg.libXinerama
          ];

          commonArgs = {
            src = craneLib.cleanCargoSource (craneLib.path ./.);
            version = (craneLib.crateNameFromCargoToml { cargoToml = ./leftwm/Cargo.toml; }).version;
            buildInputs = deps;
          };
          
          rustToolchain = pkgs.rust-bin.stable.latest.default;
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

          cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
            pname = "leftwm-deps";
          });

          leftwm = craneLib.buildPackage (commonArgs // rec {
            inherit cargoArtifacts;
            pname = "leftwm";
            
            postFixup = ''
              for p in $out/bin/left*; do
                patchelf --set-rpath "${pkgs.lib.makeLibraryPath deps}" $p
              done
            '';
            GIT_HASH = self.shortRev or "dirty";
          });
        in rec {

          # `nix build`
          packages = {
            inherit leftwm;
            default = leftwm;
          };

          # `nix develop`
          devShells.default = pkgs.mkShell
            {
              buildInputs = deps ++ [ pkgs.pkg-config pkgs.systemd ];
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
              ];

              shellHook = ''
                source './nixos-vm/vm.sh'                
              '';
            };
        };

        flake = rec {
          overlays.default = final: prev: {
            leftwm = self.packages.${final.system}.leftwm;
          };

          # nixos development vm
          nixosConfigurations.leftwm = nixpkgs.lib.nixosSystem 
          {
            system = "x86_64-linux";
            modules = [
                {nixpkgs.overlays = [
                  overlays.default
                ];}
               ./nixos-vm/configuration.nix
            ]; 
          };
        };
    };
}
