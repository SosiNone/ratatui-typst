# ratatui-typst demo

A workspace application for exploring the root `ratatui-typst` library. Its
path dependency points to `..`, and both packages share the root Cargo.lock.
The demo is unpublished and excluded from the library crate archive.

From the repository root:

```sh
cargo run --locked -p ratatui-typst-demo
cargo run --locked -p ratatui-typst-demo -- 'A = mat(1, 2; 3, 4)'
cargo run --locked -p ratatui-typst-demo -- --halfblocks
cargo run --locked -p ratatui-typst-demo -- --png output/preview.png
cargo run --locked -p ratatui-typst-demo -- --showcase-assets assets/demo
```

Or run `task demo` from the repository root. `task quadratic`,
`task integral`, `task sum`, and `task matrix` open the explorer with that
example selected. `task previews` exports those four equations under
ignored `output/` without opening a terminal UI.

`task showcase-assets` regenerates the four committed README equation images.
`--showcase-assets DIR` is headless and cannot be combined with positional math,
`--png`, or `--halfblocks`. It writes `quadratic.png`, `integral.png`, `matrix.png`,
and `piecewise.png`, replacing previous exports with those names. These use
20pt text, 10pt padding, 2 pixels per point, and an opaque Paper palette for
readability in either README theme. They are renderer outputs, not terminal captures.

## Guided tour and explorer

Launching without positional math opens a six-scene guided tour: Terminal
notebook, Calculator console, Linear algebra, Worked derivation, Presentation
controls, and Error recovery. Each scene combines a real application setting
with a library capability, visible Typst source, and a live result. The tour
uses a scene navigation rail at 100 columns by 30 rows or larger, and a compact
progress indicator at smaller supported sizes.

Press `x` to switch between tour and explorer without changing the current
source, image, context, or rendering settings. Navigating to a tour scene loads
that scene's source, context, and palette preset. Unmatched source appears as
Custom equation; `n` resumes at the first scene and `p` at the last.
`e` opens the explorer editor, and Tab from the tour opens rendering controls.
Supplying positional math starts directly in the explorer, even if it matches
a built-in example.

The executable is named `ratatui-typst-demo`. Use a terminal at least 76 columns
by 24 rows. The gallery covers fractions, roots, integrals, sums, matrices,
aligned equations, piecewise functions, vectors, font warnings, and compiler
errors. In the explorer, the source, image preview, natural dimensions, raster
size, terminal cell requirements, compile time, and diagnostics stay visible together.

The preview opens as a terminal note with a heading, an explanation, the
rendered equation, and follow-up text. This surrounding text uses ordinary
terminal characters and the same foreground/background palette as the image,
so you can compare font size, padding, and how the image blends into the panel.
Press `c` to cycle through Note, Console (prompt and output), and Plain views.
The equation uses its proportionally fitted height, with one blank row before
the following text when space allows. Surrounding text has reserved space before
oversized equations are fitted. The same rectangles drive preparation and drawing.

Explorer defaults are transparent backgrounds and 2pt padding. Text size starts at
`1.2 * cell_height_pixels / pixels_per_pt`, clamped to 8–48pt: 12pt for a 20px
cell and the default 2 pixels per point. Reset in the explorer restores this
compact profile.
Tour scenes use 1.5 times the compact text size, clamped to the same 8–48pt
range, with 4pt padding and transparent backgrounds. Reset in a recognized
tour scene restores its preset; in Custom equation it restores explorer defaults.
Headless PNG exports use a 20px reference cell height and retain transparency;
`s` exports the current settings. The library's opaque 20pt/10pt defaults are
unchanged. The current math font and mathematical italics are retained.

| Key | Action |
| --- | --- |
| x | Switch tour / explorer, preserving source and settings |
| Up / Down or j / k | Browse tour scenes, explorer examples, or rendering controls |
| Tab / Shift+Tab | Open explorer controls from the tour; switch Examples / Rendering in the explorer |
| Left / Right | Previous / next tour scene; adjust a selected explorer rendering control |
| - / + | Adjust the selected explorer rendering control |
| n / p | Next / previous scene or example |
| e | Open the explorer editor with the current source |
| c | Cycle Note / Console / Plain preview context |
| r | Restore the tour scene preset or explorer rendering defaults |
| s | Save the current image to `output/equation.png` relative to the working directory, replacing the previous export |
| PageUp / PageDown | Scroll render information and diagnostics |
| q / Esc / Ctrl+C | Quit (Esc cancels when editing) |

While editing, arrows, Home, End, Backspace, and Delete work on the source.
Enter applies and compiles; Alt+Enter inserts a newline (Shift+Enter also works
if the terminal distinguishes it). Tab applies the source and moves to
Rendering. Esc discards the draft; `q` is ordinary text. Supply Typst math
without surrounding dollar delimiters. Invalid input displays diagnostics and
removes the preview until corrected; the Font warning example keeps a valid
preview while showing the warning.

Rendering controls expose text size (8–48 pt), padding (0–32 pt), raster scale
(0.5–4 pixels per point), three foreground/background palettes, and automatic
or half-block previews. Images shrink proportionally when space is limited;
they do not grow to fill the panel. Raster scale changes pixel resolution as
well as the natural terminal cell requirements.

Typst is embedded, and protocol detection is handled by ratatui-image once at
startup. `--halfblocks` skips terminal capability queries; detection errors
also fall back to half-blocks with a status message. Half-block previews are
coarse. Typesetting and image preparation run outside the draw callback;
preparation is repeated after resizing or changing the preview backend.
Compilation is synchronous and can briefly delay keyboard input.
Kitty keeps full alpha; SIXEL blends glyph edges against the panel color after
resizing and leaves clear pixels undrawn. Half-blocks use an opaque composite.
The demo enables Ratatui's `unstable-rendered-line-info` feature to measure
wrapped paragraphs consistently when reserving their space.

The maintainer reports good Windows rendering with the Sixel backend confirmed.
Detailed lifecycle checks remain unconfirmed; Kitty validation is deferred
beyond 0.1.0. Review background continuity, text size, spacing, palette changes,
replacement, resizing, compiler errors, and exit cleanup. Follow the
[terminal procedure](../docs/TERMINAL_TESTS.md) using the smaller public example.
Buffer tests and exported PNGs do not establish terminal behavior.

## README screenshot and validation (2026-10-11)

The [README screenshot](../assets/demo/terminal.png) is an actual window capture
of the first guided-tour scene, not a composed mockup or a buffer rendering.
It shows the final demo changes on top of revision `89bf105` (uncommitted at
capture time), using Rust 1.93.0 and Windows Terminal 1.24.12741.0 on Windows.
The detected backend was **Sixel**, with reported **10×20px cells** and an
actual console viewport of **106×31 cells**. No multiplexer was used. The demo
was launched by PowerShell 7 with `-NoProfile`; the tool runner's inherited
`NO_COLOR` variable was removed for the capture, and `TERM` was set to
`xterm-256color` in that process only.

To reproduce the displayed scene, launch from the repository root in a graphics
terminal with at least 100×30 cells, using its normal color environment:

```sh
cargo run --locked -p ratatui-typst-demo
```

The capture confirms SIXEL startup, readable source and surrounding text, and
the Night palette's background continuity. Automated buffer tests cover all six
scenes at 76×24, 100×30, the layout boundaries, and larger/undersized viewports;
they also cover mode switching, diagnostic recovery, and image/text separation.
The public consumer's half-block PTY run exercised tiny/full fitting, image
replacement, compiler errors and recovery, movement, overlays, hiding, alpha
changes, and normal exit (exit code 0 with alternate-screen restoration).
The demo's half-block PTY run also exercised all six scenes, editing an invalid
expression into valid math, Custom equation, mode switching, controls, context,
reset, and normal exit. The explorer was separately inspected in a real SIXEL
window using the matrix example; its controls, source, note text, and image were
readable together.
Those PTY results do not establish SIXEL or Kitty cleanup. Real SIXEL resizing,
replacement, movement, errors, palette changes, and exit cleanup still require
the [full terminal procedure](../docs/TERMINAL_TESTS.md); Kitty remains unverified.

## Checks

Checks from the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
task check
task package
```
