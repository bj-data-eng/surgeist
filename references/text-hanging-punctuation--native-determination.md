# Hanging punctuation composition

This determination applies to #71/#72. It selects bounded native composition
rules where CSS leaves attribution or overlapping edge roles unspecified. It
does not establish feature availability or browser equivalence.

## Measurement and attributed advance

Pinned [Text 4 hanging](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#hanging)
excludes a hanging character's advance from the applicable measurements while
retaining its glyph, parent-inline enclosure and justification participation.
[Punctuation eligibility](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#hanging-punctuation-property)
and [Appendix E](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#character-properties)
operate on complete typographic character units (TCUs) and their effective
properties. The Text 3 [hanging](css-text-3--CRD-css-text-3-20260814--17c26f1b4145.md#hanging),
[punctuation](css-text-3--CRD-css-text-3-20260814--17c26f1b4145.md#hanging-punctuation-property)
and [Appendix E](css-text-3--CRD-css-text-3-20260814--17c26f1b4145.md#character-properties)
counterparts preserve these distinctions.

Kerning interaction remains a UA choice under the pinned clause and the
[CSSWG 2016-10-19 resolution](https://lists.w3.org/Archives/Public/www-style/2016Oct/0115.html).
[CSSWG #2397](https://github.com/w3c/csswg-drafts/issues/2397) tracks possible
further definition; it does not select a universal attribution algorithm.

The native choice is the actual final attributed contribution advance of the
complete selected TCU. Do not reshape punctuation in isolation, reassign a pair
adjustment, substitute ink width, or take the absolute value of a signed advance.
Preserve the original contribution and glyph/source/caret associations.

For `allow-end`, fit and MinContent omit the eligible full contribution;
alignment excludes only its actual natural overflow, and MaxContent retains it.
The [existing allocation policy](SOURCE-EDITIONS.md#conditional-hanging-after-spacing-allocation)
keeps natural eligibility, edge/frame and terminal groups fixed, then remaps the
signed exclusion through allocated geometry. Applying that policy to punctuation
is an explicit native choice, rather than a claim of uniquely mandated CSS.

For example, **if independently qualified shaping** attributes 9px to `A` and
9px to a period, raw advance is 18px and `force-end` leaves a 9px measurement.
For `allow-end`, fit/MinContent are 9px and MaxContent is 18px; alignment is 18px
at width 20px and 15px at width 15px. These are prospective oracles, not observed
fixture results. A genuine kern fixture must first qualify its actual glyph
pair, attributed advances and feature execution.

## One TCU eligible at both edges

If the same authentic complete TCU qualifies for both `first` and `last`, omit
its contribution by union once. Use logical inline-start placement priority
under the actual line-box direction. Retain distinct start/end eligibility and
barrier facts: a blocked or ineligible start does not suppress a valid end.
Preserve one actual draw and source occurrence. Equal-looking quotes with
different occurrence identities remain distinct TCUs.

The union follows the one-contribution measurement invariant. Inline-start
placement priority is explicit native policy; CSS does not resolve this
degenerate placement tie. Normal eligibility, formatted-line conditions and
inline border/padding barriers still apply independently at each edge.

## Current-line border and padding incidence

Under the pinned [hanging rule](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#hanging),
nonzero inline-axis border or padding **between the glyph and the edge of the
line** prevents hanging. Captured enclosure or shaping-context reachability alone does
not establish that spatial relationship. The pinned
[fragment-decoration rules](css-break-3--CR-css-break-3-20181204--4d47d4b2dd36.md#break-decoration)
distinguish `slice`, which inserts no border/padding at a broken edge, from
`clone`, which decorates each fragment. A remote endpoint cannot establish
decoration on the current line merely because it belongs to an ancestor inline.

The native selected-contribution profile uses authentic, canonically ordered
line-local endpoint contributions for this barrier. Require actual current-
fragment incidence, intervention between the complete selected TCU and that
physical edge, and genuine used border/padding. Positive selected BP blocks that
edge and contributes once; zero is transparent. Margin, container padding and
indentation do not supply this veto.

An off-selected `boundary_context` allocation has joining authority and supplies
no selected flow, placement or fragment. It therefore cannot independently veto
hanging, even when its value is positive or captured ancestry is available.
Preserve its shaping effect, geometry validation and demand ordering. Do not
infer scope pairing from flat `ParagraphControl` order: pairing remains owned by
contextual capture. A following incoming Start can affect joining without
preventing a selected closing mark from hanging under a valid Last role.

This selected-profile boundary is explicit native policy, not a claim that CSS
forbids all exterior fragment decoration. A requested decoration outside this
model, such as a cloned continuation edge, requires actual capture/Layout-owned
current-line decoration incidence and used geometry. Until that capability is
provided, refuse the consumed operation precisely; do not infer an envelope,
manufacture zero geometry or import a remote joining endpoint as decoration.

## Effective-unit eligibility

Apply Appendix E before category and literal-code-point punctuation eligibility.
A genuinely reclassified `So`/Common or `Sk` unit cannot recover quote, period
or U+3000 permission from its raw base. This is an explicit native composition
rule: it preserves original text and does not disable all marked punctuation.
Ordinary nonreclassified literal exceptions remain valid over the whole TCU.

| Unit | Eligibility consequence, subject to the actual line and edge conditions |
| --- | --- |
| Quote or period plus U+20DD | Ordinary literal eligibility survives; this mark does not trigger the Common-script enclosing-mark exception. |
| `(` plus U+0301 | The ordinary `Ps` base retains `first` eligibility over the whole TCU. |
| U+3000 alone | The literal ideographic-space exception permits `first`. |
| U+3000 plus U+0301 | The real `Zs`-plus-mark exception yields `Sk`; raw U+3000 cannot restore `first`. |

The [Unicode 17 character data](https://www.unicode.org/Public/17.0.0/ucd/UnicodeData.txt),
[Scripts](https://www.unicode.org/Public/17.0.0/ucd/Scripts.txt) and
[ScriptExtensions](https://www.unicode.org/Public/17.0.0/ucd/ScriptExtensions.txt)
contain no Common-script `Me`: these marks are Inherited or Cyrillic. U+20DD is `Me`/Inherited with no Common ScriptExtensions. Existing
[lexical-fallback assertions](../crates/surgeist-text/src/engine/analysis/lexical_fallback.rs)
and [hyphenation assertions](../crates/surgeist-text/src/engine/analysis/hyphenation/tests.rs)
corroborate the U+20DD distinction. A resolved shaping run's script does not
replace a mark's actual Script/ScriptExtensions properties.

Do not manufacture a real Unicode 17 quote-plus-Common-`Me` fixture or treat
U+20DD as that witness. The effective `So`/Common rule remains conditional on
genuine reclassification. The examples above qualify metadata and complete-unit
eligibility; they do not certify font coverage for combining glyphs. No test,
fixture or browser execution is asserted by this determination.
