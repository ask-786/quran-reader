# Translation & Tafsir — Implementation Plan (Phases 10 & 11)

**Date drafted:** 2026-08-04
**Status:** Phase 11 (tafsir) is implemented for **English**, on
`claude/tafseer-implementation-plan-ntg1kx`. Tafsīr al-Jalālayn (tr. Feras
Hamza) ships in the seed database and the drawer works in both reading views,
verified in the real Tauri/WebKitGTK app. Phase 10 (translation) is untouched
apart from the schema groundwork in migration 006. The Arabic side of the
tafsir, the download packs and the wider Shāfiʿī shelf are still ahead — see
"Phasing" for what is ticked.
**Goal:** put a translation under every Ayah and a tafsir one keystroke away,
choosing editions that a Shāfiʿī reader can rely on, without abandoning
offline-first or letting the installer grow unchecked.

## TL;DR — the decisions

| Question        | Decision                                                                                                                                          |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| Default tafsir  | **Tafsīr al-Jalālayn**, English (tr. Feras Hamza), bundled — 2.13 MB of text. **Shipped.** Arabic deferred with the rest of the Arabic side       |
| Why that one    | Both authors are Egyptian Shāfiʿīs, Ashʿarī in creed, and it is _the_ tafsīr of the Kerala dars tradition since the Ponnani school                |
| Further tafsirs | al-Baghawī, al-Bayḍāwī, al-Māwardī, al-Wāḥidī, al-Rāzī, al-Suyūṭī — all Shāfiʿī, all **download-on-demand** (too large to bundle)                 |
| Default English | **Pickthall** bundled (public domain), **The Clear Qur'an** (Khattab) offered as a download — Phase 10, not started                               |
| Malayalam       | Nothing shippable: every digitised edition is Mujāhid or Jamāʿat-e-Islāmī, and the Sunni work isn't digitised. A custom-import path is the route  |
| Not carried     | Hilālī-Khān, Mawdūdī/Tafhīm, Fī Ẓilāl, al-Saʿdī, al-Muyassar/al-Mukhtaṣar, al-Qurṭubī, al-Nasafī — not offered at all, not "offered with a label" |
| Tafsir UI       | Side drawer that follows the current Ayah; works in the scrolling reader _and_ Mushaf page view                                                   |
| Delivery        | Bundled editions live in the seed DB; everything else is a signed content pack downloaded into the app data dir                                   |

Every figure below was measured, not assumed — method is given where it matters.

---

# Part 1 — Choosing the texts

## The filter being applied

A translation or tafsir is not a neutral pipe. Two things decide whether a
given edition reads as _yours_:

- **Madhhab** — matters for the āyāt al-aḥkām (wuḍūʾ, prayer times, ṭalāq,
  ribā). A Ḥanbalī or Ḥanafī commentary will derive a different ruling from
  the same verse and present it as the meaning of the verse.
- **ʿAqīda** — matters far more often, and mostly silently: the divine
  attributes (istiwāʾ, yad, wajh), tawassul, the status of taʾwīl. An Atharī
  edition and an Ashʿarī edition disagree on hundreds of verses, and the
  disagreement is usually invisible unless you already know to look.

The Shāfiʿī school of fiqh runs with Ashʿarī creed, and that is the Kerala
Sunni (Samastha) position. So the filter here is **Shāfiʿī in fiqh, Ashʿarī
in creed**, and it is a filter on what exists in the app at all — not a
labelling scheme over an everything-shelf. An edition outside it is not
offered.

The labels still matter, and they are a schema decision rather than a UI one:
`school` and `creed` columns on `tafsir` and `translation` (see Part 3), shown
in the picker so the reader knows what they are opening rather than having to
take it on trust. They are informational, not a warning badge on something
that shouldn't be there.

In practice this lives in one place: the `EDITIONS` list in
`importer/src/tafsir.rs`. Nothing can reach the app without an entry there,
and adding one is a decision about the app's purpose rather than a
data-sourcing convenience — which is why the omissions are recorded in that
file too.

## Tafsir candidates

Sizes are actual downloaded bytes. Where marked ≈, a 9-surah sample
(1, 2, 4, 9, 18, 36, 55, 78, 114 — 915 āyāt) was measured and scaled by the
×4.8 factor that the same sample gave against a full download of Jalālayn.

| Work                             | Author                                   | Madhhab / creed                        | Lang    | Size            | Verdict                     |
| -------------------------------- | ---------------------------------------- | -------------------------------------- | ------- | --------------- | --------------------------- |
| **Tafsīr al-Jalālayn**           | al-Maḥallī (d. 864) + al-Suyūṭī (d. 911) | **Shāfiʿī / Ashʿarī**                  | ar      | 2.94 MB         | **Bundle — default**        |
| **Tafsīr al-Jalālayn (English)** | tr. Feras Hamza (Royal Aal al-Bayt)      | as above                               | en      | 2.50 MB         | **Bundle — default**        |
| Maʿālim al-Tanzīl                | al-Baghawī (d. 516)                      | **Shāfiʿī / Ashʿarī**                  | ar      | ≈41 MB          | Download pack               |
| Anwār al-Tanzīl                  | al-Bayḍāwī (d. 685)                      | **Shāfiʿī / Ashʿarī**                  | ar      | ≈11 MB          | Download pack               |
| al-Nukat wa-l-ʿUyūn              | al-Māwardī (d. 450)                      | **Shāfiʿī** (chief qāḍī of the school) | ar      | ≈14 MB          | Download pack               |
| al-Wajīz + Asbāb al-Nuzūl        | al-Wāḥidī (d. 468)                       | **Shāfiʿī**                            | ar/en   | ≈8 MB           | Download pack               |
| Mafātīḥ al-Ghayb                 | Fakhr al-Dīn al-Rāzī (d. 606)            | **Shāfiʿī / Ashʿarī**                  | ar      | ≈90 MB          | Download pack (large)       |
| al-Durr al-Manthūr               | al-Suyūṭī (d. 911)                       | **Shāfiʿī**                            | ar      | ≈60 MB          | Download pack               |
| Laṭāʾif al-Ishārāt               | al-Qushayrī (d. 465)                     | **Shāfiʿī / Ashʿarī**                  | en      | ≈9 MB           | Download pack (sufi/ishārī) |
| Tafsīr Ibn Kathīr                | Ibn Kathīr (d. 774)                      | Shāfiʿī in fiqh, Atharī in creed       | ar / en | ≈75 MB / ≈32 MB | Open question — see below   |
| al-Jāmiʿ li-Aḥkām al-Qurʾān      | al-Qurṭubī (d. 671)                      | Mālikī                                 | ar      | ≈55 MB          | Not carried                 |
| Madārik al-Tanzīl                | al-Nasafī (d. 710)                       | Ḥanafī / Māturīdī                      | ar      | ≈20 MB          | Not carried                 |
| Taysīr al-Karīm al-Raḥmān        | al-Saʿdī (d. 1376)                       | Ḥanbalī / Salafī                       | ar+     | ≈12 MB          | Not carried                 |
| al-Mukhtaṣar / al-Muyassar       | Saudi committee                          | Salafī                                 | many    | ≈6 MB           | Not carried                 |
| Fī Ẓilāl al-Qurʾān               | Sayyid Quṭb                              | Ikhwānī                                | ur      | —               | Not carried                 |

**Ibn Kathīr is the one genuine borderline case.** He was a Shāfiʿī faqīh, so
he passes the madhhab half of the filter; his creed is Atharī, and the
ubiquitous English abridgement is a Darussalam edition with Salafi editorial
framing on exactly the verses where creed shows. Under a rule of "Shāfiʿī
_and_ Ashʿarī" he does not qualify, so he is left out — but that is a
judgement call, not an oversight, and it is the one worth revisiting
deliberately. If he is added, the Arabic original and the English abridgement
belong in the list as separate editions, because they are not the same book.

## Why Jalālayn is the right default

1. **Provenance.** Both Jalāls were Egyptian Shāfiʿīs. Al-Suyūṭī is a
   madhhab authority in his own right, not merely a member of it.
2. **This is the Malabar text.** Jalālayn entered the Kerala dars syllabus via
   the Ponnani dars — by tradition through al-Makhdūm II, a student of
   al-Suyūṭī — and has been taught there ever since. Of everything available
   as open data, it is the one work with a direct line to the reader's own
   tradition of study.
3. **It fits.** One volume, glosses rather than essays, 2.94 MB of Arabic. It
   is the only serious classical tafsīr that can be _bundled_ rather than
   downloaded — everything else on the Shāfiʿī shelf is 11–90 MB.
4. **It suits a reader, not a researcher.** Word-level glosses interleaved
   with the āya read well next to the text. Al-Rāzī does not.

### Two data-quality findings, both verified

- **The Arabic edition carries 6,010 entries, not 6,236.** 226 āyāt across 56
  surahs have no row. This is _not_ corruption: spot-checking against the
  English edition (2:82, 26:27, 3:2, 37:43) shows those āyāt receive no gloss
  in Jalālayn — mostly self-explanatory verses and the refrains of Sūrat
  al-Shuʿarāʾ (47 gaps) and al-Ṣāffāt (12). The importer's validation must
  therefore **not** require 6,236 rows for a tafsīr edition, and the UI must
  say "no separate commentary on this verse" rather than render an empty
  panel. The English edition has all 6,236 because Hamza supplies the plain
  verse translation where the Jalāls are silent.
- **Two English editions exist in the source and they are not equal.**
  `tafsir-al-jalalayn` preserves Hamza's punctuation and bracketed
  interpolations; `en-al-jalalayn` has had punctuation stripped, which turns
  the gloss structure into mush. Use `tafsir-al-jalalayn`. (Compare 2:255 in
  both to see it.)

## Translations

| Edition                               | Translator                                                   | Position                          | Size    | Licence              | Verdict              |
| ------------------------------------- | ------------------------------------------------------------ | --------------------------------- | ------- | -------------------- | -------------------- |
| **The Meaning of the Glorious Koran** | Marmaduke Pickthall (1930)                                   | Traditional Sunni, creed-neutral  | 1.04 MB | Public domain        | **Bundle — default** |
| The Clear Qur'an                      | Mustafa Khattab (Al-Azhar)                                   | Mainstream Sunni                  | 1.08 MB | © Book of Signs Fdn. | Download pack        |
| The Holy Qur'an                       | Abdullah Yusuf Ali (1934)                                    | Traditional, sufi-inflected notes | 1.11 MB | PD (author d. 1953)  | Optional             |
| The Study Quran                       | Nasr et al.                                                  | Traditional/Ashʿarī, heavy notes  | 1.01 MB | © HarperOne          | Do not bundle        |
| The Noble Qur'an                      | Hilālī & Khān                                                | Salafī, heavy interpolation       | —       | —                    | Not carried          |
| Towards Understanding…                | Mawdūdī                                                      | Jamāʿat-e-Islāmī                  | —       | —                    | Not carried          |
| Malayalam                             | Cheriyamundam Abdul Hameed Madani & Kunhi Mohammed Parappoor | Mujāhid / Salafī                  | 3.41 MB | via Tanzil           | Not carried          |
| Malayalam                             | Muhammad Karakunnu & Vanidas Elayavoor                       | Jamāʿat-e-Islāmī                  | 3.01 MB | via Tanzil           | Not carried          |

**Pickthall over Clear Qur'an as the bundled default is a licensing call, not
a quality one.** Khattab's English is markedly better to read; his text is
copyright Book of Signs Foundation and bundling it in signed installers needs
written permission this project does not have. Downloading it at the user's
request does not redistribute it. If permission is obtained, swap the default
— nothing else in the plan changes.

### Malayalam: the honest position

There is **no Sunni/Samastha Malayalam translation in any open dataset.** The
three Malayalam editions that exist digitally are two Mujāhid (Kerala Nadwatul
Mujahideen lineage — Cheriyamundam/Abdul Hameed Madani) and one Jamāʿat-e-Islāmī
(Karakunnu & Vanidas). The Ahlus-Sunna work a Shāfiʿī Kerala reader would
actually want — _Fatḥ al-Raḥmān fī Tafsīr al-Qurʾān_, K.V. Muhammed Musliyar,
1970, Chemmad — is in print and not digitised anywhere reachable. Amani
Moulavi's widely-used commentary is likewise Mujāhid, despite being the one
most Malayalam Quran apps ship.

Under the rule this app actually follows — carry only what is Shāfiʿī in fiqh
and Ashʿarī in creed, rather than carry everything and label it — that
conclusion is uncomfortable but clear: **no Malayalam translation can ship at
all today.** Labelling Cheriyamundam's affiliation was the earlier answer and
it is not available any more; the edition is either fit to offer or it isn't.

So the one thing left to build is the **custom edition import** path — a
CSV/JSON of 6,236 lines plus a metadata block — so _Fatḥ al-Raḥmān_ (or any
other edition) can be added by whoever can obtain and digitise it with
permission. That is now the only route this app has to Malayalam, which makes
it worth building on its own, and it is the route to the tradition the app is
being built for.

---

# Part 2 — Sources and licensing

## Where the data comes from

| Source                                                            | Covers       | Shape                                                                   | Licence (the repo)          |
| ----------------------------------------------------------------- | ------------ | ----------------------------------------------------------------------- | --------------------------- |
| [spa5k/tafsir_api](https://github.com/spa5k/tafsir_api)           | 122 tafsirs  | `tafsir/<slug>/<surah>.json` — 114 files per edition, and per-āya files | MIT (the tooling)           |
| [fawazahmed0/quran-api](https://github.com/fawazahmed0/quran-api) | 492 editions | one JSON per edition, `{quran:[{chapter,verse,text}]}`                  | Unlicense (the compilation) |
| [QUL — qul.tarteel.ai](https://qul.tarteel.ai)                    | both         | ready-made SQLite exports                                               | per-resource                |

Both GitHub-hosted sources are reachable over `raw.githubusercontent.com`,
which the existing `importer/src/fetch.rs` agent (15 s connect / 180 s global,
4 attempts with backoff) already handles well — same posture as the Mushaf
layout fetch. QUL is the better _format_ (SQLite, no 114-request fan-out) but
needs a browser download; keep it as the documented fallback if a raw source
goes away.

**The repo licence is not the text licence.** MIT/Unlicense covers those
projects' own compilation work, not the underlying translations, which carry
their translators' terms — Tanzil-sourced editions in particular have their own
redistribution conditions. This is the same distinction `LICENSE` already makes
for fonts and Quran text, and it extends cleanly.

## Bundling rules

- **Bundled** (into `quran.db`, therefore into every installer): only editions
  that are public domain or under a licence that plainly permits
  redistribution — Jalālayn Arabic (classical, PD), Pickthall (PD).
- **Download-on-demand**: everything with unresolved or restrictive terms —
  Hamza's English Jalālayn (© 2007 Royal Aal al-Bayt Institute; freely readable
  on altafsir.com, redistribution terms unverified), Clear Qur'an, the Malayalam
  editions, all large Arabic tafsirs.
- Every bundled edition gets a `THIRD-PARTY-NOTICES.md` entry (source, author,
  translator, licence, URL) and a README line, matching how the fonts and
  Tanzil text are already handled.

**Open verification task, mirroring the QCF font one already in PLAN.md
Phase 2:** confirm the redistribution terms for Hamza's English Jalālayn and
for the Tanzil-sourced Malayalam text before either is bundled rather than
downloaded. Until confirmed, both are download-only. This blocks nothing —
the pack mechanism exists either way.

---

# Part 3 — Implementation

## 3.1 Schema (migration 006)

The Phase 3 tables already exist and are close to right. What they lack is
provenance, labelling, and verse-grouping.

```sql
ALTER TABLE translation ADD COLUMN slug        TEXT;    -- 'eng-mohammedmarmadu'
ALTER TABLE translation ADD COLUMN name_native TEXT;    -- 'ഖുർആൻ പരിഭാഷ'
ALTER TABLE translation ADD COLUMN direction   TEXT NOT NULL DEFAULT 'ltr';
ALTER TABLE translation ADD COLUMN school      TEXT;    -- 'shafii' | 'hanafi' | …
ALTER TABLE translation ADD COLUMN creed       TEXT;    -- 'ashari' | 'athari' | …
ALTER TABLE translation ADD COLUMN source_url  TEXT;
ALTER TABLE translation ADD COLUMN license     TEXT;
ALTER TABLE translation ADD COLUMN sort_order  INTEGER NOT NULL DEFAULT 0;
-- same eight on tafsir

-- Grouped commentary: Ibn Kathīr and others comment on a run of verses at
-- once. Store per-āya (so lookup stays a point query) but record the run, so
-- the UI can say "2:1–5" once instead of repeating the same block five times.
ALTER TABLE tafsir_ayah ADD COLUMN group_start_ayah_id INTEGER;
ALTER TABLE tafsir_ayah ADD COLUMN group_end_ayah_id   INTEGER;
```

Two things to fix while in there:

- **`fts_translation` has no triggers.** The virtual table is declared in
  `schema.sql` but nothing keeps it in sync — it works only if rebuilt after
  a bulk insert. Add insert/delete/update triggers mirroring `fts_ayah`'s, or
  commit to rebuild-after-import and say so in a comment. Triggers are right,
  because packs are installed at runtime, not just at import time.
- **Add `fts_tafsir`** the same way. Searching tafsir is the natural companion
  to Phase 7's search UI and costs nothing to index now.

`is_bundled` already exists on both tables and finally means something: it is
what stops an uninstall from deleting rows that live in the seed DB.

## 3.2 Content packs

The seed DB is embedded with `include_bytes!`, so anything bundled costs
installer size on every platform. Current: 10.5 MB DB inside a ~100 MB
installer (36 MB of that is QCF v4 fonts). Bundling Jalālayn AR+EN plus
Pickthall adds ~6.5 MB of text — the DB goes to ~17 MB. That is a real cost
against the "small install size" principle, and it buys a reader who has a
Shāfiʿī tafsir and an English translation with no network, ever. Worth it;
everything past that point is a download.

**Pack format:** a standalone SQLite file, `<slug>.qrpack`, containing the
edition's metadata row and its 6,236 (or fewer) text rows, plus a manifest
row (schema version, sha256, byte count). Reasons over gzipped JSON: the
importer already speaks rusqlite, install becomes `ATTACH` + two
`INSERT…SELECT`s inside one transaction, and a truncated download fails
`PRAGMA integrity_check` instead of half-installing.

**Install flow:** fetch manifest → download to a temp file in the app data dir
→ verify sha256 and integrity → ATTACH, copy, DETACH, delete temp → rebuild
that edition's FTS rows. **Uninstall:** `DELETE FROM tafsir_ayah WHERE
tafsir_id = ?` guarded by `is_bundled = 0`.

**Hosting:** publish packs as assets on the GitHub release, so the same
release workflow that ships installers ships packs, and no new infrastructure
appears. A `packs.json` index at a stable URL lists what is available.

**Offline-first is not violated.** The app never phones home on its own; the
pack list is fetched only when the user opens the library, and everything
bundled works with no network at all. Say this plainly in the README.

## 3.3 Importer (`importer/`)

New subcommands, following the existing `--import-mushaf-v4` pattern:

```
cargo run --release -- --import-translation eng-mohammedmarmadu [--bundle]
cargo run --release -- --import-tafsir ar-tafsir-al-jalalayn [--bundle]
cargo run --release -- --build-pack <slug> --out ../packs/<slug>.qrpack
cargo run --release -- --import-custom <metadata.json> <text.csv>
```

- `--bundle` writes into `database/quran.db` (so the edition ships in the
  seed); without it, output is a `.qrpack`.
- A small `editions.rs` registry maps slug → {source URL template, language,
  author, title, school, creed, licence, source_url}. Curated by hand — the
  school/creed labelling is the whole point and cannot be scraped.
- Translations: one request. Tafsirs: 114 requests, sequential with the
  existing retry/backoff (the Mushaf importer's 604-request experience says
  don't parallelise into a CDN without timeouts).
- **Validation:** every `(chapter, verse)` maps to a real `ayah.id`; no rows
  outside 1–114 / valid āya numbers; no empty strings; translations must be
  complete (6,236) — tafsirs must not (see the Jalālayn finding), but log the
  gap count and refuse anything under ~90% coverage as probably broken.
- **HTML:** the sampled Jalālayn and Ibn Kathīr text is plain, but quran.com
  sourced editions do carry markup (`<b>`, `<i>`, `<sup>`, footnote anchors).
  Normalise at import — strip to an allowlist of `b i em strong sup sub br p`
  and drop everything else — so the renderer never has to trust the DB.
- Arabic tafsir quotes the āya inside `﴿ ﴾`; keep those, they render well and
  give the UI something to style.

## 3.4 Rust backend

`db/queries.rs` — `get_translation_for_surah` already exists and is
`#[allow(dead_code)]`; the rest is new:

```rust
get_translations(&conn)                                  // exists
get_tafsirs(&conn)
get_translations_for_ayahs(&conn, &[u32], &[ayah_id])    // batched, N editions
get_tafsir_for_ayah(&conn, tafsir_id, ayah_id)           // -> Option<TafsirEntry>
get_tafsir_for_surah(&conn, tafsir_id, surah_id)
search_translation(&conn, translation_id, query, limit)  // feeds Phase 7
install_pack(&conn, path) / uninstall_edition(&conn, kind, id)
```

Commands in `commands/mod.rs` + registration in `lib.rs`, same thin shape as
the existing ones. Pack download needs an HTTP client in the app — either
`tauri-plugin-http` or a `ureq` call on a blocking task; prefer the plugin so
the capability shows up in `capabilities/default.json` and the permission is
explicit.

`models/mod.rs` already has `AyahWithContext { ayah, translation, tafsir }`
sitting dead. Either make it the return shape of the batched query or delete
it — don't leave it as a third half-answer.

## 3.5 Frontend

**Settings** (new keys, plus store/type/Rust-struct updates in all three
places — `settings.svelte.ts`, `types/database.ts`, `models/mod.rs`):

| Key                  | Value               | Notes                                                                                   |
| -------------------- | ------------------- | --------------------------------------------------------------------------------------- |
| `translation_ids`    | JSON array, ordered | Replaces `preferred_translation_id`; parallel translations were always the Phase 10 ask |
| `show_translation`   | bool                | exists                                                                                  |
| `tafsir_id`          | int or empty        | active tafsir                                                                           |
| `show_tafsir`        | bool                | drawer open/closed, persisted                                                           |
| `tafsir_panel_width` | px                  | drawer is resizable                                                                     |

Keep `preferred_translation_id` readable for one version and migrate it into
`translation_ids` on load, so an existing install doesn't lose its choice.

**Translations in the reader.** `ReaderView` already takes
`translations: Record<number, string>` and `AyahRow` already renders one under
the verse — the prop is plumbed and nothing fills it. Widen to
`Record<number, TranslationLine[]>` (`{id, title, text, direction}`) for
parallel editions, load in the route's `+page.ts` alongside the āyāt, and set
`dir` per line — Malayalam and English are LTR, an Arabic-language edition
would not be.

Note the windowing constraint in `AyahRow`: rows outside the render radius
drop their content and hold a measured `reservedHeight`. Translation text is
inside that guard already, so adding lines is safe _as long as_ the measured
height is taken with translations rendered. Toggling a translation on or off
must invalidate the cached heights, or rows below will jump.

**Tafsir drawer** (`components/tafsir/TafsirPanel.svelte`): right-side,
resizable, opened from a per-Ayah button and a `t` shortcut, following the
current Ayah as you scroll (the `observeCenteredAyah` util already tracks
it). A drawer rather than an inline accordion for three reasons: it doesn't
perturb the reserved-height windowing, it works identically in `PageView`
where there is no per-Ayah row to expand into, and it keeps the Quran column
undisturbed — which is the project's stated guiding principle. An inline
expansion can come later as an option; it is not the thing to build first.

Panel contents: edition switcher (labelled with school/creed), the entry for
the current Ayah, the "no separate commentary on this verse" state for the 226
Jalālayn gaps, and copy. Arabic tafsir renders RTL with the ﴿﴾ quotes styled;
long entries scroll inside the panel, never the page.

**Fonts.** `src/app.css` already warns that nothing bundled is suitable for
running Arabic prose — Scheherazade New is a Quranic face and the reader's
Arabic is per-page QCF glyphs. Two additions, each behind its own variable:

- `--font-arabic-prose: 'Noto Naskh Arabic'` — for Arabic tafsir. ~120 KB
  subsetted.
- `--font-malayalam: 'Noto Sans Malayalam'` — Malayalam has no reliable
  system fallback on Linux; without this the translation renders as boxes on
  a clean install. Only load it when a Malayalam edition is active.

Do **not** point either at `--font-quran` or `--font-surah-name`; the comments
in `app.css` explain what breaks.

**Search (Phase 7 tie-in).** `fts_translation` + the new `fts_tafsir` give the
translation-search row in Phase 7 something real to query. Not in scope here
beyond making sure the indexes are correct and populated.

---

# Risks

1. **Install size.** +6.5 MB seed DB against a principle that is already
   strained (PLAN.md Phase 13 flags ~100 MB installers). Mitigation: only
   three editions are bundled, everything else is a pack. Revisit if the
   fonts ever move out of the binary — then bundling should be reconsidered
   wholesale, not per-edition.
2. **Licensing.** The two verification tasks above (Hamza's Jalālayn, Tanzil
   Malayalam) must land before those texts move from download to bundle.
   Getting this wrong ships a licence violation in a signed installer.
3. **Height invalidation.** Toggling translations or switching editions
   changes row heights under a windowing implementation that caches them. Test
   with a long translation on a long surah (2:282 is the stress case).
4. **114 requests × N editions.** Same CDN-stall failure mode the Mushaf v4
   import hit. Reuse the existing agent and backoff; do not fan out.
5. **Grouped-commentary editions** (Ibn Kathīr) repeat a block per āya. Without
   `group_start/group_end` the drawer re-renders the same 4 KB five times and
   looks broken. Populate the columns at import.
6. **Labelling could read as sectarian.** The school/creed chip has to be
   informational — "Shāfiʿī · Ashʿarī", "Ḥanbalī · Atharī" — never a warning
   badge. Wrong tone here makes the app feel like it is picking fights.

---

# Phasing

## Phase 10 — Translation

- [ ] **10.1** Migration 006 (columns, FTS triggers, `fts_tafsir`); editions
      registry in the importer; `--import-translation`; Pickthall imported and
      bundled; notices updated
- [ ] **10.2** Batched query + command; `+page.ts` loads translations;
      `ReaderView`/`AyahRow` render N parallel editions with per-edition
      direction; height invalidation on toggle
- [ ] **10.3** Translation picker in the settings panel (multi-select,
      ordered, school/creed labelled); `translation_ids` migration from
      `preferred_translation_id`
- [ ] **10.4** Malayalam: `--font-malayalam` (Noto Sans Malayalam), and the
      `--import-custom` path documented in `importer/README.md` — which, given
      that no Malayalam edition passes the filter, is the whole of Malayalam
      support rather than a fallback for it

Acceptance: a fresh install shows Pickthall under every āya with no network;
two translations can be shown at once.

## Phase 11 — Tafsir

- [x] **11.1** `--import-tafsir` with an editions registry, coverage
      validation and plain-text normalisation; Jalālayn **English** imported
      into the seed (6,236 entries, 2.13 MB of text; DB 10.5 → 14.4 MB after a
      VACUUM). Arabic deferred with the rest of the Arabic side
- [x] **11.2** Migration 006 + a seed-copy on upgrade (with a test); tafsir
      queries and commands; `TafsirPanel` drawer following the current Ayah;
      toolbar toggle, `t`, per-Ayah button with pin-until-scroll;
      empty-commentary state built (unused until the Arabic edition lands)
- [ ] **11.3** Word-level trigger in `PageView` — the drawer already works
      there by following the centred Ayah, but tapping a word does nothing
      (the Phase 5 word-action gap)
- [ ] **11.4** Pack infrastructure: `--build-pack`, `packs.json`, install/
      uninstall commands, library UI, release-workflow publishing. Carries a
      constraint recorded in `copy_bundled_tafsir_from_seed`: pack installs
      must allocate `tafsir.id` outside the seed's range
- [ ] **11.5** Shāfiʿī shelf published as packs: al-Baghawī, al-Bayḍāwī,
      al-Māwardī, al-Wāḥidī, al-Qushayrī
- [ ] **11.6** Verify the English Jalālayn's redistribution terms, or move it
      out of the seed and into a download (see Part 2)

Acceptance: Jalālayn opens on any āya offline ✓; the drawer follows the reader
in both views ✓; a pack installs, survives an app restart, and uninstalls
without touching bundled rows (not yet).

---

# Open questions

Decided since drafting:

- **Scope.** English tafsir first; translations deferred.
- **What the app carries.** Only Shāfiʿī/Ashʿarī works — not an
  everything-shelf with labels. That is why the Malayalam section ends where
  it does, and why Ibn Kathīr sits outside `EDITIONS`.
- **Tafsir UI.** Drawer, built.

Still open:

1. **Ibn Kathīr.** Shāfiʿī in fiqh, Atharī in creed — in or out? The only
   entry where the two halves of the filter disagree.
2. **English translation default** (Phase 10): Pickthall bundled with The
   Clear Qur'an as a download, or pursue permission and bundle Khattab?
3. **Inline tafsir.** Drawer only, or an inline expansion in the scrolling
   reader as well, accepting the height-invalidation work?
