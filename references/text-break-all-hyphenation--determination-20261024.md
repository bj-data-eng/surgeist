# Break-all, source SHY and mixed-style hyphenation custody

Determination for [#65](https://github.com/bj-data-eng/surgeist/issues/65),
24 October 2026. Source review against `dcdfbf9d`; no implementation or runtime
tests performed. Browser mechanisms below are evidence, not a claim that their
complete hyphenation behavior conforms or that the reviewed tests passed.

## Standards and browser evidence

The current [CSS Text 4 draft](https://drafts.csswg.org/css-text-4/#word-break-property)
still identifies the 14 August 2026 edition, retained in
[the full Markdown reference](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md).
Break-all supplies ordinary opportunities by its letter/class tailoring and
suppresses hyphenation. Hyphens controls additional hyphenation opportunities;
SHY is conditional, unlike an always-visible hyphen. A SHY indication uses its
own character's style. Inline boundaries do not partition lexical words.

[CSSWG #791](https://github.com/w3c/csswg-drafts/issues/791#issuecomment-295310593)
was closed as invalid following explanations that break-all takes precedence
over hyphenation. This is explanatory issue disposition, not a new resolution
defining source-cut identity or mixed-style ownership. Searches of current
CSSWG discussion did not establish a contrary adopted rule for those details.

Pinned browser source distinguishes ordinary breaking from SHY permission:

- Blink `daf70f9ddee4b45d4f8870f574fbde61c0db4ca7`,
  [text_break_iterator.cc](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/platform/text/text_break_iterator.cc),
  SHA-256 `ea0fb1d3795561e3666de1842e20be6d1c7a65c0d92c571e678f3a2cf25b82f4`:
  lines 248–250 disable the fast SHY opportunity, but lines 310–338 independently
  consult the break-all class table before honoring that no-break fast result.
  The BA-to-AL entry is enabled. Thus this iterator can admit after-SHY before a
  letter under break-all even with SHY disabled. It does not turn SHY into a
  letter or authorize a break before SHY.
- The same revision's
  [text_break_iterator_test.cc](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/platform/text/text_break_iterator_test.cc#L368),
  SHA-256 `e18b47828913e067c7ac76fc1cc16854ee2d03619b296f8604682cd70f66783f`,
  tests removal of normal SHY opportunities when disabled. That test does not
  exercise the separate break-all branch.
- Blink's
  [line_breaker.cc](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/line_breaker.cc#L4490),
  SHA-256 `84d6fe431463f7d28423f810824b06251957b7e5ddad25a79ce8533fd3864380`,
  does not itself force hyphens:none in its break-all switch. Its whole pipeline
  is therefore not an oracle for the mandated suppression; adopt only the
  independently justified iterator distinction.
- WebKit `73aa6c89e2cb77c46184a81aec944e4ab99d114d`,
  [complete retained TextUtil.cpp](webkit-text-source-gaps-hyphenation--73aa6c89e2cb.md#textutilcpp),
  SHA-256 `ef914f2a1941915eba05dcb311a6cf6f455184a290d88a46d63198d532270232`:
  `wordBreakBehavior` returns the break-all rule before automatic hyphenation;
  `mayBreakInBetween` separately checks the previous SHY's hyphens property
  before using the next text box's ordinary break settings. This supports
  separate authorities, not universal agreement on every combined case.

Reviewed WPT at `f4febe305ac97a2aa53435bf6bf98c0a78d8128f`:

| Test | Source hash | What it establishes |
| --- | --- | --- |
| [hyphens-auto-wordbreak-breakall-crash.html](https://github.com/web-platform-tests/wpt/blob/f4febe305ac97a2aa53435bf6bf98c0a78d8128f/css/css-text/hyphens/hyphens-auto-wordbreak-breakall-crash.html) | `cfcfaeddc47e0d6d30e687d088e38ed2f6d8cb953fd66b26f7d6a537b15a87a9` | Non-crash coverage only, no SHY or mixed-owner output oracle. |
| [hyphens-span-002.html](https://github.com/web-platform-tests/wpt/blob/f4febe305ac97a2aa53435bf6bf98c0a78d8128f/css/css-text/hyphens/hyphens-span-002.html) | `a3dba7b59d59cb7a277018d4ff56e6529f2a39738f6f664acc0fc6c4d6ee6ae0` | Unstyled spans must not change automatic hyphenation of highway; no mixed-property ownership rule. |

The scoped hyphens/word-break WPT search did not yield a direct SHY plus
break-all expected-output test. This absence is not an interoperability claim.

## Selected behavior

Suppress manual/source/control and automatic hyphenation alternatives when
their actual hyphenation owner has break-all. Do not merely hide the glyph:
there is no hyphenation trial, continuation or hyphenated-line effect for that
suppressed alternative. Preserve truthful cut/resource observations.

Use the existing source SHY style, actual discretionary control style, or the
logical-preceding actual text contribution for an automatic cut. Extend that
owner's existing hyphens gate with its word-break gate. Do not replace it with
the following ordinary text box. Automatic mixed-style custody is established
Surgeist policy; extending it to this gate is explicit policy too, not a claim
that CSS fully specifies mixed-style arbitration. Wrapping permission and
resource language qualification retain their separate authorities.

Independently evaluate ordinary permission at the actual mixed cut. Adopt the
bounded Blink iterator distinction: in `a SHY b` under uniform break-all,
allow the ordinary cut **after SHY**, with no indication, while retaining the
no-break before SHY. The ordinary following-letter/class rule supplies this
permission; suppressing the discretionary alternative does not supply it.
Apply #75's following-box ordinary owner, genuine typographic-unit boundaries,
Unicode neighbor restrictions and wrapping permission. This exact source-gap
mapping is explicit Surgeist policy informed by the iterator mechanism.

Do not erase SHY and reclassify every equal-offset endpoint as adjacent letters.
Only the actual after-SHY relation qualifies in that example. In the absence
of an independent ordinary or explicit owner, the suppressed SHY contributes
no offer. Consecutive SHYs, punctuation, combining marks and glue still require
their actual rule/context evaluation. A discretionary control has no implicit
BA character and cannot gain this source-SHY permission by sharing an offset.
Preserve genuine wbr, ZWSP, forced and atomic permissions and always-visible
hyphens under their own rules, without transferring their proof to another cut.

For an otherwise permitted Latin after-SHY relation, with hyphens:manual:

| SHY owner | Following ordinary owner | Result |
| --- | --- | --- |
| BreakAll | BreakAll | Ordinary only; no hyphenation alternative. |
| BreakAll | Normal | No SHY-derived offer; unrelated independent permissions survive. |
| Normal | BreakAll | Ordinary and manual alternatives may coexist at that exact cut. |
| Normal | Normal | Manual discretionary alternative; no ordinary promotion from SHY. |

## Replace the invalid Allowed plus Auto fixture

Use genuine source `as<wbr>sociate`, Normal (also valid under KeepAll), en-US,
hyphens:auto, wrapping enabled, and no word-space transform. Author the actual
Wbr occurrence after the second letter. Its explicit opportunity remains
non-hyphenating. The complete lexical word remains associate and the existing
dictionary proposal at two letters independently supplies Auto. Select the
issued boundary after that Wbr and assert both the real None variant and Auto;
do not create or merge offers by numeric source offset.

This is supported by the current model: projection barriers include forced
breaks, disconnected text and in-flow atomics, not Wbr; automatic queries use
the complete processed word; preparation can attach automatic proposals to the
actual explicit-control endpoint. It is a legitimate authored fixture, not a
fabricated production control. An eligible actual OptionalBreak is also a
native fixture, but Wbr more clearly expresses the web-facing case.

Retain the fixture's real ordinary-versus-generated advance distinction (20
versus 30 in its existing 10-unit font). Use a later genuine issued offer as
the conditional ceiling; Normal need not offer the old break-all offset three.
AutoFitExhausted or AutoIntrinsicRefinement must not discharge the same
endpoint's genuine None variant. Assert exact Wbr/cut/resource identity so an
unrelated earlier barrier cannot accidentally satisfy the test.

Do not use transformed full-width letters as an assumed replacement: the
current resource queries processed spelling with no width normalization.
Ordinary ID behavior alone does not prove that this resource supplies Auto.

## Acceptance and scope

Test the four mixed-owner cases, both directions, actual source-after-SHY cut
identity, disabled hyphens and empty indication, and preserved independent
Wbr/ZWSP/forced/atomic/hard-hyphen offers. Test source/control distinction and
that no equal-offset promotion or inserted synthetic control occurs. Add the
Normal/KeepAll Wbr plus Auto barrier fixture and retain the separate break-all
suppression regression. Existing unforced-break joining remains intact.

Text owns eligibility and opportunity provenance; Layout consumes actual
variants and conditional evidence. No public API expansion is required by this
determination. The implementer owns RED/fix execution and contract integration.
