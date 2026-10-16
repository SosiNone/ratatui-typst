# First Public Release Preparation

Target: **0.1.0**, source crate. Status on **2026-10-16**: Ghostty 1.3.1 on Linux
has passed real **Kitty** graphics and half-block lifecycle validation. The
maintainer reports good Windows rendering and confirms the **Sixel** backend;
individual Windows lifecycle checks remain unverified and are optional follow-up
for 0.1.0.
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
- [x] Push the reviewed [initial commit](https://github.com/SosiNone/ratatui-typst/commit/dfb78df0839ee2e646787f583eb42745628549ca) to `main`.
- [x] Review the candidate archive from the clean initial Git commit; verify
  the extracted library and its public consumer without the workspace demo.
- [x] Pass workspace CI on Windows stable, Linux stable, and Linux Rust 1.92.
- [x] Validate the public consumer and demo on Linux/Ghostty with Kitty graphics,
  half-block fallback, resizing, transparency, and normal/Ctrl+C exit cleanup;
  see the [Linux report](TERMINAL_TESTS.md#linux--ghostty-report-2026-10-16).
- [x] Query crates.io: `ratatui-typst` returned HTTP 404 on 2026-10-11; no existing
  crate was returned. This does not reserve or guarantee the name.

## Remaining release gates

- [ ] Commit the final candidate and pass CI on that exact revision (Windows
  stable, Linux stable, and Linux Rust 1.92), including the extracted crate's
  tests and public consumer. Historical CI results below cover earlier revisions.
- [ ] Confirm the desired crates.io name is still available and configure the
  publishing account/ownership immediately before publishing.
- [ ] Review the locked dependency graph against the current RustSec database
  (for example, `cargo audit --file Cargo.lock`) and resolve actionable findings.
  The license inventory is separate from an advisory check.
- [ ] Set a release date in [CHANGELOG.md](../CHANGELOG.md), update the README's
  candidate wording and local path dependency to the published version, and
  explicitly enable publishing after those gates pass. `publish = false` is
  intentionally retained for preparation; keep the demo unpublished.
- [ ] Inspect the final archive and run `cargo publish --dry-run --locked` after
  publishing is enabled. Publish only with the owner's authorization, then tag
  the published revision `v0.1.0` and verify the crates.io page and docs.rs build.

Per the maintainer's decision on 2026-10-16, the full Windows Terminal/SIXEL
lifecycle report is optional follow-up, not a 0.1.0 release gate. Basic rendering
has a positive report with Sixel confirmed; individual lifecycle steps and exit
cleanup remain unverified. Use [TERMINAL_TESTS.md](TERMINAL_TESTS.md) for any future
validation.

Kitty graphics are validated on Ghostty 1.3.1 on Linux. Other terminal
implementations and multiplexers have no compatibility guarantee.
A binary download would require an additional
dependency-notice review; none is planned here.

## Cleanup review (2026-10-16)

Reviewed the library's render/prepare/draw path, demo, public consumer, tests,
CI, package boundaries, and release/license documentation on Windows with
Rust/Cargo 1.93.0, starting from `a75ea95` with a clean working tree.

The cleanup replaces the demo's manual character wrapping with Ratatui's
existing word/grapheme wrapper and removes redundant picker background setup;
the library already handles canvas padding and compositing. A regression test
covers Unicode diagnostics and scroll clamping after resize/recovery. README
images and the excluded demo guide now use repository URLs. The public API,
dependency versions, and font assets are unchanged. CI now tests the extracted
crate and its public consumer without the demo's feature set.

| Check on the cleanup working tree | Result |
| --- | --- |
| `task check` | Passed: formatting, workspace Clippy, 23 tests, 1 doctest, consumer smoke, and docs |
| Rustdoc warnings denied | Passed with `RUSTDOCFLAGS=-D warnings` |
| Offline, locked package verification | Passed with `--allow-dirty` for this uncommitted review |
| Extracted crate tests and consumer | Passed: 9 tests, 1 doctest, and headless smoke in a separate target directory |
| Dependency license inventory | 445 third-party packages; bundled Typst notices match; 31 missing notice files remain a binary-distribution collection gap |

Rustdoc initially failed to write its executable in the sandbox's default
temporary directory. Re-running with `TEMP` and `TMP` set to the workspace's
`target/review-tmp` passed. Incremental compilation was disabled for these checks
to avoid sandbox hard-link warnings. Packaging was offline against cached
dependencies and produced a source archive of approximately 410 KiB compressed.

No new real-terminal graphics session, remote CI run, or Rust 1.92 run was
performed here. `cargo-audit` was not installed, and a fresh crates.io name
lookup was unavailable from this environment. Those release gates remain open.
A Windows real-terminal visual check of the diagnostic wrapping change is
optional follow-up using [TERMINAL_TESTS.md](TERMINAL_TESTS.md). Publishing remains
disabled.

## Local Windows validation (2026-10-11)

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
| Real Kitty | Not run in the original Windows session; later Linux/Ghostty results below |
| Linux / Rust 1.92 | Passed remote CI; not run locally |

The Kitty and Linux entries above describe the original Windows validation;
the later local Linux results are recorded below.

Installed Windows Terminal version: 1.24.12741.0. The tested terminal version was
not recorded; the Sixel backend was confirmed separately by the maintainer.

## Local Linux validation (2026-10-16)

Host: Omarchy 4.0.4, Rust/Cargo 1.98.1, Ghostty 1.3.1-arch2, Kitty backend,
19×42px cells, no multiplexer. The
[terminal report](TERMINAL_TESTS.md#linux--ghostty-report-2026-10-16) includes
the tested revision, interaction results, capture method, and screenshots.

Formatting, workspace Clippy with warnings denied, all 22 workspace tests,
the doctest, library/demo builds, the headless public consumer, and the library
documentation build passed. Task was unavailable, so its `check` commands were
run directly with Cargo. These are local results; no new remote CI run is claimed.
No library, dependency, or public API changes were needed for this validation.

## Remote CI evidence

The [initial commit CI run](https://github.com/SosiNone/ratatui-typst/actions/runs/38145063622)
checks the same library/demo sources on all three targets:

| Environment | Result |
| --- | --- |
| Windows, stable Rust | Passed |
| Linux, stable Rust | Passed |
| Linux, Rust 1.92.0 | Passed |

Each job checks formatting, workspace Clippy/tests, the public consumer smoke
sequence, demo PNG export, library documentation, and standalone crate packaging.
These headless checks do not establish real SIXEL lifecycle behavior.

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
cargo test --locked --manifest-path target/package/ratatui-typst-0.1.0/Cargo.toml --target-dir target
cargo run --locked --manifest-path target/package/ratatui-typst-0.1.0/Cargo.toml --target-dir target --example terminal -- --smoke
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
