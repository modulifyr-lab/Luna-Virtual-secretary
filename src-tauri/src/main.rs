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

use tauri::Manager;

fn main() {
    let app_config = config::load_config();

    tauri::Builder::default()
        .setup(move |app| {
            app.manage(app_config.clone());
            let handle = app.handle();

            let kokoro_tts = tts::setup_tts(handle.clone(), &app_config);
            app.manage(kokoro_tts);

            if let Err(e) = stt::setup_hotkey_and_stt(handle, &app_config) {
                eprintln!("[Main] Failed to setup STT and Hotkey: {}", e);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::process_user_input,
            commands::get_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
