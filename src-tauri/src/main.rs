// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri_plugin_store::Builder;

fn main() {
  tauri::Builder::default()
      .plugin(Builder::default().build())
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
}
