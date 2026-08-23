-- Migration 006: reading position & reading history
--
-- Replaces the two `settings` keys that used to carry the reading position
-- (`last_read_surah_id` / `last_read_ayah_id`). That pair could only ever
-- describe one Surah, so reading a Juz, a Hizb or a Mushaf page left no trace
-- at all — and the position it did keep was written by the scrolling view
-- only, never by the Mushaf page view the app opens in.
--
-- Two tables replace it:
--
--   reading_position  where you are *right now* in each scope you have read,
--                     so reopening Juz 5 resumes inside Juz 5 and reopening
--                     Al-Baqarah resumes inside Al-Baqarah, independently. The
--                     most recently touched row is also "continue reading".
--
--   reading_session   a sitting: the Ayah a stretch of reading started at and
--                     the one it has reached. Rows are appended rather than
--                     overwritten, which is what makes a browsable history.
--
-- Only `ayah_id` is stored for a position — Surah, Ayah number, Juz, Hizb and
-- page all hang off `ayah`, so deriving them at read time keeps one truth.

CREATE TABLE IF NOT EXISTS reading_position (
    -- Which kind of range was open. 'page' is a Mushaf page number, the other
    -- three match the routes of the same name.
    scope       TEXT    NOT NULL CHECK (scope IN ('surah', 'juz', 'hizb', 'page')),
    scope_id    INTEGER NOT NULL CHECK (scope_id > 0),
    ayah_id     INTEGER NOT NULL REFERENCES ayah(id),
    updated_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    PRIMARY KEY (scope, scope_id)
);

-- "Continue reading" is the newest row across every scope.
CREATE INDEX IF NOT EXISTS idx_reading_position_updated ON reading_position(updated_at DESC);

CREATE TABLE IF NOT EXISTS reading_session (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    scope           TEXT    NOT NULL CHECK (scope IN ('surah', 'juz', 'hizb', 'page')),
    scope_id        INTEGER NOT NULL CHECK (scope_id > 0),

    -- Where the sitting began and how far it has got. Both are global Ayah
    -- ids; a session that never moved has them equal.
    start_ayah_id   INTEGER NOT NULL REFERENCES ayah(id),
    end_ayah_id     INTEGER NOT NULL REFERENCES ayah(id),

    started_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_reading_session_updated ON reading_session(updated_at DESC);

-- Carry the old position over so an upgrading reader resumes where they were.
-- `last_read_ayah_id` is a global Ayah id, so the Surah it belongs to is read
-- back off `ayah` rather than trusting the companion `last_read_surah_id` key
-- (nothing ever guaranteed the two agreed).
INSERT OR IGNORE INTO reading_position (scope, scope_id, ayah_id)
SELECT 'surah', a.surah_id, a.id
FROM ayah a
WHERE a.id = (
    SELECT CAST(value AS INTEGER) FROM settings WHERE key = 'last_read_ayah_id'
);

-- Retired keys. `scroll_position` belongs to the same abandoned attempt: it
-- has been in the defaults since v1 and was never read or written.
DELETE FROM settings
WHERE key IN ('last_read_surah_id', 'last_read_ayah_id', 'scroll_position');

INSERT OR IGNORE INTO schema_version (version) VALUES (6);
