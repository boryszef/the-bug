#!/bin/sh
# Render assets/icons/*.svg to matching *.png for the gui to load at runtime.
# Run this after editing any icon .svg, and commit the .png alongside it.
#
# Needs one of, in preference order:
#   - rsvg-convert   (apt install librsvg2-bin)   -- small, deterministic
#   - inkscape
#
# See docs/icons.md and docs/adr/0001 "Amendment: raster icon assets".
set -eu

size=256
dir="$(CDPATH= cd -- "$(dirname -- "$0")/../assets/icons" && pwd)"

if command -v rsvg-convert >/dev/null 2>&1; then
    render() { rsvg-convert -w "$size" -h "$size" "$1" -o "$2"; }
elif command -v inkscape >/dev/null 2>&1; then
    render() { inkscape "$1" -w "$size" -h "$size" -o "$2" >/dev/null 2>&1; }
else
    echo "render-icons: need rsvg-convert (apt install librsvg2-bin) or inkscape" >&2
    exit 1
fi

for svg in "$dir"/*.svg; do
    png="${svg%.svg}.png"
    render "$svg" "$png"
    echo "  $(basename "$svg") -> $(basename "$png")  (${size}x${size})"
done
