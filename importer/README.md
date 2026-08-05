# Quran Importer

Standalone Rust binary that fetches Quran data from authoritative sources,
validates it, and writes a ready-to-ship `quran.db` SQLite database.

---

## What it does

| Step         | Action                                                                               |
| ------------ | ------------------------------------------------------------------------------------ |
| **Fetch**    | Downloads Tanzil Uthmani XML, Tanzil Simple Arabic XML, and surah metadata JSON      |
| **Parse**    | Fetches per-ayah metadata (juz, hizb, page, ruku, manzil, sajdah) from alquran.cloud |
| **Validate** | Verifies 114 Surahs, 6236 Ayahs, all Juz/Hizb/Pages present, sajdah count            |
| **Write**    | Applies the schema, inserts all data in a single transaction, rebuilds FTS5 index    |

---

## Data sources

| Source                                                  | Data                                                      | License   |
| ------------------------------------------------------- | --------------------------------------------------------- | --------- |
| [Tanzil Project](https://tanzil.net)                    | Uthmani Arabic text, Simple Arabic text                   | CC BY 3.0 |
| [alquran.cloud](https://alquran.cloud)                  | Per-ayah metadata (juz, hizb, page, ruku, manzil, sajdah) | Open      |
| [spa5k/quran_data](https://github.com/spa5k/quran_data) | Surah names, revelation types, verse counts               | Open      |
| [spa5k/tafsir_api](https://github.com/spa5k/tafsir_api) | Tafsir editions (per-Surah JSON)                          | Per text  |

---

## Usage

```bash
# From the importer/ directory:
cargo run --release
```

The database is written to `../database/quran.db` (relative to the workspace root).

---

## Tafsir

A separate pass that writes a commentary edition into an existing
`quran.db` — it does not rebuild the Quran data.

```bash
cargo run --release -- --list-tafsir
cargo run --release -- --import-tafsir tafsir-al-jalalayn --bundle

# Re-run from a local directory of 1.json … 114.json instead of 114 requests:
cargo run --release -- --import-tafsir tafsir-al-jalalayn --bundle --tafsir-dir ./cache
```

`--bundle` marks the edition `is_bundled`, which means it ships inside the
committed seed database and survives an in-place upgrade. Leave it off for an
edition that is only being written locally.

**Which editions exist is a deliberate list, not a passthrough.** See
`EDITIONS` in `src/tafsir.rs`: this reader carries commentary of the Shāfiʿī
school and Ashʿarī creed, because a tafsir's madhhab decides how it reads the
legal verses and its creed decides how it reads the attribute verses, and
neither is visible in the text. Adding a slug is a decision about the app, not
a data-sourcing convenience.

Two behaviours worth knowing before adding an edition:

- **Missing verses are normal.** Al-Jalalayn's Arabic passes over 226 of the
  6,236 Ayahs with no gloss. The importer logs coverage and refuses anything
  under 90%, rather than demanding a complete set.
- **Markup is stripped, not sanitised.** Entries are reduced to plain text, so
  the renderer never has to trust the database. An edition whose formatting
  carries meaning would need that decision revisited on both sides.

---

## Output

```
database/quran.db
```

- ~5–8 MB SQLite file
- 114 Surahs
- 6,236 Ayahs (Uthmani + Simple text)
- FTS5 index on simple Arabic text
- All reading divisions (Juz, Hizb, Rub al-Hizb, Manzil, Ruku, Page, Sajdah)

---

## Attribution

When releasing the app, the following must be included:

```
Quran text: Tanzil Project (tanzil.net) — CC BY 3.0
```

See `docs/research/phase1-research.md` for full attribution requirements.
