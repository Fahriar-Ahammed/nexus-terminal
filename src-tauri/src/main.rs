#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::Mutex;
use std::thread;
use tauri::{AppHandle, Emitter, Manager, State}; // Emitter is now imported

// This struct will hold our shell's writer process.
// We will manage it using Tauri's state management.
struct PtyState {
    writer: Mutex<Box<dyn Write + Send>>,
}

// This command is exposed to the frontend. It gets the writer from Tauri's state.
#[tauri::command]
fn write_to_shell(text: String, state: State<PtyState>) -> Result<(), String> {
    let mut writer = state.writer.lock().unwrap();
    write!(writer, "{}", text).map_err(|e| e.to_string())
}

// This function spawns the PTY and starts the background reader thread.
fn spawn_pty(app_handle: AppHandle) {
    // This runs in a background thread
    thread::spawn(move || {
        // Create the PTY
        let pty_system = NativePtySystem::default();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("Failed to create PTY");

        // Spawn a shell into the PTY
        let mut cmd = CommandBuilder::new("bash");
        cmd.env("TERM", "xterm-256color");
        let _child = pair.slave.spawn_command(cmd).expect("Failed to spawn shell");

        // Get the PTY reader and writer
        let mut reader = pair.master.try_clone_reader().expect("Failed to clone reader");
        let writer = pair.master; // The master IS the writer

        // Get a handle to the state object and replace the dummy writer with the real one
        let state = app_handle.state::<PtyState>();
        *state.writer.lock().unwrap() = writer;

        // Background thread to continuously read from the PTY
        let mut buffer = [0u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(count) => {
                    if count > 0 {
                        let chunk = &buffer[..count];
                        // Emit the raw byte data to the frontend
                        app_handle.emit("terminal-output", chunk).unwrap();
                    } else {
                        // EOF, the shell process has exited.
                        app_handle.emit("terminal-output", "\r\n[SHELL EXITED]").unwrap();
                        break;
                    }
                }
                Err(e) => {
                    let error_msg = format!("\r\n[ERROR] {}", e);
                    app_handle.emit("terminal-output", error_msg).unwrap();
                    break;
                }
            }
        }
    });
}

fn main() {
    // Create a dummy writer for the initial state. It will be replaced once the PTY is spawned.
    let dummy_writer = Mutex::new(Box::new(std::io::sink()) as Box<dyn Write + Send>);

    tauri::Builder::default()
        .manage(PtyState { writer: dummy_writer })
        .setup(|app| {
            // When the app starts, spawn the PTY in the background
            let handle = app.handle().clone();
            spawn_pty(handle);
            Ok(())
        })
        // Register our `write_to_shell` command
        .invoke_handler(tauri::generate_handler![write_to_shell])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}