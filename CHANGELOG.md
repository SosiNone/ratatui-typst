# Changelog

## 0.1.0 (unreleased)

- Typeset block math with embedded Typst 0.15.1 and bundled fonts.
- Configure size, padding, raster scale, foreground, background, and transparency.
- Retain source-relative diagnostics, warnings, pixels, and natural dimensions.
- Prepare fixed-size Ratatui equations using the application's shared image picker.
- Expose fitted terminal dimensions through `PreparedEquation::size()`.
- Bound natural rasters and whole-cell preparation canvases before allocation.
- Include a public consumer example and reproducible terminal lifecycle checks.
- License the project code under MIT; preserve separate bundled-font notices.

Initial terminal validation targets Windows Terminal with SIXEL. Windows
rendering is reported good with Sixel confirmed; detailed lifecycle results are
pending.
Kitty validation is deferred beyond 0.1.0. See [the release checklist](docs/RELEASE.md).
