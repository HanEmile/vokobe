{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    #flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          # inherit system;
          system = "x86_64-linux";
        };

        naersk' = pkgs.callPackage naersk {};
        
      in rec {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage {
          src = ./.;
        };

        # For `nix develop` (optional, can be skipped):
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ rustc cargo ];
        };

        # hydraJobs."<attr>"."<system>" = derivation;

        hydraJobs = {
          build."x86_64-linux" = naersk'.buildPackage {
            src = ./.;
          };
        };
      };
    # );
}