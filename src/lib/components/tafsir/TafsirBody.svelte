<script lang="ts">
  import { tafsirStore } from '$lib/stores/tafsir.svelte';
  import { surahsStore } from '$lib/stores/surahs.svelte';

  /**
   * The commentary itself, shared by the popover and the side panel so the two
   * surfaces cannot drift apart. Everything it needs is on the store — which
   * Ayah is on show differs between the surfaces, but only one of them is ever
   * mounted (see `tafsirStore.view`), so there is a single answer at any time.
   */
  const edition = $derived(tafsirStore.active);
  const entry = $derived(tafsirStore.entry);

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
</script>

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
  <!-- A gap is normal, not a failure: al-Jalalayn passes over verses that need
       no gloss (226 of them in the Arabic edition), and saying so is more
       useful than an empty panel. -->
  <p class="state">
    No separate commentary on this verse — this edition comments on the surrounding verses instead.
  </p>
{/if}

<style>
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

  .text p:last-child {
    margin-bottom: 0;
  }

  /* Amiri, bundled for exactly this (see the --font-arabic-prose note in
     app.css). Larger and looser than the Latin text above because vocalised
     Arabic carries marks above and below the line: at the Latin size the
     harakat collide with the line below.

     17px, not the 18px this ran at under Noto Naskh: Amiri sets a larger and
     darker body at the same pixel size, so the smaller number lands on the
     same apparent size and colour. */
  .text.rtl p {
    font-family: var(--font-arabic-prose);
    font-size: 17px;
    line-height: 2.05;
  }

  .state {
    margin: 0;
    font-size: 13.5px;
    line-height: 1.7;
    color: var(--color-text-muted);
  }
</style>
