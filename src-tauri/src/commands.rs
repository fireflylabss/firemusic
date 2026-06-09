use std::sync::Arc;

use firemusic::gui::runtime::{GuiCommand, GuiRuntime, tab_from_str};
use firemusic::gui::GuiSnapshot;
use tauri::State;

type Rt = Arc<GuiRuntime>;

fn dispatch(runtime: &GuiRuntime, cmd: GuiCommand) -> GuiSnapshot {
    runtime.send(cmd);
    runtime.snapshot()
}

#[tauri::command]
pub fn get_snapshot(runtime: State<'_, Rt>) -> GuiSnapshot {
    runtime.snapshot()
}

#[tauri::command]
pub fn ping(runtime: State<'_, Rt>) -> GuiSnapshot {
    runtime.snapshot()
}

#[tauri::command]
pub fn set_tab(runtime: State<'_, Rt>, tab: String) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::SetTab(tab_from_str(&tab)))
}

#[tauri::command]
pub fn library_select(runtime: State<'_, Rt>, index: usize) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::LibrarySelect(index))
}

#[tauri::command]
pub fn library_enter(runtime: State<'_, Rt>) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::LibraryEnter)
}

#[tauri::command]
pub fn library_back(runtime: State<'_, Rt>) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::LibraryBack)
}

#[tauri::command]
pub fn library_rescan(runtime: State<'_, Rt>) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::LibraryRescan)
}

#[tauri::command]
pub fn library_filter(runtime: State<'_, Rt>, filter: String) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::LibraryFilter(filter))
}

#[tauri::command]
pub fn library_add_selected(runtime: State<'_, Rt>) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::LibraryAddSelected)
}

#[tauri::command]
pub fn queue_select(runtime: State<'_, Rt>, index: usize) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::QueueSelect(index))
}

#[tauri::command]
pub fn queue_play(runtime: State<'_, Rt>, index: usize) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::QueuePlay(index))
}

#[tauri::command]
pub fn queue_remove(runtime: State<'_, Rt>, index: usize) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::QueueRemove(index))
}

#[tauri::command]
pub fn toggle_pause(runtime: State<'_, Rt>) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::TogglePause)
}

#[tauri::command]
pub fn seek(runtime: State<'_, Rt>, delta: f64) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::Seek(delta))
}

#[tauri::command]
pub fn seek_to(runtime: State<'_, Rt>, position: f64) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::SeekTo(position))
}

#[tauri::command]
pub fn next_track(runtime: State<'_, Rt>) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::NextTrack)
}

#[tauri::command]
pub fn prev_track(runtime: State<'_, Rt>) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::PrevTrack)
}

#[tauri::command]
pub fn set_volume(runtime: State<'_, Rt>, volume: f64) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::SetVolume(volume))
}

#[tauri::command]
pub fn toggle_mute(runtime: State<'_, Rt>) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::ToggleMute)
}

#[tauri::command]
pub fn toggle_loop(runtime: State<'_, Rt>) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::ToggleLoop)
}

#[tauri::command]
pub fn playlist_select(runtime: State<'_, Rt>, index: usize) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::PlaylistSelect(index))
}

#[tauri::command]
pub fn playlist_load(runtime: State<'_, Rt>) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::PlaylistLoad)
}

#[tauri::command]
pub fn playlist_back(runtime: State<'_, Rt>) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::PlaylistBack)
}

#[tauri::command]
pub fn playlist_refresh(runtime: State<'_, Rt>) -> GuiSnapshot {
    dispatch(&runtime, GuiCommand::PlaylistRefresh)
}