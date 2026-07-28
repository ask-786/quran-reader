#!/usr/bin/env bash
# Regenerates every app icon from the single master source,
# src-tauri/icons/icon.svg — the Rub el Hizb mark in the app's own dark
# theme colours (see the comment block inside that file).
#
# Two outputs come out of one source:
#   src-tauri/icons/*  the platform set Tauri bundles (PNG sizes, the Windows
#                      Store logos, icon.ico and icon.icns), produced by
#                      `tauri icon` so the sizes stay exactly what the bundler
#                      expects rather than something hand-rolled here.
#   static/favicon.png the browser/dev-server tab icon.
#
# `tauri icon` always emits an iOS and Android set as well. This is a desktop
# -only app with no src-tauri/gen mobile projects, so those are deleted again
# below rather than committed as dead weight; same for the 64x64.png it adds,
# which nothing in tauri.conf.json references.
#
# The Linux packaging does NOT need a follow-up edit: packaging/aur/PKGBUILD
# unpacks its icons out of the released .deb, which Tauri builds from
# src-tauri/icons, and quran-reader.desktop refers to them by the unchanged
# name `quran-reader`.
#
# Requires pnpm install to have run (for the Tauri CLI) plus one SVG
# rasteriser: rsvg-convert, ImageMagick or Inkscape.

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
src="$root/src-tauri/icons/icon.svg"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

[ -f "$src" ] || { echo "missing master icon: $src" >&2; exit 1; }

# Rasterise the master once at 1024px; everything else is downscaled from it.
render() { # render <width> <outfile>
  if command -v rsvg-convert >/dev/null 2>&1; then
    rsvg-convert -w "$1" -h "$1" "$src" -o "$2"
  elif command -v magick >/dev/null 2>&1; then
    magick -background none -density 384 "$src" -resize "$1x$1" "$2"
  elif command -v convert >/dev/null 2>&1; then
    convert -background none -density 384 "$src" -resize "$1x$1" "$2"
  elif command -v inkscape >/dev/null 2>&1; then
    inkscape "$src" --export-type=png --export-width="$1" --export-height="$1" \
      --export-filename="$2" >/dev/null
  elif python3 -c "import cairosvg" >/dev/null 2>&1; then
    python3 -c "import cairosvg,sys; cairosvg.svg2png(url=sys.argv[1], \
write_to=sys.argv[2], output_width=int(sys.argv[3]), \
output_height=int(sys.argv[3]))" "$src" "$2" "$1"
  else
    echo "need one of: rsvg-convert, magick, convert, inkscape," >&2
    echo "or python3 with cairosvg (pip install cairosvg)" >&2
    exit 1
  fi
}

echo "==> rasterising master at 1024px"
render 1024 "$tmp/master.png"

echo "==> generating src-tauri/icons"
(cd "$root" && pnpm tauri icon "$tmp/master.png" --output src-tauri/icons)
rm -rf "$root/src-tauri/icons/android" \
       "$root/src-tauri/icons/ios" \
       "$root/src-tauri/icons/64x64.png"

echo "==> generating static/favicon.png"
render 128 "$root/static/favicon.png"

echo "done"
