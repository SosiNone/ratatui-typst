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
```

Or run `task demo` from the repository root. `task quadratic`,
`task integral`, `task sum`, and `task matrix` open the explorer with that
example selected. `task previews` exports those four equations under
ignored `output/` without opening a terminal UI.

The executable is named `ratatui-typst-demo`. Use a terminal at least 76 columns
by 24 rows. The gallery covers fractions, roots, integrals, sums, matrices,
aligned equations, piecewise functions, vectors, font warnings, and compiler
errors. The source, image preview, natural dimensions, raster size, terminal
cell requirements, compile time, and diagnostics stay visible together.

The preview opens as a terminal note with a heading, an explanation, the
rendered equation, and follow-up text. This surrounding text uses ordinary
terminal characters and the same foreground/background palette as the image,
so you can compare font size, padding, and how the image blends into the panel.
Press `c` to cycle through Note, Console (prompt and output), and Plain views.
The equation uses its proportionally fitted height, with one blank row before
the following text when space allows. Surrounding text has reserved space before
oversized equations are fitted. The same rectangles drive preparation and drawing.

Demo defaults are transparent backgrounds and 2pt padding. Text size starts at
`1.2 * cell_height_pixels / pixels_per_pt`, clamped to 8–48pt: 12pt for a 20px
cell and the default 2 pixels per point. Reset restores this compact profile.
Headless PNG exports use a 20px reference cell height and retain transparency;
`s` exports the current settings. The library's opaque 20pt/10pt defaults are
unchanged. The current math font and mathematical italics are retained.

| Key | Action |
| --- | --- |
| Up / Down or j / k | Browse examples or select a rendering control |
| Tab / Shift+Tab | Switch between Examples and Rendering |
| Left / Right or - / + | Adjust the selected rendering control |
| n / p | Next / previous example from either panel |
| e | Edit the current source |
| c | Cycle Note / Console / Plain preview context |
| r | Reset rendering options, keeping the current source |
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

Checks from the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```
