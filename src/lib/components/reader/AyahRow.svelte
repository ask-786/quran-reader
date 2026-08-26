<script lang="ts">
  import { Bookmark, BookmarkCheck, ScrollText, Copy, Check } from 'lucide-svelte';
  import type { Ayah, AyahGlyphWord } from '$lib/types/database';
  import { bookmarksStore } from '$lib/stores/bookmarks.svelte';
  import { tafsirStore } from '$lib/stores/tafsir.svelte';

  let {
    ayah,
    words,
    reservedHeight = null,
    translation = null,
  }: {
    ayah: Ayah;
    words: AyahGlyphWord[];
    /**
     * When this row is outside the render window its glyphs are dropped, and
     * this holds the space they occupied so nothing below it moves. Null while
     * the row is rendered, when the content sets its own height.
     */
    reservedHeight?: number | null;
    translation?: string | null;
  } = $props();

  const bookmarked = $derived(bookmarksStore.isBookmarked(ayah.id));
  let copied = $state(false);

  async function copyText() {
    await navigator.clipboard.writeText(ayah.uthmani_text);
    copied = true;
    setTimeout(() => (copied = false), 1200);
  }

  function tafsirOnClick(node: HTMLElement) {
    const onClick = () => {
      // A click that ends a drag-selection is the user selecting text to copy,
      // not asking for commentary.
      if (!window.getSelection()?.isCollapsed) return;
      tafsirStore.openFromClick(ayah.id, node);
    };
    node.addEventListener('click', onClick);
    return {
      destroy() {
        node.removeEventListener('click', onClick);
      },
    };
  }
</script>

<div
  class="ayah-row"
  id="ayah-{ayah.id}"
  data-ayah-id={ayah.id}
  data-page={ayah.page}
  data-juz={ayah.juz}
  style:height={reservedHeight === null ? null : `${reservedHeight}px`}
>
  <!-- Outside the render window the row keeps its measured height (set above)
       and drops everything inside it. Rendering the translation or the action
       buttons here would overflow that reserved box, which is the one thing
       windowing must never do. -->
  {#if reservedHeight === null}
    <div class="ayah-actions">
      <button
        class="action-btn"
        class:active={bookmarked}
        onclick={() => bookmarksStore.toggle(ayah.id)}
        aria-pressed={bookmarked}
        aria-label={bookmarked ? 'Remove bookmark' : 'Add bookmark'}
      >
        {#if bookmarked}<BookmarkCheck size={15} />{:else}<Bookmark size={15} />{/if}
      </button>
      <button class="action-btn" onclick={copyText} aria-label="Copy ayah text">
        {#if copied}<Check size={15} />{:else}<Copy size={15} />{/if}
      </button>
      <button
        class="action-btn"
        onclick={(e) => tafsirStore.openFor(ayah.id, e.currentTarget)}
        aria-label="Show tafsir for this ayah"
        title="Tafsir"
      >
        <ScrollText size={15} />
      </button>
    </div>

    <!-- Click the verse, get its commentary. Deliberately NOT `role="button"`
         with a key handler: this is running Quranic text, and announcing a
         verse to a screen reader as a button would be a worse reading
         experience than the shortcut is worth. The keyboard and AT route is
         the labelled button above, plus `t`. Hence an imperative listener
         through an action, which claims nothing about what this element is. -->
    <p class="ayah-text quran-text" class:clickable={tafsirStore.clickOpens} use:tafsirOnClick>
      {#each words as w, i (i)}<span
          class="word"
          style:font-family={w.fontFamily}
          aria-label={w.uthmani_text}
          title={w.uthmani_text}>{w.glyph_v4}</span
        >
      {/each}
    </p>

    {#if translation}
      <p class="ayah-translation">{translation}</p>
    {/if}
  {/if}
</div>

<style>
  .ayah-row {
    position: relative;
    padding: 10px 8px 14px;
    border-radius: var(--radius);
    transition: background var(--transition);
  }

  .ayah-row:hover,
  .ayah-row:focus-within {
    background: var(--color-bg-hover);
  }

  .ayah-actions {
    position: absolute;
    top: 6px;
    left: 8px;
    display: flex;
    gap: 4px;
    opacity: 0;
    transition: opacity var(--transition);
  }

  .ayah-row:hover .ayah-actions,
  .ayah-row:focus-within .ayah-actions {
    opacity: 1;
  }

  /* Touch devices have no hover state, so the actions must stay visible —
     otherwise they'd only ever appear via keyboard focus. */
  @media (hover: none) {
    .ayah-actions {
      opacity: 1;
    }
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: 6px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .action-btn:hover {
    background: var(--color-border);
    color: var(--color-text);
  }

  .action-btn.active {
    color: var(--color-accent);
  }

  .ayah-text {
    margin: 0;
    /* Words are individual glyph spans with a single literal space between
       them (see markup) rather than free-flowing text, so a pathologically
       long single word still has somewhere to break instead of overflowing. */
    overflow-wrap: anywhere;
    /* Load-bearing, not cosmetic. QCF v4's glyphs live in the Private Use
       Area, whose Unicode bidi class is L (strong left-to-right) — unlike
       QCF v2's Arabic Presentation Forms, which are class AL and got ordered
       right-to-left by the browser for free. Without this override every word
       span here resolves into one continuous LTR run (the spaces between them
       are neutrals flanked by L, so they join it), which renders the whole
       ayah backwards. `direction: rtl` alone does NOT fix that — it sets the
       paragraph direction, but strong-L characters still lay out LTR inside
       it. The override forces every character to be treated as RTL, which
       restores both the word order and the verse-marker position within each
       span. See docs/qcf-v4-font-migration-plan.md, risk 2. */
    unicode-bidi: bidi-override;
  }

  /* The only sign that a click will do something. Without it tafsir mode is
     invisible until you happen to click, which is the same guesswork the mode
     exists to remove. */
  .ayah-text.clickable {
    cursor: pointer;
  }

  .word {
    /* The QCF glyph fonts' own space character is much narrower than the
       gap a live-shaped Arabic font would give — pad it out explicitly so
       words in the flowing (non-justified) list view don't read as cramped. */
    margin-inline-end: 0.35em;
    cursor: default;
  }

  .word:hover {
    color: var(--color-accent);
  }

  .ayah-translation {
    margin: 8px 4px 0;
    font-size: 15px;
    line-height: 1.7;
    color: var(--color-text-muted);
    direction: ltr;
    text-align: left;
  }
</style>
