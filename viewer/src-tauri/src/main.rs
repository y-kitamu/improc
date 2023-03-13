// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use image::{self, GenericImageView};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Image {
    size: Vec<u32>,
    data: Vec<u8>,
}

impl Image {
    pub fn from_dynamic_image(img: image::DynamicImage) -> Self {
        let (w, h) = img.dimensions();
        Image {
            size: vec![w, h],
            data: img.into_bytes(),
        }
    }
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn read_image(path: &str) -> Image {
    println!("Reading image from {}", path);
    match image::open(path) {
        Ok(image) => Image::from_dynamic_image(image),
        Err(_) => Image {
            size: vec![],
            data: vec![],
        },
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, read_image])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
