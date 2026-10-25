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
changes described below derive from that baseline. Equivalent `Option` early
returns are also simplified in handwritten `src/hb/{glyph_metrics,unicode}.rs`.
Ragel sources, generated Rust, Unicode companion data, upstream README, license
and VCS metadata remain unchanged.

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
corresponding truncation fact. Ordinary requests retain their context capacity
and joining behavior.

The outcome describes logical implementation limits. It is not a general
physical-allocation failure protocol, a font-conformance validator, or proof that
a consuming layout/cache accepted the status. Text's owned adapter checks the
actual returned status before accepting shaped output. No shaping replay,
output-size heuristic, new provider trait, guard removal or generated-machine
modification is part of this extension.

## Joining domains and selected presentation

The maintained provider also separates Unicode joining analysis from emitted
presentation. The repository-authored `src/hb/original_joining.rs` owns this
operation, with qualification in `src/hb/original_joining_tests.rs` and the Text
tests. `JoiningSource::analyze` retains a complete Unicode domain for an
explicit script and direction, independently of fonts. Text supplies the authentic
unbroken source and the actual selected-line presentation as separate domains,
following the [Text/Layout contract](../../docs/text-layout-contract.md).
`UnicodeBuffer::set_joining_sources` validates both scalar correspondences against
the actual emitted input without replacing a prior binding on admission failure.
Shaping revalidates the final input and profile; clearing disposes the binding.
An invalidated binding reports `JoiningInput` before shaping can accept it.
Original contributions receive source-domain actions; computed contributions
receive selected-presentation actions. Font and metric
runs retain that Unicode context while actual substitution and positioning stay
within otherwise-compatible shaping runs.

Independent contributor lineage survives normalization. Before applying actions,
the provider proves unique, compatible joining-bearing correspondence; ambiguous
composition, class-changing normalization or unsupported selector contraction
returns `JoiningCorrespondence`. At shaping, incompatible selected profiles or
requested positive tatweel-safety flags return `JoiningProfile`. Admitted requests
conservatively mark the emitted buffer unsafe to split or concatenate. These
bounds do not promise all-Unicode normalization support or identical glyph IDs
under legitimate contextual substitutions and ligatures.

The Unicode analyses perform only required action work, without the ordinary
five-character external-context scan or discarded flag-range work. Ordinary
context, normalization, substitution and positioning guards retain their own
completion reporting. Checked aggregate allowances cover the retained domains,
input mappings, lineage, proof temporaries and their overlapping allocations and
work. Borrowed correspondence spans contribute their logical sizes; spare
caller-owned capacity and allocator overhead are outside this provider-owned
storage guarantee. At introduction, the independent lineage field grew both
`GlyphInfo` and `GlyphPosition` from 20 to 24 bytes, including ordinary buffers.
The DI extension below adds another independent word; their equal layouts
preserve the existing safe buffer casts.

## Closed ordinary-symbol presentation

The repository-authored `src/hb/ordinary_symbol.rs` owns an explicit native
presentation for Cc other than TAB, LF and CR. Its public
`UnicodeBuffer::set_ordinary_symbol_positions` accepts strictly increasing
scalar indices in the complete actual Unicode input, independently of cluster
values. Admission is atomic and returns typed index/order/character or work/growth
failure without replacing the previous metadata. An empty list removes it.
Appending after a nonempty binding refuses the actual shaping invocation as
`OrdinarySymbolInput`; rebinding the final input is permitted. Both buffer-clear
front doors dispose the binding. Text retains document-defined delimiter
eligibility and its bidi/script/segmentation policy.

The closed provider presentation is category So, zero combining class and
nonjoining, without control, mark, whitespace or default-ignorable treatment.
Actual Cc code points remain the nominal cmap input. Unicode initialization,
normalization, Arabic joining and Indic/Khmer/Myanmar/USE property setup consume
the selected presentation. Other Text-level profile properties remain Text's
analysis responsibility. This does not introduce a Unicode-database override,
font substitution or default-ignorable suppression operation.

The high bit of the existing independent scalar-lineage word carries the
presentation through real scalar normalization; low 31 bits carry lineage-node
identity. Accessors mask and preserve the independent fields. Node issuance
refuses `GrowthLimit` before a lineage ID exceeds its representable low-bit
range. This Cc presentation adds no row storage beyond the joining-era 24-byte
rows; the independent DI extension below documents the current 28-byte rows
and unchanged allowance. Cc does not decompose or compose into another scalar under the closed
profile; ordinary neighboring Unicode normalization and its lineage proof remain
active. The normalized correspondence proof checks presentation as well as
joining class and code point.

`JoiningSource::analyze` now accepts the same scalar-indexed presentation input;
it validates it before publishing a source. Source/body mismatches refuse typed
`SourcePresentation` admission, and final-input revalidation preserves the
existing `JoiningInput` shaping fence. No compatibility wrapper is retained.
`ordinary_symbol_tests.rs` qualifies all 62 eligible Cc nominal mappings, atomic
failure, stale input and clear/reuse, normalization with aliased clusters,
source/profile mismatch, exact admission-work and growth refusal, final-input
rebinding/removal, and all 62 controls followed by a real combining mark under
Arabic, Devanagari, Khmer, Myanmar, Balinese and Hangul plans in both directions.
The joining tests also prove that each control interrupts full-domain Arabic
joining. The only raw external-context property consumer is the Arabic joining
scan; all 62 controls already have its nonjoining class, and actual pre/post
scans with neighboring BEH are qualified in both directions. No separate
external-context presentation representation is needed for this closed inventory.
Existing completion and joining tests remain in force. No generated Unicode or
machine table, dependency version, downloaded source or font fixture was changed.


## Original default-ignorable custody

The maintained wiring is in `src/lib.rs`, `src/hb/{buffer,face,mod,
original_joining,ot_layout_gsubgpos,ot_shape,ot_shape_normalize,
ot_shaper_arabic,ot_shaper_hangul,ot_shaper_syllabic,ot_shaper_thai}.rs`,
`src/hb/ot/gsub/multiple.rs`, and
`src/hb/aat/{layout_common,layout_morx_table}.rs`. The existing joining budget
tests distinguish the frozen allowance from the actual enlarged rows.

The repository-authored `src/hb/default_ignorable.rs` owns a narrow receipt for
original Unicode-17 Default_Ignorable_Code_Point inputs. Its immutable range
fact is independent from the retained lookup-skipping predicate: the full fact
includes U180F, the four Hangul fillers and U1BCA0..3 without removing their
native matching opportunities. The property is the retained Unicode-17
DerivedCoreProperties repertoire (4174 members); no Unicode edition, generated
Unicode data, dependency or archive source was upgraded.

`GlyphBuffer::default_ignorable_receipt()` admits three borrowed immutable
slices only for a complete result: residual and deleted original body scalar
ordinals, and final residual glyph-array positions. Context, UTF16, caller
clusters and joining-domain ordinals do not identify these inputs. The owner
snapshots original DI candidates before shaping and follows their sparse sets
through real normalization, UVS, GSUB, AAT insertion/ligatures/deletion and native
postprocessing. A genuine nondeleted replacement, consumed sequence or qualified
nonzero four-filler resource establishes Normal output. Nominal DI coverage,
cluster merging, GDEF class, private substituted bits, synthetic repair, ink and
advance alone do not. Sets remain until finalization, so later deletion of an
earlier replacement is reflected and partial surviving descendants remain Normal.

Residual output suppression is separate from terminal input classification:
AAT insertion can give one DI input a Normal inserted descendant while its
unchanged unresolved anchor remains a residual glyph slot. Default presentation
retains zero-advance residual slots, replacing their glyph with the ordinary
invisible/SPACE placeholder when available; consumers must omit the receipt's
residual slots from paint. REMOVE removes those carriers and PRESERVE keeps the
selected diagnostic glyph behavior; neither flag establishes native support.
The consuming Text adapter owns zero independent allocation and paint admission.

The independent DI carrier word increases each interchangeable GlyphInfo and
GlyphPosition row from 24 to 28 bytes. Their POD layouts remain equal and the
joining lineage word's Cc high bit remains independent. Ordinary buffers also
carry this row overhead. Actual 56-byte row-pair storage, DI sets/members,
terminal slices, joining retained data, temporaries and overlapping old/new
capacities share the unchanged logical allowance of 48 bytes per permitted row
pair and the existing operation budget. No limit is enlarged. Checked logical
failures latch GrowthLimit or WorkLimit and refuse receipt admission; physical
allocator failure keeps the existing provider semantics. Finalization precedes
leave's budget reset and covers empty/all-deleted output; both clear paths
remove identity and receipt state.

Carrier copies and fallback/final scans charge the shared work allowance.
Owning expansion operations preflight their output; native input traversal
still makes bounded progress after a latched refusal, including generated
Unicode repair loops. Handwritten preparation loops stop on failure, and the
Thai two-output expansion admits both outputs before indexing them. These
refusal paths neither publish an admitted receipt nor alter generated sources.

Buffer-mode switches debit the exact existing output prefix copied into the
alternate row buffer. Forward and rewound moves debit their exact copied
ranges; a rewind needing input-tail relocation admits the tail and rewind
together. Allocation-free cursor/work preflight covers complete known output,
insertion and ligature-store sequences before changing carrier/output state,
including morx marked-anchor rewind and return. Existing native insertion and
input-tail charges remain single debits, with no separate allowance. Hangul
decomposition stops immediately after a refused replacement or following input
copy, before indexing its expected Jamo output rows. Complete-font controls and
existing-capacity resource tests qualify these paths and GDEF-class-only DI
inputs independently of native substitution support.

The repository-owned `default_ignorable_tests.rs` and
`default_ignorable_fixture.rs` supply real complete-font/table behavioral
qualification, using the retained licensed Ahem source through the existing
relative include. They author bounded cmap/hmtx/GSUB/morx tables in safe Rust;
private action proxies do not replace the complete Shaper witnesses. No new
font copy or package is introduced.
