/// Raw data downloaded from Tanzil and external sources.
pub struct RawData {
    /// Tanzil Uthmani script XML (`quran-uthmani.xml`)
    pub uthmani_xml: String,
    /// Tanzil simple (undiacritized) Arabic XML (`quran-simple.xml`)
    pub simple_xml: String,
    /// Surah metadata JSON (name_ar, name_en, transliteration, revelation_type, etc.)
    pub surah_meta_json: String,
}

// ---------------------------------------------------------------------------
// Tanzil canonical download URLs
// ---------------------------------------------------------------------------
// The text files are served directly from tanzil.net. We use the plain-text
// XML edition (UTF-8, no BOM). These URLs are stable and well-known.

const UTHMANI_URL: &str =
    "https://tanzil.net/pub/download/index.php?quranType=uthmani&outType=xml&agree=true";

const SIMPLE_URL: &str =
    "https://tanzil.net/pub/download/index.php?quranType=simple&outType=xml&agree=true";

/// We host surah metadata as a bundled JSON (derived from Tanzil quran-data.js).
/// The data is small (114 records) and rarely changes.
const SURAH_META_URL: &str =
    "https://raw.githubusercontent.com/spa5k/quran_data/main/surahs/surahInfo.json";

/// Fetch all required raw data sources.
pub fn fetch_all() -> anyhow::Result<RawData> {
    log::info!("  → Downloading Uthmani XML …");
    let uthmani_xml = get(UTHMANI_URL)?;
    log::info!("      {} bytes", uthmani_xml.len());

    log::info!("  → Downloading Simple Arabic XML …");
    let simple_xml = get(SIMPLE_URL)?;
    log::info!("      {} bytes", simple_xml.len());

    log::info!("  → Downloading Surah metadata JSON …");
    let surah_meta_json = get(SURAH_META_URL)?;
    log::info!("      {} bytes", surah_meta_json.len());

    Ok(RawData {
        uthmani_xml,
        simple_xml,
        surah_meta_json,
    })
}

/// Perform a blocking HTTP GET and return the body as a String.
/// Public variant used by other modules (e.g. parse.rs fetching alquran.cloud).
pub fn get_pub(url: &str) -> anyhow::Result<String> {
    get(url)
}

/// Perform a blocking HTTP GET and return the body as a String.
fn get(url: &str) -> anyhow::Result<String> {
    let response = ureq::get(url)
        .call()
        .map_err(|e| anyhow::anyhow!("GET {url} failed: {e}"))?;

    let body = response
        .into_string()
        .map_err(|e| anyhow::anyhow!("Reading body from {url} failed: {e}"))?;

    Ok(body)
}
