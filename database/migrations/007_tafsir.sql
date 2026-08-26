-- Migration 007: tafsir and translation edition metadata, verse grouping, FTS
--
-- Three things, all in service of Phase 11 (see docs/translation-tafsir-plan.md):
--
-- 1. Provenance and labelling columns on `tafsir` and `translation`. `school`
--    and `creed` exist so an edition picker can say whose reading it is
--    offering rather than presenting every commentary as interchangeable —
--    the difference between a Shafi'i/Ash'ari tafsir and a Hanbali/Athari one
--    is invisible on most verses and decisive on some.
--
-- 2. Verse grouping on `tafsir_ayah`. Ibn Kathir and others comment on a run
--    of verses at once. The run is recorded so the UI can label it "2:1–5"
--    once instead of repeating the same block five times — and so the block
--    is stored once too: it sits on the run's first Ayah, and the remaining
--    Ayahs of the run keep a row with an empty text pointing back at it
--    through `group_start_ayah_id`. Lookup stays a point query on the primary
--    key; `get_tafsir_for_ayah` follows the pointer in the same statement.
--    Null for per-Ayah editions like al-Jalalayn.
--
-- 3. FTS. `fts_translation` was declared back in 001 with no sync triggers at
--    all, so it has been silently stale since the day it was created — it
--    only ever held what a manual rebuild put there. Triggers are added here
--    for it and for the new `fts_tafsir`, because editions now arrive at
--    runtime (a downloaded pack) and not just at import time.
--
-- The bundled tafsir *content* is not inserted here. It arrives the same way
-- the Mushaf glyphs do: connection.rs copies it from the embedded seed once
-- this migration has reshaped the local tables. See copy_bundled_tafsir_from_seed.

-- 1. Edition metadata -------------------------------------------------------

ALTER TABLE tafsir ADD COLUMN slug        TEXT;
ALTER TABLE tafsir ADD COLUMN translator  TEXT;
ALTER TABLE tafsir ADD COLUMN name_native TEXT;
ALTER TABLE tafsir ADD COLUMN direction   TEXT NOT NULL DEFAULT 'ltr';
ALTER TABLE tafsir ADD COLUMN school      TEXT;
ALTER TABLE tafsir ADD COLUMN creed       TEXT;
ALTER TABLE tafsir ADD COLUMN source_url  TEXT;
ALTER TABLE tafsir ADD COLUMN license     TEXT;
ALTER TABLE tafsir ADD COLUMN sort_order  INTEGER NOT NULL DEFAULT 0;

ALTER TABLE translation ADD COLUMN slug        TEXT;
ALTER TABLE translation ADD COLUMN name_native TEXT;
ALTER TABLE translation ADD COLUMN direction   TEXT NOT NULL DEFAULT 'ltr';
ALTER TABLE translation ADD COLUMN school      TEXT;
ALTER TABLE translation ADD COLUMN creed       TEXT;
ALTER TABLE translation ADD COLUMN source_url  TEXT;
ALTER TABLE translation ADD COLUMN license     TEXT;
ALTER TABLE translation ADD COLUMN sort_order  INTEGER NOT NULL DEFAULT 0;

-- Not partial: SQLite treats NULLs as distinct in a unique index, so pre-006
-- rows with no slug coexist here without a WHERE clause — and a plain index
-- can be an upsert's ON CONFLICT target, which a partial one cannot.
CREATE UNIQUE INDEX IF NOT EXISTS idx_tafsir_slug ON tafsir(slug);
CREATE UNIQUE INDEX IF NOT EXISTS idx_translation_slug ON translation(slug);

-- 2. Grouped commentary -----------------------------------------------------

ALTER TABLE tafsir_ayah ADD COLUMN group_start_ayah_id INTEGER REFERENCES ayah(id);
ALTER TABLE tafsir_ayah ADD COLUMN group_end_ayah_id   INTEGER REFERENCES ayah(id);

-- 3. Full-text search -------------------------------------------------------

CREATE VIRTUAL TABLE IF NOT EXISTS fts_tafsir
USING fts5(
    text,
    content='tafsir_ayah',
    content_rowid='rowid',
    tokenize='unicode61 remove_diacritics 1'
);

CREATE TRIGGER IF NOT EXISTS fts_tafsir_insert
AFTER INSERT ON tafsir_ayah BEGIN
    INSERT INTO fts_tafsir(rowid, text) VALUES (NEW.rowid, NEW.text);
END;

CREATE TRIGGER IF NOT EXISTS fts_tafsir_delete
AFTER DELETE ON tafsir_ayah BEGIN
    INSERT INTO fts_tafsir(fts_tafsir, rowid, text) VALUES ('delete', OLD.rowid, OLD.text);
END;

CREATE TRIGGER IF NOT EXISTS fts_tafsir_update
AFTER UPDATE ON tafsir_ayah BEGIN
    INSERT INTO fts_tafsir(fts_tafsir, rowid, text) VALUES ('delete', OLD.rowid, OLD.text);
    INSERT INTO fts_tafsir(rowid, text) VALUES (NEW.rowid, NEW.text);
END;

CREATE TRIGGER IF NOT EXISTS fts_translation_insert
AFTER INSERT ON translation_ayah BEGIN
    INSERT INTO fts_translation(rowid, text) VALUES (NEW.rowid, NEW.text);
END;

CREATE TRIGGER IF NOT EXISTS fts_translation_delete
AFTER DELETE ON translation_ayah BEGIN
    INSERT INTO fts_translation(fts_translation, rowid, text) VALUES ('delete', OLD.rowid, OLD.text);
END;

CREATE TRIGGER IF NOT EXISTS fts_translation_update
AFTER UPDATE ON translation_ayah BEGIN
    INSERT INTO fts_translation(fts_translation, rowid, text) VALUES ('delete', OLD.rowid, OLD.text);
    INSERT INTO fts_translation(rowid, text) VALUES (NEW.rowid, NEW.text);
END;

INSERT INTO schema_version (version) VALUES (7);
