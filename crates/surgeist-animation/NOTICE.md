# surgeist-animation

The project's own license is [MIT](LICENSE).

## Dependencies

The [manifest](Cargo.toml) declares no runtime dependencies. Allocation
measurements use the development dependency **stats_alloc 0.1.10**, distributed
by Marcus Griep:

- License: MIT, as declared upstream; see the local
  [licensing record and missing-text limitation](licenses/stats_alloc/NOTICE.md).
- Homepage: [stats_alloc](https://github.com/neoeinstein/stats_alloc).

This dependency is used by development executables, not linked into the library.
The resolved package declares no dependencies of its own. No third-party source
or binary is vendored in this crate directory.

Coverage is limited to this crate's source distribution: its manifest,
Rust source, documentation, and legal files. It does not cover the Rust toolchain,
standard library, other workspace packages, downstream applications, or
binaries assembled by consumers.

See the [attribution update procedure](docs/how-to.md#refresh-attribution-after-changing-shipped-material)
when dependencies or distributed material change.
