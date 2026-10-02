-- Migration 010: ayah.page re-derived from the Mushaf page layout
-- Pure content update — no schema change.
--
-- `ayah.page` came from Tanzil's metadata, while `page_line`/`page_line_word`
-- come from the QCF v4 layout (see 008). The two break pages differently for
-- 56 Ayahs, and the reader computes the pages to load from `ayah.page` — so a
-- range whose last Ayahs Tanzil puts a page earlier than v4 never loaded the
-- page they are actually drawn on. Ash-Sharh (94) is the visible case: Tanzil
-- puts all eight Ayahs on 596, v4 sets 3–8 on 597, and the Surah view stopped
-- after Ayah 2.
--
-- The layout is what the reader draws, so it is the source of truth: an
-- Ayah's page is the page its first word is laid out on. The importer does the
-- same right after writing the layout (importer/src/mushaf.rs), so the seed
-- and an upgraded install agree.

UPDATE ayah SET page = (
    SELECT MIN(pl.page)
    FROM page_line_word w
    JOIN page_line pl ON pl.id = w.page_line_id
    WHERE w.ayah_id = ayah.id
)
WHERE EXISTS (SELECT 1 FROM page_line_word w WHERE w.ayah_id = ayah.id);

INSERT INTO schema_version (version) VALUES (10);
