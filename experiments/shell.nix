{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell rec {
  name = "hyprside-opengl-env";

  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  buildInputs = with pkgs; [
    glfw
    mesa
    libGL
    libGLU
    egl-wayland
    wayland
    libdrm
    xorg.libX11
    xorg.libXrandr
    xorg.libXinerama
    xorg.libXcursor
    xorg.libXi
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
