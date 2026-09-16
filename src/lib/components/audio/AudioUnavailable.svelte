<!--
  Why there is no recitation on this machine, and the one command that fixes it.

  Shown wherever a play control would otherwise be, because the reader is the
  only person who can fix this: WebKitGTK decodes through GStreamer, whose
  plugins are system packages this app cannot ship. Naming the missing piece and
  the install line is the difference between a two-minute fix and a bug report
  about an app that "freezes".
-->
<script lang="ts">
  import { VolumeX } from 'lucide-svelte';
  import { playbackStore } from '$lib/stores/playback.svelte';

  let { compact = false }: { compact?: boolean } = $props();

  const blocked = $derived(playbackStore.unavailable);
</script>

{#if blocked}
  <div class="unavailable" class:compact>
    <p class="reason">
      <VolumeX size={13} />
      <span>Recitation cannot play on this system. {blocked.reason}</span>
    </p>
    {#if blocked.install}
      <!-- Selectable rather than a copy button: this is read once, in a desktop
           window where selecting text is already how text is taken. -->
      <code class="install">{blocked.install}</code>
    {/if}
  </div>
{/if}

<style>
  .unavailable {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px 10px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg-hover);
  }

  .compact {
    padding: 6px 8px;
    border: none;
    background: transparent;
  }

  .reason {
    display: flex;
    align-items: flex-start;
    gap: 6px;
    margin: 0;
    font-size: 12px;
    line-height: 1.45;
    color: var(--color-text-muted);
  }

  .reason :global(svg) {
    flex-shrink: 0;
    margin-top: 2px;
  }

  .install {
    padding: 4px 6px;
    border-radius: 4px;
    background: var(--color-bg);
    color: var(--color-text);
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 11px;
    user-select: text;
    overflow-wrap: anywhere;
  }
</style>
