# Mushaf Page View — Status (resolved)

**Date resolved:** 2026-07-25
**Goal:** render each of the 604 Mushaf pages with the same line breaks and word
placement as the printed Madani Mushaf (pixel-accurate glyph fonts), instead of
the existing infinite-scroll-by-ayah reader.

## TL;DR

Done and confirmed working, including a live screenshot check in the actual
Tauri app (WebKitGTK on Linux, the primary target). The blocking bug (QCF v2
rendering blank) is fixed by re-vendoring the per-page fonts from a different,
properly HarfBuzz-compatible source. No importer or layout-data changes were
needed.

## What's done and working

1. **Schema**: `page_line` / `page_line_word` tables in `database/schema.sql`
   / `database/migrations/002_mushaf_layout.sql`.
2. **Importer** (`importer/src/mushaf.rs`): fetches all 604 page-layout JSON
   files from `zonetecde/mushaf-layout` (ISC-licensed), synthesizes all 114
   Surah headers from validated Surah/Ayah data rather than trusting the
   source's own unreliable header lines (see git history for the details of
   the two data-quality bugs found and fixed there). Unchanged by the font
   fix below — the `qpcV2` glyph codepoints in this data turned out to be
   exactly the codepoints the fixed font uses.
3. **Fonts vendored**: `static/fonts/mushaf/QCF_P001..604.woff2` +
   `QCF_BSML.woff2` via `scripts/vendor-mushaf-fonts.sh` (~54MB).
4. **Rust API / TS API / Frontend**: unchanged from the original
   implementation — `MushafPage`/`PageLine`/`PageLineWord` models, `get_page`
   query/command, `getPage()` wrapper, `PageView.svelte` +
   `mushaf-fonts.ts`, toggled from the toolbar via `uiStore.readingMode`.
5. `pnpm check` and `pnpm lint` both pass clean.

## The bug that blocked this, and the fix

**QCF v2 as originally vendored from `nuqayah/qpc-fonts`
(`mushaf-woff2/QCF_PNNN.woff2`) rendered almost entirely blank** in any
HarfBuzz-based engine — Chromium, and critically WebKitGTK (what Tauri uses
on Linux).

Root cause (confirmed with `fontTools`): the font's word-glyph codepoints map
via `cmap` directly to **zero-contour glyphs**. The real letterforms only
exist behind the font's `morx` table (Apple Advanced Typography glyph
substitution) — there's no `GSUB` table at all. `morx` is CoreText/Safari
machinery; HarfBuzz doesn't process it for this font, so the substitution
never happens.

**The fix**: Quran Foundation (the org behind quran.com/QUL) hosts a properly
converted build of the exact same font — same King Fahd Complex/Uthman Taha
artwork, same `qpcV2` codepoints as the already-imported
`zonetecde/mushaf-layout` data, but with a real `GSUB` table (no `morx`) and
`cmap` mapping directly to non-empty contours. Confirmed with `fontTools`,
confirmed with a browser render, and confirmed live in the actual Tauri app.

- **Basmala font** (`QCF_BSML`, from `nuqayah/qpc-fonts`) was already fine —
  its glyphs have real contours directly on `cmap` even though the per-page
  fonts didn't. Left as-is. `PageView.svelte` previously used the current
  page's own font for the Basmala line too (a latent bug, since the Basmala
  codepoints aren't even present in most pages' fonts) — fixed by adding
  `loadBasmalaFont()` in `mushaf-fonts.ts` and using it specifically for the
  `basmala` line type.
- Considered and rejected: **QCF4** (`MohamadHajjRabee/quran-qcf4`) also
  renders correctly (no `morx`, real contours on `cmap`) and was the
  fallback plan, but has worse provenance (independently repackaged,
  "previously unpublished") and would have required redoing the importer
  against a different JSON shape and re-vendoring 47 files. Not needed once
  the Quran Foundation build was found.
- Quran Foundation's own docs recommend loading these fonts live from their
  CDN rather than storing them locally (for freshness — they periodically
  ship corrections). Vendored anyway, deliberately, to keep this app
  offline-first like the rest of its data; see `scripts/vendor-mushaf-fonts.sh`
  for the reasoning.
- **Vendoring caveat worth remembering**: the download loop hit intermittent
  stalls/rate-limiting against this CDN. The script now uses
  `--connect-timeout`/`--max-time`/`--retry` and fails loudly on any bad
  download rather than silently leaving a stale or truncated file in place
  — a first pass without those guards left several pages silently on the old
  broken build, and separately a few files were truncated mid-transfer with
  a valid-looking WOFF2 header but corrupt Brotli payload. If re-vendoring
  ever needs to happen again, verify afterwards with `fontTools` (check
  `cmap`-mapped glyphs for non-zero contours) rather than trusting the
  script's own log or file sizes alone.

## Other bugs found and fixed along the way

1. **`name_ar` double-"سورة" prefix**: `surah.name_ar` in the DB already
   includes the "سُورَةُ" prefix, but `SurahHeader.svelte` and `PageView.svelte`
   both prepended "سورة" again. Fixed by dropping the hardcoded prefix from
   both components (display-only fix, no re-import needed).

## Remaining known simplification (unchanged from original plan)

Line justification uses flexbox `space-between`, not true kashida
stretching (browsers don't support the OpenType `jstf` justification tables
these fonts were authored for). Revisit if it reads as too sparse on short
lines.
