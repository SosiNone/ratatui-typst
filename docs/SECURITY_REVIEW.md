# Locked dependency advisory review

Reviewed on **2026-10-18** with `cargo-audit 0.22.2` on Windows. The official
RustSec Windows release archive was verified against its published SHA-256.
This review is separate from the [license inventory](../licenses/REVIEW.md).

## Scope and database

- Entire workspace `Cargo.lock`: 447 packages, including the two local packages
  and all build, demo, and target-specific dependencies.
- Lockfile SHA-256:
  `67c85cc4f94b3bb39b44c673a5144d3cedce697de7e8aefcfae3f9702070353b`.
- Freshly fetched [RustSec database](https://github.com/RustSec/advisory-db/tree/999ec89d5d3f10f20be14750ec8c2d8b44e0ac9b):
  1,338 advisories at commit `999ec89d5d3f10f20be14750ec8c2d8b44e0ac9b`,
  updated `2026-10-17T17:23:59+02:00`.
- No advisory ignores, target filters, stale-database allowance, or disabled
  yanked-crate checks were used.

The audit reports **two vulnerability matches and six unmaintained warnings**.
No yanked-crate, unsoundness, or other informational warnings were reported.
The lockfile is unchanged: the mitigation below removes the exposed entry
points, rather than claiming that the transitive packages have been patched.
Consequently, the raw `cargo audit` command still exits with status 1.

## Vulnerabilities and mitigation

Both matches affect `quick-xml 0.38.4`, through
`typst-library 0.15.1 -> hayagriva 0.10.1 -> citationberg 0.7.0`.
The separate `quick-xml 0.42.0` dependency through `plist` is already patched.

| Advisory | Affected behavior | Disposition |
| --- | --- | --- |
| [RUSTSEC-2026-0194](https://rustsec.org/advisories/RUSTSEC-2026-0194.html) | Quadratic duplicate-attribute checking can exhaust CPU | Mitigated by removing user access to CSL style parsing |
| [RUSTSEC-2026-0195](https://rustsec.org/advisories/RUSTSEC-2026-0195.html) | Unbounded namespace declarations in `NsReader` can exhaust memory | Mitigated by removing user access to CSL style parsing |

RustSec requires `quick-xml >=0.41.0` for both fixes. The current published
`citationberg 0.7.0` requires `quick-xml ^0.38.1`, so a lockfile-only update
cannot select a fixed version. crates.io checks found no newer published
`citationberg`, `hayagriva`, or Typst release on 2026-10-18.

File-access restrictions alone do **not** mitigate these advisories. Typst's
[`cite` style parser](https://docs.rs/typst-library/0.15.1/src/typst_library/model/cite.rs.html)
and [`bibliography` style parser](https://docs.rs/typst-library/0.15.1/src/typst_library/model/bibliography.rs.html)
accept raw `bytes` as well as paths. Both route through `CslStyle::load` and
`CslStyle::from_data` to `citationberg::IndependentStyle::from_xml`, whose
quick-xml deserializer uses `NsReader`.

`MathRenderer` now omits `cite` and `bibliography` from the Typst global scope
and rebuilds the `std` binding from that restricted scope. Expressions cannot
call either function, including through `std` or `eval`. This is a deliberate
expression-compatibility restriction for the equation renderer; the public
Rust API is unchanged. The regression test
`rejects_citation_parsers_before_loading_inline_csl` checks these access paths
using small inline XML inputs.

Revisit both advisories when citationberg adopts a patched quick-xml release.
Do not restore citation functions until the complete locked graph is patched
and re-audited. General Typst evaluation still has no CPU or memory budget;
hostile expressions require a separate constrained process.

## Maintenance warnings

These are upstream maintenance notices, with no patched version listed in
RustSec. They are retained visibly and are not treated as vulnerability fixes.
Current Typst and syntect releases still require these dependencies.

| Dependency | Advisory | Dependency path / disposition |
| --- | --- | --- |
| `bincode 1.3.3` | [RUSTSEC-2025-0141](https://rustsec.org/advisories/RUSTSEC-2025-0141.html) | Typst's `syntect` syntax assets; replacement requires upstream serialization changes |
| `paste 1.0.15` | [RUSTSEC-2024-0436](https://rustsec.org/advisories/RUSTSEC-2024-0436.html) | `hayagriva` / `biblatex` compile-time macros; replacement requires upstream changes |
| `rustybuzz 0.20.1` | [RUSTSEC-2026-0206](https://rustsec.org/advisories/RUSTSEC-2026-0206.html) | Typst / `usvg` text shaping; follow upstream migration to a maintained shaper |
| `ttf-parser 0.25.1` | [RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192.html) | Typst / `fontdb` / `rustybuzz` / `pixglyph` / `usvg` font parsing; follow upstream migration |
| `unsafe-libyaml 0.2.11` | [RUSTSEC-2024-0449](https://rustsec.org/advisories/RUSTSEC-2024-0449.html) | Typst / `hayagriva` via `serde_yaml`; replacement requires upstream parser changes |
| `yaml-rust 0.4.5` | [RUSTSEC-2024-0320](https://rustsec.org/advisories/RUSTSEC-2024-0320.html) | `syntect` syntax definitions; replacement requires upstream changes |

Recheck these notices on dependency upgrades. The font and shaping libraries
remain part of rendering; disabling citation functions does not remove all
maintenance risk from the graph.

## Validation

Local checks used Rust/Cargo 1.93.0 on Windows, with cached dependencies,
incremental compilation disabled, and `TEMP`/`TMP` inside `target/review-tmp`.

| Check | Result |
| --- | --- |
| Citation regression before the mitigation (2026-10-16) | Failed as expected: inline CSL reached the parser and reported `failed to load CSL style` |
| `task check` after the mitigation | Passed: formatting, workspace Clippy with warnings denied, 24 tests, 1 doctest, consumer smoke, and docs with `RUSTDOCFLAGS=-D warnings` |
| `cargo package --locked --offline --allow-dirty -p ratatui-typst` | Passed: source archive built and extracted library compiled |
| Extracted crate without the workspace demo | Passed: 10 tests, 1 doctest, and public consumer smoke, including the citation regression |
| Final audit with a fresh database fetch | Same two mitigated vulnerability matches and six maintenance warnings; exit 1, no ignores |

The initial archive check used `--allow-dirty` during preparation. The fresh
audit retained exactly the two vulnerability matches and six maintenance warnings
documented here; no advisory ignores were added. The machine-readable report is
kept locally at `target/release-prep/audit/audit.json`. See the
[release checklist](RELEASE.md) for candidate CI and final archive verification.
No new real-terminal validation is claimed. Publishing remains disabled.

## Reproduce

Install cargo-audit separately from the crate, then run from the repository root:

```sh
cargo install cargo-audit --locked
cargo audit --file Cargo.lock
cargo tree --workspace --locked --target all --invert quick-xml@0.38.4
cargo test --locked --test rendering rejects_citation_parsers_before_loading_inline_csl
```

Run against a freshly fetched database before publishing and after dependency
changes. Review every reported advisory, including new matches and maintenance
warnings; the two documented mitigations do not justify ignoring future findings.
