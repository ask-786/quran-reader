#!/usr/bin/env bash
# Downloads the QCF v2 mushaf fonts (King Fahd Quran Printing Complex, Uthman
# Taha calligraphy) used to render pixel-accurate Mushaf pages.
#
# Per-page fonts: https://verses.quran.foundation (Quran Foundation's official
# CDN) — a properly GSUB-converted build. The originally-vendored build from
# nuqayah/qpc-fonts (mushaf-woff2/) only exposes real glyphs via an Apple
# AAT `morx` table with zero-contour glyphs directly on `cmap`, so it renders
# blank in every HarfBuzz-based engine (Chromium, WebKitGTK) — see
# docs/mushaf-page-view-status.md. The Quran Foundation build's `cmap` maps
# directly to real contours (confirmed with fontTools) and uses the exact
# same codepoints as the already-imported zonetecde/mushaf-layout `qpcV2`
# data, so no importer changes were needed.
#
# Basmala font: QCF_BSML.woff2 from nuqayah/qpc-fonts already works correctly
# (real contours on cmap for its glyphs, unlike the per-page fonts) — kept
# as-is.
#
# License: see http://dm.qurancomplex.gov.sa/copyright-2/ — provided for
# Quranic rendering purposes; not to be redistributed/modified/sold as fonts.
# Quran Foundation's docs recommend loading fonts live from their CDN rather
# than storing locally (for freshness); vendored here anyway to keep this
# app offline-first, consistent with how the rest of its data is bundled.
#
# Re-run this script any time the vendored fonts need to be refreshed.

set -euo pipefail

BSML_URL="https://raw.githubusercontent.com/nuqayah/qpc-fonts/master/mushaf-woff2/QCF_BSML.woff2"
PAGE_URL_BASE="https://verses.quran.foundation/fonts/quran/hafs/v2/woff2"
DEST="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/static/fonts/mushaf"

mkdir -p "$DEST"

echo "Downloading QCF_BSML.woff2 (basmala font) …"
curl -fsSL --connect-timeout 10 --max-time 30 --retry 3 --retry-delay 2 -o "$DEST/QCF_BSML.woff2" "$BSML_URL"

failed=()
for i in $(seq 1 604); do
  padded="$(printf '%03d' "$i")"
  out="$DEST/QCF_P${padded}.woff2"
  if ! curl -fsSL --connect-timeout 10 --max-time 30 --retry 3 --retry-delay 2 -o "$out" "$PAGE_URL_BASE/p${i}.woff2"; then
    failed+=("$i")
  fi
  if [ $(( i % 50 )) -eq 0 ]; then
    echo "  … $i/604"
  fi
done

if [ "${#failed[@]}" -gt 0 ]; then
  echo "FAILED to download ${#failed[@]} page(s): ${failed[*]}" >&2
  exit 1
fi

echo "Done. $(ls "$DEST" | wc -l) font files in $DEST"
