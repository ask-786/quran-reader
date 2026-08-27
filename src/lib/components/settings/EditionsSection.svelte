<!--
  The edition shelf, lifted out of the tafsir panel.

  It used to live behind a Library button inside the open side panel, which
  meant that in card view — the default — there was no route to it at all. It
  is a preference-shaped thing (what is installed, what it costs on disk, what
  licence it carries) and this is where preference-shaped things are.
-->
<script lang="ts">
  import TafsirEditions from '$lib/components/tafsir/TafsirEditions.svelte';
</script>

<p class="intro">
  Commentaries are downloaded on demand and stored with the app's data. Everything already installed
  works offline.
</p>

<TafsirEditions />

<style>
  .intro {
    margin: 0 0 14px;
    font-size: 12px;
    line-height: 1.6;
    color: var(--color-text-muted);
  }
</style>
