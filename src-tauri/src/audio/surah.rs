//! Whole-Surah streaming, for the listen panel.
//!
//! Stitching per-Ayah files together plays the cuts between them: each file is
//! trimmed out of a longer recording, and the breath or pause the reciter took
//! between two verses is exactly what the trim removes. Listening to a Surah
//! from one recording plays the recitation as it was recited.
//!
//! The verse card does not use any of this. One verse is still the per-Ayah
//! file from the parent module, cached on disk — that path is what a
//! pronunciation check needs, and it already works offline.
//!
//! # What comes from where
//!
//! mp3quran.net publishes each reciter's Surahs as single recordings, and for
//! many of them where each Ayah starts and ends. The timings are fetched from
//! its API; the file is named by the catalogue (`StreamSource::folder`), since
//! the timings response does not name it. The answer is kept in
//! `surah_recitation` after the first fetch, so a Surah's timings are asked for
//! once.
//!
//! The audio itself is not fetched here. The webview streams it straight from
//! `cdn.mp3quran.net` with range requests, so pressing play does not mean
//! waiting for al-Baqara's 100-odd MB. Nothing about the stream touches the
//! disk, which is why the listen panel's offline download still fetches
//! per-Ayah files: those are what the player falls back to without a network.

use super::{AudioError, AudioResult, StreamSource, CONNECT_TIMEOUT, REQUEST_TIMEOUT};
use crate::db::error::DbResult;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

/// The one API this build asks for timings. `www.` because the bare domain
/// answers with a redirect, and redirects are not followed.
const TIMINGS_API: &str = "https://www.mp3quran.net/api/v3/ayat_timing";

/// Where every streamed file is. Also how a timing row kept from an earlier
/// source is recognised and ignored: its file is somewhere else.
const STREAM_PREFIX: &str = "https://cdn.mp3quran.net/audio/";

/// Al-Baqara's response is about 70 KB. Anything far past that is not a
/// timings document.
const MAX_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;

/// How far one verse may be timed to run into the next before the two are
/// taken to disagree. Below this it is rounding between two boundary
/// estimates; above it, one of them is simply wrong.
const OVERLAP_TOLERANCE_MS: u32 = 1_000;

/// One Surah's recording, as the frontend plays it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SurahRecitation {
    pub audio_url: String,
    /// The last verse's end. The file runs on a little past it; the player
    /// takes the file's own length once it knows it.
    pub duration_ms: u32,
    /// In reading order, one per Ayah of the Surah.
    pub verses: Vec<VerseTiming>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VerseTiming {
    pub ayah_id: u32,
    /// Milliseconds into the Surah's file.
    pub start_ms: u32,
    pub end_ms: u32,
}

/// Numbers are read as `f64` so that `6420` and `6420.0` both parse; a strict
/// integer parse would fail a Surah over the difference.
#[derive(Deserialize)]
struct ApiVerse {
    /// 0 is the isti'adha and basmala the file opens with, outside any verse.
    ayah: i64,
    start_time: f64,
    end_time: f64,
}

/// Ask the API for one Surah's timings. Network only.
fn fetch(read: u32, surah_id: u32) -> AudioResult<Vec<ApiVerse>> {
    let agent = ureq::Agent::config_builder()
        .max_redirects(0)
        .max_redirects_will_error(true)
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .timeout_global(Some(REQUEST_TIMEOUT))
        .build()
        .new_agent();

    let body = agent
        .get(format!("{TIMINGS_API}?surah={surah_id}&read={read}"))
        .call()
        .map_err(|err| AudioError::Http(err.to_string()))?
        .into_body()
        .with_config()
        .limit(MAX_RESPONSE_BYTES)
        .read_to_string()
        .map_err(|err| AudioError::Http(err.to_string()))?;

    serde_json::from_str(&body)
        .map_err(|err| AudioError::Timings(format!("unreadable response: {err}")))
}

/// Check the API's answer against the Surah it is supposed to describe.
///
/// `ayah_ids` is the Surah's Ayahs in order, from this database. A response is
/// refused rather than repaired: timings for the wrong number of verses, or
/// verses out of order, would mark one verse while another is heard, and the
/// per-Ayah player is a working fallback for exactly this. Husary's al-Fatiha
/// is the one case in the catalogue: its verses are numbered from 0.
fn validate(
    source: &StreamSource,
    timings: Vec<ApiVerse>,
    surah_id: u32,
    ayah_ids: &[u32],
) -> AudioResult<SurahRecitation> {
    let bad = |why: String| Err(AudioError::Timings(why));

    let timed: Vec<&ApiVerse> = timings.iter().filter(|t| t.ayah != 0).collect();
    if timed.len() != ayah_ids.len() {
        return bad(format!(
            "{} verses timed, the Surah has {}",
            timed.len(),
            ayah_ids.len()
        ));
    }

    let mut verses = Vec::with_capacity(ayah_ids.len());
    let mut previous_start = 0u32;
    for (n, (timing, &ayah_id)) in timed.iter().zip(ayah_ids).enumerate() {
        let expected = n as i64 + 1;
        if timing.ayah != expected {
            return bad(format!(
                "expected {surah_id}:{expected}, got {}",
                timing.ayah
            ));
        }
        let (Some(start_ms), Some(end_ms)) = (ms(timing.start_time), ms(timing.end_time)) else {
            return bad(format!("{surah_id}:{expected} has no usable timestamps"));
        };
        if end_ms < start_ms || start_ms < previous_start {
            return bad(format!("{surah_id}:{expected} is out of order"));
        }
        previous_start = start_ms;
        verses.push(VerseTiming {
            ayah_id,
            start_ms,
            end_ms,
        });
    }

    // The player moves on when the next verse starts, so a verse timed to end
    // after that is an end it never reaches — and "repeat this verse" would
    // skip it. A slight overlap is clipped. A large one is refused: two
    // boundaries seconds apart are a disagreement, not rounding, and guessing
    // would mark and repeat the wrong verse.
    for i in 1..verses.len() {
        let next_start = verses[i].start_ms;
        let prev = &mut verses[i - 1];
        if prev.end_ms > next_start.saturating_add(OVERLAP_TOLERANCE_MS) {
            return bad(format!(
                "verse {} overlaps the next by {} ms",
                i,
                prev.end_ms - next_start
            ));
        }
        prev.end_ms = prev.end_ms.min(next_start).max(prev.start_ms);
    }

    Ok(SurahRecitation {
        audio_url: format!("{STREAM_PREFIX}{}/{surah_id:03}.mp3", source.folder),
        duration_ms: verses.last().map_or(0, |v| v.end_ms),
        verses,
    })
}

fn ms(value: f64) -> Option<u32> {
    (value.is_finite() && value >= 0.0 && value <= u32::MAX as f64).then(|| value.round() as u32)
}

/// Fetch and check one Surah's recording. Network only; the caller records it
/// with [`store`], taking the lock after the slow part rather than across it.
pub fn fetch_checked(
    source: &StreamSource,
    surah_id: u32,
    ayah_ids: &[u32],
) -> AudioResult<SurahRecitation> {
    validate(source, fetch(source.read, surah_id)?, surah_id, ayah_ids)
}

/// The Surah's Ayah ids in reading order.
pub fn ayah_ids(conn: &Connection, surah_id: u32) -> DbResult<Vec<u32>> {
    let mut stmt = conn.prepare("SELECT id FROM ayah WHERE surah_id = ?1 ORDER BY ayah_number")?;
    let rows = stmt.query_map(params![surah_id], |r| r.get(0))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// A recording fetched before, or `None`. A row from a different `source_id`,
/// or for a file on another host, is a recording the catalogue no longer
/// points at, and counts as absent.
pub fn cached(
    conn: &Connection,
    reciter_id: u32,
    source_id: u32,
    surah_id: u32,
) -> DbResult<Option<SurahRecitation>> {
    let head: Option<(String, u32)> = conn
        .query_row(
            "SELECT audio_url, duration_ms FROM surah_recitation
             WHERE reciter_id = ?1 AND surah_id = ?2 AND source_id = ?3",
            params![reciter_id, surah_id, source_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    // A row from an earlier source is that source's timings for its own file.
    let Some((audio_url, duration_ms)) = head.filter(|(url, _)| url.starts_with(STREAM_PREFIX))
    else {
        return Ok(None);
    };

    let mut stmt = conn.prepare(
        "SELECT v.ayah_id, v.start_ms, v.end_ms
         FROM surah_recitation_verse v
         JOIN ayah a ON a.id = v.ayah_id
         WHERE v.reciter_id = ?1 AND a.surah_id = ?2
         ORDER BY a.ayah_number",
    )?;
    let verses = stmt
        .query_map(params![reciter_id, surah_id], |r| {
            Ok(VerseTiming {
                ayah_id: r.get(0)?,
                start_ms: r.get(1)?,
                end_ms: r.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Written in one transaction, so a short list means the rows were edited
    // or lost from under it. Fetching again is cheaper than reasoning about it.
    if verses.is_empty() || verses.len() != ayah_ids(conn, surah_id)?.len() {
        return Ok(None);
    }

    Ok(Some(SurahRecitation {
        audio_url,
        duration_ms,
        verses,
    }))
}

/// Keep a fetched recording, replacing whatever was kept for this Surah before.
pub fn store(
    conn: &Connection,
    reciter_id: u32,
    source_id: u32,
    surah_id: u32,
    recitation: &SurahRecitation,
) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO surah_recitation (reciter_id, surah_id, source_id, audio_url, duration_ms)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(reciter_id, surah_id) DO UPDATE SET
             source_id   = excluded.source_id,
             audio_url   = excluded.audio_url,
             duration_ms = excluded.duration_ms,
             fetched_at  = excluded.fetched_at",
        params![
            reciter_id,
            surah_id,
            source_id,
            recitation.audio_url,
            recitation.duration_ms
        ],
    )?;
    {
        let mut upsert = tx.prepare(
            "INSERT INTO surah_recitation_verse (reciter_id, ayah_id, start_ms, end_ms)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(reciter_id, ayah_id) DO UPDATE SET
                 start_ms = excluded.start_ms,
                 end_ms   = excluded.end_ms",
        )?;
        for v in &recitation.verses {
            upsert.execute(params![reciter_id, v.ayah_id, v.start_ms, v.end_ms])?;
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: StreamSource = StreamSource {
        read: 123,
        folder: "mishary-alafasy/r1",
    };

    fn verse(ayah: i64, from: f64, to: f64) -> ApiVerse {
        ApiVerse {
            ayah,
            start_time: from,
            end_time: to,
        }
    }

    /// The shape of Alafasy's al-Fatiha: verses timed from 1, after a lead-in.
    fn fatiha_like() -> Vec<ApiVerse> {
        vec![
            verse(1, 6420.0, 11120.0),
            verse(2, 11120.0, 17640.0),
            verse(3, 17640.0, 21280.0),
        ]
    }

    #[test]
    fn a_well_formed_response_maps_onto_the_ayah_ids() {
        let rec = validate(&SOURCE, fatiha_like(), 1, &[1, 2, 3]).unwrap();
        assert_eq!(
            rec.audio_url,
            "https://cdn.mp3quran.net/audio/mishary-alafasy/r1/001.mp3"
        );
        assert_eq!(rec.verses.len(), 3);
        assert_eq!(
            rec.verses[1],
            VerseTiming {
                ayah_id: 2,
                start_ms: 11120,
                end_ms: 17640
            }
        );
        assert_eq!(rec.duration_ms, 21280);
    }

    /// The isti'adha and basmala before verse 1 come as `ayah: 0`.
    #[test]
    fn the_lead_in_is_not_a_verse() {
        let mut timings = fatiha_like();
        timings.insert(0, verse(0, 0.0, 6420.0));
        let rec = validate(&SOURCE, timings, 1, &[1, 2, 3]).unwrap();
        assert_eq!(rec.verses[0].start_ms, 6420);
    }

    /// Husary's al-Fatiha, numbered from 0: one verse short once the lead-in
    /// is set aside, and refused.
    #[test]
    fn verses_numbered_from_zero_are_refused() {
        let timings = vec![
            verse(0, 0.0, 6000.0),
            verse(1, 6000.0, 9000.0),
            verse(2, 9000.0, 12000.0),
        ];
        assert!(validate(&SOURCE, timings, 1, &[1, 2, 3]).is_err());
    }

    #[test]
    fn the_wrong_number_of_verses_is_refused() {
        assert!(validate(&SOURCE, fatiha_like(), 1, &[1, 2, 3, 4]).is_err());
    }

    #[test]
    fn a_slight_overlap_is_clipped_to_where_the_next_verse_starts() {
        let mut timings = fatiha_like();
        timings[1] = verse(2, 11120.0, 18200.0);
        let rec = validate(&SOURCE, timings, 1, &[1, 2, 3]).unwrap();
        assert_eq!(rec.verses[1].end_ms, 17640);
        assert_eq!(rec.verses[2].start_ms, 17640);
    }

    #[test]
    fn a_large_overlap_is_refused() {
        let mut timings = fatiha_like();
        timings[1] = verse(2, 11120.0, 21000.0);
        assert!(validate(&SOURCE, timings, 1, &[1, 2, 3]).is_err());
    }

    #[test]
    fn verses_out_of_order_are_refused() {
        let mut timings = fatiha_like();
        timings.swap(0, 1);
        assert!(validate(&SOURCE, timings, 1, &[1, 2, 3]).is_err());

        let mut timings = fatiha_like();
        timings[2] = verse(3, 5000.0, 21280.0);
        assert!(validate(&SOURCE, timings, 1, &[1, 2, 3]).is_err());
    }

    #[test]
    fn a_stored_recording_reads_back_and_ignores_another_source() {
        let path = std::env::temp_dir().join(format!(
            "quranreader-surah-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let conn = crate::db::connection::open(&path).unwrap();
        super::super::sync_catalog(&conn).unwrap();
        let reciter_id: u32 = conn
            .query_row("SELECT id FROM reciter LIMIT 1", [], |r| r.get(0))
            .unwrap();

        let ids = ayah_ids(&conn, 1).unwrap();
        assert_eq!(ids.len(), 7);
        let timings = (1..=7)
            .map(|n| verse(n, n as f64 * 1000.0, n as f64 * 1000.0 + 900.0))
            .collect();
        let rec = validate(&SOURCE, timings, 1, &ids).unwrap();

        store(&conn, reciter_id, 123, 1, &rec).unwrap();
        assert_eq!(
            cached(&conn, reciter_id, 123, 1).unwrap(),
            Some(rec.clone())
        );
        assert_eq!(cached(&conn, reciter_id, 124, 1).unwrap(), None);

        // Storing again replaces rather than duplicates.
        store(&conn, reciter_id, 123, 1, &rec).unwrap();
        assert_eq!(
            cached(&conn, reciter_id, 123, 1).unwrap(),
            Some(rec.clone())
        );

        // A row kept from an earlier source, whose file is on another host.
        let elsewhere = SurahRecitation {
            audio_url: "https://download.quranicaudio.com/qdc/x/1.mp3".to_string(),
            ..rec
        };
        store(&conn, reciter_id, 123, 1, &elsewhere).unwrap();
        assert_eq!(cached(&conn, reciter_id, 123, 1).unwrap(), None);

        drop(conn);
        let _ = std::fs::remove_file(&path);
    }

    /// Opt-in, like the per-Ayah fetch test:
    /// `cargo test -- --ignored fetches_real_surah_timings`.
    #[test]
    #[ignore]
    fn fetches_real_surah_timings() {
        let ids: Vec<u32> = (1..=7).collect();
        let rec = fetch_checked(&SOURCE, 1, &ids).expect("fetch failed");
        assert_eq!(rec.verses.len(), 7);
        assert!(rec.audio_url.starts_with(STREAM_PREFIX));
    }
}
