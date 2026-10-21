// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // WebKitGTK's network stack (libsoup) multiplexes every request to a host
    // over one HTTP/2 connection. The listen panel's stream leaves its first
    // download suspended once enough is buffered, and a jump in the Surah opens
    // a second one beside it — which then gets one flow-control window, about
    // 64 KB or three seconds of audio, and nothing after. HTTP/1.1 gives each
    // request its own connection. Set before any webview exists, so the
    // network process inherits it.
    #[cfg(target_os = "linux")]
    if std::env::var_os("SOUP_FORCE_HTTP1").is_none() {
        std::env::set_var("SOUP_FORCE_HTTP1", "1");
    }
    quran_reader_lib::run()
}
