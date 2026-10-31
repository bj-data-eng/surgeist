# surgeist-window

## Dependencies

This notice covers the nine direct production dependencies and the
`accesskit_consumer` development dependency pinned in [Cargo.toml](Cargo.toml),
including the optional `accessibility` feature and dependency material for all
declared platforms. The maintained consumer is bundled under
[vendor/accesskit-consumer](../../vendor/accesskit-consumer); the other direct
dependencies are not vendored here. This inventory does not cover other transitive dependencies or
material incorporated into a particular application binary. The project's own
license is in [LICENSE](LICENSE).

Keep this notice and its linked `licenses/` files together when distributing this
attribution material.

### accesskit 0.24.1

This product depends on accesskit, distributed by The AccessKit contributors:

* License: [MIT](licenses/accesskit/LICENSE-MIT) OR [Apache-2.0](licenses/accesskit/LICENSE-APACHE)
* Additional material: [Chromium BSD license](licenses/accesskit/LICENSE.chromium), [copyright and derivation notices](licenses/accesskit/NOTICE.md), and [upstream authors](licenses/accesskit/AUTHORS)
* Homepage: [AccessKit](https://github.com/AccessKit/accesskit)

### accesskit_consumer 0.36.0

The native accessibility platform adapters use the workspace-selected
maintained consumer. A direct development dependency tests original-unit word
navigation against that same consumer. Its source, legal material and local
correction are retained in the [maintained copy](../../vendor/accesskit-consumer/UPSTREAM.md).

* License: [MIT](licenses/accesskit/LICENSE-MIT) OR [Apache-2.0](licenses/accesskit/LICENSE-APACHE)
* Additional material: [copyright and derivation notices](licenses/accesskit/NOTICE.md), [upstream authors](licenses/accesskit/AUTHORS)
* Homepage: [AccessKit](https://github.com/AccessKit/accesskit)
* Legal source: [exact package revision](https://github.com/AccessKit/accesskit/tree/f40dfc01a0c0e76de535969f82fb35e19513737d)

### accesskit_winit 0.33.0

This product depends on accesskit_winit, distributed by The AccessKit contributors:

* License: [Apache-2.0](licenses/accesskit/LICENSE-APACHE)
* Additional material: [copyright notices](licenses/accesskit/NOTICE.md) and [upstream authors](licenses/accesskit/AUTHORS)
* Homepage: [AccessKit](https://github.com/AccessKit/accesskit)

### cursor-icon 1.2.0

This product depends on cursor-icon, distributed by Kirill Chibisov:

* License: [MIT](licenses/cursor-icon/LICENSE-MIT) OR [Apache-2.0](licenses/cursor-icon/LICENSE-APACHE) OR [Zlib](licenses/cursor-icon/LICENSE-ZLIB)
* Additional material: [W3C license and notices](licenses/cursor-icon/NOTICE.md)
* Homepage: [cursor-icon](https://github.com/rust-windowing/cursor-icon)

### keyboard-types 0.7.0

This product depends on keyboard-types, distributed by Pyfisch:

* License: [MIT](licenses/keyboard-types/LICENSE-MIT) OR [Apache-2.0](licenses/keyboard-types/LICENSE-APACHE)
* Additional material: [upstream W3C attribution](licenses/keyboard-types/NOTICE.md)
* Homepage: [keyboard-types](https://github.com/pyfisch/keyboard-types)

### objc2 0.6.4

This product depends on objc2, distributed by Mads Marquart. It is a macOS-only
direct dependency for native UI font and monitor observation.

* License: [MIT declaration and upstream terms](licenses/objc2/LICENSE.md)
* Homepage: [objc2](https://github.com/madsmtm/objc2)
* Legal source: [exact package revision](https://github.com/madsmtm/objc2/blob/8852b424193ca41602281b3d7540d7c8ed51e49a/LICENSE.md)

### objc2-app-kit 0.3.2

This product depends on objc2-app-kit. It is a macOS-only direct dependency for
native UI font, accessibility preference and monitor observation.

* License: [Zlib OR Apache-2.0 OR MIT declaration and upstream terms](licenses/objc2/LICENSE.md)
* Homepage: [objc2](https://github.com/madsmtm/objc2)
* Legal source: [exact package revision](https://github.com/madsmtm/objc2/blob/7b1abfd750a2cacaea71d6a56ecfb83cb7de560b/LICENSE.md)

### objc2-foundation 0.3.2

This product depends on objc2-foundation. It is a macOS-only direct dependency
for checked native display identifiers and Foundation values.

* License: [MIT declaration and upstream terms](licenses/objc2/LICENSE.md)
* Homepage: [objc2](https://github.com/madsmtm/objc2)
* Legal source: [exact package revision](https://github.com/madsmtm/objc2/blob/7b1abfd750a2cacaea71d6a56ecfb83cb7de560b/LICENSE.md)

The two upstream revisions carry byte-identical license declarations, retained
unchanged in the shared local copy. The declaration links the upstream license
terms and separately discusses Apple SDK provenance. The upstream packages and
repository revisions provide no separate package copyright text.

### raw-window-handle 0.6.2

This product depends on raw-window-handle, distributed by Osspial:

* License: [MIT](licenses/raw-window-handle/LICENSE-MIT.md) OR [Apache-2.0](licenses/raw-window-handle/LICENSE-APACHE.md) OR [Zlib](licenses/raw-window-handle/LICENSE-ZLIB.md)
* Homepage: [raw-window-handle](https://github.com/rust-windowing/raw-window-handle)

### winit 0.30.13

This product depends on winit, distributed by The winit contributors and Pierre
Krieger:

* License: [Apache-2.0](licenses/winit/LICENSE)
* Additional material: [W3C license and source attribution notices](licenses/winit/NOTICE.md)
* Homepage: [winit](https://github.com/rust-windowing/winit)
