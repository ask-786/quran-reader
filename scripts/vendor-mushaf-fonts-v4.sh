#!/usr/bin/env bash
# Downloads the QCF v4 mushaf fonts (King Fahd Complex, Uthman Taha
# calligraphy, Madinah Mushaf 1441 AH, font build by Ahmad ElGharib) used to
# render pixel-accurate Mushaf pages — see docs/qcf-v4-font-migration-plan.md.
#
# 47 per-font-group files (each covering ~13 pages) + the Surah-title banner
# font (QCF4_QBSML), replacing the 604 per-page QCF v2 files in
# static/fonts/mushaf/ (kept in place — see the migration plan's rollback
# section). Also fetches font-map.json, the page (1–604) -> font-group lookup
# the frontend needs since a v4 page no longer implies its own font name.
#
# Source: MohamadHajjRabee/quran-qcf4 (public domain JSON, non-MIT fonts — see
# THIRD-PARTY-NOTICES.md). Fetched from raw.githubusercontent.com rather than
# the jsDelivr CDN the upstream README suggests: this is the origin, not a
# mirror, so it's the stronger fit for "vendor rather than CDN-load", and it's
# also what plain HTTPS access allows in some sandboxed environments where
# jsDelivr is blocked but GitHub's raw content host isn't.
#
# Re-run this script any time the vendored v4 fonts need to be refreshed.

set -euo pipefail

RAW_BASE="https://raw.githubusercontent.com/MohamadHajjRabee/quran-qcf4/main"
DEST="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/static/fonts/mushaf-v4"

mkdir -p "$DEST"

echo "Downloading font-map.json (page -> font group) …"
curl -fsSL --connect-timeout 10 --max-time 30 --retry 3 --retry-delay 2 \
  -o "$DEST/font-map.json" "$RAW_BASE/font-map.json"

# QBSML is the Surah-title *banner* font only. It is not used at runtime (the
# banner is drawn live from surah.name_ar in Scheherazade New), and it is NOT
# the basmala font despite the name — its basmala codepoints are zero-contour
# blanks, while QCF4_Hafs_01 carries the real artwork. Vendored anyway so the
# v4 asset set stays complete for anyone wanting real banner glyphs later.
echo "Downloading QCF4_QBSML.woff2 (surah-title banner font, currently unused) …"
curl -fsSL --connect-timeout 10 --max-time 30 --retry 3 --retry-delay 2 \
  -o "$DEST/QCF4_QBSML.woff2" "$RAW_BASE/fonts-woff2/QCF4_QBSML.woff2"

failed=()
for i in $(seq 1 47); do
  padded="$(printf '%02d' "$i")"
  name="QCF4_Hafs_${padded}_W.woff2"
  out="$DEST/$name"
  if ! curl -fsSL --connect-timeout 10 --max-time 30 --retry 3 --retry-delay 2 -o "$out" "$RAW_BASE/fonts-woff2/$name"; then
    failed+=("$name")
  fi
done

if [ "${#failed[@]}" -gt 0 ]; then
  echo "FAILED to download ${#failed[@]} file(s): ${failed[*]}" >&2
  exit 1
fi

echo "Done. $(ls "$DEST" | wc -l) files in $DEST"
