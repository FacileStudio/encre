#import "tokens.typ": *

#let tone-colors(tone) = {
  if tone == "neutral" {
    (bg: fc-surface, fg: fc-fg, border: fc-border)
  } else if tone == "accent" {
    (bg: fc-accent, fg: fc-accent-fg, border: fc-accent)
  } else if tone == "info" {
    (bg: fc-info-bg, fg: fc-info, border: fc-info)
  } else if tone == "success" {
    (bg: fc-success-bg, fg: fc-success, border: fc-success)
  } else if tone == "warning" {
    (bg: fc-warning-bg, fg: fc-warning, border: fc-warning)
  } else if tone == "danger" {
    (bg: fc-danger-bg, fg: fc-danger, border: fc-danger)
  } else if tone == "owner" {
    (bg: fc-warning-bg, fg: fc-owner, border: fc-owner)
  } else if tone == "admin" {
    (bg: rgb("#f5f3ff"), fg: fc-admin, border: fc-admin)
  } else {
    (bg: fc-surface, fg: fc-fg, border: fc-border)
  }
}

#let badge(content, tone: "neutral", size: "sm") = {
  let colors = tone-colors(tone)
  let (pt-size, py, px) = if size == "xs" {
    (7pt, 2pt, 6pt)
  } else if size == "sm" {
    (7.5pt, 2.5pt, 7pt)
  } else {
    (8.5pt, 3.5pt, 9pt)
  }
  
  box(
    fill: colors.bg,
    radius: radius-pill,
    inset: (x: px, y: py),
    baseline: 5%,
    outset: 0pt,
    [
      #text(
        fill: colors.fg,
        size: pt-size,
        weight: "medium",
        font: font-sans,
        content
      )
    ]
  )
}

#let status-dot(tone: "success", size: 5pt) = {
  let colors = tone-colors(tone)
  box(
    circle(
      radius: size / 2,
      fill: colors.fg,
    )
  )
}

#let alert(body, title: none, tone: "info", stroke: none) = {
  let colors = tone-colors(tone)
  block(
    width: 100%,
    fill: colors.bg,
    radius: radius-md,
    inset: 12pt,
    stroke: stroke,
    breakable: false,
    [
      #if title != none [
        #text(weight: "semibold", size: 10pt, fill: colors.fg, title)
        #v(4pt)
      ]
      #text(size: 9.5pt, fill: fc-fg, body)
    ]
  )
}

#let card(body, title: none, subtitle: none, aside: none, fill: fc-component, inset: 13pt, radius: radius-md, stroke: none, breakable: false) = {
  block(
    width: 100%,
    fill: fill,
    radius: radius,
    inset: inset,
    stroke: stroke,
    breakable: breakable,
    [
      #if title != none [
        #grid(
          columns: (1fr, auto),
          align: (left, right + horizon),
          [
            #text(weight: "semibold", size: 11pt, fill: fc-fg, title)
            #if subtitle != none [
              #v(3pt)
              #text(size: 8.5pt, fill: fc-fg-muted, subtitle)
            ]
          ],
          aside
        )
        #v(8pt)
        #line(length: 100%, stroke: 0.5pt + fc-border-subtle)
        #v(8pt)
      ]
      #body
    ]
  )
}

#let metric(val, label, sub: none, tone: "accent", stroke: none) = {
  let colors = tone-colors(tone)
  block(
    width: 100%,
    fill: fc-component,
    radius: radius-md,
    inset: (x: 9pt, y: 10pt),
    stroke: stroke,
    breakable: false,
    [
      #text(size: 8pt, weight: "medium", fill: fc-fg-muted, label)
      #v(3pt)
      #text(size: 13pt, weight: "bold", fill: fc-fg, val)
      #if sub != none [
        #v(3pt)
        #text(size: 7.5pt, fill: fc-fg-muted, sub)
      ]
    ]
  )
}

#let metric-grid(columns: (1fr, 1fr, 1fr), ..metrics) = {
  grid(
    columns: columns,
    column-gutter: 10pt,
    row-gutter: 10pt,
    ..metrics.pos()
  )
}

#let divider(length: 100%, stroke: 0.5pt + fc-border) = {
  line(length: length, stroke: stroke)
}

#let step-item(num, title, role: none, budget: none, content, stroke: none) = {
  block(
    width: 100%,
    fill: fc-component,
    radius: radius-md,
    inset: 12pt,
    stroke: stroke,
    breakable: false,
    [
      #grid(
        columns: (auto, 1fr),
        gutter: 10pt,
        align: (center + top, left + horizon),
        [
          #circle(
            radius: 11pt,
            fill: fc-accent,
            align(center + horizon)[
              #text(fill: fc-accent-fg, weight: "bold", size: 9.5pt, str(num))
            ]
          )
        ],
        [
          #text(weight: "semibold", size: 10.5pt, fill: fc-fg, title)
          #if role != none or budget != none [
            #v(4pt)
            #if budget != none [
              #badge(budget, tone: "accent", size: "xs")
              #h(4pt)
            ]
            #if role != none [
              #badge(role, tone: "neutral", size: "xs")
            ]
          ]
        ]
      )
      #v(8pt)
      #line(length: 100%, stroke: 0.5pt + fc-border-subtle)
      #v(8pt)
      #content
    ]
  )
}

#let team-card(name, role, scope, stroke: none) = {
  block(
    width: 100%,
    fill: fc-component,
    radius: radius-md,
    inset: 12pt,
    stroke: stroke,
    breakable: false,
    [
      #grid(
        columns: (1fr, auto),
        align: (left + horizon, right + horizon),
        [
          #text(weight: "bold", size: 10.5pt, fill: fc-fg, name)
        ],
        [
          #badge(role, tone: "neutral", size: "xs")
        ]
      )
      #v(6pt)
      #text(size: 8.5pt, fill: fc-fg-muted, scope)
    ]
  )
}
