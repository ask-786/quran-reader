<script lang="ts">
  import { progressStore } from '$lib/stores/progress.svelte';

  const TRACK_HEIGHT = 140;

  const percent = $derived(Math.round(progressStore.fraction * 100));
  const label = $derived(
    `${percent}% through this Surah` +
      (progressStore.juz ? ` · Juz ${progressStore.juz}` : '') +
      (progressStore.hizb ? ` · Hizb ${progressStore.hizb}` : ''),
  );
</script>

<div class="progress-indicator" title={label}>
  <div class="track" style:height="{TRACK_HEIGHT}px">
    <div class="fill" style:height="{percent}%"></div>
  </div>
  <div class="label">{percent}%</div>
</div>

<style>
  .progress-indicator {
    position: absolute;
    left: 10px;
    top: 50%;
    transform: translateY(-50%);
    z-index: 5;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }

  .track {
    position: relative;
    width: 4px;
    border-radius: 2px;
    background: var(--color-border);
    overflow: hidden;
  }

  .fill {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    background: var(--color-accent);
    transition: height 80ms linear;
  }

  .label {
    font-size: 10px;
    color: var(--color-text-faint);
    font-variant-numeric: tabular-nums;
  }
</style>
