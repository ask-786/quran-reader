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

---

## Usage

```bash
# From the importer/ directory:
cargo run --release
```

The database is written to `../database/quran.db` (relative to the workspace root).

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
