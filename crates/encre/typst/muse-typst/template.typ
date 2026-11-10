#import "tokens.typ": *
#import "components.typ": *

#let muse-doc(
  title: "",
  subtitle: none,
  client: none,
  recipient: none,
  date: none,
  version: "1.0",
  author: "Facile Studio",
  doc-type: "Étude de cadrage & Proposition",
  status: none,
  cover: true,
  logo: "assets/facile-logo.svg",
  header-logo: auto,
  header-logo-height: 0.38cm,
  footer: [Facile Studio · Document de cadrage confidentiel],
  body
) = {
  let running-logo = if header-logo == auto { logo } else { header-logo }

  set document(
    title: title,
    author: author,
  )

  set page(
    paper: "a4",
    margin: (top: 2.6cm, bottom: 2.4cm, left: 2.4cm, right: 2.4cm),
    header: context {
      let page-num = counter(page).get().first()
      if cover and page-num == 1 {
        none
      } else {
        block(
          width: 100%,
          stroke: (bottom: 0.5pt + fc-border),
          inset: (bottom: 8pt),
          [
            #grid(
              columns: (1fr, auto),
              align: (left + horizon, right + horizon),
              [
                #if running-logo != none [
                  #box(image(running-logo, height: header-logo-height), baseline: 12%)
                ] else [
                  #text(size: 8pt, weight: "bold", fill: fc-fg, [FACILE])
                  #h(3pt)
                  #text(size: 8pt, fill: fc-fg-muted, [STUDIO])
                ]
                #h(6pt)
                #text(size: 8pt, fill: fc-border, [·])
                #h(6pt)
                #text(size: 8pt, fill: fc-fg-muted, doc-type)
              ],
              [
                #if client != none [
                  #text(size: 8pt, weight: "medium", fill: fc-fg, client)
                  #if status != none [
                    #h(6pt)
                    #badge(status, tone: "neutral", size: "xs")
                  ]
                ]
              ]
            )
          ]
        )
      }
    },
    footer: context {
      let page-num = counter(page).get().first()
      let total-pages = counter(page).final().first()
      if cover and page-num == 1 {
        none
      } else {
        block(
          width: 100%,
          stroke: (top: 0.5pt + fc-border),
          inset: (top: 8pt),
          [
            #grid(
              columns: (1fr, 1fr),
              align: (left, right),
              [
                #if footer != none [
                  #text(size: 8pt, fill: fc-fg-muted, footer)
                ]
              ],
              [
                #text(size: 8pt, fill: fc-fg-muted, [Page #page-num sur #total-pages])
              ]
            )
          ]
        )
      }
    }
  )

  set text(
    font: font-sans,
    size: 9.5pt,
    fill: fc-fg,
    lang: "fr",
    spacing: 100%,
  )

  set par(
    justify: true,
    leading: 0.65em,
    spacing: 1.1em,
  )

  show heading.where(level: 1): it => block(
    width: 100%,
    above: 2.2em,
    below: 1.2em,
    breakable: false,
    [
      #text(
        size: 15pt,
        weight: "bold",
        fill: fc-fg,
        it.body
      )
      #v(6pt)
      #line(length: 100%, stroke: 0.5pt + fc-border)
      #v(4pt)
    ]
  )

  show heading.where(level: 2): it => block(
    above: 1.8em,
    below: 0.9em,
    breakable: false,
    [
      #text(
        size: 11.5pt,
        weight: "bold",
        fill: fc-fg,
        it.body
      )
    ]
  )

  show heading.where(level: 3): it => block(
    above: 1.4em,
    below: 0.7em,
    breakable: false,
    [
      #text(
        size: 10pt,
        weight: "semibold",
        fill: fc-fg,
        it.body
      )
    ]
  )

  show table: set table(
    stroke: (x, y) => if y == 0 { (bottom: 1pt + fc-fg) } else { (bottom: 0.5pt + fc-border-subtle) },
    fill: (x, y) => if y == 0 { fc-surface } else { none },
    inset: (x: 9pt, y: 7pt),
  )

  show raw.where(block: true): it => block(
    width: 100%,
    fill: fc-surface,
    radius: radius-md,
    inset: 10pt,
    stroke: none,
    breakable: false,
    [
      #set text(font: font-mono, size: 8pt)
      #it
    ]
  )

  show raw.where(block: false): it => box(
    fill: fc-surface,
    radius: radius-xs,
    inset: (x: 3.5pt, y: 2pt),
    baseline: 0%,
    [
      #set text(font: font-mono, size: 8pt, fill: fc-fg)
      #it
    ]
  )

  if cover {
    v(2cm)
    
    grid(
      columns: (1fr, auto),
      align: (left + horizon, right + horizon),
      [
        #if logo != none [
          #image(logo, height: 1.1cm)
        ] else [
          #text(size: 14pt, weight: "bold", fill: fc-fg, [FACILE])
          #h(4pt)
          #text(size: 14pt, weight: "light", fill: fc-fg-muted, [STUDIO])
        ]
      ],
      [
        #if status != none [
          #badge(status, tone: "accent", size: "sm")
        ]
      ]
    )

    v(3.5cm)

    block(
      width: 100%,
      [
        #badge(doc-type, tone: "neutral", size: "sm")
        #v(12pt)
        #text(size: 26pt, weight: "bold", fill: fc-fg, tracking: -0.5pt, title)
        #if subtitle != none [
          #v(10pt)
          #text(size: 12.5pt, fill: fc-fg-muted, subtitle)
        ]
      ]
    )

    v(4.5cm)

    line(length: 100%, stroke: 0.5pt + fc-border)
    v(12pt)

    grid(
      columns: (1fr, 1fr, 1fr),
      gutter: 15pt,
      [
        #text(size: 8pt, weight: "bold", fill: fc-fg-muted, [DESTINATAIRE])
        #v(4pt)
        #text(size: 10pt, weight: "semibold", fill: fc-fg, if recipient != none { recipient } else { "David" }) \
        #if client != none [
          #text(size: 8.5pt, fill: fc-fg-muted, client)
        ]
      ],
      [
        #text(size: 8pt, weight: "bold", fill: fc-fg-muted, [ÉMIS PAR])
        #v(4pt)
        #text(size: 10pt, weight: "semibold", fill: fc-fg, author) \
        #text(size: 8.5pt, fill: fc-fg-muted, [facile.studio])
      ],
      [
        #text(size: 8pt, weight: "bold", fill: fc-fg-muted, [DATE & VERSION])
        #v(4pt)
        #text(size: 10pt, weight: "semibold", fill: fc-fg, if date != none { date } else { "Août 2026" }) \
        #text(size: 8.5pt, fill: fc-fg-muted, [Version #version])
      ]
    )

    pagebreak()
  }

  body
}
