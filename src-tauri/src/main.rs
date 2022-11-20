#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use png::DecodingError;
use serde::Serialize;
use std::{
    fs::File,
    sync::{
        atomic::{AtomicU32, Ordering},
        Mutex,
    },
};
use tauri::{Manager};

fn read_png_metadata(filename: &str) -> anyhow::Result<Vec<(String, String)>> {
    let f = File::open(filename)?;
    let decoder = png::Decoder::new(f);
    let reader = decoder.read_info().map_err(|e| {
        match e {
            DecodingError::Format(_) => {
                anyhow::anyhow!("Not a PNG file: {}", filename)
            }
            _ => {
                anyhow::anyhow!("Failed to open PNG: {}", e)
            }
        }
    })?;
    let info = reader.info();
    let ret = info
        .uncompressed_latin1_text
        .iter()
        .map(|chunk| (chunk.keyword.clone(), chunk.text.clone()))
        .collect();
    Ok(ret)
}

#[derive(Debug, Clone, Serialize)]
struct Image {
    id: u32,
    filename: String,
    metadata: Vec<(String, String)>,
}

#[derive(Debug)]
struct State {
    images: Mutex<Vec<Image>>,
    next_id: AtomicU32,
    focus_on: AtomicU32,
}

impl State {
    fn new() -> Self {
        Self {
            images: Mutex::new(vec![]),
            next_id: AtomicU32::new(1),
            focus_on: AtomicU32::new(0),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct StateChangedPayload {
    images: Vec<Image>,
    focus_on: u32,
}

fn emit_state_changed(
    state: tauri::State<'_, State>,
    app_handle: tauri::AppHandle,
) {
    let images = state.images.lock().unwrap();
    app_handle
        .emit_all(
            "state-changed",
            StateChangedPayload {
                images: images.to_vec(),
                focus_on: state.focus_on.load(Ordering::SeqCst),
            },
        )
        .unwrap();
}

#[tauri::command]
fn add_image(
    filename: &str,
    state: tauri::State<'_, State>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    {
        let metadata = read_png_metadata(filename).map_err(|e| e.to_string())?;
        let mut images = state.images.lock().unwrap();
        let id = state.next_id.fetch_add(1, Ordering::SeqCst);
        images.push(Image {
            id,
            filename: filename.to_owned(),
            metadata,
        });
        state.focus_on.store(id, Ordering::SeqCst);
    }
    emit_state_changed(state, app_handle);
    Ok(())
}

#[tauri::command]
fn remove_image(
    id: u32,
    state: tauri::State<'_, State>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    {
        let mut images = state.images.lock().unwrap();
        images.retain(|image| image.id != id);
        state.focus_on.store(0, Ordering::SeqCst);
    }
    emit_state_changed(state, app_handle);
    Ok(())
}

fn main() {
    let state = State::new();
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![add_image, remove_image,])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
