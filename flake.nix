{
  description = "A simple Rust CLI template";

  inputs.nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";

  outputs =
    { self, nixpkgs, ... }:
    let
      supportedSystems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forEachSystem = nixpkgs.lib.genAttrs supportedSystems;
      overlay = final: _previous: {
        project = final.callPackage ./package.nix { };
      };
      mkPkgs = system: import nixpkgs {
        inherit system;
        overlays = [ overlay ];
      };
    in
    {
      overlays.default = overlay;

      packages = forEachSystem (
        system:
        let
          pkgs = mkPkgs system;
        in
        rec {
          inherit (pkgs) project;
          default = project;
        }
      );

      devShells = forEachSystem (
        system:
        let
          pkgs = mkPkgs system;
        in
        {
          default = pkgs.mkShell {
            packages = [
              pkgs.just
              pkgs.rustup
            ];
          };
        }
      );
    };
}
