<script lang="ts">
  import { onMount } from "svelte";
  import {
    api,
    connectBackend,
    emptySnapshot,
    type ConnectionStatus,
    type GuiSnapshot,
    type LibraryEntryDto,
  } from "./lib/api";
  import Sidebar from "./components/Sidebar.svelte";
  import PlayerBar from "./components/PlayerBar.svelte";
  import TrackTable from "./components/TrackTable.svelte";

  let snap = $state<GuiSnapshot>(emptySnapshot());
  let filterInput = $state("");
  let connection = $state<ConnectionStatus>("connecting");
  let connectionDetail = $state<string | undefined>();

  const sectionTitle = $derived(
    snap.tab === "queue"
      ? "Queue"
      : snap.tab === "library"
        ? "Your Library"
        : snap.playlist_viewing
          ? "Playlist"
          : "Playlists",
  );

  async function navigate(tab: string) {
    const next = await api.setTab(tab);
    snap = next;
  }

  async function applyFilter() {
    const next = await api.libraryFilter(filterInput);
    snap = next;
  }

  let filterTimer: ReturnType<typeof setTimeout> | undefined;
  function onFilterInput() {
    clearTimeout(filterTimer);
    filterTimer = setTimeout(() => applyFilter(), 320);
  }

  function entryRows(entries: LibraryEntryDto[]) {
    return entries.map((e, i) => ({
      i,
      title:
        e.kind === "folder"
          ? e.name === ".."
            ? "Go back"
            : e.name
          : e.title,
      isFolder: e.kind === "folder",
    }));
  }

  onMount(() => {
    let cleanup: (() => void) | undefined;
    connectBackend(
      (s) => {
        snap = s;
        if (snap.tab === "library" && filterInput !== snap.library_filter) {
          filterInput = snap.library_filter;
        }
      },
      (status, detail) => {
        connection = status;
        connectionDetail = detail;
      },
    ).then((fn) => {
      cleanup = fn;
    });
    return () => cleanup?.();
  });
</script>

<div class="app">
  <Sidebar
    tab={snap.tab}
    queueCount={snap.queue.length}
    libraryPath={snap.library_path}
    onNavigate={navigate}
  />

  <main class="main">
    <header class="top">
      <div class="hero">
        <p class="eyebrow">FireSuite</p>
        <h1>{sectionTitle}</h1>
      </div>
      <div class="status" class:live={connection === "live"} class:err={connection === "error"}>
        {#if connection === "live"}
          <span class="dot"></span> Backend live · rev {snap.revision}
        {:else if connection === "connecting"}
          Connecting to Rust backend…
        {:else}
          {connectionDetail ?? "Offline"}
        {/if}
      </div>
    </header>

    {#if snap.tab === "library"}
      <div class="search-row">
        <input
          class="search"
          placeholder="Search in library"
          bind:value={filterInput}
          oninput={onFilterInput}
        />
        <button class="ghost" onclick={() => api.libraryBack()}>Up</button>
        <button class="ghost" onclick={() => api.libraryRescan()}>Rescan</button>
        <button class="accent" onclick={() => api.libraryAddSelected()}>Add to queue</button>
      </div>
    {/if}

    <section class="content">
      {#if snap.tab === "queue"}
        <TrackTable
          tracks={snap.queue}
          selected={snap.queue_cursor}
          playing={snap.current_track_idx}
          showRemove
          onSelect={(i) => api.queueSelect(i).then((s) => (snap = s))}
          onPlay={(i) => api.queuePlay(i).then((s) => (snap = s))}
          onRemove={(i) => api.queueRemove(i).then((s) => (snap = s))}
        />
      {:else if snap.tab === "library"}
        {#if snap.library_entries.length === 0}
          <p class="empty">No tracks in {snap.library_path}. Try Rescan.</p>
        {:else}
          <div class="table">
            <div class="thead"><span>#</span><span>Name</span><span></span></div>
            {#each entryRows(snap.library_entries) as row}
              <button
                class="row"
                class:selected={row.i === snap.library_selected}
                onclick={() => api.librarySelect(row.i).then((s) => (snap = s))}
                ondblclick={() => api.libraryEnter().then((s) => (snap = s))}
              >
                <span class="idx">{row.isFolder ? "📁" : "♫"}</span>
                <span class="title">{row.title}</span>
                <span class="actions">
                  {#if !row.isFolder}
                    <span
                      class="play-hint"
                      role="button"
                      tabindex="0"
                      onclick={(e) => {
                        e.stopPropagation();
                        api.librarySelect(row.i).then(() => api.libraryAddSelected()).then((s) => (snap = s));
                      }}
                    >+</span>
                  {/if}
                </span>
              </button>
            {/each}
          </div>
        {/if}
      {:else if snap.playlist_viewing}
        <div class="search-row">
          <button class="ghost" onclick={() => api.playlistBack().then((s) => (snap = s))}>← Playlists</button>
        </div>
        <TrackTable
          tracks={snap.playlist_tracks}
          selected={snap.playlist_selected}
          playing={-1}
          onSelect={(i) => api.playlistSelect(i).then((s) => (snap = s))}
          onPlay={() => api.playlistLoad().then((s) => (snap = s))}
        />
      {:else}
        <div class="search-row">
          <button class="ghost" onclick={() => api.playlistRefresh().then((s) => (snap = s))}>Refresh</button>
        </div>
        {#if snap.playlists.length === 0}
          <p class="empty">No playlists in ~/.config/firemusic/playlists/</p>
        {:else}
          <div class="playlist-grid">
            {#each snap.playlists as name, i}
              <button
                class="card"
                class:selected={i === snap.playlist_selected}
                onclick={() => api.playlistSelect(i).then((s) => (snap = s))}
                ondblclick={() => api.playlistLoad().then((s) => (snap = s))}
              >
                <div class="card-art">📋</div>
                <p class="card-title">{name}</p>
                <p class="card-sub">Playlist</p>
              </button>
            {/each}
          </div>
        {/if}
      {/if}
    </section>

    {#if snap.status_msg}
      <div class="toast">{snap.status_msg}</div>
    {/if}
  </main>

  <PlayerBar {snap} />
</div>