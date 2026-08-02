-- Migration 005: rub-el-hizb (۞) ornament glyphs
-- Pure content update — no schema change. The QCF v4 layout ships a "quarter"
-- entry carrying the ۞ ornament's own glyph, which the importer used to drop
-- because the v2-derived page_line_word table had no row to attach it to (see
-- write_glyph_v4 in importer/src/mushaf.rs). The importer now prepends it onto
-- the glyph of the word that follows it, so all 199 ornaments render for the
-- first time — page_line_word.uthmani_text has always read "۞ سَيَقُولُ" while
-- the glyph beside it drew only the word.
--
-- The data itself arrives via connection.rs rebuilding page_line/page_line_word
-- wholesale from the embedded seed, which is why this file has no UPDATEs of
-- its own. Bumping the version is what triggers that rebuild for installs
-- already sitting at v4.

INSERT INTO schema_version (version) VALUES (5);
