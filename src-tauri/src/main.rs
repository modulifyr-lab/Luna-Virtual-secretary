// Prevents additional console window on Windows in release, do not remove!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod brain;
mod commands;
mod config;
mod context;
mod memory;
mod skills;
mod stt;
mod tts;

fn main() {
    // TODO: Load application configuration
    let _app_config = config::load_config();

    // TODO: Register global push-to-talk hotkey using global-hotkey crate

    // TODO: Initialize SQLite memory store

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::process_user_input,
            commands::get_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
