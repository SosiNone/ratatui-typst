# ratatui-typst

An experimental companion crate for typeset block equations in Ratatui.
Typst is embedded in Rust; `ratatui-image` handles terminal graphics. No Typst
executable, LaTeX installation, or PNG round trip is needed to display math.

This is an **unpublished 0.1.0 candidate** with a provisional API. The project and
crate name is `ratatui-typst`; Rust imports use `ratatui_typst`.
See [architecture](docs/ARCHITECTURE.md) and [CONTRIBUTING.md](CONTRIBUTING.md)
for the development workflow. Release status and remaining gates are tracked
in [the release checklist](docs/RELEASE.md). Repository: [SosiNone/ratatui-typst](https://github.com/SosiNone/ratatui-typst).

## Requirements and development

Requires Rust 1.92 or newer. The first build compiles Typst and can take several
minutes; embedded typesetting adds substantial build and binary size. Fonts are embedded through
`typst-assets`; installed fonts and the current working directory do not affect
font discovery.

From this repository root, run the development checks with
[Task](https://taskfile.dev/):

```sh
task check       # Formatting, Clippy, tests, and documentation
```

Or use Cargo directly, as detailed in [CONTRIBUTING.md](CONTRIBUTING.md):

```sh
cargo test --workspace --locked
cargo doc --locked -p ratatui-typst --no-deps
```

The root package is the library and default workspace member. The repository
also includes the interactive [demo](https://github.com/SosiNone/ratatui-typst/blob/main/demo/README.md), a small public API example,
tests, documentation, release tooling, and font notices. Both packages share
one lockfile. The demo is unpublished and excluded from the library archive.

Explore the crate with ordinary terminal text around each rendered equation:

```sh
cargo run --locked -p ratatui-typst-demo
cargo run --locked -p ratatui-typst-demo -- --halfblocks
cargo run --locked -p ratatui-typst-demo -- --png output/equation.png
```

Browse examples, press `e` to edit math, Tab to select rendering controls, and
`c` to switch Note / Console / Plain contexts. `s` exports a PNG; `q` quits.
Optional Task commands include `task demo`, `task previews`, and `task check`.
For the smaller terminal lifecycle example:

```sh
cargo run --locked --example terminal                 # Detect terminal graphics
cargo run --locked --example terminal -- --halfblocks  # Coarse fallback
cargo run --locked --example terminal -- --smoke       # Headless consumer check
```

The example uses only public APIs and exercises replacement, errors, movement,
overlays, hiding, shrinking, and transparency. See the [terminal test procedure](docs/TERMINAL_TESTS.md)
and [API review](docs/API_REVIEW.md).

Windows Terminal with SIXEL is the initial release's terminal validation target.
The maintainer reports good rendering on Windows with the Sixel backend;
detailed lifecycle results are still to be recorded. PowerShell is the shell;
graphics support comes from its terminal host. Kitty graphics are available
through ratatui-image but remain unverified for 0.1.0; validation is deferred
beyond the initial release. Other terminals can fall back to coarse half-block
characters.

## Library use

[src/lib.rs](src/lib.rs) is the public library entry point. Supply Typst math
without `$` delimiters. Until publication, use a local path dependency from
an application in a sibling directory:

```toml
[dependencies]
ratatui-typst = { path = "../ratatui-typst" }
ratatui = "0.30.2"
ratatui-image = { version = "11.1", default-features = false, features = ["crossterm"] }
```

```rust,no_run
use ratatui::layout::Size;
use ratatui_image::picker::Picker;
use ratatui_typst::{Equation, MathRenderer, RenderOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let renderer = MathRenderer::new();
    let equation = renderer.render("x = sqrt(2)", &RenderOptions::default())?;
    // Query once after terminal initialization, before reading events.
    let picker = Picker::from_query_stdio()?;
    let prepared = equation.prepare(&picker, Size::new(40, 10))?;
    // Inside terminal.draw: frame.render_widget(Equation::new(&prepared), area);
    Ok(())
}
```

`MathRenderer` reuses the standard library and shared bundled fonts. Rendering
returns pixels, natural size in points, the original source, and warnings.
Compilation errors include expression-relative byte ranges and line/column
locations where available. `RenderOptions` controls size, padding, resolution,
and foreground/background colors. Backgrounds are opaque by default. Set
`transparent_background: true` to omit the page fill while retaining
`background` as the matte color for SIXEL edges and half-block previews.
`required_cells` calculates natural
cell dimensions from the terminal's cell size in pixels.

`prepare` proportionally shrinks an image if needed and encodes it for the
application's shared `Picker`. Keep the result between draws and reprepare when
the panel or cell dimensions change. Zero-sized panels produce an empty widget.
`PreparedEquation::size()` reports the fitted dimensions in terminal cells for
placement and centering; zero-sized preparations report `0 x 0`.
The `Equation` draw path only displays the prepared image. Both rendering and
preparation are synchronous and can still delay input handling outside
`terminal.draw`. Applications can schedule them on their
own workers. A worker example and a bounded result cache are future work.

Transparent images retain full alpha with Kitty. Preparation first fits the
image onto a transparent canvas rounded to whole cells. SIXEL then blends
antialiased edge pixels against the configured background and keeps fully
transparent pixels undrawn; half-blocks receive an opaque composite. Match
`background` to the widget's panel color, including when using transparency.
PNG output from `image()` retains its original alpha.

## Current limits and release work

- Block math only; no inline paragraph layout, evaluation, or plotting engine.
- Rasterized text is not selectable or accessible as native terminal text;
  keep the source available to users.
- File/package resolution and the current date are unavailable. This does not
  bound Typst evaluation time or memory for hostile input.
- Raster output is capped at 16 megapixels and 16384 pixels per side. This
  check occurs after typesetting, so it is not a compiler resource limit.
  The same bounds apply to the whole-cell canvas allocated during preparation.
- Uses Typst's full bundled font set initially; custom fonts and a smaller
  default bundle are pending.
- Typst's process-wide memoization cache is aged after each compilation; there
  is no byte-bounded equation cache or asynchronous stale-result handling yet.
- The Windows Terminal lifecycle report is pending completion; Kitty validation
  is deferred beyond 0.1.0. Buffer tests cannot establish real graphics behavior.

Project code is licensed under [MIT](LICENSE), copyright 2026 ratatui-typst
contributors. Bundled dependencies and fonts retain their own licenses;
see the [license review](licenses/REVIEW.md) and [font notices](licenses/README.md).
`publish = false` remains set until the remaining
[release checks](docs/RELEASE.md) pass. No version has been published to crates.io.

References: [Typst math](https://typst.app/docs/reference/math/),
[Typst World interface](https://docs.rs/typst/0.15.1/typst/trait.World.html),
[ratatui-image](https://github.com/ratatui/ratatui-image).
