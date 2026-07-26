<script lang="ts">
  import type { GlyphSpan, Surah } from '$lib/types/database';
  import { BISMILLAH_TEXT, shouldShowBismillahHeader } from '$lib/utils/bismillah';

  let {
    surah,
    basmalaWords = [],
    basmalaFontFamily = null,
  }: {
    surah: Surah;
    basmalaWords?: GlyphSpan[];
    basmalaFontFamily?: string | null;
  } = $props();
</script>

<div class="surah-banner" dir="rtl">
  <div class="banner-frame">
    <span class="motif" aria-hidden="true">۞</span>
    <h2 class="surah-name quran-text">{surah.name_ar}</h2>
    <span class="motif" aria-hidden="true">۞</span>
  </div>

  <p class="subtitle">
    {surah.transliteration} · {surah.revelation_type} · {surah.verses_count} verses
  </p>

  {#if shouldShowBismillahHeader(surah)}
    <p class="bismillah" style:font-family={basmalaWords.length ? basmalaFontFamily : null}>
      {#if basmalaWords.length}
        {#each basmalaWords as w, i (i)}<span aria-label={w.uthmani_text}>{w.glyph_v2}</span>
        {/each}
      {:else}
        <span class="quran-text">{BISMILLAH_TEXT}</span>
      {/if}
    </p>
  {/if}
</div>

<style>
  .surah-banner {
    text-align: center;
    padding: 32px 8px 20px;
    margin-bottom: 20px;
  }

  .banner-frame {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 8px 0;
    border-top: 1px solid var(--color-accent);
    border-bottom: 1px solid var(--color-accent);
    color: var(--color-accent);
  }

  .motif {
    font-size: 13px;
    line-height: 1;
  }

  .surah-name {
    margin: 0;
    font-size: 22px;
    font-weight: 400;
    line-height: 1.3;
    white-space: nowrap;
    color: var(--color-accent);
  }

  .subtitle {
    margin: 14px 0 0;
    font-size: 12px;
    letter-spacing: 0.02em;
    color: var(--color-text-muted);
  }

  .bismillah {
    margin: 22px 0 0;
    font-size: 26px;
    color: var(--color-accent);
  }
</style>
