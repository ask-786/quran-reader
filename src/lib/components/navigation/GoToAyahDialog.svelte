<script lang="ts">
  import { tick } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { surahsStore } from '$lib/stores/surahs.svelte';

  let value = $state('');
  let errorMsg = $state('');
  let inputEl = $state<HTMLInputElement>();

  $effect(() => {
    if (uiStore.goToAyahOpen) {
      value = '';
      errorMsg = '';
      tick().then(() => inputEl?.focus());
    }
  });

  function parse(input: string): { surahId: number; ayahNumber: number } | null {
    const trimmed = input.trim();
    const pair = trimmed.match(/^(\d{1,3})\s*[:./]\s*(\d{1,3})$/);
    if (pair) {
      return { surahId: Number(pair[1]), ayahNumber: Number(pair[2]) };
    }
    if (/^\d{1,3}$/.test(trimmed)) {
      const currentSurahId = Number($page.params.id);
      if (!currentSurahId) return null;
      return { surahId: currentSurahId, ayahNumber: Number(trimmed) };
    }
    return null;
  }

  function submit(e: SubmitEvent) {
    e.preventDefault();
    const parsed = parse(value);
    if (!parsed) {
      errorMsg = 'Enter an ayah number, or surah:ayah (e.g. 2:255)';
      return;
    }
    const surah = surahsStore.get(parsed.surahId);
    if (!surah) {
      errorMsg = `Surah ${parsed.surahId} doesn't exist`;
      return;
    }
    if (parsed.ayahNumber < 1 || parsed.ayahNumber > surah.verses_count) {
      errorMsg = `${surah.transliteration} has ${surah.verses_count} verses`;
      return;
    }
    uiStore.closeGoToAyah();
    goto(resolve(`/surah/[id]?ayah=${parsed.ayahNumber}`, { id: String(parsed.surahId) }));
  }
</script>

{#if uiStore.goToAyahOpen}
  <div
    class="backdrop"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) uiStore.closeGoToAyah();
    }}
  >
    <form class="dialog" onsubmit={submit}>
      <label class="label" for="go-to-ayah-input">Go to Ayah</label>
      <input
        id="go-to-ayah-input"
        bind:this={inputEl}
        bind:value
        type="text"
        inputmode="numeric"
        placeholder="e.g. 255 or 2:255"
        class="input"
        autocomplete="off"
      />
      <p class="hint" class:error={!!errorMsg}>
        {errorMsg || 'Ayah number in this surah, or Surah:Ayah'}
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
