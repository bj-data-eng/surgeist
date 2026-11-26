# surgeist-layout

## Dependencies

This notice covers third-party material incorporated into the `surgeist-layout`
0.1.0 source package and its direct development dependencies, as declared in
[Cargo.toml](Cargo.toml) and resolved in [the product Cargo.lock](../../Cargo.lock).
Surgeist's own license remains in [LICENSE](LICENSE).

The default library has no external Cargo dependencies. The development
dependencies below are part of the shared product resolution; their external
source is not vendored here. Their entries describe use and preserve license
alternatives.
This is not a transitive license inventory for a compiled test binary or
downstream application. The separate dependency inventory of
`tools/surgeist-layout-audits`, excluded from the package by `Cargo.toml`, is
outside this notice's scope. The browser-parity corpus, its generator and their
attributions belong to [surgeist-layout-parity](../surgeist-layout-parity/NOTICE.md).

### Ahem

This product includes the Ahem test font, created by Todd Fahrner and updated by
Paul Nelson, distributed through Kozea/Ahem:

* License: [Public domain / Creative Commons Zero declaration](licenses/ahem/COPYING)
* Homepage: [Ahem](https://github.com/Kozea/Ahem)

Layout tests read an unchanged W3C-distributed
[Ahem.ttf](tests/fixtures/fonts/ahem/Ahem.ttf) and its derivative
[PhraseAhem.ttf](tests/fixtures/fonts/phrase-ahem/PhraseAhem.ttf), both copied from
the Text fixtures; their provenance files record the hashes. The included
`COPYING` is the original declaration from
[Kozea/Ahem revision `7f4a69f546fd7bb7ba7031536d8470fbe87f4e26`](https://github.com/Kozea/Ahem/tree/7f4a69f546fd7bb7ba7031536d8470fbe87f4e26).
The upstream README credits Todd Fahrner and Myles C. Maxfield and is retained in
[licenses/ahem/UPSTREAM-README](licenses/ahem/UPSTREAM-README). The same
public-domain/CC0 declaration applies to these TTF bytes.

### Proptest

This product depends on Proptest 1.11.0, authored by Jason Lingle:

* License: [MIT](licenses/proptest/LICENSE-MIT) OR [Apache-2.0](licenses/proptest/LICENSE-APACHE)
* Homepage: [Proptest](https://proptest-rs.github.io/proptest/proptest/index.html)

The `proptest` crate is a development dependency for property tests, with default
features disabled and `std` enabled. Its license retains the copyright notice
for FullContact, Inc.

### Taffy

This product includes code and test material ported and adapted from Taffy
0.10.1, authored by Alice Cecile, Johnathan Kelley, Nico Burns, and the Taffy
contributors:

* License: [MIT](licenses/taffy/LICENSE.md)
* Homepage: [Taffy](https://github.com/DioxusLabs/taffy)

The layout implementation has since diverged substantially, with Surgeist's own
public data model, traversal contracts, algorithm phases, and test oracle.
Taffy is incorporated source, rather than a current Cargo dependency.

The included upstream license preserves Taffy's original-author notice and its
acknowledgment of the Stretch crate, copyright 2018 Visly Inc.

## Implementation references

The implementation has also been informed by CSS layout specifications and
browser behavior observed through local parity fixtures. WebKit and Blink
algorithms were studied for grid, subgrid, grid-lanes, inline layout, and baseline
behavior. This acknowledgment records those implementation references; the
incorporated code and fixture material is identified above.
