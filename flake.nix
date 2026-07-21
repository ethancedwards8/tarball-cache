{
  description = "dracula.sh powerline";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    crate2nix.url = "github:nix-community/crate2nix";
    crate2nix.inputs.nixpkgs.follows = "nixpkgs";

    flake-compat = {
      url = "github:NixOS/flake-compat";
      flake = false;
    };
  };

  outputs = inputs@{ self, nixpkgs, ... }:
  let
      forAllSystems = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed;
  in
    {
      devShell = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system; 
            overlays = [ inputs.crate2nix.overlays.default ];
          };
        in
        with pkgs;
        mkShell {
          buildInputs = [
            git
            cargo
            crate2nix
          ];
        }
      );

      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
        in rec {
          default = (pkgs.callPackage ./Cargo.nix { }).workspaceMembers.tarball-cache.build;
          tarball-cache = default;
        }
      );

    };
}

