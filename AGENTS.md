# Repository Guidelines

## Project

This is the development environment for the `ratatui-typst` crate. Follow open
source best practices and maintain professional grade software. Keep public API
changes deliberate and document compatibility limits honestly.

## Project Structure

The root Cargo package is the library and default workspace member. `src/renderer.rs`
embeds Typst; `src/widget.rs` prepares terminal images and draws the equation.
`tests/` holds integration tests, and `examples/terminal.rs` exercises the public
API. `demo/` is the unpublished interactive explorer. `docs/` contains architecture,
API review, terminal validation, and release instructions. `tools/` contains
release utilities; `licenses/` preserves third-party font notices.

## Build, Test, and Development

- `cargo build --locked`: build the library with the shared lockfile.
- `task check`: run formatting, workspace Clippy/tests, consumer smoke, and docs.
- `cargo test --workspace --locked`: test both library and demo, including doctests.
- `cargo run --locked -p ratatui-typst-demo`: explore rendering interactively.
- `cargo run --locked --example terminal -- --smoke`: check the public consumer headlessly.
- `task previews`: export sample PNGs to ignored `output/`.
- `task package`: verify the library archive without publishing.

## Coding Style and Testing

Use Rust 2024, four-space indentation, and rustfmt. Use `snake_case` for functions,
modules, and tests; `PascalCase` for types; and `SCREAMING_SNAKE_CASE` for constants.
Clippy must pass with warnings denied. Keep compilation and encoding outside
widget drawing. Name tests after the behavior they verify and use Rust's built-in
test framework. Add focused tests for behavior changes; no numerical coverage
threshold is enforced. Buffer tests cannot prove terminal graphics cleanup:
follow `docs/TERMINAL_TESTS.md` for visual changes.

## Commits and Pull Requests

Use short imperative commit subjects and focused changes; no established
historical convention exists yet. Explain the problem, resulting behavior, and
checks actually run. Link related issues when applicable. For visual changes,
include terminal/backend details and screenshots. Note API changes and preserve
license notices. Follow `docs/RELEASE.md` for release work; publishing is disabled
until the owner authorizes it.
