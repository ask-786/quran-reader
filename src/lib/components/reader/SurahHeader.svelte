<script lang="ts">
  import type { Surah } from '$lib/types/database';
  import { BISMILLAH_TEXT, shouldShowBismillahHeader } from '$lib/utils/bismillah';

  let { surah }: { surah: Surah } = $props();
</script>

<div class="surah-header">
  <p class="eyebrow">Surah {surah.id} · {surah.revelation_type} · {surah.verses_count} verses</p>
  <h2 class="name-ar quran-text">{surah.name_ar}</h2>
  <p class="transliteration">{surah.transliteration}</p>

  {#if shouldShowBismillahHeader(surah)}
    <p class="bismillah quran-text">{BISMILLAH_TEXT}</p>
  {/if}
</div>

<style>
  .surah-header {
    text-align: center;
    padding: 32px 24px 24px;
    border-bottom: 1px solid var(--color-border);
    margin-bottom: 24px;
  }

  .eyebrow {
    margin: 0 0 12px;
    font-size: 12px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .name-ar {
    margin: 0;
    font-size: 40px;
    line-height: 1.3;
    direction: rtl;
  }

  .transliteration {
    margin: 6px 0 0;
    font-size: 14px;
    color: var(--color-text-muted);
  }

  .bismillah {
    margin: 24px 0 0;
    font-size: 26px;
    color: var(--color-accent);
  }
</style>
