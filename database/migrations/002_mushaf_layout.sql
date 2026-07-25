-- Migration 002: Mushaf page layout
-- Adds page_line / page_line_word so a page can be rendered with the same
-- line breaks and word placement as the printed Madani Mushaf (QCF v2).
-- Mirrors the tables defined in database/schema.sql.

CREATE TABLE IF NOT EXISTS page_line (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    page            INTEGER NOT NULL CHECK (page BETWEEN 1 AND 604),
    line_number     INTEGER NOT NULL CHECK (line_number > 0),
    line_type       TEXT    NOT NULL CHECK (line_type IN ('surah_header', 'basmala', 'text')),
    surah_id        INTEGER REFERENCES surah(id),
    first_ayah_id   INTEGER REFERENCES ayah(id),
    last_ayah_id    INTEGER REFERENCES ayah(id),
    text            TEXT,

    UNIQUE (page, line_number)
);

CREATE INDEX IF NOT EXISTS idx_page_line_page ON page_line(page);

CREATE TABLE IF NOT EXISTS page_line_word (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    page_line_id    INTEGER NOT NULL REFERENCES page_line(id) ON DELETE CASCADE,
    position        INTEGER NOT NULL,
    ayah_id         INTEGER REFERENCES ayah(id),
    word_index      INTEGER,
    uthmani_text    TEXT    NOT NULL,
    glyph_v2        TEXT    NOT NULL,

    UNIQUE (page_line_id, position)
);

CREATE INDEX IF NOT EXISTS idx_page_line_word_line ON page_line_word(page_line_id);
CREATE INDEX IF NOT EXISTS idx_page_line_word_ayah ON page_line_word(ayah_id);

INSERT INTO schema_version (version) VALUES (2);
