<!--
  The docked sidebar. The list, the filter, and the keyboard cursor all live in
  NavPanel, which the command palette renders too — this is the same navigator
  in its column-shaped skin.
-->
<script lang="ts">
  import NavPanel from '$lib/components/navigation/NavPanel.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { isNarrowViewport } from '$lib/utils/viewport';
</script>

<aside class="sidebar">
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
  }
</style>
