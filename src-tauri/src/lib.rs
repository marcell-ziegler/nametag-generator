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
    let csv_path = Path::new("./src/test.csv");
    let resources: &[&Path] = &[
        Path::new("./src/raketlager.png"),
        Path::new("./src/au-logga.png"),
    ];

    match compiler::compile(
        templ,
        csv_path,
        resources,
        "*Marcell Ziegler*: 123456789\\ *Johanna Zazzi:* 123456789",
    ) {
        Ok(pdf) => {
            std::fs::write("./exm.pdf", pdf).expect("Could not write PDF");
            println!("PDF written to ./exm.pdf");
        }
        Err(e) => eprintln!("Compilation failed: {e}"),
    }
}
