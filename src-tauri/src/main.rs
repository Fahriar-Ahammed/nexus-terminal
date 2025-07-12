#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter, State}; // Removed unused Manager
use tauri_plugin_store::StoreBuilder;

// --- Data Structure for SSH connections ---
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct SshConnection {
    id: String,
    name: String,
    host: String,
    user: String,
    password: Option<String>,
}

// --- PTY State ---
struct PtyState {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

// --- Tauri Commands ---

// Command to write user input to the terminal
#[tauri::command]
fn write_to_pty(text: String, state: State<PtyState>) {
    if let Ok(mut writer) = state.writer.lock() {
        let _ = writer.write_all(text.as_bytes());
    }
}

// Command to get the list of saved SSH connections
#[tauri::command]
async fn get_connections(app_handle: AppHandle) -> Result<Vec<SshConnection>, String> {
    let store = StoreBuilder::new(&app_handle, "connections.json".parse::<PathBuf>().unwrap()).build().map_err(|e| e.to_string())?;
    let connections = match store.get("connections") {
        Some(value) => serde_json::from_value(value.clone()).unwrap_or_else(|_| Vec::new()),
        None => Vec::new(),
    };
    Ok(connections)
}

// Command to save the entire list of SSH connections
#[tauri::command]
async fn update_connections(connections: Vec<SshConnection>, app_handle: AppHandle) -> Result<(), String> {
    let mut store = StoreBuilder::new(&app_handle, "connections.json".parse::<PathBuf>().unwrap()).build().map_err(|e| e.to_string())?;

    // THE FIX: The `?` operator is removed from this line.
    store.set("connections".to_string(), serde_json::to_value(&connections).unwrap());

    // The .save() method *does* return a Result, so this line is correct.
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

// --- PTY Management Thread ---
fn pty_reader_thread(app: AppHandle, mut reader: Box<dyn Read + Send>) {
    thread::spawn(move || {
        let mut buffer = [0u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(count) if count > 0 => {
                    let _ = app.emit("terminal-output", &buffer[..count]);
                }
                _ => break,
            }
        }
        let _ = app.emit("terminal-output", "\r\n[SHELL EXITED]");
    });
}

fn main() {
    let pty_system = NativePtySystem::default();
    let pair = pty_system
        .openpty(PtySize::default())
        .expect("Failed to create initial PTY");

    let shell = env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
    let mut cmd = CommandBuilder::new(shell);
    cmd.env("TERM", "xterm-256color");
    let _child = pair.slave.spawn_command(cmd).expect("Failed to spawn initial shell");

    let writer = Arc::new(Mutex::new(pair.master.try_clone_writer().unwrap()));
    let reader = pair.master.try_clone_reader().unwrap();

    let pty_state = PtyState { writer };

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(pty_state)
        .setup(move |app| {
            pty_reader_thread(app.handle().clone(), reader);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            write_to_pty,
            get_connections,
            update_connections
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}