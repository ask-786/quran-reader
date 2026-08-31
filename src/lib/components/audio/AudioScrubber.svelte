<!--
  Play, pause, and move around inside one verse.

  A bar rather than a bare play button, because a verse is not a uniform unit of
  time: al-Fatiha 1 is about five seconds and 2:282 runs for minutes. Someone
  who stopped to check how one phrase is pronounced should be able to get to
  that phrase, not sit through the whole of the longest āya in the Quran.

  Used by the tafsir card (popover and panel), where it plays one verse and
  stops, and by the listen panel, where the same controls sit over a queue.
-->
<script lang="ts">
  import { Loader, Pause, Play } from 'lucide-svelte';
  import { playbackStore } from '$lib/stores/playback.svelte';

  let {
    ayahId,
    /** Range mode plays on through the queue; the default plays this verse and
     *  stops, which is what a clarification is. */
    range = false,
  }: { ayahId: number; range?: boolean } = $props();

  const isCurrent = $derived(playbackStore.isCurrent(ayahId));
  const playing = $derived(playbackStore.isPlaying(ayahId));
  const loading = $derived(playbackStore.loading && isCurrent);
  const blocked = $derived(playbackStore.needsPermission && isCurrent);

  /** The track is dead until this verse is the loaded one with a known length —
   *  a slider that cannot move is worse than one that is visibly not ready. */
  const seekable = $derived(isCurrent && playbackStore.duration > 0);
  const position = $derived(isCurrent ? playbackStore.currentTime : 0);
  const length = $derived(isCurrent ? playbackStore.duration : 0);

  function toggle() {
    if (playing) {
      playbackStore.pause();
      return;
    }
    if (isCurrent && playbackStore.currentTime > 0) {
      void playbackStore.resume();
      return;
    }
    if (range) void playbackStore.playRange(ayahId);
    else void playbackStore.playOne(ayahId);
  }

  function clock(seconds: number): string {
    if (!Number.isFinite(seconds) || seconds < 0) return '0:00';
    const whole = Math.floor(seconds);
    return `${Math.floor(whole / 60)}:${String(whole % 60).padStart(2, '0')}`;
  }
</script>

<div class="scrubber">
  <button
    class="icon-btn play"
    class:on={playing}
    onclick={toggle}
    aria-label={playing ? 'Pause' : 'Play this verse'}
  >
    {#if loading}
      <Loader size={15} class="spin" />
    {:else if playing}
      <Pause size={15} />
    {:else}
      <Play size={15} />
    {/if}
  </button>

  <input
    class="track"
    type="range"
    min="0"
    max={length || 1}
    step="1"
    value={position}
    disabled={!seekable}
    aria-label="Position in this verse"
    oninput={(e) => playbackStore.seek(Number(e.currentTarget.value))}
  />

  <span class="time">{clock(position)} / {clock(length)}</span>
</div>

{#if blocked}
  <p class="note">Not downloaded, and downloads are off in Settings → Audio.</p>
{:else if isCurrent && playbackStore.error}
  <p class="note error">{playbackStore.error}</p>
{/if}

<style>
  .scrubber {
    display: flex;
    align-items: center;
    gap: 2px;
    min-width: 0;
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

  .icon-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .icon-btn:disabled {
    opacity: 0.35;
    cursor: default;
  }

  .icon-btn.on {
    color: var(--color-accent);
  }

  .track {
    flex: 1;
    min-width: 40px;
    margin: 0 6px;
    accent-color: var(--color-accent);
    cursor: pointer;
  }

  .track:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .time {
    flex-shrink: 0;
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    color: var(--color-text-muted);
  }

  .note {
    margin: 4px 0 0;
    font-size: 11px;
    line-height: 1.4;
    color: var(--color-text-muted);
  }

  .note.error {
    color: var(--color-danger, #e5484d);
  }

  :global(.spin) {
    animation: spin 900ms linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
