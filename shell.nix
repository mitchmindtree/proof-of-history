{ lib
, stdenv
, pkgs ? import <nixpkgs> { }
}:

pkgs.mkShell {
  buildInputs = [
    pkgs.cargo
    pkgs.rust-analyzer
    pkgs.rustfmt
    pkgs.gcc
  ] ++ (lib.optionals stdenv.isDarwin [
    pkgs.libiconv
  ]);
  env = lib.optionalAttrs stdenv.isDarwin {
    LD_LIBRARY_PATH = "${pkgs.libiconv}/lib:$LD_LIBRARY_PATH";
  };
}
