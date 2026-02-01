{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    devshell.url = "github:numtide/devshell";
  };

  outputs = { nixpkgs, rust-overlay, devshell, flake-utils, ... }: 
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          (import rust-overlay)
          devshell.overlays.default
        ];
      };

      toolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
        extensions = [ "rust-src" "rust-analyzer" "rustc-codegen-cranelift-preview" ];
      });
    in {
      devShell = pkgs.devshell.mkShell {
        packages = with pkgs; [
          toolchain

          mold clang

          bacon tailwindcss_4
        ];
        motd = "\n  Welcome to the {2}digest{reset} shell.\n";
      };
    });
}
