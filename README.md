# Quran Reader

An offline-first Quran reader for desktop, built with [Tauri 2](https://tauri.app),
SvelteKit and Rust. No account, no network calls, no telemetry — the Quran
data ships inside the binary.

> **Status: early.** Reading, navigation, the Mushaf page view and tafsir
> (Arabic + English) work and are used daily. Search, translations and audio
> are not built yet. See [PLAN.md](PLAN.md) for what exists and what doesn't.

---

## Features

**Two ways to read**

- **Mushaf page view** — renders all 604 pages of the Madani Mushaf with the
  same line breaks and word placement as the printed original, using the
  official KFGQPC (Uthman Taha) glyph fonts
- **Scrolling reader** — continuous Surah reading, per-ayah bookmark, copy
  and tafsir actions, page/juz boundary markers

**Tafsir**

- **Tafsīr al-Jalālayn** is bundled in both the **Arabic original** and
  **English** (tr. Feras Hamza) — a side panel that follows the verse you are
  reading, in both views (`t`), with a picker to switch between them
- Editions are Shāfiʿī in fiqh and Ashʿarī in creed, and each is labelled with
  both. A commentary's school decides how it reads the legal verses and its
  creed decides how it reads the attribute verses; neither is visible in the
  text, so the reader is told rather than left to guess

**Navigation**

- Browse by Surah, Juz, Hizb or page
- Go-to overlay (`Ctrl`/`Cmd`+`G`): `255`, `2:255`, or `p255`
- Keyboard-driven sidebar search (`/`): type, arrow through the results, `Enter`
- Deep links: `/surah/2?ayah=255`, `/juz/30`, `/hizb/59`, `/page/604`
- Reopens where you left off — each Surah, Juz, Hizb and page keeps its own
  place, and launching goes back to whichever you were last reading
- A Recent tab listing your reading sittings, each with the verses it covered

**Reading**

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

| Key                                                           | Action                                                                    |
| ------------------------------------------------------------- | ------------------------------------------------------------------------- |
| `Ctrl`/`Cmd`+`K` or `Ctrl`/`Cmd`+`G`                          | Open the navigation palette                                               |
| `/` or `Ctrl`/`Cmd`+`F`                                       | Focus the sidebar's filter (same lists, docked instead of floating)       |
| `↑` / `↓`, `PgUp` / `PgDn` _(in either)_                      | Move the highlight through the list                                       |
| `Tab` / `Shift`+`Tab` _(palette)_, `Alt`+`←` / `→` _(either)_ | Switch list: Surah → Juz → Hizb → Recent                                  |
| `Enter` _(in either)_                                         | Open what's highlighted, or the `2:255` / `p255` you typed                |
| `f`                                                           | Toggle focus mode                                                         |
| `m`                                                           | Switch between Mushaf and scrolling view                                  |
| `n` / `p`                                                     | Next / previous surah, juz, hizb or page                                  |
| `↓` / `↑` (or `PgDn` / `PgUp`)                                | Next / previous Mushaf page                                               |
| `Home` / `End`                                                | Jump to the start / end of what's open                                    |
| `a` or `Space`                                                | Start or stop auto-scroll                                                 |
| `t`                                                           | Toggle tafsir mode — then click a verse for its commentary                |
| `Shift`+`↑` / `Shift`+`↓`                                     | Auto-scroll faster / slower                                               |
| `+` / `-` / `0`                                               | Reader zoom in / out / reset (normal and focus view keep separate levels) |
| `Ctrl`/`Cmd`+`+` / `-` / `0`                                  | App zoom in / out / reset                                                 |
| `Esc`                                                         | Clear the filter, then close the palette or leave the box / focus mode    |

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
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

CI runs all of the above plus a compile on Linux, Windows and macOS. Commits
are linted by husky (`lint-staged` + conventional commits via commitlint).

### Releasing

The version lives in four files that must agree — `package.json`,
`src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock`
— and the release workflow rechecks them against the tag before building:

```bash
./scripts/bump-version.sh patch --commit   # or minor / major / an explicit X.Y.Z
git push origin master && git push origin vX.Y.Z
```

The tag push builds Linux, macOS and Windows bundles into a **draft** release.
Publishing that draft is what triggers the AUR workflow, which updates
`packaging/aur/` with the new version and `.deb` checksum on its own.

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

**Tafsir** — no commentary ships with the app. Each edition is built into a
content pack, published as a release asset, and downloaded by the reader on
request. `--emit-pack` writes the file and prints the SHA-256 that goes into
`PACKS` in `src-tauri/src/packs`:

```bash
cd importer && cargo run --release -- --list-tafsir
cd importer && cargo run --release -- --emit-pack ar-tafsir-al-jalalayn   # → packs/*.qpack
```

There is deliberately no way to write a tafsir into `database/quran.db`: that
file is embedded in the binary with `include_bytes!`, so anything in it is paid
for by every user of every platform whether they want the edition or not.

**Mushaf fonts** — re-downloads the 47 QCF v4 font-group files (~36MB):

```bash
./scripts/vendor-mushaf-fonts-v4.sh
```

---

## Project structure

```
src/              SvelteKit frontend (components, stores, routes)
src-tauri/        Rust backend — SQLite access, Tauri commands
importer/         Standalone importer that builds quran.db
database/         schema.sql, migrations, and the built quran.db
static/fonts/     Vendored fonts (47 QCF v4 groups, Scheherazade New, Amiri)
scripts/          Font vendoring, icon generation, version bumping
docs/             Research and design notes
```

---

## Data sources

| Source                                                                        | Provides                              | License                                                                    |
| ----------------------------------------------------------------------------- | ------------------------------------- | -------------------------------------------------------------------------- |
| [Tanzil Project](https://tanzil.net)                                          | Uthmani and simple Arabic text        | CC BY 3.0                                                                  |
| [zonetecde/mushaf-layout](https://github.com/zonetecde/mushaf-layout)         | Mushaf page line layout               | ISC                                                                        |
| [alquran.cloud](https://alquran.cloud)                                        | Per-ayah juz/hizb/page metadata       | Open                                                                       |
| [spa5k/quran_data](https://github.com/spa5k/quran_data)                       | Surah metadata                        | Open                                                                       |
| [MohamadHajjRabee/quran-qcf4](https://github.com/MohamadHajjRabee/quran-qcf4) | QCF v4 Mushaf glyph fonts + layout    | JSON: MIT · fonts: Restricted (KFGQPC) — see below                         |
| [spa5k/tafsir_api](https://github.com/spa5k/tafsir_api)                       | Tafsīr al-Jalālayn (Arabic + English) | Arabic: public domain · English: © Royal Aal al-Bayt Institute — see below |
| [aliftype/amiri](https://github.com/aliftype/amiri)                           | Amiri (Arabic prose)                  | SIL OFL 1.1                                                                |

---

## License

The source code is [MIT licensed](LICENSE).

The bundled fonts and Quran data are not covered by the MIT license — they
are third-party works under their own terms. The QCF v4 Mushaf fonts in
particular may be used, copied and distributed, but not sold or modified.

See [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) before forking,
redistributing, or selling a build.
