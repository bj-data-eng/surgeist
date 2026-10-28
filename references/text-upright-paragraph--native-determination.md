# Native upright paragraph spacing, arrangement and script admission

Root adopted this Text-owned policy on 2026-10-27 from the Architecture
Coordinator's determination relayed in the upright-paragraph successor task.
That determination, rather than browser parity or an unstated standards rule,
is the authority for the native choices below. It applies to the admitted native
whole-upright Text paragraph profile under #781. It supplements the separate
[used Central/em reference](font-used-upright-baselines--native-determination.md)
and [nominal glyph metrics](font-vertical-metrics--native-determination.md);
it changes neither observation contract and does not itself admit a paragraph.

## Metrics-relative normal extent

Keep unscaled design quantities until their actual once-only size conversion:

`normal = used_over - used_under + gap`

`MetricsRelative(factor) = normal * size / UPEM * factor`

Reject a negative or nonfinite `normal` before applying a factor that could hide
it. Preserve signed gap and signed half-leading without clamping. Apply the
existing numeric checks again at actual scaling and consumption boundaries.
Gap changes line spacing only; it never changes Central, em edges, nominal or
effective glyph origins, or nominal glyph advances.

The shared strict metric owner supplies gap from qualified vhea 1.1
`vertTypoLineGap` plus the actual same-instance MVAR `vlgp` delta. Extend its
existing metric-tag storage, retaining the owner's rounding, structural record
validation, typed errors and instance identity. Missing MVAR or a missing `vlgp`
record supplies delta zero only on this vhea 1.1 path.

Genuine vhea absence or a qualified vhea 1.0 header supplies gap zero and does
not demand `vlgp`. No hhea substitute is used. A vhea 1.0 reserved nonzero gap
must not affect spacing; retain existing strict validation even when it rejects
such a header. Malformed/truncated demanded headers and unsupported versions
are errors, not absence. The glyph-free gap operation asks for no vmtx, outline,
SPACE or glyph-zero observation.

MetricsRelative demands gap even when BASE supplies em edges: BASE plus a bad
consumed vhea/MVAR still errors. Absolute and FontSizeRelative heights do not
independently demand gap or `vlgp`; a glyph-free operation with qualified BASE
can therefore ignore an otherwise irrelevant bad vhea. Explicit height does not
protect data genuinely demanded for em edges or an actual nominal glyph query.
Retain shared structural MVAR validation, including its record validation,
whenever that table is actually demanded.

[OpenType vhea table formats](https://learn.microsoft.com/en-us/typography/opentype/spec/vhea#table-formats)
identify the v1.1 gap field and recommend em-box width plus gap; the
[variation mapping](https://learn.microsoft.com/en-us/typography/opentype/spec/vhea#vhea-table-and-opentype-font-variations)
identifies `vlgp`. The formula's application to this native normal extent,
zero-gap absence/v1.0 policy and exact demand boundaries are the adopted native
policy. No browser-equivalence claim follows.

At design width1000, UPEM1000, size10 and factor1.2: gap200 gives14.4;
a same-instance gap delta+100 gives15.6; gap−200 gives9.6; gap below−1000
refuses before the factor. Qualify separate instance, body/default-strut and
explicit-height demand cases; a scalar arithmetic test alone is insufficient.

## Tibetan vertical-arrangement units

Start from the actual presentation extended grapheme clusters (EGCs). The exact
punctuation-base set is `{0F08, 0F0B, 0F0C, 0F0D..0F12, 0F14}`. Exclude
U+0F13 and head marks U+0F04..U+0F07. Do not infer membership by name matching.

Within one actual shaping domain, a complete EGC whose base belongs to that set
appends to the immediately preceding arrangement unit, including all its
combining characters. At a fragment/domain start it owns an orphan unit;
subsequent leading set members append to that unit. Never seek a letter base
backward across whitespace, controls or a boundary. Immediate whitespace can
be the preceding unit, so shad–space–shad is not one syllable unit. Create or
remove no spaces or source content.

Hard boundaries, captured actual shaping boundaries and genuine source-free
occurrences are barriers. Source chunks, font fallback, style/font intersections
or storage boundaries alone are not barriers. Carry complete-unit context
through their intersections; transparent capture metadata retains custody
without becoming an invented shaping barrier.

Arrangement units do not replace editing EGCs or line-break eligibility.
U+0F0B and U+0F0C retain their distinct breaking behavior. A legitimate issued
line cut can intersect a unit precomputed for a larger domain. Preserve that
cut's identity and all content, then rerun the same arrangement algorithm on
the authentic selected fragment, applying the orphan rule at its start. Do not
suppress an offered/required break, widen the selected range, drop a remainder,
or refuse an otherwise legitimate cut merely to retain the earlier grouping.
Editing, caret and logical selection endpoints remain unchanged. Actual foreign
or invalid ranges still fail at their existing owner.

[Pinned Text 3 §1.4](css-text-3--CRD-css-text-3-20260814--17c26f1b4145.md#characters)
supplies the EGC-based grouping objective and Tibetan tsek/shad example; exact
membership, orphan, immediate-predecessor, barrier and fragment rules here are
native tailoring. The [Unicode18 Tibetan NamesList](https://www.unicode.org/Public/18.0.0/charts/nameslist/0f00/)
identifies the characters, and Richard Ishida's
[Tibetan orthography notes](https://r12a.github.io/scripts/tibt/bo#phrase)
provide orthographic evidence for U+0F08 and spaced shad usage. These references
do not upgrade the repository's Unicode data or establish browser parity.

Independent cases include: `[0F40,0F72,0F0B,0F0D]` as one unit, with appended
U+0F41 starting a second; leading `[0F0D,0F0D,0F40]` as `[marks],[letter]`;
each included member and excluded set; a punctuation EGC with combining marks;
shad–space–shad; actual barriers versus transparent captures; and legitimate
U+0F0B cuts through a larger grouping with content-preserving fragment rebuild.
Check actual glyph/context behavior and source receipts as well as grouping.

## Effective unit properties and narrow script admission

Root subsequently adopted the Architecture Coordinator's explicit-script
resolution and the conservative complete-unit style-coherence boundary below.
They are bounded Surgeist capability policies, not standard/browser rejection
rules and not a new public source input.

Resolve [Text 3 Appendix E](css-text-3--CRD-css-text-3-20260814--17c26f1b4145.md#character-properties)
effective unit properties first, from the base of its first actual EGC. Do not
union the ScriptExtensions of every scalar in the arrangement unit. A bare Zs
retains its own properties. A Zs-base combining sequence becomes Sk, keeps the
base EAW and takes other properties from its first combining character. Apply
the Common-Me→U+FFFD property rule only when its actual ScriptExtensions
predicate holds; Inherited U+20DD is not Common. No source character is replaced.

Apply the pinned [Writing Modes script exceptions](css-writing-modes-4--CR-css-writing-modes-4-20190730--30a59de49343.md#script-orientations)
to that effective width **before** intrinsic-script membership: F/W takes the
vertical-character path and H the horizontal-character upright path. Do not
apply UAX50's R classification to rotate whole-upright horizontal characters.
The ordinary effective set is then evaluated as follows:

- If every candidate is supported unrotated, pass the capability gate without
  selecting one script merely for admission.
- If every candidate requires an unsupported intrinsic transform/direction,
  refuse with the actual candidate set and content range.
- For a mixed set, resolve only from the actual captured `Style.locale`'s
  explicit single Unicode script subtag, and only if it is a member. Every
  actual governing style intersecting the complete arrangement unit must agree
  on that same explicit single-script subtag. A qualified supported member
  passes; a qualified intrinsically unsupported member refuses precisely.

Missing explicit script, a nonmember, an aggregate/unqualified script tag or
conflicting governing contexts leaves the mixed set unresolved and refuses
precisely. A transparent storage/font split alone changes no source context.
Use the existing qualified locale parser plus actual Unicode-script lookup;
no likely subtags, aggregate expansion, neighboring/provider shaping script,
font/default locale, paragraph direction or hyphenation convention supplies a
fallback answer. Bare `fr` and `mn` are meaningful language tags; this bounded
owner lacks a qualified language→writing-system resolver and therefore cannot
use them to resolve the mixed set.

Admission covers actual body and real indication content, retaining their
source/alternative custody. It does not sweep a font cmap or invent hypothetical
source units. The source classifier remains distinct from contextual shaping
script and resource itemization. The complete-unit coherence rule does not
suppress legitimate line cuts: rebuild the fragment's actual units/context
through the existing selected-presentation path described above.

[Pinned Text 3 Appendix F](css-text-3--CRD-css-text-3-20260814--17c26f1b4145.md#script-tagging)
establishes the relevance of declared writing-system context. [UAX24's
Script_Extensions definition](https://www.unicode.org/reports/tr24/#Script_Extensions)
and [usage model](https://www.unicode.org/reports/tr24/#Usage_Model) distinguish
set-valued script associations from a context-sensitive selected script. These
sources do not mandate this profile's exact unsupported/coherence boundary.
The explicit-subtag-only resolver is the adopted native capability limit;
qualified implied-language, aggregate and conflicting-context support remains
under #781, alongside actual intrinsic transforms.

Discriminating [Unicode17 ScriptExtensions](https://www.unicode.org/Public/17.0.0/ucd/ScriptExtensions.txt)
cases are bare U+202F `{Latn,Mong,Phag}`, U+205A
`{Cari,Geor,Glag,Hung,Lyci,Orkh}`, and U+2E30 `{Avst,Orkh}`: unresolved
without qualifying explicit context, regardless of a Latin neighbor, font or
paragraph direction. `und-Latn` resolves U+202F to an admitted member;
`und-Mong`/`und-Phag` select unsupported intrinsic members. `und-Geor` admits
U+205A; `und-Orkh` refuses. `und-Avst` admits U+2E30;
`und-Latn` is a nonmember and stays unresolved. Bare `fr` does not resolve
U+202F. U+3008's W exception passes before its Mong membership, even with a
Mong locale. Actual U+202F+U+0363 instead uses its first combining character's
Latin properties while preserving the space's width; it is not the bare-space
candidate set. A qualified Mongolian combining sequence uses effective Mong
properties and refuses. Verify actual governing-style disagreement, membership
and complete-unit spans independently from glyph behavior; a constructed
Common-Me predicate case is rule qualification when bundled Unicode supplies
no positive source witness.

## Explicit decoration capability

The initial profile admits disabled decorations and enabled decorations with
both explicit physical offset and thickness. Offset is measured toward
line-over/right from Central; thickness occupies the cross axis and advance
the inline axis. The common line rectangle map supplies the physical rectangle.
A fully explicit operation must not demand unrelated horizontal underline or
strike metrics.

An enabled decoration requiring an automatic upright offset or thickness
refuses precisely before preparation publication, identifying style owner,
decoration kind and missing role. Automatic upright decoration policy remains
separate #781 work. This is Root's adopted capability boundary, not a claim that
CSS or OpenType prohibit automatic decorations.

Text owns these transitions and their independent behavioral qualification.
A successful scalar metric observation or grouping calculation does not qualify
shaping, placement, readback, Layout/Render admission or #781 completion. No
exceptions to this determination are established; change requires an explicit
replacement determination by its owner.
