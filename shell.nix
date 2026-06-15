{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain
    rustup
    
    # Node.js (npm included)
    nodejs
    
    # Tauri v2 Linux system dependencies
    webkitgtk_4_1
    gtk3
    libayatana-appindicator
    openssl
    pkg-config
    dbus
    libsoup_3
  ];

  shellHook = ''
    echo "=== Wakfu-Builder Tauri Dev Shell ==="
    echo "Rust: $(rustc --version 2>/dev/null || echo 'setup needed')"
    echo "Node: $(node --version)"
    echo "npm:  $(npm --version)"
    echo ""
    echo "If rust is not installed: rustup default stable"
    echo "Tauri CLI:  cargo install tauri-cli --version '^2'"
  '';
}
