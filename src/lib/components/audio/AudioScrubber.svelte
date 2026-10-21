<!--
  Play, pause, and move around inside one verse.

  A bar rather than a bare play button, because a verse is not a uniform unit of
  time: al-Fatiha 1 is about five seconds and 2:282 runs for minutes. Someone
  who stopped to check how one phrase is pronounced should be able to get to
  that phrase, not sit through the whole of the longest āya in the Quran.

  Used by the tafsir card (popover and panel), where it plays one verse and
  stops, and by the listen panel, where the same controls sit over a queue.
  There the bar can instead span the whole Surah, while it streams as one
  recording.
-->
<script lang="ts">
  import { Loader, Pause, Play } from 'lucide-svelte';
  import { playbackStore } from '$lib/stores/playback.svelte';
  import AudioUnavailable from './AudioUnavailable.svelte';

  let {
    ayahId,
    /** Range mode plays on through the queue; the default plays this verse and
     *  stops, which is what a clarification is. */
    range = false,
    /** Span the Surah's recording rather than the verse. Falls back to the
     *  verse whenever the Surah is not streaming, since then there is no one
     *  file to have a position in. */
    surah = false,
  }: { ayahId: number; range?: boolean; surah?: boolean } = $props();

  const isCurrent = $derived(playbackStore.isCurrent(ayahId));
  const playing = $derived(playbackStore.isPlaying(ayahId));
  const loading = $derived(playbackStore.loading && isCurrent);
  const blocked = $derived(playbackStore.needsPermission && isCurrent);
  /** Not merely disabled for tidiness: pressing play here hangs the webview. */
  const unplayable = $derived(!playbackStore.playable);

  /** The track is dead until this verse is the loaded one with a known length —
   *  a slider that cannot move is worse than one that is visibly not ready. */
  const whole = $derived(surah && playbackStore.surahLength > 0);
  const seekable = $derived(whole || (isCurrent && playbackStore.duration > 0));
  const position = $derived(
    whole ? playbackStore.surahTime : isCurrent ? playbackStore.currentTime : 0,
  );
  const length = $derived(
    whole ? playbackStore.surahLength : isCurrent ? playbackStore.duration : 0,
  );

  /**
   * Where a drag along the whole-Surah bar is, before it is let go. Seeking on
   * every step would scroll the reader through each verse passed over on the
   * way, and ask the host for a new piece of the file each time; one verse's
   * bar seeks as it goes, because there the audio is already in memory and
   * hearing it move is the point.
   */
  let scrub = $state<number | null>(null);
  // Only the Surah bar reads it. A drag whose release went unseen would
  // otherwise leave the verse bar showing that drag's position for good,
  // while its seeks went through underneath.
  const shown = $derived(whole && scrub !== null ? scrub : position);
  /** How far along the bar is shown, for placing the landing verse over it. */
  const fraction = $derived(length > 0 ? Math.min(Math.max(shown / length, 0), 1) : 0);
  const landing = $derived(whole && scrub !== null ? playbackStore.verseNumberAt(scrub) : null);

  function onInput(value: number) {
    if (whole) scrub = value;
    else playbackStore.seek(value);
  }

  function onChange(value: number) {
    scrub = null;
    if (whole) playbackStore.seekSurah(value);
  }

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
    disabled={unplayable}
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

  <!-- The landing verse floats over the track rather than sitting beside it:
       beside it, it took its width from the bar mid-drag, and the bar shrank
       under the pointer that was dragging it. -->
  <div class="track-wrap">
    <input
      class="track"
      type="range"
      min="0"
      max={length || 1}
      step="1"
      value={shown}
      disabled={!seekable || unplayable}
      aria-label={whole ? 'Position in this Surah' : 'Position in this verse'}
      oninput={(e) => onInput(Number(e.currentTarget.value))}
      onchange={(e) => onChange(Number(e.currentTarget.value))}
    />
    {#if landing !== null}
      <span
        class="landing"
        style:left="{fraction * 100}%"
        style:translate="{fraction * -100}% 0"
        aria-hidden="true"
      >
        Verse {landing}
      </span>
    {/if}
  </div>

  <!-- Wide enough for the longest it can read, so the bar does not twitch as
       9:59 becomes 10:00. -->
  <span class="time" style:min-width="{clock(length).length * 2 + 3}ch">
    {clock(shown)} / {clock(length)}
  </span>
</div>

{#if unplayable}
  <AudioUnavailable compact />
{:else if blocked}
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

  .track-wrap {
    position: relative;
    flex: 1;
    min-width: 40px;
    margin: 0 6px;
  }

  .track {
    display: block;
    width: 100%;
    margin: 0;
    accent-color: var(--color-accent);
    cursor: pointer;
  }

  .track:disabled {
    opacity: 0.4;
    cursor: default;
  }

  /* Above the thumb. Shifted back by as much of its own width as the thumb is
     along the track, so it is never clipped at either end of the card. */
  .landing {
    position: absolute;
    bottom: calc(100% + 4px);
    padding: 2px 6px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg-elevated);
    color: var(--color-accent);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
    pointer-events: none;
  }

  .time {
    flex-shrink: 0;
    text-align: right;
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
