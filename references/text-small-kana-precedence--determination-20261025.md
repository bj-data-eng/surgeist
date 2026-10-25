# Small-kana and word-break precedence

Source determination for #65, 25 October 2026. Text owns break eligibility;
Layout consumes issued opportunities. This resolves the native LineBreak
integration question against `faee3ef9cf29eccfd7efbe6a9e2245d0841b52bf`.
It changes no production code, declared property value, or public API.

## Determination

Apply BreakAll's effective `ID` classification to processed small-kana `CJ`
letter units before applying the class-based CJ restriction. Do not veto that
opportunity merely because the processed character originally had class CJ.
Under Normal, retain the CSS prohibition before CJ for Normal/Strict line
breaking; under Loose retain the required opportunity, including under KeepAll.
The provider's Normal CJ relaxation and its BreakAll CJ conversion are different
cases: the first needs CSS tailoring, the second must survive that tailoring.

This is the adopted interpretation of the interacting CSS rules, supported by
the editor's explicit CJ clarification below. It is not a claim that CSSWG
adopted a separate resolution containing this exact precedence matrix, nor a
claim of cross-browser interoperability.

Named iteration marks and hyphen-like characters retain their specific
line-break rules. Do not implement a blanket rule that every Unicode Letter
escapes every line-start restriction: the six explicitly named iteration marks
remain restricted even where their general category is Letter. A required
Loose opportunity cannot subsequently be removed by KeepAll.

## Boundary matrix

These are ordinary opportunities at the indicated text/text boundary, with
wrapping enabled, intact typographic units, and no independent glue/control
restriction. Columns are line-break values. The preceding unit is an ordinary
eligible letter, for example A or a Han ideograph. No lexical exception is
assumed for KeepAll.

| Target and word-break | Loose | Normal | Strict |
| --- | --- | --- | --- |
| Before processed CJ, Normal | allow | forbid | forbid |
| Before processed CJ, BreakAll | allow | allow | allow |
| Before processed CJ, KeepAll | allow | forbid | forbid |
| Before full-size kana ID, Normal | allow | allow | allow |
| Before full-size kana ID, BreakAll | allow | allow | allow |
| Before full-size kana ID, KeepAll | forbid | forbid | forbid |
| Before named iteration mark, any of these word-break values | allow | forbid | forbid |

The iteration row covers U+3005, U+303B, U+309D, U+309E, U+30FD, U+30FE.
It is a codepoint-specific requirement, not a permission to erase all NS rules.

| Hyphen-like boundary, any of these word-break values | Loose | Normal | Strict |
| --- | --- | --- | --- |
| Before U+301C/U+30A0, Chinese/Japanese writing system | allow | allow | forbid |
| Before U+301C/U+30A0, other writing systems | forbid | forbid | forbid |
| Before U+2010/U+2013, preceding effective ID | allow | forbid | forbid |
| Before U+2010/U+2013, preceding non-ID | forbid | forbid | forbid |

For A followed by U+2010/U+2013, BreakAll makes A effective ID and therefore
permits the Loose boundary; Normal and KeepAll do not. A preceding Han ID
qualifies under all three word-break values. KeepAll cannot remove the required
Loose opportunity. These punctuation rules remain operative under BreakAll.

## Processing and ownership

Classify the actual processed text after text-transform; retain original source
correspondence separately. Full-size-kana can turn a CJ input into ID, so an
original-source CJ test is wrong even under Normal. At ordinary mixed-style
boundaries retain the adopted following-box authority; this determination does
not transfer style ownership or combine unrelated offers at equal offsets.

Text's private adapter must distinguish provider observations, effective
word-break classes, named line-break requirements, and required opportunities.
Keep declared Auto distinct; its existing private Strict selection remains.
Do not change the facade fixture to Loose. Preserve grapheme/typographic-unit
integrity, source ranges, no-hyphenation under BreakAll, and independently owned
explicit, atomic, and forced opportunities. No new public enum is needed.

## Decisive sources and limits

- [CSS Text 4](https://drafts.csswg.org/css-text-4/#word-break-property), live
  checked 25 October 2026 (displayed edition 14 August 2026), and the retained
  [complete pinned Markdown](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md):
  sections 1.4, 6.1, and 6.2 define letter units, effective ID treatment,
  mandatory line-break opportunities, and named punctuation restrictions.
- [CSSWG #2559 editor clarification](https://github.com/w3c/csswg-drafts/issues/2559#issuecomment-444698382):
  CJ and Hangul classes already belong to typographic letter units; their
  omission from the additional non-letter class list does not exclude them.
  The same explanation assigns NS restrictions to line-break. The
  [follow-up](https://github.com/w3c/csswg-drafts/issues/2559#issuecomment-446442964)
  reports editorial interaction cleanup. This is editor explanation, not a
  separately minuted matrix resolution.
- [CSSWG #785 resolution](https://github.com/w3c/csswg-drafts/issues/785#issuecomment-375019136)
  retains the specification. Its punctuation/CJK discussion does not establish
  an original-CJ veto overriding BreakAll's effective class.
- ICU4X `icu_segmenter` 2.2.0, `src/line.rs`,
  `get_linebreak_property_utf32_with_rule`, explicitly converts CJ to ID under
  BreakAll even with Strict. Its separate Normal conversion must not be adopted
  as CSS Normal's CJ rule. This is provider precedent, not normative authority.
  [Versioned source](https://docs.rs/crate/icu_segmenter/2.2.0/source/src/line.rs).
- [Pinned WebKit Markdown](webkit-text-source-gaps-hyphenation--73aa6c89e2cb.md),
  revision `73aa6c89e2cb77c46184a81aec944e4ab99d114d`,
  `InlineContentBreaker::canBreakBefore`: the BreakAll path does not impose a
  general small-kana veto, but explicitly rejects U+2010/U+2013 outside Loose.
  Its FIXME acknowledges incomplete restrictions; do not import it as a complete
  named-iteration or supplementary-character oracle.
- [Pinned Blink iterator](https://chromium.googlesource.com/chromium/src/+/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/platform/text/text_break_iterator.cc):
  its BreakAll pair table is not a universal letter-to-ID conversion. Static
  inspection of that table alone does not establish complete browser output.
  No browser-consensus claim is made.
- The scoped inventory of `word-break-break-all*.html` in WPT revision
  `f4febe305ac97a2aa53435bf6bf98c0a78d8128f` supplied no direct small-kana,
  strictness, or named-iteration precedence matrix. Tests 007/008 cover combining
  clusters, not this question. No WPT or browser execution was performed.

## Required implementation evidence

Exercise every matrix cell, including real U+1B132 and BMP small kana, all six
named iteration marks, each named hyphen-like character, language gates,
effective preceding ID, and genuine full-size-kana transformation. Include
KeepAll plus required Loose opportunities and a combining-unit negative case.

Retain the actual facade kana-Ahem fixture at width 17, BreakAll, Auto/Strict:

| Transformation | Source ranges and actual advances | Min-content | Max-content |
| --- | --- | --- | --- |
| Disabled | 0..1:10, 1..3:10, 3..4:10 | 10 | 30 |
| Enabled | 0..3:15, 3..4:10 | 10 | 25 |

These are existing fixture expectations, not newly executed results. Preserve
the original UTF-16 source and the supplementary character's 1..3 correspondence.
The allocated line bounds remain width 17; they are not the measured advances.
Add Normal and KeepAll cases to distinguish transformation-induced class changes
from BreakAll conversion. Run the focused native, Layout and facade checks under
the implementation coordinator's existing validation scope. Reference inspection
does not certify the frozen candidate or its tests.
