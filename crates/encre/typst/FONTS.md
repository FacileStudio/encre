# Fonts embedded in the PDF compiler

`pdf::compile` runs with no system fonts. It embeds the files in `fonts/` and nothing else. A font is only added when its licence allows redistribution and embedding in PDF files.

| File | Family | Version | Licence | Embedding |
|---|---|---|---|---|
| `fonts/DejaVuSans.ttf` | DejaVu Sans | 2.37 | Bitstream Vera Fonts licence, DejaVu changes public domain (`fonts/LICENSE`) | allowed, no restriction |
| `fonts/DejaVuSans-Bold.ttf` | DejaVu Sans Bold | 2.37 | same | allowed |
| `fonts/DejaVuSansMono.ttf` | DejaVu Sans Mono | 2.37 | same | allowed |
| `fonts/DejaVuSansMono-Bold.ttf` | DejaVu Sans Mono Bold | 2.37 | same | allowed |

Source: Debian `fonts-dejavu-core` 2.37-8, upstream https://dejavu-fonts.github.io/. The licence text ships next to the files, as it requires.

## Not embedded

- **Satoshi**, the first family in muse-typst's `font-sans`, is distributed by Fontshare under the ITF Free Font License. It does not grant redistribution of the font files inside other software, so it is not in this repository. Documents fall back to DejaVu Sans.
- **Menlo**, muse-typst's first `font-mono`, is Apple's and not redistributable. Code falls back to DejaVu Sans Mono.
- There is no italic face: DejaVu Sans Oblique is not in `fonts-dejavu-core`. Typst then sets emphasis upright. Add `DejaVuSans-Oblique.ttf` and `DejaVuSans-BoldOblique.ttf` (same licence) from the full DejaVu release when italics matter.
