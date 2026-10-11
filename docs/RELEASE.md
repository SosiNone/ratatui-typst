# First Public Release Preparation

Target: **0.1.0**, source crate, with Windows Terminal as the initial terminal
validation target. Status on **2026-10-11**: locally prepared; the maintainer
reports good Windows rendering and confirms the **Sixel** backend. Individual
lifecycle checks still need recording. Kitty validation is deferred and does
not block 0.1.0.
Repository: https://github.com/SosiNone/ratatui-typst.

## Completed preparation

- [x] Keep the library archive independently buildable without the workspace demo.
- [x] Exercise the public API in a small consumer ([terminal example](../examples/terminal.rs)).
- [x] Document API contracts and review findings in [API_REVIEW.md](API_REVIEW.md).
- [x] Add fitted-size measurement and reject excessive preparation canvases.
- [x] Adopt MIT for project code and include the complete license text.
- [x] Record exact bundled-font terms and verify notice copies in the [license review](../licenses/REVIEW.md).
- [x] Add a reproducible dependency inventory/notice collector.
- [x] Set repository URL, README, discovery metadata, and package file boundaries.
- [x] Add changelog, consumer CI check, and local crate archive verification.
- [x] Consolidate the library, demo, and tools into one workspace with a shared
  lockfile; remove obsolete experiments and refresh contributor documentation.
- [x] Query crates.io: `ratatui-typst` returned HTTP 404 on 2026-10-11; no existing
  crate was returned. This does not reserve or guarantee the name.

## Remaining release gates

- [ ] Push the initial reviewed commit and pass CI on Windows, Linux, and Rust 1.92.
- [ ] Finalize the Windows Terminal report using [TERMINAL_TESTS.md](TERMINAL_TESTS.md),
  recording completed lifecycle checks and exit cleanup. The maintainer has
  confirmed Sixel as the detected backend and reported good visual rendering.
- [ ] Confirm the desired crates.io name is still available and configure the
  publishing account/ownership immediately before publishing.
- [ ] Review this candidate and its archive after the initial Git commit.
- [ ] Set a release date in [CHANGELOG.md](../CHANGELOG.md) and explicitly enable publishing after
  those gates pass. `publish = false` is intentionally retained for preparation.

Kitty graphics remain available through ratatui-image but are unverified for
0.1.0; a real Kitty report is follow-up work. Other terminals and multiplexers
have no compatibility guarantee. A binary download would require an additional
dependency-notice review; none is planned here.

## Local validation

Host: Windows, Rust/Cargo 1.93.0. These are local results, not remote CI evidence.

| Check | Result |
| --- | --- |
| Rust formatting | Passed |
| Clippy, all targets, warnings denied | Passed |
| Library unit tests | 3 passed |
| Library integration tests | 6 passed |
| Rustdoc example | 1 compiled |
| Demo regression tests | 8 passed; formatting and Clippy passed |
| Headless public consumer | Passed lifecycle and zero-size sequences |
| API documentation build | Passed |
| Demo PNG exports | Four named preview tasks passed; exports are ignored |
| Workspace dependency inventory | 445 third-party packages; font notices match upstream; 31 missing notice texts remain a binary-distribution collection gap |
| Local crate packaging and extracted-source build | Passed, including the packaged consumer's headless check |
| Interactive PTY | Half-block startup, placement, overlay, replacement/error, hide/show, alpha and clean exit |
| Windows SIXEL rendering | Maintainer reports good visual results and confirms Sixel; individual lifecycle checks and exit cleanup unconfirmed |
| Real Kitty | Not run; deferred beyond 0.1.0 |
| Linux / Rust 1.92 | CI configured; not run locally |

Installed Windows Terminal version: 1.24.12741.0. Confirm the version used in the
manual report; the Sixel backend was confirmed separately by the maintainer.

## Reproduce before publishing

From the library root:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo run --locked -p ratatui-typst --example terminal -- --smoke
cargo doc --locked -p ratatui-typst --no-deps
cargo package --locked -p ratatui-typst --list
cargo package --locked -p ratatui-typst
python tools/release_inventory.py --output target/release-licenses
```

Inspect the archive for sources, tests, example, license, font notices, and
release docs. It must not contain the workspace demo, development tools, generated
images, or build artifacts. Packaging compiles the extracted library; it does
not upload it. After explicitly enabling publishing, `cargo publish --dry-run
--locked` is the final local rehearsal. Actual publishing and tagging are a
separate owner-authorized step.

Cargo's [publishing guide](https://doc.rust-lang.org/cargo/reference/publishing.html)
documents the archive inspection and dry-run process.
