<script lang="ts">
  import { tick } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { surahsStore } from '$lib/stores/surahs.svelte';

  const MAX_PAGE = 604;

  let value = $state('');
  let errorMsg = $state('');
  let inputEl = $state<HTMLInputElement>();

  $effect(() => {
    if (uiStore.goToOpen) {
      value = '';
      errorMsg = '';
      tick().then(() => inputEl?.focus());
    }
  });

  type Target =
    { type: 'ayah'; surahId: number; ayahNumber: number } | { type: 'page'; page: number };

  function parse(input: string): Target | null {
    const trimmed = input.trim();

    const pageMatch = trimmed.match(/^p\s*(\d{1,3})$/i);
    if (pageMatch) {
      return { type: 'page', page: Number(pageMatch[1]) };
    }

    const pair = trimmed.match(/^(\d{1,3})\s*[:./]\s*(\d{1,3})$/);
    if (pair) {
      return { type: 'ayah', surahId: Number(pair[1]), ayahNumber: Number(pair[2]) };
    }

    if (/^\d{1,3}$/.test(trimmed)) {
      const currentSurahId = Number($page.params.id);
      if (!currentSurahId) return null;
      return { type: 'ayah', surahId: currentSurahId, ayahNumber: Number(trimmed) };
    }

    return null;
  }

  function submit(e: SubmitEvent) {
    e.preventDefault();
    const target = parse(value);
    if (!target) {
      errorMsg = 'Enter an ayah, surah:ayah (2:255), or a page (p255)';
      return;
    }

    if (target.type === 'page') {
      if (target.page < 1 || target.page > MAX_PAGE) {
        errorMsg = `Page must be between 1 and ${MAX_PAGE}`;
        return;
      }
      uiStore.closeGoTo();
      uiStore.readingMode = 'mushaf';
      goto(resolve('/page/[id]', { id: String(target.page) }));
      return;
    }

    const surah = surahsStore.get(target.surahId);
    if (!surah) {
      errorMsg = `Surah ${target.surahId} doesn't exist`;
      return;
    }
    if (target.ayahNumber < 1 || target.ayahNumber > surah.verses_count) {
      errorMsg = `${surah.transliteration} has ${surah.verses_count} verses`;
      return;
    }
    uiStore.closeGoTo();
    goto(resolve(`/surah/[id]?ayah=${target.ayahNumber}`, { id: String(target.surahId) }));
  }
</script>

{#if uiStore.goToOpen}
  <div
    class="backdrop"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) uiStore.closeGoTo();
    }}
  >
    <form class="dialog" onsubmit={submit}>
      <label class="label" for="go-to-input">Go to</label>
      <input
        id="go-to-input"
        bind:this={inputEl}
        bind:value
        type="text"
        placeholder="e.g. 255, 2:255, or p255"
        class="input"
        autocomplete="off"
      />
      <p class="hint" class:error={!!errorMsg}>
        {errorMsg || 'Ayah in this surah, Surah:Ayah, or p + Page number'}
      </p>
    </form>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    display: flex;
    justify-content: center;
    padding-top: 18vh;
    background: rgba(0, 0, 0, 0.35);
    z-index: 100;
  }

  .dialog {
    width: min(360px, 90vw);
    height: fit-content;
    padding: 16px;
    border-radius: var(--radius);
    border: 1px solid var(--color-border);
    background: var(--color-bg-elevated);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.3);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .label {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .input {
    width: 100%;
    padding: 9px 10px;
    border-radius: var(--radius);
    border: 1px solid var(--color-border);
    background: var(--color-bg);
    color: var(--color-text);
    font-size: 15px;
  }

  .input:focus {
    outline: 2px solid var(--color-accent);
    outline-offset: -1px;
  }

  .hint {
    margin: 0;
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .hint.error {
    color: #d9736a;
  }
</style>
