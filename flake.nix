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
    in
    {
      packages.um = craneLib.buildPackage {
        src = craneLib.cleanCargoSource (craneLib.path ./.);

        buildInputs = [
          pkgs.iconv
        ];
      };

      packages.default = self.packages.${system}.um;

      devShells.default = pkgs.mkShell {
        buildInputs = [
          # build
          pkgs.cargo
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
