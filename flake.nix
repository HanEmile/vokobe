{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    let
      pkgs = (import nixpkgs) {
        system = "x86_64-linux";
      };

      naersk' = pkgs.callPackage naersk {};
      
    in rec {
      packages."x86_64-linux".vokobe = naersk'.buildPackage {
        src = ./.;
      };
    
      # For `nix build` & `nix run`:
      defaultPackage = packages."x86_64-linux".vokobe;

      # For `nix develop` (optional, can be skipped):
      devShell = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [ rustc cargo ];
      };

      # hydraJobs."<attr>"."<system>" = derivation;
      hydraJobs = {
        build."x86_64-linux" = packages."x86_64-linux".vokobe;
      };
    };
}