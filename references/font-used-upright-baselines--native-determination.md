# Used upright font baselines

This defines the native used-font reference for upright composition. It is
separate from the [raw named-baseline observation](font-baselines--native-determination.md)
and [nominal glyph metrics](font-vertical-metrics--native-determination.md).
Those observations retain their existing coordinates, provenance and errors.
Availability of a used metric does not itself admit an upright paragraph.

## References and positioning

A used font/script observation carries an explicit reference:

- **Font design X:** absolute coordinates from qualified vertical BASE/em-box
  evidence.
- **Nominal vertical pen X:** distances from the adopted vertical pen, whose
  conversion for an actual glyph is its nominal origin `Ox(g) = W(g) / 2`.

Vertical BASE coordinates are design X, not distances from each glyph's nominal
origin. Do not treat `defaultBaselineIndex` as an origin or subtract a guessed
half em. In the nominal pen frame an absolute Central coordinate `C` lies at
`C - Ox(g)`. Different glyph widths therefore require different conversions.

Resolve the effective origin before font positioning:

| Used reference | Effective origin X |
| --- | --- |
| Font design X, Central `C` | `C` |
| Nominal vertical pen X, Central distance `c` | `Ox(g) + c` |

Retain the actual nominal advance and origin Y. Default outline offset X is
minus the effective origin. GPOS, mark/cursive attachment and fallback
positioning must consume the same effective reference; a later stage must not
silently restore nominal origins. A post-GPOS per-glyph `Ox - C` correction
cannot repair attachment geometry. A common run-to-line translation remains a
separate placement operation.

The Text/provider adapter carries this used reference explicitly, bound to the
actual retained font instance and script. Reuse the metric supplier only when
its contract faithfully carries the effective origin; otherwise provide a
narrow pre-positioning reference input. Do not mutate the raw or nominal query,
reconstruct attachment roots, or introduce a second shaper.

## Source precedence

1. Consume the existing qualified raw vertical Central/em-edge resolution when
   available. Retain each authored, format-derived or synthesized edge's actual
   provenance and use the font-design-X reference. A malformed consumed BASE
   record is an error. A valid unsupported reference retains its precise
   refusal; it is not missing evidence.
2. When the raw reference is genuinely unavailable, valid vhea 1.1 em-side data
   gives `under = vertTypoDescender`, `over = vertTypoAscender`, and
   `c = (over + under) / 2` in the nominal-vertical-pen reference. Mapping the
   vhea centerline to this pen is an explicit native adapter convention, not
   evidence of an absolute design-X origin. Preserve asymmetry: the name of
   that reference does not force the CSS midpoint to zero.
3. With neither qualified raw evidence nor usable vhea 1.1 em-side data, use
   `under = -UPEM / 2`, `over = UPEM / 2`, and `c = 0`, centered on the declared
   nominal vertical pen. This includes valid vhea 1.0, whose fields describe
   interline distances rather than the 1.1 em-side reference. Record
   synthesized-upright-em-at-nominal-pen provenance.

Present malformed required vhea data or inconsistent table combinations fail
through the strict metric owner. Unknown valid unsupported versions/references
refuse; they do not select synthesis. The raw baseline query remains unavailable
where it was unavailable before this used-reference resolution.

Do not normalize or transplant horizontal ascent/descent into X, or manufacture
a font-wide origin from glyph zero, ink bounds or average width. Line gap does
not enter either em edge or Central; its separate line-spacing owner remains
responsible for applicable line-height behavior. Preserve exact fractional
halves without integer or pixel rounding.

Retain any fractional coordinate already qualified by the raw observation.
A positioning operation's supported coordinate grid and range are separate
capabilities. An unrepresentable valid reference needs a precise capability
refusal at that boundary, rather than rounding, a malformed-data label, or
selection of the missing-data heuristic.

## Numeric witnesses

The following values are design units, with UPEM 1000 and nominal glyph width
1001, so the unchanged nominal origin X is 500.5. Used edges are measured
relative to the resolved Central.

| Qualified evidence | Reference and Central | Effective origin X | Used under/over |
| --- | --- | --- | --- |
| BASE edges 0 and 1200 | Font design X, `C = 600` | 600 | -600 / 600 |
| No BASE; vhea 1.1 over 700, under -300 | Nominal pen X, `c = 200` | 700.5 | -500 / 500 |
| No BASE; vhea 1.1 over 500, under -500 | Nominal pen X, `c = 0` | 500.5 | -500 / 500 |
| No qualified BASE or vhea 1.1 em sides | Nominal pen X, `c = 0` | 500.5 | -500 / 500 |

The last two rows have different provenance. At size 10, scale each design
quantity by `10 / 1000` once: the first row's Central/effective origin is 6,
while its nominal origin remains 5.005. The asymmetric row has Central distance
2 and effective origin 7.005. With a second glyph of width 800, absolute BASE
Central 600 still gives effective origin 600 rather than its nominal 400.

Changing horizontal ascent/descent from 800/-200 to 900/-300 at fixed UPEM and
width does not change synthesized cross edges, Central or effective origin X.
The independently synthesized nominal advance/origin Y may change from
1000/800 to 1200/900. Changing line gap alone changes neither cross edge nor
Central. Qualify these distinctions through actual GPOS and fallback attachment
fixtures, not isolated glyph arithmetic alone.

## Identity and ownership

Glyph-free default-font/strut metrics consume the actual font/script reference,
not a fabricated glyph-zero query. Retain reference kind, resolved Central and
edges, provenance and actual font/script identity in dependent shaping and
used-metric cache inputs. Equal numeric results with different reference or
provenance do not establish interchangeable requests. Preserve retained instance
lifetimes and original typed metric/table/profile failures.

These choices do not establish mixed orientation, bidi interpretation, vertical
feature selection, child-box baseline placement or renderer support. Those
owners must qualify their actual consuming profiles separately.

## Sources

- Pinned [Writing Modes 4](css-writing-modes-4--CR-css-writing-modes-4-20190730--30a59de49343.md),
  section 4.2: upright dominant Central and missing-baseline midpoint synthesis.
- Pinned [Inline Layout 3](css-inline-3--WD-css-inline-3-20241218--a73a73a45bf5.md),
  baseline definitions and synthesis. Appendix A.1's Canvas-oriented normalized
  metrics are not this CSS used-reference policy.
- [OpenType BASE](https://learn.microsoft.com/en-us/typography/opentype/spec/base)
  and [ideographic em-box tags](https://learn.microsoft.com/en-us/typography/opentype/spec/baselinetags#ideographic-em-box):
  vertical design-X coordinates and qualified em-box references.
- [OpenType vhea formats](https://learn.microsoft.com/en-us/typography/opentype/spec/vhea#table-formats):
  distinct version 1.1 center-relative em-side and version 1.0 interline fields.

The vhea-to-pen mapping and absent-data centered-em rule are explicit native
policies. They do not claim browser equivalence or change the repository's
pinned standards editions.
