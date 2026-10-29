# Shaping coverage and qualification

This map describes the behavior exercised by the maintained tests and the
limits of that evidence. Always check the actual result's completion status;
plausible glyph output can accompany refusal. The [README](../README.md) explains
use and feature commands; [provenance](../PROVENANCE.md) records implementation
origins and the maintained extension contracts.

## Source and verification basis

The audited runtime basis is
[`d6e1fa3b4ef236017bb2363f87a63e2f292ea05e`](https://github.com/bj-data-eng/surgeist/commit/d6e1fa3b4ef236017bb2363f87a63e2f292ea05e).
It preserves the maintained provider's shaping algorithms, tests, fixtures and
generated inputs from `87b23145803f7630cc1ac658df1f1f4b2b8298b7`, with the package,
import and fixture paths migrated. The original Harfrust 0.6.2 import baseline is
`5b6bf65cc368662dbd46fd1297961c215443a6dd`.

The checks ran on the mechanical migration working tree subsequently committed
at that basis; the retained source correspondence permits reuse of those results.

The migration verification observed these results on 2026-10-28 using rustc
1.97.0 (`2d8144b7880597b6e6d3dfd63a9a9efae3f533d3`) and Cargo 1.97.0 on
`aarch64-apple-darwin`:

| Configuration and target | Observed result |
| --- | --- |
| `cargo test --offline --locked -j 1 -p surgeist-shaper` | 306 library tests passed; zero doctests. |
| `cargo check --offline --locked -j 1 -p surgeist-shaper --no-default-features` | Passed. |
| `cargo test --offline --locked -j 1 -p surgeist-shaper --no-default-features` | 306 library tests passed; zero doctests. |
| `cargo test --offline --locked -j 1 -p surgeist-text` | 1,426 tests passed across library, integration and doc targets, including 28 public DI integration tests. |
| `cargo test --offline --locked -j 1 -p surgeist-text --lib --features text-accessibility` | 1,168 library tests passed. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked -j 1 -p surgeist-shaper --no-deps` | Passed. |
| `cargo fmt -p surgeist-shaper -p surgeist-text -- --check` | Passed. |

The alloc-only tests execute with the host test harness. They qualify that
feature configuration, not a freestanding target, the declared MSRV or every
platform. Counts identify executed suites; they do not measure behavioral
coverage.

Subsequent strict Clippy checks passed for Shaper's default and alloc-only
configurations and Text's default and `text-accessibility` all-target
configurations. Those checks include the separately assessed
[`cargo_common_metadata` policy adjustment](../README.md#features-and-verification):
only that workspace-wide metadata lint is disabled for Shaper. Other inherited
settings and the unsafe prohibition remain unchanged. This manifest/documentation
adjustment does not alter the audited runtime basis.

The facade's `placed_text_render` and `placed_float_text_render` integration
targets passed all 38 tests with `render-text,text-accessibility`; strict facade
Clippy passed with that same feature set and all targets. Shaper, Text and root
public API freshness checks passed through the root generator. These affected
consumer checks qualify the exercised composition and paint/accessibility paths,
not every host adapter or feature combination.

## Inputs and inherited tests

The inherited tests in [common.rs](../src/hb/common.rs),
[tag.rs](../src/hb/tag.rs), [set_digest.rs](../src/hb/set_digest.rs) and
[ot_shape_plan.rs](../src/hb/ot_shape_plan.rs) qualify feature-string parsing,
script/language tags, set-filter helpers and `Send + Sync`. They do not constitute
a full-font shaping corpus. No upstream HarfBuzz shaping corpus or font
collection is retained with the imported package; historical upstream claims
are not local conformance evidence.

The maintained complete-font witnesses borrow the licensed
[Text Ahem fixture](../../surgeist-text/tests/fixtures/fonts/ahem/PROVENANCE.md).
[Completion](../src/hb/completion_fixture.rs),
[DI](../src/hb/default_ignorable_fixture.rs) and
[vertical](../src/hb/vertical_position_fixture.rs) fixtures construct explicit
SFNT/table operands in safe Rust over that resource. Tests assert authored font
facts, literal glyphs and independently derived arithmetic. Some tests compare
bound and ordinary shaping to establish preservation; that comparison alone is
not independent conformance proof. Private resource probes complement, rather
than replace, calls through real font shaping.

## What the suites establish

| Behavior | Exercising suites and limits |
| --- | --- |
| GSUB and contextual work | [DI tests](../src/hb/default_ignorable_tests.rs) exercise actual single, multiple, alternate, ligature, reverse and contextual substitutions, deletion and final descendant custody. [Lookup completion tests](../src/hb/ot_layout_gsubgpos_completion_tests.rs) include full-font recursion refusal and clear/reuse; matching/context boundaries also have private-owner probes. These are selected table cases, not every lookup format or script feature chain. |
| GPOS and attachment | [Vertical positioning](../src/hb/vertical_position_tests.rs) and [reference tests](../src/hb/vertical_reference_tests.rs) assert actual single/pair adjustments, mark-to-base, mark-to-mark and cursive propagation with exact integer/half-unit arithmetic. [Attachment completion](../src/hb/ot_layout_gpos_completion_tests.rs) directly tests the 64/65-link boundary. Mark-to-ligature component selection and general device/variation positioning breadth remain unqualified; the width observer below has narrow adjustment witnesses. |
| Requested width actions | [Core observation](../src/hb/width_observation_tests.rs), [independent operands](../src/hb/width_feature_qualification.rs), [lazy anchors](../src/hb/width_feature_lazy_qualification.rs), [native data and mask custody](../src/hb/width_feature_data_qualification.rs) and [effective scope](../src/hb/width_feature_scope_qualification.rs) qualify opt-in `hwid`/`twid`/`qwid` evidence. Authored fonts cover split/merge/deletion, source versus generated roots, identity/zero/absent actions, pair/mark/cursive operands, actual contextual provenance, normalized and later merged clusters, selected script/language and FeatureVariations, rejected lookup admission, demanded lazy/declared-data recovery and cache reuse, variation default/unused/applied-zero/malformed data, lifecycle and logical budgets. These prove action evidence for the executed scope, separately from numeric fit. Mark-to-ligature-specific footprints, exhaustive positioning/variation formats and allocator OOM remain outside this evidence. |
| Normalization and scripts | [Normalization completion](../src/hb/ot_shape_normalize_completion_tests.rs) asserts canonical ordering at 32 marks and truthful skipped-work refusal at 33. [Joining tests](../src/hb/original_joining_tests.rs) cover decomposition, reordering, Hangul composition, UVS and Mongolian actions. [Ordinary-symbol tests](../src/hb/ordinary_symbol_tests.rs) exercise all 62 eligible controls plus a mark under Arabic, Devanagari, Khmer, Myanmar, Balinese/USE and Hangul in both directions. These extension/progress witnesses do not qualify representative syllable shaping throughout those scripts. |
| Native AAT | DI full-font tests qualify selected morx contextual/noncontextual replacement, deletion, insertion and ligature operations. [Feature tests](../src/hb/ot_map_completion_tests.rs) assert real kern/kerx pair and cross-stream effects and selected-path distinctions. [morx](../src/hb/aat/morx_completion_tests.rs), [kerx](../src/hb/aat/kerx_completion_tests.rs) and [kern](../src/hb/kerning_completion_tests.rs) private probes qualify forced progress, rearrangement limits, component history, stack overflow, reset/pop and recovery. Successful ordinary trak interpolation and remaining AAT format/action breadth are not established. |
| Font and variation inputs | Complete-font tests cover static nominal/UVS mappings, real metrics and outlines, missing optional records, gid0, signed values and actual fallback lenders. Text observes default/nondefault variable coordinates and separately qualifies variable font metrics. Provider FeatureVariations output, named-instance/reset selection and varied glyph advances are not established by those tests. |
| Clusters, ordering and lifecycle | DI and joining tests assert original body ordinals independent of aliased clusters/context, selected LTR/RTL order, actual substitutions and clear/reuse. [Buffer completion](../src/hb/buffer_completion_tests.rs) qualifies pre-shape refusal surviving shape/serialization, first-refusal retention and glyph-flag limits. These do not exhaust every cluster level and flag combination. |
| Resource and feature limits | Buffer, DI, joining and lookup suites test exact work/growth/context boundaries, old-plus-new retained capacity, atomic admission, preflight, plausible partial output and denied receipts. [Feature completion](../src/hb/ot_map_completion_tests.rs) separates real selected mask consumers from absent, disabled, empty or unused paths. [Arabic context tests](../src/hb/ot_shaper_arabic_completion_tests.rs) distinguish transparent context exhaustion, terminators, unselected plans, overwrite and clear. The contracts describe logical limits, not allocator OOM or general font validation. |
| Joining and ordinary symbols | Joining tests cover literal full-domain actions, typed atomic source/correspondence admission, stale final input, normalization custody and reuse. Ordinary-symbol tests cover all 62 eligible nominal mappings, source/presentation mismatches, aliased clusters, work/growth refusal and rebinding/removal. Text owns delimiter, bidi and paragraph policy. |
| Default-ignorable custody | DI tests assert residual/Normal/deleted original inputs through GSUB, UVS, morx and final deletion, including skipped members, four-filler resources, remove/preserve precedence, empty output, first refusal and clear. Nominal ink, GDEF class and substituted bits alone do not authorize support. Property counts are not exhaustive independent Unicode conformance. |
| Supplied vertical metrics and references | Vertical positioning/reference suites qualify actual post-GSUB supplier queries, exact half-unit decoded positions, odd/even origins, anchor propagation, raw integer fallback quotients, absolute/relative references, staged overflow, original supplier errors, refusal precedence, empty results and invocation-local clear/reuse. [Fallback-origin tests](../src/hb/vertical_fallback_origin_tests.rs) assert actual ordinary/supplied TTB ink geometry. Only selected OT plans are supported; selected native AAT paths are explicitly rejected before supplier use. |
| Backward supplied vertical metrics | [Backward vertical tests](../src/hb/backward_vertical_tests.rs) exercise actual cmap mirroring and masked `rtlm` fallback, used-glyph metrics and half origins, original input ordinals, Normal/Residual/Deleted DI custody, required-feature provenance, GPOS attachment, explicit feature controls, context/supplier/refusal errors and clear/reuse. BTT retains negative Y advances; horizontal `rtla` remains independent. These are authored font witnesses, not exhaustive mirrored-character or script coverage. |

[Ancestry qualification](../src/hb/width_feature_ancestry_qualification.rs)
distinguishes genuine removal from malformed direct, contextual, cached and
earlier-rewrite dependencies, including rewritten contextual observers enabling
another root's removal and removals performed by non-width features.
Valid first-sibling removal and non-mutating misses preserve complete evidence.

## Consuming Text evidence

[Text engine tests](../../surgeist-text/src/engine/tests.rs) assert literal
ligatures, mark anchors, Arabic forms, retained font resources and an actual
shaping refusal preserving prior output and warm cache. Selected joining tests
exercise original source/replacement boundaries and actual instance edges.
[Control integration tests](../../surgeist-text/src/control/integration_tests.rs)
qualify symbol ink, source units and delimiter policy.

[Public DI integration tests](../../surgeist-text/tests/default_ignorable.rs)
exercise actual preparation, selection, placement and published glyph/allocation
output. They distinguish visible nominal-but-residual input, real native
replacement/deletion/joint output, UVS/fillers and morx normal descendants with
unpainted residual anchors in LTR/RTL and both positioning plans. This is
consumer evidence beyond provider receipts.

[Variable metric tests](../../surgeist-text/tests/variable_font_metrics.rs) qualify
Text's MVAR/instance metric boundary; they do not qualify provider feature
variation selection. Text's
[orientation tests](../../surgeist-text/src/protocol/tests/vertical_metrics_orientation.rs)
explicitly keep nominal vertical metrics separate from nonhorizontal paragraph
admission. Shaper's supplied vertical API does not itself qualify upright Text
paragraph composition.

## Remaining qualification limits

The audit's finite qualification gaps and separate investigation are tracked as
follows. Existing public Text DI receipt-consumption coverage needs no duplicate
follow-up.

| Follow-up | Disposition |
| --- | --- |
| [Representative complex-script syllables #1538](https://github.com/bj-data-eng/surgeist/issues/1538) | Add bounded Indic/Khmer/Myanmar/USE ordinary and broken-syllable witnesses with independent output expectations. |
| [Feature variations and reusable instances #1539](https://github.com/bj-data-eng/surgeist/issues/1539) | Qualify actual feature-dependent output for valid user/normalized coordinates and default/named-instance resets. |
| [Mark-to-ligature component positioning #1540](https://github.com/bj-data-eng/surgeist/issues/1540) | Qualify matching-component and last-component attachment with actual GSUB/GPOS and ordinary/supplied positions. |
| [Ordinary AAT tracking numerics #1541](https://github.com/bj-data-eng/surgeist/issues/1541) | Establish omitted/nonpositive point-size policy, then qualify interpolation, rounding and once-per-grapheme advances. |
| [Instance-reset selection investigation #1542](https://github.com/bj-data-eng/surgeist/issues/1542) | Unexecuted cached-selection hypothesis for invalid/absent reset inputs; establish the contract and full-font outcome before asserting a production defect. |

Generated machines and Unicode/tag/Arabic tables remain runtime coverage scope
even where their bytes match the original import. The current suites also do
not establish all OT/AAT formats, CFF/collection shaping, every malformed-font
path or universal HarfBuzz/browser equivalence. No line/branch coverage
percentage or fuzz-campaign qualification is claimed.
