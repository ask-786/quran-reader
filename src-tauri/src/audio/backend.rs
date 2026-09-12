//! Whether this machine can play a verse at all.
//!
//! Recitation is decoded and played by the webview, and on Linux the webview is
//! WebKitGTK, which hands media to GStreamer. GStreamer's elements are separate
//! system packages: WebKitGTK depends on the *libraries* but not on the
//! elements, so an ordinary Arch install carrying `webkit2gtk-4.1` and nothing
//! else has no `autoaudiosink` and no MP3 decoder.
//!
//! That would be an ordinary recoverable error if it merely failed. It does
//! not. With no sink to build, `HTMLAudioElement.play()` wedges the
//! WebKitWebProcess: the promise never settles, no `error` event is fired, and
//! the window stops responding — the whole reader, not just the player. Nothing
//! in the frontend can catch it, because the frontend is the thing that stops
//! running, and a timeout cannot save it either since the timer dies with the
//! process.
//!
//! So the question has to be asked *before* an `<audio>` element is ever handed
//! a source, out here in the Rust process, which is still alive and still able
//! to say what is wrong. `playback.svelte.ts` refuses to load a verse when this
//! reports unplayable.
//!
//! # Why the filesystem and not GStreamer itself
//!
//! Asking GStreamer would mean linking it, which puts its development headers
//! into the build matrix for seven bundles in order to answer one yes/no
//! question — and docs/audio-plan.md chose the webview precisely to keep audio
//! libraries out of this build. The plugin registry is a directory of
//! `libgst<plugin>.so` files, so looking for the files gives the same answer
//! for the cost of a `stat`.

use serde::Serialize;

/// What the reader is told when recitation cannot work.
///
/// `missing` is phrased for a person rather than listing element names, and
/// `install_command` is a line they can paste. A message that says only
/// "audio unavailable" sends someone to a bug tracker for what is one
/// `pacman -S` away.
#[derive(Debug, Clone, Serialize)]
pub struct AudioBackend {
    /// False only when playing would hang the webview. A missing decoder is
    /// reported without clearing this: that failure is survivable and already
    /// surfaces through the element's `error` event.
    pub playable: bool,
    pub missing: Vec<String>,
    pub install_command: Option<String>,
}

/// Anywhere but Linux the media stack ships with the OS and there is nothing
/// to be missing.
#[cfg(not(target_os = "linux"))]
pub fn probe() -> AudioBackend {
    AudioBackend {
        playable: true,
        missing: Vec::new(),
        install_command: None,
    }
}

#[cfg(target_os = "linux")]
pub use linux::probe;

#[cfg(target_os = "linux")]
mod linux {
    use super::AudioBackend;
    use std::path::{Path, PathBuf};

    /// One thing the pipeline needs, and the plugin files that can provide it.
    struct Requirement {
        /// Alternatives, not a list: any one of these satisfies the
        /// requirement. `autoaudiosink` is happy with whichever output it
        /// finds, and either decoder plays an MP3.
        files: &'static [&'static str],
        /// Named the way the reader would describe it, not the way GStreamer
        /// registers it.
        label: &'static str,
        /// Whether its absence is the kind that hangs the webview rather than
        /// merely failing.
        fatal: bool,
    }

    const REQUIREMENTS: &[Requirement] = &[
        Requirement {
            files: &["libgstautodetect.so"],
            label: "an audio sink (GStreamer autoaudiosink)",
            fatal: true,
        },
        Requirement {
            files: &["libgstpulseaudio.so", "libgstpipewire.so", "libgstalsa.so"],
            label: "an audio output (PulseAudio, PipeWire or ALSA)",
            fatal: true,
        },
        Requirement {
            files: &["libgstmpg123.so", "libgstlibav.so"],
            label: "an MP3 decoder",
            fatal: false,
        },
    ];

    pub fn probe() -> AudioBackend {
        let dirs = plugin_dirs();
        let mut missing = Vec::new();
        let mut playable = true;

        for requirement in REQUIREMENTS {
            let found = requirement
                .files
                .iter()
                .any(|file| dirs.iter().any(|dir| dir.join(file).exists()));
            if found {
                continue;
            }
            missing.push(requirement.label.to_string());
            if requirement.fatal {
                playable = false;
            }
        }

        AudioBackend {
            install_command: if missing.is_empty() {
                None
            } else {
                install_command()
            },
            playable,
            missing,
        }
    }

    /// Where GStreamer would look for its plugins.
    ///
    /// `GST_PLUGIN_SYSTEM_PATH` *replaces* the compiled-in system directories
    /// rather than adding to them, and that is not a detail to paper over: it
    /// is exactly what a bundle sets to point at its own copies. Searching
    /// `/usr/lib` anyway would find the host's plugins and report a bundle
    /// playable on the strength of files it will never load.
    fn plugin_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        let system = env_paths("GST_PLUGIN_SYSTEM_PATH_1_0")
            .or_else(|| env_paths("GST_PLUGIN_SYSTEM_PATH"));
        match system {
            Some(paths) => dirs.extend(paths),
            None => {
                for base in [
                    "/usr/lib",
                    "/usr/lib64",
                    "/usr/local/lib",
                    "/usr/lib/x86_64-linux-gnu",
                    "/usr/lib/aarch64-linux-gnu",
                ] {
                    dirs.push(Path::new(base).join("gstreamer-1.0"));
                }
                if let Some(home) = std::env::var_os("HOME") {
                    dirs.push(Path::new(&home).join(".local/share/gstreamer-1.0/plugins"));
                }
            }
        }

        // Additive in every case, including alongside a replaced system path.
        for var in ["GST_PLUGIN_PATH_1_0", "GST_PLUGIN_PATH"] {
            if let Some(paths) = env_paths(var) {
                dirs.extend(paths);
            }
        }

        dirs
    }

    fn env_paths(var: &str) -> Option<Vec<PathBuf>> {
        let value = std::env::var_os(var)?;
        if value.is_empty() {
            return None;
        }
        Some(std::env::split_paths(&value).collect())
    }

    /// The line to paste, for distributions whose package names we know.
    ///
    /// `ID_LIKE` is read as well as `ID`, so the derivatives — and there are a
    /// great many of them — get an answer their package manager understands
    /// without this list having to name every one.
    fn install_command() -> Option<String> {
        let os_release = std::fs::read_to_string("/etc/os-release").ok()?;
        let mut ids = Vec::new();
        for line in os_release.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            if key != "ID" && key != "ID_LIKE" {
                continue;
            }
            let value = value.trim().trim_matches('"');
            ids.extend(value.split_whitespace().map(str::to_ascii_lowercase));
        }

        ids.iter().find_map(|id| {
            let command = match id.as_str() {
                "arch" | "archlinux" => "sudo pacman -S gst-plugins-good gst-plugins-base",
                "debian" | "ubuntu" => {
                    "sudo apt install gstreamer1.0-plugins-good gstreamer1.0-plugins-base"
                }
                "fedora" | "rhel" | "centos" => {
                    "sudo dnf install gstreamer1-plugins-good gstreamer1-plugins-base"
                }
                "opensuse" | "suse" => {
                    "sudo zypper install gstreamer-plugins-good gstreamer-plugins-base"
                }
                "alpine" => "sudo apk add gst-plugins-good gst-plugins-base",
                _ => return None,
            };
            Some(command.to_string())
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn scratch(name: &str) -> PathBuf {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let dir = std::env::temp_dir().join(format!(
                "quranreader-gst-{name}-{nonce}-{}",
                std::process::id()
            ));
            std::fs::create_dir_all(&dir).unwrap();
            dir
        }

        fn plugin(dir: &PathBuf, file: &str) {
            std::fs::write(dir.join(file), b"").unwrap();
        }

        /// One test and not three: these set process-wide environment
        /// variables, and `cargo test` runs test functions on parallel threads.
        ///
        /// `GST_PLUGIN_SYSTEM_PATH` is what makes this testable at all — it is
        /// the same lever that reproduces the Arch failure on a machine whose
        /// audio works, by pointing GStreamer at a directory with nothing in it.
        #[test]
        fn reports_missing_pieces_and_keeps_survivable_gaps_playable() {
            std::env::remove_var("GST_PLUGIN_SYSTEM_PATH_1_0");
            std::env::remove_var("GST_PLUGIN_PATH_1_0");
            std::env::remove_var("GST_PLUGIN_PATH");

            // Nothing at all: the state the reader's Arch box was in.
            let empty = scratch("empty");
            std::env::set_var("GST_PLUGIN_SYSTEM_PATH", &empty);
            let probed = probe();
            assert!(!probed.playable);
            assert_eq!(probed.missing.len(), REQUIREMENTS.len());

            // A sink and an output but no decoder. That failure is survivable —
            // it raises `error` on the element instead of hanging — so it is
            // reported without taking recitation away.
            let partial = scratch("partial");
            plugin(&partial, "libgstautodetect.so");
            plugin(&partial, "libgstalsa.so");
            std::env::set_var("GST_PLUGIN_SYSTEM_PATH", &partial);
            let probed = probe();
            assert!(probed.playable);
            assert_eq!(probed.missing, vec!["an MP3 decoder".to_string()]);

            // GST_PLUGIN_PATH adds to a replaced system path rather than being
            // dropped alongside it.
            let extra = scratch("extra");
            plugin(&extra, "libgstmpg123.so");
            std::env::set_var("GST_PLUGIN_PATH", &extra);
            let probed = probe();
            assert!(probed.playable);
            assert!(probed.missing.is_empty());
            assert!(probed.install_command.is_none());

            std::env::remove_var("GST_PLUGIN_SYSTEM_PATH");
            std::env::remove_var("GST_PLUGIN_PATH");
            for dir in [empty, partial, extra] {
                let _ = std::fs::remove_dir_all(dir);
            }
        }
    }
}
