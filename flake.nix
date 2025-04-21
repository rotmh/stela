{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        pkgs = import nixpkgs {
          inherit system;
        };

        buildInputs = with pkgs; [
          glib
          gtk4
          gtk4-layer-shell
        ];
      in {
        devShells.default = pkgs.mkShell {
          inherit buildInputs;

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };
      }
    );
}
