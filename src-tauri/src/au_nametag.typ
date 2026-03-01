
#let nametag_content(name_and_role) = {
  let role = name_and_role.at(0)
  let name = shorten_name(name_and_role.at(1))

  [
    #par(leading: .8em)[#text(size, weight: "bold", font: "Exo 2")[#name]\
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
      [#image("../images/raketlager.png", height: 100%)], [#image("../images/au-logga.png", height: 100%)],
    ))
  ]
}
