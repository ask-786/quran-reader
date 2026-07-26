<script lang="ts">
  import { Play, Pause } from 'lucide-svelte';
  import {
    autoScrollStore,
    AUTO_SCROLL_SPEED_MIN,
    AUTO_SCROLL_SPEED_MAX,
  } from '$lib/stores/auto-scroll.svelte';

  const TRACK_HEIGHT = 140;
  const RANGE = AUTO_SCROLL_SPEED_MAX - AUTO_SCROLL_SPEED_MIN;
  const KEY_STEP = 10;

  let track = $state<HTMLDivElement>();
  let dragging = $state(false);
  let moved = false;

  const percent = $derived(autoScrollStore.speed);

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

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      autoScrollStore.setSpeed(autoScrollStore.speed + KEY_STEP);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      autoScrollStore.setSpeed(autoScrollStore.speed - KEY_STEP);
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
    class:dragging
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
    <div class="fill" class:active={autoScrollStore.active} style:height="{percent}%"></div>
  </div>
  <div class="label" class:active={autoScrollStore.active}>
    {#if autoScrollStore.active}
      <Pause size={10} />
      {percent}%
    {:else}
      <Play size={10} />
    {/if}
  </div>
</div>

<style>
  .auto-scroll-handle {
    position: absolute;
    right: 10px;
    top: 50%;
    transform: translateY(-50%);
    z-index: 5;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }

  .track {
    position: relative;
    width: 4px;
    border-radius: 2px;
    background: var(--color-border);
    overflow: visible;
    cursor: pointer;
    touch-action: none;
  }

  /* Widen the hit area without widening the visible track, so the slim
     bar stays visually consistent with ProgressIndicator's rail. */
  .track::before {
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

  .track:not(.dragging) .fill {
    transition:
      height 80ms linear,
      background var(--transition);
  }

  .fill.active {
    background: var(--color-accent);
  }

  .label {
    display: flex;
    align-items: center;
    gap: 3px;
    font-size: 10px;
    color: var(--color-text-faint);
    font-variant-numeric: tabular-nums;
  }

  .label.active {
    color: var(--color-accent);
  }
</style>
