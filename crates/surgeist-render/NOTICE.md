# surgeist-render

## Dependencies

This notice covers the Vello-derived source and Ahem font bundled in this
checkout, and the direct dependencies pinned in [Cargo.toml](Cargo.toml) for
`surgeist-render` 0.1.0. The inventory includes default builds, `render-window`,
`render-web`, native targets, and `wasm32-unknown-unknown` development dependencies.
Each entry identifies its role; a manifest dependency does not establish that
it is included in every distributed artifact. Transitive dependencies and
release-specific binary contents are outside this inventory.

The project's own [MIT license](LICENSE) is separate from the third-party terms
below. The linked license texts, [Vello adaptation record](NOTICE-VELLO.md), and
Ahem fixture notices accompany this notice in the source checkout. License
alternatives are retained without selecting one for downstream distributions.

### Ahem

This product's tests use Ahem, developed by Todd Fahrner and Myles C. Maxfield
and distributed by W3C:

* License: [Public domain with the upstream Creative Commons Zero fallback](tests/fixtures/fonts/ahem/COPYING)
* Homepage: [Ahem test font](https://www.w3.org/Style/CSS/Test/Fonts/Ahem/)

The bundled font is `tests/fixtures/fonts/ahem/Ahem.ttf`. Its
[upstream README](tests/fixtures/fonts/ahem/UPSTREAM-README) retains authorship
and usage information; [fixture provenance](tests/fixtures/fonts/ahem/PROVENANCE.md)
records the source files and their SHA-256 hashes. This is a test asset.

### bytemuck

This product depends on bytemuck 1.25.0, distributed by Lokathor:

* License: [Zlib](LICENSES/bytemuck/LICENSE-ZLIB) OR [Apache-2.0](LICENSES/bytemuck/LICENSE-APACHE) OR [MIT](LICENSES/bytemuck/LICENSE-MIT)
* Homepage: [bytemuck](https://github.com/Lokathor/bytemuck)

This is a direct library dependency with default features disabled.

### getrandom

This product's development dependencies include getrandom 0.3.4, distributed by
The Rand Project Developers:

* License: [MIT](LICENSES/getrandom/LICENSE-MIT) OR [Apache-2.0](LICENSES/getrandom/LICENSE-APACHE)
* Homepage: [getrandom](https://github.com/rust-random/getrandom)

The direct dependency applies only to `wasm32-unknown-unknown` development
builds, with default features disabled and `wasm_js` enabled. It is not a
direct production dependency of this crate.

### kurbo

This product depends on kurbo 0.13.1:

* License: [Apache-2.0](LICENSES/kurbo/LICENSE-APACHE) OR [MIT](LICENSES/kurbo/LICENSE-MIT)
* Homepage: [Kurbo](https://github.com/linebender/kurbo)

This is a direct library dependency. The included MIT license retains Raph
Levien's copyright notice.

### log

This product depends on log 0.4.33, distributed by The Rust Project Developers:

* License: [MIT](LICENSES/log/LICENSE-MIT) OR [Apache-2.0](LICENSES/log/LICENSE-APACHE)
* Homepage: [log](https://github.com/rust-lang/log)

This is a direct library dependency.

### peniko

This product depends on peniko 0.6.1:

* License: [Apache-2.0](LICENSES/peniko/LICENSE-APACHE) OR [MIT](LICENSES/peniko/LICENSE-MIT)
* Homepage: [Peniko](https://github.com/linebender/peniko)

This is a direct library dependency. The included MIT license retains Raph
Levien's copyright notice.

### png

This product depends on png 0.18.1, distributed by The image-rs Developers:

* License: [MIT](LICENSES/image-png/LICENSE-MIT) OR [Apache-2.0](LICENSES/image-png/LICENSE-APACHE)
* Homepage: [image-png](https://github.com/image-rs/image-png)

This is a direct library dependency. The included MIT license retains nwin's
copyright notice.

### pollster

This product's development dependencies include pollster 0.4.0, distributed by
Joshua Barretto:

* License: [Apache-2.0](LICENSES/pollster/LICENSE-APACHE) OR [MIT](LICENSES/pollster/LICENSE-MIT)
* Homepage: [Pollster](https://github.com/zesterer/pollster)

This is a development dependency used by native tests and the window smoke
example. The package declares the alternatives as `Apache-2.0/MIT`.

### proptest

This product's development dependencies include proptest 1.11.0, distributed by
Jason Lingle:

* License: [MIT](LICENSES/proptest/LICENSE-MIT) OR [Apache-2.0](LICENSES/proptest/LICENSE-APACHE)
* Homepage: [Proptest](https://proptest-rs.github.io/proptest/proptest/index.html)

This is a development dependency with default features disabled and `std`
enabled. The included MIT license retains FullContact, Inc's copyright notice.

### skrifa

This product depends on skrifa 0.42.1 from Fontations:

* License: [MIT](LICENSES/fontations/LICENSE-MIT) OR [Apache-2.0](LICENSES/fontations/LICENSE-APACHE)
* Homepage: [Fontations](https://github.com/googlefonts/fontations)

This is a direct library dependency with default features disabled and
`autohint_shaping` and `std` enabled. Both included license files retain Colin
Rothfels's copyright notice.

### surgeist-window

This product optionally depends on surgeist-window 0.1.0, distributed by
bj-data-eng:

* License: [MIT](LICENSES/surgeist-window/LICENSE)
* Homepage: [surgeist-window](https://github.com/bj-data-eng/surgeist-window)

The `render-window` feature enables this sibling workspace crate through
the local path declared in `Cargo.toml`. The included license preserves the
pre-import sibling source attribution; the sibling's own dependencies are outside this notice's
direct-dependency inventory.

### Vello

This product depends on vello_encoding 0.9.0 and vello_shaders 0.9.0 and includes
adaptations of Vello 0.9.0, distributed by the Vello Authors:

* License: [Apache-2.0](LICENSES/Vello-0.9.0-APACHE-2.0.txt) OR [MIT](LICENSES/Vello-0.9.0-MIT.txt)
* Homepage: [Vello](https://github.com/linebender/vello)

`vello_encoding` and `vello_shaders` are direct library dependencies. The shader
crate has default features disabled and `wgsl` enabled. Its WGSL source files
also offer the [Unlicense](LICENSES/vello/shader/UNLICENSE), as identified by
their `Apache-2.0 OR MIT OR Unlicense` headers; this additional alternative
applies to those shader files.

The private modules in `src/vello_engine/` contain the local adaptations.
[NOTICE-VELLO.md](NOTICE-VELLO.md) retains the pinned package checksum, imported
file hashes, material modifications, and omitted upstream sources. The shared
Apache-2.0 and MIT files also match both direct Vello dependency packages.

### wgpu

This product depends on wgpu 29.0.3, distributed by the gfx-rs developers:

* License: [MIT](LICENSES/wgpu/LICENSE.MIT) OR [Apache-2.0](LICENSES/wgpu/LICENSE.APACHE)
* Homepage: [wgpu](https://wgpu.rs/)

This is a direct library dependency. The crate's `render-web` feature enables
`wgpu/webgpu`; the dependency retains its default features.
