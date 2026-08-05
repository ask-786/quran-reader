# Quran Reader — Development Roadmap

> **Project Goal**
>
> Build a beautiful, fast, lightweight, offline-first Quran reader focused on the reading experience.
>
> Primary target: Linux
> Secondary targets: Windows & macOS (via Tauri)
>
> Core principles:
>
> - Offline first
> - Beautiful typography
> - Native-feeling performance
> - Small install size
> - Privacy (no telemetry)
> - Open source
> - Extensible architecture for future features

---

# Technology Stack

## Desktop

- [x] Tauri v2

## Backend

- [x] Rust

## Frontend

- [x] SvelteKit
- [x] TypeScript

## Database

- [x] SQLite

## Search

- [x] SQLite FTS5 (`fts_ayah` / `fts_translation` tables + `search_arabic`
      command; no search UI yet — see Phase 7)

## Styling

- [ ] TailwindCSS (optional — skipped for now, plain CSS + variables kept install size down)
- [x] CSS Variables for themes

## Icons

- [x] Lucide

---

# Overall Development Phases

- [x] Phase 1 — Research
- [ ] Phase 2 — Project Setup (CI/CD, README and licensing done; CONTRIBUTING
      and screenshots left)
- [x] Phase 3 — Database Design
- [x] Phase 4 — Import Quran Data
- [x] Phase 5 — Reader MVP
- [x] Phase 6 — Navigation
- [ ] Phase 7 — Search (backend done, no UI)
- [ ] Phase 8 — Bookmarks (toggle done, no list UI)
- [ ] Phase 9 — Settings
- [ ] Phase 10 — Translations (schema groundwork landed with Phase 11; no
      translation imported yet)
- [ ] Phase 11 — Tafsir (English done — al-Jalalayn bundled, drawer in both
      views; Arabic editions and download packs left)
- [ ] Phase 12 — Audio
- [ ] Phase 13 — Release
- [x] Phase 14 — Reading Experience Enhancements

---

# Phase 1 — Research

## Quran Structure

Learn and understand

- [x] Surah
- [x] Ayah
- [x] Juz
- [x] Hizb
- [x] Rub' al-Hizb
- [x] Page (Madinah Mushaf)
- [x] Sajdah
- [x] Revelation Type (Makki / Madani)
- [x] Bismillah rules
- [x] Verse numbering
- [x] Uthmani Script

---

## Fonts

Research

- [x] KFGQPC Uthmanic Script
- [x] Amiri Quran
- [x] Hafs Smart
- [x] Noto Naskh Arabic

Decide

- [x] Default font
- [x] Fallback font

---

## Quran Dataset

Research

- [x] Quran text
- [x] Metadata
- [x] License

Choose one primary source.

---

## Mushaf Page Layout (pixel-accurate page rendering)

Goal: render each of the 604 Mushaf pages with the same line breaks and word
placement as the printed Madani Mushaf, instead of a reflowed list of Ayahs.

Research

- [x] Line-layout data source — zonetecde/mushaf-layout (ISC license), 604
      page JSON files: surah-header / basmala / text lines, word-level QPC
      v1 + v2 glyph strings, verse ranges
- [x] Glyph font generation — chose **QCF v2** (King Fahd Complex, Uthman
      Taha calligraphy, 604 files) over the newer QCF4 (47 files): official
      provenance and years of production use in Quran.com/Tarteel outweighed
      QCF4's smaller file count and unclear release status
- [x] Font source — the initially-vendored build from nuqayah/qpc-fonts
      (`mushaf-woff2/`) turned out to be dead on arrival on Linux/Windows: its
      word glyphs are blank shells on `cmap`, only reachable via an Apple
      AAT `morx` table that HarfBuzz-based engines (Chromium, WebKitGTK)
      don't process. Fixed by switching the per-page fonts to
      `verses.quran.foundation`'s build (Quran Foundation's official CDN,
      same King Fahd Complex/Uthman Taha artwork, properly GSUB-converted —
      confirmed with fontTools and a live render). Same `qpcV2` codepoints as
      the already-imported zonetecde layout data, so no importer changes were
      needed — just re-vendored the 604 page fonts
      (`scripts/vendor-mushaf-fonts.sh`). The Basmala font (`QCF_BSML`, still
      from nuqayah/qpc-fonts) already worked correctly and was left as-is.
      Fonts are usage-restricted (KFGQPC-owned, for Quranic rendering;
      distribution permitted, selling and modification not — see
      `THIRD-PARTY-NOTICES.md`, and the verification task under Phase 2).
      Quran Foundation's docs recommend
      loading fonts live from their CDN rather than vendoring; done anyway
      to keep this app offline-first, consistent with how the rest of its
      data is bundled.
- [x] Data quality check — the source layout data's own `surah-header` lines
      are unreliable (17 Surahs missing one, 13 with a stray duplicate from a
      page-boundary bug); the importer discards them and synthesizes all 114
      headers from our own validated Surah/Ayah data instead. Two Surahs
      (81, 85) are missing basmala glyphs in the source with no safe
      substitute (glyph codepoints are specific to each page's own font) —
      importer logs this gap rather than guessing.

Decide

- [x] Schema — `page_line` / `page_line_word` tables (database/schema.sql,
      migration 002)
- [x] Vendoring — fonts committed under `static/fonts/mushaf/` via
      `scripts/vendor-mushaf-fonts.sh` (~95MB, 605 files); layout JSON fetched
      live by the importer (like the existing Tanzil/alquran.cloud sources),
      not vendored
- [x] Rendering — per-page `@font-face` loaded lazily via the CSS Font
      Loading API with a small LRU (`$lib/utils/mushaf-fonts.ts`), not 604
      eager font-face declarations
- [x] UI — `PageView.svelte`, toggled against the scrolling `ReaderView` from
      the toolbar (`uiStore.readingMode`)
- [ ] Known simplification — line justification uses flexbox
      `space-between`, not true kashida stretching (browsers don't support
      the OpenType `jstf` justification tables these fonts were authored
      for); revisit if it reads as too sparse on short lines

---

# Phase 2 — Project Setup

## Repository

- [x] Create GitHub repository — `ask-786/quran-reader`
- [x] Choose license — MIT for the source code (`LICENSE`), with an explicit
      carve-out: bundled fonts and Quran data are third-party and not covered.
      `THIRD-PARTY-NOTICES.md` records the terms for each, and
      `licenses/OFL-1.1.txt` satisfies the OFL redistribution requirement for
      Scheherazade New and Inter (previously shipped with no license text)
- [x] Configure README — features, install, build-from-source, data sources
      and the licensing carve-out
- [ ] Add screenshots to the README — nothing visual in it yet
- [ ] Configure CONTRIBUTING
- [ ] Verify the QCF font terms against the primary source —
      <http://dm.qurancomplex.gov.sa/copyright-2/> was unreachable
      (connection refused) when the notices were written, so
      `THIRD-PARTY-NOTICES.md` currently relies on secondary descriptions:
      use/copy/distribute permitted, selling and modification prohibited.
      Note this is _less_ restrictive than this plan previously assumed —
      bundling them in release installers appears to be contemplated, and
      the open question is narrower: whether the woff2/`GSUB`-converted
      builds count as prohibited "modification", and whether the
      non-commercial reading some downstream packages assert is real.
      Neither blocks MIT-licensing the source code — licenses attach per
      work, and the `LICENSE` carve-out already keeps the MIT grant from
      reaching assets this project doesn't own

---

## Create Tauri App

- [x] Create project
- [x] Setup SvelteKit
- [x] Setup Rust
- [x] Setup TypeScript
- [x] Configure linting
- [x] Configure formatting

---

## Development

- [x] Git hooks — husky: `lint-staged` on pre-commit, `commitlint`
      (conventional commits) on commit-msg
- [x] CI — `.github/workflows/ci.yml`, on push/PR to `master`:
  - `frontend` — prettier `--check`, eslint, `svelte-check`, `vite build`
  - `rust` — `cargo fmt --check` + `clippy -D warnings` for both the app and
    the importer crate (the importer isn't in a workspace with the app, so
    each is invoked by `--manifest-path`)
  - `bundle` — compiles the real Tauri app on Linux/Windows/macOS with
    `--no-bundle`, so a platform-specific break is caught before tagging
- [x] Release workflow — `.github/workflows/release.yml`, on a `v*` tag (or
      manual dispatch): `tauri-apps/tauri-action` matrix building Linux
      (deb/rpm/AppImage), Windows (msi/nsis) and a universal macOS binary,
      published as a **draft** GitHub release
- [x] Dependabot — monthly npm / cargo (×2) / actions updates
- [ ] Code signing — macOS artifacts are unsigned (Gatekeeper warning) and
      Windows installers unsigned (SmartScreen warning); needs an Apple
      Developer ID + a Windows certificate
- [ ] Updater — no `tauri-plugin-updater`; releases are manual downloads

---

# Phase 3 — Database Design

## Main Tables

- [x] Surah
- [x] Ayah
- [x] Translation
- [x] Translation Ayah
- [x] Tafsir
- [x] Tafsir Ayah
- [x] Bookmark
- [x] Notes
- [x] Settings

---

## Surah

Fields

- [x] id
- [x] name_ar
- [x] name_en
- [x] transliteration
- [x] revelation_type
- [x] verses_count
- [x] order_of_revelation
- [x] has_bismillah

---

## Ayah

Fields

- [x] id
- [x] surah_id
- [x] ayah_number
- [x] uthmani_text
- [x] simple_text
- [x] juz
- [x] hizb
- [x] rub_hizb
- [x] page
- [x] sajdah
- [x] ruku
- [x] manzil

---

## Translation

Fields

- [x] id
- [x] language
- [x] translator
- [x] version

---

## Translation Ayah

- [x] translation_id
- [x] ayah_id
- [x] text

---

## Tafsir

Fields

- [x] id
- [x] language
- [x] author
- [x] title

---

## Tafsir Ayah

- [x] tafsir_id
- [x] ayah_id
- [x] text

---

## Bookmarks

- [x] id
- [x] ayah_id
- [x] created_at

---

## Notes

- [x] id
- [x] ayah_id
- [x] content
- [x] created_at

---

## Settings

- [x] Theme
- [x] Font
- [x] Font Size
- [x] Line Height
- [x] Last Read
- [x] Preferred Translation

---

# Phase 4 — Import Pipeline

## Import Tool

Create separate Rust importer

- [x] Read Quran source
- [x] Convert
- [x] Validate
- [x] Export SQLite

---

## Validation

- [x] 114 Surahs
- [x] 6236 Ayahs
- [x] All Pages
- [x] All Juz
- [x] All Hizb

---

# Phase 5 — Reader MVP

## Layout

- [x] Sidebar
- [x] Reader
- [x] Toolbar

---

## Sidebar

- [x] Surah List
- [x] Search (filter surah list by name/number; full Arabic/translation search is Phase 7)
- [ ] Settings (only a theme toggle exists in the toolbar; full settings panel is Phase 9)

---

## Reader

- [x] Load Surah
- [x] Render Ayahs
- [x] Scroll smoothly
- [x] Responsive layout

---

## Typography

- [x] Proper font
- [x] Line spacing
- [x] Letter spacing
- [x] Margins
- [x] RTL support
- [x] Text selection

---

## Reader Information

Display

- [x] Surah Name
- [x] Surah Number
- [x] Revelation Type
- [x] Verse Count

Display for every ayah

- [x] Ayah Number (per-ayah marker)
- [x] Page (boundary divider on change, not repeated per ayah — avoids clutter)
- [x] Juz (boundary divider on change, alongside Page)
- [ ] Hizb (not surfaced yet)

---

## Mushaf Page View (alternate to the scrolling reader)

See "Mushaf Page Layout" under Phase 1 for data sourcing.

- [x] `PageView.svelte` — line-by-line page rendering with QCF v2 glyphs
- [x] Per-page font lazy-loading with LRU eviction
- [x] Toggle between scrolling Reader and Mushaf Page view (toolbar)
- [x] Prev/next page + jump-to-page within the page view
- [ ] Word-level tap actions (bookmark/note/copy) — only available in the
      scrolling Reader for now
- [x] Deep-linking to a specific page (`/page/[id]` route) — see Phase 6
      quick navigation

---

# Phase 6 — Navigation

Navigate

- [x] Surah
- [x] Ayah
- [x] Juz — sidebar tab lists all 30, browsable via `/juz/[id]`
- [x] Hizb — sidebar tab lists all 60, browsable via `/hizb/[id]`
- [x] Page — deep-linkable via `/page/[id]` (1–604), opens in Mushaf view

---

## Quick Navigation

- [x] Go to Surah — `S:N` in the overlay, or the sidebar Surah tab (which
      filters by name/number); no name-fuzzy-match in the overlay itself
- [x] Go to Ayah — keyboard-triggered overlay (Ctrl/Cmd+G), accepts `N`
      (ayah in current surah) or `S:N` (surah:ayah); deep-links via
      `/surah/[id]?ayah=N`
- [x] Go to Page — same overlay, `p255` jumps to `/page/[id]` in Mushaf view
- [ ] Go to Juz — browsable from the sidebar Juz tab, no overlay prefix yet
- [ ] Go to Hizb — browsable from the sidebar Hizb tab, no overlay prefix yet

---

# Phase 7 — Search

> Backend is in place — FTS5 tables are populated by the importer and the
> `search_arabic` command returns ranked results with `<mark>` snippets.
> What's missing is the UI to drive it.

Search

- [ ] Arabic — query works end to end from the Rust side, no frontend yet
- [ ] Simple Arabic — `simple_text` is indexed; needs diacritic-insensitive
      query normalisation before it's usable
- [x] Surah — sidebar filters the Surah list by name/number
- [ ] Translation — `fts_translation` table exists but no translation data
      imported yet (Phase 10)

---

## Search Features

- [ ] Highlight results — `snippet()` already emits `<mark>` spans
- [ ] Jump to ayah — the `/surah/[id]?ayah=N` deep link it would target
      already works
- [ ] Search history

---

# Phase 8 — Bookmarks

- [x] Bookmark ayah — per-ayah toggle in the scrolling reader (`AyahRow`)
- [x] Remove bookmark — same toggle
- [ ] Bookmark list — no panel to browse/jump to saved bookmarks
- [ ] Bookmark from the Mushaf page view (word-level actions, see Phase 5)
- [ ] Notes UI — `upsert_note` / `delete_note` / `get_notes_for_ayah` exist on
      the Rust side with no frontend

---

## Reading History

- [x] Last Read — `last_read_surah_id` / `last_read_ayah_id` persisted to the
      settings table as you read
- [x] Continue Reading — app launch redirects `/` to the last-read Surah and
      restores the ayah position

---

# Phase 9 — Settings

> No settings panel exists yet. Everything below is stored in the settings
> table and applied on launch; only the theme and the zoom levels are
> reachable from the UI.

Appearance

- [x] Light
- [x] Dark
- [x] Sepia — all three cycle from the toolbar theme button, persisted
- [ ] Settings panel to pick a theme directly rather than cycling

Typography

- [ ] Font — the `font` setting is dead: it still defaults to `amiri-quran`,
      a font no longer bundled, and `applyTypography()` never reads it. The
      reader's Arabic is per-page QCF glyphs, which can't be swapped for
      another family at all — so this is either a picker over the
      live-shaped-text font only, or the setting should be dropped
- [ ] Font Size — persisted and applied, no control (reader zoom scales it)
- [ ] Line Height — persisted and applied, no control
- [ ] Reader Width — persisted and applied, no control

---

# Phase 10 — Translation

> Editions, sources, licensing and the full implementation breakdown live in
> `docs/translation-tafsir-plan.md`. Summary of what was decided:
>
> - Editions are chosen against a **Shāfiʿī in fiqh, Ashʿarī in creed** filter,
>   and every edition carries `school`/`creed` metadata so the picker can say
>   whose reading it is. Editions outside that filter stay available, always
>   labelled, never a default.
> - Bundled means embedded in every installer, so only public-domain or
>   plainly-redistributable editions get bundled; everything else is a
>   downloadable content pack. Nothing phones home on its own.

Support

- [ ] Multiple translations — `translation_ids` (ordered JSON array) replaces
      `preferred_translation_id`, which migrates on load
- [ ] Enable/Disable
- [ ] Parallel translations — `ReaderView`/`AyahRow` already have the
      single-translation prop plumbed and unfilled; widen it to N editions,
      each with its own text direction

Languages

- [ ] English — **Pickthall bundled** (public domain, traditional Sunni,
      creed-neutral). The Clear Qur'an (Khattab) reads better but is
      © Book of Signs Foundation, so it ships as a download, not in the
      installer. Hilālī-Khān and Mawdūdī are excluded from defaults
- [ ] Malayalam — no Sunni/Samastha edition exists in open data. The Ahlus-Sunna
      work (_Fatḥ al-Raḥmān_, K.V. Muhammed Musliyar, 1970) is in print only;
      the three digitised editions are two Mujāhid and one Jamāʿat-e-Islāmī.
      Ship Cheriyamundam **labelled**, and build a documented custom-edition
      import path so the Sunni text can be added if it can be obtained
- [ ] Arabic — better served by tafsir than by an Arabic "translation"; the
      slot stays empty deliberately

Groundwork

- [x] Migration 006 — `slug`/`name_native`/`direction`/`school`/`creed`/
      `source_url`/`license`/`sort_order` on `translation` and `tafsir`
      (landed with Phase 11; the translation half is unused until this phase)
- [x] `fts_translation` had **no sync triggers** (declared in `schema.sql` in
      001, never kept up to date, so silently stale since) — added in 006
      alongside a new `fts_tafsir`, so Phase 7's search has something real to
      query
- [ ] `--font-malayalam` (Noto Sans Malayalam) — Malayalam has no reliable
      system fallback on Linux and renders as boxes without it

---

# Phase 11 — Tafsir

> English is done and in the app: Tafsīr al-Jalālayn (Hamza) ships in the seed
> database, with a drawer that follows the verse you are reading in both views.
> See `docs/translation-tafsir-plan.md` for how the editions were chosen.

Support

- [x] Multiple Tafsir — schema, importer registry and picker all take N
      editions; one is installed today
- [x] Expandable panel — a right-side resizable **drawer** that follows the
      current Ayah, not an inline accordion: it doesn't perturb the reserved-
      height windowing in `ReaderView`, and it works in Mushaf `PageView` too.
      Toolbar toggle, `t`, and a per-Ayah button in the scrolling reader that
      pins one verse until the reader scrolls off it
- [ ] Word-level trigger in Mushaf `PageView` — the panel follows the page's
      centred Ayah there, but tapping a word still does nothing (the Phase 5
      word-action gap is still open)
- [x] Switch Tafsir — edition picker in the panel header, shown once more than
      one edition is installed; each labelled with its school and creed
- [ ] Content packs — standalone `.qrpack` SQLite files published as release
      assets, verified by sha256 on install, `is_bundled` guarding uninstall.
      Note the constraint recorded in `copy_bundled_tafsir_from_seed`: pack
      installs must allocate `tafsir.id` outside the seed's range, or the
      positional copy on upgrade collides with a user's edition

What the app will carry

> Only works of the **Shāfiʿī school and Ashʿarī creed**. Not "everything,
> labelled" — a commentary's madhhab decides how it reads the āyāt al-aḥkām
> and its creed decides how it reads the attribute verses, and neither is
> visible in the text, so an edition outside that is not offered at all. The
> school/creed labels exist to tell the reader what they are opening, not to
> caveat something that shouldn't be there. The list lives in `EDITIONS`
> (`importer/src/tafsir.rs`).

- [x] **Al-Jalalayn (English, tr. Feras Hamza) — bundled default.** Al-Maḥallī
      and al-Suyūṭī were both Egyptian Shāfiʿīs; it entered the Kerala dars
      syllabus through the Ponnani school and has been taught there since; and
      at 2.13 MB of text it is the only serious classical tafsir small enough
      to bundle. Of the two English editions in the source, this is
      `tafsir-al-jalalayn` (punctuation and brackets intact), not
      `en-al-jalalayn` (punctuation stripped). Seed DB: 10.5 → 14.4 MB
- [ ] **Al-Jalalayn (Arabic)** — deferred with the rest of the Arabic side:
      nothing bundled is fit for running Arabic prose yet (see the note in
      `src/app.css` — needs `--font-arabic-prose`). Note for whoever lands it:
      the Arabic edition has **6,010 entries, not 6,236** — 226 āyāt genuinely
      carry no gloss, which is why the importer validates a coverage floor
      rather than a complete set, and why the panel has a
      "no separate commentary on this verse" state already built
- [ ] **Shāfiʿī shelf, download-only** (all too large to bundle):
      al-Baghawī ≈41 MB, al-Bayḍāwī ≈11 MB, al-Māwardī ≈14 MB,
      al-Wāḥidī ≈8 MB, al-Qushayrī ≈9 MB, al-Rāzī ≈90 MB,
      al-Suyūṭī's al-Durr al-Manthūr ≈60 MB
- [ ] **Ibn Kathir — open question.** Shāfiʿī in fiqh, so he passes the
      madhhab half, but Atharī in creed, and the common English abridgement is
      Darussalam-edited. Left out under the rule above; carrying him is a
      judgement call, not an oversight
- [ ] Verify the English Jalalayn's redistribution terms before the next
      release — © 2007 Royal Aal al-Bayt Institute, free to read on
      altafsir.com, terms unread (the site was unreachable). Same standard as
      the QCF fonts: bundling in signed installers is what needs the
      permission. See `THIRD-PARTY-NOTICES.md`

---

# Phase 12 — Audio

Features

- [ ] Play Surah
- [ ] Play Ayah
- [ ] Repeat Ayah
- [ ] Auto-scroll
- [ ] Offline audio

---

# Phase 13 — Polish

Performance

- [x] Virtualized rendering — the scrolling reader renders a window of rows
      around the viewport instead of the whole Surah
- [x] Lazy loading — per-page Mushaf fonts loaded on demand with an LRU
- [x] Cache — page data cached in `$lib/api/page-cache.ts`
- [ ] Startup profiling — the seed DB is embedded with `include_bytes!`, which
      makes the binary ~11MB larger; revisit if startup or install size bites

Accessibility

- [x] Keyboard shortcuts — Ctrl/Cmd+G go-to overlay, `f` focus mode, `m`
      switch reading mode, `n`/`p` next/prev, `Escape` to dismiss; guarded
      against firing while typing in an input
- [ ] Discoverable shortcut list — nothing in the UI documents the above
- [ ] Screen reader support
- [ ] High contrast

Packaging

> Automated by the release workflow (Phase 2). Proven by the `v0.1.0` tag —
> all seven bundles built green on the first run. None has been _launched_
> on Windows or macOS yet, only compiled and packaged.

- [x] Linux — deb / rpm / AppImage, built on Ubuntu 22.04 for a glibc 2.35
      floor
- [x] Windows — msi + nsis (unsigned)
- [x] macOS — universal (Apple Silicon + Intel) dmg (unsigned)
- [ ] Arch — no pacman target exists in Tauri's bundler; AppImage is the
      stopgap, an AUR `quran-reader-bin` PKGBUILD is the real answer
- [ ] Install size — v0.1.0 ships at ~100MB per installer (175MB AppImage,
      203MB dmg), against a stated core principle of "small install size".
      Structural: 95MB of QCF page fonts bundled into the frontend dist plus
      the 11MB embedded database. Levers are QCF4 (47 files, rejected in
      Phase 1 on provenance) or fetching fonts on first run (costs
      offline-first). Decide deliberately rather than let it drift

---

- [x] Phase 14 — Reading Experience Enhancements

## Zoom

- [x] App-wide zoom (scales entire UI, via Tauri's native webview zoom API)
- [x] Reader-only zoom (scales just the Quran text, on top of app zoom, via
      CSS `calc()` scaling of font-size/width — the non-standard `zoom`
      property was tried first but broke `overflow-y:auto` scrolling in
      WebKitGTK)
- [x] Both persist across app relaunch (settings table)

---

## Focus Mode

- [x] Toggle hides sidebar + toolbar, leaving only the reader
- [x] Reader zoom control remains reachable (top-right corner overlay,
      alongside an exit button; `Escape` also exits)
- [ ] Known tradeoff — window drag/minimize/maximize/close live in the
      custom toolbar-as-titlebar, so they're unreachable until focus mode
      is exited

---

## Auto-scroll

- [x] Available in both normal and focus reader views
- [x] Right-side vertical handle to adjust scroll speed (drag) and
      play/pause (click), no separate toolbar button

---

## Reading Progress Indicator

- [x] Left-edge vertical bar on the reader
- [x] Shows % progress through the current Surah (scroll-fraction based),
      with current Juz/Hizb on hover

---

# Future Features

## Notes

- [ ] Personal notes
- [ ] Highlights

---

## Memorization

- [ ] Hide words
- [ ] Hide ayahs
- [ ] Repeat mode

---

## Statistics

- [ ] Reading streak
- [ ] Progress
- [ ] Completed Juz

---

## Collections

- [ ] Favorite Ayahs
- [ ] Favorite Surahs

---

## Sharing

- [ ] Copy ayah
- [ ] Copy translation
- [ ] Export image

---

## Themes

- [ ] Mushaf Theme
- [ ] Sepia
- [ ] AMOLED

---

## Developer

- [ ] Plugin architecture
- [ ] Database migrations
- [ ] Localization
- [ ] Unit tests
- [ ] Integration tests

---

# Suggested Project Structure

```
quran-reader/
│
├── .github/
│   ├── workflows/         (ci.yml, release.yml)
│   └── dependabot.yml
│
├── src/
│   ├── lib/
│   │   ├── api/
│   │   ├── components/
│   │   │   ├── reader/
│   │   │   ├── sidebar/
│   │   │   ├── search/
│   │   │   ├── bookmarks/
│   │   │   ├── settings/
│   │   │   └── common/
│   │   ├── stores/
│   │   ├── models/
│   │   ├── services/
│   │   ├── themes/
│   │   ├── fonts/
│   │   ├── utils/
│   │   └── types/
│   │
│   └── routes/
│
├── src-tauri/
│   ├── src/
│   │   ├── commands/
│   │   ├── db/
│   │   ├── models/
│   │   ├── services/
│   │   ├── importer/
│   │   └── migrations/
│   │
│   └── assets/
│
├── database/
│   ├── schema.sql
│   ├── migrations/
│   └── quran.db
│
├── importer/
│
├── static/
│   └── fonts/
│       └── mushaf-v4/     (vendored QCF v4 font groups — see scripts/)
│
├── scripts/
│   └── vendor-mushaf-fonts-v4.sh
│
├── docs/
│
└── README.md
```

---

# MVP Definition

The application is considered **v1.0** when all of the following are complete:

- [x] Offline — Quran DB embedded in the binary, all fonts vendored, no
      network calls at runtime
- [x] Beautiful Arabic rendering — QCF v2 page glyphs in _both_ the Mushaf
      page view and the scrolling reader; Scheherazade New only for
      live-shaped text (Surah names, ۞ ornament, Bismillah fallback)
- [x] Fast startup
- [x] Browse all 114 Surahs
- [x] Navigate by Surah
- [x] Navigate by Juz
- [x] Navigate by Hizb
- [x] Navigate by Page
- [ ] Search Arabic — backend only (Phase 7)
- [ ] Bookmarks — toggle works, no bookmark list (Phase 8)
- [x] Remember last read
- [ ] Adjustable typography — reader zoom only; no font/size/line-height
      controls (Phase 9)
- [x] Dark mode
- [ ] Windows support — release workflow builds it, not yet verified on a
      real machine
- [x] Linux support
- [ ] macOS support — release workflow builds it, not yet verified on a real
      machine

---

# Guiding Principle

> **Every design decision should improve the reading experience.**
>
> Features are valuable only if they help users read, understand, and navigate the Quran more comfortably. Performance, typography, simplicity, and offline reliability should always take precedence over adding more functionality.
