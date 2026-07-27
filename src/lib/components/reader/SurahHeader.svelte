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
        <span class="bismillah-fallback quran-text">{BISMILLAH_TEXT}</span>
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

  /* ۞ is U+06DE, which the UI font has no glyph for — without an Arabic font
     named here it fell through to whatever system fallback the platform
     happened to pick, rendering as a generic asterisk rather than the
     rub-el-hizb ornament. */
  .motif {
    font-family: var(--font-surah-name);
    font-size: 13px;
    line-height: 1;
  }

  /* Live-shaped, so it needs --font-surah-name rather than the --font-quran
     that .quran-text supplies. See app.css for why the two are separate. */
  .surah-name {
    margin: 0;
    font-family: var(--font-surah-name);
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

  /* Only reached when the Surah's own page (and so its QCF basmala glyphs) is
     outside the fetched range — e.g. a Juz view opening mid-Surah. Live-shaped,
     and BISMILLAH_TEXT contains U+0670 in ٱلرَّحْمَٰنِ, so it needs the same
     font as the Surah name. */
  .bismillah-fallback {
    font-family: var(--font-surah-name);
  }
</style>
