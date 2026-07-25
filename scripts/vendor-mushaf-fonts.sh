#!/usr/bin/env bash
# Downloads the QCF v2 mushaf fonts (King Fahd Quran Printing Complex, Uthman
# Taha calligraphy) used to render pixel-accurate Mushaf pages.
#
# Source: https://github.com/nuqayah/qpc-fonts (mushaf-woff2/)
# License: see http://dm.qurancomplex.gov.sa/copyright-2/ — provided for
# Quranic rendering purposes; not to be redistributed/modified/sold as fonts.
#
# Re-run this script any time the vendored fonts need to be refreshed.

set -euo pipefail

REPO_RAW="https://raw.githubusercontent.com/nuqayah/qpc-fonts/master/mushaf-woff2"
DEST="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/static/fonts/mushaf"

mkdir -p "$DEST"

echo "Downloading QCF_BSML.woff2 (header font) …"
curl -sL -o "$DEST/QCF_BSML.woff2" "$REPO_RAW/QCF_BSML.woff2"

for i in $(seq -w 1 604); do
  out="$DEST/QCF_P${i}.woff2"
  if [ -s "$out" ]; then
    continue
  fi
  curl -sL -o "$out" "$REPO_RAW/QCF_P${i}.woff2"
  if [ $(( 10#$i % 50 )) -eq 0 ]; then
    echo "  … $i/604"
  fi
done

echo "Done. $(ls "$DEST" | wc -l) font files in $DEST"
