{pkgs, ...}: {
  packages = with pkgs; [
    # Code formatting tools
    treefmt
    alejandra
    mdl

    # Rust toolchain
    rustup
  ];
}
