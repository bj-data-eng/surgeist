# Whole-upright bidi interpretation

This determination owns the native Text interpretation of whole-upright source
analysis under #781. It applies when the analysis front door admits that resolved
typography. It preserves ordinary horizontal composition, including clockwise
sideways composition, and does not define nested mixed-orientation behavior.

## Standards basis and native decision

The [pinned Writing Modes 4 upright rule](css-writing-modes-4--CR-css-writing-modes-4-20190730--30a59de49343.md#text-orientation)
requires used LTR direction and strong-LTR treatment for bidi ordering. Its
[control-translation rules](css-writing-modes-4--CR-css-writing-modes-4-20190730--30a59de49343.md#bidi-algo)
model CSS formatting through a character stream and generally honor source
controls. Their exact precedence for every explicit control in whole-upright
text leaves an interpretation gap. [CSSWG #755's final resolution](https://github.com/w3c/csswg-drafts/issues/755#issuecomment-270897344)
added an author workaround note; it did not remove the upright requirement.

Surgeist resolves that gap by neutralizing the directional effects of both
literal source controls and captured semantic formatting controls in its bounded
whole-upright profile. Source-free capture must not let RLO bypass the
interpretation applied to literal RLO. This is an explicit native policy, not a
claim that CSSWG resolved Surgeist control types or that browsers interoperate
on this interaction. Keeping captured directional instructions active while
neutralizing their literal counterparts is not the selected profile.

## Used stream and preserved ownership

The shared Text analysis-stream owner applies one used-bidi projection after
processing and capture establish the genuine stream and structural boundaries.
Original UTF-16, processed scalar identities, Unicode properties, source ranges,
control occurrences, owner identities and event order remain intact. The
projection does not mutate original character properties or become another
source analyzer.

Within each body, participating source/transformed characters use effective
bidi class L. Literal LRE, RLE, LRO, RLO, PDF, LRI, RLI, FSI, PDI and direction
marks, and their captured semantic equivalents, cannot operate an effective
embedding/isolate stack or impose an override. They may occupy inert L-class
anchors retaining their original owners. Their original semantic role still
owns invisibility, removal and custody; projected L never turns a control into
rendered text or invents source content.

Genuine source B separators, Forced controls and issued computed boundaries
retain their structural function. Establish those delimiters before projection;
they split UBA/DirectionBody contexts at their actual event cuts, including a
trailing empty context. A removed separator stays removed. Soft wraps, isolates
and physical empty/out-of-flow seams do not create extra paragraph bodies.
Existing source-versus-captured close/reopen custody around a true boundary
remains distinct, even though its directional effect is inert here.

Every admitted whole-upright body uses base 0. Effective nonstructural content
and control-anchor levels are 0; B terminators and empty-body bases are 0.
Original brackets cannot restore RTL mirroring. Whitespace/control roles retain
their existing trimming, L1 receipt and extraction duties. Plaintext and body
direction helpers consume this same used interpretation, including LTR fallback
for an empty body; raw property observers continue to report original facts.

## Input authority and admission

Computed direction is a separate retained authority from used paragraph
direction. An admitted computed RTL value stays RTL while whole-upright analysis
uses LTR. Existing `ParagraphInput.base_direction` and the issued contextual paragraph
base are already-resolved used paragraph authorities. A captured paragraph used
RTL claim is contradictory and must fail at the new input boundary; used LTR is
admissible. They cannot be relabelled as computed direction and silently replaced.
The nearest-block [tab reference](../crates/surgeist-text/src/tabs.rs) has its own
used direction, explicitly independent of paragraph bidi. Assigning typography
to an analysis paragraph neither rewrites nor rejects that independent fact,
including an RTL block reference accompanying a used-LTR paragraph. Existing
`Style.direction` admission is not widened implicitly.

Original captured pairing, coverage and ownership validation still runs.
Neutralization does not admit malformed native captures. Literal source controls
retain the existing raw-text validity rules; this policy adds no unmatched-literal
rejection. Invalid capture or conflicting authority fails before analysis is
published. Bind actual typography and computed-direction facts to the analysis
allocation at all three real construction fronts. Private owner tests may inspect
body/level facts; no public per-control-level test API is needed.

## Independent acceptance cases

| Genuine input | Whole-upright result | Ordinary horizontal control |
| --- | --- | --- |
| Hebrew ALEF followed by `1` | Body base 0, content levels [0, 0], source order | Automatic RTL base and its existing independently derived levels |
| Literal RLO, `AB`, PDF in a fixed-LTR body | Content `AB`, levels [0, 0]; original control bytes retained | Content `BA`, levels [1, 1] |
| Native RLO-open, source `AB`, matching close | Same content `AB`/[0, 0], with source-free control owners retained | Content `BA`/[1, 1] |
| Nested captured RLE/RLI/FSI and paired closes | No levels 1/2 or isolate-created body; original capture retained | Existing embedding/isolate behavior |
| Surviving source `A`, LF/B, Hebrew | Two actual bodies at base 0, each content level 0; separator range retained | Existing source paragraph behavior |
| Genuine Forced between `A` and Hebrew | Corresponding two contexts at event cuts; source extraction contains only actual source | Existing native control behavior |

A terminal source/Forced separator retains the existing trailing empty context.
An issued computed boundary must be qualified through its real producer, not
substituted with fabricated source LF. RLO spanning or adjoining B/Forced cannot
erase the boundary or carry an override through it. Qualify original validation
and custody separately from effective content levels. Raw X9-removable-control
bookkeeping cannot substitute for content-level assertions.

Keep original brackets, digits, marks, surrogate/source distinctions, transforms,
segmentation and logical accessibility text identity. A valid font-independent
upright analysis does not itself admit Vertical preparation, font shaping or
placed geometry. Generated indication bodies belong to their real later
preparation/selection consumer, which must use this interpretation when enabled.
The character-oriented WPT [direction-upright-001](https://github.com/web-platform-tests/wpt/blob/master/css/css-writing-modes/direction-upright-001.html)
and [direction-upright-002](https://github.com/web-platform-tests/wpt/blob/master/css/css-writing-modes/direction-upright-002.html)
are reference precedents, not execution evidence for the native control cases.
