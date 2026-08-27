<script lang="ts">
  import { onMount } from 'svelte';
  import { getVersion } from '@tauri-apps/api/app';

  let version = $state<string | null>(null);

  onMount(() => {
    // The installed app's own version, not package.json's — those can drift in
    // a dev build, and the one worth reporting in a bug is what is running.
    getVersion()
      .then((v) => (version = v))
      .catch(() => (version = null));
  });

  /**
   * The works this app is built out of, with the terms that actually bind a
   * redistributor. Not every dependency — THIRD-PARTY-NOTICES.md in the
   * repository is the full record, and this is the part a reader has a reason
   * to know: whose text they are reading and whose hand shaped it.
   */
  const CREDITS: { group: string; items: { name: string; terms: string }[] }[] = [
    {
      group: 'Quran text and layout',
      items: [
        { name: 'Quran text — Tanzil Project', terms: 'CC BY 3.0' },
        {
          name: 'Mushaf page glyphs — QCF v4, King Fahd Complex',
          terms: 'Use and distribution permitted; not open source',
        },
        { name: 'Page layout data — zonetecde/mushaf-layout', terms: 'See notices' },
      ],
    },
    {
      group: 'Commentary',
      items: [
        { name: 'Tafsīr al-Jalālayn — Arabic and English', terms: 'See notices' },
        { name: 'Tafsīr Ibn Kathīr — Arabic and abridged English', terms: 'Downloadable' },
      ],
    },
    {
      group: 'Fonts',
      items: [
        { name: 'Amiri — The Amiri Project Authors', terms: 'SIL OFL 1.1' },
        { name: 'Scheherazade New — SIL Global', terms: 'SIL OFL 1.1' },
        { name: 'Inter — Rasmus Andersson', terms: 'SIL OFL 1.1' },
      ],
    },
  ];
</script>

<div class="identity">
  <h2>Quran Reader</h2>
  <p class="version">{version ? `Version ${version}` : 'Version unavailable'}</p>
  <p class="blurb">
    An offline-first Quran reader. Its own source is MIT licensed; the Quran text, the Mushaf layout
    and the bundled fonts are third-party works under their own terms.
  </p>
</div>

{#each CREDITS as credit (credit.group)}
  <section class="group">
    <h3>{credit.group}</h3>
    <ul>
      {#each credit.items as item (item.name)}
        <li>
          <span class="name">{item.name}</span>
          <span class="terms">{item.terms}</span>
        </li>
      {/each}
    </ul>
  </section>
{/each}

<p class="note">
  The complete record, including every application dependency, is in
  <code>THIRD-PARTY-NOTICES.md</code> in the source repository.
</p>

<style>
  .identity h2 {
    margin: 0;
    font-size: 17px;
    font-weight: 600;
    color: var(--color-text);
  }

  .version {
    margin: 3px 0 0;
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    color: var(--color-text-muted);
  }

  .blurb {
    margin: 12px 0 0;
    font-size: 12.5px;
    line-height: 1.65;
    color: var(--color-text-muted);
  }

  .group {
    margin-top: 22px;
  }

  h3 {
    margin: 0 0 6px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 14px;
    padding: 7px 0;
    font-size: 12.5px;
    line-height: 1.5;
  }

  li + li {
    border-top: 1px solid var(--color-border);
  }

  .name {
    color: var(--color-text);
  }

  .terms {
    flex-shrink: 0;
    text-align: end;
    color: var(--color-text-muted);
  }

  .note {
    margin: 22px 0 0;
    font-size: 12px;
    line-height: 1.6;
    color: var(--color-text-muted);
  }

  code {
    padding: 1px 5px;
    border-radius: 4px;
    background: var(--color-bg);
    font-size: 11.5px;
  }
</style>
