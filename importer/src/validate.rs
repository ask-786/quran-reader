use anyhow::{bail, Result};
use std::collections::HashSet;

use crate::parse::QuranData;

/// Run all validation checks on the parsed Quran data.
/// Returns Ok(()) if everything passes, or the first error encountered.
pub fn validate(quran: &QuranData) -> Result<()> {
    validate_surah_count(quran)?;
    validate_ayah_count(quran)?;
    validate_surah_completeness(quran)?;
    validate_juz(quran)?;
    validate_hizb(quran)?;
    validate_rub_hizb(quran)?;
    validate_pages(quran)?;
    validate_manzil(quran)?;
    validate_sajdah(quran)?;
    validate_text_not_empty(quran)?;
    Ok(())
}

fn validate_surah_count(quran: &QuranData) -> Result<()> {
    if quran.surahs.len() != 114 {
        bail!(
            "Expected 114 Surahs, got {}",
            quran.surahs.len()
        );
    }
    // Verify IDs are 1–114 with no gaps
    let ids: HashSet<u32> = quran.surahs.iter().map(|s| s.id).collect();
    for expected in 1..=114u32 {
        if !ids.contains(&expected) {
            bail!("Missing Surah id={}", expected);
        }
    }
    Ok(())
}

fn validate_ayah_count(quran: &QuranData) -> Result<()> {
    if quran.ayahs.len() != 6236 {
        bail!(
            "Expected 6236 Ayahs, got {}",
            quran.ayahs.len()
        );
    }
    Ok(())
}

fn validate_surah_completeness(quran: &QuranData) -> Result<()> {
    // Count ayahs per surah and compare against surah.verses_count
    let mut counts: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    for a in &quran.ayahs {
        *counts.entry(a.surah_id).or_insert(0) += 1;
    }

    for surah in &quran.surahs {
        let actual = counts.get(&surah.id).copied().unwrap_or(0);
        if actual != surah.verses_count {
            bail!(
                "Surah {} expected {} verses, got {}",
                surah.id,
                surah.verses_count,
                actual
            );
        }
    }
    Ok(())
}

fn validate_juz(quran: &QuranData) -> Result<()> {
    let juz_seen: HashSet<u32> = quran.ayahs.iter().map(|a| a.juz).collect();
    for expected in 1..=30u32 {
        if !juz_seen.contains(&expected) {
            bail!("Juz {} not present in any ayah", expected);
        }
    }
    for a in &quran.ayahs {
        if !(1..=30).contains(&a.juz) {
            bail!(
                "Ayah {}:{} has invalid juz={}",
                a.surah_id, a.ayah_number, a.juz
            );
        }
    }
    Ok(())
}

fn validate_hizb(quran: &QuranData) -> Result<()> {
    let hizb_seen: HashSet<u32> = quran.ayahs.iter().map(|a| a.hizb).collect();
    for expected in 1..=60u32 {
        if !hizb_seen.contains(&expected) {
            bail!("Hizb {} not present in any ayah", expected);
        }
    }
    for a in &quran.ayahs {
        if !(1..=60).contains(&a.hizb) {
            bail!(
                "Ayah {}:{} has invalid hizb={}",
                a.surah_id, a.ayah_number, a.hizb
            );
        }
    }
    Ok(())
}

fn validate_rub_hizb(quran: &QuranData) -> Result<()> {
    for a in &quran.ayahs {
        if !(1..=240).contains(&a.rub_hizb) {
            bail!(
                "Ayah {}:{} has invalid rub_hizb={}",
                a.surah_id, a.ayah_number, a.rub_hizb
            );
        }
    }
    Ok(())
}

fn validate_pages(quran: &QuranData) -> Result<()> {
    // All 604 pages should be present
    let pages_seen: HashSet<u32> = quran.ayahs.iter().map(|a| a.page).collect();
    for expected in 1..=604u32 {
        if !pages_seen.contains(&expected) {
            bail!("Page {} not referenced by any ayah", expected);
        }
    }
    for a in &quran.ayahs {
        if !(1..=604).contains(&a.page) {
            bail!(
                "Ayah {}:{} has invalid page={}",
                a.surah_id, a.ayah_number, a.page
            );
        }
    }
    Ok(())
}

fn validate_manzil(quran: &QuranData) -> Result<()> {
    let manzil_seen: HashSet<u32> = quran.ayahs.iter().map(|a| a.manzil).collect();
    for expected in 1..=7u32 {
        if !manzil_seen.contains(&expected) {
            bail!("Manzil {} not present in any ayah", expected);
        }
    }
    Ok(())
}

fn validate_sajdah(quran: &QuranData) -> Result<()> {
    let sajdah_count = quran.ayahs.iter().filter(|a| a.sajdah).count();
    // 15 sajdah verses (the 16th in some schemata is the disputed Hanafi one —
    // alquran.cloud includes 15 by default)
    if !(14..=16).contains(&sajdah_count) {
        bail!("Expected ~15 Sajdah verses, got {}", sajdah_count);
    }
    log::info!("      Sajdah verses: {}", sajdah_count);
    Ok(())
}

fn validate_text_not_empty(quran: &QuranData) -> Result<()> {
    for a in &quran.ayahs {
        if a.uthmani_text.is_empty() {
            bail!(
                "Empty uthmani_text at {}:{}",
                a.surah_id, a.ayah_number
            );
        }
        if a.simple_text.is_empty() {
            bail!(
                "Empty simple_text at {}:{}",
                a.surah_id, a.ayah_number
            );
        }
    }
    Ok(())
}
