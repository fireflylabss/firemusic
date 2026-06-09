<script lang="ts">
  import { onMount } from "svelte";
  import {
    api,
    emptySnapshot,
    fetchSnapshot,
    onState,
    type GuiSnapshot,
    type LibraryEntryDto,
  } from "./lib/api";

  let snap = $state<GuiSnapshot>(emptySnapshot());
  let filterInput = $state("");

  const tabs = [
    { id: "queue", label: "Queue" },
    { id: "library", label: "Library" },
    { id: "playlists", label: "Playlists" },
  ];

  function fmtTime(sec: number) {
    const s = Math.max(0, Math.floor(sec));
    const m = Math.floor(s / 60);
    const r = s % 60;
    return `${String(m).padStart(2, "0")}:${String(r).padStart(2, "0")}`;
  }

  function progressPct() {
    if (snap.playback.duration <= 0) return 0;
    return (snap.playback.time / snap.playback.duration) * 100;
  }

  function coverSrc() {
    return snap.cover_base64 ? `data:image/png;base64,${snap.cover_base64}` : null;
  }

  function entryLabel(entry: LibraryEntryDto) {
    if (entry.kind === "folder") return entry.name === ".." ? "⬅ .." : `📁 ${entry.name}`;
    return `🎵 ${entry.title}`;
  }

  async function setTab(tab: string) {
    await api.setTab(tab);
  }

  async function applyFilter() {
    await api.libraryFilter(filterInput);
  }

  onMount(() => {
    let unlisten: (() => void) | undefined;

    (async () => {
      try {
        snap = await fetchSnapshot();
      } catch {
        /* browser preview without tauri */
      }
      unlisten = await onState((s) => {
        snap = s;
        if (snap.tab === "library" && filterInput !== snap.library_filter) {
          filterInput = snap.library_filter;
        }
      });
    })();

    return () => unlisten?.();
  });
</script>

<div class="app-shell">
  <header class="titlebar">🔥 Firemusic</header>

  <div class="body">
    <aside class="sidebar">
      <h3>Playback</h3>
      <div class="stat"><span>Volume</span><strong>{Math.round(snap.playback.volume)}%</strong></div>
      <div class="stat"><span>Speed</span><strong>{snap.playback.speed.toFixed(1)}x</strong></div>
      <div class="stat"><span>Loop</span><strong>{snap.playback.is_loop ? "on" : "off"}</strong></div>
      <div class="stat"><span>Muted</span><strong>{snap.playback.muted ? "yes" : "no"}</strong></div>

      <h3 style="margin-top: 1rem">Library</h3>
      <div class="stat"><span>Path</span><strong>{snap.library_path}</strong></div>
      <div class="stat"><span>Queue</span><strong>{snap.queue.length} tracks</strong></div>
    </aside>

    <section class="main">
      <nav class="tabs">
        {#each tabs as tab}
          <button class:active={snap.tab === tab.id} onclick={() => setTab(tab.id)}>
            {tab.label}
          </button>
        {/each}
      </nav>

      {#if snap.tab === "queue"}
        <div class="panel">
          {#if snap.queue.length === 0}
            <p class="empty">Queue empty — browse Library to add tracks.</p>
          {:else}
            <ul class="list">
              {#each snap.queue as track, i}
                <li
                  class:selected={i === snap.queue_cursor}
                  class:playing={i === snap.current_track_idx}
                  onclick={() => api.queueSelect(i)}
                  ondblclick={() => api.queuePlay(i)}
                >
                  <span>{i + 1}.</span> {track.title}
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {:else if snap.tab === "library"}
        <div class="toolbar">
          <input
            placeholder="Filter library…"
            bind:value={filterInput}
            onkeydown={(e) => e.key === "Enter" && applyFilter()}
          />
          <button onclick={applyFilter}>Filter</button>
          <button onclick={() => api.libraryRescan()}>Rescan</button>
          <button onclick={() => api.libraryAddSelected()}>Add</button>
        </div>
        <div class="panel">
          {#if snap.library_entries.length === 0}
            <p class="empty">No entries — check music folder or rescan.</p>
          {:else}
            <ul class="list">
              {#each snap.library_entries as entry, i}
                <li
                  class:selected={i === snap.library_selected}
                  onclick={() => api.librarySelect(i)}
                  ondblclick={() => api.libraryEnter()}
                >
                  {entryLabel(entry)}
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {:else if snap.tab === "playlists"}
        <div class="toolbar">
          {#if snap.playlist_viewing}
            <button onclick={() => api.playlistBack()}>Back</button>
          {:else}
            <button onclick={() => api.playlistRefresh()}>Refresh</button>
          {/if}
          <button class="primary" onclick={() => api.playlistLoad()}>
            {snap.playlist_viewing ? "Add track" : "Open"}
          </button>
        </div>
        <div class="panel">
          {#if snap.playlist_viewing}
            <ul class="list">
              {#each snap.playlist_tracks as track, i}
                <li
                  class:selected={i === snap.playlist_selected}
                  onclick={() => api.playlistSelect(i)}
                  ondblclick={() => api.playlistLoad()}
                >
                  {i + 1}. {track.title}
                </li>
              {/each}
            </ul>
          {:else if snap.playlists.length === 0}
            <p class="empty">No playlists in ~/.config/firemusic/playlists/</p>
          {:else}
            <ul class="list">
              {#each snap.playlists as name, i}
                <li
                  class:selected={i === snap.playlist_selected}
                  onclick={() => api.playlistSelect(i)}
                  ondblclick={() => api.playlistLoad()}
                >
                  📋 {name}
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {/if}
    </section>
  </div>

  <footer class="now-playing">
    {#if coverSrc()}
      <img class="cover" src={coverSrc()} alt="Cover art" />
    {:else}
      <div class="cover placeholder">🎵</div>
    {/if}

    <div class="track-meta">
      <h2>{snap.playback.title}</h2>
      <div class="stat" style="margin:0">
        <span>{fmtTime(snap.playback.time)} / {fmtTime(snap.playback.duration)}</span>
      </div>
      <div class="progress"><span style={`width:${progressPct()}%`}></span></div>
    </div>

    <div class="controls">
      <button class="primary" onclick={() => api.togglePause()}>
        {snap.playback.paused ? "Play" : "Pause"}
      </button>
      <button onclick={() => api.seek(-5)}>-5s</button>
      <button onclick={() => api.seek(5)}>+5s</button>
      <button onclick={() => api.toggleMute()}>Mute</button>
      <button onclick={() => api.toggleLoop()}>Loop</button>
      <button onclick={() => api.setVolume(Math.min(100, snap.playback.volume + 5))}>Vol+</button>
      <button onclick={() => api.setVolume(Math.max(0, snap.playback.volume - 5))}>Vol-</button>
      {#if snap.tab === "queue" && snap.queue.length > 0}
        <button onclick={() => api.queueRemove(snap.queue_cursor)}>Remove</button>
      {/if}
    </div>
  </footer>

  <div class="statusbar">{snap.status_msg ?? "Ready"}</div>
</div>