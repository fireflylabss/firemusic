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
  revision: number;
  backend_online: boolean;
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
  revision: 0,
  backend_online: false,
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
    title: "Not connected",
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

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    throw new Error("Backend unavailable — run via firemusic-gui / tauri dev");
  }
  return invoke<T>(cmd, args);
}

export async function fetchSnapshot(): Promise<GuiSnapshot> {
  return call<GuiSnapshot>("get_snapshot");
}

export async function pingBackend(): Promise<GuiSnapshot> {
  return call<GuiSnapshot>("ping");
}

export function onState(cb: (snap: GuiSnapshot) => void) {
  if (!isTauri()) return Promise.resolve(() => {});
  return listen<GuiSnapshot>("state", (event) => cb(event.payload));
}

export const api = {
  setTab: (tab: string) => call<GuiSnapshot>("set_tab", { tab }),
  librarySelect: (index: number) => call<GuiSnapshot>("library_select", { index }),
  libraryEnter: () => call<GuiSnapshot>("library_enter"),
  libraryBack: () => call<GuiSnapshot>("library_back"),
  libraryRescan: () => call<GuiSnapshot>("library_rescan"),
  libraryFilter: (filter: string) => call<GuiSnapshot>("library_filter", { filter }),
  libraryAddSelected: () => call<GuiSnapshot>("library_add_selected"),
  queueSelect: (index: number) => call<GuiSnapshot>("queue_select", { index }),
  queuePlay: (index: number) => call<GuiSnapshot>("queue_play", { index }),
  queueRemove: (index: number) => call<GuiSnapshot>("queue_remove", { index }),
  togglePause: () => call<GuiSnapshot>("toggle_pause"),
  seek: (delta: number) => call<GuiSnapshot>("seek", { delta }),
  seekTo: (position: number) => call<GuiSnapshot>("seek_to", { position }),
  nextTrack: () => call<GuiSnapshot>("next_track"),
  prevTrack: () => call<GuiSnapshot>("prev_track"),
  setVolume: (volume: number) => call<GuiSnapshot>("set_volume", { volume }),
  toggleMute: () => call<GuiSnapshot>("toggle_mute"),
  toggleLoop: () => call<GuiSnapshot>("toggle_loop"),
  playlistSelect: (index: number) => call<GuiSnapshot>("playlist_select", { index }),
  playlistLoad: () => call<GuiSnapshot>("playlist_load"),
  playlistBack: () => call<GuiSnapshot>("playlist_back"),
  playlistRefresh: () => call<GuiSnapshot>("playlist_refresh"),
};

export type ConnectionStatus = "connecting" | "live" | "offline" | "error";

export async function connectBackend(
  onUpdate: (snap: GuiSnapshot) => void,
  onStatus: (status: ConnectionStatus, detail?: string) => void,
): Promise<() => void> {
  if (!isTauri()) {
    onStatus("offline", "Open with firemusic-gui or bun run tauri:dev");
    return () => {};
  }

  onStatus("connecting");
  let lastRevision = -1;
  let staleTicks = 0;
  const cleanups: Array<() => void> = [];

  const apply = (snap: GuiSnapshot) => {
    if (snap.revision !== lastRevision) {
      lastRevision = snap.revision;
      staleTicks = 0;
      onStatus("live");
    }
    onUpdate(snap);
  };

  try {
    apply(await pingBackend());
  } catch (err) {
    onStatus("error", String(err));
    return () => {};
  }

  const unlisten = await onState(apply);
  if (unlisten) cleanups.push(unlisten);

  const poll = setInterval(async () => {
    try {
      const snap = await fetchSnapshot();
      if (snap.revision === lastRevision) {
        staleTicks += 1;
        if (staleTicks > 8) onStatus("error", "Backend not responding");
      } else {
        apply(snap);
      }
    } catch (err) {
      onStatus("error", String(err));
    }
  }, 500);
  cleanups.push(() => clearInterval(poll));

  return () => cleanups.forEach((fn) => fn());
}