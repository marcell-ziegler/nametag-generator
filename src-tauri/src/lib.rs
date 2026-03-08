use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::compiler::{compile, CompilationError};

mod compiler;

#[tauri::command]
async fn compile_to_bytes(
    template: String,
    csv_path: String,
    resource_paths: Vec<String>,
    nodkontakt: String,
) -> Result<Vec<u8>, CompilationError> {
    let csv_pathbuf: PathBuf = csv_path.into();

    let resource_pathbufs: Vec<PathBuf> = resource_paths.into_iter().map(PathBuf::from).collect();
    tauri::async_runtime::spawn_blocking(move || {
        let resource_refs: Vec<&Path> = resource_pathbufs.iter().map(|p| p.as_path()).collect();
        compile(
            &template,
            &csv_pathbuf,
            resource_refs.as_slice(),
            &nodkontakt,
        )
    })
    .await
    .map_err(|e| CompilationError::TypstError {
        phase: compiler::TypstPhase::Compile,
        msg: format!("worker thread failed: {e}"),
    })?
}

#[tauri::command]
async fn compile_to_file(
    template: String,
    csv_path: String,
    resource_paths: Vec<String>,
    nodkontakt: String,
    destination_path: String,
) -> Result<(), CompilationError> {
    let csv_pathbuf: PathBuf = csv_path.into();

    let resource_pathbufs: Vec<PathBuf> = resource_paths.into_iter().map(PathBuf::from).collect();
    let pdf_bytes = tauri::async_runtime::spawn_blocking(move || {
        let resource_refs: Vec<&Path> = resource_pathbufs.iter().map(|p| p.as_path()).collect();
        compile(
            &template,
            &csv_pathbuf,
            resource_refs.as_slice(),
            &nodkontakt,
        )
    })
    .await
    .map_err(|e| CompilationError::TypstError {
        phase: compiler::TypstPhase::Compile,
        msg: format!("worker thread failed: {e}"),
    })??;

    let dest = PathBuf::from(destination_path);

    fs::write(dest.as_path(), pdf_bytes).map_err(|e| CompilationError::FileNotWritable {
        path: dest,
        source: e,
    })?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![compile_to_file, compile_to_bytes])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    // let templ = include_str!("./au_nametag.typ");
    // let csv_path = Path::new("./src/test.csv");
    // let resources: &[&Path] = &[
    //     Path::new("./src/raketlager.png"),
    //     Path::new("./src/au-logga.png"),
    // ];
    //
    // match compiler::compile(
    //     templ,
    //     csv_path,
    //     resources,
    //     "*Marcell Ziegler*: 123456789\\ *Johanna Zazzi:* 123456789",
    // ) {
    //     Ok(pdf) => {
    //         std::fs::write("./exm.pdf", pdf).expect("Could not write PDF");
    //         println!("PDF written to ./exm.pdf");
    //     }
    //     Err(e) => eprintln!("Compilation failed: {e}"),
    // }
}
