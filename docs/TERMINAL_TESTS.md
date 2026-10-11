# Terminal Lifecycle Validation

Run from the library repository root:

```sh
cargo run --locked --example terminal
```

For 0.1.0, use Windows Terminal to validate SIXEL. Kitty validation (for example,
in Ghostty) is deferred beyond the initial release and is not a release gate.
The example displays the selected backend and cell dimensions. If detection
chooses Halfblocks, record a fallback result; do not count it as a graphics pass.
The terminal's shell is not evidence of graphics support.

## Procedure

Check that adjacent terminal text and borders remain readable after every step.
Record failures with screenshots and the exact key sequence.

| Step | Action | Expected result |
| --- | --- | --- |
| 1 | Start normally | Centered equation; correct colors and no stray graphics |
| 2 | Resize the terminal smaller, then larger | Proportional fitting and correct placement; no old image remains |
| 3 | Press `s` twice | Fit into a tiny viewport, then restore full space |
| 4 | Press `n` | Replace root expression with a matrix; old pixels disappear |
| 5 | Press `n` | Compiler error replaces the image with source-relative diagnostics |
| 6 | Press `n` | Recover the original equation |
| 7 | Press `m` twice | Move the widget and restore it without ghost images |
| 8 | Press `o` twice | Remove image, show readable overlay, restore image |
| 9 | Press `h` twice | Remove and restore image without residue |
| 10 | Press `t`; repeat steps 2–9 | Transparent edges and padding blend correctly |
| 11 | Press `q` | Restore normal screen and cursor; no graphics remain |
| 12 | Relaunch; press Ctrl+C | Clean exit through the application's handler |

Repeat with `--halfblocks` as a separate fallback check. Test multiplexers
separately if you intend to claim support for them.

## Report template

- Date and Git commit:
- OS and Rust version:
- Terminal name/version and shell:
- Multiplexer (or none):
- Detected backend and cell dimensions:
- Opaque steps 1–12: PASS / FAIL / NOT RUN, with notes:
- Transparent steps 2–9: PASS / FAIL / NOT RUN, with notes:
- Exit and graphics cleanup:
- Screenshots and reproduction steps:

## Current evidence (2026-10-11)

- Automated half-block consumer: passed against Ratatui's `TestBackend`.
- Interactive PTY: half-block startup and keyboard lifecycle checks; results in
  [RELEASE.md](RELEASE.md). This is not Kitty/SIXEL validation.
- The maintainer reports that rendering looks good on Windows with PowerShell.
  The displayed backend was confirmed as **Sixel**. This is a positive manual
  SIXEL rendering result. Windows Terminal 1.24.12741.0 is installed; the tested
  terminal version, cell dimensions, and individual procedure results have not
  yet been recorded. Lifecycle coverage and exit cleanup remain unconfirmed.
- Real Kitty procedure is **not run**; deferred beyond 0.1.0.
- Linux and Rust 1.92 CI results are pending the initial repository push.
