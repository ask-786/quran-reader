<!--
  Listening to a whole Surah.

  Opened deliberately, from the Listen button in the Surah banner, and closed
  the moment it is not wanted. That is the whole reason it is allowed to exist
  over the reader at all: it is never there unless it was asked for, and it is
  not remembered between sessions.

  A card in the corner rather than a full-height column, so the page underneath
  keeps its measure and its text. It steps aside for the tafsir panel through
  --tafsir-inset, the same way the reader's other corner controls do.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import {
    Check,
    Download,
    Repeat,
    Repeat1,
    SkipBack,
    SkipForward,
    Settings2,
    X,
  } from 'lucide-svelte';
  import type { AudioProgress, RepeatMode } from '$lib/types/database';
  import { cachedAudioInRange, cancelAudioDownload, downloadAudioRange } from '$lib/api/db';
  import AudioScrubber from './AudioScrubber.svelte';
  import { playbackStore } from '$lib/stores/playback.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';

  const repeatMode = $derived(settingsStore.current.audio_repeat_mode);
  const current = $derived(playbackStore.currentAyahId);

  let cachedCount = $state(0);
  let downloading = $state(false);
  let progress = $state<AudioProgress | null>(null);

  const total = $derived(playbackStore.queue.length);
  const complete = $derived(total > 0 && cachedCount >= total);

  /** How much of the queued Surah is already on disk. Recounted when the queue,
   *  the reciter or the quality changes, since each asks a different question
   *  of the cache. */
  $effect(() => {
    const reciter = playbackStore.reciter;
    const bitrate = settingsStore.current.audio_bitrate;
    const queue = playbackStore.queue;
    if (!reciter || queue.length === 0) {
      cachedCount = 0;
      return;
    }
    let cancelled = false;
    cachedAudioInRange(reciter.slug, bitrate, queue[0], queue[queue.length - 1])
      .then((ids) => {
        if (!cancelled) cachedCount = ids.length;
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  });

  onMount(() => {
    const unlisten = listen<AudioProgress>('audio-download-progress', (event) => {
      progress = event.payload;
    });
    return () => void unlisten.then((off) => off());
  });

  /** Off → this verse → the whole Surah → off. One button, because these are
   *  three answers to one question rather than three settings. */
  function cycleRepeat() {
    const next: Record<RepeatMode, RepeatMode> = { off: 'ayah', ayah: 'range', range: 'off' };
    void settingsStore.setAudioRepeatMode(next[repeatMode]);
  }

  const repeatLabel = $derived(
    repeatMode === 'ayah'
      ? `Repeating each verse ${settingsStore.current.audio_repeat_count}×`
      : repeatMode === 'range'
        ? 'Repeating the whole Surah'
        : 'Repeat off',
  );

  /** Fetch the queued Surah up front — before a journey, or a flight. */
  async function downloadAll() {
    const reciter = playbackStore.reciter;
    const queue = playbackStore.queue;
    if (!reciter || queue.length === 0) return;
    downloading = true;
    progress = null;
    try {
      const first = queue[0];
      const last = queue[queue.length - 1];
      await downloadAudioRange(reciter.slug, settingsStore.current.audio_bitrate, first, last);
      const ids = await cachedAudioInRange(
        reciter.slug,
        settingsStore.current.audio_bitrate,
        first,
        last,
      );
      cachedCount = ids.length;
    } catch (err) {
      console.error('Surah download failed', err);
    }
    downloading = false;
    progress = null;
  }

  function close() {
    playbackStore.stop();
    uiStore.closeListen();
  }
</script>

<aside class="listen-panel" aria-label="Listen">
  <header class="head">
    <div class="titles">
      <span class="title">{playbackStore.queueLabel ?? 'Listen'}</span>
      <span class="sub">
        {playbackStore.label ?? `${total} verses`} · {playbackStore.reciter?.name_en}
      </span>
    </div>
    <button
      class="icon-btn"
      onclick={() => uiStore.openSettings('audio')}
      aria-label="Audio settings"
      title="Reciter, quality, repeat"
    >
      <Settings2 size={15} />
    </button>
    <!-- Closing stops the recitation. A player that keeps going after its panel
         is gone is a sound with no visible cause. -->
    <button class="icon-btn" onclick={close} aria-label="Close and stop">
      <X size={15} />
    </button>
  </header>

  {#if current !== null}
    <AudioScrubber ayahId={current} range />
  {/if}

  <div class="row">
    <button
      class="icon-btn"
      onclick={() => playbackStore.previous()}
      aria-label="Previous verse"
      title="Previous verse"
    >
      <SkipBack size={15} />
    </button>
    <button
      class="icon-btn"
      onclick={() => playbackStore.next()}
      aria-label="Next verse"
      title="Next verse"
    >
      <SkipForward size={15} />
    </button>
    <button
      class="icon-btn"
      class:on={repeatMode !== 'off'}
      onclick={cycleRepeat}
      aria-label={repeatLabel}
      title={repeatLabel}
    >
      {#if repeatMode === 'ayah'}<Repeat1 size={15} />{:else}<Repeat size={15} />{/if}
    </button>

    <span class="spacer"></span>

    {#if downloading}
      <span class="progress">{progress ? `${progress.done}/${progress.total}` : 'Starting…'}</span>
      <button class="icon-btn" onclick={() => cancelAudioDownload()} aria-label="Stop downloading">
        <X size={15} />
      </button>
    {:else if complete}
      <!-- Stated rather than hidden: "will this work on the plane" is the
           question the button answers, and so is the answer. -->
      <span class="offline"><Check size={13} /> Offline</span>
    {:else}
      <button
        class="text-btn"
        onclick={downloadAll}
        title="Download all {total} verses for offline listening"
      >
        <Download size={13} />
        {cachedCount ? `${total - cachedCount} left` : 'Download'}
      </button>
    {/if}
  </div>
</aside>

<style>
  .listen-panel {
    position: absolute;
    right: calc(16px + var(--tafsir-inset, 0px));
    bottom: 16px;
    z-index: 8;
    display: flex;
    flex-direction: column;
    gap: 6px;
    width: min(340px, calc(100% - 32px));
    padding: 10px 12px 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-bg-elevated);
    box-shadow: 0 8px 28px rgb(0 0 0 / 0.28);
  }

  .head {
    display: flex;
    align-items: flex-start;
    gap: 4px;
  }

  .titles {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }

  .title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text);
  }

  .sub {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .row {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .spacer {
    flex: 1;
  }

  .icon-btn {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
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

  .icon-btn.on {
    color: var(--color-accent);
  }

  .text-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 4px 9px;
    border: 1px solid var(--color-border);
    border-radius: 999px;
    background: transparent;
    color: var(--color-text-muted);
    font-family: inherit;
    font-size: 11px;
    cursor: pointer;
  }

  .text-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .progress,
  .offline {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    color: var(--color-text-muted);
  }
</style>
