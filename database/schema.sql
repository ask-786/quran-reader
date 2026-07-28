-- =============================================================================
-- Quran Reader — Master Database Schema
-- Version: 3
-- Engine: SQLite 3 with FTS5
-- =============================================================================
-- Attribution: Quran text provided by Tanzil Project (tanzil.net) — CC BY 3.0
-- =============================================================================

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;
PRAGMA encoding = 'UTF-8';

-- =============================================================================
-- SCHEMA VERSION TRACKING
-- =============================================================================

CREATE TABLE IF NOT EXISTS schema_version (
    version     INTEGER PRIMARY KEY,
    applied_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- =============================================================================
-- SURAH (Chapter)
-- 114 chapters of the Quran
-- =============================================================================

CREATE TABLE IF NOT EXISTS surah (
    id                  INTEGER PRIMARY KEY,   -- 1–114 (Mushaf order)
    name_ar             TEXT    NOT NULL,       -- Arabic name e.g. البقرة
    name_en             TEXT    NOT NULL,       -- English name e.g. The Cow
    transliteration     TEXT    NOT NULL,       -- e.g. Al-Baqarah
    revelation_type     TEXT    NOT NULL CHECK (revelation_type IN ('Makki', 'Madani')),
    verses_count        INTEGER NOT NULL CHECK (verses_count > 0),
    order_of_revelation INTEGER NOT NULL CHECK (order_of_revelation BETWEEN 1 AND 114),
    has_bismillah       INTEGER NOT NULL DEFAULT 1 CHECK (has_bismillah IN (0, 1))
    -- Surah 1:  has_bismillah = 1 (Bismillah IS Ayah 1)
    -- Surah 9:  has_bismillah = 0 (no Bismillah at all)
    -- Others:   has_bismillah = 1 (Bismillah is header, not an Ayah)
);

-- =============================================================================
-- AYAH (Verse)
-- 6,236 verses across all Surahs
-- =============================================================================

CREATE TABLE IF NOT EXISTS ayah (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    surah_id        INTEGER NOT NULL REFERENCES surah(id),
    ayah_number     INTEGER NOT NULL CHECK (ayah_number > 0),

    -- Text variants
    uthmani_text    TEXT    NOT NULL,   -- Full Uthmani script with all diacritics
    simple_text     TEXT    NOT NULL,   -- Simplified Arabic (for FTS search)

    -- Reading divisions
    juz             INTEGER NOT NULL CHECK (juz BETWEEN 1 AND 30),
    hizb            INTEGER NOT NULL CHECK (hizb BETWEEN 1 AND 60),
    rub_hizb        INTEGER NOT NULL CHECK (rub_hizb BETWEEN 1 AND 240),
    manzil          INTEGER NOT NULL CHECK (manzil BETWEEN 1 AND 7),
    ruku            INTEGER NOT NULL CHECK (ruku > 0),

    -- Layout
    page            INTEGER NOT NULL CHECK (page BETWEEN 1 AND 604),

    -- Special markers
    sajdah          INTEGER NOT NULL DEFAULT 0 CHECK (sajdah IN (0, 1)),
    -- 1 = this ayah has a prostration (sujud al-tilawah)

    UNIQUE (surah_id, ayah_number)
);

CREATE INDEX IF NOT EXISTS idx_ayah_surah       ON ayah(surah_id);
CREATE INDEX IF NOT EXISTS idx_ayah_juz         ON ayah(juz);
CREATE INDEX IF NOT EXISTS idx_ayah_hizb        ON ayah(hizb);
CREATE INDEX IF NOT EXISTS idx_ayah_rub_hizb    ON ayah(rub_hizb);
CREATE INDEX IF NOT EXISTS idx_ayah_page        ON ayah(page);
CREATE INDEX IF NOT EXISTS idx_ayah_manzil      ON ayah(manzil);

-- =============================================================================
-- TRANSLATION (Metadata for a translation edition)
-- =============================================================================

CREATE TABLE IF NOT EXISTS translation (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    language    TEXT    NOT NULL,   -- BCP-47 language tag e.g. "en", "ml", "ar"
    translator  TEXT    NOT NULL,   -- Translator name(s)
    title       TEXT    NOT NULL,   -- Display title e.g. "Saheeh International"
    version     TEXT    NOT NULL DEFAULT '1.0',
    is_bundled  INTEGER NOT NULL DEFAULT 0 CHECK (is_bundled IN (0, 1)),
    -- Bundled translations are included in the app; others are downloaded on demand

    UNIQUE (language, translator)
);

-- =============================================================================
-- TRANSLATION_AYAH (Per-verse translation text)
-- =============================================================================

CREATE TABLE IF NOT EXISTS translation_ayah (
    translation_id  INTEGER NOT NULL REFERENCES translation(id)  ON DELETE CASCADE,
    ayah_id         INTEGER NOT NULL REFERENCES ayah(id),
    text            TEXT    NOT NULL,

    PRIMARY KEY (translation_id, ayah_id)
);

CREATE INDEX IF NOT EXISTS idx_trans_ayah_ayah ON translation_ayah(ayah_id);

-- =============================================================================
-- TAFSIR (Metadata for a tafsir work)
-- =============================================================================

CREATE TABLE IF NOT EXISTS tafsir (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    language    TEXT    NOT NULL,
    author      TEXT    NOT NULL,
    title       TEXT    NOT NULL,
    version     TEXT    NOT NULL DEFAULT '1.0',
    is_bundled  INTEGER NOT NULL DEFAULT 0 CHECK (is_bundled IN (0, 1)),

    UNIQUE (language, author, title)
);

-- =============================================================================
-- TAFSIR_AYAH (Per-verse tafsir text)
-- =============================================================================

CREATE TABLE IF NOT EXISTS tafsir_ayah (
    tafsir_id   INTEGER NOT NULL REFERENCES tafsir(id) ON DELETE CASCADE,
    ayah_id     INTEGER NOT NULL REFERENCES ayah(id),
    text        TEXT    NOT NULL,

    PRIMARY KEY (tafsir_id, ayah_id)
);

CREATE INDEX IF NOT EXISTS idx_tafsir_ayah_ayah ON tafsir_ayah(ayah_id);

-- =============================================================================
-- BOOKMARK
-- =============================================================================

CREATE TABLE IF NOT EXISTS bookmark (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    ayah_id     INTEGER NOT NULL REFERENCES ayah(id) ON DELETE CASCADE,
    label       TEXT,               -- Optional user-defined label
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    UNIQUE (ayah_id)
);

CREATE INDEX IF NOT EXISTS idx_bookmark_ayah ON bookmark(ayah_id);

-- =============================================================================
-- NOTE (Personal notes attached to an Ayah)
-- =============================================================================

CREATE TABLE IF NOT EXISTS note (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    ayah_id     INTEGER NOT NULL REFERENCES ayah(id) ON DELETE CASCADE,
    content     TEXT    NOT NULL,
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_note_ayah ON note(ayah_id);

-- Auto-update updated_at on note edits
CREATE TRIGGER IF NOT EXISTS note_updated_at
AFTER UPDATE ON note
BEGIN
    UPDATE note SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
    WHERE id = NEW.id;
END;

-- =============================================================================
-- PAGE_LINE / PAGE_LINE_WORD
-- Line-by-line Mushaf layout (Madani print, King Fahd Complex), so a page can
-- be rendered with the same line breaks and word placement as the printed
-- Mushaf instead of a reflowed list of Ayahs. Line breaks and word boundaries
-- come from the QCF v2 source; each word additionally carries its QCF v4
-- glyph, attached to the same rows (see docs/qcf-v4-font-migration-plan.md).
-- Source: zonetecde/mushaf-layout (ISC, v2 layout + glyphs) +
--         MohamadHajjRabee/quran-qcf4 (MIT JSON, v4 glyphs)
-- =============================================================================

CREATE TABLE IF NOT EXISTS page_line (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    page            INTEGER NOT NULL CHECK (page BETWEEN 1 AND 604),
    line_number     INTEGER NOT NULL CHECK (line_number > 0),
    line_type       TEXT    NOT NULL CHECK (line_type IN ('surah_header', 'basmala', 'text')),

    -- surah_header lines: which Surah the banner announces
    surah_id        INTEGER REFERENCES surah(id),

    -- text lines: the Ayah range covered by this line (a single Ayah can
    -- span several consecutive lines, e.g. Ayat al-Kursi)
    first_ayah_id   INTEGER REFERENCES ayah(id),
    last_ayah_id    INTEGER REFERENCES ayah(id),

    -- surah_header: plain Arabic Surah name text, rendered with the QCF
    -- header font. basmala/text lines leave this null — their content lives
    -- in page_line_word rows instead.
    text            TEXT,

    UNIQUE (page, line_number)
);

CREATE INDEX IF NOT EXISTS idx_page_line_page ON page_line(page);

CREATE TABLE IF NOT EXISTS page_line_word (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    page_line_id    INTEGER NOT NULL REFERENCES page_line(id) ON DELETE CASCADE,
    position        INTEGER NOT NULL,   -- 0-based order within the line

    -- Set for ordinary words. Null for a basmala line's single glyph row,
    -- which is a header convention rather than part of Ayah 1 of the Surah.
    ayah_id         INTEGER REFERENCES ayah(id),
    word_index      INTEGER,            -- 1-based word position within the Ayah

    uthmani_text    TEXT    NOT NULL,   -- plain Uthmani text (search/copy/screen readers)
    glyph_v2        TEXT    NOT NULL,   -- QCF v2 glyph string — render with font-family
                                        -- 'QCF_P{page:03}' (or 'QCF_BSML' for basmala)
    glyph_v4        TEXT,               -- QCF v4 glyph string — render with the page's
                                        -- font-map.json family (or 'QCF4_QBSML' for basmala).
                                        -- Nullable: see mushaf.rs's write_glyph_v4 for which
                                        -- rows don't get one.

    UNIQUE (page_line_id, position)
);

CREATE INDEX IF NOT EXISTS idx_page_line_word_line ON page_line_word(page_line_id);
CREATE INDEX IF NOT EXISTS idx_page_line_word_ayah ON page_line_word(ayah_id);

-- =============================================================================
-- SETTINGS (Key-value store for all user preferences)
-- =============================================================================

CREATE TABLE IF NOT EXISTS settings (
    key     TEXT PRIMARY KEY,
    value   TEXT NOT NULL
);

-- Default settings
INSERT OR IGNORE INTO settings (key, value) VALUES
    ('theme',                   'dark'),
    ('font',                    'amiri-quran'),
    ('font_size',               '28'),
    ('line_height',             '2.2'),
    ('reader_width',            'normal'),
    ('last_read_surah_id',      '1'),
    ('last_read_ayah_id',       '1'),
    ('preferred_translation_id',''),
    ('show_translation',        'true'),
    ('show_transliteration',    'false'),
    ('show_ayah_numbers',       'true'),
    ('scroll_position',         '0'),
    ('app_zoom',                '1'),
    ('reader_zoom',             '1');

-- =============================================================================
-- FULL-TEXT SEARCH (FTS5)
-- Enables fast Arabic and translation search
-- =============================================================================

-- Arabic simple-text search (diacritic-stripped)
CREATE VIRTUAL TABLE IF NOT EXISTS fts_ayah
USING fts5(
    simple_text,
    content='ayah',
    content_rowid='id',
    tokenize='unicode61 remove_diacritics 1'
);

-- Triggers to keep FTS index in sync with ayah table
CREATE TRIGGER IF NOT EXISTS fts_ayah_insert
AFTER INSERT ON ayah BEGIN
    INSERT INTO fts_ayah(rowid, simple_text) VALUES (NEW.id, NEW.simple_text);
END;

CREATE TRIGGER IF NOT EXISTS fts_ayah_delete
AFTER DELETE ON ayah BEGIN
    INSERT INTO fts_ayah(fts_ayah, rowid, simple_text) VALUES ('delete', OLD.id, OLD.simple_text);
END;

CREATE TRIGGER IF NOT EXISTS fts_ayah_update
AFTER UPDATE ON ayah BEGIN
    INSERT INTO fts_ayah(fts_ayah, rowid, simple_text) VALUES ('delete', OLD.id, OLD.simple_text);
    INSERT INTO fts_ayah(rowid, simple_text) VALUES (NEW.id, NEW.simple_text);
END;

-- Translation FTS (one virtual table, filtered by translation_id at query time)
CREATE VIRTUAL TABLE IF NOT EXISTS fts_translation
USING fts5(
    text,
    content='translation_ayah',
    content_rowid='rowid',
    tokenize='unicode61'
);

-- =============================================================================
-- RECORD SCHEMA VERSION
-- =============================================================================

INSERT OR IGNORE INTO schema_version (version) VALUES (1);
INSERT OR IGNORE INTO schema_version (version) VALUES (2);
INSERT OR IGNORE INTO schema_version (version) VALUES (3);
