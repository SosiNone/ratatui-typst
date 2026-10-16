# Architecture

`ratatui-typst` typesets block equations using embedded Typst and displays the
result through Ratatui and ratatui-image. It does not own an application's event
loop or implement another terminal graphics protocol. The 0.1.0 API is provisional;
see the [API review](API_REVIEW.md) for contracts and tradeoffs.

## Render, prepare, draw

```text
Typst math source + RenderOptions
  -> MathRenderer: compile and rasterize
  -> RenderedEquation: pixels, dimensions, source, warnings
  -> prepare(shared Picker, viewport size): fit and encode
  -> PreparedEquation: encoded image and fitted cell dimensions
  -> Equation: draw the prepared image into a Ratatui area
```

`src/renderer.rs` implements synchronous compilation and rasterization. A reusable
`MathRenderer` shares a lazily initialized bundled font set. Its restricted Typst
world provides the generated equation source and standard library; filesystem,
package resolution, system-font discovery, and the current date are unavailable.
The source is math without surrounding dollar delimiters, wrapped in an
automatically sized Typst page with configured text size, padding, and colors.

Diagnostics map wrapper locations back to expression-relative byte ranges and
one-based line/Unicode-scalar column positions where available. Compilation
failures and graphics preparation errors are separate types. Successful rendering
retains warnings and the original source. Typst's process-wide memoization cache
is aged after compilation; there is no byte-bounded result cache.

## Dimensions and pixels

Natural layout dimensions are points. Raster dimensions are pixels, controlled by
pixels per point. `required_cells` uses the terminal cell dimensions in pixels;
it does not perform typesetting reflow. `PreparedEquation::size()` reports the
fitted dimensions in terminal cells for placement and centering.

`src/widget.rs` proportionally shrinks images only when needed, pads them to whole
cells, and delegates encoding to the application's shared `Picker`. Filtered
downsampling uses premultiplied alpha internally to preserve glyph edges; exposed
image pixels use straight RGBA. Kitty retains alpha. SIXEL composites partial
alpha against the configured background and leaves fully transparent pixels
undrawn. Half-block previews use an opaque composite. Match the configured
background to the surrounding panel, including with transparent rendering.

Natural rasters and whole-cell preparation canvases are capped at 16 megapixels
and 16384 pixels per side. Zero-sized viewports produce an empty preparation.
These checks limit raster allocations after typesetting; they do not bound
compiler CPU or memory use. Hostile input needs a separately constrained process.

## Application responsibilities

Initialize the terminal and query capabilities once before reading events.
Compile and prepare outside `Terminal::draw`, retain prepared images between
frames, and reprepare after viewport, cell-size, or backend changes. Both steps
are synchronous and can delay input. Applications may schedule work on their own
workers; the crate does not require an async runtime or coordinate stale results.

The draw adapter performs no compilation, resizing, or encoding. Applications
own repainting, image removal, overlays, diagnostics presentation, and terminal
restoration. Rasterized math has no native text-selection or screen-reader
semantics, so keep the original source accessible.

## Repository and validation

The root library and unpublished `demo/` share a Cargo workspace and lockfile.
The demo provides editable examples, rendering controls, and surrounding native
terminal text. `examples/terminal.rs` is a smaller public API consumer for
headless lifecycle tests and manual terminal checks. The crate archive excludes
the demo and development tools.

Ghostty 1.3.1 on Linux has passed Kitty rendering and lifecycle validation.
The maintainer reports good Windows rendering with Sixel confirmed; detailed
Windows lifecycle checks remain unconfirmed. Follow the
[terminal procedure](TERMINAL_TESTS.md); buffer tests and PNG exports cannot
establish actual graphics cleanup or compatibility with multiplexers.

Fonts come from typst-assets 0.15.1; preserve the [font notices](../licenses/README.md).
Custom fonts, a smaller default font bundle, a bounded result cache, worker
examples, inline math, and arbitrary document rendering are future work.
Current release gates are maintained in [RELEASE.md](RELEASE.md).
