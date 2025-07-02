#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::env;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter, Manager, State};

// A state object to hold a thread-safe handle to the PTY writer
struct PtyState {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

// This command receives input from the frontend and writes it to the PTY
#[tauri::command]
fn write_to_pty(bytes: Vec<u8>, state: State<PtyState>) {
    if let Ok(mut writer) = state.writer.lock() {
        // Write the bytes directly to the shell process
        let _ = writer.write_all(&bytes);
    }
}

// A thread that continuously reads output from the shell and emits it to the frontend
fn pty_reader_thread(app: AppHandle, mut reader: Box<dyn Read + Send>) {
    thread::spawn(move || {
        let mut buffer = [0u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(count) if count > 0 => {
                    let _ = app.emit("terminal-output", &buffer[..count]);
                }
                _ => break, // PTY has been closed
            }
        }
        let _ = app.emit("terminal-output", "\r\n[SHELL EXITED]");
    });
}

fn main() {
    let pty_system = NativePtySystem::default();
    let pair = pty_system
        .openpty(PtySize::default())
        .expect("Failed to create PTY");

    // Use the default shell for the user's system
    let shell = env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
    let mut cmd = CommandBuilder::new(shell);
    cmd.env("TERM", "xterm-256color");
    let _child = pair.slave.spawn_command(cmd).expect("Failed to spawn shell");

    let writer = Arc::new(Mutex::new(pair.master.try_clone_writer().unwrap()));
    let reader = pair.master.try_clone_reader().unwrap();

    let pty_state = PtyState { writer };

    tauri::Builder::default()
        .manage(pty_state)
        .setup(move |app| {
            // Start the reader thread when the app is ready
            pty_reader_thread(app.handle().clone(), reader);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![write_to_pty])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}