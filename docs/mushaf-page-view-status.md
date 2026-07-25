# Mushaf Page View — Status (paused mid-implementation)

**Date paused:** 2026-07-25
**Goal:** render each of the 604 Mushaf pages with the same line breaks and word
placement as the printed Madani Mushaf (pixel-accurate glyph fonts), instead of
the existing infinite-scroll-by-ayah reader.

## TL;DR

The data pipeline (schema, importer, Rust/TS API, Svelte component) is fully
built and verified correct. **The font we chose (QCF v2) does not render —
it's blocked on a font-compatibility bug, not a bug in this app's code.**
Was about to try QCF4 as the fix when this was paused. Resume there.

---

## What's done and working (don't redo)

1. **Schema**: `page_line` / `page_line_word` tables in `database/schema.sql`
   - `database/migrations/002_mushaf_layout.sql`. Bumped `CURRENT_VERSION` to
     2 in `src-tauri/src/db/connection.rs`.
2. **Importer** (`importer/src/mushaf.rs`): fetches all 604 page-layout JSON
   files from `zonetecde/mushaf-layout` (ISC-licensed), resolves
   `surah:ayah:word` locations to `ayah.id`, inserts `page_line` /
   `page_line_word`. Wired into `importer/src/main.rs` as steps 5–6.
   - **Fixed two real data-quality bugs in that source dataset**: 17 Surahs
     were missing their `surah-header` line, and 13 had a stray duplicate
     with the wrong Surah number at a page boundary (verified against the
     actual Mushaf, e.g. bogus "Surah 3" header on page 76, the page _after_
     Aal-Imran already ended). Fix: **discard the source's header lines
     entirely and synthesize all 114 from our own validated Surah/Ayah data**
     (`synthesize_surah_headers()` in `mushaf.rs`). This part is solid,
     keep it regardless of font decision.
   - Two Surahs (81 At-Takwir, 85 Al-Burooj) have no Basmala glyphs in the
     source data at all — logged as a warning
     (`warn_missing_basmala()`), not faked. Minor, cosmetic-only gap.
3. **Fonts vendored**: `static/fonts/mushaf/QCF_P001..604.woff2` +
   `QCF_BSML.woff2` via `scripts/vendor-mushaf-fonts.sh` (~48MB). This is the
   part that turned out to be broken — see below.
4. **Rust API**: `MushafPage`/`PageLine`/`PageLineWord` models
   (`src-tauri/src/models/mod.rs`), `get_page()` query
   (`src-tauri/src/db/queries.rs`), `get_page` Tauri command
   (`src-tauri/src/commands/mod.rs`), registered in `src-tauri/src/lib.rs`.
5. **TS API**: types in `src/lib/types/database.ts`, `getPage()` wrapper in
   `src/lib/api/db.ts`.
6. **Frontend**: `src/lib/components/reader/PageView.svelte` (renders lines,
   prev/next + jump-to-page nav) + `src/lib/utils/mushaf-fonts.ts` (lazy
   per-page `FontFace` loading with a small LRU so we don't declare 604
   `@font-face` rules up front). Toggle wired into
   `src/lib/components/layout/Toolbar.svelte` (book icon) via
   `uiStore.readingMode` (`src/lib/stores/ui.svelte.ts`), rendered from
   `src/routes/surah/[id]/+page.svelte`.
7. `PLAN.md` updated with the full research/decision trail under Phase 1 and
   a new subsection under Phase 5.
8. `pnpm check` and `pnpm lint` both pass clean (0 errors/warnings). Had to
   add a small ESLint config fix (`eslint.config.js`) — base `no-unused-vars`
   doesn't understand TS type-only positions in `.svelte` files, so extended
   the existing TS-aware override to `.svelte` files too.

## The blocking bug

**QCF v2 (the King Fahd Complex font we vendored from `nuqayah/qpc-fonts`,
`mushaf-woff2/QCF_PNNN.woff2`) renders almost entirely blank** in any
HarfBuzz-based engine — Chromium, and critically **WebKitGTK, which is what
Tauri uses on Linux, our primary target platform.**

Root cause, confirmed with `fontTools`:

- The font's word-glyph codepoints (the `qpcV2` strings from the
  `zonetecde/mushaf-layout` dataset) map via `cmap` directly to glyphs with
  **zero contours** — intentionally empty shells.
- The real letterforms only exist behind the font's `morx` table (Apple
  Advanced Typography glyph substitution — confirmed present via
  `fontTools`; there is **no `GSUB`** table at all). `morx` is CoreText/Safari
  machinery; HarfBuzz (used by Chromium/Firefox/WebKitGTK) does not process
  it for this font, so the substitution never happens and you get blank glyphs.
- Tested and ruled out as a red herring: WOFF2 vs. original TTF (both blank,
  so it's not a bad conversion), single `<span>` per line vs. one `<span>`
  per word (no difference — not a DOM-structure issue).
- **Confirmed live in the actual Tauri app** (not just a synthetic test):
  toggling to Mushaf view on Al-Baqara page 2 shows the Surah banner, then
  only 2–3 words of the first line, then blank for the rest of the page.
  Screenshots are in this conversation if needed; not saved to disk.
- The Basmala line also needs its _own_ separate font (`QCF_BSML`/`QPC2BSML`,
  confirmed by reading `zonetecde`'s own generator source — see
  `QPCFontProvider.getBasmalaFont()` in their `src/index.ts`), not the
  per-page font as `PageView.svelte` currently assumes. Moot until the main
  text-glyph issue is fixed, but fix both at once.

**Practical implication**: this font would only ever have worked correctly on
macOS (WKWebView/CoreText). It's dead on arrival for Linux (and probably
Windows/WebView2, also Chromium-based) — Tauri's two other target platforms
per `PLAN.md`.

### What was about to be tried next: QCF4

`MohamadHajjRabee/quran-qcf4` (47 files, the option not chosen earlier for
provenance reasons) was inspected with `fontTools` right before this was
paused:

- **No `morx`/`feat`/AAT tables at all** — just plain `cmap`/`glyf`.
- Sampled codepoints (`0xf101`, `0xf8dd`, `0xf124`, `0xf125`, `0xf126`) all
  have **real, non-zero contours directly via `cmap`** — i.e. no
  substitution step needed, which is exactly what was missing for QCF v2.
- This strongly suggests QCF4 will "just work" in any standard engine, but
  **this was not yet actually rendered/screenshotted** — that's the next
  concrete step, not a conclusion yet.
- Font already downloaded to the test harness:
  `/tmp/claude-1000/.../scratchpad/servedir/fonts/mushaf/QCF4_Hafs_01.woff2`
  (this is in `/tmp`, will not survive a reboot — re-download from
  `https://cdn.jsdelivr.net/gh/MohamadHajjRabee/quran-qcf4@main/fonts-woff2/QCF4_Hafs_01_W.woff2`
  if it's gone).
- Page 2 layout JSON for QCF4 also already fetched to
  `scratchpad/qcf4-page002.json` (structure: `page.lines[].words[]` with
  `type` (`word`/`end`/`surah_header`/`bismillah`/`quarter`), `char` (PUA
  codepoint), `font`, `verse_key`, `position`) — this is a **different JSON
  shape** than `zonetecde`'s data, so swapping fonts means swapping the whole
  layout-data source too, which means redoing the importer's parsing (not
  just the font vendoring step).
- Provenance caveat still stands from the original research: QCF4 is,
  per its own README, "previously unpublished," repackaged by one
  independent developer — QUL's own official V4 release was still listed as
  "unavailable pending review" as of that research. Worth another look at
  whether that's changed, and/or whether there's a way to get a proper
  GSUB-converted build of QCF v2 instead (Quran.com renders QCF v2 fine on
  the web today, so a compatible build likely exists somewhere in that
  ecosystem — wasn't tracked down before this paused).

## Other things found along the way (unrelated bugs, still open)

1. **Pre-existing bug, not introduced by this work**: `surah.name_ar` in the
   DB already includes the "سُورَةُ" ("Surah") prefix (e.g. "سُورَةُ البَقَرَةِ" for
   Al-Baqara), but both `SurahHeader.svelte` (existing scroll reader) and my
   new `PageView.svelte` header prepend "سورة" again, producing a visible
   duplicate ("سورة سُورَةُ البَقَرَةِ"). `importer/src/parse.rs` even has a comment
   saying "We strip the 'سُورَةُ ' prefix if present" but the code never actually
   does it (`let name_ar = sm.name.clone();`). Fix in one of two places: strip
   the prefix in `parse.rs` at import time (cleaner, fixes it everywhere), or
   drop the hardcoded "سورة" prefix from the two Svelte components.
2. **Runtime-vs-bundled DB gotcha (dev-environment only, but worth
   remembering)**: `src-tauri/src/db/connection.rs` only seeds
   `~/.local/share/quranreader/quran.db` from the bundled DB **if that file
   doesn't already exist**. Migrations only add empty tables (schema, no
   data) for pre-existing installs. During this session that meant the
   running dev app kept reading a stale pre-mushaf DB even after rebuilding
   `database/quran.db` — had to manually `rm` the runtime copy to force a
   reseed. If this bites again: delete
   `~/.local/share/quranreader/quran.db*` and relaunch.
3. **Environment note**: `pnpm` briefly went missing after a disk cleanup
   (corepack's cached binary got wiped). Fix: `corepack enable && corepack
prepare pnpm@10.28.2 --activate`.
4. **Environment note**: the Tauri window is a native Wayland surface under
   Hyprland — `xdotool` (X11-only) can't see or click it, only the app owner
   interacting directly (or a Wayland-native automation tool, none installed)
   can drive it. Screenshots/clicks in this session were done by the user,
   not by me.

## Suggested next steps (in order)

1. Render QCF4 in the test harness (`scratchpad/servedir/` +
   `scratchpad/screenshot.mjs` pattern already set up) using the already
   fetched `QCF4_Hafs_01.woff2` + `qcf4-page002.json`. Confirm it actually
   shows real Arabic before doing anything else.
2. If QCF4 renders correctly: decide whether the provenance concern is
   acceptable, then redo the importer against `quran-qcf4`'s JSON shape
   (different fields — `code`/`char`/`type`/`font` per word, `font-map.json`
   for page→font, `verses.json` for the reverse index) and re-vendor 47 font
   files instead of 604+1.
3. If QCF4 also fails: fall back to the originally-recommended approach —
   keep the `page_line`/`page_line_word` layout data (it's font-agnostic,
   already correct), drop glyph-substitution fonts, render with the existing
   bundled Amiri Quran font and CSS-driven line justification instead. Real
   fidelity loss (browser-rendered ligatures, not identical to the printed
   page), but guaranteed to work everywhere.
4. Either way: fix the `name_ar` double-"سورة" bug, and if sticking with a
   per-page-font approach, fix the Basmala font (needs its own
   `QCF_BSML`/equivalent, not the page's own font).
5. Update `PLAN.md`'s "Mushaf Page Layout" section with whatever gets decided
   — right now it still documents the QCF v2 decision as final, which it
   isn't.
