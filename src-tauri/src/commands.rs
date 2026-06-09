use std::sync::Arc;

use firemusic::gui::runtime::{GuiCommand, GuiRuntime, tab_from_str};
use firemusic::gui::GuiSnapshot;
use tauri::State;

type Rt = Arc<GuiRuntime>;

#[tauri::command]
pub fn get_snapshot(runtime: State<'_, Rt>) -> GuiSnapshot {
    runtime.snapshot()
}

#[tauri::command]
pub fn set_tab(runtime: State<'_, Rt>, tab: String) {
    runtime.send(GuiCommand::SetTab(tab_from_str(&tab)));
}

#[tauri::command]
pub fn library_select(runtime: State<'_, Rt>, index: usize) {
    runtime.send(GuiCommand::LibrarySelect(index));
}

#[tauri::command]
pub fn library_enter(runtime: State<'_, Rt>) {
    runtime.send(GuiCommand::LibraryEnter);
}

#[tauri::command]
pub fn library_back(runtime: State<'_, Rt>) {
    runtime.send(GuiCommand::LibraryBack);
}

#[tauri::command]
pub fn library_rescan(runtime: State<'_, Rt>) {
    runtime.send(GuiCommand::LibraryRescan);
}

#[tauri::command]
pub fn library_filter(runtime: State<'_, Rt>, filter: String) {
    runtime.send(GuiCommand::LibraryFilter(filter));
}

#[tauri::command]
pub fn library_add_selected(runtime: State<'_, Rt>) {
    runtime.send(GuiCommand::LibraryAddSelected);
}

#[tauri::command]
pub fn queue_select(runtime: State<'_, Rt>, index: usize) {
    runtime.send(GuiCommand::QueueSelect(index));
}

#[tauri::command]
pub fn queue_play(runtime: State<'_, Rt>, index: usize) {
    runtime.send(GuiCommand::QueuePlay(index));
}

#[tauri::command]
pub fn queue_remove(runtime: State<'_, Rt>, index: usize) {
    runtime.send(GuiCommand::QueueRemove(index));
}

#[tauri::command]
pub fn toggle_pause(runtime: State<'_, Rt>) {
    runtime.send(GuiCommand::TogglePause);
}

#[tauri::command]
pub fn seek(runtime: State<'_, Rt>, delta: f64) {
    runtime.send(GuiCommand::Seek(delta));
}

#[tauri::command]
pub fn set_volume(runtime: State<'_, Rt>, volume: f64) {
    runtime.send(GuiCommand::SetVolume(volume));
}

#[tauri::command]
pub fn toggle_mute(runtime: State<'_, Rt>) {
    runtime.send(GuiCommand::ToggleMute);
}

#[tauri::command]
pub fn toggle_loop(runtime: State<'_, Rt>) {
    runtime.send(GuiCommand::ToggleLoop);
}

#[tauri::command]
pub fn playlist_select(runtime: State<'_, Rt>, index: usize) {
    runtime.send(GuiCommand::PlaylistSelect(index));
}

#[tauri::command]
pub fn playlist_load(runtime: State<'_, Rt>) {
    runtime.send(GuiCommand::PlaylistLoad);
}

#[tauri::command]
pub fn playlist_back(runtime: State<'_, Rt>) {
    runtime.send(GuiCommand::PlaylistBack);
}

#[tauri::command]
pub fn playlist_refresh(runtime: State<'_, Rt>) {
    runtime.send(GuiCommand::PlaylistRefresh);
}