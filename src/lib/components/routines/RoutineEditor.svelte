<!--
  Making and changing a routine: its name, which Surahs, and in what order.

  A dialog rather than a route, like settings: a route would unmount the reader,
  and someone setting up a routine while reading would come back to the top of
  the Surah. The order is the point of a routine, so it is edited where it is
  shown — the list on the left is the routine, top to bottom, and the picker on
  the right only adds to it.
-->
<script lang="ts">
  import { tick, untrack } from 'svelte';
  import { ArrowDown, ArrowUp, Check, Plus, Trash2, X } from 'lucide-svelte';
  import { filterSurahs } from '$lib/navigation/navigator.svelte';
  import { routinesStore } from '$lib/stores/routines.svelte';
  import { surahsStore } from '$lib/stores/surahs.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { stripTashkeel } from '$lib/utils/arabic-text';

  let name = $state('');
  let surahIds = $state<number[]>([]);
  let query = $state('');
  let error = $state('');
  let saving = $state(false);
  let confirmingDelete = $state(false);

  let nameEl = $state<HTMLInputElement>();
  let listEl = $state<HTMLOListElement>();

  const editing = $derived(uiStore.routineEditor);
  const routine = $derived(editing?.id != null ? (routinesStore.get(editing.id) ?? null) : null);

  // Fresh state each time the dialog opens, from the routine it opened on.
  // Keyed on the request object alone, which uiStore replaces on every open —
  // the rest is untracked so a routine list refreshing underneath an open
  // editor can't wipe what is being typed.
  $effect(() => {
    const request = uiStore.routineEditor;
    if (!request) return;
    untrack(() => {
      const source = request.id != null ? routinesStore.get(request.id) : undefined;
      name = source?.name ?? '';
      surahIds = source ? [...source.surah_ids] : [];
      query = '';
      error = '';
      saving = false;
      confirmingDelete = false;
      surahsStore.init();
    });
    void tick().then(() => nameEl?.focus());
  });

  const selected = $derived(new Set(surahIds));
  const matches = $derived(filterSurahs(surahsStore.list, query));
  const canSave = $derived(name.trim().length > 0 && surahIds.length > 0 && !saving);

  function toggle(id: number) {
    surahIds = selected.has(id) ? surahIds.filter((s) => s !== id) : [...surahIds, id];
  }

  function move(index: number, delta: 1 | -1) {
    const target = index + delta;
    if (target < 0 || target >= surahIds.length) return;
    const next = [...surahIds];
    [next[index], next[target]] = [next[target], next[index]];
    surahIds = next;
    // Keep the keyboard on the item that moved, so Alt+↓ held down walks it
    // down the list rather than leaving focus on whatever slid into its place.
    void tick().then(() =>
      listEl?.querySelectorAll<HTMLElement>('.item')[target]?.focus({ preventScroll: false }),
    );
  }

  function onItemKeydown(e: KeyboardEvent, index: number) {
    if (!e.altKey || (e.key !== 'ArrowUp' && e.key !== 'ArrowDown')) return;
    e.preventDefault();
    move(index, e.key === 'ArrowUp' ? -1 : 1);
  }

  async function save() {
    if (!canSave) return;
    saving = true;
    error = '';
    try {
      await routinesStore.save(editing?.id ?? null, name, surahIds);
      uiStore.closeRoutineEditor();
    } catch (err) {
      error = String(err);
      saving = false;
    }
  }

  async function remove() {
    if (editing?.id == null) return;
    try {
      await routinesStore.remove(editing.id);
      uiStore.closeRoutineEditor();
    } catch (err) {
      error = String(err);
    }
  }

  function onDialogKeydown(e: KeyboardEvent) {
    // A pending delete is the innermost layer, so Escape backs out of that
    // before the dialog's own Escape (in +layout) closes everything.
    if (e.key === 'Escape' && confirmingDelete) {
      e.stopPropagation();
      confirmingDelete = false;
    }
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      void save();
    }
  }
</script>

{#if editing}
  <div
    class="backdrop"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) uiStore.closeRoutineEditor();
    }}
  >
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="routine-editor-title"
      tabindex="-1"
      onkeydown={onDialogKeydown}
    >
      <header class="head">
        <h2 id="routine-editor-title">{routine ? 'Edit routine' : 'New routine'}</h2>
        <button class="icon" onclick={() => uiStore.closeRoutineEditor()} aria-label="Close">
          <X size={16} />
        </button>
      </header>

      <div class="name-row">
        <input
          bind:this={nameEl}
          class="name"
          type="text"
          placeholder="Name, e.g. Morning"
          aria-label="Routine name"
          maxlength="60"
          bind:value={name}
          onkeydown={(e) => {
            if (e.key === 'Enter') void save();
          }}
        />
      </div>

      <div class="body">
        <section class="column order" aria-label="Surahs in this routine">
          <h3>In this order</h3>
          {#if surahIds.length === 0}
            <p class="empty">Pick the Surahs you read, in the order you read them.</p>
          {:else}
            <ol class="items scrollbar-thin" bind:this={listEl}>
              {#each surahIds as id, i (id)}
                {@const surah = surahsStore.get(id)}
                <li>
                  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
                  <div
                    class="item"
                    tabindex="0"
                    role="group"
                    aria-label="{i + 1}. {surah?.transliteration ?? `Surah ${id}`}"
                    onkeydown={(e) => onItemKeydown(e, i)}
                  >
                    <span class="step">{i + 1}</span>
                    <span class="item-name">{surah?.transliteration ?? `Surah ${id}`}</span>
                    {#if surah}
                      <span class="arabic">{stripTashkeel(surah.name_ar)}</span>
                    {/if}
                    <span class="item-actions">
                      <button
                        class="icon"
                        tabindex="-1"
                        disabled={i === 0}
                        aria-label="Move up"
                        title="Move up (Alt+↑)"
                        onclick={() => move(i, -1)}
                      >
                        <ArrowUp size={14} />
                      </button>
                      <button
                        class="icon"
                        tabindex="-1"
                        disabled={i === surahIds.length - 1}
                        aria-label="Move down"
                        title="Move down (Alt+↓)"
                        onclick={() => move(i, 1)}
                      >
                        <ArrowDown size={14} />
                      </button>
                      <button
                        class="icon"
                        aria-label="Remove {surah?.transliteration ?? `Surah ${id}`}"
                        title="Remove"
                        onclick={() => toggle(id)}
                      >
                        <X size={14} />
                      </button>
                    </span>
                  </div>
                </li>
              {/each}
            </ol>
          {/if}
        </section>

        <section class="column picker" aria-label="Add Surahs">
          <input
            class="filter"
            type="text"
            placeholder="Find a surah…"
            aria-label="Find a surah"
            bind:value={query}
            autocomplete="off"
          />
          <ul class="choices scrollbar-thin">
            {#each matches as surah (surah.id)}
              {@const on = selected.has(surah.id)}
              <li>
                <button class="choice" class:on aria-pressed={on} onclick={() => toggle(surah.id)}>
                  <span class="choice-mark">
                    {#if on}<Check size={13} />{:else}<Plus size={13} />{/if}
                  </span>
                  <span class="choice-no">{surah.id}</span>
                  <span class="item-name">{surah.transliteration}</span>
                  <span class="arabic">{stripTashkeel(surah.name_ar)}</span>
                </button>
              </li>
            {:else}
              <li class="empty">No match for “{query}”.</li>
            {/each}
          </ul>
        </section>
      </div>

      <footer class="foot">
        {#if routine}
          {#if confirmingDelete}
            <span class="confirm">Delete “{routine.name}”?</span>
            <button class="btn" onclick={() => (confirmingDelete = false)}>Keep</button>
            <button class="btn danger" onclick={remove}>Delete</button>
          {:else}
            <button class="btn subtle danger-text" onclick={() => (confirmingDelete = true)}>
              <Trash2 size={14} />
              Delete
            </button>
          {/if}
        {/if}
        <span class="spacer"></span>
        {#if error}
          <span class="error" role="alert">{error}</span>
        {/if}
        <button class="btn" onclick={() => uiStore.closeRoutineEditor()}>Cancel</button>
        <button class="btn primary" disabled={!canSave} onclick={save}>
          {routine ? 'Save' : 'Create'}
        </button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
    background: rgba(0, 0, 0, 0.35);
    z-index: 100;
  }

  .dialog {
    display: flex;
    flex-direction: column;
    width: min(720px, 100%);
    height: min(600px, 100%);
    overflow: hidden;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-bg-elevated);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.3);
  }

  .dialog:focus {
    outline: none;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px 10px 20px;
  }

  .head h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
  }

  .name-row {
    padding: 0 20px 14px;
    border-bottom: 1px solid var(--color-border);
  }

  .name,
  .filter {
    width: 100%;
    padding: 8px 10px;
    border-radius: var(--radius);
    border: 1px solid var(--color-border);
    background: var(--color-bg);
    color: var(--color-text);
    font-size: 14px;
  }

  .name {
    font-size: 15px;
  }

  .name:focus,
  .filter:focus {
    outline: 2px solid var(--color-accent);
    outline-offset: -1px;
  }

  .body {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 1fr 1fr;
  }

  .column {
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 12px;
  }

  .order {
    border-right: 1px solid var(--color-border);
  }

  .column h3 {
    margin: 4px 8px 10px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .empty {
    margin: 0;
    padding: 8px;
    font-size: 13px;
    color: var(--color-text-muted);
  }

  .items,
  .choices {
    flex: 1;
    min-height: 0;
    margin: 0;
    padding: 0;
    list-style: none;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .choices {
    margin-top: 10px;
  }

  .item,
  .choice {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 7px 8px;
    border: none;
    border-radius: var(--radius);
    background: transparent;
    color: var(--color-text);
    font: inherit;
    font-size: 14px;
    text-align: start;
  }

  .item:hover,
  .choice:hover {
    background: var(--color-bg-hover);
  }

  .item:focus-visible,
  .choice:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  .choice {
    cursor: pointer;
  }

  .step,
  .choice-mark {
    flex-shrink: 0;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: 1px solid var(--color-border);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .step {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .choice.on .choice-mark {
    border-color: var(--color-accent);
    background: var(--color-accent);
    color: var(--color-bg);
  }

  .choice-no {
    flex-shrink: 0;
    width: 24px;
    font-size: 11px;
    color: var(--color-text-muted);
    font-variant-numeric: tabular-nums;
  }

  .item-name {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* Live-shaped, so it needs --font-surah-name like the navigator's list. */
  .arabic {
    flex-shrink: 0;
    font-family: var(--font-surah-name);
    font-size: 16px;
    line-height: 1;
  }

  .item-actions {
    display: flex;
    gap: 2px;
    flex-shrink: 0;
  }

  .icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .icon:hover:not(:disabled) {
    background: var(--color-bg);
    color: var(--color-text);
  }

  .icon:disabled {
    opacity: 0.3;
    cursor: default;
  }

  .foot {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--color-border);
  }

  .spacer {
    flex: 1;
  }

  .confirm {
    font-size: 13px;
  }

  .error {
    font-size: 12px;
    color: #d9736a;
  }

  .btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 7px 14px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: transparent;
    color: var(--color-text);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }

  .btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
  }

  .btn:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .btn.primary {
    border-color: var(--color-accent);
    background: var(--color-accent);
    color: var(--color-bg);
  }

  .btn.primary:hover:not(:disabled) {
    background: var(--color-accent);
    filter: brightness(1.08);
  }

  .btn.subtle {
    border-color: transparent;
  }

  .danger-text {
    color: #d9736a;
  }

  .btn.danger {
    border-color: #d9736a;
    background: #d9736a;
    color: #fff;
  }

  /* Phone-sized window: full screen, and the order above the picker. */
  @media (max-width: 620px) {
    .backdrop {
      padding: 0;
    }

    .dialog {
      width: 100%;
      height: 100%;
      border: none;
      border-radius: 0;
    }

    .body {
      grid-template-columns: 1fr;
      grid-template-rows: minmax(120px, 40%) 1fr;
    }

    .order {
      border-right: none;
      border-bottom: 1px solid var(--color-border);
    }
  }
</style>
