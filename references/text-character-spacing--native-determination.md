# Native character spacing

This reference records the design contract for [#1566](https://github.com/bj-data-eng/surgeist/issues/1566), contributing to #70, #69 and #72. It specifies ownership and native policies; current implementation capabilities remain documented by the owning crates. The pinned [CSS Text 4](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md) §§1.4, 4.4, 7.5 and 8.2 and [CSS Text 3](css-text-3--CRD-css-text-3-20260814--17c26f1b4145.md) govern the standard behavior.

## Operation and ownership

Text owns presentation EGCs, effective Appendix-E properties, operation-specific writing-system tailoring, actual visual neighbors and source custody. Tracking supplies each unit's half on its adjacent sides after bidi ordering, suppressing the outer halves at each actual selected line edge. Adjacent text values therefore average. Invisible zero-width formatting supplies no extra unit or gap. Cursive letter pairs have no internal tracking or InterCharacter opportunity when elongation is unavailable, whether or not the particular letters join.

Layout supplies actual normalized captures, atomic measurements, constraints and allocations, then owns fallback and coherent placement. It does not choose Text units or infer missing style facts. Render consumes the resulting placed geometry.

**Native interaction policy:** one effective boundary gap belongs to the preceding visual unit's logical extent. All subsequent glyphs and atomics move together; the two full endpoint values retain attribution without requiring independently executed half-cell positions. A(4), B(8) therefore has paint0/16, total26 and shared cell/caret edge16. A(4), B(−4) has paint0/10, total20 and unchanged edge10. CSS determines the paint/extent contribution; this cell ownership is a native interaction choice.

Compute the combined mean without avoidable overflow or underflow, recognizing exact opposite-value cancellation as zero. Do not reject a representable gap merely because separately materialized private halves would underflow. Check the actual executed displacement and observable numeric categories; a nonzero physical effect must not silently disappear.

Required shaping protection does not redefine source EGCs, editing units or line-breaking units. Protect the actual relation at a spacing boundary while retaining those independent records. Ordinary kerning or a reshaping dependency alone must not erase a spacing boundary.

Each supported native profile must state its qualified writing-system variants and actual unit tailoring. Generic provider clusters are not a universal answer. Myanmar/Indic syllables and Thai/Lao spacing decomposition retain their positive requirements until supported by the appropriate operation-specific evidence. Missing live Style producers do not prevent a contribution using genuine normalized inputs.

The first family boundary is Latin, Greek, Cyrillic, Armenian, Hebrew, Han, Kana, Bopomofo and Hangul, plus source-qualified Common punctuation/symbols and the Appendix-D cursive no-elongation families. This is not a claim that every orthography within those Unicode scripts has identical units. Named source variants retain their real context and require qualified treatment. For an unspecified source whose effective properties unambiguously select a supported family, the native default uses the EGC basis; that explicit native policy does not invent a known content language. Material unresolved or unsupported complex tailoring returns its precise capability result, while zero-effect operations remain usable. Other writing-system positives remain under the parents.

## Actual shaping constraints

Shaper owns observations of its actual invocation, not CSS units, opportunities or guessed writing systems. The minimum useful observation is a compact, input-mapped structural constraint receipt:

- Preserve reliable original component membership through substitution and reordering, including noncontiguous components and marks. Bind correspondence to the actual invocation and plan.
- Preserve actual applied multi-component ligature relations and the applicable effective-plan provenance set. Carry parent provenance through nested contextual application. Several features can reach one lookup; the first feature tag is not its authority. Normalization or a one-to-one substitution is not automatically a ligature.
- Retain actual cross-unit mark/cursive attachment dependencies where spacing needs them. An ordinary base-plus-mark relation internal to one EGC needs no separate public event stream. Kerning is not attachment indivisibility.

Completeness must follow the qualified provider path. Missing records are not affirmative independent-movement evidence on an unqualified path. Receipts must survive subsequent substitutions and completion, with actual output geometry qualifying protected components moving together at admitted outside boundaries. Do not expose provider pointers or a complete shaping-event transcript.

`UNSAFE_TO_BREAK` reports reshaping dependence at a cut, not a universal prohibition on spacing. Shaper machine syllable IDs are likewise not universally CSS TCUs: some categories represent broken or nonsyllable clusters. Consume a syllable partition only with a named mapping for that particular shaper, script and operation. Such a receipt is not a blanket prerequisite for an EGC-qualified profile, and cannot replace Thai/Lao decomposition.

Inside a genuine operation unit or a qualified required-shaping relation, independent component displacement is prohibited. A source-qualified cursive pair follows the no-elongation rule. At another mandatory boundary, an unclassified shared output or attachment relation cannot silently remove the candidate or prove that the line is unexpandable. If its requested material spacing cannot preserve the relation, return the precise unsupported spacing capability. Zero-effect and compatible outside operations remain usable.

## Selected feature basis

The selected spacing-capable shaping basis precedes natural measurement and Layout residual computation. Suppress `liga`, `clig`, `dlig` and `hlig` only where they are optional for the effective script plan, then apply actual low-level authored overrides with their specified precedence. Never disable required composition, ligation, marks, joining or script-required features to obtain convenient glyph widths.

The maintained Khmer shaper explicitly requires `clig`; a universal tag blacklist or `rlig`-only required classifier is insufficient. The [CSSWG clarification](https://github.com/w3c/csswg-drafts/issues/2644) retains the OpenType required-ligature convention and exceptional script handling. The [current tracking clause](https://drafts.csswg.org/css-text-4/#letter-spacing-property) and [feature precedence](https://drafts.csswg.org/css-fonts-4/#feature-precedence) corroborate the pinned distinction.

Automatic default-feature masking is local to endpoint units touched by a demanded boundary, using the provider's ordinary range-mask machinery without splitting its shaping invocation. It can also change a neighboring default optional ligature involving that touched unit at a zero boundary. No automatic demand introduces no such controls; this is not a guarantee that each zero boundary preserves optional output when an adjoining boundary is material. Explicit overrides and required behavior remain protected. A requested InterCharacter spacing-capable basis has its own declared potential-boundary demand before natural measurement, independent of the latest allocations.

An author-enabled optional ligature remains optional; it does not become a TCU or a required feature. Preserve the requested shaping. If a nonzero internal spacing request cannot be realized while preserving it, the bounded native capability returns a typed failure at that exact boundary. Do not move the amount outside its glyph, suppress the mandatory candidate or override the author's setting. Successful internal spacing for that preserved output remains a positive parent requirement.

`with_spacing` retains its immutable, no-font-query contract: replace complete amounts over the exact selected basis. Zero allocation restores that basis's natural geometry, not another feature plan. A method/request needing a different plan obtains a new selection and handles. Actual plan, controls and allocation identity participate in the existing canonical keys.

## Atomic and divided-unit values

Capture a finite signed absolute **full tracking value per atomic occurrence** in the canonical `Source::InlineBox` inventory. Its documented initial zero means actual zero; it never means infer a neighboring value. Avoid a full text Style for a replaced object and avoid another independently matchable atomic style table.

The native producer resolves the atomic's applicable exterior value, including its own used font-size context for relative values, before preparation. Atomic interior paragraph style is not that exterior fact. Carry the value through existing occurrence, contextual binding, edits, snapshots and cache/replay identity. A post-shaping geometry allocation cannot patch an issued preparation.

After actual selected-line bidi order, a maximal consecutive in-flow atomic run is one tracking unit. Out-of-flow content and the specified invisible formatting introduce no additional in-flow unit. Preserve actual visible text and real boundaries. Do not split a run merely at a storage or style boundary.

**Native policy for unequal atomic values:** the first visual atomic supplies the run's exposed visual-left half, and the last supplies its exposed visual-right half. Emit no interior atomic halves, even when their values differ. In vertical text use the corresponding visual inline coordinates. Recompute endpoints after each actual selected cut and suppress real outer line halves. CSS specifies run grouping and halves; this unequal-endpoint policy is not claimed as browser interoperability.

**Native policy for a text TCU divided by style:** its actual first EGC base, selected by the shared Appendix-E owner, governs both tracking halves. Marks supply no separate tracking value. Retain governing provenance. A protected multi-EGC required relation is not a new TCU: retain its first/last endpoint unit values and suppress only the protected internal spacing, without averaging the whole group.

NCA controls justification method, not authored tracking. Ordinary adjacent text units retain their own half values.

Examples use physical visual order and raw advances10:

| Units and full values | Internal gaps | Paint positions | Total |
| --- | --- | --- | --- |
| A(4), B(8) | 6 | 0,16 | 26 |
| A(4), X(8), Y(12), B(6), with X/Y atomic | 6,0,9 | 0,16,26,45 | 55 |
| X(8), Y(12), both atomic | 0 | 0,10 | 20 |
| X(8), Y(12), B(6), with X/Y atomic | 0,9 | 0,10,29 | 39 |
| A(4), atomic X(0) | 2 | 0,12 | 22 |

Reverse visual atomic order by moving each occurrence with its own value; never swap the values independently. A mark's differing style cannot move its base. Margins, borders, authored word spacing and selected allocations remain separate contributions.

The [linked atomic WPT](https://github.com/web-platform-tests/wpt/blob/master/css/css-text/letter-spacing/letter-spacing-204.html) corroborates uniform run grouping, not unequal endpoint ownership. Existing WebKit trailing-removal mechanics likewise do not establish that native policy.

## Qualification boundaries

Prove the actual changed natural width for optional `liga`/`clig`, author override preservation and precise internal refusal, a genuine `rlig` relation with a usable outside boundary, and kerning that retains its admitted gap. A `ccmp` fixture alone does not qualify `rlig`. Qualify mark attachment and a real cross-unit attachment, joining and nonjoining cursive pairs, actual atomic placements/cuts and source/AX geometry. Profile-specific complex units require their own evidence.

The numeric tab-size SPACE query is a resource operation, not a selected one-unit line. The pinned §4.4 explicitly includes its associated letter-spacing and word-spacing. Preserve that contract and actual intrinsic-zero word-spacing qualification while suppressing outer halves on real lines.

The native allocator first uses qualified actual responses. Proven exact fill can retain a constrained or deferred handle with an explicit zero allocation; that does not remove its candidate or prove it unexpandable. If known responses cannot prove fill, demand the needed material operation through Text before fallback and preserve its precise failure. Equal-priority distribution among admitted Direct opportunities remains independent of script.

Authored tracking, authored word spacing and allocated amounts remain separately attributable. Preserve source, raw packets, exact allocation/plan identity and finite/physical displacement proofs. Missing capability must not become silent underfill, false unexpandability or a cached failure. Broader Auto/Ruby/NoCompress, complete computed ancestry and unavailable live producer acceptance remain outside this bounded native outcome.
