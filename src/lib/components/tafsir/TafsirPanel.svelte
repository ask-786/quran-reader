<script lang="ts">
  import { X, ScrollText } from 'lucide-svelte';
  import { tafsirStore, clampTafsirWidth } from '$lib/stores/tafsir.svelte';
  import { readerPosition } from '$lib/stores/reader-position.svelte';
  import { surahsStore } from '$lib/stores/surahs.svelte';

  /** Live width while dragging; null when the stored width is in force. */
  let dragWidth = $state<number | null>(null);
  let dragging = $state(false);

  const width = $derived(dragWidth ?? tafsirStore.width);
  const edition = $derived(tafsirStore.active);
  const entry = $derived(tafsirStore.entry);

  // An explicit open pins one Ayah; the pin lifts the moment the reader moves,
  // so the panel goes back to following without the user having to dismiss a
  // mode. This effect tracks the reader position and nothing else.
  $effect(() => {
    tafsirStore.syncPosition(readerPosition.ayahId);
  });

  $effect(() => {
    const ayahId = tafsirStore.targetAyahId;
    const active = tafsirStore.active;
    if (!active || ayahId === null) return;
    void tafsirStore.load(active.id, ayahId);
  });

  const surahName = $derived.by(() => {
    if (!entry) return null;
    return surahsStore.get(entry.surah_id)?.transliteration ?? null;
  });

  /** "2:255", or "2:1–5" where the edition comments on a run of verses. */
  const verseLabel = $derived.by(() => {
    if (!entry) return null;
    const { group_start_key: start, group_end_key: end } = entry;
    if (start && end && start !== end) {
      const endAyah = end.split(':')[1] ?? end;
      return `${start}–${endAyah}`;
    }
    return `${entry.surah_id}:${entry.ayah_number}`;
  });

  // Paragraphs rather than one block: the importer keeps blank-line breaks and
  // strips everything else, so this is the whole of the text's structure.
  const paragraphs = $derived(entry ? entry.text.split(/\n{2,}/).filter(Boolean) : []);

  const attribution = $derived.by(() => {
    if (!edition) return null;
    const parts = [edition.author];
    if (edition.translator) parts.push(`tr. ${edition.translator}`);
    return parts.join(' · ');
  });

  /** "Shāfiʿī · Ashʿarī" — informational, and the reason the picker exists. */
  const SCHOOL_LABELS: Record<string, string> = {
    shafii: 'Shāfiʿī',
    hanafi: 'Ḥanafī',
    maliki: 'Mālikī',
    hanbali: 'Ḥanbalī',
  };
  const CREED_LABELS: Record<string, string> = {
    ashari: 'Ashʿarī',
    maturidi: 'Māturīdī',
    athari: 'Atharī',
  };

  const schoolLabel = $derived.by(() => {
    if (!edition) return null;
    const school = edition.school ? (SCHOOL_LABELS[edition.school] ?? edition.school) : null;
    const creed = edition.creed ? (CREED_LABELS[edition.creed] ?? edition.creed) : null;
    return [school, creed].filter(Boolean).join(' · ') || null;
  });

  function startResize(e: PointerEvent) {
    const handle = e.currentTarget as HTMLElement;
    handle.setPointerCapture(e.pointerId);
    dragging = true;
    dragWidth = width;
  }

  function resize(e: PointerEvent) {
    if (!dragging) return;
    // The panel is pinned to the right edge, so its width is whatever is left
    // between the pointer and that edge.
    dragWidth = clampTafsirWidth(window.innerWidth - e.clientX);
  }

  function endResize(e: PointerEvent) {
    if (!dragging) return;
    const handle = e.currentTarget as HTMLElement;
    handle.releasePointerCapture(e.pointerId);
    dragging = false;
    if (dragWidth !== null) void tafsirStore.setWidth(dragWidth);
    dragWidth = null;
  }

  function nudge(e: KeyboardEvent) {
    const step = e.shiftKey ? 40 : 10;
    if (e.key === 'ArrowLeft') {
      e.preventDefault();
      void tafsirStore.setWidth(width + step);
    } else if (e.key === 'ArrowRight') {
      e.preventDefault();
      void tafsirStore.setWidth(width - step);
    }
  }
</script>

<aside class="tafsir-panel" style:width="{width}px" aria-label="Tafsir">
  <!-- A button, not a div with role="separator": the ARIA splitter pattern
       wants a focusable widget, and a real button is the one element that is
       focusable, keyboard-operable and announced as interactive without any
       of it having to be re-declared. A button's role doesn't carry
       aria-valuenow, so the current width goes in the label instead — the
       arrow keys change it, so it has to be announced somewhere. -->
  <button
    type="button"
    class="resize-handle"
    class:dragging
    aria-label="Resize tafsir panel, currently {width} pixels wide"
    onpointerdown={startResize}
    onpointermove={resize}
    onpointerup={endResize}
    onpointercancel={endResize}
    onkeydown={nudge}
  ></button>

  <header class="panel-header">
    <div class="title-block">
      <div class="title-row">
        <ScrollText size={15} />
        {#if tafsirStore.editions.length > 1}
          <select
            class="edition-select"
            aria-label="Tafsir edition"
            value={edition?.id}
            onchange={(e) => tafsirStore.setEdition(Number(e.currentTarget.value))}
          >
            {#each tafsirStore.editions as t (t.id)}
              <option value={t.id}>{t.title}</option>
            {/each}
          </select>
        {:else}
          <span class="edition-title">{edition?.title ?? 'Tafsir'}</span>
        {/if}
      </div>
      {#if attribution}
        <p class="attribution">
          {attribution}
          {#if schoolLabel}<span class="school">{schoolLabel}</span>{/if}
        </p>
      {/if}
    </div>
    <button class="icon-btn" onclick={() => tafsirStore.setOpen(false)} aria-label="Close tafsir">
      <X size={16} />
    </button>
  </header>

  <div class="panel-body">
    {#if !edition}
      <p class="state">No tafsir is installed.</p>
    {:else if tafsirStore.error}
      <p class="state">{tafsirStore.error}</p>
    {:else if tafsirStore.targetAyahId === null}
      <p class="state">Open a Surah to read its commentary.</p>
    {:else if entry}
      <p class="verse-ref">
        {#if surahName}<span class="surah">{surahName}</span>{/if}
        <span class="key">{verseLabel}</span>
      </p>
      <div class="text" dir={edition.direction} class:rtl={edition.direction === 'rtl'}>
        {#each paragraphs as para, i (i)}
          <p>{para}</p>
        {/each}
      </div>
    {:else if tafsirStore.loading}
      <p class="state">Loading…</p>
    {:else}
      <!-- A gap is normal, not a failure: al-Jalalayn passes over verses that
           need no gloss (226 of them in the Arabic edition), and saying so is
           more useful than an empty panel. -->
      <p class="state">
        No separate commentary on this verse — this edition comments on the surrounding verses
        instead.
      </p>
    {/if}
  </div>
</aside>

<style>
  .tafsir-panel {
    position: relative;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    height: 100%;
    min-width: 0;
    background: var(--color-bg-elevated);
    border-left: 1px solid var(--color-border);
    z-index: 7;
  }

  .resize-handle {
    position: absolute;
    top: 0;
    left: -3px;
    width: 7px;
    height: 100%;
    padding: 0;
    border: none;
    cursor: col-resize;
    background: transparent;
    z-index: 1;
  }

  .resize-handle:hover,
  .resize-handle:focus-visible,
  .resize-handle.dragging {
    background: var(--color-accent);
    opacity: 0.35;
    outline: none;
  }

  .panel-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 8px;
    padding: 12px 12px 10px 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .title-block {
    min-width: 0;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--color-text);
  }

  .edition-title,
  .edition-select {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text);
  }

  .edition-select {
    max-width: 100%;
    border: none;
    background: transparent;
    cursor: pointer;
    font-family: inherit;
  }

  .attribution {
    margin: 3px 0 0;
    font-size: 11.5px;
    line-height: 1.5;
    color: var(--color-text-muted);
  }

  .school {
    margin-inline-start: 6px;
    padding: 1px 6px;
    border: 1px solid var(--color-border);
    border-radius: 999px;
    white-space: nowrap;
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .icon-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .panel-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 14px 16px 32px;
  }

  .verse-ref {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin: 0 0 10px;
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .surah {
    font-weight: 600;
    color: var(--color-text);
  }

  .text p {
    margin: 0 0 12px;
    font-size: 14.5px;
    line-height: 1.75;
    color: var(--color-text);
  }

  /* No bundled font here is fit for running Arabic prose (see app.css), so an
     Arabic edition falls through to the system stack until one is added. */
  .text.rtl p {
    font-size: 17px;
    line-height: 2;
  }

  .state {
    margin: 0;
    font-size: 13.5px;
    line-height: 1.7;
    color: var(--color-text-muted);
  }

  /* Below the tablet breakpoint the drawer covers the reader rather than
     squeezing it into a column too narrow to read. */
  @media (max-width: 900px) {
    .tafsir-panel {
      position: absolute;
      inset: 0 0 0 auto;
      width: min(100%, 480px) !important;
      box-shadow: -4px 0 24px rgba(0, 0, 0, 0.3);
    }

    .resize-handle {
      display: none;
    }
  }
</style>
