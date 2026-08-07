<script lang="ts">
  import { ScrollText } from 'lucide-svelte';
  import { tafsirStore } from '$lib/stores/tafsir.svelte';

  /** Compact drops the attribution line, for the popover's tighter header. */
  let { compact = false }: { compact?: boolean } = $props();

  const edition = $derived(tafsirStore.active);

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
</script>

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
    {#if compact && schoolLabel}
      <span class="school">{schoolLabel}</span>
    {/if}
  </div>
  {#if !compact && attribution}
    <p class="attribution">
      {attribution}
      {#if schoolLabel}<span class="school">{schoolLabel}</span>{/if}
    </p>
  {/if}
</div>

<style>
  .title-block {
    min-width: 0;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    color: var(--color-text);
  }

  .edition-title,
  .edition-select {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text);
  }

  .edition-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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
    font-size: 11px;
    color: var(--color-text-muted);
    white-space: nowrap;
  }
</style>
