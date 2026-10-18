# License and Asset Review

Reviewed on 2026-10-18 against the committed Cargo.lock and cached registry
sources. Project code is [MIT licensed](../LICENSE), copyright 2026
ratatui-typst contributors. Font and dependency licenses remain separate.

## Embedded fonts

`typst_assets::fonts()` supplies 17 unmodified files. The complete upstream
[NOTICE](https://github.com/typst/typst-assets/blob/v0.15.1/NOTICE) identifies their terms:

| Embedded files | Count | Upstream terms |
| --- | --- | --- |
| `LibertinusSerif-*.otf` | 6 | SIL Open Font License 1.1 |
| `NewCMMath-*.otf`, NewCM10 bold/italic/bold-italic | 6 | GUST Font License 1.0 / LPPL 1.3c-or-later |
| `NewCM10-Regular.otf` | 1 | GPL-3.0-or-later plus Font Exception and Distribution Exception |
| `DejaVuSansMono*.ttf` | 4 | Bitstream Vera/DejaVu terms reproduced in NOTICE |

NewCM10-Regular needs particular attention: its distribution exception addresses
unmodified fonts in GPLv3-compatible programs; it is distinct from the exception
for documents. Do not assume the package's Apache-2.0 metadata covers its fonts.
Changing glyphs or glyph sets invalidates that distribution exception. This
project includes the upstream font bytes without such modifications.

Preserve the complete upstream license and notice alongside distributed
embedded fonts; retain reserved-name and attribution requirements. The current
font set is retained for reproducible output rather than modified for release.
Review any future font substitution or subsetting before distributing it.
The complete LPPL 1.3c text referenced by GUST is supplied separately in
`LICENSE-LPPL-1.3c.txt`, copied verbatim from the LaTeX Project's official source.

Both stored copies match the locked typst-assets 0.15.1 registry files byte for
byte (SHA-256):

- `typst-assets-LICENSE.txt`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`
- `typst-assets-NOTICE.txt`: `80ac46fb7d70f1c30bf0c14d3408e6a6f0f9cc9997a00b592aa0d2c8a5243a9d`

## Dependency evidence

Generate the inventory and candidate notice bundle with Python 3.11+:

```sh
python tools/release_inventory.py --output target/release-licenses
```

This uses offline, locked Cargo metadata and hashes detected license/notice
files. Fetch dependencies first if the registry cache is empty. The inventory
covers the entire workspace graph, including the unpublished demo, build, and
target-only dependencies; it is broader than the library's production graph.
Use the generated inventory for current package counts and missing notice texts.
The 2026-10-18 inventory contains 445 third-party packages and 31 missing notice
files; both bundled Typst notice copies still match. The lockfile SHA-256 is
`67c85cc4f94b3bb39b44c673a5144d3cedce697de7e8aefcfae3f9702070353b`.
A missing filename-scan result is a collection gap, not a determination that a
package is unlicensed. Adding the demo to the workspace changes the lockfile
and graph scope without changing the original font notice copies.

The first release is a source crate, not a standalone executable bundle.
Before distributing a compiled consumer, resolve missing notice texts, inspect
nested assets and the actual target/feature graph, and review the candidate
`THIRD_PARTY_NOTICES.txt`. It is not a complete binary redistribution clearance.
Re-run the review after dependency or feature changes.

Primary references: [MIT](https://opensource.org/license/mit),
[SIL OFL](https://openfontlicense.org/open-font-license-official-text/), and the
locked upstream Typst notice linked above.
