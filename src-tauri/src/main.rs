// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use std::{
    io::{BufRead, BufReader, Write},
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};
use std::process::Command;
use tauri::{async_runtime::Mutex as AsyncMutex, State};
#[macro_use]
extern crate shells;
struct AppState {
    pty_pair: Arc<AsyncMutex<PtyPair>>,
    writer: Arc<AsyncMutex<Box<dyn Write + Send>>>,
}
#[tauri::command]
fn get_os_name(/* state: State<'_, AppState>*/) -> String {
    std::env::consts::OS.to_string()
}

#[tauri::command]
fn exec_cmd(cmd: &str) -> String {
    let output = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd").arg("/C").arg(cmd).output().unwrap()
    }else{
        std::process::Command::new("sh").arg("-c").arg(cmd).output().unwrap()
    };
    let x = std::str::from_utf8(&output.stdout[..]).unwrap();
    x.into()
}
#[tauri::command]
async fn async_shell(state: State<'_, AppState>) -> Result<(), ()> {
    #[cfg(target_os = "windows")]
        let cmd = CommandBuilder::new("powershell.exe");
    #[cfg(not(target_os = "windows"))]
        let(_,stdout,_)=sh!("echo $SHELL");
        let cmd = CommandBuilder::new(stdout.trim());
    let mut child = state.pty_pair.lock().await.slave.spawn_command(cmd).unwrap();

    thread::spawn(move || {
        child.wait().unwrap();
    });
    Ok(())

}

#[tauri::command]
async fn async_write_to_pty(data: &str, state: State<'_, AppState>) -> Result<(), ()> {
    write!(state.writer.lock().await, "{}", data).map_err(|_| ())
}

#[tauri::command]
async fn async_resize_pty(rows: u16, cols: u16, state: State<'_, AppState>) -> Result<(), ()> {
    state
        .pty_pair
        .lock()
        .await
        .master
        .resize(PtySize {
            rows,
            cols,
            ..Default::default()
        })
        .map_err(|_| ())
}

fn main() {
    let pty_system = native_pty_system();

    let pty_pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();



    let reader = pty_pair.master.try_clone_reader().unwrap();
    let writer = pty_pair.master.take_writer().unwrap();

    let reader = Arc::new(Mutex::new(Some(BufReader::new(reader))));

    tauri::Builder::default()
        .on_page_load(move |window, _| {
            let window = window.clone();
            let reader = reader.clone();

            thread::spawn(move || {
                let reader = reader.lock().unwrap().take();
                if let Some(mut reader) = reader {
                    loop {
                        sleep(Duration::from_millis(1));
                        let data = reader.fill_buf().unwrap().to_vec();
                        reader.consume(data.len());
                        if data.len() > 0 {
                            window.emit("data", data).unwrap();
                        }
                    }
                }
            });
        })
        .manage(AppState {
            pty_pair: Arc::new(AsyncMutex::new(pty_pair)),
            writer: Arc::new(AsyncMutex::new(writer)),
        })
        .invoke_handler(tauri::generate_handler![
            get_os_name,
            async_shell,
            async_write_to_pty,
            async_resize_pty
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
