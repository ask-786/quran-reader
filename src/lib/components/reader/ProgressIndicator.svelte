<script lang="ts">
  import { progressStore } from '$lib/stores/progress.svelte';

  const percent = $derived(Math.round(progressStore.fraction * 100));
  const label = $derived(
    `${percent}% through ${progressStore.scopeText}` +
      (progressStore.juz && !progressStore.scopeText.startsWith('Juz')
        ? ` · Juz ${progressStore.juz}`
        : '') +
      (progressStore.hizb && !progressStore.scopeText.startsWith('Hizb')
        ? ` · Hizb ${progressStore.hizb}`
        : ''),
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

  /* Sit flush against the window edge instead of floating 10px in — the
     reading column's side padding (see --reader-side-padding) is widened
     at this breakpoint to keep text clear of it. */
  @media (max-width: 900px) {
    .progress-indicator {
      left: 0;
    }
  }

  @media (max-width: 480px) {
    .progress-indicator {
      display: none;
    }
  }
</style>
