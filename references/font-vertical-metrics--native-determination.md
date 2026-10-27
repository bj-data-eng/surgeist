# Native vertical glyph metrics

Text owns this unhinted, same-instance glyph-metric policy for
[#1513](https://github.com/bj-data-eng/surgeist/issues/1513), a producer
contribution to [#781](https://github.com/bj-data-eng/surgeist/issues/781).
The crate documentation and source establish implemented capabilities. This
determination supplies neither vertical paragraph placement nor named-baseline
authority; [named font baselines](font-baselines--native-determination.md) retain
their separate coordinate contract.

## Observation and coordinates

The input identifies an actual retained font instance and nominal glyph. Preserve
its resource, catalogue, face index, size and complete canonical coordinates.
Glyph zero is valid; a glyph outside the actual glyph count is an input error.
Historical resources remain usable under their original catalogue publication;
a foreign expected catalogue refuses.

Observe the glyph before substitutions and positioning. The result contains its
vertical advance magnitude and origin relative to the horizontal font-design
origin: positive x points right and positive y points up. A downward nominal
advance therefore has vector `(0, -advance)` in this frame. Multiply each design
quantity by `size / units_per_em` once, retaining fractional coordinates. Do not
apply hinting, pixel rounding, synthetic bold/italic, vertical features or page
rotation. Half a horizontal advance is a local centering convention, not proof
of a Central or alphabetic baseline.

Each component retains its provenance. A synthesized component identifies its
actual selected horizontal metric source. Text's existing
[horizontal metric owner](../crates/surgeist-text/src/font/metrics.rs) owns
selection; this operation consumes its resolution without independently
reinterpreting OS/2 or hhea. An unavailable required component cannot become a
guessed value.

## Static TrueType profile

The qualified profile is static TrueType with glyf outlines. Actual variable
faces, including their default coordinates, and unqualified CFF/CFF2 profiles
return explicit unsupported-capability errors. Valid unsupported formats are
distinct from malformed consumed data. This profile does not authorize a
variation or CFF implementation by substituting default deltas or fake outlines.

| Vertical data | Advance magnitude | Origin x | Origin y |
| --- | --- | --- | --- |
| Coherent native vhea/vmtx | Glyph advance height | Horizontal nominal advance / 2 | Glyph yMax + top side bearing |
| Both tables genuinely absent | Resolved horizontal ascent - signed descent | Horizontal nominal advance / 2 | Resolved horizontal ascent |

Native table consumption validates the actual directory records, supported
versions, metric counts, lengths and glyph reference. Repetition of the last
long advance is legitimate; required trailing bearings still exist. Qualified
composite glyph bounds are real references. An actually empty glyf record has
yMax zero; an unavailable or malformed reference is not an empty glyph.
The [OpenType glyph header](https://learn.microsoft.com/en-us/typography/opentype/spec/glyf#glyph-headers)
uses negative contour counts for composites; -1 is recommended. Another negative
count does not make the stored header malformed.

Exactly one vertical table is an inconsistent pair and errors. Malformed
consumed data cannot invoke synthesis. TrueType VORG is irrelevant and is
ignored even when malformed. An applicable vertical variation table without its
required companions is inconsistent. Native queries need not consume unrelated
horizontal line or strikeout metrics; synthesis consumes the existing ascent
and descent policy without adding strikeout-only requirements.

## Missing-metric policy and evidence

Pinned [Writing Modes 4 §5.1.1](css-writing-modes-4--CR-css-writing-modes-4-20190730--30a59de49343.md#vertical-font-features)
requires synthesis when vertical metrics are missing and leaves the heuristic
undefined. The absent-table row above is explicit native policy. Keep line gap
out of its advance and origin. Preserve valid zero advances and signed finite
origins. Do not substitute one em, enlarge the advance to contain ink, center
individual ink or infer BASE coordinates.

The ascent origin is supported by the pinned
[WebKit vertical-data implementation](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/opentype/OpenTypeVerticalData.cpp#L499)
and [Blink implementation](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/platform/fonts/opentype/open_type_vertical_data.cc#L265).
The retained [Harfrust advance mechanism](../vendor/harfrust/src/hb/glyph_metrics.rs)
provides separate ascent-minus-descent precedent. Browser/provider integer
rounding and permissive parsing are not adopted. Harfrust's ink-centering origin
is not selected: an ascent origin remains stable across glyphs and requires no
outline when vertical tables are absent.

The official OpenType 1.9.1
[vmtx](https://learn.microsoft.com/en-us/typography/opentype/spec/vmtx),
[VORG](https://learn.microsoft.com/en-us/typography/opentype/spec/vorg) and
[VVAR](https://learn.microsoft.com/en-us/typography/opentype/spec/vvar)
references supply format rules, separately from this synthesis policy. They are
external references, not newly retained source snapshots. A future qualified
CFF path uses its applicable VORG override/default, or actual CharString bounds
and bearing; VORG never supplies advance. A future variable path must qualify
all relevant same-instance horizontal/vertical metrics and outline deltas. It
cannot add CFF2-specific VORG deltas to a TrueType origin or combine VVAR and
phantom-point adjustments twice.

## Independent acceptance values

These are design/scale oracles for faithful owned fixtures, not observed output.
Qualify actual font tables, glyph mapping and required references first. At
UPEM 1000 and size 10, with horizontal advance 1000 and glyph yMax 800:

| Case | Vertical advance | Origin |
| --- | ---: | --- |
| Native advance 1200, bearing 100 | 12 | (5, 9) |
| Absent tables, ascent 800, descent -200 | 10 | (5, 8) |
| Absent tables, ascent 900, descent -300 | 12 | (5, 9) |
| Absent tables, ascent 1000, descent -200 | 12 | (5, 10) |

The third row distinguishes ascent-minus-descent from one em. The fourth also
distinguishes the selected origin from ink-centering. Changing line gap affects
neither component; changing horizontal advance to 1001 yields x 5.005. Doubling
size doubles every component. For genuine empty native glyphs, origin y equals
their bearing; absent-table synthesis still uses ascent without reading outlines.

Acceptance also covers coherent long-metric repetition, incomplete or malformed
consumed records, zero advances, negative bearings, actual composite references,
unsupported profiles, catalogue/resource lifetime and unchanged paragraph
orientation refusals. A successful metric observation alone does not establish
vertical shaping, orientation, line geometry or renderer integration.
