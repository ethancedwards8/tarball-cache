{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    cargo-nix-plugin.url = "github:anthropics/cargo-nix-plugin";
    cargo-nix-plugin.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, cargo-nix-plugin }:
    let
      forAllSystems = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed;
    in {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          cargoNix = cargo-nix-plugin.lib { inherit pkgs; src = ./.; };
        in {
          default = cargoNix.rootCrate.build;
        });

      devShells = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          # The plugin must be loaded by the exact Nix it was built against.
          plugin = cargo-nix-plugin.packages.${system}.cargo-nix-plugin-nix_2_34;
          nixWithPlugin = pkgs.runCommand "nix-with-cargo-nix-plugin"
            { nativeBuildInputs = [ pkgs.makeWrapper ]; }
            ''
              for prog in nix nix-build nix-instantiate nix-shell nix-store nix-env; do
                makeWrapper ${pkgs.nixVersions.nix_2_34}/bin/nix "$out/bin/$prog" --argv0 "$prog" \
                  --add-flags "--option plugin-files ${plugin}/lib/nix/plugins"
              done
            '';
        in {
          default = pkgs.mkShell { packages = with pkgs; [ nixWithPlugin cargo ]; };
        });
    };
}
