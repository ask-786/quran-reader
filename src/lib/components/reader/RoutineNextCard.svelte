<!--
  The foot of a Surah read as part of a routine: where the routine goes next.

  The same job `n` does, put where the reader's eyes already are when they
  finish — the key is quicker, but you have to know it exists. Renders nothing
  unless the open Surah is the active routine's current step, so reading the
  same Surah outside the routine ends the way it always has.

  The last step finishes the routine on an explicit press rather than on the
  card scrolling into view: a short Surah fits on one screen, and a routine
  marked done the moment its last Surah opened would be claiming something
  nobody has done yet.
-->
<script lang="ts">
  import { page } from '$app/stores';
  import { ArrowRight, Check } from 'lucide-svelte';
  import { routinesStore } from '$lib/stores/routines.svelte';
  import { surahsStore } from '$lib/stores/surahs.svelte';

  const surahId = $derived(Number($page.params.id) || 0);
  const onStep = $derived(routinesStore.isOnStep($page.route.id ?? null, surahId));
  const routine = $derived(onStep ? routinesStore.active : null);
  const step = $derived(routinesStore.activeStep ?? 0);
  const done = $derived(routinesStore.activeStatus?.kind === 'done');
  const next = $derived(
    routinesStore.nextSurahId !== null ? surahsStore.get(routinesStore.nextSurahId) : undefined,
  );
  const nextHref = $derived(onStep ? routinesStore.stepHref(1) : null);
</script>

{#if routine}
  <!-- The href comes from routineStepHref, which resolves the route itself. -->
  <!-- eslint-disable svelte/no-navigation-without-resolve -->
  <section class="routine-next" aria-label="{routine.name} routine">
    <p class="eyebrow">{routine.name} · {step + 1} of {routine.surah_ids.length}</p>
    {#if nextHref}
      <a class="action" href={nextHref}>
        <span class="label">Next</span>
        <span class="target">{next?.transliteration ?? `Surah ${routinesStore.nextSurahId}`}</span>
        <ArrowRight size={16} />
        <kbd>n</kbd>
      </a>
    {:else if done}
      <p class="done"><Check size={16} /> Done for today</p>
    {:else}
      <button class="action" onclick={() => routinesStore.complete()}>
        <span class="target">Finish {routine.name}</span>
        <Check size={16} />
        <kbd>n</kbd>
      </button>
    {/if}
  </section>
  <!-- eslint-enable svelte/no-navigation-without-resolve -->
{/if}

<style>
  .routine-next {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    margin: 32px auto 24px;
    padding: 18px 24px;
    width: min(420px, 100%);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-bg-elevated);
    font-family: var(--font-ui);
  }

  .eyebrow {
    margin: 0;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .action {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 9px 16px;
    border: 1px solid var(--color-accent);
    border-radius: var(--radius);
    background: transparent;
    color: var(--color-accent);
    font: inherit;
    font-size: 15px;
    text-decoration: none;
    cursor: pointer;
    transition: background var(--transition);
  }

  .action:hover {
    background: var(--color-bg-hover);
  }

  .label {
    color: var(--color-text-muted);
    font-size: 13px;
  }

  .target {
    font-weight: 600;
  }

  kbd {
    margin-left: 4px;
    min-width: 18px;
    padding: 1px 5px;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    font-family: var(--font-ui);
    font-size: 11px;
    line-height: 1.5;
    text-align: center;
    color: var(--color-text-muted);
  }

  .done {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    font-size: 15px;
    color: var(--color-accent);
  }
</style>
