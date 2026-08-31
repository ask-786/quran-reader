<!--
  Recitation: who reads, how it repeats, and what it has put on the disk.

  Storage is here rather than in Data because audio is the only thing in this
  app that can grow to hundreds of megabytes on its own. A reader who has played
  through half the Quran should be able to see what that cost and take it back,
  per reciter, without leaving the page where they turned it on.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { Check, Trash2, X } from 'lucide-svelte';
  import { audioUsage, clearAudioCache } from '$lib/api/db';
  import { playbackStore } from '$lib/stores/playback.svelte';
  import {
    settingsStore,
    AUDIO_RATE_MAX,
    AUDIO_RATE_MIN,
    AUDIO_REPEAT_MAX,
    AUDIO_REPEAT_MIN,
    AUDIO_REPEAT_PAUSE_MAX,
  } from '$lib/stores/settings.svelte';
  import type { AudioBitrate, ReciterUsage, RepeatMode } from '$lib/types/database';
  import SettingRow from './SettingRow.svelte';
  import Segmented from './Segmented.svelte';
  import Slider from './Slider.svelte';
  import Toggle from './Toggle.svelte';

  let usage = $state<ReciterUsage[]>([]);
  /** Which reciter's cache the trash button is armed for, or 'all'. Session
   *  only, and one at a time — this deletes files that have to be fetched
   *  again, and a bare icon is easy to hit by mistake. */
  let confirming = $state<string | null>(null);

  const REPEAT_MODES: { value: RepeatMode; label: string }[] = [
    { value: 'off', label: 'Off' },
    { value: 'ayah', label: 'This verse' },
    { value: 'range', label: 'Whole range' },
  ];

  const BITRATES: { value: string; label: string }[] = [
    { value: '64', label: '64 kbps' },
    { value: '128', label: '128 kbps' },
  ];

  const settings = $derived(settingsStore.current);
  const cached = $derived(usage.filter((u) => u.files > 0));
  const totalBytes = $derived(cached.reduce((sum, u) => sum + u.bytes, 0));

  onMount(() => {
    void playbackStore.init();
    void refresh();
  });

  async function refresh() {
    try {
      usage = await audioUsage();
    } catch (err) {
      console.error('Failed to read audio usage', err);
    }
  }

  async function clear(slug?: string) {
    confirming = null;
    try {
      await clearAudioCache(slug);
    } catch (err) {
      console.error('Failed to clear audio cache', err);
    }
    // The verse in the player's hand may have just been deleted underneath it.
    playbackStore.stop();
    await refresh();
  }

  function mb(bytes: number): string {
    if (bytes < 1_048_576) return `${Math.max(1, Math.round(bytes / 1024))} KB`;
    return `${(bytes / 1_048_576).toFixed(bytes < 104_857_600 ? 1 : 0)} MB`;
  }
</script>

<SettingRow
  label="Reciter"
  description="Recitation is fetched from cdn.islamic.network as you play it, one verse at a time. Choosing a reciter here turns that on; nothing is fetched until you do."
  stacked
>
  {#snippet control()}
    <div class="reciters">
      <button
        type="button"
        class="reciter"
        class:on={settings.reciter_id === null}
        onclick={() => settingsStore.setReciter(null)}
      >
        <span class="name">None</span>
        <span class="meta">No recitation, no requests</span>
      </button>
      {#each playbackStore.reciters as reciter (reciter.id)}
        <button
          type="button"
          class="reciter"
          class:on={settings.reciter_id === reciter.id}
          onclick={() => settingsStore.setReciter(reciter.id)}
        >
          <span class="name">
            {reciter.name_en}
            <span class="native">{reciter.name_ar}</span>
          </span>
          <!-- The riwaya is shown for the reason a tafsir shows its school:
               it decides what you are hearing and is invisible otherwise. -->
          <span class="meta">{reciter.riwaya} · {reciter.style}</span>
        </button>
      {/each}
    </div>
  {/snippet}
</SettingRow>

<SettingRow
  label="Download recitation"
  description="Off keeps everything already downloaded playable and stops the app contacting cdn.islamic.network at all."
>
  {#snippet control()}
    <Toggle
      label="Download recitation"
      checked={settings.audio_downloads_allowed}
      onchange={(on) => settingsStore.setAudioDownloadsAllowed(on)}
    />
  {/snippet}
</SettingRow>

<SettingRow
  label="Quality"
  description="A verse is roughly 150 KB at 64 kbps and twice that at 128. Each is cached separately, so switching downloads again."
  stacked
>
  {#snippet control()}
    <Segmented
      label="Quality"
      options={BITRATES}
      value={String(settings.audio_bitrate)}
      onchange={(value) => settingsStore.setAudioBitrate(Number(value) as AudioBitrate)}
    />
  {/snippet}
</SettingRow>

<SettingRow
  label="Repeat"
  description="What happens when a verse ends. Repeating one verse is how a pronunciation you are unsure of gets heard three times."
  stacked
>
  {#snippet control()}
    <Segmented
      label="Repeat"
      options={REPEAT_MODES}
      value={settings.audio_repeat_mode}
      onchange={(mode) => settingsStore.setAudioRepeatMode(mode)}
    />
  {/snippet}
</SettingRow>

{#if settings.audio_repeat_mode === 'ayah'}
  <SettingRow label="Times" description="How often a verse repeats before moving on." stacked>
    {#snippet control()}
      <Slider
        label="Repeats per verse"
        value={settings.audio_repeat_count}
        min={AUDIO_REPEAT_MIN}
        max={AUDIO_REPEAT_MAX}
        step={1}
        format={(v) => `${v}×`}
        oninput={(v) => settingsStore.setAudioRepeatCount(v)}
      />
    {/snippet}
  </SettingRow>
{/if}

{#if settings.audio_repeat_mode !== 'off'}
  <SettingRow
    label="Pause between"
    description="Silence before it comes round again — room to say the verse back."
    stacked
  >
    {#snippet control()}
      <Slider
        label="Pause between repeats"
        value={settings.audio_repeat_pause_ms}
        min={0}
        max={AUDIO_REPEAT_PAUSE_MAX}
        step={500}
        format={(v) => (v === 0 ? 'None' : `${(v / 1000).toFixed(1)} s`)}
        oninput={(v) => settingsStore.setAudioRepeatPause(v)}
      />
    {/snippet}
  </SettingRow>
{/if}

<SettingRow label="Speed" description="Slower for following along, word by word." stacked>
  {#snippet control()}
    <Slider
      label="Playback speed"
      value={settings.audio_playback_rate}
      min={AUDIO_RATE_MIN}
      max={AUDIO_RATE_MAX}
      step={0.05}
      format={(v) => `${v.toFixed(2)}×`}
      oninput={(v) => settingsStore.setAudioPlaybackRate(v)}
    />
  {/snippet}
</SettingRow>

<SettingRow label="Volume" stacked>
  {#snippet control()}
    <Slider
      label="Volume"
      value={settings.audio_volume}
      min={0}
      max={1}
      step={0.05}
      format={(v) => `${Math.round(v * 100)}%`}
      oninput={(v) => settingsStore.setAudioVolume(v)}
    />
  {/snippet}
</SettingRow>

<SettingRow
  label="Follow the recitation"
  description="Scrolls the reader along when a whole range is playing. A single verse played from the tafsir card is already on screen, so it is left alone."
>
  {#snippet control()}
    <Toggle
      label="Follow the recitation"
      checked={settings.audio_follow}
      onchange={(on) => settingsStore.setAudioFollow(on)}
    />
  {/snippet}
</SettingRow>

<div class="storage">
  <div class="storage-head">
    <span class="label">Downloaded audio</span>
    <span class="total">{cached.length ? mb(totalBytes) : 'Nothing yet'}</span>
  </div>
  <p class="description">
    Every verse you play is kept, so what you have listened to works offline. Deleting it does not
    change your reciter — playing a verse fetches it again.
  </p>

  {#each cached as u (u.reciter_id)}
    <div class="usage-row">
      <span class="usage-name">{u.name_en}</span>
      <span class="usage-figure">{u.files} verses · {mb(u.bytes)}</span>
      {#if confirming === u.slug}
        <div class="confirm">
          <button class="btn danger" onclick={() => clear(u.slug)}>
            <Check size={13} /> Delete
          </button>
          <button class="btn" onclick={() => (confirming = null)} aria-label="Keep this audio">
            <X size={13} />
          </button>
        </div>
      {:else}
        <button
          class="btn danger"
          onclick={() => (confirming = u.slug)}
          aria-label="Delete {u.name_en}'s audio"
        >
          <Trash2 size={13} />
        </button>
      {/if}
    </div>
  {/each}

  {#if cached.length > 1}
    <div class="usage-row all">
      {#if confirming === 'all'}
        <div class="confirm">
          <button class="btn danger" onclick={() => clear()}>
            <Check size={13} /> Delete all audio
          </button>
          <button class="btn" onclick={() => (confirming = null)} aria-label="Keep all audio">
            <X size={13} />
          </button>
        </div>
      {:else}
        <button class="btn danger" onclick={() => (confirming = 'all')}>
          <Trash2 size={13} /> Clear all audio
        </button>
      {/if}
    </div>
  {/if}
</div>

<p class="note">
  Recitations are fetched from <code>cdn.islamic.network</code> one verse at a time, only while you play
  them. They are copyrighted recordings streamed from their publisher — this app neither hosts nor redistributes
  them.
</p>

<style>
  .reciters {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .reciter {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 10px;
    border: 1px solid var(--color-border);
    border-radius: 8px;
    background: transparent;
    color: var(--color-text);
    font-family: inherit;
    text-align: left;
    cursor: pointer;
  }

  .reciter:hover {
    background: var(--color-bg-hover);
  }

  .reciter.on {
    border-color: var(--color-accent);
  }

  .name {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 10px;
    font-size: 13px;
    font-weight: 500;
  }

  .native {
    font-family: var(--font-arabic-prose, inherit);
    font-size: 14px;
    font-weight: 400;
    color: var(--color-text-muted);
  }

  .meta {
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .storage {
    margin-top: 18px;
    padding-top: 14px;
    border-top: 1px solid var(--color-border);
  }

  .storage-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 16px;
  }

  .label {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text);
  }

  .total {
    font-size: 13px;
    font-variant-numeric: tabular-nums;
    color: var(--color-text);
  }

  .description {
    margin: 4px 0 10px;
    font-size: 12px;
    line-height: 1.5;
    color: var(--color-text-muted);
  }

  .usage-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 0;
    border-top: 1px solid var(--color-border);
    font-size: 12px;
  }

  .usage-row.all {
    justify-content: flex-end;
  }

  .usage-name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--color-text);
  }

  .usage-figure {
    color: var(--color-text-muted);
    font-variant-numeric: tabular-nums;
  }

  .confirm {
    display: flex;
    flex-shrink: 0;
    gap: 4px;
  }

  .btn {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    border: 1px solid var(--color-border);
    border-radius: 7px;
    background: transparent;
    color: var(--color-text-muted);
    font-family: inherit;
    font-size: 12px;
    cursor: pointer;
  }

  .btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .btn.danger:hover {
    border-color: var(--color-danger, #e5484d);
    color: var(--color-danger, #e5484d);
  }

  .note {
    margin: 20px 0 0;
    font-size: 12px;
    line-height: 1.6;
    color: var(--color-text-muted);
  }

  code {
    font-size: 11px;
  }
</style>
