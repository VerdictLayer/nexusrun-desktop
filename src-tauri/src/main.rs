#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

use tauri::Manager;

struct Engine(Mutex<Option<Child>>);

// The engine ships next to the app; during development fall back to PATH.
fn engine_path() -> PathBuf {
    let name = if cfg!(windows) { "nexus.exe" } else { "nexus" };
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let beside = dir.join(name);
            if beside.exists() {
                return beside;
            }
        }
    }
    PathBuf::from(name)
}

fn workspace() -> PathBuf {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join("NexusRun")
}

fn main() {
    tauri::Builder::default()
        .manage(Engine(Mutex::new(None)))
        .setup(|app| {
            let mut child = Command::new(engine_path())
                .args(["studio", "--no-browser", "--workspace"])
                .arg(workspace())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("could not start the NexusRun engine: {e}"))?;
            let out = child.stdout.take().ok_or("engine gave no output")?;
            *app.state::<Engine>().0.lock().unwrap() = Some(child);

            // The engine prints its address once, as "NexusRun Studio → <link>".
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                for line in BufReader::new(out).lines().map_while(Result::ok) {
                    if let Some(link) = line.split("→").nth(1).map(str::trim) {
                        if link.starts_with("http://127.0.0.1:") || link.starts_with("http://[::1]:") {
                            if let Some(win) = handle.get_webview_window("main") {
                                if let Ok(url) = link.parse() {
                                    let _ = win.navigate(url);
                                }
                            }
                            break;
                        }
                    }
                }
            });
            Ok(())
        })
        .on_window_event(|win, event| {
            if let tauri::WindowEvent::Destroyed = event {
                if let Some(mut c) = win.state::<Engine>().0.lock().unwrap().take() {
                    let _ = c.kill();
                    let _ = c.wait();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("failed to run NexusRun Studio");
}
