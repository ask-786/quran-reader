<script lang="ts">
  import type { Surah } from '$lib/types/database';
  import { BISMILLAH_TEXT, shouldShowBismillahHeader } from '$lib/utils/bismillah';
  import { toArabicNumerals } from '$lib/utils/arabic-numerals';

  let { surah, rukuCount }: { surah: Surah; rukuCount: number } = $props();

  const revelationLabelAr = $derived(surah.revelation_type === 'Makki' ? 'مكية' : 'مدنية');
</script>

<div class="surah-banner">
  <div class="banner-frame" dir="rtl">
    <span class="motif" aria-hidden="true">۞</span>

    <div class="banner-cell">
      <span class="roundel">{toArabicNumerals(surah.id)}</span>
    </div>

    <div class="banner-center">
      <span class="bracket" aria-hidden="true">﴾</span>
      <h2 class="surah-name quran-text">{surah.name_ar}</h2>
      <span class="bracket" aria-hidden="true">﴿</span>
    </div>

    <div class="banner-cell">
      <span class="roundel">{toArabicNumerals(surah.verses_count)}</span>
    </div>
  </div>

  <p class="subtitle">
    {surah.transliteration} · {surah.revelation_type} · {surah.verses_count} verses · {rukuCount} ruku'
    <span class="revelation-chip">{revelationLabelAr}</span>
  </p>

  {#if shouldShowBismillahHeader(surah)}
    <p class="bismillah quran-text">
      <span class="bracket small" aria-hidden="true">﴾</span>
      {BISMILLAH_TEXT}
      <span class="bracket small" aria-hidden="true">﴿</span>
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
    position: relative;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 14px 22px;
    border-top: 1px solid var(--color-accent);
    border-bottom: 1px solid var(--color-accent);
  }

  .banner-frame::before {
    content: '';
    position: absolute;
    inset: 3px 0;
    border-top: 1px solid var(--color-accent);
    border-bottom: 1px solid var(--color-accent);
    opacity: 0.5;
    pointer-events: none;
  }

  .motif {
    position: absolute;
    top: -0.62em;
    left: 50%;
    transform: translateX(-50%);
    background: var(--color-bg);
    padding: 0 8px;
    color: var(--color-accent);
    font-size: 15px;
    line-height: 1;
  }

  .banner-cell {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 0 0 auto;
  }

  .roundel {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    border-radius: 50%;
    border: 1px solid var(--color-accent);
    color: var(--color-accent);
    font-family: var(--font-quran);
    font-size: 15px;
  }

  .banner-center {
    display: flex;
    align-items: baseline;
    justify-content: center;
    gap: 8px;
    flex: 1;
    min-width: 0;
  }

  .bracket {
    color: var(--color-accent);
    font-size: 22px;
    line-height: 1;
  }

  .bracket.small {
    font-size: 18px;
    margin: 0 0.15em;
  }

  .surah-name {
    margin: 0;
    font-size: 26px;
    line-height: 1.3;
    white-space: nowrap;
  }

  .subtitle {
    margin: 14px 0 0;
    font-size: 12px;
    letter-spacing: 0.02em;
    color: var(--color-text-muted);
  }

  .revelation-chip {
    display: inline-block;
    margin-inline-start: 8px;
    padding: 1px 8px;
    border-radius: 999px;
    border: 1px solid var(--color-border);
    font-family: var(--font-quran);
    font-size: 12px;
  }

  .bismillah {
    margin: 22px 0 0;
    font-size: 26px;
    color: var(--color-accent);
  }
</style>
