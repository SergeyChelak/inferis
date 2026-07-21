{ 
  pkgs ? import <nixpkgs> {} 
}: pkgs.mkShell {
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  buildInputs = with pkgs; [ 
    rustc 
    cargo 
    rustfmt 
    bacon
    rustPackages.clippy 
    gcc
    pkgs.SDL2
    pkgs.SDL2_mixer
    pkgs.SDL2_image
    # lldb
	];  
  RUST_BACKTRACE = 1;
}
