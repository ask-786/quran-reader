<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { getLastReadingPosition } from '$lib/api/db';
  import { scopeHref } from '$lib/utils/reading-scope';

  onMount(async () => {
    // Continue reading: back to the *range* last read, not just the Surah it
    // happened to fall in. Reading Juz 22 or page 300 used to land you on a
    // Surah route on next launch, because the only thing remembered was a
    // Surah id — the range you were actually reading in was lost.
    let href = resolve('/surah/[id]', { id: '1' });
    try {
      const last = await getLastReadingPosition();
      if (last) href = scopeHref(last.scope, last.scope_id, last.ayah.id);
    } catch (err) {
      console.error('Failed to load the last reading position', err);
    }
    goto(href, { replaceState: true });
  });
</script>

<div class="loading">
  <p>Loading…</p>
</div>

<style>
  .loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-muted);
  }
</style>
