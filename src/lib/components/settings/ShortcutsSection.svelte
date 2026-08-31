<!--
  The key map, which until now existed only as a switch statement in
  +layout.svelte and four keys in the palette's footer.

  Kept in step by hand — there is no registry to generate it from. If you add a
  binding to that handler, add it here.
-->
<script lang="ts">
  type Shortcut = { keys: string[]; label: string };

  const GROUPS: { title: string; shortcuts: Shortcut[] }[] = [
    {
      title: 'Getting around',
      shortcuts: [
        { keys: ['Ctrl', 'K'], label: 'Go to — search, Surah, Juz, Hizb, or 2:255' },
        { keys: ['Ctrl', 'F'], label: 'Search the sidebar list' },
        { keys: ['/'], label: 'Search the sidebar list' },
        { keys: ['n'], label: 'Next Surah, Juz, Hizb or page' },
        { keys: ['p'], label: 'Previous one' },
        { keys: ['↓'], label: 'Next Mushaf page' },
        { keys: ['↑'], label: 'Previous Mushaf page' },
        { keys: ['Home'], label: 'To the top' },
        { keys: ['End'], label: 'To the bottom' },
      ],
    },
    {
      title: 'Reading',
      shortcuts: [
        { keys: ['m'], label: 'Switch between scrolling and Mushaf page view' },
        { keys: ['f'], label: 'Focus mode' },
        { keys: ['t'], label: 'Verse card — commentary and recitation for a verse' },
        {
          keys: ['Space'],
          label: 'Play or pause recitation in the verse card — auto-scroll when no card is open',
        },
        { keys: ['a'], label: 'Start or stop auto-scroll' },
        { keys: ['Shift', '↑'], label: 'Auto-scroll faster' },
        { keys: ['Shift', '↓'], label: 'Auto-scroll slower' },
      ],
    },
    {
      title: 'Size',
      shortcuts: [
        { keys: ['+'], label: 'Zoom the reader in' },
        { keys: ['−'], label: 'Zoom the reader out' },
        { keys: ['0'], label: 'Reset reader zoom' },
        { keys: ['Ctrl', '+'], label: 'Zoom the whole app in' },
        { keys: ['Ctrl', '−'], label: 'Zoom the whole app out' },
        { keys: ['Ctrl', '0'], label: 'Reset app zoom' },
      ],
    },
    {
      title: 'This dialog',
      shortcuts: [
        { keys: ['Ctrl', ','], label: 'Open settings' },
        { keys: ['Esc'], label: 'Close whatever is open' },
      ],
    },
  ];
</script>

{#each GROUPS as group (group.title)}
  <section class="group">
    <h3>{group.title}</h3>
    <dl>
      {#each group.shortcuts as shortcut (shortcut.keys.join('+') + shortcut.label)}
        <div class="shortcut">
          <dt>
            {#each shortcut.keys as key, i (i)}
              {#if i > 0}<span class="plus">+</span>{/if}<kbd>{key}</kbd>
            {/each}
          </dt>
          <dd>{shortcut.label}</dd>
        </div>
      {/each}
    </dl>
  </section>
{/each}

<style>
  .group + .group {
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

  dl {
    margin: 0;
  }

  .shortcut {
    display: flex;
    align-items: baseline;
    gap: 14px;
    padding: 6px 0;
  }

  dt {
    display: flex;
    flex-shrink: 0;
    align-items: baseline;
    gap: 3px;
    /* A fixed gutter so the descriptions line up into a readable column
       instead of stepping in and out with the width of each chord. */
    width: 96px;
  }

  dd {
    margin: 0;
    font-size: 12.5px;
    line-height: 1.5;
    color: var(--color-text-muted);
  }

  kbd {
    padding: 2px 6px;
    border: 1px solid var(--color-border);
    border-bottom-width: 2px;
    border-radius: 5px;
    background: var(--color-bg);
    color: var(--color-text);
    font-family: var(--font-ui);
    font-size: 11px;
    line-height: 1.4;
    white-space: nowrap;
  }

  .plus {
    font-size: 10px;
    color: var(--color-text-faint);
  }
</style>
