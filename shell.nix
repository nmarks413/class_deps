{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  packages = with pkgs; [openssl];
  shellHook = ''
    export OPENSSL_DIR="${pkgs.openssl.dev}"
    export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
  '';
}
