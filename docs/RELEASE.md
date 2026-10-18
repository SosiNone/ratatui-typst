# Release 0.1.0

Planned release date: **2026-10-18**. This is a source-crate release.
Repository: [SosiNone/ratatui-typst](https://github.com/SosiNone/ratatui-typst).
Publishing remains disabled in both manifests until the owner authorizes it.

## Candidate preparation (2026-10-18)

The public API, locked dependencies, and bundled fonts are unchanged. The
changelog is dated, the README includes the versioned dependency and an explicit
prepublication path alternative, and local package checks now exercise the
extracted crate's tests and public consumer. Documentation warnings fail both
Task and CI. The separate latest-dependencies CI job exercises a disposable
updated lockfile; the three locked jobs continue to check the reviewed graph.

| Local check on Windows, Rust/Cargo 1.93.0 | Result |
| --- | --- |
| `task check` | Passed: formatting, workspace Clippy with warnings denied, 24 tests, 1 doctest, consumer smoke, and docs with warnings denied |
| Locked offline archive verification during preparation | Passed with `--allow-dirty`; 27 files, approximately 414 KiB compressed |
| Extracted crate without the workspace demo | Passed: 10 tests, 1 doctest, and public consumer smoke |
| Dependency inventory | 445 third-party packages; bundled Typst notices match; 31 missing notice files remain a binary-distribution collection gap |
| Fresh RustSec audit | Same two mitigated quick-xml findings and six maintenance warnings; raw exit 1, no ignores |
| crates.io name lookup | HTTP 404 for `ratatui-typst` on 2026-10-18; this does not reserve the name |

The [security review](SECURITY_REVIEW.md) records the database revision,
lockfile hash, upstream versions, and mitigation. The
[license review](../licenses/REVIEW.md) records font terms and inventory scope.
Generated reports and audit tooling stay under ignored `target/`.
These reviews cover source distribution; no binary release is planned.

Local validation uses cached dependencies, `CARGO_INCREMENTAL=0`, and `TEMP`
and `TMP` set to `target/review-tmp`. These are local results, separate from CI.
No new real-terminal session is claimed for this documentation/tooling cleanup.

## Terminal compatibility

Ghostty 1.3.1 on Linux passed real Kitty graphics and forced half-block lifecycle
validation, including resizing, transparency, replacement, overlays, and
normal/Ctrl+C exit cleanup. See the dated
[report and captures](TERMINAL_TESTS.md#linux--ghostty-report-2026-10-16).

The maintainer reports good Windows rendering and confirms the Sixel backend.
Individual Windows lifecycle checks and exit cleanup remain unverified and are
optional follow-up for 0.1.0, per the maintainer's decision on 2026-10-16.
Other terminals and multiplexers have no compatibility guarantee.

## CI evidence

The [last pre-preparation CI run](https://github.com/SosiNone/ratatui-typst/actions/runs/38956620593)
passed Windows stable, Linux stable, and Linux Rust 1.92 on
`902daaec76cb2bc79a3cf3cd18544d9e7c047586`, including extracted crate checks.
It predates this candidate's release documentation and CI changes.

Use the [Actions page](https://github.com/SosiNone/ratatui-typst/actions/workflows/ci.yml)
to verify the exact commit selected for publication. All three locked matrix
jobs and the latest-dependencies job must pass. Historical runs and local
results do not substitute for that check.

## Reproduce candidate verification

From a clean checkout, using the committed Cargo.lock:

```sh
task check
task package
python tools/release_inventory.py --output target/release-licenses
cargo audit --file Cargo.lock
```

Without Task, run the checks directly:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo run --locked -p ratatui-typst --example terminal -- --smoke
cargo rustdoc --locked -p ratatui-typst -- -D warnings
cargo package --locked -p ratatui-typst --list
cargo package --locked -p ratatui-typst
cargo test --locked --manifest-path target/package/ratatui-typst-0.1.0/Cargo.toml --target-dir target
cargo run --locked --manifest-path target/package/ratatui-typst-0.1.0/Cargo.toml --target-dir target --example terminal -- --smoke
```

Install cargo-audit separately as described in the
[security review](SECURITY_REVIEW.md). Use a freshly fetched advisory database.
The raw audit currently exits 1 for the two documented mitigated matches; review
all findings against that record and investigate every new advisory.
Do not add blanket advisory ignores to make the command green.

Inspect the archive for sources, tests, example, license, font notices, and
release docs. It must exclude the workspace demo, tools, README showcase assets,
and generated/build artifacts. The terminal report's four supporting screenshots
are deliberately included. Cargo-generated metadata and the normalized manifest
are expected. Packaging compiles the extracted library without uploading it.

## Owner-authorized publication

Complete preparation and review before requesting final publication approval.
After explicit owner authorization:

1. Recheck the crate name on crates.io and confirm the intended publishing
   account has a verified email and an appropriately scoped credential. Keep
   credentials out of the repository, logs, and chat.
2. Remove the pending-publication sentences from README and CHANGELOG. Replace
   the README's closing `publish = false` paragraph with the release/version
   link, remove its prepublication path alternative, and set only the library's
   `publish = ["crates-io"]`. Keep `demo/Cargo.toml` unpublished.
3. Commit the final metadata changes and pass all CI jobs on that exact revision.
   Require a clean checkout, inspect `cargo package --list`, then run:

   ```sh
   cargo publish --dry-run --locked -p ratatui-typst
   cargo publish --locked -p ratatui-typst
   ```

4. After crates.io confirms publication, tag that exact revision `v0.1.0` and
   push the tag. Use the 0.1.0 changelog as the GitHub release notes. Do not attach
   executable downloads without a separate binary notice review.
5. Verify the [crates.io page](https://crates.io/crates/ratatui-typst), the
   [docs.rs build](https://docs.rs/crate/ratatui-typst/0.1.0), and installation
   from a consumer outside this workspace. Confirm the intended account owns
   the crate; additional owners require a deliberate owner decision.

`cargo package` provides the local packaging/build rehearsal while publishing
is disabled. `cargo publish --dry-run` remains the final rehearsal after
publishing is explicitly enabled. Follow Cargo's
[publishing guide](https://doc.rust-lang.org/cargo/reference/publishing.html).
