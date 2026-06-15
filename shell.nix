{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    pkg-config
    openssl
    sqlite # For caching if we choose SQLite later
  ];

  shellHook = ''
    echo "Welcome to the Wakfu Builder development environment!"
  '';
}
