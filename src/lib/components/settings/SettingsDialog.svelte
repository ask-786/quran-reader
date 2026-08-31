<!--
  Settings: everything that is a preference rather than a mode.

  A dialog over the reader rather than a route, for the same reason the command
  palette is one. A `/settings` page would unmount the reader, and coming back
  would land the reader at the top of the range instead of where they were
  reading — a cost nothing here is worth. It also means these controls work in
  focus mode and with the sidebar shut, without dragging either back on screen.

  Fixed size, like the palette: the rail switches between sections of very
  different heights, and a box that resized under the cursor would move the
  thing being aimed at.
-->
<script lang="ts">
  import {
    X,
    Palette,
    BookOpen,
    ScrollText,
    Library,
    Database,
    AudioLines,
    Keyboard,
    Info,
  } from 'lucide-svelte';
  import type { Component } from 'svelte';

  import { uiStore, SETTINGS_SECTIONS, type SettingsSection } from '$lib/stores/ui.svelte';
  import AppearanceSection from './AppearanceSection.svelte';
  import ReaderSection from './ReaderSection.svelte';
  import TafsirSection from './TafsirSection.svelte';
  import AudioSection from './AudioSection.svelte';
  import EditionsSection from './EditionsSection.svelte';
  import DataSection from './DataSection.svelte';
  import ShortcutsSection from './ShortcutsSection.svelte';
  import AboutSection from './AboutSection.svelte';

  type SectionMeta = {
    label: string;
    /** The line under the section heading — what this page is for, in the
     *  terms the reader would describe it in. */
    blurb: string;
    /** Any lucide icon. They are legacy class components and all share one
     *  props shape, so one of them stands for the type of all of them. */
    icon: typeof Palette;
    /** Section bodies take no props at all — each reaches into the stores it
     *  needs, exactly as the panels elsewhere in the app do. */
    component: Component<Record<string, never>>;
  };

  const SECTIONS: Record<SettingsSection, SectionMeta> = {
    appearance: {
      label: 'Appearance',
      blurb: 'How the app looks and how big it is.',
      icon: Palette,
      component: AppearanceSection,
    },
    reader: {
      label: 'Reader',
      blurb: 'The typography of the Quran text, in both reading views.',
      icon: BookOpen,
      component: ReaderSection,
    },
    tafsir: {
      label: 'Tafsir',
      blurb: 'Which commentary you read, and how it opens.',
      icon: ScrollText,
      component: TafsirSection,
    },
    audio: {
      label: 'Audio',
      blurb: 'Who recites, how it repeats, and what it has stored.',
      icon: AudioLines,
      component: AudioSection,
    },
    editions: {
      label: 'Editions',
      blurb: 'Commentaries installed, and the ones available to download.',
      icon: Library,
      component: EditionsSection,
    },
    data: {
      label: 'Data',
      blurb: 'What the app is storing, and how to clear it.',
      icon: Database,
      component: DataSection,
    },
    shortcuts: {
      label: 'Shortcuts',
      blurb: 'Every key the reader listens for.',
      icon: Keyboard,
      component: ShortcutsSection,
    },
    about: {
      label: 'About',
      blurb: 'Version, and the works this is built from.',
      icon: Info,
      component: AboutSection,
    },
  };

  const active = $derived(SECTIONS[uiStore.settingsSection]);
  // Capitalised so the template renders it as a component rather than an
  // element. A member expression (`active.component`) would work too, but
  // `meta.icon` below would not — `meta` is an HTML tag name.
  const ActiveSection = $derived(active.component);

  let pane = $state<HTMLDivElement>();
  let dialog = $state<HTMLDivElement>();

  /**
   * Each section starts at its own top. Without this, opening a long section
   * after a scrolled one shows its middle, which reads as content missing
   * above rather than as a scroll position carried over.
   */
  $effect(() => {
    void uiStore.settingsSection;
    pane?.scrollTo({ top: 0 });
  });

  // Focus lands in the dialog on open so Tab starts inside it and screen
  // readers announce where they are. The rail's current button, not the close
  // button — the first thing here is a choice, not an exit.
  $effect(() => {
    if (!uiStore.settingsOpen) return;
    dialog?.querySelector<HTMLElement>('.rail-item.on')?.focus();
  });

  /**
   * Roving arrows down the rail, the standard vertical tablist behaviour.
   * Stopped as well as consumed: the reader's global handler reads bare arrows
   * as page turns, and its text-field guard does not cover buttons.
   */
  function onRailKeydown(e: KeyboardEvent) {
    const forward = e.key === 'ArrowDown';
    const back = e.key === 'ArrowUp';
    if (!forward && !back) return;
    e.preventDefault();
    e.stopPropagation();
    const i = SETTINGS_SECTIONS.indexOf(uiStore.settingsSection);
    const next = (i + (forward ? 1 : -1) + SETTINGS_SECTIONS.length) % SETTINGS_SECTIONS.length;
    uiStore.settingsSection = SETTINGS_SECTIONS[next];
    (e.currentTarget as HTMLElement).querySelectorAll<HTMLElement>('.rail-item')[next]?.focus();
  }
</script>

{#if uiStore.settingsOpen}
  <div
    class="backdrop"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) uiStore.closeSettings();
    }}
  >
    <div bind:this={dialog} class="dialog" role="dialog" aria-modal="true" aria-label="Settings">
      <!-- tabindex on the list itself is never reached — the tabs inside it
           are what take focus — but the role is an interactive one and the
           a11y rule asks for it regardless. -->
      <div
        class="rail"
        role="tablist"
        tabindex="-1"
        aria-label="Settings sections"
        onkeydown={onRailKeydown}
      >
        <h2 class="rail-title">Settings</h2>
        {#each SETTINGS_SECTIONS as section (section)}
          {@const meta = SECTIONS[section]}
          {@const Icon = meta.icon}
          {@const on = section === uiStore.settingsSection}
          <button
            type="button"
            role="tab"
            class="rail-item"
            class:on
            aria-selected={on}
            tabindex={on ? 0 : -1}
            onclick={() => (uiStore.settingsSection = section)}
          >
            <Icon size={15} />
            <span>{meta.label}</span>
          </button>
        {/each}
      </div>

      <div class="pane-wrap">
        <header class="pane-header">
          <div class="heading">
            <h3>{active.label}</h3>
            <p>{active.blurb}</p>
          </div>
          <button class="close" onclick={() => uiStore.closeSettings()} aria-label="Close settings">
            <X size={16} />
          </button>
        </header>
        <div bind:this={pane} class="pane scrollbar-thin" role="tabpanel" aria-label={active.label}>
          <ActiveSection />
        </div>
      </div>
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
    /* Above the palette's 100 is not required — they are mutually exclusive
       (see uiStore.openSettings) — but the same order stops a stale frame of
       one showing through the other during a switch. */
    z-index: 100;
  }

  .dialog {
    display: flex;
    width: min(820px, 100%);
    height: min(600px, 100%);
    overflow: hidden;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-bg-elevated);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.3);
  }

  .rail {
    display: flex;
    flex-direction: column;
    gap: 1px;
    flex-shrink: 0;
    width: 188px;
    padding: 14px 10px;
    background: var(--color-bg);
    border-right: 1px solid var(--color-border);
    overflow-y: auto;
  }

  .rail-title {
    margin: 2px 8px 12px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .rail-item {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 8px 10px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--color-text-muted);
    font-family: inherit;
    font-size: 13px;
    text-align: start;
    cursor: pointer;
    transition:
      background var(--transition),
      color var(--transition);
  }

  .rail-item:hover:not(.on) {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .rail-item.on {
    background: var(--color-bg-hover);
    color: var(--color-text);
    font-weight: 500;
  }

  .rail-item:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  .pane-wrap {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }

  .pane-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    flex-shrink: 0;
    padding: 18px 20px 14px;
    border-bottom: 1px solid var(--color-border);
  }

  .heading h3 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: var(--color-text);
  }

  .heading p {
    margin: 3px 0 0;
    font-size: 12px;
    line-height: 1.5;
    color: var(--color-text-muted);
  }

  .close {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border: none;
    border-radius: 7px;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .close:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .pane {
    flex: 1;
    min-height: 0;
    padding: 6px 20px 22px;
    overflow-y: auto;
  }

  /* Phone-sized window: the rail becomes a scrolling strip across the top, so
     the pane keeps the full width for its controls. */
  @media (max-width: 620px) {
    .backdrop {
      padding: 0;
    }

    .dialog {
      flex-direction: column;
      width: 100%;
      height: 100%;
      border: none;
      border-radius: 0;
    }

    .rail {
      flex-direction: row;
      width: 100%;
      gap: 4px;
      padding: 8px;
      border-right: none;
      border-bottom: 1px solid var(--color-border);
      overflow-x: auto;
      overflow-y: hidden;
    }

    .rail-title {
      display: none;
    }

    .rail-item {
      flex-shrink: 0;
    }
  }
</style>
