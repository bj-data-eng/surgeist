# Missing lexical capability and KeepAll

Determination for #65, 25 October 2026, against published
`2ddbaf57e8320fa2682dd3f3afa9d61693c8c323`. Text owns analysis and issued break
permission; Layout consumes it. This source-only determination authorizes no
provider acquisition, feature change, API expansion, or production edit.

## Decision and evidence boundary

**KeepAll retains required missing-lexical-capability fallback.** Treat the
assumed TLU-pair opportunities as substitutes for the unavailable lexical
operation, within KeepAll's lexical exception. Do not run a generic KeepAll
letter-pair veto over them afterward. They are ordinary soft opportunities,
available before overflow and included in min-content, not conditional
OverflowWrap permission and not forced breaks.

The fallback obligation and KeepAll's lexical exception are normative. Their
exact composition when no lexical analyzer exists is not stated explicitly in
the retained text; this precedence is an **explicit bounded Surgeist policy**.
It preserves the fallback's stated purpose without turning KeepAll into Manual.
The alternative of suppressing these fallback cuts would recreate the overflow
that section 6.1.1.3 requires the unavailable-analysis fallback to avoid.

Manual deliberately disables lexical analysis and treats SA as AL. That is not
a missing capability requiring replacement: do not run lexical fallback under
Manual. Preserve orthographic breaking and independently owned opportunities.
This does not settle broader orthographic tailoring or new dictionary admission.

## Capability and mode matrix

For two adjacent qualified Thai/Lao letter TCUs in one connected analyzed run,
with wrapping enabled and no glue or explicit/control complication:

| Mode | Lexical capability unavailable | Qualified analyzer, no internal boundary | Qualified analyzer, boundary present |
| --- | --- | --- | --- |
| Normal | ordinary fallback cut | no lexical cut | ordinary lexical cut |
| KeepAll | ordinary fallback cut | no lexical cut | ordinary lexical cut |
| BreakAll | ordinary effective-ID cut | ordinary effective-ID cut | ordinary effective-ID cut |
| Manual, when implemented | no lexical/fallback cut | no lexical/fallback cut | no lexical/fallback cut |

BreakAll permission does not depend on missing capability. Do not attribute its
independent ID-based permission to a dictionary result or require duplicate
offers at the same authentic cut. A qualified empty result is successful
analysis, never evidence of unavailability. Conversely the current constructor's
empty model slots cannot certify that a run is one linguistic word.

Loose, Normal, Strict, and the existing Auto-to-Strict choice do not change this
Thai/Lao missing-capability matrix. LineBreak Anywhere supplies its separate
ordinary permission, including under Manual. OverflowWrap, discretionary
hyphenation, genuine Wbr/ZWSP, whitespace, forced breaks and atomic relations
keep their existing owners and priorities. The table removes none of them.

## Exact scope and guards

1. Analyze actual post-transformation text and real SA/script/TLU properties.
   Limit this first unit to qualified Thai/Lao TCUs: CSS explicitly supports
   EGCs as their line-breaking units. Do not generalize this to all SA or to
   Myanmar/Devanagari syllables that can span EGCs. A punctuation-only or
   formatting-only unit is not a letter unit merely because it is in a script
   range. Never split a surrogate pair, EGC or larger established TCU.
2. Preserve GL/WJ/ZWJ restrictions and existing mandatory line-breaking guards.
   The fallback is not Anywhere. Do not use assumed opportunities as fabricated
   linguistic word boundaries for editing, selection, word-space-transform,
   phrase detection, or hyphenation queries. They are line-breaking assumptions
   with a missing-lexical cause, not successful word detection.
3. Preserve the adopted following-box ordinary style authority and actual NCA
   wrapping gate. NCANone/nowrap suppresses this ordinary permission. A forced
   or genuine explicit offer is evaluated by its own owner, not suppressed
   because fallback is absent. No permission crosses a disconnected, hard, or
   in-flow barrier or turns an atomic relation into a text pair.
4. Connected inline/style/empty/out-of-flow seams do not truncate the analysis
   context. Preserve exact source/cut provenance and paragraph direction.
   An adjacent Manual contribution gets its actual ordinary policy; do not
   enable fallback over the whole component solely because another span is
   Normal or KeepAll.
5. For the current no-model path, actual qualified Thai/Lao runs require the
   fallback under known, unknown/und, or mismatched declared language. Missing
   metadata does not turn unavailable analysis into supported-empty analysis.
   Inspect the writing system without rewriting or inferring the actual style
   locale. Known language should tailor a capable analyzer; selecting/admitting
   a future dictionary for a specific language remains a separate qualification.
6. Keep capability distinct from result. A private missing-capability cause is
   sufficient; no public API is prescribed. Provider/runtime failure must not
   be silently relabeled as successful empty analysis. Existing cancellation,
   work limits and error handling still apply; this decision adds no blanket
   catch-all recovery path.

## Sources and mechanism checks

The live [CSS Text 4 draft](https://drafts.csswg.org/css-text-4/#fallback-breaking)
was checked on 25 October 2026 and still displays 14 August 2026. The complete
[pinned Markdown](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md) supplies
sections 1.4, 6.1 and 6.1.1, plus normative Appendix H. In particular it separates
inability to analyze from a supported analyzer finding no word boundary, retains
lexical opportunities under KeepAll, and expressly disables lexical breaking
under Manual. The unknown-language rule above follows the run-analysis mandate
and its conditional instruction to use language information when known.

The reviewed CSSWG discussions do not provide a direct adopted resolution for
missing-lexical fallback versus KeepAll:

- [#1619](https://github.com/w3c/csswg-drafts/issues/1619#issuecomment-370652139)
  clarifies letter-unit coverage, not unavailable lexical capability.
- [#7193 resolution](https://github.com/w3c/csswg-drafts/issues/7193#issuecomment-1611772475)
  moves automatic boundary functionality into word-break. Its discussion
  distinguishes default Thai dictionary breaking from deliberate manual control.
  [Later resolutions](https://github.com/w3c/csswg-drafts/issues/7193#issuecomment-1663101848)
  name AutoPhrase and address phrase overflow. Neither is a KeepAll/no-model
  matrix; do not use optional CJK phrase enhancement as the SA fallback oracle.
- [#2455 test discussion](https://github.com/w3c/csswg-drafts/issues/2455#issuecomment-374437766)
  identifies Thai dictionary/KeepAll tests, not simulated missing capability.

[Pinned Blink](https://chromium.googlesource.com/chromium/src/+/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/platform/text/text_break_iterator.cc)
has `ShouldKeepAfterKeepAll` excluding complex-context characters from its
letter-pair veto, then reaches the underlying iterator. That supports keeping
complex lexical ownership separate from KeepAll suppression. It does not prove
what a browser does when its dictionary is absent. Pinned WebKit's
`BreakablePositions.h` at `73aa6c89e2cb77c46184a81aec944e4ab99d114d` similarly
combines KeepAll fast-path rules with ICU lookup; it is not an unavailable-model
test. No browser execution or consensus is claimed.

[WPT word-break-keep-all-003](https://github.com/web-platform-tests/wpt/blob/f4febe305ac97a2aa53435bf6bf98c0a78d8128f/css/css-text/word-break/word-break-keep-all-003.html),
SHA-256 `b23b55cc476342ee6c333cf018462434d84898f62412dd20829513313da4e4b2`,
uses Thai `และ และและ` and expects a dictionary boundary in the joined words
under KeepAll. It does not disable a model. This Markdown record preserves its
relevant assertion and fixture meaning, not an executed result.

The installed [ICU4X 2.2.0 line source](https://docs.rs/crate/icu_segmenter/2.2.0/source/src/line.rs)
documents `new_for_non_complex_scripts` as lacking complex-script support and
constructs empty complex payloads. In `src/complex/mod.rs`, the absent-model
branch returns only the slice end. Thus the current missing interior cut is
not a qualified linguistic result. This is implementation evidence for the
trigger; CSS, not that provider's default output, supplies the fallback oracle.

## Acceptance for the bounded implementation

Use genuine Normal `กก`, known th, no models, OverflowWrap Normal, real wrapping
NCA: expect ordinary cut 1. Repeat KeepAll, unknown/und and mismatched locale
without locale mutation; add a qualified Lao analogue. Check all ordinary
line-break strictness values. This determination predicts behavior; it runs no
RED or GREEN test.

Add base-plus-tone-mark units, punctuation/formatting exclusions, GL/WJ/ZWJ,
source/style seams, disconnected barriers, NCANone, both paragraph directions,
forced/explicit opportunities, and real issued-inventory/intrinsic consumers.
Assert fallback is ordinary, not overflow-conditional. Later Manual must remove
only lexical fallback and lexical analysis while preserving genuine Wbr/ZWSP,
Anywhere and independently qualified orthographic behavior. Once a dictionary
is qualified, add a successful no-internal-boundary negative control: absence
of cuts alone must never activate fallback. Broader script and provider work
does not block these missing-capability determinations or count as completed.
