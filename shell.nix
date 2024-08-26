{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    meson
    ninja
    pkg-config
    glib
    gtk3
    gtk4
    gtksourceview5
    gtksourceview5.dev
    libadwaita
    libadwaita.dev
    openssl
    openssl.dev
    (python3.withPackages (ps: with ps; [
      pygobject3
    ]))
    blueprint-compiler
    glib
    gobject-introspection
    cargo
    rustc
  ];

  shellHook = ''
    export PKG_CONFIG_PATH="${pkgs.glib.dev}/lib/pkgconfig:${pkgs.gtk3.dev}/lib/pkgconfig:${pkgs.gtk4.dev}/lib/pkgconfig:${pkgs.gtksourceview5.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
    export XDG_DATA_DIRS="${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS"
  '';
}