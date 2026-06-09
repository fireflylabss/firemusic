<script lang="ts">
  import type { TrackDto } from "../lib/api";

  interface Props {
    tracks: TrackDto[];
    selected: number;
    playing: number;
    onSelect: (i: number) => void;
    onPlay: (i: number) => void;
    onRemove?: (i: number) => void;
    showRemove?: boolean;
  }

  let {
    tracks,
    selected,
    playing,
    onSelect,
    onPlay,
    onRemove,
    showRemove = false,
  }: Props = $props();
</script>

{#if tracks.length === 0}
  <p class="empty">Nothing here yet.</p>
{:else}
  <div class="table">
    <div class="thead">
      <span>#</span>
      <span>Title</span>
      <span></span>
    </div>
    {#each tracks as track, i}
      <button
        class="row"
        class:selected={i === selected}
        class:playing={i === playing}
        onclick={() => onSelect(i)}
        ondblclick={() => onPlay(i)}
      >
        <span class="idx">
          {#if i === playing}
            <span class="eq">♫</span>
          {:else}
            {i + 1}
          {/if}
        </span>
        <span class="title">{track.title}</span>
        <span class="actions">
          <span
            class="play-hint"
            role="button"
            tabindex="0"
            onclick={(e) => { e.stopPropagation(); onPlay(i); }}
            onkeydown={(e) => e.key === "Enter" && onPlay(i)}
          >▶</span>
          {#if showRemove && onRemove}
            <span
              class="remove"
              role="button"
              tabindex="0"
              onclick={(e) => { e.stopPropagation(); onRemove(i); }}
              onkeydown={(e) => e.key === "Enter" && onRemove(i)}
            >✕</span>
          {/if}
        </span>
      </button>
    {/each}
  </div>
{/if}