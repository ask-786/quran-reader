# Phase 1 Research — Quran Reader

> Completed: 2026-07-25
> Status: Done

---

## 1. Quran Structure

### 1.1 Core Units

#### Surah (Chapter)
- The Quran contains **114 Surahs** of varying lengths.
- Each Surah has:
  - An Arabic name (e.g., البقرة)
  - An English name (e.g., The Cow)
  - A transliteration (e.g., Al-Baqarah)
  - A revelation type (Makki or Madani)
  - A verse count
  - An order of revelation (different from Mushaf order)
- Surahs are ordered by length (roughly), not chronologically.

#### Ayah (Verse)
- The Quran contains **6,236 Ayahs** total.
- Each Ayah belongs to exactly one Surah.
- Ayah numbers reset at the start of each Surah.
- Numbered with standard Madinah Mushaf numbering.

---

### 1.2 Reading Divisions

#### Juz (Part / Para)
- The Quran is divided into **30 Juz** (plural: Ajza).
- Designed for reading the entire Quran in 30 days (one Juz per day).
- Commonly used during Ramadan.
- Also called "Para" or "Siparah" in South Asia.

#### Hizb
- Each Juz is divided into **2 Hizbs**.
- Total: **60 Hizbs** across the Quran.

#### Rub al-Hizb (Quarter Hizb)
- Each Hizb is divided into **4 quarters** (Rub = quarter).
- Total: **240 Rub al-Hizbs** across the Quran.
- Also called "Maqra".
- Allows very granular tracking of recitation progress.
- Marked in Mushafs with the symbol ۞.

#### Manzil (Stage / Station)
- 7 divisions designed for completing the Quran in one week.
- Less commonly used in digital apps but worth including in the database.

---

### 1.3 Prayer and Reflection Sections

#### Ruku (Section / Bowing)
- Logical, thematic sections within a Surah.
- Total: approximately **558 Rukus** (some sources cite 540).
- Mark natural stopping points for Salah or thematic study.
- Marked in the Mushaf margin with the symbol **ع**.

#### Sajdah (Prostration Verse)
- **15 specific Ayahs** in the Quran.
- When recited or heard, a prostration of gratitude (Sujud al-Tilawah) is recommended.
- Marked in Mushafs with the symbol **۩**.
- Complete list of Sajdah verses:
  1. Al-A'raf 7:206
  2. Ar-Ra'd 13:15
  3. An-Nahl 16:50
  4. Al-Isra 17:109
  5. Maryam 19:58
  6. Al-Hajj 22:18
  7. Al-Hajj 22:77 *(disputed — Hanafi school omits this)*
  8. Al-Furqan 25:60
  9. An-Naml 27:26
  10. As-Sajdah 32:15
  11. Sad 38:24
  12. Fussilat 41:38
  13. An-Najm 53:62
  14. Al-Inshiqaq 84:21
  15. Al-Alaq 96:19

---

### 1.4 Page Structure (Madinah Mushaf)

- The Madinah Mushaf (King Fahd Complex edition) spans **604 pages**.
- Each page has **15 lines** (except the first and last few pages).
- Every double-page spread starts a new Juz, Hizb boundary, or thematic section — this is intentional design.
- Page numbers are specific to this edition; other editions may differ.
- The standard dataset source (Tanzil) includes Madinah Mushaf page numbers per Ayah.

---

### 1.5 Verse Numbering

- Standard numbering follows the **Madinah (Hafs) Mushaf**.
- Other numbering systems exist (Kufi, Basri, etc.) but are rarely used in software.
- For this project: **use Madinah numbering exclusively**.

---

### 1.6 Bismillah Rules

The phrase **بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ** appears at the start of every Surah except Surah 9 (At-Tawbah), and once within the text of Surah 27 (An-Naml).

#### Rule Table

| Case | Behavior |
|------|----------|
| Surah 1 (Al-Fatihah) | Bismillah **IS** verse 1 (Ayah 1:1) |
| Surahs 2–8, 10–26, 28–114 | Bismillah is a **header**, NOT an Ayah — not numbered |
| Surah 9 (At-Tawbah) | **No Bismillah** at all |
| Surah 27 (An-Naml) | Bismillah appears as header **AND** within Ayah 27:30 as part of the text |

#### Implementation Impact
- The database must store a `has_bismillah` flag per Surah.
- Surah 1 is a special case: Bismillah is stored as the first Ayah.
- All other Surahs: display Bismillah as a header element; do not include it in Ayah records.
- Surah 9: suppress the Bismillah header entirely.

---

### 1.7 Revelation Type (Makki / Madani)

- **Makki**: Revealed **before** the Hijrah (migration to Madinah).
  - Approximately **86 Surahs**
  - Themes: Tawhid (monotheism), prophethood, Day of Judgment
  - Style: Short, rhythmic, metaphorical
  - Addresses: "O mankind"
- **Madani**: Revealed **after** the Hijrah.
  - Approximately **28 Surahs**
  - Themes: Law, governance, community, family
  - Style: Longer, detailed, instructional
  - Addresses: "O you who believe"
- Note: A few Surahs are disputed between scholars.

---

### 1.8 Uthmani Script

The Uthmani script is the canonical orthography of the Quran as standardized by the Caliph Uthman ibn Affan.

#### Unicode Blocks
| Block | Range | Contents |
|-------|-------|----------|
| Arabic | U+0600–U+06FF | Core letters, standard diacritics (tashkil) |
| Arabic Extended-A | U+08A0–U+08FF | Additional Quranic annotations, letter variants |
| Arabic Extended-B | U+0870–U+089F | Specialized Quranic annotation signs |

#### Key Characters
- **Tashkil** (diacritics): Fatha, Damma, Kasra, Shadda, Sukun — combining characters in U+0600–U+06FF
- **Superscript Alef**: U+0670
- **Alef Wasla**: U+0671
- **Small High Meem** (iqlab): U+06E2
- **Small High Jeem**: U+06DA
- **Sukun**: U+0652
- **Rub al-Hizb mark**: U+06DE (۞)
- **Sajdah mark**: U+06E9 (۩)
- **End of Ayah**: U+06DD (۝)

#### Important Notes
- **Do NOT use Presentation Forms** (U+FB50–U+FDFF) — these are for backward compatibility only.
- Use base Arabic block characters; the font/shaping engine handles ligatures.
- The contextual shaping (isolated, initial, medial, final letter forms) is automatic.
- Uthmani differs from modern Arabic orthography — spellings are intentionally archaic.

---

## 2. Fonts

### 2.1 Fonts Evaluated

#### KFGQPC Uthmanic Script (Hafs)
- **Source**: King Fahd Glorious Qur'an Printing Complex
- **Purpose**: The authoritative digital font of the Madinah Mushaf
- **Quality**: Industry standard; engineered for complex diacritic stacking and specialized Quranic symbols
- **License**: ⚠️ **NOT open source**. All rights reserved by KFGQPC. Cannot be redistributed or modified without written permission.
- **Decision**: ❌ Cannot bundle with an open-source project. Would need to link to official KFGQPC source or request permission.

#### Amiri Quran
- **Source**: [github.com/alif-type/amiri](https://github.com/alif-type/amiri) — developed by Alif Type (Khaled Hosny)
- **Purpose**: A Naskh revival typeface for classical and Quranic typesetting
- **Quality**: Excellent — beautiful classical aesthetic, full Quranic diacritic support
- **License**: ✅ **SIL Open Font License (OFL) 1.1** — fully open source; can be bundled, redistributed, modified
- **Variants**: Amiri, Amiri Quran, Amiri Quran Colored
- **Google Fonts**: Available at [fonts.google.com/specimen/Amiri](https://fonts.google.com/specimen/Amiri)
- **Status**: Mature/stable release (v1.000+, 2022). Active development concluded but fully supported.

#### Noto Naskh Arabic
- **Source**: Google (part of the Noto project)
- **Purpose**: Screen and web readability — "no tofu" (no missing characters)
- **Quality**: Good — clean, modern, consistent cross-platform rendering
- **License**: ✅ **SIL Open Font License (OFL)** — fully open source
- **Best for**: UI elements, translation text, fallback rendering

#### Hafs Smart (Not researched deeply)
- Limited information found. Appears to be a variant of KFGQPC/Hafs fonts.
- Licensing unclear; treat as non-redistributable until confirmed.
- **Decision**: ❌ Skip for now.

---

### 2.2 Font Decision

| Role | Font | Reason |
|------|------|--------|
| **Primary (Quran Arabic)** | **Amiri Quran** | Open source (OFL), beautiful classical Naskh, full Quranic diacritic support, mature and stable |
| **Fallback / UI Arabic** | **Noto Naskh Arabic** | Open source, reliable cross-platform rendering, good for UI text |

> **Note**: If a premium experience is ever desired and licensing is obtained, KFGQPC can be added as an optional user-selectable font. Bundle Amiri Quran as the default.

---

## 3. Quran Dataset

### 3.1 Sources Evaluated

#### Tanzil Project (`tanzil.net`)
- **What it provides**: Industry-standard verified Quranic text in Uthmani and simplified Arabic
- **Formats**: Text, XML, SQL (MySQL dump), JavaScript (`quran-data.js` with metadata)
- **Metadata**: Includes Juz, Hizb, Hizb quarters, Manzil, Ruku, Page, and Sajdah markers per Ayah
- **License**: **Creative Commons Attribution 3.0 (CC BY 3.0)**
  - ✅ Free to use and redistribute
  - ✅ Can include in software
  - ⚠️ Must attribute: "Quran text provided by Tanzil Project (tanzil.net)"
  - ⚠️ Must link to tanzil.net in the app
  - ❌ The Quranic text itself must remain verbatim — no modification allowed
- **Accuracy**: Highly scrutinized; considered the most accurate digital source

#### quran-json by risan (`github.com/risan/quran-json`)
- Quran text in clean JSON format; derived from Tanzil data
- Useful for fast prototyping
- Available via CDN (jsDelivr/UNPKG)

#### spa5k/quran_data (`github.com/spa5k/quran_data`)
- Structured data with editions, translations, Juz, Ayah info, Sajdah, CLI operations

#### Mendeley Data — Quran Dataset
- Academic-focused; SQLite, JSON, CSV, SQL
- License: CC BY 4.0

---

### 3.2 Dataset Decision

**Primary source: Tanzil Project**

Reasons:
1. Most accurate and widely trusted
2. CC BY 3.0 allows open-source redistribution with attribution
3. Provides both Uthmani text and complete metadata (Juz, Hizb, Page, Sajdah, Ruku, Manzil)
4. Provides MySQL dump that can be converted to SQLite
5. Used by quran.com, and virtually every major Quran app

**Data to download from Tanzil:**
- Uthmani script text (the canonical Hafs text)
- Simple (unvocalized) text — useful for search/FTS
- `quran-data.js` — contains the full metadata (page, juz, hizb, ruku, manzil, sajdah)
- Surah metadata (names, revelation types, verse counts)

---

### 3.3 Translations to include (Phase 10)

| Language | Translator | Notes |
|----------|-----------|-------|
| English | Saheeh International | Clean, modern, widely used |
| English | Muhsin Khan / Hilali | King Fahd edition |
| Malayalam | Abdul Hameed & Panakkommal | Standard Malayalam translation |
| Arabic | Author's Tafsir (optional) | Simple Arabic tafsir |

---

## 4. Summary of Decisions

| Topic | Decision |
|-------|----------|
| **Primary Arabic Font** | Amiri Quran (OFL — open source) |
| **Fallback Font** | Noto Naskh Arabic (OFL — open source) |
| **Quran Text Source** | Tanzil Project (CC BY 3.0) |
| **Text Script** | Uthmani (Hafs) — primary; Simple Arabic — for search |
| **Verse Numbering** | Madinah Mushaf numbering (standard) |
| **Page Reference** | Madinah Mushaf (604 pages, 15 lines per page) |
| **Bismillah (Surah 1)** | Counted as Ayah 1:1 |
| **Bismillah (Surahs 2–8, 10–114)** | Header only — not an Ayah |
| **Bismillah (Surah 9)** | Not present |
| **Sajdah count** | 15 verses |
| **Juz count** | 30 |
| **Hizb count** | 60 |
| **Rub al-Hizb count** | 240 |
| **Ruku count** | ~558 |
| **Manzil count** | 7 |

---

## 5. Open Questions for Future Phases

- **KFGQPC font**: Should we contact KFGQPC for permission to bundle their font? (Phase 9 — Settings could allow user to install it manually)
- **Warsh vs Hafs**: Currently targeting Hafs only. Warsh support could be added later as it's used in North Africa.
- **Translations**: Which translations to bundle vs download on-demand? (Affects install size)
- **Tafsir data source**: Not researched yet — to be done in Phase 11 planning.
- **Audio data**: Not researched yet — to be done in Phase 12 planning.

---

## 6. Attribution Requirements

When the app is released, the following must be included:

### In-app (About / Credits screen)
```
Quran text: Tanzil Project (tanzil.net) — CC BY 3.0
Font: Amiri Quran by Alif Type / Khaled Hosny — SIL OFL 1.1
Font: Noto Naskh Arabic by Google — SIL OFL 1.1
```

### In the repository (LICENSE or CREDITS file)
- Copy of CC BY 3.0 license terms for Tanzil text
- Copy of OFL 1.1 for Amiri and Noto fonts
- Attribution links to tanzil.net

---

*Phase 1 complete. Proceed to Phase 2 — Project Setup.*
