-- Migration 012: whole-Surah streaming for the listen panel
--
-- The listen panel streams one recording per Surah rather than stitching
-- per-Ayah files together, so the reciter's own breaths and pauses between
-- verses are what the reader hears instead of the seams between cuts. The
-- verse card is untouched: one verse is still the per-Ayah file, cached on
-- disk, as migration 009 set up. See docs/audio-plan.md, "Whole-Surah
-- streaming".
--
-- A stream has no verse boundaries of its own, so where each Ayah starts and
-- ends comes from the same API response that names the file. These tables keep
-- that answer after the first fetch: a Surah listened to once needs no second
-- request for its timings, and the audio itself is never stored here.
--
-- `source_id` is the reciter's id on the timings API at the time of the fetch.
-- The catalogue is compiled in and can move a reciter to a different recording;
-- a row from the old one is then ignored rather than played against the new
-- file with someone else's timings.

CREATE TABLE IF NOT EXISTS surah_recitation (
    reciter_id  INTEGER NOT NULL REFERENCES reciter(id) ON DELETE CASCADE,
    surah_id    INTEGER NOT NULL REFERENCES surah(id),
    source_id   INTEGER NOT NULL,
    audio_url   TEXT    NOT NULL,
    duration_ms INTEGER NOT NULL,
    fetched_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    PRIMARY KEY (reciter_id, surah_id)
);

-- Milliseconds into the Surah's file, not into the verse.
CREATE TABLE IF NOT EXISTS surah_recitation_verse (
    reciter_id  INTEGER NOT NULL REFERENCES reciter(id) ON DELETE CASCADE,
    ayah_id     INTEGER NOT NULL REFERENCES ayah(id),
    start_ms    INTEGER NOT NULL,
    end_ms      INTEGER NOT NULL CHECK (end_ms >= start_ms),

    PRIMARY KEY (reciter_id, ayah_id)
);

INSERT INTO schema_version (version) VALUES (12);
