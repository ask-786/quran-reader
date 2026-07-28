# Third-Party Notices

Quran Reader's own source code is MIT licensed (see `LICENSE`). The Quran
text, the Mushaf layout data and every bundled font are third-party works
under their own terms. This file records what each one is, where it came
from, and what its terms allow.

If you fork this project, redistribute a build, or publish your own release,
these terms apply to you and not the MIT license.

---

## Fonts

### QCF v2 Mushaf page fonts — `static/fonts/mushaf/QCF_P001…P604.woff2`

|           |                                                                                   |
| --------- | --------------------------------------------------------------------------------- |
| Source    | [verses.quran.foundation](https://verses.quran.foundation) (Quran Foundation CDN) |
| Copyright | King Fahd Glorious Quran Printing Complex (KFGQPC) — Uthman Taha calligraphy      |
| Terms     | <http://dm.qurancomplex.gov.sa/copyright-2/>                                      |

**These fonts are not open source.** They are owned by KFGQPC and provided
for the purpose of rendering Quranic text. As described by secondary sources
(the primary copyright page was unreachable when this was written), the terms
grant permission to _use, copy and distribute_ the font software, while
prohibiting **selling, modifying, altering, translating, reverse-engineering
or decompiling** it. Some downstream packages additionally treat the fonts as
non-commercial-use only.

Two caveats worth knowing before you rely on the above:

1. It has not been verified against the primary source at
   <http://dm.qurancomplex.gov.sa/copyright-2/>. Check it yourself.
2. The woff2 builds distributed by Quran Foundation and by nuqayah/qpc-fonts
   are format-converted (and, in Quran Foundation's case, `GSUB`-rebuilt)
   versions of the originals. Whether that counts as prohibited
   "modification" is a grey area that the entire Quran-app ecosystem
   currently sits in. This project redistributes those existing builds
   unaltered; it does not convert or rebuild anything itself.

This repository vendors all 604 page fonts, and release builds bundle them,
so that the app works fully offline. Quran Foundation's own guidance is to
load these fonts live from their CDN rather than storing them locally.

**If you redistribute this app or its source, you are redistributing these
fonts, and you are responsible for satisfying yourself that you may.** If
that is a problem for your use case, remove `static/fonts/mushaf/` and fetch
the fonts at runtime from the CDN instead. Selling a build that bundles them
is the case most likely to be a problem.

The Basmala font (`QCF_BSML.woff2`) is the same artwork, taken from
[nuqayah/qpc-fonts](https://github.com/nuqayah/qpc-fonts), and carries the
same restrictions.

### QCF v4 Mushaf fonts — `static/fonts/mushaf-v4/QCF4_Hafs_01…47_W.woff2`

|           |                                                                                                                                            |
| --------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| Source    | [MohamadHajjRabee/quran-qcf4](https://github.com/MohamadHajjRabee/quran-qcf4)                                                              |
| Copyright | King Fahd Glorious Quran Printing Complex (KFGQPC) — Uthman Taha calligraphy, Madinah Mushaf 1441 AH edition; font build by Ahmad ElGharib |
| Terms     | Same KFGQPC ownership and terms as QCF v2 above — see that entry.                                                                          |

An in-progress migration from QCF v2 to this 47-file v4 set — see
`docs/qcf-v4-font-migration-plan.md`. Same calligraphy, same copyright
holder, same terms; the legal position is neither better nor worse than
QCF v2's, and the same two caveats noted there apply here too.

This source's provenance is weaker than QCF v2's: it is an independently
repackaged, previously-unpublished build (by a font-version author, not
KFGQPC or Quran Foundation directly), rather than a straight redistribution
of an org-published CDN build. `docs/mushaf-page-view-status.md` records
that this exact source was considered and **rejected** on those provenance
grounds when QCF v2's blank-glyph bug was fixed; the v4 migration plan
accepts that trade-off deliberately, for the ~58MB install-size reduction
(93.4 MiB -> 35.65 MiB) a 47-file set gives over 604 per-page files. Confirm
you're comfortable with that trade-off before shipping a release built on
this branch.

The `pages/*.json` layout data this project's importer reads from the same
repository is MIT licensed (its README says so explicitly); only the font
files themselves carry the non-MIT KFGQPC terms above.

The Surah-title banner font (`QCF4_QBSML.woff2`) is the same artwork and
terms, from the same repository. It is vendored for completeness but not used
at runtime — the banner is drawn live from `surah.name_ar`, and the Basmala
glyph comes from `QCF4_Hafs_01` rather than from this font.

### Scheherazade New — `static/fonts/scheherazade-new-regular.woff2`

|           |                                                             |
| --------- | ----------------------------------------------------------- |
| Source    | [SIL International](https://software.sil.org/scheherazade/) |
| Copyright | © 2015–2024 SIL International                               |
| License   | SIL Open Font License 1.1 — see `licenses/OFL-1.1.txt`      |

Freely redistributable, including in commercial products, provided the
license text and copyright notice travel with it (which is what this file
and `licenses/OFL-1.1.txt` are for). Reserved Font Name: "Scheherazade".

### Inter — bundled via `@fontsource-variable/inter`

|           |                                                        |
| --------- | ------------------------------------------------------ |
| Source    | [rsms/inter](https://github.com/rsms/inter)            |
| Copyright | © 2016 The Inter Project Authors                       |
| License   | SIL Open Font License 1.1 — see `licenses/OFL-1.1.txt` |

---

## Quran text and data

Compiled by `importer/` into `database/quran.db`, which is embedded in the
application binary.

### Quran text — Tanzil Project

|         |                                              |
| ------- | -------------------------------------------- |
| Source  | [tanzil.net](https://tanzil.net)             |
| Data    | Uthmani text, Simple Arabic text             |
| License | Creative Commons Attribution 3.0 (CC BY 3.0) |

Tanzil additionally asks that the text not be modified, that this copyright
notice be preserved, and that Tanzil be credited as the source. The importer
does not alter the text it receives.

### Mushaf page layout — zonetecde/mushaf-layout

|         |                                                                       |
| ------- | --------------------------------------------------------------------- |
| Source  | [zonetecde/mushaf-layout](https://github.com/zonetecde/mushaf-layout) |
| Data    | Per-page line breaks and word-level QPC v1/v2 glyph codepoints        |
| License | ISC                                                                   |

```
ISC License

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY
SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
```

Note: the importer discards this source's `surah-header` lines (they are
unreliable) and synthesizes all 114 headers from validated Surah data
instead. The line and word layout is used as published.

Each word's QCF v4 glyph (`page_line_word.glyph_v4`) additionally comes from
[MohamadHajjRabee/quran-qcf4](https://github.com/MohamadHajjRabee/quran-qcf4)
(MIT-licensed JSON; the fonts it references are proprietary — see the QCF v4
Mushaf fonts entry above), matched onto these same rows by ayah and
within-ayah word position. Line breaks and word boundaries are still this
source's (zonetecde/mushaf-layout), unchanged by which font is active.

### Ayah metadata — alquran.cloud

|        |                                                      |
| ------ | ---------------------------------------------------- |
| Source | [alquran.cloud](https://alquran.cloud)               |
| Data   | Per-ayah juz, hizb, rub', page, ruku, manzil, sajdah |

### Surah metadata — spa5k/quran_data

|        |                                                         |
| ------ | ------------------------------------------------------- |
| Source | [spa5k/quran_data](https://github.com/spa5k/quran_data) |
| Data   | Surah names, revelation types, verse counts             |

---

## Application dependencies

Rust crates and npm packages are not enumerated here. Their licenses are
resolvable from `src-tauri/Cargo.lock`, `importer/Cargo.lock` and
`pnpm-lock.yaml` — e.g. via `cargo license` and `pnpm licenses list`.
