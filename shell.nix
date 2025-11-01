{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  packages = [
    pkgs.qemu_full
    pkgs.mesa.drivers
    pkgs.libGL
    pkgs.virglrenderer
    pkgs.libepoxy
  ];
  shellHook = ''
    export LIBGL_ALWAYS_INDIRECT=0
    export GDK_BACKEND=x11
    export LD_LIBRARY_PATH=${pkgs.mesa.drivers}/lib:${pkgs.libGL}/lib:$LD_LIBRARY_PATH
  '';
}
