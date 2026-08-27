<!--
  Al-Fatihah 1–4, set exactly as the reader sets it.

  The same page-glyph words from the same QCF v4 font group the Mushaf itself
  renders from, carrying `.quran-text` so the size, leading and measure come
  from the very custom properties the sliders write. A specimen in some other
  face would answer a different question from the one being asked — these
  glyphs have their own advances and their own mark heights, and choosing a
  text size against Amiri would tell you nothing about how a page will land.

  Page 1 rather than any page: its font group, QCF4_Hafs_01, is the one the
  loader pins for the Basmala and never evicts, so previewing costs at most one
  fetch that the reader was going to make anyway.

  Reader zoom is deliberately not applied. This shows the base size the slider
  sets; zoom multiplies it per view, and folding it in here would put a number
  under the slider that did not match the text above it.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getPage } from '$lib/api/db';
  import { ensurePageFont, familyForPage, loadFontMap } from '$lib/utils/mushaf-fonts';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { hasVerseMarker, withoutVerseMarker } from '$lib/utils/ayah-marker';
  import type { PageLineWord } from '$lib/types/database';

  const SPECIMEN_PAGE = 1;
  /**
   * Lines 2–4 of the page — verses 1 to 4, ending cleanly on a verse marker.
   * Long enough to wrap at every text size, which is what gives the leading
   * slider something to show, and short enough not to turn the panel into a
   * second reader.
   */
  const SPECIMEN_LINES = 3;

  let words = $state<PageLineWord[]>([]);
  let family = $state<string | null>(null);

  onMount(async () => {
    try {
      await loadFontMap();
      const [page] = await Promise.all([getPage(SPECIMEN_PAGE), ensurePageFont(SPECIMEN_PAGE)]);
      words = page.lines
        .filter((line) => line.line_type === 'text')
        .slice(0, SPECIMEN_LINES)
        .flatMap((line) => line.words);
      // Only after the font is registered: the glyphs are Private Use Area
      // codepoints, so painting them in any other family renders tofu.
      family = familyForPage(SPECIMEN_PAGE) ?? null;
    } catch (err) {
      console.error('Failed to load the typography specimen', err);
    }
  });

  const shown = $derived(
    settingsStore.current.show_ayah_numbers
      ? words
      : words.map((w) => (hasVerseMarker(w) ? withoutVerseMarker(w) : w)),
  );
</script>

<!-- aria-hidden: this is a picture of a typographic setting, not something to
     read. The reader is one keystroke away and says the same words. -->
<div class="specimen" aria-hidden="true">
  {#if family}
    <p class="specimen-text quran-text" style:font-family={family}>
      {#each shown as w, i (i)}<span class="word">{w.glyph_v4}</span>
      {/each}
    </p>
  {:else}
    <!-- Holds the box open at roughly its filled height, so the controls below
         do not jump down when the glyphs land. -->
    <p class="loading"></p>
  {/if}
</div>

<style>
  .specimen {
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 4px;
    padding: 14px 16px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-bg);
  }

  /* A measure in the text's own em, so the specimen wraps to about three lines
     at every size. Without it the passage fits on one line at the small end of
     the slider and the leading control looks inert. */
  .specimen-text {
    max-width: 13em;
    margin: 0;
    text-align: center;
    color: var(--color-text);
    /* Both carried from AyahRow, and both load-bearing there: see its note on
       QCF v4's Private Use Area glyphs being bidi class L, which without the
       override lays the whole passage out backwards. */
    unicode-bidi: bidi-override;
    overflow-wrap: anywhere;
  }

  .loading {
    margin: 0;
    /* Three lines of Quran text at the current setting. */
    height: calc(var(--font-size-quran) * var(--line-height-quran) * 3);
  }

  /* The QCF fonts' own space is much narrower than a live-shaped Arabic font
     would give — the same explicit padding AyahRow applies for the same
     reason. */
  .word {
    margin-inline-end: 0.35em;
  }
</style>
