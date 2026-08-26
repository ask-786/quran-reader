<script lang="ts">
  import { ChevronDown, Check } from 'lucide-svelte';
  import type { Tafsir } from '$lib/types/database';

  let {
    editions,
    selectedId,
    onselect,
  }: {
    editions: Tafsir[];
    selectedId: number | undefined;
    onselect: (id: number) => void;
  } = $props();

  let open = $state(false);
  /** Which row the keyboard is on. Separate from the selection: moving through
   *  the list must not change the edition until it is committed. */
  let activeIndex = $state(0);
  let trigger = $state<HTMLButtonElement>();
  let list = $state<HTMLDivElement>();

  const selected = $derived(editions.find((t) => t.id === selectedId) ?? editions[0]);

  const LANGUAGE_LABELS: Record<string, string> = {
    ar: 'Arabic',
    en: 'English',
    ur: 'Urdu',
    id: 'Indonesian',
    ml: 'Malayalam',
    tr: 'Turkish',
    fr: 'French',
  };

  /**
   * Whether to present this edition under its own name rather than the
   * romanised one. `translator` is the schema's marker for "this is a
   * translation of the work" — so an edition without one is the original, and
   * the original is the thing that has a name of its own to be called by.
   */
  function showsNativeName(t: Tafsir): boolean {
    return t.translator === null && t.name_native !== null;
  }

  /**
   * What to call an edition. Both al-Jalālayns carry the same `title` and the
   * same `name_native` — the work is one work — so a row that shows only a
   * name shows the same thing twice. The original answers in Arabic and the
   * translation in Latin script, which separates them before the sub-line has
   * to say anything.
   */
  function displayName(t: Tafsir): string {
    return (showsNativeName(t) ? t.name_native : t.title) ?? t.title;
  }

  /**
   * The line that actually distinguishes them: which language you will be
   * reading, and whose hand it came through. "Arabic · the original" against
   * "English · tr. Feras Hamza" is the whole choice on offer here.
   */
  function provenance(t: Tafsir): string {
    const language = LANGUAGE_LABELS[t.language] ?? t.language.toUpperCase();
    return `${language} · ${t.translator ? `tr. ${t.translator}` : 'the original'}`;
  }

  function openList() {
    activeIndex = Math.max(
      0,
      editions.findIndex((t) => t.id === selected?.id),
    );
    open = true;
  }

  function close(focusTrigger = true) {
    open = false;
    if (focusTrigger) trigger?.focus();
  }

  function commit(index: number) {
    const edition = editions[index];
    if (edition) onselect(edition.id);
    close();
  }

  /**
   * Every key this handles is stopped as well as consumed. The reader's global
   * shortcuts sit on `window` and treat the arrows as page turns and `t` as a
   * tafsir toggle — with a listbox open those belong to the listbox, and its
   * trigger is a `<button>`, so the text-field guard up there does not cover
   * it.
   */
  function onKeydown(e: KeyboardEvent) {
    if (!open) {
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp' || e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        e.stopPropagation();
        openList();
      }
      return;
    }

    switch (e.key) {
      case 'ArrowDown':
        activeIndex = (activeIndex + 1) % editions.length;
        break;
      case 'ArrowUp':
        activeIndex = (activeIndex - 1 + editions.length) % editions.length;
        break;
      case 'Home':
        activeIndex = 0;
        break;
      case 'End':
        activeIndex = editions.length - 1;
        break;
      case 'Enter':
      case ' ':
        commit(activeIndex);
        break;
      case 'Escape':
        close();
        break;
      case 'Tab':
        // Tab keeps its meaning — it just takes the list away on the way out.
        close(false);
        return;
      default:
        return;
    }
    e.preventDefault();
    e.stopPropagation();
  }

  // Dismiss on a click anywhere outside the picker. Capture, so it lands
  // before the click can reach whatever it was aimed at.
  $effect(() => {
    if (!open) return;
    const onPointerDown = (e: PointerEvent) => {
      const target = e.target as Node | null;
      if (target && (trigger?.contains(target) || list?.contains(target))) return;
      open = false;
    };
    window.addEventListener('pointerdown', onPointerDown, true);
    return () => window.removeEventListener('pointerdown', onPointerDown, true);
  });
</script>

<div class="picker" onkeydown={onKeydown} role="presentation">
  <button
    bind:this={trigger}
    type="button"
    class="trigger"
    class:open
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-label="Tafsir edition"
    onclick={() => (open ? close(false) : openList())}
  >
    <span
      class="trigger-label"
      class:native={!!selected && showsNativeName(selected)}
      dir={selected && showsNativeName(selected) ? selected.direction : null}
    >
      {selected ? displayName(selected) : 'Tafsir'}
    </span>
    <ChevronDown size={14} class="chevron" />
  </button>

  {#if open}
    <div bind:this={list} class="menu" role="listbox" aria-label="Tafsir edition" tabindex="-1">
      {#each editions as t, i (t.id)}
        <button
          type="button"
          class="option"
          class:active={i === activeIndex}
          role="option"
          aria-selected={t.id === selected?.id}
          onclick={() => commit(i)}
          onmousemove={() => (activeIndex = i)}
        >
          <span class="tick">
            {#if t.id === selected?.id}<Check size={13} />{/if}
          </span>
          <span class="option-text">
            <span
              class="option-title"
              class:native={showsNativeName(t)}
              dir={showsNativeName(t) ? t.direction : null}
            >
              {displayName(t)}
            </span>
            <span class="option-provenance">{provenance(t)}</span>
          </span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .picker {
    position: relative;
    min-width: 0;
  }

  .trigger {
    display: flex;
    align-items: center;
    gap: 3px;
    max-width: 100%;
    margin-inline-start: -5px;
    padding: 2px 5px;
    border: 1px solid transparent;
    border-radius: 6px;
    background: transparent;
    color: var(--color-text);
    font-family: inherit;
    font-size: 14px;
    font-weight: 600;
    line-height: 1.4;
    cursor: pointer;
  }

  .trigger:hover {
    background: var(--color-bg-hover);
  }

  .trigger.open,
  .trigger:focus-visible {
    border-color: var(--color-border);
    background: var(--color-bg-hover);
    outline: none;
  }

  .trigger-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* Muted so the edition name carries the row, and rotated while open so the
     control says which way it is facing. */
  .trigger :global(.chevron) {
    flex-shrink: 0;
    color: var(--color-text-muted);
    transition: transform 120ms ease;
  }

  .trigger.open :global(.chevron) {
    transform: rotate(180deg);
  }

  .menu {
    position: absolute;
    top: calc(100% + 6px);
    inset-inline-start: -5px;
    z-index: 5;
    min-width: max(100%, 220px);
    max-width: min(320px, 80vw);
    padding: 4px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-bg-elevated);
    box-shadow: 0 10px 28px rgba(0, 0, 0, 0.28);
    outline: none;
  }

  .option {
    display: flex;
    align-items: flex-start;
    gap: 7px;
    width: 100%;
    padding: 7px 9px;
    border: none;
    border-radius: 7px;
    background: transparent;
    color: var(--color-text);
    font-family: inherit;
    font-size: 13px;
    font-weight: 500;
    text-align: start;
    cursor: pointer;
  }

  /* One highlight for both routes in: the pointer writes `activeIndex` on
     move, so hover and the arrow keys can never light different rows. */
  .option.active {
    background: var(--color-bg-hover);
  }

  .tick {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    justify-content: center;
    width: 13px;
    height: 18px;
    color: var(--color-accent);
  }

  .option-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }

  .option-title {
    line-height: 1.4;
  }

  .option-provenance {
    font-size: 12px;
    font-weight: 400;
    line-height: 1.5;
    color: var(--color-text-muted);
  }

  /* Noto Naskh Arabic, the same face the Arabic commentary itself is set in
     (see --font-arabic-prose in app.css). Nudged up a little because vocalised
     Arabic reads small next to Latin at the same px. */
  .native {
    font-family: var(--font-arabic-prose);
    font-size: 1.08em;
  }
</style>
