-- Migration 003: QCF v4 glyphs
-- Adds page_line_word.glyph_v4 alongside the existing glyph_v2 so both font
-- versions' glyph data can coexist (see docs/qcf-v4-font-migration-plan.md).
-- Nullable: unlike glyph_v2, not every row gets a v4 glyph (see mushaf.rs's
-- write_glyph_v4 doc comment for which rows don't).
--
-- This ALTER alone leaves every existing row's glyph_v4 NULL — connection.rs
-- follows it by rebuilding page_line/page_line_word from the bundled seed DB,
-- which already has this migration's data populated.

ALTER TABLE page_line_word ADD COLUMN glyph_v4 TEXT;

INSERT INTO schema_version (version) VALUES (3);
