# Contributing

The project is an experimental companion crate. Read the
[architecture](docs/ARCHITECTURE.md) before extending scope and the
[release checklist](docs/RELEASE.md) before preparing a release.
The project and package name is `ratatui-typst`; the repository is
[SosiNone/ratatui-typst](https://github.com/SosiNone/ratatui-typst).
Project code is MIT licensed; keep third-party and font notices intact.

## Development

Install Rust 1.92 or newer and optionally Task. No Typst CLI is required.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo run --locked -p ratatui-typst --example terminal -- --smoke
cargo doc --locked -p ratatui-typst --no-deps
cargo package --locked -p ratatui-typst
```

`task check` runs formatting, Clippy, tests, the headless consumer, and docs;
`task package` verifies the crate archive, its tests, and its public consumer
without the workspace demo. Documentation warnings are errors in both Task and
CI. CI runs both sequences on stable Rust on
Windows/Linux and Rust 1.92 on Linux. A workflow file is not evidence that those
remote checks have passed; local verification is recorded separately.
Commit the single root Cargo.lock so local checks and CI use the same dependency
versions. The root library and unpublished `demo/` share a workspace, with the
library as its default member. Run `cargo run --locked -p ratatui-typst-demo`
or `task demo` for the explorer; `task previews` exports equations to ignored
`output/`. The public `examples/terminal.rs` consumer and published library
archive do not require the demo. Optional release inventory tooling requires
Python 3.11+; see the [license review](licenses/REVIEW.md).

Following Cargo's [CI guidance](https://doc.rust-lang.org/cargo/guide/continuous-integration.html#verifying-latest-dependencies),
a separate Linux stable job runs `cargo update` before workspace tests and the
public consumer smoke check. It tests the newest versions allowed by the
manifests, with `CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=allow` so the declared
minimum Rust version does not restrict dependency selection. This job changes
only its disposable checkout's lockfile; the locked Windows/Linux and Rust 1.92
jobs continue to validate the committed graph. When upgrading the committed
lockfile, review advisories and bundled font notices as described below.

Keep `renderer.rs` independent of terminal detection and event handling.
`widget.rs` adapts images to ratatui-image. The application owns its shared
Picker and chooses when to compile and prepare. Avoid doing expensive work in
the widget's draw path or implementing another graphics protocol.

Use small changes with focused tests: rendering, diagnostics, sizing, and pixel
conversion are good automated targets. Avoid treating exact raster snapshots
as stable across Typst/font upgrades. A test buffer cannot prove that graphical
images are erased or composited correctly by a terminal.

## Terminal reports

Record OS, Rust and application revision, terminal name/version, selected
backend, cell dimensions, and whether a multiplexer is in use. Include the
source expression and commands to reproduce the result. Check:

1. Initial display alongside borders and text.
2. Shrinking to a tiny panel and growing again.
3. Replacing the equation and displaying a compiler error.
4. Moving, covering, and removing the widget in a small reproduction app.
5. Exiting normally and verifying no graphics remain.

Use [docs/TERMINAL_TESTS.md](docs/TERMINAL_TESTS.md) for an exact key sequence and
report template and the completed Ghostty/Kitty Linux report. Windows rendering
has a positive maintainer report with Sixel confirmed; its individual lifecycle
checks remain unconfirmed. Keep limitations for other terminals and multiplexers
visible until verified. Screenshot comparisons are useful evidence, but record
the interaction steps too.

## Commits and pull requests

Use short imperative commit subjects and focused changes. Describe the problem,
resulting behavior, and checks actually run; link a related issue when applicable.
Use the pull request template and include screenshots plus terminal/backend
details for visual changes. Call out public API changes and dependency/font
notice updates. There is no numerical coverage requirement; test meaningful
behavior and failure paths rather than mirroring implementation details.

## Fonts and dependencies

Bundled fonts come from typst-assets 0.15.1. Preserve the upstream notices in
`licenses/` and update them when upgrading that dependency. Review all bundled
dependency notices before distributing a binary; the included font notices do
not constitute a complete third-party inventory for the executable.
