import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface TrackDto {
  title: string;
  path: string;
}

export type LibraryEntryDto =
  | { kind: "folder"; name: string }
  | { kind: "track"; title: string; path: string };

export interface PlaybackDto {
  title: string;
  time: number;
  duration: number;
  paused: boolean;
  muted: boolean;
  volume: number;
  speed: number;
  is_loop: boolean;
}

export interface GuiSnapshot {
  tab: string;
  queue: TrackDto[];
  queue_cursor: number;
  current_track_idx: number;
  library_path: string;
  library_entries: LibraryEntryDto[];
  library_selected: number;
  library_is_root: boolean;
  library_filter: string;
  playlists: string[];
  playlist_tracks: TrackDto[];
  playlist_viewing: boolean;
  playlist_selected: number;
  playback: PlaybackDto;
  status_msg: string | null;
  cover_base64: string | null;
}

export const emptySnapshot = (): GuiSnapshot => ({
  tab: "queue",
  queue: [],
  queue_cursor: 0,
  current_track_idx: 0,
  library_path: "~",
  library_entries: [],
  library_selected: 0,
  library_is_root: true,
  library_filter: "",
  playlists: [],
  playlist_tracks: [],
  playlist_viewing: false,
  playlist_selected: 0,
  playback: {
    title: "...",
    time: 0,
    duration: 0,
    paused: true,
    muted: false,
    volume: 100,
    speed: 1,
    is_loop: false,
  },
  status_msg: null,
  cover_base64: null,
});

export async function fetchSnapshot(): Promise<GuiSnapshot> {
  return invoke<GuiSnapshot>("get_snapshot");
}

export function onState(cb: (snap: GuiSnapshot) => void) {
  return listen<GuiSnapshot>("state", (event) => cb(event.payload));
}

export const api = {
  setTab: (tab: string) => invoke("set_tab", { tab }),
  librarySelect: (index: number) => invoke("library_select", { index }),
  libraryEnter: () => invoke("library_enter"),
  libraryBack: () => invoke("library_back"),
  libraryRescan: () => invoke("library_rescan"),
  libraryFilter: (filter: string) => invoke("library_filter", { filter }),
  libraryAddSelected: () => invoke("library_add_selected"),
  queueSelect: (index: number) => invoke("queue_select", { index }),
  queuePlay: (index: number) => invoke("queue_play", { index }),
  queueRemove: (index: number) => invoke("queue_remove", { index }),
  togglePause: () => invoke("toggle_pause"),
  seek: (delta: number) => invoke("seek", { delta }),
  setVolume: (volume: number) => invoke("set_volume", { volume }),
  toggleMute: () => invoke("toggle_mute"),
  toggleLoop: () => invoke("toggle_loop"),
  playlistSelect: (index: number) => invoke("playlist_select", { index }),
  playlistLoad: () => invoke("playlist_load"),
  playlistBack: () => invoke("playlist_back"),
  playlistRefresh: () => invoke("playlist_refresh"),
};