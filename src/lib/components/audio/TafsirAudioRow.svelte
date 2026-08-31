<!--
  The audio strip in the tafsir card, in both the popover and the side panel.

  Recitation lives here and not on the reading surface. The card is already open
  on this āya and already answers "what is going on here"; "how is this said" is
  the same reader stopping for the same reason. Reading is the app — this is a
  clarification you reach for, so it plays one verse and stops.
-->
<script lang="ts">
  import { Volume2 } from 'lucide-svelte';
  import AudioScrubber from './AudioScrubber.svelte';
  import { playbackStore } from '$lib/stores/playback.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';

  let { ayahId }: { ayahId: number } = $props();
</script>

<div class="audio-row">
  {#if playbackStore.enabled}
    <AudioScrubber {ayahId} />
  {:else}
    <!-- Present before a reciter is chosen, or the feature is invisible until
         someone goes looking in Settings for a thing they have no reason to
         think exists. Choosing there is also where fetching is explained and
         turned on, so this points at it rather than asking here. -->
    <button class="choose" onclick={() => uiStore.openSettings('audio')}>
      <Volume2 size={14} />
      Choose a reciter to hear this verse
    </button>
  {/if}
</div>

<style>
  /* As short as a 26px control can make it. Every pixel here is a pixel of
     commentary, which is what the card is actually for. */
  .audio-row {
    padding: 5px 12px;
    border-bottom: 1px solid var(--color-border);
  }

  .choose {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 4px 2px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    font-family: inherit;
    font-size: 12px;
    text-align: left;
    cursor: pointer;
  }

  .choose:hover {
    color: var(--color-text);
  }
</style>
