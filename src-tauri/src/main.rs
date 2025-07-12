#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem, PtyPair};
use std::env;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_store::StoreBuilder;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SshConnection {
    host: String,
    user: String,
    password: Option<String>,
}

// A state object to hold a thread-safe handle to the PTY writer and the PTY master
struct PtyState {
    pty_system: Arc<NativePtySystem>,
    pty_pair: Arc<Mutex<Option<PtyPair>>>,
    writer: Arc<Mutex<Option<Box<dyn Write + Send>>>>,
    reader_thread_handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

// This command receives input from the frontend and writes it to the PTY
#[tauri::command]
fn write_to_pty(bytes: Vec<u8>, state: State<PtyState>) {
    if let Ok(mut writer_guard) = state.writer.lock() {
        if let Some(writer) = writer_guard.as_mut() {
            let _ = writer.write_all(&bytes);
        }
    }
}

// A Tauri command to initiate an SSH connection or switch to a new process
#[tauri::command]
async fn connect_ssh(host: String, user: String, password: Option<String>, app_handle: AppHandle, state: State<'_, PtyState>) -> Result<String, String> {
    println!("Rust: connect_ssh called with host: {}, user: {}", host, user);
    let pty_system = state.pty_system.clone();

    println!("Rust: Attempting to acquire mutex guards...");
    // All operations that require MutexGuards should be done here,
    // and the guards should be dropped before any await.
    {
        let mut pty_pair_guard = state.pty_pair.lock().unwrap();
        let mut writer_guard = state.writer.lock().unwrap();
        let mut reader_thread_handle_guard = state.reader_thread_handle.lock().unwrap();
        println!("Rust: Mutex guards acquired.");

        println!("Rust: Terminating existing PTY process if any...");
        // Terminate existing PTY process if any
        if let Some(existing_pair) = pty_pair_guard.take() {
            drop(existing_pair.master);
            drop(existing_pair.slave);
            println!("Rust: Existing PTY pair dropped.");
        }
        if let Some(handle) = reader_thread_handle_guard.take() {
            println!("Rust: Detaching existing reader thread.");
            // The thread will continue to run until it finishes, or the PTY is closed.
            // We don't need to join it here, as it might panic or block.
        }
        println!("Rust: Existing PTY processes terminated.");

        println!("Rust: Opening a new PTY pair...");
        // Open a new PTY pair
        let new_pair = pty_system.openpty(PtySize::default()).map_err(|e| format!("Failed to create PTY: {}", e))?;
        *pty_pair_guard = Some(new_pair);
        println!("Rust: New PTY pair opened.");

        // Prepare the SSH command
        let mut cmd: CommandBuilder;
        if let Some(ref p) = password {
            println!("Rust: Password provided. Checking for sshpass...");
            // Check if sshpass is available
            match std::process::Command::new("sshpass").arg("-V").output() {
                Ok(output) => {
                    if !output.status.success() {
                        println!("Rust: sshpass check failed. Status: {:?}", output.status);
                        return Err("sshpass is not installed or not in PATH. Please install it to use password-based SSH connections.".to_string());
                    }
                    println!("Rust: sshpass found.");
                },
                Err(e) => {
                    println!("Rust: Failed to run sshpass check: {}", e);
                    return Err(format!("Failed to run sshpass check: {}. Is sshpass installed and in your PATH?", e));
                }
            }
            cmd = CommandBuilder::new("sshpass");
            cmd.arg(format!("-p{}", p));
            cmd.arg("ssh");
            cmd.arg(format!("{}@{}", user, host));
            println!("Rust: sshpass command prepared: {:?}", cmd);
        } else {
            cmd = CommandBuilder::new("ssh");
            cmd.arg(format!("{}@{}", user, host));
            println!("Rust: SSH command prepared (no password): {:?}", cmd);
        }
        cmd.env("TERM", "xterm-256color");

        println!("Rust: Attempting to spawn SSH command...");
        // Spawn the new process
        let child = pty_pair_guard.as_ref().unwrap().slave.spawn_command(cmd);
        match child {
            Ok(_) => println!("Rust: SSH command spawned successfully."),
            Err(e) => {
                eprintln!("Rust: Failed to spawn SSH command: {}", e);
                return Err(format!("Failed to spawn SSH command: {}", e));
            }
        }
        println!("Rust: SSH command spawned. Updating writer and reader...");

        // Update writer and restart reader thread
        *writer_guard = Some(pty_pair_guard.as_ref().unwrap().master.try_clone_writer().map_err(|e| format!("Failed to clone writer: {}", e))?);
        let new_reader = pty_pair_guard.as_ref().unwrap().master.try_clone_reader().map_err(|e| format!("Failed to clone reader: {}", e))?;
        *reader_thread_handle_guard = Some(pty_reader_thread(app_handle.clone(), new_reader));
    } // pty_pair_guard, writer_guard, reader_thread_handle_guard are dropped here

    // Now, call the async function
    let _ = save_connection(host.clone(), user.clone(), password.clone(), app_handle.clone()).await;
    Ok(format!("Successfully initiated SSH connection to {}@{}", user, host))
}

#[tauri::command]
async fn save_connection(host: String, user: String, password: Option<String>, app_handle: AppHandle) -> Result<(), String> {
    let mut store = StoreBuilder::new(&app_handle, "connections.dat".parse::<PathBuf>().unwrap()).build().map_err(|e| e.to_string())?;
    let connection = SshConnection { host, user, password };
    let mut connections: Vec<SshConnection> = store.get("connections").and_then(|v| serde_json::from_value(v).ok()).unwrap_or_else(Vec::new);
    connections.push(connection);
    store.set("connections".to_string(), serde_json::to_value(connections).unwrap());
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn load_connections(app_handle: AppHandle) -> Result<Vec<SshConnection>, String> {
    let store = StoreBuilder::new(&app_handle, "connections.dat".parse::<PathBuf>().unwrap()).build().map_err(|e| e.to_string())?;
    let connections: Vec<SshConnection> = store.get("connections").and_then(|v| serde_json::from_value(v).ok()).unwrap_or_else(Vec::new);
    Ok(connections)
}

#[tauri::command]
async fn delete_connection(host: String, user: String, _password: Option<String>, app_handle: AppHandle) -> Result<(), String> {
    let store = StoreBuilder::new(&app_handle, "connections.dat".parse::<PathBuf>().unwrap()).build().map_err(|e| e.to_string())?;
    let mut connections: Vec<SshConnection> = store.get("connections").and_then(|v| serde_json::from_value(v).ok()).unwrap_or_else(Vec::new);
    
    // Filter out the connection to be deleted
    connections.retain(|c| !(c.host == host && c.user == user));
    
    store.set("connections".to_string(), serde_json::to_value(connections).unwrap());
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

// A thread that continuously reads output from the shell and emits it to the frontend
fn pty_reader_thread(app: AppHandle, mut reader: Box<dyn Read + Send>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut buffer = [0u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(count) if count > 0 => {
                    println!("Rust: pty_reader_thread received {} bytes.", count);
                    if let Err(e) = app.emit("terminal-output", &buffer[..count]) {
                        eprintln!("Rust: Error emitting terminal output: {}", e);
                        break; // Break on emit error
                    }
                }
                Err(e) => {
                    eprintln!("Rust: Error reading from PTY: {}", e);
                    break; // Break on read error
                }
                _ => {
                    println!("Rust: pty_reader_thread received 0 bytes or PTY closed.");
                    break; // PTY has been closed or no bytes read
                }
            }
        }
        if let Err(e) = app.emit("terminal-output", "\r\n[SHELL EXITED]") {
            eprintln!("Rust: Error emitting shell exited message: {}", e);
        }
    })
}

fn main() {
    let pty_system = Arc::new(NativePtySystem::default());
    let initial_pair = pty_system
        .openpty(PtySize::default())
        .expect("Failed to create initial PTY");

    // Use the default shell for the user's system
    let shell = env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
    let mut cmd = CommandBuilder::new(shell);
    cmd.env("TERM", "xterm-256color");
    let _child = initial_pair.slave.spawn_command(cmd).expect("Failed to spawn initial shell");

    let writer = Arc::new(Mutex::new(Some(initial_pair.master.try_clone_writer().unwrap())));
    let reader = initial_pair.master.try_clone_reader().unwrap();

    let pty_state = PtyState {
        pty_system: pty_system.clone(),
        pty_pair: Arc::new(Mutex::new(Some(initial_pair))),
        writer,
        reader_thread_handle: Arc::new(Mutex::new(None)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(pty_state)
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let state_clone = app.state::<PtyState>();
            let mut reader_thread_handle_guard = state_clone.reader_thread_handle.lock().unwrap();
            *reader_thread_handle_guard = Some(pty_reader_thread(app_handle, reader));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![write_to_pty, connect_ssh, save_connection, load_connections, delete_connection])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}