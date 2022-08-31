with import <nixpkgs> {};

mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    rustfmt
    rust-analyzer
    clippy
    libiconv
  ];

  RUST_BACKTRACE = 1;
}