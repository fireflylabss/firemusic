<script lang="ts">
  interface Props {
    tab: string;
    queueCount: number;
    libraryPath: string;
    onNavigate: (tab: string) => void;
  }

  let { tab, queueCount, libraryPath, onNavigate }: Props = $props();

  const items = [
    { id: "queue", label: "Queue", icon: "♫" },
    { id: "library", label: "Your Library", icon: "▦" },
    { id: "playlists", label: "Playlists", icon: "☰" },
  ];
</script>

<aside class="sidebar">
  <div class="brand">
    <span class="logo">🔥</span>
    <span class="name">Firemusic</span>
  </div>

  <nav class="nav">
    {#each items as item}
      <button
        class="nav-item"
        class:active={tab === item.id}
        onclick={() => onNavigate(item.id)}
      >
        <span class="icon">{item.icon}</span>
        {item.label}
        {#if item.id === "queue" && queueCount > 0}
          <span class="badge">{queueCount}</span>
        {/if}
      </button>
    {/each}
  </nav>

  <div class="library-meta">
    <p class="label">Library path</p>
    <p class="path">{libraryPath}</p>
  </div>
</aside>