<script lang="ts">
  /**
   * Shown once the reader reaches the end of An-Nas — the close of a Khatm.
   * A suggestion, not a ritual the app imposes: the text is the supplication
   * most Mushafs print after the last page, and the note under it says plainly
   * how strong its attribution is, so nobody takes it for more than it is.
   *
   * Collapsed behind a small button under An-Nas, so finishing the Surah on
   * its own (not a Khatm) isn't met with a card nobody asked for.
   */
  import { HandHeart } from 'lucide-svelte';

  let open = $state(false);

  const DUA_TEXT =
    'اللَّهُمَّ ارْحَمْنِي بِالْقُرْآنِ، وَاجْعَلْهُ لِي إِمَامًا وَنُورًا وَهُدًى وَرَحْمَةً، ' +
    'اللَّهُمَّ ذَكِّرْنِي مِنْهُ مَا نَسِيتُ، وَعَلِّمْنِي مِنْهُ مَا جَهِلْتُ، ' +
    'وَارْزُقْنِي تِلَاوَتَهُ آنَاءَ اللَّيْلِ وَأَطْرَافَ النَّهَارِ، ' +
    'وَاجْعَلْهُ لِي حُجَّةً يَا رَبَّ الْعَالَمِينَ';

  const DUA_TRANSLATION =
    'O Allah, have mercy on me through the Quran, and make it for me a leader, a light, ' +
    'guidance and mercy. O Allah, remind me of what I have forgotten of it, teach me what ' +
    'I do not know of it, grant me its recitation through the hours of the night and the ' +
    'ends of the day, and make it a proof for me, O Lord of the worlds.';
</script>

<div class="khatm-toggle">
  <button
    type="button"
    aria-expanded={open}
    aria-controls="khatm-dua"
    onclick={() => (open = !open)}
  >
    <HandHeart size={14} />
    {open ? 'Hide Khatm al-Quran dua' : 'Khatm al-Quran dua'}
  </button>
</div>

{#if open}
  <section id="khatm-dua" class="khatm" aria-labelledby="khatm-title">
    <p class="eyebrow">Khatm al-Quran</p>
    <h2 id="khatm-title">You have completed the Quran</h2>
    <p class="lead">May Allah accept it from you. A supplication you may make:</p>

    <p class="dua" dir="rtl" lang="ar">{DUA_TEXT}</p>
    <p class="translation">{DUA_TRANSLATION}</p>

    <p class="note">
      Printed at the end of many Mushafs. It is attributed to the Prophet ﷺ through a weak chain, so
      it is a recommended wording rather than a required one — any sincere supplication is good.
      Anas ibn Malik would gather his family when he completed the Quran and make dua (al-Darimi).
    </p>
  </section>
{/if}

<style>
  .khatm-toggle {
    display: flex;
    justify-content: center;
    margin-top: 24px;
  }

  .khatm-toggle button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border: 1px solid color-mix(in srgb, var(--color-accent) 40%, transparent);
    border-radius: 999px;
    background: var(--color-bg-elevated);
    color: var(--color-accent);
    font-family: var(--font-ui);
    font-size: 12px;
    cursor: pointer;
  }

  .khatm-toggle button:hover {
    background: var(--color-bg-hover);
  }

  .khatm {
    margin: 16px auto 0;
    padding: 28px 24px;
    border: 1px solid color-mix(in srgb, var(--color-accent) 40%, transparent);
    border-radius: 12px;
    background: var(--color-bg-elevated);
    color: var(--color-text);
    font-family: var(--font-ui);
    text-align: center;
  }

  .eyebrow {
    margin: 0;
    color: var(--color-accent);
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  h2 {
    margin: 6px 0 4px;
    font-size: 20px;
    font-weight: 600;
  }

  .lead {
    margin: 0;
    color: var(--color-text-muted);
    font-size: 14px;
  }

  /* Amiri via --font-arabic-prose: a supplication is running Arabic, not
     Mushaf text, so it gets the prose face rather than a QCF glyph font. */
  .dua {
    margin: 24px 0 16px;
    font-family: var(--font-arabic-prose);
    font-size: calc(var(--font-size-quran) * var(--reader-zoom, 1) * 0.85);
    line-height: 2.1;
  }

  .translation {
    margin: 0;
    color: var(--color-text);
    font-size: 15px;
    line-height: 1.6;
  }

  .note {
    margin: 20px 0 0;
    color: var(--color-text-faint);
    font-size: 12px;
    line-height: 1.5;
  }
</style>
