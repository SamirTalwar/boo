{
  description = "Boo";

  inputs = {
    flake-utils.url = github:numtide/flake-utils;
    nixpkgs.url = github:NixOS/nixpkgs/master;
    crane = {
      url = github:ipetkov/crane;
      inputs.flake-utils.follows = "flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self
    , flake-utils
    , nixpkgs
    , crane
    }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs { inherit system; };
      craneLib = crane.lib.${system};

      snapFilter = path: _type: builtins.match ".*\\.snap$" path != null;
    in
    {
      packages.boo = craneLib.buildPackage {
        src = pkgs.lib.cleanSourceWith {
          src = craneLib.path ./.;
          filter = path: type: snapFilter path type || craneLib.filterCargoSources path type;
        };

        buildInputs = [
          pkgs.iconv
        ];
      };

      packages.default = self.packages.${system}.boo;

      devShells.default = pkgs.mkShell {
        buildInputs = [
          # build
          pkgs.cargo
          pkgs.cargo-edit
          pkgs.cargo-insta
          pkgs.cargo-machete
          pkgs.clippy
          pkgs.rust-analyzer
          pkgs.rustPlatform.rustcSrc
          pkgs.rustc
          pkgs.rustfmt

          # runtime
          pkgs.libiconv

          # testing
          pkgs.nushell

          # benchmarking
          pkgs.gnuplot
        ];
      };
    });
}
