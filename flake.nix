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
          pkgs.clippy
          pkgs.rust-analyzer
          pkgs.rustPlatform.rust.cargo
          pkgs.rustPlatform.rust.rustc
          pkgs.rustPlatform.rustcSrc
          (if pkgs.stdenv.isDarwin
            then
              pkgs.writeShellScriptBin "rustfmt" ''
                export DYLD_LIBRARY_PATH="${pkgs.rustPlatform.rust.rustc}/lib/rustlib/aarch64-apple-darwin/lib:$DYLD_LIBRARY_PATH"
                ${pkgs.rustfmt}/bin/rustfmt "$@"
              ''
            else pkgs.rustfmt)

          # runtime
          pkgs.libiconv

          # testing
          pkgs.nushell
        ];
      };
    });
}
