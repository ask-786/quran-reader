# Routines

A routine is a named, ordered list of Surahs someone reads every day — "Morning:
Ya-Sin, Al-Waqi'ah, Al-Mulk". Starting one opens its first Surah; finishing a
Surah leads to the next in the routine's order rather than the next in the
Mushaf. A reader can have as many routines as they like.

## Shape of the feature

A routine is a **layer over the existing `/surah/[id]` route**, not a route of
its own. Reading a routine's Surah is ordinary Surah reading — tafsir, audio,
both reading modes, position recording and the Recent list all keep working
untouched. Only three things become routine-aware:

1. what `n` / `p` do,
2. what sits at the foot of the Surah,
3. the toolbar's Routines button, which shows where in the routine you are.

The route knows it is in a routine through `?routine=<id>`, so starting one is a
plain link. The route's load only _plans_ the step (where it opens) and the
page _enters_ the routine once the navigation lands — never the load itself,
because SvelteKit runs loads to preload links on hover.

## Storage — migration 011

```sql
routine (
  id, name, sort_order,
  run_step,        -- index of the step the current run is on; = item count when done
  run_started_at,  -- UTC timestamp the current run began
  created_at, updated_at
)
routine_item (
  routine_id → routine ON DELETE CASCADE,
  position, surah_id,
  start_ayah, end_ayah  -- reserved for partial readings (Ayat al-Kursi, the
                        -- last two of Al-Baqarah); always NULL for now
)
```

- **Progress lives on each routine**, so being 2/5 through Morning survives
  starting Before sleep. One routine is _active_ at a time (in memory); every
  routine keeps its own run.
- **A run belongs to a day**: it counts as today's when `run_started_at` falls
  on today's _local_ date. The day turns at local midnight. Anything older is
  "not started", and opening the routine begins a new run at step 1.
- Saving a routine replaces its items in one transaction. Changing the list of
  Surahs drops the run in progress, since its step index no longer means what
  it did. Renaming doesn't.
- A Surah appears at most once per routine (so "which step is this Surah?" has
  one answer). The same Surah may sit in several routines.

Commands: `get_routines`, `save_routine`, `delete_routine`,
`reorder_routines`, `set_routine_step`.

## Behaviour

- **Starting**: a **Routines** button in the toolbar, beside Search, drops a
  menu of every routine with today's progress ("2 of 5 today", done), plus New
  routine; `r` opens it. The sidebar is left alone — it opens on the Surah list,
  which is what people expect of it. (Tried and dropped: a Routines tab in the
  sidebar, too cramped at five tabs; a section under Recent and a Home/Browse
  split, which hid routines or the Surah list respectively.) A routine's link
  opens the step today's run is on, or step 1 for a fresh or finished run.
- **Where a step opens**: at the top of the Surah — a daily routine starts each
  Surah from its beginning — unless the Surah's stored reading position was
  written during this run, in which case it resumes there. That covers closing
  the app mid-Surah and coming back the same day.
- **`n` / `p`** step through the routine while the open Surah is the routine's
  current step. Anywhere else they keep their usual meaning, so looking up a
  verse elsewhere pauses the routine rather than losing it. On the last step
  `n` marks the routine done.
- **`Shift+N` / `Shift+P`**: the plain next / previous Surah, to step out of the
  routine on purpose.
- **`r`** opens and closes the routines menu.
- **End of a Surah**: a card — "Next: Al-Mulk →", or on the last step "Finish
  Morning". Finishing is an explicit press (or `n`), not the card scrolling into
  view: a short Surah fits on one screen, and would be marked done the moment it
  opened.
- **Toolbar button as progress**: while a routine is active the button reads
  "Morning · 2/5"; drawn dashed when you have stepped out of it, when its row in
  the menu says "Back to Al-Mulk". The menu's last item leaves the routine.

## Editing

A dialog (`RoutineEditor`), opened from the Routines tab: a name, a Surah picker
filtered the same way as the navigator, and the ordered list with ↑/↓ buttons
and Alt+↑/↓. Rename, delete (confirmed), and reorder the routines themselves.

## Where the code goes

| Piece           | File                                                                         |
| --------------- | ---------------------------------------------------------------------------- |
| Tables          | `database/migrations/011_routines.sql`, `database/schema.sql`                |
| Migration arm   | `src-tauri/src/db/connection.rs`                                             |
| Queries + tests | `src-tauri/src/db/queries.rs`                                                |
| Model           | `src-tauri/src/models/mod.rs`, `src/lib/types/database.ts`                   |
| Commands        | `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`, `src/lib/api/db.ts` |
| State           | `src/lib/stores/routines.svelte.ts`                                          |
| Toolbar menu    | `src/lib/components/routines/RoutinesMenu.svelte`, `Toolbar.svelte`          |
| Entering a step | `src/routes/surah/[id]/+page.ts` (plan), `+page.svelte` (enter)              |
| Keys            | `src/routes/+layout.svelte`, `ShortcutsSection.svelte`                       |
| End card        | `RoutineNextCard.svelte`, in `ReaderView` and `PageView`                     |
| Editor          | `src/lib/components/routines/RoutineEditor.svelte`                           |

## Later

- Partial readings, using the reserved `start_ayah` / `end_ayah`.
- Recitation carrying on into the next step when a Surah's audio ends.
- A day boundary at Fajr rather than midnight, if prayer times ever arrive.
- A streak / completion history.
