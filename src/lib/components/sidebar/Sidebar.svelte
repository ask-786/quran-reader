<!--
  The docked sidebar. The list, the filter, and the keyboard cursor all live in
  NavPanel, which the command palette renders too — this is the same navigator
  in its column-shaped skin.
-->
<script lang="ts">
  import { PanelLeft } from 'lucide-svelte';
  import NavPanel from '$lib/components/navigation/NavPanel.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { windowChrome } from '$lib/stores/window-chrome.svelte';
  import { isNarrowViewport } from '$lib/utils/viewport';
</script>

<aside class="sidebar">
  <!-- The traffic lights' row: the sidebar runs to the top of the window, as
       in Finder or Notes, and this strip lines up with the toolbar beside it.
       The toolbar hands its sidebar toggle over to it while it is showing. -->
  {#if windowChrome.trafficLights}
    <div class="titlebar" data-tauri-drag-region>
      <button class="icon-btn" onclick={() => uiStore.toggleSidebar()} aria-label="Hide sidebar">
        <PanelLeft size={18} />
      </button>
    </div>
  {/if}
  <NavPanel
    variant="sidebar"
    onselect={() => {
      // On a narrow window the sidebar is a drawer over the reader, so opening
      // something from it has to get it out of the way.
      if (isNarrowViewport()) uiStore.sidebarOpen = false;
    }}
  />
</aside>

<style>
  .sidebar {
    width: 100%;
    height: 100%;
    flex-shrink: 0;
    overflow: hidden;
    background: var(--color-bg-elevated);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
  }

  /* 84px clears the traffic lights, matching the toolbar's inset. */
  .titlebar {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    height: var(--toolbar-height);
    padding-left: 84px;
    border-bottom: 1px solid var(--color-border);
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius);
    background: transparent;
    border: none;
    color: var(--color-text-muted);
    transition: background var(--transition);
  }

  .icon-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }
</style>
