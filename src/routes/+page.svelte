<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { settingsStore } from '$lib/stores/settings.svelte';

  onMount(async () => {
    await settingsStore.init();
    const id = String(settingsStore.current.last_read_surah_id);
    goto(resolve('/surah/[id]', { id }), { replaceState: true });
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
