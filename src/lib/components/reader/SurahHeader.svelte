<script lang="ts">
  import { Headphones } from 'lucide-svelte';
  import type { GlyphSpan, Surah } from '$lib/types/database';
  import { BISMILLAH_TEXT, shouldShowBismillahHeader } from '$lib/utils/bismillah';
  import { playbackStore } from '$lib/stores/playback.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';

  let {
    surah,
    basmalaWords = [],
    basmalaFontFamily = null,
    glyphsPending = false,
  }: {
    surah: Surah;
    basmalaWords?: GlyphSpan[];
    basmalaFontFamily?: string | null;
    // Set while the caller's glyph fetch is still in flight, so an empty
    // basmalaWords means "not here yet" rather than "not available" — see the
    // fallback note in the style block below.
    glyphsPending?: boolean;
  } = $props();

  /**
   * Listening to the Surah starts here and nowhere in the text.
   *
   * The banner is chrome announcing which Surah this is, so "hear this Surah"
   * is the same kind of statement about the same thing — and putting it here
   * keeps every transport control out of the reading column, where the whole
   * point is that nothing competes with the text.
   */
  function listen() {
    if (!playbackStore.enabled) {
      uiStore.openSettings('audio');
      return;
    }
    uiStore.openListen();
    void playbackStore.playSurah(surah.id);
  }
</script>

<div class="surah-banner" dir="rtl">
  <div class="banner-frame">
    <span class="motif" aria-hidden="true">۞</span>
    <!-- Vocalised, as stored. It was stripped while --font-surah-name was
         Scheherazade, which the WebKitGTK mark bug left misplacing 59% of
         these marks; under Amiri that is 2%. See app.css. The sidebar's copy
         in NavPanel is still stripped — that is a dense 17px list at
         line-height 1, where restoring marks means re-pitching every row. -->
    <h2 class="surah-name">{surah.name_ar}</h2>
    <span class="motif" aria-hidden="true">۞</span>
  </div>

  <p class="subtitle">
    {surah.transliteration} · {surah.revelation_type} · {surah.verses_count} verses
  </p>

  <div class="listen-slot" dir="ltr">
    <button class="listen" onclick={listen} title="Listen to this Surah">
      <Headphones size={13} />
      Listen
    </button>
  </div>

  {#if shouldShowBismillahHeader(surah) && (basmalaWords.length || !glyphsPending)}
    <p class="bismillah" style:font-family={basmalaWords.length ? basmalaFontFamily : null}>
      {#if basmalaWords.length}
        {#each basmalaWords as w, i (i)}<span class="glyph" aria-label={w.uthmani_text}
            >{w.glyph_v4}</span
          >
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
    /* Comfortably above the Bismillah's 26px below it — at the old 22px the
       banner's title was the smaller of the two and read as a label rather
       than a heading.

       The vw cap exists because of `nowrap`: the longest name ("سورة
       المطففين") at reader zoom 2 would otherwise run past a phone-width
       window rather than being allowed to wrap. It only binds above roughly
       zoom 1.2 on narrow viewports; at zoom 1 the px value always wins. */
    font-size: min(calc(32px * var(--reader-zoom)), 13vw);
    font-weight: 400;
    /* Room for the marks the name carries again: a kasra drops below the
       baseline and a damma sits well above it, and 1.3 was set for a line
       with neither. */
    line-height: 1.5;
    white-space: nowrap;
    color: var(--color-accent);
  }

  .subtitle {
    margin: 14px 0 0;
    font-size: 12px;
    letter-spacing: 0.02em;
    color: var(--color-text-muted);
  }

  /* Faint until wanted. It sits in the banner because the banner is chrome,
     but it is still the one control in this app that makes a sound, and it
     should not be the loudest thing under the Surah's name. */
  .listen-slot {
    display: flex;
    justify-content: center;
    margin-top: 10px;
  }

  .listen {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 3px 10px;
    border: 1px solid var(--color-border);
    border-radius: 999px;
    background: transparent;
    color: var(--color-text-muted);
    font-family: inherit;
    font-size: 11px;
    letter-spacing: 0.02em;
    cursor: pointer;
    opacity: 0.7;
    transition: opacity var(--transition);
  }

  .listen:hover,
  .listen:focus-visible {
    opacity: 1;
    color: var(--color-text);
    border-color: var(--color-accent);
  }

  /* Sized here for both the QCF glyph spans and the fallback below, so the
     basmala scales with the banner rather than with the Ayah text. */
  .bismillah {
    margin: 22px 0 0;
    font-size: calc(26px * var(--reader-zoom));
    color: var(--color-accent);
  }

  /* QCF v4 renders the Basmala as one Private Use Area glyph, bidi class L.
     A single glyph can't be mis-ordered on its own, but the override keeps
     this consistent with the word spans elsewhere and stays correct if a
     future font version splits the phrase across several glyphs (QCF v2
     already used three). See AyahRow's .ayah-text for the full reasoning. */
  .glyph {
    unicode-bidi: bidi-override;
  }

  /* Only reached when the Surah's own page (and so its QCF basmala glyphs) is
     outside the fetched range — e.g. a Juz view opening mid-Surah. Live-shaped,
     and BISMILLAH_TEXT contains U+0670 in ٱلرَّحْمَٰنِ, so it needs the same
     font as the Surah name.

     Callers that render before their glyph fetch resolves (ReaderView, whose
     list paints straight off its `ayahs` prop) must pass glyphsPending — this
     branch otherwise paints for a frame and then swaps to the QCF glyphs,
     which reads as the Bismillah blinking between two fonts. */
  .bismillah-fallback {
    font-family: var(--font-surah-name);
  }
</style>
