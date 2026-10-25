# BreakSpaces and Unicode glue: source reconciliation

Retrieved and inspected on 24 October 2026 for
[#64](https://github.com/bj-data-eng/surgeist/issues/64).
This Markdown reference records source identities, findings and limits. The
owning issue records the implementation determination. No browser execution or
Surgeist test result is claimed. No upstream implementation code is reproduced.

## Standards evidence

The current [CSS Text 4 editor's draft](https://drafts.csswg.org/css-text-4/)
still displays 14 August 2026. Its relevant clauses agree with the existing
[pinned readable edition](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md):

- [BreakSpaces](https://drafts.csswg.org/css-text-4/#valdef-white-space-collapse-break-spaces)
  describes opportunities after preserved document whitespace and other space
  separators. The latter category includes U+2007 and U+202F, but excludes NBSP.
- [Line-breaking details](https://drafts.csswg.org/css-text-4/#line-breaking)
  retain Unicode glue/joiner rules unless explicitly overridden and prohibit
  soft breaks inside typographic character units.
- [Character properties](https://drafts.csswg.org/css-text-4/#character-properties)
  give Zs-based combining clusters a symbol classification. A base SPACE does
  not justify moving an illegal interior break to the cluster's far edge.

The broad separator description and the specific GL behavior need reconciliation;
the after-every wording alone does not establish an intentional GL override.

The current draft directly links the two WPT cases below. They explicitly test
the continued GL behavior of U+2007 and U+202F under BreakSpaces, not merely
their retained width. The selected interpretation retains these protections.

## CSSWG and test-history checks

- [CSSWG #8972 resolution](https://github.com/w3c/csswg-drafts/issues/8972#issuecomment-1646217049)
  concerns atomic inlines and gives explicit controls precedence, except NBSP.
  It does not authorize an own-GL exception for BreakSpaces separators.
- [CSSWG #3221 clarification](https://github.com/w3c/csswg-drafts/issues/3221#issuecomment-430860597)
  distinguishes ordinary after-space wrapping from emergency breaking before a
  first space. It does not remove glue or character-unit constraints.
- [CSSWG #5783](https://github.com/w3c/csswg-drafts/issues/5783) discusses
  overflowing first spaces and earlier legal opportunities, not a GL override.
- [CSSWG #9724 resolution](https://github.com/w3c/csswg-drafts/issues/9724#issuecomment-2628481466)
  concerns mixed hanging sequences; it does not resolve this break-legality overlap.
- [WPT #21381](https://github.com/web-platform-tests/wpt/issues/21381) corrected
  the separator tests' UAX14 interpretation. These tests are not treated as
  obsolete merely because they predate the current CSS draft.

No examined adopted CSSWG resolution establishes a BreakSpaces GL override.
This is a bounded research finding, not proof that no other discussion exists.

## Pinned browser and test witnesses

Blink revision: `daf70f9ddee4b45d4f8870f574fbde61c0db4ca7`.
WebKit revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`.
Hashes below cover complete original file bytes, retained in ignored research
material; the public record contains metadata and original analysis only.

| Source | SHA-256 | Decisive evidence |
| --- | --- | --- |
| [Blink imported WPT U+2007, lines 7 and 42](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/web_tests/external/wpt/css/css-text/white-space/trailing-other-space-separators-break-spaces-009.html#L7) | `654b2e43850ead4749491f221b3f4ff18e5d324a806c23891c2dc57c25add746` | Expected rendering retains GL suppression after FIGURE SPACE. |
| [Blink imported WPT U+202F, lines 7 and 42](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/web_tests/external/wpt/css/css-text/white-space/trailing-other-space-separators-break-spaces-013.html#L7) | `8cf39ce2df0c6206395a66b86fbbf1ecbc12dc3a3d77cb5a581bb4f405be8ebc` | Expected rendering retains GL suppression after NARROW NO-BREAK SPACE. |
| [Blink character predicate, lines 151–154](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/platform/text/character.h#L151) | `23adc4661785411305d727107f664dd88a8f2e22bd45ff8f509dbdd405eaca8b` | Its other-separator predicate recognizes U+3000 only. |
| [Blink iterator, lines 291–300](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/platform/text/text_break_iterator.cc#L291) | `ea0fb1d3795561e3666de1842e20be6d1c7a65c0d92c571e678f3a2cf25b82f4` | The after-every-space shortcut does not add U+2007/U+202F opportunities. Its direct return does not establish neighboring WJ/ZWJ/combining protection. |
| [WebKit item builder, lines 1125–1140](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/layout/formattingContexts/inline/InlineItemsBuilder.cpp#L1125) | `d7ae9c4259a6fb2a8204c8f4437e4b2bfcad58e3c590f752a81065e100be9d67` | BreakSpaces splits ordinary SPACE/TAB items; U+2007/U+202F follow ordinary breaking. |

These are static witnesses, not recorded WPT pass results. Browser whitespace
shortcuts must not be copied as permission to split a typographic unit or erase
the independent WJ/ZWJ rules required by CSS. The tests support the GL decision;
they do not certify all neighboring-control combinations.

The [current upstream U+2007 test](https://github.com/web-platform-tests/wpt/blob/f4febe305ac97a2aa53435bf6bf98c0a78d8128f/css/css-text/white-space/trailing-other-space-separators-break-spaces-009.html)
and [U+202F test](https://github.com/web-platform-tests/wpt/blob/f4febe305ac97a2aa53435bf6bf98c0a78d8128f/css/css-text/white-space/trailing-other-space-separators-break-spaces-013.html)
at WPT `f4febe305ac97a2aa53435bf6bf98c0a78d8128f` are byte-identical to those
imported witnesses. They carry the `should` flag; tests corroborate the selected
interpretation but do not themselves acquire normative authority. Their
[2020 UAX14 correction](https://github.com/web-platform-tests/wpt/commit/75e279ecef9322470f7790844b77bd96e5e25dbc)
explicitly retained GL's after-prohibition in the expected layouts.

## Unicode edition boundary

The repository's [UAX14 import](uax14--17.0.0--rev55-metadata.md) remains Unicode
17, revision 55. Its LB12a exceptions include SP, BA, HY and HH. Thus SPACE and
TAB (BA) before NBSP are not blanket next-GL prohibitions under that baseline.

The newer [Unicode 18 revision 57](https://www.unicode.org/reports/tr14/tr14-57.html#LB12a)
was inspected. It removes BA from that exception, retaining SP/HY/HH. This
clarification does not silently upgrade property data or the line-breaking
algorithm. Any future revision must change the edition, properties, algorithm
and affected acceptance cases together. The requested U+2007/U+202F, SPACE/NBSP,
WJ and combining-unit determinations do not depend on that BA difference.

Unicode report text is not reproduced, following the existing import's public
full-text exception. External clause links identify the rules precisely.
