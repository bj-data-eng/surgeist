# Surgeist Font

This notice covers the font-container and catalogue implementation and its
dependencies, including the Brotli allocator interface.
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

### Fontique 0.9.0

The registered catalogue uses Fontique's registration and font metadata
interfaces, distributed by the Parley Authors.

* License: [Apache-2.0](licenses/fontique/LICENSE-APACHE) or [MIT](licenses/fontique/LICENSE-MIT)
* Homepage: [Fontique](https://github.com/linebender/parley)

### linebender_resource_handle 0.1.1

The catalogue shares admitted resource bytes with Fontique through the
Linebender resource handle interface.

* License: [Apache-2.0](licenses/linebender-resource-handle/LICENSE-APACHE) or [MIT](licenses/linebender-resource-handle/LICENSE-MIT)
* Homepage: [Linebender resource handles](https://github.com/linebender/raw_resource_handle)

### miniz_oxide 0.8.9

WOFF1 uses the safe, fixed-output core decoder from miniz_oxide, distributed by
the authors named in its retained legal material.

* License alternatives: [MIT](licenses/miniz_oxide/LICENSE-MIT.md), [Zlib](licenses/miniz_oxide/LICENSE-ZLIB.md), or [Apache-2.0](licenses/miniz_oxide/LICENSE-APACHE.md)
* Additional attribution: [supplied license](licenses/miniz_oxide/LICENSE)
* Homepage: [miniz_oxide](https://github.com/Frommi/miniz_oxide)

### Parlance 0.1.0

The catalogue shares font attributes and variation settings with consumers
using Parlance, distributed by the Parley Authors.

* License: [Apache-2.0](licenses/parlance/LICENSE-APACHE) or [MIT](licenses/parlance/LICENSE-MIT)
* Homepage: [Parlance](https://github.com/linebender/parley)

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
