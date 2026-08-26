# Third-Party Notices

Quran Reader's own source code is MIT licensed (see `LICENSE`). The Quran
text, the Mushaf layout data and every bundled font are third-party works
under their own terms. This file records what each one is, where it came
from, and what its terms allow.

If you fork this project, redistribute a build, or publish your own release,
these terms apply to you and not the MIT license.

---

## Fonts

### QCF v4 Mushaf fonts — `static/fonts/mushaf-v4/QCF4_Hafs_01…47_W.woff2`

|           |                                                                                                                                            |
| --------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| Source    | [MohamadHajjRabee/quran-qcf4](https://github.com/MohamadHajjRabee/quran-qcf4)                                                              |
| Copyright | King Fahd Glorious Quran Printing Complex (KFGQPC) — Uthman Taha calligraphy, Madinah Mushaf 1441 AH edition; font build by Ahmad ElGharib |
| Terms     | <http://dm.qurancomplex.gov.sa/copyright-2/>                                                                                               |

**These fonts are not open source.** They are owned by KFGQPC and provided
for the purpose of rendering Quranic text. As described by secondary sources
(the primary copyright page was unreachable when this was written), the terms
grant permission to _use, copy and distribute_ the font software, while
prohibiting **selling, modifying, altering, translating, reverse-engineering
or decompiling** it. Some downstream packages additionally treat the fonts as
non-commercial-use only.

Three caveats worth knowing before you rely on the above:

1. It has not been verified against the primary source at
   <http://dm.qurancomplex.gov.sa/copyright-2/>. Check it yourself.
2. The woff2 builds in circulation are format-converted versions of the
   originals. Whether that counts as prohibited "modification" is a grey area
   that the entire Quran-app ecosystem currently sits in. This project
   redistributes an existing build unaltered; it does not convert or rebuild
   anything itself.
3. This source's provenance is weaker than the Quran Foundation CDN builds
   this project used through v0.1.1: it is an independently repackaged,
   previously-unpublished build (by a font-version author, not KFGQPC or
   Quran Foundation directly), rather than a straight redistribution of an
   org-published build. `docs/mushaf-page-view-status.md` records that this
   exact source was considered and **rejected** on those provenance grounds
   when the earlier QCF v2 blank-glyph bug was fixed; the migration to v4
   accepts that trade-off deliberately, for the ~58 MB install-size reduction
   (93.4 MiB -> 35.65 MiB) a 47-file set gives over 604 per-page files.
   Confirm you're comfortable with that trade-off before shipping a release.

This repository vendors all 47 font files, and release builds bundle them, so
that the app works fully offline.

**If you redistribute this app or its source, you are redistributing these
fonts, and you are responsible for satisfying yourself that you may.** If
that is a problem for your use case, remove `static/fonts/mushaf-v4/` and
fetch the fonts at runtime instead. Selling a build that bundles them is the
case most likely to be a problem.

The Basmala glyph comes from `QCF4_Hafs_01`, under the same terms; this
project no longer ships a separate Basmala font.

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

### Amiri — `static/fonts/amiri-{arabic,latin}-regular.woff2`

|           |                                                                                    |
| --------- | ---------------------------------------------------------------------------------- |
| Source    | [aliftype/amiri](https://github.com/aliftype/amiri), via `@fontsource/amiri` 5.3.0 |
| Copyright | © 2010–2022 The Amiri Project Authors                                              |
| License   | SIL Open Font License 1.1 — see `licenses/OFL-1.1.txt`                             |

Used for running Arabic prose — the Arabic tafsir today, Arabic translations
later. Scheherazade New above is a Quranic face and reads wrong at paragraph
length, which is why this is a second Arabic font rather than a reuse. Amiri
is a revival of the Būlāq Press naskh, drawn for book setting.

It replaces Noto Naskh Arabic, which was bundled for this same job through
v0.1.9. The reason is a WebKitGTK defect rather than a matter of taste — see
the long note in `src/app.css` — and Amiri is the face that degrades best
under it.

Two files, both unmodified fontsource build artifacts: the Arabic subset plus
the Latin one, because the Arabic subset does not cover the ASCII punctuation
the tafsir text uses throughout.

Unlike Scheherazade New, this font declares **no Reserved Font Name** (checked
against the package's own LICENSE), so subset builds may ship as they are and
this project does not have to carry the full font.

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

### Tafsir — Tafsīr al-Jalālayn (Arabic)

|         |                                                                                                                         |
| ------- | ----------------------------------------------------------------------------------------------------------------------- |
| Work    | Tafsīr al-Jalālayn, by Jalāl al-Dīn al-Maḥallī (d. 864/1459) and Jalāl al-Dīn al-Suyūṭī (d. 911/1505)                   |
| Source  | [spa5k/tafsir_api](https://github.com/spa5k/tafsir_api) (edition `ar-tafsir-al-jalalayn`), which mirrors qul.tarteel.ai |
| License | Public domain — the work predates copyright by four centuries                                                           |

The Arabic original carries none of the restrictions the English translation
below does: its authors died in 864 AH and 911 AH, so the text itself is long
out of copyright. What is credited here is the digital edition and the effort
of the people who transcribed and published it.

This edition glosses 6,010 of the 6,236 Ayahs. The 226 gaps are the work's own
— verses al-Jalālayn passes over without comment — not an incomplete import.

### Tafsir — Tafsīr al-Jalālayn (English)

|            |                                                                                                                                |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------ |
| Work       | Tafsīr al-Jalālayn, by Jalāl al-Dīn al-Maḥallī (d. 864/1459) and Jalāl al-Dīn al-Suyūṭī (d. 911/1505)                          |
| Translator | Feras Hamza                                                                                                                    |
| Publisher  | © 2007 Royal Aal al-Bayt Institute for Islamic Thought, Amman                                                                  |
| Source     | [spa5k/tafsir_api](https://github.com/spa5k/tafsir_api) (edition `tafsir-al-jalalayn`), which mirrors quran.com / altafsir.com |
| License    | **Unverified — see below**                                                                                                     |

The Arabic original is a classical work in the public domain. This English
translation is not: it is Feras Hamza's, published by the Royal Aal al-Bayt
Institute and distributed free of charge on
[altafsir.com](https://www.altafsir.com). Its redistribution terms could not
be confirmed when this notice was written — altafsir.com was unreachable from
the build environment (HTTP 403), so the front matter stating the permissions
has not been read directly.

**Open task before the next release:** read those terms from the primary
source and either record them here or move this edition out of the bundled
database and into a download the user requests. The same standard as the QCF
font entry above — bundling in signed installers is the part that needs the
permission, not the reading of the text.

The importer reduces each entry to plain text (markup stripped, entities
decoded) and does not otherwise alter the translation.

### Tafsir — Tafsīr Ibn Kathīr (Arabic)

|         |                                                                                          |
| ------- | ---------------------------------------------------------------------------------------- |
| Work    | Tafsīr al-Qurʾān al-ʿAẓīm, by Ismāʿīl ibn ʿUmar ibn Kathīr (d. 774/1373)                 |
| Source  | [spa5k/tafsir_api](https://github.com/spa5k/tafsir_api) (edition `ar-tafsir-ibn-kathir`) |
| License | Public domain — the work predates copyright by six centuries                             |

Not bundled. Published as a downloadable content pack and installed only when
the reader asks for it; see `src-tauri/src/packs`.

Like the Arabic al-Jalālayn above, the composition itself is long out of
copyright and what is credited here is the digital edition. The pack carries
23.6 MB of text, which is the reason it is a download rather than part of the
installer.

This edition comments on runs of verses rather than single Ayahs — 1,911
blocks covering all 6,236 — and the importer records each run rather than
repeating its text under every verse it covers.

### Tafsir — Tafsīr Ibn Kathīr (English, abridged)

|           |                                                                                          |
| --------- | ---------------------------------------------------------------------------------------- |
| Work      | Tafsīr al-Qurʾān al-ʿAẓīm, by Ismāʿīl ibn ʿUmar ibn Kathīr (d. 774/1373)                 |
| Abridger  | A committee under Ṣafī al-Raḥmān al-Mubārakfūrī                                          |
| Publisher | © Darussalam, Riyadh                                                                     |
| Source    | [spa5k/tafsir_api](https://github.com/spa5k/tafsir_api) (edition `en-tafisr-ibn-kathir`) |
| License   | **Unverified — see below**                                                               |

Not bundled, and for this edition that is not only a question of size. This is
not a translation of the Arabic above but a modern _abridgement_ of it: most
of the isnads and much of the linguistic discussion are gone, and an editorial
apparatus that Ibn Kathīr did not write has been added. The two are listed as
separate editions for that reason.

The abridgement is a Darussalam publication and its copyright is live. Whether
it may be redistributed in this form has not been established, and no claim
either way is made here.

**Open task before this edition is offered:** establish those terms from
Darussalam directly. Until then this entry documents an edition the importer
can build, not one the app is publishing — `PACKS` in `src-tauri/src/packs`
carries no URL for it, so nothing can be downloaded.

The importer reduces each entry to plain text (markup stripped, entities
decoded) and does not otherwise alter the text.

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
