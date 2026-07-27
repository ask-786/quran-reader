# Quran Reader

A beautiful, fast, offline-first Quran reader for the desktop, built around
one goal: **the reading experience.**

Built with [Tauri 2](https://tauri.app), SvelteKit and Rust. No account, no
network calls, no telemetry — the entire Quran ships inside the binary.

> **Status: early.** Reading, navigation and the Mushaf page view are solid
> and in daily use. Search, translations, tafsir and audio are not built yet.
> See [PLAN.md](PLAN.md) for what exists and what doesn't.

---

## The reading experience is the point

This is not a Quran app that happens to display text. **Reading is the
primary goal, and every other consideration loses to it.**

That principle is not decorative — it is why the app is built the way it is:

- **The page is the printed page.** Rather than reflowing verses into a list,
  the Mushaf view reproduces all 604 pages of the Madani Mushaf line for line
  and word for word, using the official KFGQPC glyph fonts. A hafiz who has
  memorised by page position sees the same shapes in the same places.
- **Typography got the hard hours.** The bundled font was replaced outright
  when its glyphs turned out to render blank on Linux, and again when a
  dagger alef detached and floated as a stray mark on four Surah names.
  Every one of the 114 headers was rendered and checked by eye.
- **The interface gets out of the way.** Focus mode strips the app down to
  the text alone. Auto-scroll lets you read hands-free. Page and Juz markers
  appear at boundaries instead of repeating on every verse.
- **Nothing interrupts.** No notifications, no streaks, no nags, no network.
  The app opens where you stopped reading and gets on with it.
- **Speed is a reading feature.** A reader that stutters while you read is a
  reader you stop using — so fonts load lazily and the scrolling view renders
  only what's near the viewport.

Features are only worth adding if they help you read, understand and navigate
the Quran more comfortably. When something improves the reading experience at
the cost of a longer feature list, the reading experience wins.

---

## Features

**Two ways to read**

- **Mushaf page view** — renders all 604 pages of the Madani Mushaf with the
  same line breaks and word placement as the printed original, using the
  official KFGQPC (Uthman Taha) glyph fonts
- **Scrolling reader** — continuous Surah reading, per-ayah bookmark and
  copy actions, page/juz boundary markers

**Navigation**

- Browse by Surah, Juz, Hizb or page
- Go-to overlay (`Ctrl`/`Cmd`+`G`): `255`, `2:255`, or `p255`
- Deep links: `/surah/2?ayah=255`, `/juz/30`, `/hizb/59`, `/page/604`
- Reopens where you left off

**Reading experience**

- Focus mode — hides everything but the text
- Auto-scroll with a drag-to-adjust speed handle
- Independent app zoom and reader zoom, both persisted
- Reading-progress bar showing position in the Surah, with Juz/Hizb on hover
- Dark, light and sepia themes

**Offline and small**

- The Quran database is compiled into the binary — first launch works with
  no network, no import step, no download
- Mushaf page fonts load lazily behind an LRU, so opening Al-Baqara doesn't
  pull 48 font files up front

### Keyboard shortcuts

| Key              | Action                                   |
| ---------------- | ---------------------------------------- |
| `Ctrl`/`Cmd`+`G` | Go to ayah or page                       |
| `f`              | Toggle focus mode                        |
| `m`              | Switch between Mushaf and scrolling view |
| `n` / `p`        | Next / previous surah or page            |
| `Esc`            | Close the overlay, or leave focus mode   |

---

## Install

Download the installer for your platform from the
[Releases](https://github.com/ask-786/quran-reader/releases) page.

| Platform | Format                                       |
| -------- | -------------------------------------------- |
| Linux    | `.deb`, `.rpm`, `.AppImage` (glibc 2.35+)    |
| Windows  | `.msi`, `.exe` (NSIS)                        |
| macOS    | `.dmg` (universal — Apple Silicon and Intel) |

Builds are **not code-signed**. macOS will show a Gatekeeper warning and
Windows a SmartScreen warning until signing certificates are in place.

---

## Build from source

### Prerequisites

- [Rust](https://rustup.rs) (stable)
- Node.js 22+ and [pnpm](https://pnpm.io)
- Tauri's platform dependencies — see the
  [Tauri prerequisites guide](https://tauri.app/start/prerequisites/)

On Debian/Ubuntu:

```bash
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
                 libgtk-3-dev libxdo-dev libssl-dev patchelf
```

### Run

```bash
pnpm install
pnpm tauri dev
```

### Build

```bash
pnpm tauri build               # installers for the current platform
pnpm tauri build --no-bundle   # just compile, skip packaging
```

### Checks

```bash
pnpm format:check && pnpm lint && pnpm check   # frontend
cargo fmt --all --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

CI runs all of the above plus a compile on Linux, Windows and macOS. Commits
are linted by husky (`lint-staged` + conventional commits via commitlint).

---

## Regenerating the data

Neither step is needed for normal development — the database and fonts are
committed.

**Quran database** — fetches from Tanzil and alquran.cloud, validates
(114 Surahs, 6,236 Ayahs, all juz/hizb/pages), and writes
`database/quran.db`:

```bash
cd importer && cargo run --release
```

See [`importer/README.md`](importer/README.md) for details.

**Mushaf fonts** — re-downloads the 604 QCF v2 page fonts (~95MB):

```bash
./scripts/vendor-mushaf-fonts.sh
```

---

## Project structure

```
src/              SvelteKit frontend (components, stores, routes)
src-tauri/        Rust backend — SQLite access, Tauri commands
importer/         Standalone importer that builds quran.db
database/         schema.sql, migrations, and the built quran.db
static/fonts/     Vendored fonts (604 QCF page fonts + Scheherazade New)
scripts/          Font vendoring
docs/             Research and design notes
```

---

## Data sources

| Source                                                                                                       | Provides                        | License                |
| ------------------------------------------------------------------------------------------------------------ | ------------------------------- | ---------------------- |
| [Tanzil Project](https://tanzil.net)                                                                         | Uthmani and simple Arabic text  | CC BY 3.0              |
| [zonetecde/mushaf-layout](https://github.com/zonetecde/mushaf-layout)                                        | Mushaf page line layout         | ISC                    |
| [alquran.cloud](https://alquran.cloud)                                                                       | Per-ayah juz/hizb/page metadata | Open                   |
| [spa5k/quran_data](https://github.com/spa5k/quran_data)                                                      | Surah metadata                  | Open                   |
| [KFGQPC](http://dm.qurancomplex.gov.sa/copyright-2/) via [Quran Foundation](https://verses.quran.foundation) | QCF v2 Mushaf glyph fonts       | Restricted — see below |

---

## License

The source code is [MIT licensed](LICENSE).

**The bundled fonts and Quran data are not** — they are third-party works
under their own terms, and the MIT grant above does not extend to them. Most
notably, the QCF v2 Mushaf fonts are owned by the King Fahd Glorious Quran
Printing Complex and provided for rendering Quranic text: they may be used,
copied and distributed, but not sold or modified.

If you fork or redistribute this project — and especially if you intend to
sell a build — read [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) first.
Those terms apply to you regardless of the MIT license on the code.

The Quran text itself is, of course, not anyone's to license.
