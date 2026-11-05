# Surgeist Font

This notice covers the font-container implementation and its two direct
decompression dependencies, including the Brotli allocator-interface dependency.
It does not inventory a separately linked application's other dependencies.

## Dependencies

### alloc-no-stdlib 2.0.4

The Brotli dependency uses allocation interfaces distributed by Dropbox, Inc.
Surgeist supplies its own bounded storage through these safe interfaces; it uses
the normal Rust standard library and does not replace the global allocator.

* License: [BSD-3-Clause](licenses/alloc-no-stdlib/LICENSE)
* Homepage: [rust-alloc-no-stdlib](https://github.com/dropbox/rust-alloc-no-stdlib)

### brotli-decompressor 5.0.0

This product depends on the Brotli decoder distributed by Dropbox, Inc. Its
manifest declares BSD-3-Clause/MIT; the retained package's supplied license
contains the BSD-3-Clause terms reproduced here.

* License: [supplied BSD-3-Clause license](licenses/brotli-decompressor/LICENSE)
* Homepage: [rust-brotli-decompressor](https://github.com/dropbox/rust-brotli-decompressor)

### miniz_oxide 0.8.9

WOFF1 uses the safe, fixed-output core decoder from miniz_oxide, distributed by
the authors named in its retained legal material.

* License alternatives: [MIT](licenses/miniz_oxide/LICENSE-MIT.md), [Zlib](licenses/miniz_oxide/LICENSE-ZLIB.md), or [Apache-2.0](licenses/miniz_oxide/LICENSE-APACHE.md)
* Additional attribution: [supplied license](licenses/miniz_oxide/LICENSE)
* Homepage: [miniz_oxide](https://github.com/Frommi/miniz_oxide)

### Wuff 0.2.9 adapted decoder algorithms

The WOFF2 reconstruction adapts Wuff's library algorithms, distributed by the
WOFF2 Authors and Nico Burns. Copyright 2013–2017 the WOFF2 Authors;
copyright 2025 Nico Burns. Its command-line interfaces and bundled native
implementation are not included.

* License: [MIT](licenses/wuff/LICENSE)
* Homepage: [Wuff](https://github.com/nicoburns/wuff)
* Exact source and modifications: [decoder provenance](PROVENANCE.md)

The repository's shared [Ahem notice](../../NOTICE.md#ahem) covers the retained
test font. Its original WOFF2 and the later TTF are distinct resources.
The independently sourced CFF and variable-color data retain their licenses
with the [font fixture provenance](tests/fixtures/PROVENANCE.md).
