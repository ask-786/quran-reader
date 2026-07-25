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

- [ ] TailwindCSS (optional)
- [ ] CSS Variables for themes

## Icons

- [ ] Lucide

---

# Overall Development Phases

- [x] Phase 1 — Research
- [ ] Phase 2 — Project Setup
- [x] Phase 3 — Database Design
- [ ] Phase 4 — Import Quran Data
- [ ] Phase 5 — Reader MVP
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

- [ ] Read Quran source
- [ ] Convert
- [ ] Validate
- [ ] Export SQLite

---

## Validation

- [ ] 114 Surahs
- [ ] 6236 Ayahs
- [ ] All Pages
- [ ] All Juz
- [ ] All Hizb

---

# Phase 5 — Reader MVP

## Layout

- [ ] Sidebar
- [ ] Reader
- [ ] Toolbar

---

## Sidebar

- [ ] Surah List
- [ ] Search
- [ ] Settings

---

## Reader

- [ ] Load Surah
- [ ] Render Ayahs
- [ ] Scroll smoothly
- [ ] Responsive layout

---

## Typography

- [ ] Proper font
- [ ] Line spacing
- [ ] Letter spacing
- [ ] Margins
- [ ] RTL support
- [ ] Text selection

---

## Reader Information

Display

- [ ] Surah Name
- [ ] Surah Number
- [ ] Revelation Type
- [ ] Verse Count

Display for every ayah

- [ ] Ayah Number
- [ ] Page
- [ ] Juz
- [ ] Hizb

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
