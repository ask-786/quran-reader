<script lang="ts">
  import {
    autoScrollStore,
    AUTO_SCROLL_SPEED_MIN,
    AUTO_SCROLL_SPEED_MAX,
  } from '$lib/stores/auto-scroll.svelte';

  const RANGE = AUTO_SCROLL_SPEED_MAX - AUTO_SCROLL_SPEED_MIN;

  let track = $state<HTMLDivElement>();
  let dragging = $state(false);
  let moved = false;

  // Top of the track = stopped, bottom = fastest — matches the fill
  // growing downward from 0, so the filled height always reads as "how fast".
  function speedFromClientY(clientY: number) {
    if (!track) return autoScrollStore.speed;
    const rect = track.getBoundingClientRect();
    const ratio = Math.min(Math.max((clientY - rect.top) / rect.height, 0), 1);
    return AUTO_SCROLL_SPEED_MIN + ratio * RANGE;
  }

  function onPointerDown(e: PointerEvent) {
    dragging = true;
    moved = false;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragging) return;
    moved = true;
    autoScrollStore.setSpeed(speedFromClientY(e.clientY));
  }

  function onPointerUp() {
    if (dragging && !moved) autoScrollStore.toggle();
    dragging = false;
  }

  // While this slider has focus its arrow keys set the speed, so they must not
  // also reach the layout's shortcuts (where the same keys page the Mushaf) —
  // hence stopPropagation on everything handled here.
  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      e.stopPropagation();
      autoScrollStore.faster();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      e.stopPropagation();
      autoScrollStore.slower();
    } else if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      e.stopPropagation();
      autoScrollStore.toggle();
    }
  }
</script>

<div
  bind:this={track}
  class="auto-scroll-handle"
  class:dragging
  title="Auto-scroll — drag to set speed, click to play/pause"
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onpointercancel={onPointerUp}
  onkeydown={onKeydown}
  role="slider"
  aria-label="Auto-scroll speed"
  aria-valuemin={AUTO_SCROLL_SPEED_MIN}
  aria-valuemax={AUTO_SCROLL_SPEED_MAX}
  aria-valuenow={autoScrollStore.speed}
  tabindex="0"
>
  <div
    class="fill"
    class:active={autoScrollStore.active}
    style:height="{autoScrollStore.speed}%"
  ></div>
</div>

<style>
  .auto-scroll-handle {
    position: absolute;
    right: 10px;
    top: 16px;
    bottom: 16px;
    width: 4px;
    border-radius: 2px;
    background: var(--color-border);
    overflow: visible;
    cursor: pointer;
    touch-action: none;
    z-index: 5;
  }

  /* Widen the hit area without widening the visible track. */
  .auto-scroll-handle::before {
    content: '';
    position: absolute;
    inset: 0 -8px;
  }

  .fill {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    border-radius: 2px;
    background: var(--color-text-faint);
    overflow: hidden;
  }

  .auto-scroll-handle:not(.dragging) .fill {
    transition:
      height 80ms linear,
      background var(--transition);
  }

  .fill.active {
    background: var(--color-accent);
  }

  /* Sit flush against the window edge instead of floating 10px in — the
     reading column's side padding (see --reader-side-padding) is widened
     at this breakpoint to keep text clear of both the bar and its drag
     hit-area (the ::before above extends 8px past the visible bar). */
  @media (max-width: 900px) {
    .auto-scroll-handle {
      right: 0;
    }
  }

  @media (max-width: 480px) {
    .auto-scroll-handle {
      display: none;
    }
  }
</style>
