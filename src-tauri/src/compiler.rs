use std::{
    collections::HashSet,
    io,
    path::{Path, PathBuf},
};

use thiserror::Error;
use typst::foundations::{Dict, Str, Value};
use typst_as_lib::{typst_kit_options::TypstKitFontOptions, TypstEngine};
use typst_pdf::PdfOptions;

/// The phase of Typst processing where an error occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypstPhase {
    Compile,
    Pdf,
}

impl std::fmt::Display for TypstPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypstPhase::Compile => write!(f, "compile"),
            TypstPhase::Pdf => write!(f, "pdf"),
        }
    }
}

/// Errors that can occur during PDF compilation.
#[derive(Debug, Error)]
pub enum CompilationError {
    /// A required file was not found or is not a file.
    #[error("file not found: {0}")]
    FileNotFound(PathBuf),

    /// A file could not be read.
    #[error("file not readable: {path}: {source}")]
    FileNotReadable {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    /// A file path does not have a valid filename component.
    #[error("invalid filename: {0}")]
    InvalidFileName(PathBuf),

    /// Two or more files share the same filename.
    #[error("filename collision: \"{0}\"")]
    NameCollision(String),

    /// A Typst processing error.
    #[error("typst {phase} error: {msg}")]
    TypstError { phase: TypstPhase, msg: String },
}

impl serde::Serialize for CompilationError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

const PREAMBLE: &str = r#"// Get names as FirstName A. B. LastName
#let shorten_name(name) = {
  let split_names = name.split(" ") // For middle name separation
  let names = [
    #split_names.at(0) // First name
    // Make every middle name a single letter and dot
    #for subname in split_names.slice(1, -1) {
      [#subname.first(). ]
    }
    // Last name
    #split_names.at(-1)
  ]
  return names
}

// Return a box with emergency contact info printed from the content in info.
#let nodkontakt(width, height, size, info) = {
  align(horizon + center, box(width: width, height: height, stroke: (black + .5pt), align(horizon + center, [
    #text(size: size, weight: "bold")[NÖDKONTAKT]\
    #v(10%)
    #par(leading: .8em, text(size: .6 * size)[#info])\
  ])))
}

#let nametag(width, height, content) = {
  align(center + horizon, box(width: width, height: height, stroke: (black + .5pt), content))
}

// Inställningar
#let tag_height = 6cm
#let tag_width = 100%
#let size = 26pt


#let generate(content_list, nodkontakt_info) = {
  // Space efficiency
  set page(margin: .3cm)
  set par(spacing: 0pt, leading: 0pt)
  set block(below: 0pt, above: 0pt)

  // Align everything to the center and add columns
  show: it => { columns(2, gutter: 0cm, align(right + horizon)[#it]) }

  set columns(gutter: .1cm)

  let num_tag = calc.floor(297mm / tag_height) * 2

  let num = 0
  // Generate nametags
  for i in range(content_list.len()) {
    if calc.rem-euclid(i, num_tag) == 0 and i != 0 {
      for j in range(num_tag) {
        [#nodkontakt(tag_width, tag_height, size, nodkontakt_info)]
      }
    }
    num += 1
    [#nametag(tag_width, tag_height, content_list.at(i))]
  }

  if (calc.rem-euclid(num, num_tag) != 0) {
    for i in range(num_tag - calc.rem-euclid(num, num_tag)) {
      [#nametag(tag_width, tag_height, [])]
    }
  }

  colbreak()

  for j in range(num_tag) {
    [#nodkontakt(tag_width, tag_height, size, nodkontakt_info)]
  }
}"#;

const EXECUTION: &str = r#"#let cl = ()

#import sys: inputs

#let cl = ()

#for nr in csv(inputs.csv_path) {
  cl.push(nametag_content(nr))
}
"#;

/// Compile a Typst nametag document into PDF bytes.
///
/// # Arguments
/// * `template` – Typst source for the `nametag_content` function.
/// * `csv_path` – Path to the CSV file.  The file is read as bytes and fed to
///   Typst via a static resolver so that `csv("<filename>")` works inside the
///   document.  No filesystem resolver is used.
/// * `resources` – Additional resource files (images, fonts, …).  Each file is
///   read as bytes and registered under its bare filename.  Filenames must be
///   unique across all supplied resources and the CSV file.
/// * `nodkontakt` – Emergency-contact string rendered in the document.
///   The string is interpreted as Typst markup: formatting such as `*bold*`
///   or `_italic_` will be rendered accordingly.
///
///   # Security
///   The string is embedded verbatim into the Typst source inside a content
///   block (`[…]`).  Do not pass untrusted user input here, as arbitrary
///   Typst code could be injected.
///
/// # Errors
/// Returns a [`CompilationError`] if any file cannot be found or read, if two
/// files share the same filename, or if Typst fails during compilation or PDF
/// generation.
pub fn compile(
    template: &str,
    csv_path: &Path,
    resources: &[&Path],
    nodkontakt: &str,
) -> Result<Vec<u8>, CompilationError> {
    // --- Validate and read the CSV ---
    if !csv_path.is_file() {
        return Err(CompilationError::FileNotFound(csv_path.to_path_buf()));
    }
    let csv_name = csv_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| CompilationError::InvalidFileName(csv_path.to_path_buf()))?
        .to_owned();
    let csv_bytes = std::fs::read(csv_path).map_err(|e| CompilationError::FileNotReadable {
        path: csv_path.to_path_buf(),
        source: e,
    })?;

    // --- Validate and read resource files, checking for name collisions ---
    let mut seen_names: HashSet<String> = HashSet::new();
    seen_names.insert(csv_name.clone());

    let mut resource_blobs: Vec<(String, Vec<u8>)> = Vec::new();
    for &res_path in resources {
        if !res_path.is_file() {
            return Err(CompilationError::FileNotFound(res_path.to_path_buf()));
        }
        let res_name = res_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| CompilationError::InvalidFileName(res_path.to_path_buf()))?
            .to_owned();
        if !seen_names.insert(res_name.clone()) {
            return Err(CompilationError::NameCollision(res_name));
        }
        let bytes =
            std::fs::read(res_path).map_err(|e| CompilationError::FileNotReadable {
                path: res_path.to_path_buf(),
                source: e,
            })?;
        resource_blobs.push((res_name, bytes));
    }

    // --- Build the Typst document source ---
    // nodkontakt is embedded as a Typst content literal so the markup it
    // contains is parsed and rendered correctly (e.g. *bold*, _italic_).
    let mut main_file = String::from(PREAMBLE);
    main_file.push_str(template);
    main_file.push_str(EXECUTION);
    main_file.push_str(&format!("#generate(\n  cl,\n  [{}],\n)", nodkontakt));

    // --- Assemble static binary resolver entries ---
    // CSV first, then remaining resources.
    let mut static_files: Vec<(&str, &[u8])> =
        Vec::with_capacity(1 + resource_blobs.len());
    static_files.push((csv_name.as_str(), csv_bytes.as_slice()));
    for (name, bytes) in &resource_blobs {
        static_files.push((name.as_str(), bytes.as_slice()));
    }

    let engine = TypstEngine::builder()
        .main_file(main_file)
        .with_static_file_resolver(static_files)
        .search_fonts_with(TypstKitFontOptions::default())
        .build();

    // --- Inject sys.inputs ---
    let mut inputs = Dict::default();
    inputs.insert(Str::from("csv_path"), Value::Str(csv_name.into()));
    // nodkontakt is not passed through sys.inputs — it is embedded directly
    // in the Typst source as a content literal (see main_file construction above).

    // --- Compile ---
    let doc = engine
        .compile_with_input(inputs)
        .output
        .map_err(|e| CompilationError::TypstError {
            phase: TypstPhase::Compile,
            msg: format!("{e}"),
        })?;

    // --- Generate PDF ---
    let options = PdfOptions::default();
    let pdf = typst_pdf::pdf(&doc, &options).map_err(|e| CompilationError::TypstError {
        phase: TypstPhase::Pdf,
        msg: format!("{e:?}"),
    })?;

    Ok(pdf)
}
