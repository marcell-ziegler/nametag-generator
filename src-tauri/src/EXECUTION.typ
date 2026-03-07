#let cl = ()

#import sys: inputs

#let cl = ()

#set text(font: "Open Sans")

#for nr in csv(inputs.csv_path) {
  cl.push(nametag_content(nr))
}

