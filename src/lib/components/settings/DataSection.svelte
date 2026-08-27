<script lang="ts">
  import { onMount } from 'svelte';
  import { Trash2, Check, X } from 'lucide-svelte';
  import { dbStats } from '$lib/api/db';
  import { tafsirStore } from '$lib/stores/tafsir.svelte';
  import { readingStore } from '$lib/stores/reading.svelte';
  import type { DbStats } from '$lib/types/database';

  let stats = $state<DbStats | null>(null);
  let statsError = $state(false);
  /**
   * Inline confirmation rather than a dialog, matching the edition list: this
   * throws away something unrecoverable, and a bare trash icon is easy to hit
   * by mistake. Session-only — nothing should still be armed next time.
   */
  let confirmingClear = $state(false);
  let cleared = $state(false);

  onMount(() => {
    dbStats()
      .then((s) => (stats = s))
      .catch(() => (statsError = true));
  });

  /** What the downloaded editions add to the database, which is the only disk
   *  figure the app actually knows. The seed database's own size is fixed and
   *  not something anyone can act on. */
  const downloadedBytes = $derived(
    tafsirStore.packs.filter((p) => p.installed).reduce((sum, p) => sum + p.installed_bytes, 0),
  );
  const downloadedCount = $derived(tafsirStore.packs.filter((p) => p.installed).length);

  function mb(bytes: number): string {
    return `${Math.round(bytes / 1_048_576)} MB`;
  }

  async function clearHistory() {
    confirmingClear = false;
    await readingStore.clearHistory();
    cleared = true;
    setTimeout(() => (cleared = false), 2000);
  }
</script>

<dl class="stats">
  <div class="stat">
    <dt>Downloaded editions</dt>
    <dd>
      {#if downloadedCount}
        {downloadedCount} · {mb(downloadedBytes)}
      {:else}
        None
      {/if}
    </dd>
  </div>
  <div class="stat">
    <dt>Bookmarks</dt>
    <dd>{stats ? stats.bookmark_count : statsError ? '—' : '…'}</dd>
  </div>
  <div class="stat">
    <dt>Notes</dt>
    <dd>{stats ? stats.note_count : statsError ? '—' : '…'}</dd>
  </div>
  <div class="stat">
    <dt>Schema version</dt>
    <dd>{stats ? stats.schema_version : statsError ? '—' : '…'}</dd>
  </div>
</dl>

<div class="action">
  <div class="text">
    <span class="label">Reading history</span>
    <span class="description">
      The list of recent sittings behind the Recent tab. Bookmarks, notes and where you left off in
      each Surah are kept.
    </span>
  </div>
  {#if confirmingClear}
    <div class="confirm">
      <button class="btn danger" onclick={clearHistory} aria-label="Confirm clearing history">
        <Check size={14} /> Clear
      </button>
      <button
        class="btn"
        onclick={() => (confirmingClear = false)}
        aria-label="Keep reading history"
      >
        <X size={14} />
      </button>
    </div>
  {:else}
    <button class="btn danger" onclick={() => (confirmingClear = true)} disabled={cleared}>
      {#if cleared}
        <Check size={14} /> Cleared
      {:else}
        <Trash2 size={14} /> Clear
      {/if}
    </button>
  {/if}
</div>

<p class="note">
  Nothing leaves this machine. The app makes no network request except a download you ask for on the
  Editions page.
</p>

<style>
  .stats {
    display: flex;
    flex-direction: column;
    gap: 0;
    margin: 0 0 8px;
  }

  .stat {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 16px;
    padding: 10px 0;
    font-size: 13px;
  }

  .stat + .stat {
    border-top: 1px solid var(--color-border);
  }

  dt {
    color: var(--color-text-muted);
  }

  dd {
    margin: 0;
    font-variant-numeric: tabular-nums;
    color: var(--color-text);
  }

  .action {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    padding: 14px 0 0;
    border-top: 1px solid var(--color-border);
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }

  .label {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text);
  }

  .description {
    font-size: 12px;
    line-height: 1.5;
    color: var(--color-text-muted);
  }

  .confirm {
    display: flex;
    flex-shrink: 0;
    gap: 4px;
  }

  .btn {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border: 1px solid var(--color-border);
    border-radius: 7px;
    background: transparent;
    color: var(--color-text-muted);
    font-family: inherit;
    font-size: 12px;
    cursor: pointer;
  }

  .btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .btn.danger:hover:not(:disabled) {
    border-color: var(--color-danger, #e5484d);
    color: var(--color-danger, #e5484d);
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: default;
  }

  .note {
    margin: 20px 0 0;
    font-size: 12px;
    line-height: 1.6;
    color: var(--color-text-muted);
  }
</style>
