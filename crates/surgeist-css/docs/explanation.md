# Explanation

## Authored syntax and browser recovery

`surgeist-css` owns authored CSS syntax and its diagnostics. Root-owned Surgeist
adapters lower parsed syntax into typed style data. The parser preserves the
information downstream consumers need without performing their processing.

The ordinary `parse_sheet` and `parse_style_attribute` entry points use browser recovery: each returns a report containing valid retained syntax and every structured recovery diagnostic in source order. Unsupported or malformed source units are dropped, replaced, retained with an implicit closure, ignored, or stopped at the documented boundary; valid siblings remain eligible. A clean report means that no recovery diagnostic was produced. An empty retained tree alone does not establish that the source was clean, so consumers should inspect `is_clean()` or `diagnostics()` rather than infer validity from syntax length.

The [report API](../src/report.rs) keeps syntax and diagnostics together.
Application-strict validation is a decision about accepting the same report,
not a second grammar. It returns retained syntax only for a clean report and
otherwise returns every diagnostic in unchanged order.

## Symbolic values and compatibility

CSS custom properties preserve case-sensitive names and authored value text, including interior trivia. Known-property values whose grammar depends on `var(...)` remain substitution-dependent authored values. The crate recognizes terminal `!important`; style owns custom-property environments, substitution and cascade.

Variable references and their fallback token text remain symbolic.

For the implemented [declaration expansion slice](reference.md#intrinsic-declaration-expansion),
CSS rechecks a caller-supplied replacement against the original property grammar
and produces complete contributions atomically. This intrinsic grammar reentry
retains token origins and the original declaration occurrence. It does not find
variables or decide invalid-at-computed-value behavior. The immutable
[stylesheet normalizer](reference.md#immutable-stylesheet-normalization) preserves
ordered rule contexts and grouped declaration contributions through that same
expansion boundary. It traverses every retained rule family, but returns an
atomic typed failure for emitted declarations outside the implemented expansion
coverage. Expansion of the remaining properties is still unfinished.

Where exposed, `i01_subset()` names a frozen earlier representation retained for
compatibility. The `font-family` and `font` wrappers use their current typed
accessors without that projection. A current property value can be valid and fully inspectable while a retained projection
is `None`. Consumers should use current typed accessors for newly represented
syntax. The projection is not a support or validity test. The [inspection guide](how-to.md#inspect-a-known-declaration)
shows the distinction.

## Support metadata describes a selected surface

The independent support catalog reports an exact support status for each declared conformance production: `Complete`, `Partial`, or `RecognizedUnsupported`. Partial records document both the accepted subset and valid-but-unsupported remainder. A clean use of a partial production's supported subset is accepted; status is metadata about the whole named production, not a parse-result validity flag.

The catalog records named productions and their source provenance. It is not a
claim that all CSS, every module at a given stability tier, or every valid future
spelling is implemented. The [reference](reference.md) describes current families
and their explicit limits; the [source registry](../src/conformance.rs) owns the
exact IDs, statuses, subsets, and exclusions.

## Selector recovery and downstream matching

Pseudo-classes for UI interaction, form state, structure, selector-list filtering, and overlay state are parsed as authored selector syntax. This crate does not evaluate pseudo-class matches; runtime matching belongs to downstream Surgeist layers with node and interaction state.

Selector-list pseudo-class arguments are parsed as authored selector syntax with bounded recovery. In recognized `:is()` and `:where()` lists, an invalid member is dropped with `DropSelectorListItem` while the other members remain in authored order. Other selector lists are unforgiving: `:not()` preserves supported complex selector lists, `:has()` preserves supported relative selector lists including leading child and sibling combinators, and `:nth-child()` / `:nth-last-child()` preserve optional `of` selector filters, but an invalid member causes the containing qualified rule to be dropped with `DropQualifiedRule`. Later sibling rules remain eligible for parsing.

## Downstream ownership

This crate owns authored CSS syntax, intrinsic grammar validation, recovery boundaries, diagnostics, and support metadata. It does not apply cascade or inheritance, substitute or resolve variables, evaluate queries, match selectors, resolve URLs or resources, perform layout or painting, serialize a CSSOM, or lower CSS into sibling Surgeist types. Root-owned integration owns cross-crate lowering and generated API audit artifacts.

Container queries are parsed as authored conditions on `@container` group rules. `surgeist-css` does not evaluate container query matches; container-dependent matching belongs to downstream Surgeist layers.

Imports are parsed as authored `@import` contracts only. `surgeist-css` preserves import targets, layer clauses, supports conditions, and media conditions, but does not resolve paths, load files, or merge imported sheets; root/style-owned Surgeist integration performs loading and composition.

Cascade layers are parsed as authored `@layer` statements and blocks, including named and anonymous layer blocks. `surgeist-css` records layer names and layer-contained rules, but does not compute cascade order, declaration precedence, or runtime cascade effects.

Scoped styles are parsed as authored `@scope` rules with optional roots, limits, scoped style
selectors, and scoped nested group rules. A scoped style rule also retains leading declarations
and ordered nesting children, including declarations after a nested rule. These children use
the containing scoped style rule as their parent selector context. Relative scoped selectors
remain structurally distinct from ordinary selectors. `surgeist-css` does not perform scope
matching, selector matching, or scoping proximity calculations.

Pseudo-elements are parsed as terminal authored selector syntax for the supported `::before`, `::after`, `::first-line`, `::first-letter`, `::marker`, `::selection`, and `::backdrop` forms. The Selectors 3 legacy single-colon spellings map to the same typed before, after, first-line, and first-letter values. The parser records pseudo-elements on selector compounds, but does not filter declarations by pseudo-element or perform generated box/layout behavior.

Generated content, list markers, and counters are parsed as typed authored property values for `content`, list-style longhands and shorthand, and counter change properties. Strings, URLs, attribute references, quote keywords, counter functions, list-style slots, and counter change lists remain symbolic. `surgeist-css` does not lay out generated content or list markers, resolve marker images, or evaluate/reset/increment counters.

Font faces are parsed as authored `@font-face` descriptor blocks only. `surgeist-css` validates supported descriptors and preserves font source hints, unicode ranges, and variation ranges, but does not perform font lookup, loading, matching, or resource validation; downstream Surgeist layers own those steps.

Font-source compatibility has a context-independent projection: legacy variation
format strings expose their base format and implied technology, and TrueType and
OpenType hints have an explicit equivalence check. Authored strings and technology
order remain available separately. Resource owners consume these requirements
alongside their own capabilities when deciding whether to load a source.

Keyframes are parsed as authored `@keyframes` rules. `surgeist-css` validates keyframe names, selector offsets, and declarations, but does not evaluate animations, match animation names to rules, interpolate values, or run animation timelines.

CSS nesting retains the authored rule tree. One style rule owns its complete selector list,
leading declarations, and ordered child rules. Declaration runs after a retained child and
inside nested conditional groups use `CssRule::NestedDeclarations`; they inherit the owning
style rule's selector context, including pseudo-elements. Nested selectors retain relative
combinators and symbolic parent anchors without copying or multiplying parent selectors.
Every explicit `&` remains attached to its compound, including repeated anchors,
anchors after other simple selectors, and anchors inside selector-list functions.
Thus `& > .item` retains its explicit parent compound, while `> .item` retains an
authored leading combinator. Selector functions inherit their nesting context;
the restriction against nested `:has()` still applies.
Downstream matching can therefore apply the parent list's maximum specificity to `&`, as
required by CSS Nesting, while preserving the source order of every declaration. The current
nested-selector grammar and separate scoped-style model remain bounded by their registered
support; preserving structure does not imply complete nesting grammar or live CSSOM support.

Normalization keeps shared immutable references to those selector and rule
contexts. Empty styles retain their selector lists, and declaration runs share
the enclosing style's complete matching context. An explicit nested style instead
retains a distinct binding to the parent selector list. Conditions and terminal
payloads remain symbolic and ordered. These occurrence references carry no
mutable CSSOM identity or revision; style owns that later phase.
