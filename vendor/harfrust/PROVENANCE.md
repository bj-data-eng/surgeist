# Maintained Harfrust dependency

This directory supplies the pinned Harfrust 0.6.2 dependency through the root
[`patch.crates-io`](../../Cargo.toml). It remains outside the product workspace.
Text owns its consuming adapter; this directory owns the provider implementation.

## Exact source

The source is the already cached crates.io `harfrust-0.6.2.crate`, SHA-256
`551ed25397e4b444e89686602877d5cf3a7f6e3d548dcac37a8357d1e195f4df`.
The package's [VCS metadata](.cargo_vcs_info.json) identifies
[`568dc6dfb778d306a92f0c52683cde5b16c9f3cf`](https://github.com/harfbuzz/harfrust/tree/568dc6dfb778d306a92f0c52683cde5b16c9f3cf/harfrust),
path `harfrust`. All 82 archive files were compared with the extracted cache before
copying. The 70 Rust source files, Ragel source files, Unicode companion data,
upstream README, license and VCS metadata retain their exact package bytes.

The original [workspace manifest](upstream/Cargo.toml.orig) and
[normalized registry manifest](upstream/Cargo.toml.registry) are preserved as
provenance. The active standalone manifest expands the package metadata, retains
the same production dependency versions, public features and lint tables, and omits
benchmark-only dependencies and targets. The root lockfile retains the exact
previously resolved production versions. Standalone feature qualification seeds
an ignored local lockfile from that root resolution and checks that its retained
package identities do not change.
The package's unrelated archived `src/hb/p` patch text, upstream lockfile and
Cargo's extraction sentinel are omitted.

The standalone manifest enables the existing `read-fonts/libm` feature: the
already resolved Read Fonts 0.39.2 requires it for `no_std` VARC math. This uses
the same already resolved `core_maths` and `libm` packages and introduces no
package or version change. Standard builds retain their native float operations.

## License and data notices

Harfrust 0.6.2 is [MIT-licensed](LICENSE), with the original HarfBuzz developers
and 2020 Yevhenii Reizner notices. Individual source copyright headers are retained.
The generated Unicode 17 tables and Unicode 16 `DoNotEmit.txt` retain their own
Unicode copyright and terms-of-use notices. The accompanying
[Unicode License v3](licenses/unicode/LICENSE) is copied unchanged from the
repository's existing legal material; keep it and those data notices together.

The project's own license remains separate. Redistribution must retain this
provenance, the upstream license, source notices and accompanying Unicode license.
