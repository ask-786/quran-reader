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

- [ ] Tauri v2

## Backend

- [ ] Rust

## Frontend

- [ ] SvelteKit
- [ ] TypeScript

## Database

- [ ] SQLite

## Search

- [ ] SQLite FTS5

## Styling

- [ ] TailwindCSS (optional — skipped for now, plain CSS + variables kept install size down)
- [x] CSS Variables for themes

## Icons

- [x] Lucide

---

# Overall Development Phases

- [x] Phase 1 — Research
- [ ] Phase 2 — Project Setup
- [x] Phase 3 — Database Design
- [x] Phase 4 — Import Quran Data
- [x] Phase 5 — Reader MVP
- [ ] Phase 6 — Navigation
- [ ] Phase 7 — Search
- [ ] Phase 8 — Bookmarks
- [ ] Phase 9 — Settings
- [ ] Phase 10 — Translations
- [ ] Phase 11 — Tafsir
- [ ] Phase 12 — Audio
- [ ] Phase 13 — Release

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
      Fonts are usage-restricted ("Quranic rendering purposes", no commercial
      redistribution) — standard across the whole Quran-app ecosystem, not a
      redistributable-as-a-font license. Quran Foundation's docs recommend
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
      `scripts/vendor-mushaf-fonts.sh` (~48MB, 605 files); layout JSON fetched
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

- [ ] Create GitHub repository
- [ ] Choose license
- [ ] Configure README
- [ ] Configure CONTRIBUTING

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

- [x] Git hooks
- [ ] CI
- [ ] Release workflow

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
- [ ] Deep-linking to a specific page (`/page/[n]` route) — folded into
      Phase 6 quick navigation instead of duplicated here

---

# Phase 6 — Navigation

Navigate

- [ ] Surah
- [ ] Ayah
- [ ] Juz
- [ ] Hizb
- [ ] Page

---

## Quick Navigation

- [ ] Go to Surah
- [ ] Go to Ayah
- [ ] Go to Page
- [ ] Go to Juz
- [ ] Go to Hizb

---

# Phase 7 — Search

Search

- [ ] Arabic
- [ ] Simple Arabic
- [ ] Surah
- [ ] Translation

---

## Search Features

- [ ] Highlight results
- [ ] Jump to ayah
- [ ] Search history

---

# Phase 8 — Bookmarks

- [ ] Bookmark ayah
- [ ] Remove bookmark
- [ ] Bookmark list

---

## Reading History

- [ ] Last Read
- [ ] Continue Reading

---

# Phase 9 — Settings

Appearance

- [ ] Light
- [ ] Dark
- [ ] Sepia

Typography

- [ ] Font
- [ ] Font Size
- [ ] Line Height
- [ ] Reader Width

---

# Phase 10 — Translation

Support

- [ ] Multiple translations
- [ ] Enable/Disable
- [ ] Parallel translations

Languages

- [ ] English
- [ ] Malayalam
- [ ] Arabic

---

# Phase 11 — Tafsir

Support

- [ ] Multiple Tafsir
- [ ] Expandable panel
- [ ] Switch Tafsir

Possible Tafsir

- [ ] Ibn Kathir
- [ ] As-Sa'di
- [ ] Al-Jalalayn

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

- [ ] Virtualized rendering
- [ ] Lazy loading
- [ ] Cache

Accessibility

- [ ] Keyboard shortcuts
- [ ] Screen reader support
- [ ] High contrast

Packaging

- [ ] Linux
- [ ] Windows
- [ ] macOS

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
│       └── mushaf/        (vendored QCF v2 page fonts — see scripts/)
│
├── scripts/
│   └── vendor-mushaf-fonts.sh
│
├── docs/
│
└── README.md
```

---

# MVP Definition

The application is considered **v1.0** when all of the following are complete:

- [ ] Offline
- [ ] Beautiful Arabic rendering
- [ ] Fast startup
- [ ] Browse all 114 Surahs
- [ ] Navigate by Surah
- [ ] Navigate by Juz
- [ ] Navigate by Hizb
- [ ] Navigate by Page
- [ ] Search Arabic
- [ ] Bookmarks
- [ ] Remember last read
- [ ] Adjustable typography
- [ ] Dark mode
- [ ] Windows support
- [ ] Linux support
- [ ] macOS support

---

# Guiding Principle

> **Every design decision should improve the reading experience.**
>
> Features are valuable only if they help users read, understand, and navigate the Quran more comfortably. Performance, typography, simplicity, and offline reliability should always take precedence over adding more functionality.
