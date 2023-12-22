{
  description = "A very basic flake";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  outputs = inputs: let
    systems = [ "x86_64-linux" "aarch64-darwin" ];
    perSystem = f: inputs.nixpkgs.lib.genAttrs systems f;
    systemPkgs = system: import inputs.nixpkgs {inherit system;};
    perSystemPkgs = f: perSystem (system: f (systemPkgs system));
  in {
    devShells = perSystemPkgs (pkgs: {
      proof-of-history-dev = pkgs.callPackage ./shell.nix {};
      default = inputs.self.devShells.${pkgs.system}.proof-of-history-dev;
    });
  };
}
