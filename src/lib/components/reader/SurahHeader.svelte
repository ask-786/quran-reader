<script lang="ts">
  import type { GlyphSpan, Surah } from '$lib/types/database';
  import { stripTashkeel } from '$lib/utils/arabic-text';
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
    <h2 class="surah-name">{stripTashkeel(surah.name_ar)}</h2>
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
        <span class="bismillah-fallback">{BISMILLAH_TEXT}</span>
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
    color: var(--color-accent);
  }

  /* ۞ is U+06DE, which the UI font has no glyph for — without an Arabic font
     named here it fell through to whatever system fallback the platform
     happened to pick, rendering as a generic asterisk rather than the
     rub-el-hizb ornament. */
  .motif {
    font-family: var(--font-surah-name);
    font-size: calc(13px * var(--reader-zoom));
    line-height: 1;
  }

  /* Deliberately not .quran-text: the banner is chrome announcing the Surah,
     not body text, so its size is the banner's own and must not track the
     Ayah font-size setting. (It used to carry .quran-text, which put it under
     ReaderView's `.reader-scroll :global(.quran-text)` font-size override in
     scroll mode but not in Mushaf mode — the same banner rendered at two
     different sizes depending on the view.) Live-shaped, so --font-surah-name
     rather than --font-quran; see app.css for why the two are separate. */
  .surah-name {
    margin: 0;
    font-family: var(--font-surah-name);
    font-size: calc(22px * var(--reader-zoom));
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

  /* Sized here for both the QCF glyph spans and the fallback below, so the
     basmala scales with the banner rather than with the Ayah text. */
  .bismillah {
    margin: 22px 0 0;
    font-size: calc(26px * var(--reader-zoom));
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
