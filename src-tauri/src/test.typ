// Get names as FirstName A. B. LastName
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
}

#let cl = ()


#let nametag_content(name_and_role) = {
  let role = name_and_role.at(0)
  let name = shorten_name(name_and_role.at(1))

  [
    #par(leading: .8em)[#text(size, weight: "bold", font: "Exo")[#name]\
      #text(.6 * size)[#role]]
    #v(10%)
    #rect(width: 100%, height: 4pt, stroke: none, fill: rgb("#ffb600"))
    #rect(width: 100%, height: 4pt, stroke: none, fill: rgb("#071d49"))
    #v(10%)
    #align(center, grid(
      columns: (1fr, 2fr),
      rows: 35%,
      align: (right, left),
      column-gutter: 10pt,
      [#image("raketlager.png", height: 100%)], [#image("au-logga.png", height: 100%)],
    ))
  ]
}

#for name_and_role in csv("./test.csv") {
  cl.push(nametag_content(name_and_role))
}

#generate(cl, [Marcell Ziegler: 123456])

