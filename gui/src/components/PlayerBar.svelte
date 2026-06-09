<script lang="ts">
  import type { GuiSnapshot } from "../lib/api";
  import { api } from "../lib/api";
  import { coverDataUrl, fmtTime } from "../lib/format";

  interface Props {
    snap: GuiSnapshot;
  }

  let { snap }: Props = $props();

  let scrubbing = $state(false);
  let scrubValue = $state(0);

  const cover = $derived(coverDataUrl(snap.cover_base64));
  const progress = $derived(
    snap.playback.duration > 0
      ? (snap.playback.time / snap.playback.duration) * 100
      : 0,
  );

  $effect(() => {
    if (!scrubbing) scrubValue = progress;
  });

  async function onScrubInput() {
    scrubbing = true;
  }

  async function onScrubCommit() {
    scrubbing = false;
    const pos = (scrubValue / 100) * snap.playback.duration;
    await api.seekTo(pos);
  }
</script>

<footer class="player">
  <div class="left">
    {#if cover}
      <img class="art" src={cover} alt="" />
    {:else}
      <div class="art placeholder">♪</div>
    {/if}
    <div class="meta">
      <p class="title">{snap.playback.title}</p>
      <p class="sub">Firemusic · MPV</p>
    </div>
  </div>

  <div class="center">
    <div class="timeline">
      <span class="time">{fmtTime(snap.playback.time)}</span>
      <input
        type="range"
        min="0"
        max="100"
        step="0.1"
        bind:value={scrubValue}
        oninput={onScrubInput}
        onchange={onScrubCommit}
        class="scrub"
      />
      <span class="time">{fmtTime(snap.playback.duration)}</span>
    </div>
    <div class="transport">
      <button class="icon-btn" title="Previous" onclick={() => api.prevTrack()}>⏮</button>
      <button class="play-btn" title="Play/Pause" onclick={() => api.togglePause()}>
        {snap.playback.paused ? "▶" : "⏸"}
      </button>
      <button class="icon-btn" title="Next" onclick={() => api.nextTrack()}>⏭</button>
    </div>
  </div>

  <div class="right">
    <button
      class="icon-btn"
      class:active={snap.playback.is_loop}
      title="Loop"
      onclick={() => api.toggleLoop()}
    >🔁</button>
    <button
      class="icon-btn"
      class:active={snap.playback.muted}
      title="Mute"
      onclick={() => api.toggleMute()}
    >{snap.playback.muted ? "🔇" : "🔊"}</button>
    <input
      type="range"
      min="0"
      max="100"
      value={snap.playback.volume}
      class="volume"
      oninput={(e) => api.setVolume(Number((e.target as HTMLInputElement).value))}
    />
  </div>
</footer>