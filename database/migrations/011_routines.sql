-- Migration 011: reading routines
--
-- A routine is a named, ordered list of Surahs read as a set — the same few
-- every morning, say — so that finishing one leads to the next in the
-- routine's order rather than the next in the Mushaf. See docs/routines-plan.md.
--
-- Progress is kept per routine rather than in one settings key, so being part
-- way through one routine survives starting another. A run belongs to the day
-- it was started on; what counts as "today" is the reader's local date, which
-- only the frontend knows, so the table stores the UTC instant and leaves the
-- calendar to the caller.

CREATE TABLE IF NOT EXISTS routine (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT    NOT NULL CHECK (length(trim(name)) > 0),
    -- Where it sits in the list of routines. Rewritten wholesale on reorder.
    sort_order      INTEGER NOT NULL DEFAULT 0,

    -- The current run: the index of the step it is on, or the item count once
    -- the last step is done. Both NULL until the routine is first started.
    run_step        INTEGER CHECK (run_step >= 0),
    run_started_at  TEXT,

    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS routine_item (
    routine_id  INTEGER NOT NULL REFERENCES routine(id) ON DELETE CASCADE,
    position    INTEGER NOT NULL CHECK (position >= 0),
    surah_id    INTEGER NOT NULL REFERENCES surah(id),

    -- Reserved for partial readings (Ayat al-Kursi, the last two Ayahs of
    -- Al-Baqarah). Nothing writes them yet; NULL means the whole Surah.
    start_ayah  INTEGER,
    end_ayah    INTEGER,

    PRIMARY KEY (routine_id, position),
    -- A Surah once per routine, so "which step is this Surah?" has one answer.
    UNIQUE (routine_id, surah_id)
);

INSERT OR IGNORE INTO schema_version (version) VALUES (11);
