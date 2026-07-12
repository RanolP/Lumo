// HTML-export wrapper around the repo-root formalization document:
// equations are unsupported by typst's HTML export, so render each one
// as an SVG frame. html.frame alone is block-level and would split the
// surrounding paragraph, so inline equations ride an inline <span>.
// Compiled with --root <repo root>.
#show math.equation.where(block: false): it => context if target() == "html" {
  html.elem("span", attrs: (class: "inline-eq"), html.frame(box(it)))
} else { it }
#show math.equation.where(block: true): it => context if target() == "html" {
  html.frame(it)
} else { it }
#include "/formalization.typ"
