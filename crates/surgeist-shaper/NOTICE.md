# surgeist-shaper

This notice covers the adapted shaping source and embedded Unicode data in
this crate. It does not enumerate every transitive compiled dependency.

## Dependencies

### Harfrust 0.6.2 adapted source

This product includes maintained Harfrust source, distributed by the HarfBuzz
developers and Yevhenii Reizner:

* License: [MIT](LICENSE)
* Homepage: [Harfrust](https://github.com/harfbuzz/harfrust)
* Exact source and maintained changes: [provenance](PROVENANCE.md)

Original source copyright headers, import identity and original package records
remain with the implementation. The Surgeist package name does not replace
those origins.

### Unicode data

The generated Unicode 17 tables and Unicode 16 `DoNotEmit.txt` retain their own
copyright and terms-of-use notices:

* License: [Unicode License v3](licenses/unicode/LICENSE)
* Homepage: [Unicode](https://www.unicode.org/)

### Borrowed Ahem test fixture

Handwritten provider tests compile the existing Text-owned Ahem font by relative
path. No font copy is bundled here; the owning [fixture provenance and
license](../surgeist-text/tests/fixtures/fonts/ahem/PROVENANCE.md) remain with it.
