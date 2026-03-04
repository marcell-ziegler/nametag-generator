use std::{fs, path::Path};

use typst::foundations::{Dict, Str, Value};
use typst_as_lib::TypstEngine;
use typst_pdf::PdfOptions;

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

// Sätt in din csv här
// Ladda  upp och ange filnamn

// Inställningar
// #let tag_height = 6cm
// #let tag_width = 100%
// #let size = 26pt


#let generate(content_list, nodkontakt_info, tag_height: 6cm, tag_width: 100%, size: 26pt) = {
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
    [#nodkontakt(tag_width, tag_height, size, nodkontakt_info, qr_image: qr_image, qr_size: qr_size)]
  }
}"#;

const EXECUTION: &str = r#"#let cl = ()

#import sys: inputs

#let cl = ()

#for nr in csv(inputs.csv_path) {
  cl.push(au_nametag(nr))
}

#generate(
  cl,
  [#inputs.nodkontakt],
  tag_width: 8.9cm,
  tag_height: 5.5cm,
)"#;

pub fn compile(template: &str, csv_path: &Path, nodkontakt: &str) {
    // Setup file contents
    let mut main_file = String::from(PREAMBLE);
    main_file.push_str(template);
    main_file.push_str(EXECUTION);

    let template = TypstEngine::builder().main_file(main_file).build();

    let mut inputs = Dict::default();
    inputs.insert(
        Str::from("csv_path"),
        Value::Str(csv_path.to_str().unwrap().into()),
    );
    inputs.insert("nodkontakt".into(), Value::Str(nodkontakt.into()));
    let doc = template.compile_with_input(inputs).output.unwrap();

    let options = PdfOptions::default();
    let pdf = typst_pdf::pdf(&doc, &options).unwrap();

    fs::write("./exm.pdf", pdf).unwrap();
}
