<script lang="ts">
  import { progressStore } from '$lib/stores/progress.svelte';

  const percent = $derived(Math.round(progressStore.fraction * 100));
  const label = $derived(
    `${percent}% through this Surah` +
      (progressStore.juz ? ` · Juz ${progressStore.juz}` : '') +
      (progressStore.hizb ? ` · Hizb ${progressStore.hizb}` : ''),
  );
</script>

<div class="progress-indicator" title={label}>
  <div class="fill" style:height="{percent}%"></div>
</div>

<style>
  .progress-indicator {
    position: absolute;
    left: 10px;
    top: 16px;
    bottom: 16px;
    width: 4px;
    border-radius: 2px;
    background: var(--color-border);
    overflow: hidden;
    z-index: 5;
  }

  .fill {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    background: var(--color-accent);
    transition: height 80ms linear;
  }
</style>
