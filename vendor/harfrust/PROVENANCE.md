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
copying. Import baseline `5b6bf65cc368662dbd46fd1297961c215443a6dd`
retains all 70 Rust source files at their exact package bytes. The maintained
changes described below derive from that baseline. Ragel sources, generated Rust,
Unicode companion data, upstream README, license and VCS metadata remain unchanged.

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

## Completion readback

The handwritten `src/lib.rs` and `src/hb/{buffer,mod,face,ot_map,ot_layout_gsubgpos,
ot_layout_gpos_table,ot_shape_normalize,ot_shaper_arabic,kerning}.rs`, plus `src/hb/aat/{layout_morx_table,
layout_kerx_table}.rs`, contain the maintained completion-status extension and
owning test registrations. The adjacent `*completion_tests.rs` files and
`src/hb/completion_fixture.rs` are repository-authored tests. They borrow the
existing [Text Ahem fixture](../../crates/surgeist-text/tests/fixtures/fonts/ahem/PROVENANCE.md)
by a relative compile-time include; no new font copy or dependency is introduced.
Original source notices are retained.

Read `GlyphBuffer::shaping_status()` on the actual returned result before accepting
its glyphs. `Complete` means no observed resource guard limited required work in
that input/result lifecycle. `Refused` retains the first actual reason: buffer
growth, operation work, lookup nesting, normalization/matching/rearrangement/kerning context,
attachment nesting, or an overwritten required morx component-history read.
Nonempty and plausible glyph output can accompany refusal. Missing optional
tables, no-match and defined font recovery do not themselves imply refusal.

The private status begins at new/clear and survives enter, leave, serialization
and ordinary reads. This includes growth refusal during Unicode input assembly,
before shaping. Explicit `GlyphBuffer::clear` or `UnicodeBuffer::clear` resets the
status for the recycled input. Existing algorithm-control flags, budget thresholds,
forced-progress decisions and partial output behavior are preserved. Unused
capacity is not refusal; the morx ring reports actual stale logical reads rather
than merely capturing more than 64 components. The kern/kerx ninth-push reset
reports the actual discarded eight-entry stack; explicit reset/pop/reuse remains
ordinary behavior. A nonzero-combining-class run longer than 32 reports the
actual skipped canonical sort/custom mark reorder without changing that skip.
Explicit glyph-flag ranges above 255 report their actual skipped flag work;
whole-buffer requests and disabled optional concat requests retain their behavior.

`FeatureCapacityLimit` has a separate conservative contract: a selected nonempty
invocation's plan cannot represent a real feature request with the existing
per-feature bits, total mask space or final shared global bit. The observation
retains the existing mask, stage and override behavior. It can refuse even when
a range selects no current glyph or a later range overrides the value; it does
not claim an actual lookup was truncated. Only real selected OT feature sides
and actual native kern/kerx mask-consuming paths qualify. Disabled features,
absent optional features, empty invocations, unused GSUB under morx, no-op
fallbacks and mask-independent native paths do not establish this refusal.

Public pre/post-context storage still retains the nearest five characters. A
per-side truncation fact is observed only when a selected Arabic joining scan
exhausts transparent retained context and needs more. Post-context observation
is limited to states that can revise the preceding action/flags. The selected
pre-scan can report insufficient context even for transparent input; this does
not claim that final glyph output must differ. A retained nontransparent
terminator, an unselected shaper and empty shaping remain complete. Context
overwrite/clear and the existing public-add post-context reset dispose the
corresponding truncation fact; no context is enlarged and joining is unchanged.

The outcome describes logical implementation limits. It is not a general
physical-allocation failure protocol, a font-conformance validator, or proof that
a consuming layout/cache accepted the status. Current Parley consumers still
require an explicit later owned completion join. No shaping replay, output-size
heuristic, new provider trait, guard removal or generated-machine modification is
part of this extension.
