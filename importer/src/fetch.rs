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

/// Fetch all required raw data sources.
pub fn fetch_all() -> anyhow::Result<RawData> {
    log::info!("  → Downloading Uthmani XML …");
    let uthmani_xml = get(UTHMANI_URL)?;
    log::info!("      {} bytes", uthmani_xml.len());

    log::info!("  → Downloading Simple Arabic XML …");
    let simple_xml = get(SIMPLE_URL)?;
    log::info!("      {} bytes", simple_xml.len());

    // alquran.cloud /v1/surah returns all 114 surahs with name (Arabic),
    // englishName, numberOfAyahs, and revelationType — no third-party repo needed.
    log::info!("  → Downloading Surah metadata (alquran.cloud) …");
    let surah_meta_json = get("https://api.alquran.cloud/v1/surah")?;
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
    let mut response = ureq::get(url)
        .call()
        .map_err(|e| anyhow::anyhow!("GET {url} failed: {e}"))?;

    // read_to_string caps the body at 10 MB and errors past it rather than
    // truncating. The largest source here is the ~1.5 MB Tanzil XML.
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|e| anyhow::anyhow!("Reading body from {url} failed: {e}"))?;

    Ok(body)
}
