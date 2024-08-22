{
  description = "Pathy ~ A pain-free path generator for PROS and EZTemplate.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in {
        devShells.default = pkgs.mkShell rec {
          # Additional dev-shell environment variables can be set directly
          # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

          nativeBuildInputs = with pkgs; [
            trunk
            rust-analyzer
          ];
          buildInputs = with pkgs; [
            libxkbcommon
            libGL
            xorg.libX11
            wayland
            xorg.libXrandr
            xorg.libXcursor
            xorg.libXi
          ];
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
        };
      });
}
