# QCF v2 → v4 Mushaf Font Migration — Plan

**Date drafted:** 2026-07-28
**Status:** implemented on `claude/qcf-v4-font-migration-jcupky` (2026-07-28),
not merged to master. Rendering switched to v4; `glyph_v2` and the v2 fonts
are kept for rollback, per this plan's own Rollback section. Line-height
retuning (step 6) is a numeric first pass only — this branch was built in a
sandbox that can't run the actual Tauri/WebKitGTK app, so the empirical
"look at a real rendered page" verification this plan calls for in step 1
and the verification checklist has **not** been done. Treat this as the spike
this plan asked for, not a finished migration — see the branch's commit
message and PR description for the full list of what still needs human eyes
before this could ship.
**Goal:** replace the 604 per-page QCF v2 fonts with the 47-file QCF v4 set,
cutting ~58 MB off the install, without changing any user-visible feature
other than the glyphs themselves.

## TL;DR

Worth doing. The v4 set renders the same Uthman Taha calligraphy in 38% of
the bytes, and is cheaper per page to decode. It is **not** a drop-in swap:
the vertical metrics and the Unicode bidi class of the glyph codepoints both
change, and each needs handling. Budget a spike before committing.

This is a font-layer change only. No feature is added, removed, or altered.

## Why — measured, not assumed

All figures below were measured locally against the vendored
`static/fonts/mushaf/` and a downloaded `QCF4_Hafs_01_W.woff2`.

|                  | Files         | Total woff2   | Per page                |
| ---------------- | ------------- | ------------- | ----------------------- |
| QCF v2 (current) | 604 + basmala | **93.4 MiB**  | 158 KB mean (40–221 KB) |
| QCF v4           | 47 + basmala  | **35.65 MiB** | 60 KB mean              |

Install size goes from ~115 MB to roughly **57 MB**, since the fonts are
compiled into the binary via `frontendDist`.

**There is no cheaper way to get this.** The v2 files are already at their
floor:

- woff2 already beats raw brotli q11 on the same TTF (185.4 KB vs 188.6 KB
  for page 300) — no recompression headroom.
- Across all 604 files: 88,531 glyphs, **88,491 unique**. QCF v2 glyphs are
  per-page justified word ligatures, so the same word is a different outline
  on every page. Zero dedup headroom.
- `glyf` is 92–99% of every file; stripping hinting/metadata gains ~1%.
- Converting to CFF makes it _larger_ (235.6 KB vs 185.4 KB) — woff2's glyf
  transform only applies to quadratics.
- Outline simplification yields 6–12% but is **prohibited by the KFGQPC
  terms** already recorded in `THIRD-PARTY-NOTICES.md` ("modifying,
  altering… "). Not an option.

v4 is smaller because KFGQPC rebuilt the font: **366 points per glyph vs
v2's 790**, at the same 2500 upem, plus glyph sharing across the ~13 pages
each file covers. Rendering both side by side shows no loss of detail —
the letterforms are the same calligraphy, and v4's end-of-verse medallion
is cleaner.

### Runtime cost

|                          | Size   | Decode  | Per page   |
| ------------------------ | ------ | ------- | ---------- |
| v2, median page (P300)   | 185 KB | 11.4 ms | 11.4 ms    |
| v4, one font (~13 pages) | 924 KB | 57.8 ms | **4.4 ms** |

Amortized, v4 is ~2.6× cheaper per page. But a cold open of a new font group
costs 57.8 ms against 11.4 ms — a 5× spike. Startup font loading was already
the dominant cost once (see the header of `src/lib/utils/mushaf-fonts.ts`),
so this spike is the single thing most worth measuring early.

## Scope boundary — what must NOT change

The whole point of this change is that it is invisible except for the glyphs.
Nothing below is touched:

- `surah`, `ayah`, `translation`, `translation_ayah`, `tafsir`, `tafsir_ayah`
  — untouched.
- `bookmark`, `note`, `settings` — untouched. Bookmarks and notes key off
  `ayah_id`, not page/line/glyph, so they survive a layout re-import intact.
- Reading modes, reader zoom, search, navigation, the go-to dialog,
  translations panel — untouched.
- The non-mushaf reading path (`uthmani_text` rendered with Scheherazade New)
  — untouched. It is the fallback if any of this goes wrong.
- Tauri commands, Rust models, the TS API surface — unchanged in shape.
  `PageLineWord` gains a field; nothing is renamed or removed.

If a step in this plan requires touching anything on that list, stop and
reconsider — it means the change has outgrown "font swap".

## The two real risks

### 1. Vertical metrics differ by ~39%

|     | ascent | descent | line box |
| --- | ------ | ------- | -------- |
| v2  | 2809   | −1301   | 1.64 em  |
| v4  | 3706   | −1986   | 2.28 em  |

At the same `font-size`, v4 occupies 39% more vertical space. A naive swap
will overflow the page and blow the 15-line grid. `--font-size-quran` and its
breakpoints (28 / 24 / 21 px in `src/app.css`) plus the page-view line height
need retuning together, against a real rendered page.

### 2. The bidi class of the glyph codepoints changes

- v2 glyphs are Arabic Presentation Forms (`U+FC41…`), bidi class **AL** —
  strong RTL. The browser orders them for free.
- v4 glyphs are Private Use Area (`U+F100…`), bidi class **L** — no implicit
  reordering at all.

`AyahRow.svelte` emits one `<span>` per word and relies on surrounding
direction for order; `.ayah-translation` already sets `direction: ltr`
explicitly. Word order, text selection, and copy-to-clipboard must all be
re-verified after the swap. **This is the subtlest risk and the most likely
to fail quietly** — the page can look plausible while selection order is
reversed.

This bit me while researching: a naive render put the ayah marker on the
wrong side for v2 precisely because of this class difference.

## Implementation steps

Ordered so each step is independently verifiable and the app still builds
between them.

### 1. Spike first — do not skip

Hand-wire a single v4 font group behind a temporary flag and look at one page
in the real app (WebKitGTK). Confirm glyph rendering, word order, line fit.
Both risks above surface immediately here. **If the spike is ugly, stop —
the cost is one afternoon instead of a fortnight.** Delete the flag before
merging; it is a scaffold, not a feature.

### 2. Vendor the fonts

Rewrite `scripts/vendor-mushaf-fonts.sh` for the v4 source: 47
`QCF4_Hafs_NN_W.woff2` + `QCF4_QBSML`. Keep the existing structure and the
license header. Source is
[MohamadHajjRabee/quran-qcf4](https://github.com/MohamadHajjRabee/quran-qcf4)
via jsDelivr; vendor rather than CDN-load, consistent with offline-first.

Also fetch and commit `font-map.json` (604 pages → 47 families).

### 3. Importer

`importer/src/mushaf.rs` currently reads `zonetecde/mushaf-layout` and its
`qpcV2` codepoints. v4 ships its own `pages/NNN.json` with a per-word `code`
(integer codepoint) and `font` field:

```json
{
  "code": 61696,
  "font": "QCF4_Hafs_01",
  "text": "بِسْمِ",
  "type": "word",
  "verse_key": "1:1",
  "position": 1
}
```

Add a v4 path that populates a new `glyph_v4` column. Keep the existing
Surah-header synthesis logic — it exists because the old source's header
lines were unreliable, and that reasoning is independent of font version.

### 4. Schema

Migration `003`: `ALTER TABLE page_line_word ADD COLUMN glyph_v4 TEXT`, and
bump `CURRENT_VERSION` to 3 in `src-tauri/src/db/connection.rs`. Mirror the
column into `database/schema.sql`.

Keep `glyph_v2` populated for now. It costs little, makes rollback a
one-line frontend change, and lets both render paths coexist during the
spike. Drop it in a separate follow-up commit once v4 is proven.

**Existing installs need care.** `connection.rs` seeds from the embedded
`SEED_DB` only `if !path.exists()`, so a user upgrading keeps their old
`~/.local/share/quranreader/quran.db` and a plain `ALTER TABLE` leaves
`glyph_v4` NULL — a blank mushaf. `page_line` / `page_line_word` hold no user
data, so the fix is to rebuild them from the embedded seed during migration
(write `SEED_DB` to a temp file, `ATTACH`, copy the two tables, `DETACH`).
That mechanism generalizes to any future content update and is worth
building properly rather than special-casing.

### 5. Frontend font loading

`src/lib/utils/mushaf-fonts.ts`:

- `familyForPage()` / `urlForPage()` become `font-map.json` lookups instead
  of deriving the name from the page number.
- The in-flight/LRU logic keys on **font family**, not page — several pages
  now share one face, and the current per-page keying would fetch the same
  file repeatedly.
- Drop `MAX_FONTS` from 64 to ~8. At v4's ~761 KB mean, 64 faces would be
  ~48 MB resident; 8 faces is ~6 MB and covers ~100 pages — strictly better
  coverage than today for less memory.
- Update the file header comment; its arithmetic ("604 files, ~95MB",
  "roughly 8MB") is all v2-specific.

### 6. Rendering

`AyahRow.svelte` and `PageView.svelte` read `glyph_v4`. Retune
`--font-size-quran` and line height per step 1's findings. Re-verify word
order, selection, and copy under the new bidi class.

### 7. Docs and licensing

Update `THIRD-PARTY-NOTICES.md`: same KFGQPC / Uthman Taha ownership and the
same terms, but now the 1441 AH Madinah Mushaf edition, sourced via
quran-qcf4 (its JSON is MIT; the fonts are not). The legal position is
unchanged — neither better nor worse than today. Note the upstream repo is
small (5 stars, last pushed 2026-03-15), which is a further argument for
vendoring over a live CDN dependency.

## Verification checklist

- [ ] All 604 pages render, no tofu, no blank pages (the v2 failure mode
      documented in `docs/mushaf-page-view-status.md`).
- [ ] 15-line grid holds at every zoom level and window size.
- [ ] Word order correct; selection and copy produce correct text order.
- [ ] Surah headers and basmala lines render (separate `QCF4_QBSML` face).
- [ ] Bookmarks and notes created before the migration still resolve.
- [ ] Reading position survives the upgrade.
- [ ] Cold page-group open measured; startup not regressed vs 0.1.1.
- [ ] Fresh install and in-place upgrade both tested — the upgrade path is
      the one that can silently produce a blank mushaf.
- [ ] `pnpm check` and `pnpm lint` clean.

## Rollback

While `glyph_v2` and the v2 fonts are both still present, rollback is a
one-line change in the render component. After the follow-up commit that
drops them, rollback is a revert of that commit. Do not drop v2 until at
least one release has shipped on v4.

## Open questions

- Does the 1441 AH edition's line breaking differ from the current layout
  data anywhere? Both are 604 pages × 15 lines, but per-line word
  distribution is worth spot-checking against a printed reference.
- Is the 57.8 ms cold-decode acceptable on first page open, or does the
  group need prefetching on an idle callback?
