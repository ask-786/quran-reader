<!--
  The toolbar's routines button and the menu it drops.

  The one place routines live. The sidebar is for finding your way around the
  Mushaf and opens on the Surah list, which is what anyone opening it expects;
  routines are something you start, so they sit with the other things you
  start, in the toolbar — on screen with the sidebar shut and in every view.

  While a routine is being read the button *is* the progress: "Daily 2/6". It
  keeps that through a detour to another Surah, drawn dashed so it reads as
  "still in it, just not here", and the routine's row in the menu is the way
  back.
-->
<script lang="ts">
  import { tick } from 'svelte';
  import { page } from '$app/stores';
  import { Check, ChevronDown, ChevronUp, ListOrdered, LogOut, Pencil, Plus } from 'lucide-svelte';
  import {
    routinesStore,
    routineStatus,
    routineStepHref,
    routineTarget,
  } from '$lib/stores/routines.svelte';
  import { surahsStore } from '$lib/stores/surahs.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import type { Routine } from '$lib/types/database';

  /** Above this many steps the progress dots stop being countable at a glance. */
  const MAX_DOTS = 10;
  /** Keeps the menu off the window edge when the button is near it. */
  const EDGE = 8;

  let trigger = $state<HTMLButtonElement>();
  let menu = $state<HTMLDivElement>();
  let position = $state({ top: 0, left: 0 });

  /**
   * Whether the menu was opened with the pointer. A click puts focus on the
   * menu itself rather than on a row: a row focused under the mouse looks
   * picked when it isn't. From the keyboard (`r`, or Enter on the button) the
   * first row takes focus, so the arrows and Enter work straight away.
   */
  let openedByPointer = false;

  const open = $derived(uiStore.routinesMenuOpen);

  const active = $derived(routinesStore.active);
  const activeStatus = $derived(routinesStore.activeStatus);
  const onStep = $derived(
    routinesStore.isOnStep($page.route.id ?? null, Number($page.params.id) || 0),
  );
  const progress = $derived.by(() => {
    if (!active || !activeStatus) return '';
    if (activeStatus.kind === 'done') return 'done';
    if (activeStatus.kind === 'running') {
      return `${activeStatus.step + 1}/${active.surah_ids.length}`;
    }
    return '';
  });

  const rows = $derived(
    routinesStore.list.map((routine) => {
      const target = targetFor(routine);
      return {
        routine,
        status: routineStatus(routine),
        target,
        href: routineStepHref(routine.id, target.surahId, target.restart),
      };
    }),
  );

  /**
   * The active routine's row goes to the step it is on — the way back after a
   * detour — rather than wherever a fresh open of it would start.
   */
  function targetFor(routine: Routine) {
    if (routine.id === routinesStore.activeId && routinesStore.activeSurahId !== null) {
      return { surahId: routinesStore.activeSurahId, restart: false };
    }
    return routineTarget(routine);
  }

  /**
   * Open a routine from its row. Usually that is just following the link — the
   * Surah route's load enters the routine. But when the routine starts on the
   * Surah already open, the link can be the very URL in the address bar (say,
   * after leaving the routine from this Surah), and SvelteKit doesn't re-run a
   * load for a URL it is already on. So in that case the routine is entered
   * here, and the reader stays exactly where it is.
   */
  function openRoutine(e: MouseEvent, routine: Routine, target: ReturnType<typeof targetFor>) {
    uiStore.closeRoutinesMenu();
    const here = $page.route.id === '/surah/[id]' && Number($page.params.id) === target.surahId;
    if (!here) return;
    e.preventDefault();
    void routinesStore.enter(routine.id, target.surahId, target.restart);
  }

  function place() {
    if (!trigger) return;
    const rect = trigger.getBoundingClientRect();
    const width = menu?.offsetWidth ?? 300;
    position = {
      top: rect.bottom + 6,
      left: Math.max(EDGE, Math.min(rect.left, window.innerWidth - width - EDGE)),
    };
  }

  /** The rows the arrow keys walk: everything you would pick, not the row tools. */
  function navItems(): HTMLElement[] {
    return menu ? [...menu.querySelectorAll<HTMLElement>('[data-nav]')] : [];
  }

  // Place it, then take the keyboard: on the routine being read if there is
  // one, otherwise the first thing in it — or, opened by a click, on the menu
  // itself (see `openedByPointer`). Positioned by the window rather than inside
  // the toolbar, which scrolls sideways and would clip it.
  $effect(() => {
    if (!open) return;
    place();
    const byPointer = openedByPointer;
    openedByPointer = false;
    void tick().then(() => {
      place();
      if (byPointer) {
        menu?.focus();
        return;
      }
      const items = navItems();
      (items.find((el) => el.classList.contains('active')) ?? items[0])?.focus();
    });

    function onPointerDown(e: PointerEvent) {
      const target = e.target as Node | null;
      if (target && (menu?.contains(target) || trigger?.contains(target))) return;
      uiStore.closeRoutinesMenu();
    }
    window.addEventListener('pointerdown', onPointerDown, true);
    window.addEventListener('resize', place);
    return () => {
      window.removeEventListener('pointerdown', onPointerDown, true);
      window.removeEventListener('resize', place);
    };
  });

  function onMenuKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      // Stopped as well as consumed: the reader's handler on `window` reads
      // bare arrows as page turns.
      e.preventDefault();
      e.stopPropagation();
      const items = navItems();
      const i = items.indexOf(document.activeElement as HTMLElement);
      // From the menu itself (opened by a click), down enters at the top and
      // up at the bottom.
      const next = i < 0 ? (e.key === 'ArrowDown' ? 0 : -1) : e.key === 'ArrowDown' ? i + 1 : i - 1;
      items[(next + items.length) % items.length]?.focus();
      return;
    }
    // The key that opened it puts it away, as Ctrl+K does for the palette.
    if (e.key.toLowerCase() === 'r' && !e.ctrlKey && !e.metaKey && !e.altKey) {
      e.preventDefault();
      uiStore.closeRoutinesMenu();
      trigger?.focus();
    }
  }

  function edit(id: number | null) {
    uiStore.openRoutineEditor(id);
  }

  function leave() {
    routinesStore.exit();
    uiStore.closeRoutinesMenu();
  }
</script>

<button
  bind:this={trigger}
  class="trigger"
  class:running={!!active}
  class:away={!!active && !onStep}
  aria-haspopup="menu"
  aria-expanded={open}
  title={active ? `${active.name} routine (r)` : 'Routines (r)'}
  onpointerdown={() => (openedByPointer = !open)}
  onclick={() => uiStore.toggleRoutinesMenu()}
>
  <ListOrdered size={15} />
  <span class="trigger-label">{active ? active.name : 'Routines'}</span>
  {#if progress}<span class="trigger-progress">{progress}</span>{/if}
  <ChevronDown size={13} />
</button>

{#if open}
  <!-- Every href here comes from routineHref / routineStepHref, which resolve
       the route themselves; the rule only recognises an inline resolve(). -->
  <!-- eslint-disable svelte/no-navigation-without-resolve -->
  <div
    bind:this={menu}
    class="menu"
    role="menu"
    tabindex="-1"
    aria-label="Routines"
    style:top="{position.top}px"
    style:left="{position.left}px"
    onkeydown={onMenuKeydown}
  >
    {#if rows.length > 0}
      <ul class="list">
        {#each rows as { routine, status, target, href }, i (routine.id)}
          {@const isActive = routine.id === routinesStore.activeId}
          <li class="routine">
            <a
              class="row"
              class:active={isActive}
              role="menuitem"
              data-nav
              {href}
              onclick={(e) => openRoutine(e, routine, target)}
            >
              <span class="roundel" class:done={status.kind === 'done'}>
                {#if status.kind === 'done'}
                  <Check size={13} />
                {:else}
                  <ListOrdered size={13} />
                {/if}
              </span>
              <span class="row-main">
                <span class="row-title">{routine.name}</span>
                <span class="row-meta">
                  {#if status.kind === 'running'}
                    {#if routine.surah_ids.length <= MAX_DOTS}
                      <span class="dots" aria-hidden="true">
                        {#each routine.surah_ids as id, d (id)}
                          <span
                            class="dot"
                            class:read={d < status.step}
                            class:current={d === status.step}
                          ></span>
                        {/each}
                      </span>
                    {/if}
                    {#if isActive && !onStep}
                      Back to {surahsStore.get(routine.surah_ids[status.step])?.transliteration ??
                        `Surah ${routine.surah_ids[status.step]}`}
                    {:else}
                      {status.step + 1} of {routine.surah_ids.length} today
                    {/if}
                  {:else if status.kind === 'done'}
                    Done today
                  {:else}
                    {routine.surah_ids.length === 1
                      ? '1 surah'
                      : `${routine.surah_ids.length} surahs`}
                  {/if}
                </span>
              </span>
            </a>
            <!-- Beside the link, not in it: a button inside an anchor is
                 invalid, and its click would open the routine as well. -->
            <span class="row-tools">
              <button
                tabindex="-1"
                aria-label="Move {routine.name} up"
                title="Move up"
                disabled={i === 0}
                onclick={() => routinesStore.move(routine.id, -1)}
              >
                <ChevronUp size={14} />
              </button>
              <button
                tabindex="-1"
                aria-label="Move {routine.name} down"
                title="Move down"
                disabled={i === rows.length - 1}
                onclick={() => routinesStore.move(routine.id, 1)}
              >
                <ChevronDown size={14} />
              </button>
              <button
                aria-label="Edit {routine.name}"
                title="Edit"
                onclick={() => edit(routine.id)}
              >
                <Pencil size={14} />
              </button>
            </span>
          </li>
        {/each}
      </ul>
    {:else if routinesStore.loaded}
      <p class="intro">
        A routine is the Surahs you read every day, in your order. Start it, and finishing one
        brings up the next.
      </p>
    {/if}

    <div class="foot">
      <button class="foot-item" role="menuitem" data-nav onclick={() => edit(null)}>
        <Plus size={14} />
        {rows.length > 0 ? 'New routine' : 'Create a routine'}
      </button>
      {#if active}
        <button class="foot-item subtle" role="menuitem" data-nav onclick={leave}>
          <LogOut size={14} />
          Leave {active.name}
        </button>
      {/if}
    </div>
  </div>
  <!-- eslint-enable svelte/no-navigation-without-resolve -->
{/if}

<style>
  .trigger {
    display: flex;
    flex-shrink: 1;
    align-items: center;
    gap: 6px;
    min-width: 0;
    max-width: 220px;
    height: 30px;
    padding: 0 8px 0 10px;
    border: 1px solid var(--color-border);
    border-radius: 15px;
    background: transparent;
    color: var(--color-text-muted);
    font: inherit;
    font-size: 12px;
    transition:
      background var(--transition),
      color var(--transition);
  }

  .trigger:hover,
  .trigger[aria-expanded='true'] {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .trigger :global(svg) {
    flex-shrink: 0;
  }

  .trigger.running {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  /* Stepped out of the routine: still in it, but not what's open. */
  .trigger.away {
    border-style: dashed;
  }

  .trigger-label {
    min-width: 0;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .trigger-progress {
    flex-shrink: 0;
    font-variant-numeric: tabular-nums;
    opacity: 0.8;
  }

  /* Phone-width toolbar: the icon carries it, and the progress if running. */
  @media (max-width: 600px) {
    .trigger:not(.running) .trigger-label {
      display: none;
    }
  }

  .menu {
    position: fixed;
    z-index: 60;
    width: 300px;
    max-width: calc(100vw - 16px);
    max-height: min(480px, 70vh);
    overflow-y: auto;
    padding: 6px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-bg-elevated);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.3);
  }

  /* The engine's own focus ring (a dashed line in WebKitGTK) is turned off
     throughout; the accent outline below stands in for it, and only for the
     keyboard. */
  .menu:focus,
  .menu :focus:not(:focus-visible) {
    outline: none;
  }

  .list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .routine {
    position: relative;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    /* Room for the row tools, so a long name ellipsises before them. */
    padding: 9px 84px 9px 10px;
    border-radius: var(--radius);
    color: var(--color-text);
    text-decoration: none;
    transition: background var(--transition);
  }

  .row:hover,
  .row:focus-visible,
  .row.active {
    background: var(--color-bg-hover);
  }

  .row:focus-visible,
  .foot-item:focus-visible,
  .row-tools button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  .row.active .row-title {
    color: var(--color-accent);
  }

  .roundel {
    flex-shrink: 0;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: 1px solid var(--color-border);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-muted);
  }

  .roundel.done,
  .row.active .roundel {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .row-main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .row-title {
    font-size: 14px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .row-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--color-text-muted);
    white-space: nowrap;
  }

  .dots {
    display: flex;
    gap: 3px;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    border: 1px solid var(--color-border);
  }

  .dot.read {
    border-color: var(--color-accent);
    background: var(--color-accent);
  }

  .dot.current {
    border-color: var(--color-accent);
  }

  .row-tools {
    position: absolute;
    top: 50%;
    right: 6px;
    display: flex;
    gap: 2px;
    transform: translateY(-50%);
    opacity: 0;
    transition: opacity var(--transition);
  }

  /* Out of the way until the row is pointed at or the keyboard is in it. */
  .routine:hover .row-tools,
  .routine:focus-within .row-tools {
    opacity: 1;
  }

  .row-tools button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--color-text-muted);
  }

  .row-tools button:hover:not(:disabled) {
    background: var(--color-bg);
    color: var(--color-text);
  }

  .row-tools button:disabled {
    opacity: 0.35;
    cursor: default;
  }

  .intro {
    margin: 0;
    padding: 8px 10px 10px;
    font-size: 13px;
    line-height: 1.5;
    color: var(--color-text-muted);
  }

  .foot {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin-top: 4px;
    padding-top: 4px;
    border-top: 1px solid var(--color-border);
  }

  .foot-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border: none;
    border-radius: var(--radius);
    background: transparent;
    color: var(--color-accent);
    font: inherit;
    font-size: 13px;
    text-align: start;
  }

  .foot-item:hover,
  .foot-item:focus-visible {
    background: var(--color-bg-hover);
  }

  .foot-item.subtle {
    color: var(--color-text-muted);
  }
</style>
