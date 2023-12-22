{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.cargo
    pkgs.rust-analyzer
    pkgs.rustfmt
    pkgs.gcc
    pkgs.libiconv
  ];
  env = pkgs.lib.optionalAttrs pkgs.stdenv.isDarwin {
    LD_LIBRARY_PATH="${pkgs.libiconv}/lib:$LD_LIBRARY_PATH";
  };
}
