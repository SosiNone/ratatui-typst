# Bundled font notices

The renderer embeds the fonts supplied by the `fonts` feature of typst-assets
0.15.1. No font binaries are copied into this repository.

`typst-assets-NOTICE.txt` and `typst-assets-LICENSE.txt` are unmodified copies of
that published crate's NOTICE and LICENSE, obtained from the Cargo registry.
The notice identifies individual font families and their terms. Preserve these
notices when distributing embedded fonts. They do not license this project's
code or cover every dependency in a compiled executable.

`LICENSE-LPPL-1.3c.txt` is a verbatim copy of the
[official LPPL 1.3c text](https://www.latex-project.org/lppl/lppl-1-3c.txt),
referenced by the GUST font terms in the upstream notice. Keep it alongside
the notice. See [REVIEW.md](REVIEW.md) for the exact embedded font families,
the NewCM10-Regular exceptions, and dependency inventory instructions.

Upstream: https://github.com/typst/typst-assets

Crate: https://crates.io/crates/typst-assets/0.15.1
