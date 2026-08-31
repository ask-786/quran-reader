# Audio — Plan (Phase 12)

**Date drafted:** 2026-08-31
**Status:** implemented. Steps 1–5 of the phasing below are in; the reciter
verification sweep, word-level highlighting and the WebKitGTK playback check
are not. See PLAN.md Phase 12 for what is still open.
**Governing doc:** `PLAN.md` Phase 12 (Play Surah, Play Ayah, Repeat Ayah,
Auto-scroll, Offline audio).
**Goal:** recitation in both reading views without adding a single byte to the
installer, and without the app phoning anywhere until the reader asks it to.

## TL;DR — the decisions

| Question           | Decision                                                                                                                    |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------- |
| Bundled in the app | **No.** Phase 13 already lists a 100 MB installer as a problem. Audio would multiply it                                     |
| Shipped as packs   | **No.** The tafsir `.qpack` route means hosting and relicensing 500 MB per reciter. Rejected                                |
| Delivery           | **Fetched per Ayah on play, cached on disk forever.** Offline audio is then a consequence, not a feature                    |
| Source             | `cdn.islamic.network`, keyed by **`ayah.id` exactly**. `everyayah.com` is the second host for the same recordings           |
| Granularity        | **Per Ayah.** Whole-Surah files exist and al-Baqara's is 121 MB, which is not a thing to fetch when someone presses play    |
| Bitrate            | **64 kbps default, 128 kbps offered.** 32, 48 and 192 return 403 on that host; there is no low-bandwidth tier to offer      |
| Verification       | No compiled SHA-256 is possible. Content type, magic bytes, size bounds, atomic rename, and audio never touching the DB     |
| Playback           | `<audio>` in the webview over Tauri's asset protocol. No Rust audio stack, no ALSA/CoreAudio matrix in the release workflow |
| Cache location     | `<data_dir>/audio/<reciter_slug>/<bitrate>/<ayah_id>.mp3`                                                                   |
| Cache policy       | Never evicted automatically. The reader clears it, per reciter or entirely, from the new Audio section in settings          |
| Word timings       | Table defined in migration 009 and left empty. Word-by-word highlighting is a later phase                                   |
| The `Space` key    | Goes to playback. Auto-scroll keeps `a`. See "The key collision" below                                                      |

---

# Part 1 — Where the recitation comes from

Every figure here was measured against the live hosts on 2026-08-31, not taken
from documentation.

## `ayah.id` is already the right key

`cdn.islamic.network/quran/audio/{bitrate}/{reciter}/{n}.mp3` numbers verses
1 to 6236 in Mushaf order, which is what `ayah.id` has been since migration 001.
`262.mp3` is 2:255 on the CDN and `id = 262` is 2:255 in `quran.db`. So the
audio layer needs no mapping table, no zero-padded Surah:Ayah arithmetic, and
no new column. The playlist for any view is the `ayah.id` list the route has
already loaded.

`everyayah.com/data/{reciter}_{bitrate}kbps/{SSSAAA}.mp3` carries the same
recordings under a padded Surah/Ayah name. 2:255 is 417,111 bytes on one host
and 416,704 on the other, which is the same recitation with a different
encoder pass. Worth keeping as a fallback host, and worth the small mapping
cost when a reciter exists on one and not the other.

## What is actually available

| Probe                                                                              | Result                                     |
| ---------------------------------------------------------------------------------- | ------------------------------------------ |
| `audio/64/ar.alafasy/1.mp3`                                                        | 200, 49,513 bytes                          |
| `audio/64/ar.alafasy/262.mp3` (2:255)                                              | 200, 417,111 bytes                         |
| `audio/128/ar.alafasy/1.mp3`                                                       | 200, 146,830 bytes, `accept-ranges: bytes` |
| `audio/32`, `audio/48`, `audio/192`                                                | 403                                        |
| `audio-surah/128/ar.alafasy/2.mp3`                                                 | 200, **121,804,776 bytes**                 |
| `ar.abdulbasitmurattal`, `ar.husary`, `ar.shaatree`, `ar.abdurrahmaansudais` at 64 | 200                                        |
| `ar.minshawi` at 64                                                                | **403**                                    |

Three things follow.

An Ayah is 50 KB to 420 KB at 64 kbps. That is small enough that fetching one
on demand is not a wait worth designing around, and the whole Quran lands at
roughly half a gigabyte for anyone who eventually plays all of it.

The Surah-level files rule themselves out. 121 MB for one Surah, with no Ayah
boundaries in it, would need a segment table before a single verse could be
highlighted or repeated. Per-Ayah files give Play Ayah, Repeat Ayah and
follow-highlighting with no timing data at all.

**Reciter availability is not uniform and cannot be assumed.** `ar.minshawi` is
a plausible identifier that 403s. A reciter enters the catalogue only after
every one of the 6,236 ids has been checked, which is what
`quran-importer --verify-reciter <slug>` is for. Finding this at release time
costs one run from a maintainer's machine. Finding it at read time looks like
the app breaking in the middle of a Surah.

## Licensing

Fetching from someone else's CDN does not make a recording free. These are
copyrighted performances, and neither alquran.cloud's nor everyayah's terms
have been read. That is the same standing item as the Jalālayn translation in
Phase 11, and it is recorded per reciter in the catalogue rather than assumed
away. What this design does avoid is _redistribution_: the app never hosts,
repackages or ships a recording.

---

# Part 2 — Delivery

## Fetch on play, cache forever, play only from cache

The player never points `<audio>` at a remote URL. It asks the cache for Ayah
_n_; if the file is missing, the Rust side fetches it with `ureq` (already a
dependency, already used by `packs`), writes it to
`<data_dir>/audio/<slug>/<bitrate>/<ayah_id>.mp3.part`, and renames it into
place. Playback then runs from the local file through `convertFileSrc`.

One code path, whether the file arrived a second ago or last month. A
prefetcher keeps three to five Ayahs ahead of the playhead, so only the first
press of play ever waits on the network, and at 50 to 420 KB that wait is a
fraction of a second on anything usable.

Two things fall out of this for free.

**Offline audio is earned rather than declared.** What you have listened to,
you keep. A reader who works through al-Kahf on Friday has al-Kahf on the
plane, with no download step they had to think about in advance.

**"Download this Surah" is the same fetcher, run eagerly.** It reuses the
progress-event shape the tafsir packs already emit. There is no second
download mechanism, no archive format, no unpacking step.

## Why not the pack machinery

The `.qpack` route works for tafsir because the files are small, the texts are
mostly public domain, and a hash compiled into the binary makes the download
tamper-evident. None of that survives contact with audio. 500 MB per reciter
would have to live on the releases page, which means hosting recordings the
project has no licence to redistribute, and paying GitHub's bandwidth for
every reciter anyone installs. Fetching from the CDN that already serves them
avoids the whole question.

---

# Part 3 — What replaces the SHA-256

`packs/mod.rs` states the rule the app has followed so far: nothing about a
downloaded file is assumed, and the bytes are hashed against a constant in the
binary before SQLite is allowed near them. That guarantee is not available
here. The bytes come from a host that can re-encode its library at any time,
and there is no published manifest to compile in.

So the guarantee changes, and the reason it is acceptable is structural rather
than a matter of degree.

**A bad tafsir pack is invisible. A bad mp3 is audible.** A pack that installed
wrong content would attach commentary to the wrong verses inside the user's own
database, silently, permanently, and indistinguishably from real content. Audio
is never merged into the database and never executed. It is a file in a
directory that the reader hears the moment it plays, and deleting it costs
nothing.

What the fetcher checks anyway:

- HTTPS only, and redirects are refused outright rather than followed and
  re-checked. This build knows one host and one URL shape; a 3xx from it is a
  change worth failing loudly on rather than chasing.
- `content-type` must be an audio type, and the first bytes must be an MP3
  frame header or an ID3 tag. An HTML error page saved as `262.mp3` is the
  common failure here, and it is worth catching before it reaches the decoder.
- Size bounds. Zero-length responses and anything past a few megabytes for one
  Ayah are refused.
- The file is written to `.part` and renamed only when complete, so a killed
  process or a dropped connection never leaves a truncated file that later
  looks cached.

---

# Part 4 — Privacy

This is the first feature where reading the Quran produces outbound network
traffic, and it deserves saying plainly: the pattern of those requests tells
the CDN which verses someone is reading and roughly when. The README currently
promises no network calls and no telemetry. Audio does not have to break that
promise, but it does have to change the sentence.

Non-negotiable in the implementation:

- Nothing is fetched until the reader has chosen a reciter. There is no default
  reciter, and no request at startup or on navigation.
- The first fetch asks, once, naming the host it is about to contact.
- No query parameters, no identifiers, no analytics. A plain request for a
  file.
- A "cached only" switch that stops the network entirely, so a reader who
  downloaded a Juz before a trip can be certain nothing else goes out.
- The Data section's note gets updated. The honest version is that the app
  makes no network request while reading, and fetches tafsir and audio only
  when asked.

---

# Part 5 — Schema (migration 009)

`CURRENT_VERSION` 8 → 9. Three tables and a handful of settings keys.

**`reciter`** mirrors the shape of `tafsir`: id, slug, `name_ar`, `name_en`,
`riwaya`, `style` (murattal or mujawwad), `source_url`, `license`,
`sort_order`. Populated from the compiled catalogue when the database opens,
upserted by slug, so adding a reciter is an app release rather than a
migration.

The riwāya is on the record for the same reason a tafsir carries its school and
creed. Nearly every recording in circulation is Ḥafṣ ʿan ʿĀṣim, which is
exactly why the one that is not should be labelled rather than left for the
reader to notice halfway through.

**`audio_file`** is (reciter_id, bitrate, ayah_id, bytes, fetched_at). It is
not the source of truth about the disk, but it turns "is this Surah
downloaded?" and "how much space is audio using?" into one query instead of
6,236 `stat` calls, and it is what the settings section counts. It is rebuilt
from a directory scan if it ever disagrees with the filesystem.

**`recitation_segment`** is (reciter_id, ayah_id, word_index, start_ms,
end_ms), and ships empty. Word-by-word highlighting needs it, `page_line_word`
already carries `word_index` to join against, and defining the table now costs
nothing but saves a migration later.

New settings keys, all with defaults in `schema.sql` and in `load_settings`,
which is the pattern `tafsir_view` established: `reciter_id`, `audio_bitrate`,
`audio_repeat_mode`, `audio_repeat_count`, `audio_playback_rate`,
`audio_follow`, `audio_volume`, `audio_downloads_allowed`.

---

# Part 6 — Playback

## The webview plays it, not Rust

`rodio` and `cpal` would pull `libasound2-dev` into the build and an
ALSA/PulseAudio/CoreAudio/WASAPI matrix into a release workflow that currently
builds seven bundles green without thinking about sound. The webview already
decodes and plays audio. Enable `assetProtocol` in `tauri.conf.json` scoped to
the audio directory, add `core:asset:default` to `capabilities/default.json`,
and hand `<audio>` a `convertFileSrc` URL.

**The risk worth testing early:** WebKitGTK decodes media through GStreamer, so
MP3 playback depends on `gst-plugins-good` being present on the user's machine.
That is fine on a normal desktop Fedora or Ubuntu and it is an open question
inside the AppImage. It becomes a `Depends:` line for the deb and rpm, and it
should be smoke-tested in a minimal container before Phase 12 goes further than
step 1, because the fallback (decoding in Rust with `symphonia`) is a different
design, not a tweak.

## Gapless

Two `<audio>` elements, ping-ponged. Element B preloads Ayah _n+1_ while A
plays _n_, and the `ended` handler swaps them. If the seam is audible on
WebKitGTK, the escape hatch is Web Audio with `decodeAudioData` and scheduled
`start(when)`, which is sample-accurate, at the cost of holding decoded PCM in
memory.

## The queue is the scope

`playback.svelte.ts` holds the current Ayah id, playing state, the queue, the
repeat policy and the rate, in the shape of `auto-scroll.svelte.ts`. The queue
is the ayah id list of whatever range is open, which `reading-scope.ts` already
knows, so Play Surah, Play Juz, Play Page and "play from this verse" are one
code path with different bounds.

Repeat is the feature this whole phase exists for, and it is the memoriser's
feature: repeat one Ayah _n_ times, repeat a range, with an optional pause
between repetitions long enough to say it back.

---

# Part 7 — Where it lives

> **Revised twice.** The first pass put a play button on every Ayah row and a
> transport bar across the foot of the reader. That was wrong, and wrong in a
> way worth writing down: it made the app look like a player that also shows
> text. Reading is the app. Recitation is what you reach for when a word does
> not sit right in your mouth, and it belongs where the other "wait, what is
> this" answers already live.

## One verse: the verse card

> **Renamed.** Once recitation moved in, "tafsir card" was no longer what the
> thing was: it answers two different questions about a verse, and
> `docs/tafsir-popover-plan.md` reserves word-level actions (root, morphology)
> for the same surface later. It is the **verse card** in the UI, and `t` and
> the toolbar button turn **verse cards** on rather than "tafsir mode". One
> surface for everything about a verse beats a surface per kind of answer —
> the alternative was a second panel and a second key for the same gesture.
>
> The settings key stays `tafsir_click`. It predates the rename, and renaming
> it would discard the preference of everyone who already set it in exchange
> for a name only the store sees.

`TafsirAudioRow` sits under the header of the tafsir popover and the tafsir
panel, above the commentary. The card is already open on that āya and already
answers "what is going on here"; "how is this said" is the same reader stopping
for the same reason. It plays **that verse and stops** — running on into the next
one is a different intent and gets a different surface.

**It is a bar, not a play button, and that is not decoration.** A verse is not a
uniform unit of time: al-Fatiha 1 is about five seconds, 2:282 runs for minutes.
Someone who stopped to check one phrase in the longest āya in the Quran should be
able to reach that phrase. So the row is play/pause, a seek track and a running
time, driven by `currentTime`/`duration` on the store — three things on one line.
Skip-back and skip-forward buttons were there for one revision and came out:
five icons in a strip that sits above the commentary is a control panel, and the
track already goes everywhere they went.

**The card's height budget counts it.** `place()` in `TafsirPopover` used to cap
the whole card at 45% of the viewport, measuring `head + body`. The strip was not
in that sum, so the card kept its old height and the row's ~39px came straight
out of the commentary — the tafsir visibly shrank the day audio arrived. The cap
is now on the _text_: chrome (header + strip) is measured and added on top, so
the commentary keeps exactly the room it had. A `ResizeObserver` on the strip
re-runs the placement, since it grows a line when it has something to say.

Before a reciter is chosen the row is a single line offering to pick one, which
is also where fetching is explained and turned on. Without that, recitation is
invisible until someone goes looking in Settings for a feature they have no
reason to believe exists.

**The card has to work with no commentary installed at all**, because none
ships: every edition is a download. So a fresh install opening a verse gets a
working audio strip above a body that says so and offers the shelf, rather than
an empty card that makes recitation look broken.

## A whole Surah: the listen panel, opened from the banner

The Surah banner carries a faint **Listen** chip under the subtitle. The banner
is chrome announcing which Surah this is, so "hear this Surah" is a statement
about the same thing, and putting it there keeps every transport control out of
the reading column.

It opens `ListenPanel`, a card in the bottom corner: the Surah and verse playing,
the same scrubber, previous/next, the repeat cycle, and the button that downloads
the whole Surah for offline listening with progress and cancel. It steps aside
for the tafsir panel through `--tafsir-inset`, like the reader's other corner
controls, and closing it stops the recitation — a sound with no visible cause is
worse than no sound.

Two rules make it acceptable over the reader at all: it is **never there unless
it was asked for**, and it is **not persisted**, so the app never comes back
wearing it.

**It buffers five verses ahead, one at a time.** Roughly a minute of recitation
of slack, so a slow moment on the network is absorbed between verses instead of
heard as a gap. Sequential rather than parallel: five simultaneous requests at
every verse boundary is a burst against someone else's CDN for files this project
does not host, and they would compete with the verse actually being waited on. A
run is abandoned as soon as the playhead moves, so skipping forward does not
leave the app fetching verses the reader has gone past.

**While it is open, it owns the queue.** The reader normally keeps the queue
pointed at the open range, which is what lets a verse played from the tafsir card
know where it sits. That stops while the panel is up: someone listening to
Al-Kahf who turns to check a verse elsewhere has not asked for the recitation to
stop.

**Closing the card stops the verse.** The recitation belongs to the surface that
started it, so when the last commentary surface goes — Escape, the close button,
a click away — playback and the mark on the verse go with it. The same is true
when a single verse simply finishes: the mark means "this is the one sounding",
and leaving it on a verse that has ended is a highlight with nothing behind it.
A range from the listen panel is untouched by any of this; it has its own surface
and its own stop.

## Nothing in the reading surface

No button on the Ayah row and no bar over the text.

`Space` is the one key that means two things, and it is still one idea: start or
stop whatever is moving. With a verse card open, that is the recitation of its
verse; with the reader alone on screen, it is the page. The card is a
deliberate, visible state, so which one the key means is never a guess — and
with no reciter chosen, or a panel open on nothing, it falls back to auto-scroll
rather than doing nothing at all. The playing verse is still
marked — a tint and a leading rule on the row, a background on the words in the
Mushaf view — which is orientation rather than chrome, and costs nothing when
nothing is playing.

Follow-scroll applies to a range only (`playbackStore.shouldFollow`). A verse
played from the card is already on screen, so the reader is left where it is.

# Phasing

1. **Groundwork.** Migration 009, the compiled reciter catalogue, the fetcher
   with its checks, the asset protocol, and one Ayah playing from cache. No UI
   beyond a play button. This proves the WebKitGTK/GStreamer question before
   anything is built on top of it.
2. **Play Ayah, Play Surah.** The queue, the prefetcher, the ping-pong
   elements, the audio bar.
3. **Repeat Ayah.** Counts, ranges, the pause between.
4. **Follow-scroll.** The interlock with auto-scroll, in both views.
5. **Offline audio.** Eager download by Surah and Juz with progress, the Audio
   settings section, storage accounting and cache clearing.
6. **Later, not this phase.** Word-level highlighting once `recitation_segment`
   has data. A second host as fallback. Resume for eager downloads, which
   `accept-ranges: bytes` already permits.

# What the first run taught

Two things, both worth keeping written down.

**The asset protocol cannot play media on Linux.** Not a scope problem, not a
permission problem: `convertFileSrc` yields `asset://localhost/…` and WebKitGTK's
media pipeline refuses custom schemes outright. The same handler serves `fetch`
and `<img>` perfectly. The failure surfaces as `NotSupportedError: The operation
is not supported.`, which names nothing and points at nothing.

**Putting audio in the reader made it a different app.** It was one bar and one
button per verse, it passed every check, and it was wrong. Where a control lives
is a claim about what the software is for.

# Open questions

- **Which reciters ship in the catalogue?** Five are verified reachable. Each
  one added needs the full 6,236-id sweep first.
- **Do bitrate changes re-fetch?** Currently the cache is keyed by bitrate, so
  switching from 64 to 128 starts a second cache rather than replacing the
  first. That is honest but it can surprise someone watching the storage
  number.
- **What happens when the network dies mid-Surah?** The decision here is to
  pause with a plain statement of why, not to skip ahead to whatever happens to
  be cached. Skipping verses in the Quran to keep audio flowing is the wrong
  trade.
