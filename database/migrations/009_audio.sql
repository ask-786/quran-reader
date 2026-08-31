-- Migration 009: recitation audio
--
-- Phase 12 (see docs/audio-plan.md). Audio is fetched per Ayah from a public
-- CDN when the reader presses play and cached on disk forever; nothing is
-- bundled and nothing is published by this project. So the tables here hold
-- metadata and bookkeeping only — never a byte of audio.
--
-- 1. `reciter` mirrors `tafsir`'s shape. It is filled from the catalogue
--    compiled into the binary (`audio::RECITERS`) every time the database
--    opens, upserted by slug, so adding a reciter is an app release and not a
--    migration. `riwaya` is on the record for the reason `school` and `creed`
--    are on a tafsir: almost every recording in circulation is Hafs 'an
--    'Asim, which is exactly why the one that isn't should be labelled rather
--    than discovered halfway through a Surah.
--
-- 2. `audio_file` is bookkeeping over the cache directory, not the truth about
--    it. The files on disk are the truth. This exists so "is this Surah
--    downloaded?" and "how much disk is audio using?" are one query instead of
--    6,236 stat calls, and it is reconciled against the filesystem on demand.
--
-- 3. `recitation_segment` ships empty. Word-by-word highlighting needs per-word
--    timings, `page_line_word.word_index` is already there to join against, and
--    declaring the table now costs nothing.

-- 1. Reciters ---------------------------------------------------------------

CREATE TABLE IF NOT EXISTS reciter (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    slug        TEXT    NOT NULL UNIQUE,  -- the id the CDN knows them by, e.g. "ar.alafasy"
    name_ar     TEXT    NOT NULL,
    name_en     TEXT    NOT NULL,
    riwaya      TEXT    NOT NULL DEFAULT 'Hafs an Asim',
    style       TEXT    NOT NULL DEFAULT 'murattal' CHECK (style IN ('murattal', 'mujawwad')),
    source_url  TEXT,
    license     TEXT,
    sort_order  INTEGER NOT NULL DEFAULT 0
);

-- 2. Cache bookkeeping ------------------------------------------------------

CREATE TABLE IF NOT EXISTS audio_file (
    reciter_id  INTEGER NOT NULL REFERENCES reciter(id) ON DELETE CASCADE,
    bitrate     INTEGER NOT NULL,
    ayah_id     INTEGER NOT NULL REFERENCES ayah(id),
    bytes       INTEGER NOT NULL,
    fetched_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    PRIMARY KEY (reciter_id, bitrate, ayah_id)
);

CREATE INDEX IF NOT EXISTS idx_audio_file_ayah ON audio_file(ayah_id);

-- 3. Word timings, for later ------------------------------------------------

CREATE TABLE IF NOT EXISTS recitation_segment (
    reciter_id  INTEGER NOT NULL REFERENCES reciter(id) ON DELETE CASCADE,
    ayah_id     INTEGER NOT NULL REFERENCES ayah(id),
    word_index  INTEGER NOT NULL,   -- 1-based, matching page_line_word.word_index
    start_ms    INTEGER NOT NULL,
    end_ms      INTEGER NOT NULL,

    PRIMARY KEY (reciter_id, ayah_id, word_index)
);

-- 4. Settings ---------------------------------------------------------------
--
-- Defaults are also in `load_settings`, which is what actually applies to a
-- database that predates this migration. Both places, same reason `tafsir_view`
-- is in both: a settings key needs no migration to gain a default.

INSERT OR IGNORE INTO settings (key, value) VALUES
    -- Empty = no reciter chosen. There is deliberately no default: choosing
    -- one is what turns the network on, and that is the reader's call.
    ('reciter_id',              ''),
    -- 64 and 128 are the only bitrates the CDN serves; 32/48/192 are 403.
    ('audio_bitrate',           '64'),
    ('audio_repeat_mode',       'off'),   -- 'off' | 'ayah' | 'range'
    ('audio_repeat_count',      '3'),
    ('audio_repeat_pause_ms',   '0'),
    ('audio_playback_rate',     '1'),
    ('audio_follow',            'true'),
    ('audio_volume',            '1'),
    -- Off until the reader approves the first fetch by name of host. Turning
    -- it off again is the "cached only" mode: playback keeps working for
    -- everything already on disk and nothing leaves the machine.
    ('audio_downloads_allowed', 'false');

INSERT INTO schema_version (version) VALUES (9);
