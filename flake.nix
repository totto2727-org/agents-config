{
  description = "Shared OpenAI-compatible agent configuration for Rust applications";

  inputs.nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";

  outputs =
    { nixpkgs, ... }:
    let
      supportedSystems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forEachSystem = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      # This project is a Cargo library, not an installable CLI application.
      devShells = forEachSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            packages = [
              pkgs.actionlint
              pkgs.git
              pkgs.just
              pkgs.rustup
              pkgs.shellcheck
            ];
          };
        }
      );
    };
}
