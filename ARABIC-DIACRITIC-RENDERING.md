# Arabic diacritic (tashkeel) rendering — investigation and plan

## Status

- **Track 1 (ayah list view via page glyphs) — done.** See "Track 1" below for what shipped, plus follow-on cleanup (wrapping/spacing/duplicate marker fixes) and a header/basmala unification that grew out of it.
- **Priority 0 (surah name dagger-alef reposition) — not yet implemented.**
- **Track 2, item 2 (baseline-shift subset) — not implemented, still a known follow-up.**
- **Sidebar surah list dagger-alef/baseline-shift bugs — not yet touched.**

## Situation

The surah header, sidebar surah list, and ayah-by-ayah list view were all showing broken tashkeel (merged, missing, or floating diacritics), even after switching `--font-quran` to only use the bundled `Amiri Quran` / `Noto Naskh Arabic` fonts (dropping the system-installed `KFGQPC Uthmanic Script HAFS`, which had its own separate, narrower shaping problem — that swap is already committed).

Further investigation (rendering real test pages through the same WebKitGTK engine the app uses, then screenshotting and pixel-measuring them) found **two distinct, confirmed font-shaping bugs** present in both bundled fonts on this system:

1. **Dagger alef (`ٰ` / U+0670) never attaches to its base letter.** It renders as a disconnected floating mark that looks like a stray "١". Reproduced in complete isolation (even a bare `ا` + `ٰ` breaks). Switching between the two bundled fonts doesn't help — both fail identically. Disabling ligatures/contextual alternates via `font-feature-settings` (`calt`, `rlig`, `liga`, etc.) has no effect either — this is a font-internal GPOS gap, not something routable around via CSS.
2. **Certain shadda + diacritic letter combinations cause the whole word to render offset from the true line baseline.** Confirmed with a same-run baseline reference (rendering `"wordx"` in one text run and comparing where the Arabic glyphs sit versus the Latin `x` shaped alongside them).

### Why the continuous Mushaf page view looked fine

`PageView.svelte` never live-shapes ayah body text at all. Each word is rendered as a pre-baked glyph from a page-specific font (`QCF_P{page}.woff2`, loaded via `loadPageFonts` in `src/lib/utils/mushaf-fonts.ts`), addressed by `word.glyph_v2`. There's no Unicode combining-mark shaping happening for verse text in that view, so neither bug can occur there.

Everywhere else renders live Unicode text through the buggy font stack and inherits both bugs:

- **Ayah-by-ayah list view** (`AyahRow.svelte`, rendered from `ReaderView.svelte`) renders `ayah.uthmani_text` directly. Dagger alef alone appears in **4,367 of 6,236 ayahs (70%)** — so this is pervasive there, not cosmetic.
- **Surah header** — both `SurahHeader.svelte`'s banner and `PageView.svelte`'s own in-page `surah_header` line render the plain `name_ar` string live.
- **Sidebar surah list** (`Sidebar.svelte`) — same `name_ar` string, live-shaped.

### Scope, verified against all 114 real surah names

Rendered every `name_ar` value from `database/quran.db` through the actual bundled fonts:

- **Dagger-alef bug:** surah IDs **42, 55, 87, 93** — deterministic, always reproducible.
- **Baseline-shift bug:** a further **~6–9 names**, severity varies — clearly confirmed for **4, 17, 43, 48, 71, 94**; milder/borderline for **21, 26, 88**.

## Plan (not yet implemented)

### Priority 0 — Surah name dagger-alef: reposition, don't strip

This is Quranic text (surah names), so removing the diacritic is off the table even though it's only a display-level fix. Since this only affects **4 fixed strings** (surah IDs 42, 55, 87, 93 — see scope below), a hand-tuned visual reposition is feasible where it wasn't for the ~6-9 baseline-shift names (Track 2, item 2):

- At the same 3 render sites (`SurahHeader.svelte`'s `{surah.name_ar}`, `PageView.svelte`'s `{line.text}` for `surah_header` lines, `Sidebar.svelte`'s `{surah.name_ar}`), wrap the dagger alef (U+0670) character in its own `<span>` instead of stripping it.
- Apply a hand-measured `position: relative` (or `transform: translate(...)`) offset per name so the mark visually sits back over its base letter, compensating for the font's broken GPOS attachment.
- Measure the actual offset needed for each of the 4 names using the same real-page-render + screenshot + pixel-measure method already used to confirm the bug.
- `font-feature-settings` (calt/rlig/liga/etc.) still won't help here — already confirmed no effect, since this is a GPOS anchor gap, not a shaping-feature toggle. The fix is manual positioning, not a CSS shaping lever.
- The underlying `name_ar` field and all non-visual consumers (search, copy) are untouched regardless — this only changes the 3 render sites.

This supersedes the "strip U+0670" approach originally sketched below for these 4 names; do this instead.

### Track 1 — Ayah list view: render via page glyphs instead of live text — ✅ done

Reworked `AyahRow.svelte`/`ReaderView.svelte` to reuse the same page-glyph technique `PageView.svelte` already had, eliminating both bugs there entirely (matching Mushaf-mode quality). Implemented as planned, with a few things the plan didn't anticipate:

**`src/lib/components/reader/ReaderView.svelte`**:

- Added the same page-fetch machinery `PageView.svelte` already had: computes `firstPage`/`lastPage` from the `ayahs` prop, `Promise.all`s `getPage(p)` (`src/lib/api/db.ts`) for every page in range plus `loadPageFonts(pageNumbers)` (`src/lib/utils/mushaf-fonts.ts`).
- Walks every fetched page's `lines[].words[]` and groups into a persistent `SvelteMap<ayahId, AyahGlyphWord[]>` (`AyahGlyphWord` — new type in `src/lib/types/database.ts` — tags each word with the font-family of the page it came from, since an ayah split across a page boundary has words in two different QCF fonts). Only `text`-type lines are grouped; `basmala`/`surah_header` lines are handled separately (see the header/basmala unification below).
- Passes the matching word list into `<AyahRow words={...} />`. The map is a single persistent `SvelteMap` instance, cleared and repopulated in-place on each effect run (reassigning to a new instance isn't reactive without wrapping in `$state`, and wrapping a `SvelteMap` in `$state` is itself redundant/lint-flagged — it's already reactive).

**`src/lib/components/reader/AyahRow.svelte`**:

- Replaced the live-text `<p class="ayah-text quran-text">{ayah.uthmani_text}...</p>` with a word-span loop mirroring PageView's `.text-line`/`.word` markup, plus fixes for two issues only visible once actually rendered:
  - **Wrapping:** copying PageView's exact markup (which deliberately suppresses inter-span whitespace, since Mushaf lines never wrap) made every ayah a single unbreakable run that overflowed the row instead of wrapping. Fixed with a literal space character between word spans (not a `{' '}` mustache — flagged by `svelte/no-useless-mustaches`) plus `overflow-wrap: anywhere` on `.ayah-text` as a safety net.
  - **Spacing:** the QCF glyph fonts' own space glyph is much narrower than a live-shaped font's, so once wrapping relied on a real space character, word gaps read as cramped. Added `margin-inline-end: 0.35em` on `.word`.
  - **Duplicate ayah-ending mark:** the last word's `glyph_v2` in each ayah already bakes in the Mushaf ayah-end ornament (digit-in-circle) as part of the page-glyph data, so the app's own hand-drawn `.ayah-marker` circle (driven by a `showAyahNumbers` prop) was a redundant second marker. Removed it, and cascaded the now-dead `showAyahNumbers` prop out of `AyahRow` → `ReaderView` → `ReaderPage` (it had no actual settings-UI toggle wired to it, so nothing user-facing regressed).
- `ayah.uthmani_text` is kept as-is for the `copyText()` clipboard handler, unrelated to the display glyphs.
- Unlike PageView, doesn't preserve per-line justification/grouping — one flowing, wrapping word sequence per ayah.

No new Tauri/DB endpoints were needed; `getPage` already returns everything required. Font loading/eviction reuses `loadPageFonts` unchanged.

#### Follow-on: unified Surah header + Bismillah across both views

Once Track 1 established the page-glyph fetch in `ReaderView`, the same technique was extended to the Surah header banner and Bismillah line so both reading modes share one component and one (bug-free) rendering path, instead of `SurahHeader.svelte` (list view) and `PageView.svelte`'s inline banner being two separate, divergent implementations:

- **`SurahHeader.svelte`** is now the single shared header component, used by both `ReaderView` and `PageView`. Restyled to the simpler bordered `۞ Name ۞` banner (previously only in `PageView`'s inline markup), dropped the decorative id/verse-count roundels and brackets that were only in the old list-view banner. Subtitle trimmed to `{transliteration} · {revelation_type} · {verses_count} verses` (dropped ruku count and the Arabic revelation-type chip per follow-up feedback). Bismillah now renders via actual QCF glyph data (`GlyphSpan[]` — new minimal type in `database.ts`, factored out of `AyahGlyphWord`) exactly like `PageView`'s basmala line always did, falling back to live `BISMILLAH_TEXT` only when glyph data isn't available in the currently-fetched page range (e.g. a Juz/Hizb view that starts mid-Surah, so the Surah's actual header+basmala page is outside the fetched range).
- **`ReaderView.svelte`** additionally fetches the Bismillah glyph words and `QCF_BSML` font (via `loadBasmalaFont()`) from the same page fetch already used for ayah words, pairing each `basmala`-type page-line to the `surah_id` of the `surah_header`-type line immediately preceding it (the two are always adjacent on the page that opens a Surah; `page_line.surah_id` is only populated on `surah_header` rows).
- **`PageView.svelte`** now renders `<SurahHeader>` instead of its own inline `surah-header-line`/`basmala-line` markup, doing the same adjacent-line pairing.
- Note: the Surah name itself (`{surah.name_ar}`) inside the shared header is still live-shaped plain text (no page-glyph data exists for it, per Track 2 below) — still subject to the dagger-alef/baseline-shift bugs until Priority 0 / Track 2 item 2 land.

### Track 2 — Surah header + sidebar list (114 fixed strings)

No page-glyph data exists for these labels (`PageLine.words` is only populated for `text`-type lines; `surah_header` lines carry just a plain `text` string), so Track 1's approach doesn't transfer here.

1. **Dagger-alef fix (IDs 42, 55, 87, 93) — superseded, see Priority 0 above.** Originally sketched as stripping U+0670 via a `stripDisplayOnlyMarks(text: string)` helper; rejected in favor of repositioning the mark instead of removing it, since this is Quranic text. Do not strip — see Priority 0.

2. **Baseline-shift subset (~6–9 names) — leave as a known follow-up, not fixed in this pass.** No clean CSS/text-level fix exists: it's a font-internal GPOS anchor gap (confirmed `font-feature-settings` has no effect). The real fixes — sourcing/vetting a third Arabic font, or hand-tuning ~10 fixed strings with manual per-glyph offsets — are bigger asks than this pass warrants. Revisit only if it turns out to bother you in practice.

## Verification

### Track 1 + header/basmala unification — done

- `pnpm run build`, `pnpm run check`, `pnpm run lint` all pass clean.
- Manually confirmed in the running app: list-mode ayah text now matches Mushaf-mode glyph quality (no floating dagger-alef, no baseline shift); wrapping/spacing/duplicate-marker issues caught in manual review were fixed (see Track 1 above).
- Still worth spot-checking: an ayah whose text spans a page boundary (font-family should switch mid-ayah without a visible glyph mismatch), and a Juz/Hizb view starting mid-Surah (Bismillah should fall back to live text gracefully, not disappear).

### Priority 0 / Track 2 item 2 — not yet done, so not yet verifiable

- Once Priority 0 is implemented: check surahs 42, 55, 87, 93 in the (now-shared) header banner and the sidebar list — dagger alef should sit correctly over its base letter, not stripped and not floating.
- Spot-check a control name without U+0670 (e.g. 2, 109) to confirm the reposition logic is a no-op there.
