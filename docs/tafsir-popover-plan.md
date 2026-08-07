# Tafsir popover — Plan

**Date drafted:** 2026-08-07
**Status:** plan, written before the implementation in the same PR.
**Governing doc:** `docs/translation-tafsir-plan.md` (edition choice, the
Shāfiʿī/Ashʿarī filter, checklist item 11.3). This one covers only how tafsir
is surfaced in the reader.

## The two problems this fixes

1. **No control over which āya.** The drawer shows whichever Ayah an
   `IntersectionObserver` decides is centred (`PageView.svelte`'s
   `observeCenteredAyah` → `readerPosition.ayahId`, and the equivalent in
   `ReaderView`). A pin exists — `openForAyah` / `syncPosition` — but it lasts
   only until the reader scrolls, so "show me _this_ verse and keep it" is not
   expressible. In Mushaf view the centred line is a guess, and often not the
   line being read.
2. **It moves the page.** `ReaderPage.svelte` mounts `<TafsirPanel>` as a
   sibling of `.reader-main`, so opening it takes width from the reader. The
   scrolling view reflows every line; the Mushaf view reflows glyph lines whose
   entire purpose is matching the printed page. Commentary should never disturb
   the text it comments on.

## What replaces it

Click an āya → a popover anchored to that āya, showing its tafsir. Overlay in a
fixed-position portal, so the reader's layout cannot change. The āya you
clicked is the āya you get, and it stays until you dismiss it.

---

# Decisions

## Keep the drawer, demote it — agreed

The popover becomes the default surface; the drawer stays, reachable from a
button in the popover, and the choice persists. The drawer's code is gated, not
deleted.

**Measured, not assumed.** Gloss lengths in the two bundled editions:

| Edition         | median | p90 | p99   | max   | under 400 chars |
| --------------- | ------ | --- | ----- | ----- | --------------- |
| English (Hamza) | 274    | 703 | 1,566 | 7,219 | 68%             |
| Arabic          | 183    | 462 | 1,016 | 4,608 | 85%             |

So a popover holding ~700 characters comfortably covers 90% of both editions,
and the drawer earns its place on the tail — 2:255's English gloss is 2.3 KB,
and the shelf in plan item 11.5 (al-Rāzī ≈90 MB, al-Baghawī ≈41 MB) is
essay-length by nature. Two surfaces for two shapes of text, rather than one
that is wrong for half of them.

## Trigger: click the āya text, in both views — agreed

- **Scrolling reader** — click anywhere on `.ayah-text`. The existing per-āya
  button (`AyahRow`) stays as the discoverable affordance; it is the only
  visible hint that tafsir exists at all.
- **Mushaf view** — click a word. Word spans already carry `data-ayah-id`, and
  the line carries `data-line-ayah-id` for basmala rows whose words have no
  Ayah. This closes plan item 11.3's word-level gap.

**What a word click means, and the corner not to paint into.** A plain click on
a word means "tafsir for the āya this word belongs to". Word-level actions
(root, morphology, per-word translation) are a real future feature, and they
need a _different_ gesture, not this one — plain click is the discoverable
default and belongs to the commonest action. Reserved for them: context menu
(right-click) and long-press, both currently unused in the reader. So that the
future action does not need re-plumbing, the word spans also gain
`data-word-index`, which they do not carry today even though `page_line_word`
has the column. Nothing reads it yet; it is one attribute, and adding it later
would mean touching the same hot render path again.

## Positioning: fixed portal, flip, clamp, arrow

- Rendered into a portal on `document.body`, `position: fixed`, so no ancestor
  of the reader participates in its layout.
- Anchored from the trigger's `getBoundingClientRect()`. Preferred side is
  below; flips above when below lacks room. **It never overlaps the anchor
  rect** — if neither side fits the preferred height, it takes the larger side
  and shrinks `max-height` to what is available. That is the whole point of the
  feature and is enforced in the placement function, not by hoping.
- Horizontally clamped to the viewport with an 8px margin; the arrow is
  clamped separately so it keeps pointing at the anchor even when the body has
  been pushed sideways.
- `max-height: min(45vh, space available)`, with the body scrolling internally.

**Rejected: draggable and/or resizable.** A popover that has been dragged away
from its anchor is no longer a popover — it is a window, with the "which āya is
this about?" problem the drawer already has. Resizing and persistence are what
the side panel is for, and it already has both. This keeps the persisted state
to one new key instead of four.

**Why not the platform primitives.** The native `popover` attribute and CSS
`anchor-position` would do most of this, and neither can be relied on in the
WebKitGTK build Tauri ships on Linux. A fixed-position portal with hand-rolled
placement is the fallback, and it is what gets tested in the real app.

## Dismiss: Escape, click outside, close button

Escape is already handled globally in `+layout.svelte`. The popover branch goes
**before** the focus-mode branch in that handler, so Escape closes the popover
first and only leaves focus mode once it is closed — otherwise a reader in
focus mode loses the whole chrome when they meant to dismiss a card.

Click-outside is a capture-phase `pointerdown` listener on `window`, skipped
when the target is inside the popover or is the trigger itself.

## Scrolling with it open: reposition, then freeze

The anchor can leave the DOM entirely while the popover is open —
`AyahRow` drops its contents outside the render window and keeps only a
reserved-height box, and `PageView` empties lines outside the glyph window.

- While the anchor is usable, reposition on scroll (rAF-throttled).
- When it stops being usable, **freeze** at the last position rather than
  closing or chasing a stale rect. The commentary is still the commentary for
  the āya that was asked about; taking it away because the reader scrolled is
  the exact bug this work exists to fix.

**Correction, from testing rather than reasoning.** "Usable" was written here
as "still in the DOM", and that is not enough. Scrolling a long Surah with a
popover open produces a third state the first draft missed: the anchor is
still attached and still reports a rect, but that rect is now above the
viewport. Following it puts the card at a large negative offset, and the
reader watches the popover apparently vanish. Caught by scrolling 40 notches
down Al-Baqara in the real app. The freeze condition is therefore all three of
detached, zero-rect, **and** scrolled out of view — plus the top coordinate is
clamped into the viewport, so a straddling anchor cannot push the card
half-off either.

**Rejected: close on scroll-out.** It reintroduces "the panel decides what you
are reading", from the opposite direction.

**Rejected: re-anchor to the reserved-height box.** The box has no text in it,
so the arrow would point at a blank space that merely used to be the āya.

## Keyboard: `t` opens for the reader position

`t` currently toggles the drawer. It now opens the popover for
`readerPosition.ayahId` — the only defensible default when the user has not
pointed at anything — anchored to that āya's element if it is rendered. In
panel mode `t` keeps its current meaning, toggling the drawer.

Focus moves into the popover when it opens and returns to the trigger when it
closes. It is **not** modal and does not trap focus: it sits beside the reader
rather than over it, and trapping would make Tab a dead end in a reading app.

---

# State and settings

`show_tafsir`, `tafsir_id` and `tafsir_panel_width` keep their current meanings
exactly. One new key:

| Key           | Values               | Default   | Meaning                       |
| ------------- | -------------------- | --------- | ----------------------------- |
| `tafsir_view` | `popover` \| `panel` | `popover` | Which surface a trigger opens |

**Popover openness is deliberately not persisted.** A popover is a transient
answer to a click; restoring one on launch, anchored to an āya the reader may
not even be looking at, would be noise. `show_tafsir` therefore keeps meaning
"the side panel is open", which is what it means today — so an existing install
that had the drawer open and switches to panel mode finds it exactly as it was.

The new key needs the same round trip as the others: `settings` default in
`database/schema.sql`, the `Settings` struct and `load_settings` in Rust, the
`Settings` type in `src/lib/types/database.ts`, and `defaultSettings()` in the
frontend store. No migration is needed — `load_settings` defaults every key it
does not find, so an older database simply reports `popover`.

## Store

`tafsirStore` keeps its cache (keyed `${tafsirId}:${ayahId}`, 200-entry cap) and
its `#requestToken` guard; there is no second loader. What goes:

- `#pinnedAyahId`, `#positionAtPin`, `syncPosition()`, `targetAyahId` — the pin
  machinery exists only because the panel follows the reader. Explicit
  selection replaces it wholesale; leaving it half-wired would be worse than
  either design.

What arrives: `selection: { ayahId, trigger } | null`, `openFor(ayahId, trigger)`,
`close()`, and `view` / `setView()`. In panel mode the panel keeps following
`readerPosition` as it does today — that is the drawer's character, and the
reason to keep both.

**`readerPosition` is read, never written.** It feeds the progress indicator and
last-read restore, not just tafsir.

---

# Risks

1. **App zoom.** `app_zoom` is applied through the Tauri webview zoom API, not
   CSS. `getBoundingClientRect()` and `window.innerHeight` are both in post-zoom
   CSS pixels, so the placement math should be zoom-invariant — _should_, which
   is why it gets checked at 0.7 and 1.5 in the real app rather than reasoned
   about here.
2. **RTL.** The Arabic edition renders RTL. The popover's own chrome stays LTR
   (it is English UI); only the commentary body flips, as it does in the panel
   today. Placement is symmetric, and the arrow is positioned from the anchor's
   centre, so neither depends on text direction.
3. **WebKitGTK.** Everything here is `position: fixed`, `transform` and
   `getBoundingClientRect` — no native popover, no anchor positioning, no
   `:has()`. Verified in the real app, not in a desktop browser.
4. **Focus mode.** The toolbar is gone there, so the popover is the only way to
   reach tafsir. It must open, position and dismiss identically.
5. **Click-to-open vs text selection.** The reader supports selecting text. A
   click that ends a drag-selection must not open a popover — the handler
   ignores clicks where the window has a non-collapsed selection.

---

# File-by-file

| File                                                                                | Change                                                                                                                                                   |
| ----------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/lib/stores/tafsir.svelte.ts`                                                   | Remove pin machinery; add `selection`, `openFor`, `close`, `view`, `setView`; keep cache and token guard                                                 |
| `src/lib/components/tafsir/TafsirPopover.svelte`                                    | New — portal, placement, arrow, focus handling, dismissal                                                                                                |
| `src/lib/components/tafsir/TafsirBody.svelte`                                       | New — the entry rendering (header meta, verse ref, paragraphs, empty/loading/error states) shared by popover and panel, so the two surfaces cannot drift |
| `src/lib/components/tafsir/TafsirPanel.svelte`                                      | Keep; render `TafsirBody`; gate on `view === 'panel'`                                                                                                    |
| `src/lib/components/reader/ReaderPage.svelte`                                       | Mount the popover; keep the panel sibling only in panel mode                                                                                             |
| `src/lib/components/reader/AyahRow.svelte`                                          | Click on `.ayah-text` opens the popover; button passes its own element as the anchor                                                                     |
| `src/lib/components/reader/PageView.svelte`                                         | Word click → āya popover; add `data-word-index`                                                                                                          |
| `src/routes/+layout.svelte`                                                         | Escape ordering; `t` opens the popover (panel mode unchanged)                                                                                            |
| `src/lib/types/database.ts`, `src/lib/stores/settings.svelte.ts`                    | `tafsir_view`                                                                                                                                            |
| `src-tauri/src/models/mod.rs`, `src-tauri/src/db/queries.rs`, `database/schema.sql` | `tafsir_view` round trip                                                                                                                                 |

---

# Verification

- `pnpm check`, `pnpm lint`, `pnpm format:check`; `cargo test`, `cargo clippy
--all-targets -D warnings`, `cargo fmt` (Rust is touched for the setting).
- Real app (Tauri/WebKitGTK): both views; popover near the top, bottom and both
  edges; scrolling while open; switching views while open; app zoom 0.7 and
  1.5; focus mode.
- **The no-reflow claim, measured.** Screenshot the reader with the popover
  closed and open, and compare the reader region pixel-for-pixel with the
  popover's own rectangle excluded. Identical pixels outside that rectangle is
  the claim; anything else falsifies it. (Construction alone — a fixed-position
  portal outside `.reader-page` — should make reflow impossible, which is why
  the check is cheap and worth doing rather than assumed.)
