use anyhow::{bail, Context, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::Deserialize;
use std::collections::HashMap;

use crate::fetch::RawData;

// ---------------------------------------------------------------------------
// Intermediate types — filled during parsing, written to DB later
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct QuranData {
    pub surahs: Vec<Surah>,
    pub ayahs:  Vec<Ayah>,
}

#[derive(Debug)]
pub struct Surah {
    pub id:                  u32,
    pub name_ar:             String,
    pub name_en:             String,
    pub transliteration:     String,
    pub revelation_type:     String, // "Makki" | "Madani"
    pub verses_count:        u32,
    pub order_of_revelation: u32,
    pub has_bismillah:       bool,
}

#[derive(Debug)]
pub struct Ayah {
    pub surah_id:     u32,
    pub ayah_number:  u32,
    pub uthmani_text: String,
    pub simple_text:  String,
    pub juz:          u32,
    pub hizb:         u32,
    pub rub_hizb:     u32,
    pub manzil:       u32,
    pub ruku:         u32,
    pub page:         u32,
    pub sajdah:       bool,
}

// ---------------------------------------------------------------------------
// Surah metadata from JSON (spa5k/quran_data)
// ---------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SurahInfo {
    number:                   u32,
    name:                     String,
    english_name:             String,
    english_name_translation: String,
    number_of_ayahs:          u32,
    revelation_type:          String,
}

// Alternative JSON schema (some sources use different field names)
#[derive(Debug, Deserialize)]
struct SurahInfoAlt {
    number:               u32,
    name:                 Option<String>,
    #[serde(rename = "nameArabic")]
    name_arabic:          Option<String>,
    #[serde(rename = "nameEnglish")]
    name_english:         Option<String>,
    #[serde(rename = "nameTrans")]
    name_trans:           Option<String>,
    transliteration:      Option<String>,
    #[serde(rename = "englishName")]
    english_name:         Option<String>,
    #[serde(rename = "revelationType")]
    revelation_type:      Option<String>,
    #[serde(rename = "numberOfAyahs")]
    number_of_ayahs:      Option<u32>,
    #[serde(rename = "versesCount")]
    verses_count:         Option<u32>,
    #[serde(rename = "orderOfRevelation")]
    order_of_revelation:  Option<u32>,
    #[serde(rename = "revelationOrder")]
    revelation_order:     Option<u32>,
}

// Hardcoded revelation order (standard Uloom al-Quran order) as fallback.
// Source: widely agreed-upon academic consensus.
// (unused — kept for reference; actual lookup uses revelation_order() function below)
#[allow(dead_code)]
const REVELATION_ORDER_REF: [u32; 10] = [96, 74, 73, 111, 81, 87, 92, 89, 93, 94];

/// Hardcoded, authoritative revelation order by surah number (1-indexed).
fn revelation_order(surah_id: u32) -> u32 {
    // Standard revelation sequence used by most academic sources
    // (based on Egyptian/Azhari order, used by Quran.com and Tanzil metadata)
    // Standard revelation order by Mushaf surah number (1-indexed).
    // Based on Egyptian/Azhari order used by Quran.com and Tanzil.
    // Index 0 is padding.
    const ORDER: [u32; 115] = [
        0,   // 0  padding
        5,   // 1  Al-Fatihah
        87,  // 2  Al-Baqarah
        89,  // 3  Al-Imran
        92,  // 4  An-Nisa
        112, // 5  Al-Ma'idah
        55,  // 6  Al-An'am
        39,  // 7  Al-A'raf
        88,  // 8  Al-Anfal
        113, // 9  At-Tawbah
        51,  // 10 Yunus
        52,  // 11 Hud
        53,  // 12 Yusuf
        96,  // 13 Ar-Ra'd
        72,  // 14 Ibrahim
        54,  // 15 Al-Hijr
        70,  // 16 An-Nahl
        50,  // 17 Al-Isra
        69,  // 18 Al-Kahf
        44,  // 19 Maryam
        45,  // 20 Ta-Ha
        73,  // 21 Al-Anbiya
        103, // 22 Al-Hajj
        74,  // 23 Al-Mu'minun
        102, // 24 An-Nur
        42,  // 25 Al-Furqan
        47,  // 26 Ash-Shu'ara
        48,  // 27 An-Naml
        49,  // 28 Al-Qasas
        85,  // 29 Al-Ankabut
        84,  // 30 Ar-Rum
        57,  // 31 Luqman
        75,  // 32 As-Sajdah
        90,  // 33 Al-Ahzab
        58,  // 34 Saba
        43,  // 35 Fatir
        41,  // 36 Ya-Sin
        56,  // 37 As-Saffat
        38,  // 38 Sad
        59,  // 39 Az-Zumar
        60,  // 40 Ghafir
        61,  // 41 Fussilat
        62,  // 42 Ash-Shura
        63,  // 43 Az-Zukhruf
        64,  // 44 Ad-Dukhan
        65,  // 45 Al-Jathiyah
        66,  // 46 Al-Ahqaf
        95,  // 47 Muhammad
        111, // 48 Al-Fath
        106, // 49 Al-Hujurat
        34,  // 50 Qaf
        23,  // 51 Adh-Dhariyat
        32,  // 52 At-Tur
        23,  // 53 An-Najm
        37,  // 54 Al-Qamar
        97,  // 55 Ar-Rahman
        46,  // 56 Al-Waqi'ah
        94,  // 57 Al-Hadid
        105, // 58 Al-Mujadila
        101, // 59 Al-Hashr
        91,  // 60 Al-Mumtahanah
        109, // 61 As-Saf
        110, // 62 Al-Jumu'ah
        104, // 63 Al-Munafiqun
        108, // 64 At-Taghabun
        99,  // 65 At-Talaq
        107, // 66 At-Tahrim
        77,  // 67 Al-Mulk
        2,   // 68 Al-Qalam
        78,  // 69 Al-Haqqah
        79,  // 70 Al-Ma'arij
        71,  // 71 Nuh
        40,  // 72 Al-Jinn
        3,   // 73 Al-Muzzammil
        4,   // 74 Al-Muddaththir
        31,  // 75 Al-Qiyamah
        98,  // 76 Al-Insan
        32,  // 77 Al-Mursalat
        80,  // 78 An-Naba
        81,  // 79 An-Nazi'at
        27,  // 80 Abasa
        7,   // 81 At-Takwir
        82,  // 82 Al-Infitar
        86,  // 83 Al-Mutaffifin
        83,  // 84 Al-Inshiqaq
        27,  // 85 Al-Buruj
        36,  // 86 At-Tariq
        8,   // 87 Al-A'la
        68,  // 88 Al-Ghashiyah
        10,  // 89 Al-Fajr
        35,  // 90 Al-Balad
        26,  // 91 Ash-Shams
        9,   // 92 Al-Layl
        11,  // 93 Ad-Duha
        12,  // 94 Ash-Sharh
        25,  // 95 At-Tin
        1,   // 96 Al-Alaq
        25,  // 97 Al-Qadr
        100, // 98 Al-Bayyinah
        33,  // 99 Az-Zalzalah
        14,  // 100 Al-Adiyat
        30,  // 101 Al-Qari'ah
        16,  // 102 At-Takathur
        13,  // 103 Al-Asr
        15,  // 104 Al-Humazah
        19,  // 105 Al-Fil
        29,  // 106 Quraysh
        17,  // 107 Al-Ma'un
        21,  // 108 Al-Kawthar
        18,  // 109 Al-Kafirun
        114, // 110 An-Nasr
        6,   // 111 Al-Masad
        22,  // 112 Al-Ikhlas
        20,  // 113 Al-Falaq
        21,  // 114 An-Nas
    ];
    if surah_id as usize >= ORDER.len() {
        surah_id // fallback
    } else {
        ORDER[surah_id as usize]
    }
}

// ---------------------------------------------------------------------------
// Tanzil metadata: juz/hizb/page/ruku/manzil/sajdah per ayah
// ---------------------------------------------------------------------------
// Tanzil provides quran-data.js. Since we can't import JS, we embed equivalent
// data. The quran-data.js is a JS file with arrays; we parse a JSON equivalent
// from spa5k/quran_data which provides per-ayah metadata in JSON.

// We'll use a second API call approach: fetch per-ayah metadata from
// alquran.cloud (Open Islamic Foundation) which returns clean JSON with
// juz, hizb, page, ruku, sajda etc.

// Structure from alquran.cloud /quran/en.asad (metadata fields)
#[derive(Debug, Deserialize)]
pub struct AlquranResponse {
    pub data: AlquranData,
}

#[derive(Debug, Deserialize)]
pub struct AlquranData {
    pub surahs: Vec<AlquranSurah>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct AlquranSurah {
    pub number:                   u32,
    pub name:                     String,
    #[serde(rename = "englishName")]
    pub english_name:             String,
    #[serde(rename = "englishNameTranslation")]
    pub english_name_translation: String,
    #[serde(rename = "numberOfAyahs")]
    pub number_of_ayahs:          u32,
    #[serde(rename = "revelationType")]
    pub revelation_type:          String,
    pub ayahs:                    Vec<AlquranAyah>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct AlquranAyah {
    #[serde(rename = "numberInSurah")]
    pub number_in_surah: u32,
    pub juz:             u32,
    pub manzil:          u32,
    pub page:            u32,
    pub ruku:            u32,
    #[serde(rename = "hizbQuarter")]
    pub hizb_quarter:    u32,
    #[serde(rename = "sajda")]
    pub sajda:           Sajda,
    pub text:            String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Sajda {
    Bool(bool),
    Object { id: u32, recommended: bool, obligatory: bool },
}

impl Sajda {
    pub fn is_sajda(&self) -> bool {
        match self {
            Sajda::Bool(b) => *b,
            Sajda::Object { .. } => true,
        }
    }
}

/// Key: (surah_id, ayah_number) → metadata
pub type MetaMap = HashMap<(u32, u32), AyahMeta>;

#[derive(Debug)]
pub struct AyahMeta {
    pub juz:      u32,
    pub hizb:     u32,     // computed from hizb_quarter
    pub rub_hizb: u32,     // hizb_quarter (1–240)
    pub manzil:   u32,
    pub ruku:     u32,
    pub page:     u32,
    pub sajdah:   bool,
}

// ---------------------------------------------------------------------------
// Main parse entry point
// ---------------------------------------------------------------------------

pub fn parse(raw: &RawData) -> Result<QuranData> {
    // 1. Parse surah metadata JSON
    let surah_meta = parse_surah_meta(&raw.surah_meta_json)
        .context("Parsing surah metadata JSON")?;

    // 2. Fetch and parse per-ayah metadata from alquran.cloud
    log::info!("  → Fetching per-ayah metadata from alquran.cloud …");
    let meta_map = fetch_ayah_metadata().context("Fetching ayah metadata")?;
    log::info!("      {} ayah metadata records", meta_map.len());

    // 3. Parse Uthmani XML
    let uthmani_map = parse_quran_xml(&raw.uthmani_xml)
        .context("Parsing Uthmani XML")?;
    log::info!("      {} Uthmani ayahs", uthmani_map.len());

    // 4. Parse Simple XML
    let simple_map = parse_quran_xml(&raw.simple_xml)
        .context("Parsing Simple Arabic XML")?;
    log::info!("      {} Simple ayahs", simple_map.len());

    // 5. Merge everything
    assemble(surah_meta, uthmani_map, simple_map, meta_map)
}

// ---------------------------------------------------------------------------
// Parse surah metadata JSON
// ---------------------------------------------------------------------------

fn parse_surah_meta(json: &str) -> Result<Vec<SurahInfoAlt>> {
    // Try to parse as array of SurahInfoAlt
    let parsed: Vec<SurahInfoAlt> = serde_json::from_str(json)
        .context("JSON parse error for surah metadata")?;
    Ok(parsed)
}

// ---------------------------------------------------------------------------
// Fetch per-ayah metadata from alquran.cloud
// ---------------------------------------------------------------------------

fn fetch_ayah_metadata() -> Result<MetaMap> {
    // Use the alquran.cloud API which returns all surahs with ayah metadata.
    // We use a minimal translation (en.asad) just to get the structure;
    // we don't actually use the translation text here.
    let url = "https://api.alquran.cloud/v1/quran/en.asad";
    let body = crate::fetch::get_pub(url)?;

    let resp: AlquranResponse = serde_json::from_str(&body)
        .context("JSON parse error for alquran.cloud response")?;

    let mut map = HashMap::new();

    for surah in &resp.data.surahs {
        for ayah in &surah.ayahs {
            // hizb_quarter is 1–240 (rub al-hizb)
            // hizb = ceil(hizb_quarter / 4)
            let rub_hizb = ayah.hizb_quarter;
            let hizb = ((rub_hizb as f64) / 4.0).ceil() as u32;
            let hizb = hizb.max(1).min(60);

            map.insert(
                (surah.number, ayah.number_in_surah),
                AyahMeta {
                    juz:      ayah.juz,
                    hizb,
                    rub_hizb,
                    manzil:   ayah.manzil,
                    ruku:     ayah.ruku,
                    page:     ayah.page,
                    sajdah:   ayah.sajda.is_sajda(),
                },
            );
        }
    }

    Ok(map)
}

// ---------------------------------------------------------------------------
// Parse Tanzil XML
// ---------------------------------------------------------------------------
// Tanzil XML format:
//   <quran>
//     <sura index="1" name="الفاتحة">
//       <aya index="1" text="بِسْمِ اللَّهِ …"/>
//       …
//     </sura>
//     …
//   </quran>

/// Returns a map of (surah_id, ayah_number) → text
fn parse_quran_xml(xml: &str) -> Result<HashMap<(u32, u32), String>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut map = HashMap::new();
    let mut current_surah: u32 = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                match e.name().as_ref() {
                    b"sura" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"index" {
                                let val = attr.unescape_value()?;
                                current_surah = val.parse()?;
                            }
                        }
                    }
                    b"aya" => {
                        let mut ayah_num = 0u32;
                        let mut text = String::new();
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"index" => {
                                    let val = attr.unescape_value()?;
                                    ayah_num = val.parse()?;
                                }
                                b"text" => {
                                    text = attr.unescape_value()?.into_owned();
                                }
                                _ => {}
                            }
                        }
                        if current_surah > 0 && ayah_num > 0 && !text.is_empty() {
                            map.insert((current_surah, ayah_num), text);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => bail!("XML parse error: {e}"),
            _ => {}
        }
    }

    Ok(map)
}

// ---------------------------------------------------------------------------
// Assemble final QuranData
// ---------------------------------------------------------------------------

fn normalize_revelation_type(s: &str) -> String {
    match s.to_lowercase().as_str() {
        "meccan" | "makki" | "mekkan" => "Makki".to_string(),
        "medinan" | "madani" | "madinah" => "Madani".to_string(),
        other => {
            log::warn!("Unknown revelation type: {other}, defaulting to Makki");
            "Makki".to_string()
        }
    }
}

fn assemble(
    surah_meta: Vec<SurahInfoAlt>,
    uthmani_map: HashMap<(u32, u32), String>,
    simple_map: HashMap<(u32, u32), String>,
    meta_map: MetaMap,
) -> Result<QuranData> {
    let mut surahs = Vec::with_capacity(114);
    let mut ayahs  = Vec::new();

    // Sort surah meta by number
    let mut sorted_meta = surah_meta;
    sorted_meta.sort_by_key(|s| s.number);

    for sm in &sorted_meta {
        let sid = sm.number;

        // Extract fields with fallbacks
        let name_ar = sm.name_arabic
            .clone()
            .or_else(|| sm.name.clone())
            .unwrap_or_else(|| format!("سورة {sid}"));

        let name_en = sm.name_english
            .clone()
            .or_else(|| sm.english_name.clone())
            .unwrap_or_else(|| format!("Surah {sid}"));

        let transliteration = sm.name_trans
            .clone()
            .or_else(|| sm.transliteration.clone())
            .unwrap_or_else(|| name_en.clone());

        let revelation_type_raw = sm.revelation_type
            .clone()
            .unwrap_or_else(|| "Makki".to_string());
        let revelation_type = normalize_revelation_type(&revelation_type_raw);

        let verses_count = sm.verses_count
            .or(sm.number_of_ayahs)
            .unwrap_or(0);

        let order_of_revelation = sm.order_of_revelation
            .or(sm.revelation_order)
            .unwrap_or_else(|| revelation_order(sid));

        let has_bismillah = sid != 9; // Surah 9 has no Bismillah

        surahs.push(Surah {
            id: sid,
            name_ar,
            name_en,
            transliteration,
            revelation_type,
            verses_count,
            order_of_revelation,
            has_bismillah,
        });

        // Collect ayahs for this surah
        for ayah_num in 1..=verses_count {
            let key = (sid, ayah_num);

            let uthmani_text = uthmani_map.get(&key)
                .cloned()
                .unwrap_or_else(|| {
                    log::warn!("Missing uthmani text for {sid}:{ayah_num}");
                    String::new()
                });

            let simple_text = simple_map.get(&key)
                .cloned()
                .unwrap_or_else(|| {
                    log::warn!("Missing simple text for {sid}:{ayah_num}");
                    String::new()
                });

            let meta = meta_map.get(&key);

            let (juz, hizb, rub_hizb, manzil, ruku, page, sajdah) = match meta {
                Some(m) => (m.juz, m.hizb, m.rub_hizb, m.manzil, m.ruku, m.page, m.sajdah),
                None => {
                    log::warn!("Missing metadata for {sid}:{ayah_num} — using defaults");
                    (1, 1, 1, 1, 1, 1, false)
                }
            };

            ayahs.push(Ayah {
                surah_id: sid,
                ayah_number: ayah_num,
                uthmani_text,
                simple_text,
                juz,
                hizb,
                rub_hizb,
                manzil,
                ruku,
                page,
                sajdah,
            });
        }
    }

    Ok(QuranData { surahs, ayahs })
}
