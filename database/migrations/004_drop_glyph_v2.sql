-- Migration 004: drop the QCF v2 glyph column
-- The v2 fonts are gone from the bundle as of this migration (see
-- docs/qcf-v4-font-migration-plan.md), so glyph_v2 has nothing left to render
-- with. Rollback is a revert of that commit, not a runtime fallback, which is
-- what keeping this column was buying.
--
-- Ordering matters. connection.rs rebuilds page_line_word from the embedded
-- seed with `INSERT ... SELECT *`, which is positional, and the seed no longer
-- has this column — so this ALTER has to run *before* that rebuild, not after.
-- run_migrations applies every migration's SQL first and rebuilds once at the
-- end for exactly this reason.

ALTER TABLE page_line_word DROP COLUMN glyph_v2;

INSERT INTO schema_version (version) VALUES (4);
