<script lang="ts">
  import { Check, Download, Loader, Trash2, TriangleAlert } from 'lucide-svelte';
  import { tafsirStore } from '$lib/stores/tafsir.svelte';

  const LANGUAGE_LABELS: Record<string, string> = {
    ar: 'Arabic',
    en: 'English',
    ur: 'Urdu',
    id: 'Indonesian',
    ml: 'Malayalam',
    tr: 'Turkish',
    fr: 'French',
  };

  function language(code: string): string {
    return LANGUAGE_LABELS[code] ?? code.toUpperCase();
  }

  /** Whole megabytes: this number exists to answer "is this a big download",
   *  and a decimal place does not help with that. */
  function mb(bytes: number): string {
    return `${Math.round(bytes / 1_048_576)} MB`;
  }

  const available = $derived(tafsirStore.packs.filter((p) => !p.installed));
  const installedCount = $derived(tafsirStore.packs.length - available.length);

  const percent = $derived.by(() => {
    const p = tafsirStore.progress;
    if (!p || p.total === 0) return null;
    return Math.round((p.received / p.total) * 100);
  });
</script>

<div class="editions">
  <!-- The bundled editions are listed too, greyed of any action: leaving them
       out would make the panel look like it holds only what was downloaded. -->
  <h3 class="group-label">Installed</h3>
  <ul class="list">
    {#each tafsirStore.editions as edition (edition.id)}
      <li class="row">
        <span class="tick" aria-hidden="true"><Check size={13} /></span>
        <span class="detail">
          <span class="title">{edition.title}</span>
          <span class="sub">
            {language(edition.language)}
            {#if edition.is_bundled}· ships with the app{/if}
          </span>
        </span>
        {#if !edition.is_bundled && edition.slug}
          {@const slug = edition.slug}
          <button
            class="action danger"
            onclick={() => tafsirStore.removePack(slug)}
            aria-label="Remove {edition.title}"
            title="Remove"
          >
            <Trash2 size={14} />
          </button>
        {/if}
      </li>
    {/each}
  </ul>

  {#if available.length}
    <h3 class="group-label">Available to download</h3>
    <ul class="list">
      {#each available as pack (pack.slug)}
        {@const busy = tafsirStore.installing === pack.slug}
        <li class="row">
          <span class="tick" aria-hidden="true"></span>
          <span class="detail">
            <span class="title">{pack.title}</span>
            <span class="sub">
              {language(pack.language)} · {mb(pack.download_bytes)} download · {mb(
                pack.installed_bytes,
              )} once installed
            </span>
            <span class="licence">{pack.license}</span>
            {#if busy}
              <!-- A determinate bar wherever the server sent a length, and a
                   percentage in text as well: a bar alone cannot say whether a
                   stalled download is at 4% or 94%. -->
              <span class="progress" role="progressbar" aria-valuenow={percent ?? undefined}>
                <span class="bar" style:width={percent !== null ? `${percent}%` : '100%'}></span>
              </span>
              <span class="sub">{percent !== null ? `${percent}%` : 'Starting…'}</span>
            {/if}
          </span>
          <button
            class="action"
            disabled={tafsirStore.installing !== null}
            onclick={() => tafsirStore.installPack(pack.slug)}
            aria-label="Download {pack.title}"
            title="Download"
          >
            {#if busy}
              <Loader size={14} class="spin" />
            {:else}
              <Download size={14} />
            {/if}
          </button>
        </li>
      {/each}
    </ul>
  {:else if !installedCount}
    <p class="state">No further editions are published for this version yet.</p>
  {/if}

  {#if tafsirStore.packError}
    <p class="error">
      <TriangleAlert size={14} />
      <span>{tafsirStore.packError}</span>
    </p>
  {/if}
</div>

<style>
  .editions {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .group-label {
    margin: 10px 0 2px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .group-label:first-child {
    margin-top: 0;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .row {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 8px 6px;
    border-radius: 8px;
  }

  .row:hover {
    background: var(--color-bg-hover);
  }

  .tick {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 19px;
    color: var(--color-accent);
  }

  .detail {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }

  .title {
    font-size: 13px;
    font-weight: 500;
    line-height: 1.4;
    color: var(--color-text);
  }

  .sub {
    font-size: 12px;
    line-height: 1.5;
    color: var(--color-text-muted);
  }

  /* The licence is the one thing here a reader may need to act on, so it is
     present on every downloadable edition rather than hidden behind a link. */
  .licence {
    font-size: 11px;
    line-height: 1.5;
    color: var(--color-text-muted);
    opacity: 0.85;
  }

  .progress {
    display: block;
    height: 4px;
    margin-top: 5px;
    border-radius: 2px;
    background: var(--color-border);
    overflow: hidden;
  }

  .bar {
    display: block;
    height: 100%;
    border-radius: 2px;
    background: var(--color-accent);
    transition: width 160ms linear;
  }

  .action {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .action:hover:not(:disabled) {
    background: var(--color-bg-elevated);
    color: var(--color-text);
  }

  .action:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .action.danger:hover {
    color: var(--color-danger, #e5484d);
  }

  .editions :global(.spin) {
    animation: spin 900ms linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .state {
    margin: 4px 0;
    font-size: 13px;
    line-height: 1.6;
    color: var(--color-text-muted);
  }

  .error {
    display: flex;
    align-items: flex-start;
    gap: 6px;
    margin: 8px 0 0;
    font-size: 12px;
    line-height: 1.5;
    color: var(--color-danger, #e5484d);
  }
</style>
