use std::path::Path;

mod compiler;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // tauri::Builder::default()
    // .plugin(tauri_plugin_opener::init())
    // .invoke_handler(tauri::generate_handler![greet])
    // .run(tauri::generate_context!())
    // .expect("error while running tauri application");

    let templ = include_str!("./au_nametag.typ");
    let path = Path::new("./src/test.csv");

    compiler::compile(templ, path, "Marcell Ziegler: 123456789");
}
