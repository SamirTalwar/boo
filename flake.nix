{
  description = "Boo";

  inputs = {
    flake-utils.url = github:numtide/flake-utils;
    nixpkgs.url = github:NixOS/nixpkgs/master;
    fenix = {
      url = github:nix-community/fenix;
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = github:ipetkov/crane;
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self
    , flake-utils
    , nixpkgs
    , fenix
    , crane
    }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          fenix.overlays.default
        ];
      };
      rustToolchain = fenix.packages.${system}.fromToolchainFile {
        dir = ./.;
        sha256 = "sha256-U2yfueFohJHjif7anmJB5vZbpP7G6bICH4ZsjtufRoU=";
      };
      craneLib = crane.lib.${system}.overrideToolchain rustToolchain;

      snapFilter = path: _type: builtins.match ".*\\.snap$" path != null;

      buildPackageArgs = {
        pname = "boo";

        src = pkgs.lib.cleanSourceWith {
          src = craneLib.path ./.;
          filter = path: type: snapFilter path type || craneLib.filterCargoSources path type;
        };

        strictDeps = true;

        buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.libiconv
        ];
      };
    in
    {
      packages.boo-deps = craneLib.buildDepsOnly buildPackageArgs;
      packages.boo = craneLib.buildPackage (buildPackageArgs // {
        cargoArtifacts = self.packages.${system}.boo-deps;
      });
      packages.default = self.packages.${system}.boo;

      formatter = pkgs.nixpkgs-fmt;

      devShells.default = pkgs.mkShell {
        nativeBuildInputs = [
          # build
          rustToolchain
          pkgs.cargo-edit
          pkgs.cargo-insta
          pkgs.cargo-machete
          pkgs.cargo-nextest

          # testing
          pkgs.nushell

          # benchmarking
          pkgs.gnuplot
        ];

        buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.libiconv
        ];
      };
    });
}
