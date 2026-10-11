---
name: Bug report
about: Report rendering, diagnostics, or terminal lifecycle failures
title: ''
labels: bug
assignees: ''
---

## Environment

- Crate version / commit:
- OS and Rust version:
- Terminal host/version and shell:
- Selected image backend and cell dimensions:
- Multiplexer (or none):

## Reproduction

Include the Typst expression, rendering options, and a small reproduction or
the exact key sequence from `examples/terminal.rs`.

## Expected and actual behavior

For terminal graphics failures, include screenshots and note whether resizing,
replacement, transparency, or exiting leaves old pixels or damages nearby text.
