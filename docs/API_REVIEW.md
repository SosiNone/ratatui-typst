# Public API Review for 0.1.0

Reviewed on 2026-10-11 using `examples/terminal.rs`, which imports only the public
crate interface, Ratatui, and ratatui-image. No private modules, CLI compiler,
demo files, or additional dependencies are required.

## Consumer path

1. Construct a reusable `MathRenderer` and `RenderOptions::default()`.
2. Render math without dollar delimiters; inspect warnings or compilation errors.
3. Query one application-owned `Picker` after terminal initialization.
4. Prepare for the actual image viewport, outside `Terminal::draw`.
5. Use `PreparedEquation::size()` to place the fixed-size `Equation` widget.
6. Reprepare after viewport, cell-size, or backend changes. Recompile when source
   or rendering options change. Reuse the encoded image between ordinary draws.

## Changes justified by the consumer

- **Fitted dimensions:** added `PreparedEquation::size()`. Natural raster
  dimensions cannot accurately place a proportionally fitted, cell-padded image.
  Empty preparations return `0 x 0`.
- **Preparation allocation bounds:** reject oversized cell-padded canvases with
  `PrepareError::OutputTooLarge` before allocating pixels. Even a tiny equation
  could previously allocate an enormous canvas from a caller-supplied cell size.
- **Raster rounding:** conservatively check rounded-up raster dimensions against
  the pixel limits; Typst rounds to the nearest pixel when allocating its raster.
- **Units:** document points, pixels per point, sRGB colors, straight alpha,
  padding, and the effect of raster scale on natural terminal dimensions.

## Retained contracts and tradeoffs

- Errors expose structured diagnostics with original-expression byte ranges and
  one-based line/Unicode-scalar column positions. Warnings do not discard pixels.
- `image()` returns the unmodified natural raster; terminal preparation applies
  fitting and backend-specific compositing to a separate image.
- The application controls layout, capability queries, source accessibility,
  repaint scheduling, and terminal lifecycle. This crate does not implement a
  second graphics protocol or own an event loop.
- Synchronous compilation can block input. A worker, byte-bounded cache, custom
  fonts, inline math, and arbitrary document rendering are outside 0.1.0 scope.
- Typesetting has no CPU/time budget; file restrictions and pixel bounds are not
  an execution sandbox. Use a separate constrained process for hostile inputs.
- Public types reference `image`, Ratatui, and ratatui-image intentionally. Their
  major-version compatibility is part of dependency upgrade review.

## Evidence and remaining work

The headless consumer covers fitting, resizing, movement, replacement, errors,
hide/show, overlay removal/restoration, transparency, and zero-sized terminals.
Integration tests check fitted dimensions and excessive/invalid cell sizes.
These validate public usage and buffer composition; they do not prove real
graphics cleanup, visual quality, or hardware/terminal compatibility.

The [Ghostty/Kitty Linux report](TERMINAL_TESTS.md#linux--ghostty-report-2026-10-16)
also validates the consumer in a real graphics terminal, without API changes.
Retain the small API for 0.1.0. Run the terminal procedure before claiming support
for additional terminals; expand this interface only when a consumer demonstrates
a need.
