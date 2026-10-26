# Native named font baselines

This determination specifies the Text-owned named-baseline measurement contract
for [#1486](https://github.com/bj-data-eng/surgeist/issues/1486), the first bounded
contribution to [#781](https://github.com/bj-data-eng/surgeist/issues/781). It
defines axis-qualified font observations. Vertical paragraph layout remains a
separate capability; existing horizontal line metrics retain their owner.

## Sources and ownership

The pinned [Writing Modes 4 §§4.1–4.3](css-writing-modes-4--CR-css-writing-modes-4-20190730--30a59de49343.md#intro-baselines)
and [Writing Modes 3 §§4.1–4.3](css-writing-modes-3--REC-css-writing-modes-3-20191210--39564c2366c0.md#intro-baselines)
provide the baseline model and typographic-axis choice. Pinned
[Inline 3 §3.2](css-inline-3--WD-css-inline-3-20241218--a73a73a45bf5.md#baseline-types)
defines the named metrics; its
[Appendix A.2](css-inline-3--WD-css-inline-3-20241218--a73a73a45bf5.md#baseline-synthesis-fonts)
permits missing-metric synthesis. The native choices below complete those
relationships without replacing the pinned editions.

Official [OpenType 1.9.1](https://learn.microsoft.com/en-us/typography/opentype/spec/)
documentation supplies the font-format rules:
[BASE](https://learn.microsoft.com/en-us/typography/opentype/spec/base),
[baseline tags and ideographic em-box](https://learn.microsoft.com/en-us/typography/opentype/spec/baselinetags#ideographic-em-box),
and [Device/VariationIndex formats](https://learn.microsoft.com/en-us/typography/opentype/spec/chapter2#device-and-variationindex-tables).
These are external format references, not newly retained source snapshots.
The immutable [WebKit FontMetrics.h:44–72 witness](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/FontMetrics.h#L44-L72)
supports using an unnormalized used-metric midpoint with line gap separate. It
does not establish a vertical design-X reference or make that fallback mandatory
in CSS.

Text owns font-table admission, script/name/basis resolution, same-instance
variation, scaling and provenance. Its existing
[font instance](../crates/surgeist-text/src/font.rs),
[metric owner](../crates/surgeist-text/src/font/metrics.rs) and
[actual run readback](../crates/surgeist-text/src/views.rs) remain the ownership
boundary. Layout later consumes these observations for dominant/named-baseline
alignment and placement; it must not duplicate font parsing or synthesis.

## Coordinate and identity contract

Measurements are **unhinted, size-scaled font-design coordinates**:

| Basis | Reference and positive direction |
| --- | --- |
| Horizontal | Font design Y=0; increasing Y, above in an unrotated horizontal run |
| Vertical | Font design X=0; increasing X, right in an upright unrotated font |

These coordinates are neither page positions nor glyph-specific vertical
origins. They do not incorporate a sideways rotation or rebase to the BASE
default baseline. Writing Modes selects vertical tables for vertical typographic
mode and horizontal tables otherwise, including sideways typographic mode.
A requested measurement basis does not attest that a run was vertically shaped.

An observation retains the actual face, resource, catalogue publication, size,
complete canonical variation coordinates, requested basis/name, script selection
and provenance. A run query derives script from its actual resolved run. An
explicit instance query instead describes the caller's Unicode-script measurement
request. Neither invents a font script or manufactures a run. A later or foreign
catalogue publication refuses; retained historical instances remain usable with
their original publication, as with existing font metrics.

Alphabetic and central are independently requestable. Return a valid `romn`
coordinate without rebasing it to zero. Expose central-minus-alphabetic only when
both exist in the same basis/reference; unavailable alphabetic must not suppress
an available central measurement.

## Script and named-coordinate selection

Map the requested or actual Unicode script to its OpenType **legacy** script
tag, then try exact `DFLT` if that record is absent. Use the existing script
mapping capability, including mappings such as Hiragana to `kana`; do not
infer a script from glyphs, font names or coverage. Do not use the first record,
GSUB's `latn` fallback, or a modern shaping-script tag in place of this BASE
selection. Validate consumed tag/script inventories, ordering, coordinate counts
and default indices. The BASE `defaultBaselineIndex` identifies a script's
default tag; it never substitutes for an explicitly requested name.

Alphabetic selects `romn`. Central uses the ideographic **design** edges `idtp`
and `ideo`, not ink-edge `icft`/`icfb` or an invented `cent` tag. In the vertical
axis, `romn` denotes the alphabetic baseline of rotated characters; it is not
an upright-Latin baseline guarantee. Missing horizontal alphabetic synthesizes
Y=0 and marks that fact. Missing vertical alphabetic is unavailable, with no
horizontal-to-vertical fallback.

## Central precedence and arithmetic

Use the first applicable row for the requested script and axis. UPEM means the
validated font's units per em; all edge arithmetic below is in font-design units
before consistent size scaling.

| Horizontal evidence | Central measurement and provenance |
| --- | --- |
| Both valid `ideo` and `idtp` | CSS midpoint of the font-defined edges |
| Only `ideo` | Derive top=`ideo+UPEM` by the font-format rule, then CSS midpoint |
| Only `idtp` | Synthesize bottom=`idtp-UPEM` by the adopted one-em heuristic, then CSS midpoint |
| Neither edge | Synthesize `(A+D)/2` from the existing resolved same-instance ascent A and signed descent D |

The last row deliberately retains the existing used-metric selector and its
actual-instance effects. Do not normalize its span to one em, add leading or
half-leading, introduce an always-typo selector, or add a nonzero `romn` coordinate
to it. Extending this fallback to non-CJK fonts is explicit native policy.
[Inline A.1](css-inline-3--WD-css-inline-3-20241218--a73a73a45bf5.md#baseline-synthesis-em)
normalizes the separate em-over/em-under metrics for Canvas consistency and
explicitly says those metrics are not used by CSS; it does not change this
chosen fallback.

| Vertical evidence | Central measurement and provenance |
| --- | --- |
| Both valid vertical edges | CSS midpoint of their X coordinates |
| Only vertical `idtp` | Font-format left=0, right=`idtp`; preserve a non-UPEM right edge |
| Only vertical `ideo=0` | Synthesize right=UPEM using the one-em heuristic |
| No vertical edges, but valid selected-script horizontal `ideo` establishes the OpenType em-box | Format-derived left=0/right=UPEM, hence central=size/2 after scaling |
| None of those references | Unavailable vertical central |

A nonzero vertical `ideo` is malformed semantic data, not a fallback opportunity.
The horizontal-em-box witness must undergo the same script and coordinate
qualification; it is not a rotation of the horizontal midpoint. Do not transplant
horizontal ascent/descent into X, infer CJK font status from a CJK run, or
substitute advance widths, VORG/vmtx origins or bounding boxes. OpenType's broader
CJK-font-metadata inference and a vhea-based reference conversion are outside
this capability. A vhea1.1 center-relative metric alone does not establish the
adopted design-X reference; that source alone leaves the measurement unavailable.

After supported instance effects, take the **exact finite CSS midpoint**,
retaining fractional design/scaled geometry with numerically safe averaging.
Do not truncate individual fractional variation contributions or the midpoint,
round to pixels, or normalize the edges. This is explicit CSS-facing native
arithmetic: it deliberately differs from the OpenType registry's whole-font-unit
toward-zero center calculation. Distinguish font-defined edges, format-derived
edges and local synthesis from the CSS midpoint calculation in provenance.

## Unhinted coordinate and variation profile

[BASE coordinate formats](https://learn.microsoft.com/en-us/typography/opentype/spec/base#basecoord-tables)
are admitted for BASE versions 1.0 and 1.1 as follows; other versions require a
precise unsupported-version refusal:

| Used coordinate | Required behavior |
| --- | --- |
| Format 1 | Use its stored design coordinate |
| Format 2, static or variable font | Validate the genuine referenced glyph/point through the supported outline-validation capability, then use its stored design coordinate |
| Format 3 with null adjustment | Use its stored coordinate with genuine absence of adjustment |
| Format 3 with genuine VariationIndex | Apply the referenced BASE ItemVariationStore using the complete canonical instance vector |
| Format 3 with a static-font Device table | Fully validate format, range and packed payload; omit ppem pixel corrections under this declared unhinted profile |

Format 2 does not claim contour measurement or execute hinting. Without a
VariationIndex its stored coordinate remains constant across variation
instances: do not apply gvar/contour deltas or refuse merely because the font is
variable. A known-invalid glyph/point is malformed. A valid outline/reference
kind that cannot be validated is a precise unsupported capability.

Unhinted does not mean default-instance. Qualify used BASE variation stores,
indices, complete referenced rows and canonical coordinate domains even at
default/inactive coordinates. Do not use MVAR as the BASE store or silently
replace failed reads with zero deltas. A genuine Device table in variable BASE
is invalid; it is not a VariationIndex and cannot be accepted by omitting its
delta. Unknown adjustment formats or valid unsupported machinery retain precise
refusals. Never guess ppem from font size or advertise a raster-grid result.

## Failure and capability boundaries

Keep legitimate missing table/axis/script/tag, unavailable coordinate reference,
malformed present data and valid unsupported capability distinct. Only genuine
absence can select a qualified fallback above. Malformed required present data
must not become synthesized success. Apply existing publication, table, finite
geometry and numeric diagnostics; publish no partial observation or cache
mutation on failure. Querying metrics leaves glyphs, cuts, placement, selection
and cache activity unchanged.

This contract adds no vhea/VORG reference adapter, hinted measurement, vertical
shaping, orientation partition or vertical glyph placement. Preserve the current
non-Horizontal selection/placement refusals until those actual capabilities
exist. It does not complete #781 or change existing horizontal line metrics.

## Decisive fixture values

These are independent expectations for real registered Ahem-derived tables,
not executed results or new font resources:

- With UPEM1000, size10, no BASE and resolved A800/D−200: horizontal alphabetic0
  and central3; size20 gives central6. Without an em-box witness both vertical
  names are unavailable. Leading changes none of these values.
- With hhea900/−300, typo700/−200 and USE_TYPO clear, the existing selector gives
  horizontal central3 at size10. Normalizing the hhea span or always selecting
  typo would give2.5, distinguishing the adopted policy.
- H edges−200/800 with `romn=25`, and V edges0/1200 with `romn=350`, give at
  size10: H central3/alphabetic0.25 and V central6/alphabetic3.5. Removing V
  `romn` preserves central6 and makes only alphabetic unavailable.
- Edges0/1001 and−1001/0 give central+5.005 and−5.005 at size10. A supported
  instance-adjusted edge1001.5 opposite0 gives central5.0075. Whole-unit
  truncation would lose these fractions.

Controls must also preserve stored Format 2 coordinates at distinct variable
instances, distinguish genuine VariationIndex changes from static Device
omission, and separate malformed references from unavailable or valid
unsupported data. Expectations come from authored font fields and the rules
above, not copied observations.
