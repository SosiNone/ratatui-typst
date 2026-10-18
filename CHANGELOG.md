# Changelog

## 0.1.0 (2026-10-18)

Planned first release; publication is pending owner authorization.

- Typeset block math with embedded Typst 0.15.1 and bundled fonts.
- Configure size, padding, raster scale, foreground, background, and transparency.
- Retain source-relative diagnostics, warnings, pixels, and natural dimensions.
- Prepare fixed-size Ratatui equations using the application's shared image picker.
- Expose fitted terminal dimensions through `PreparedEquation::size()`.
- Bound natural rasters and whole-cell preparation canvases before allocation.
- Disable Typst's `cite` and `bibliography` functions to prevent access to
  transitive quick-xml denial-of-service vulnerabilities through inline CSL.
- Include a public consumer example and reproducible terminal lifecycle checks.
- Validate Linux rendering and terminal lifecycle on Ghostty 1.3.1 with Kitty
  graphics and the half-block fallback.
- License the project code under MIT; preserve separate bundled-font notices.

Ghostty/Kitty lifecycle results are recorded in the
[terminal report](docs/TERMINAL_TESTS.md#linux--ghostty-report-2026-10-16).
Windows rendering is reported good with Sixel confirmed; detailed lifecycle
behavior remains unverified. See [the release checklist](docs/RELEASE.md).
