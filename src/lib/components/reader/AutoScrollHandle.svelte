<script lang="ts">
  import { Play, Pause } from 'lucide-svelte';
  import {
    autoScrollStore,
    AUTO_SCROLL_SPEED_MIN,
    AUTO_SCROLL_SPEED_MAX,
  } from '$lib/stores/auto-scroll.svelte';

  const TRACK_HEIGHT = 140;
  const THUMB_SIZE = 26;
  const THUMB_TRAVEL = TRACK_HEIGHT - THUMB_SIZE;
  const RANGE = AUTO_SCROLL_SPEED_MAX - AUTO_SCROLL_SPEED_MIN;

  let track = $state<HTMLDivElement>();
  let dragging = $state(false);
  let moved = false;

  const thumbTop = $derived(
    THUMB_TRAVEL - ((autoScrollStore.speed - AUTO_SCROLL_SPEED_MIN) / RANGE) * THUMB_TRAVEL,
  );

  function speedFromClientY(clientY: number) {
    if (!track) return autoScrollStore.speed;
    const rect = track.getBoundingClientRect();
    const y = Math.min(Math.max(clientY - rect.top - THUMB_SIZE / 2, 0), THUMB_TRAVEL);
    const ratio = 1 - y / THUMB_TRAVEL;
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

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      autoScrollStore.setSpeed(autoScrollStore.speed + 1);
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      autoScrollStore.setSpeed(autoScrollStore.speed - 1);
    } else if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      autoScrollStore.toggle();
    }
  }
</script>

<div class="auto-scroll-handle" title="Auto-scroll — drag to set speed, click to play/pause">
  <div
    bind:this={track}
    class="track"
    style:height="{TRACK_HEIGHT}px"
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
    <div class="thumb" class:active={autoScrollStore.active} style:top="{thumbTop}px">
      {#if autoScrollStore.active}
        <Pause size={12} />
      {:else}
        <Play size={12} />
      {/if}
    </div>
  </div>
</div>

<style>
  .auto-scroll-handle {
    position: absolute;
    right: 10px;
    top: 50%;
    transform: translateY(-50%);
    z-index: 5;
  }

  .track {
    position: relative;
    width: 8px;
    border-radius: 4px;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    cursor: pointer;
    touch-action: none;
  }

  .thumb {
    position: absolute;
    left: 50%;
    width: 26px;
    height: 26px;
    transform: translateX(-50%);
    border-radius: 50%;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    color: var(--color-text-muted);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.25);
    transition:
      background var(--transition),
      color var(--transition);
  }

  .thumb.active {
    background: var(--color-accent);
    color: var(--color-accent-contrast);
    border-color: var(--color-accent);
  }
</style>
