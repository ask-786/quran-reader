import { resolve } from '$app/paths';
import {
  deleteRoutine,
  getRoutines,
  reorderRoutines,
  saveRoutine,
  setRoutineStep,
} from '$lib/api/db';
import type { Routine } from '$lib/types/database';

/**
 * Reading routines: named, ordered lists of Surahs read as a set, and the one
 * being read right now.
 *
 * A routine is a layer over the ordinary Surah route rather than a route of its
 * own (see docs/routines-plan.md). The route says which routine it was opened
 * from with `?routine=`, `enter()` turns that into the active run, and from
 * there `n`/`p`, the card at the foot of the Surah, and the toolbar chip all ask
 * this store where the routine goes next.
 *
 * Each routine keeps its own run in the database, so being part way through one
 * survives starting another. Only which routine is *active* lives here, and only
 * for the length of the app run.
 */

/** Today's state of one routine's run. */
export type RoutineStatus = { kind: 'idle' } | { kind: 'running'; step: number } | { kind: 'done' };

/** The query parameters a routine step travels in. */
export const ROUTINE_PARAM = 'routine';
/** Asks `enter()` for a fresh run even when today's is already finished. */
export const RESTART_PARAM = 'restart';

/** `YYYY-MM-DD` of an instant in the reader's own timezone. */
function localDay(date: Date): string {
  const m = String(date.getMonth() + 1).padStart(2, '0');
  const d = String(date.getDate()).padStart(2, '0');
  return `${date.getFullYear()}-${m}-${d}`;
}

/**
 * Whether a run was started today. The day turns at local midnight, which is
 * why this can't be asked of the database: SQLite only knows UTC.
 */
function startedToday(iso: string | null): boolean {
  if (!iso) return false;
  const then = Date.parse(iso);
  return !Number.isNaN(then) && localDay(new Date(then)) === localDay(new Date());
}

export function routineStatus(routine: Routine): RoutineStatus {
  const step = routine.run_step;
  if (step === null || !startedToday(routine.run_started_at)) return { kind: 'idle' };
  if (step >= routine.surah_ids.length) return { kind: 'done' };
  return { kind: 'running', step };
}

/** The route for one step of a routine. */
export function routineStepHref(routineId: number, surahId: number, restart = false) {
  const id = String(surahId);
  return restart
    ? resolve(`/surah/[id]?${ROUTINE_PARAM}=${routineId}&${RESTART_PARAM}=1`, { id })
    : resolve(`/surah/[id]?${ROUTINE_PARAM}=${routineId}`, { id });
}

/**
 * Where opening a routine from the list should land: the step today's run is
 * on, or the first Surah of a fresh one. A run already finished today starts
 * over — opening a routine you have done is asking to read it again.
 */
export function routineTarget(routine: Routine): { surahId: number; restart: boolean } {
  const status = routineStatus(routine);
  if (status.kind === 'running') {
    return { surahId: routine.surah_ids[status.step], restart: false };
  }
  return { surahId: routine.surah_ids[0], restart: status.kind === 'done' };
}

export function routineHref(routine: Routine) {
  const { surahId, restart } = routineTarget(routine);
  return routineStepHref(routine.id, surahId, restart);
}

class RoutinesStore {
  list = $state<Routine[]>([]);
  loaded = $state(false);

  /** The routine being read, if any. */
  activeId = $state<number | null>(null);

  #loading: Promise<void> | null = null;

  /** Load once; safe to await from a route's load function on every visit. */
  init(): Promise<void> {
    this.#loading ??= (async () => {
      try {
        this.list = await getRoutines();
      } catch (err) {
        console.error('Failed to load routines', err);
      } finally {
        this.loaded = true;
      }
    })();
    return this.#loading;
  }

  get(id: number): Routine | undefined {
    return this.list.find((r) => r.id === id);
  }

  active = $derived(this.activeId === null ? null : (this.get(this.activeId) ?? null));

  /** The active routine's status, or null with no routine active. */
  activeStatus = $derived(this.active ? routineStatus(this.active) : null);

  /**
   * Which step the active routine is on. A finished run still points at its
   * last Surah, so `p` from the "complete" card walks back the way it came.
   */
  activeStep = $derived.by(() => {
    const status = this.activeStatus;
    const routine = this.active;
    if (!status || !routine) return null;
    if (status.kind === 'running') return status.step;
    if (status.kind === 'done') return routine.surah_ids.length - 1;
    return null;
  });

  /** The Surah the active routine is on. */
  activeSurahId = $derived(
    this.active && this.activeStep !== null ? this.active.surah_ids[this.activeStep] : null,
  );

  /**
   * Whether the open route is the active routine's current step — the one
   * condition under which `n`/`p` and the end card speak for the routine. A
   * reader who has gone to look something up elsewhere is still *in* the
   * routine; it just isn't what's open.
   */
  isOnStep(routeId: string | null, surahId: number): boolean {
    return routeId === '/surah/[id]' && this.activeSurahId === surahId;
  }

  #replace(updated: Routine) {
    const i = this.list.findIndex((r) => r.id === updated.id);
    if (i >= 0) this.list[i] = updated;
    else this.list.push(updated);
  }

  /**
   * What opening `surahId` as a step of a routine would mean, without doing it:
   * whether the Surah is part of the routine at all (null if not, and the route
   * should behave as if it were never asked), and when the run it would join
   * started — null for a run that would start fresh.
   *
   * Pure on purpose. The Surah route's load calls this, and SvelteKit runs a
   * load to *preload* a link the pointer is merely resting on; a load that
   * started a routine would start it on hover. The route enters the routine
   * once the navigation has actually happened (see `enter`).
   */
  async plan(
    routineId: number,
    surahId: number,
    restart: boolean,
  ): Promise<{ runStartedAt: string | null } | null> {
    await this.init();
    const routine = this.get(routineId);
    if (!routine || !routine.surah_ids.includes(surahId)) return null;
    return { runStartedAt: this.#startsNewRun(routine, restart) ? null : routine.run_started_at };
  }

  /** A run starts afresh on a new day, or on a deliberate restart of a finished one. */
  #startsNewRun(routine: Routine, restart: boolean) {
    const status = routineStatus(routine);
    return status.kind === 'idle' || (restart && status.kind === 'done');
  }

  /**
   * Make `surahId` the current step of a routine and that routine the active
   * one, starting today's run if there isn't one. Does nothing if the Surah
   * isn't part of the routine.
   */
  async enter(routineId: number, surahId: number, restart: boolean) {
    await this.init();
    const routine = this.get(routineId);
    if (!routine) return;
    const step = routine.surah_ids.indexOf(surahId);
    if (step < 0) return;

    const newRun = this.#startsNewRun(routine, restart);
    if (newRun || routine.run_step !== step) {
      try {
        this.#replace(await setRoutineStep(routine.id, step, newRun));
      } catch (err) {
        console.error('Failed to record routine progress', err);
      }
    }
    this.activeId = routine.id;
  }

  /** The route one step forward or back in the active routine, if there is one. */
  stepHref(delta: 1 | -1): string | null {
    const routine = this.active;
    const step = this.activeStep;
    if (!routine || step === null) return null;
    const target = step + delta;
    if (target < 0 || target >= routine.surah_ids.length) return null;
    return routineStepHref(routine.id, routine.surah_ids[target]);
  }

  /** The Surah after the current step, for the end card to name. */
  nextSurahId = $derived.by(() => {
    const routine = this.active;
    const step = this.activeStep;
    if (!routine || step === null) return null;
    return routine.surah_ids[step + 1] ?? null;
  });

  /** Mark the active routine's run finished. */
  async complete() {
    const routine = this.active;
    if (!routine || this.activeStatus?.kind !== 'running') return;
    try {
      this.#replace(await setRoutineStep(routine.id, routine.surah_ids.length, false));
    } catch (err) {
      console.error('Failed to record routine progress', err);
    }
  }

  /** Stop reading the active routine. Its progress stays where it was. */
  exit() {
    this.activeId = null;
  }

  async save(routineId: number | null, name: string, surahIds: number[]): Promise<Routine> {
    const saved = await saveRoutine(routineId, name, surahIds);
    this.#replace(saved);
    return saved;
  }

  async remove(routineId: number) {
    await deleteRoutine(routineId);
    this.list = this.list.filter((r) => r.id !== routineId);
    if (this.activeId === routineId) this.activeId = null;
  }

  /** Move a routine one place up or down the list. */
  async move(routineId: number, delta: 1 | -1) {
    const i = this.list.findIndex((r) => r.id === routineId);
    const j = i + delta;
    if (i < 0 || j < 0 || j >= this.list.length) return;
    const next = [...this.list];
    [next[i], next[j]] = [next[j], next[i]];
    const previous = this.list;
    this.list = next;
    try {
      await reorderRoutines(next.map((r) => r.id));
    } catch (err) {
      console.error('Failed to reorder routines', err);
      this.list = previous;
    }
  }
}

export const routinesStore = new RoutinesStore();
