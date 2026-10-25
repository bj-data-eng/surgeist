# Anywhere: hyphenation custody and atomic glue precedence

Source determination for [#65](https://github.com/bj-data-eng/surgeist/issues/65),
25 October 2026, at `75fa2b242ff386f16925b84d40b86498a2e7a5ad`.
This resolves two native LineBreak questions, without implementation or API
changes. No builds or browser/runtime tests were executed.

## Sources and their limits

The current [CSS Text 4 draft](https://drafts.csswg.org/css-text-4/), checked
25 October, still identifies its 14 August 2026 edition. The complete
[retained reference](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md)
contains Anywhere, hyphenation ownership, Unicode/TCU restrictions, atomic
compatibility and line-breaking scope. Anywhere supplies ordinary opportunities
at typographic-unit edges, overrides specified text glue/word-break prohibitions
and excludes hyphenation. The atomic clause separately prohibits non-NBSP
GL/WJ/ZWJ adjacency. Boundary property ownership remains incompletely specified.

[CSSWG #8972's adopted resolution](https://github.com/w3c/csswg-drafts/issues/8972#issuecomment-1646217049)
gives non-NBSP Unicode breaking controls precedence over atomic compatibility
breaks. It does not discuss Anywhere's interaction with that requirement.
[CSSWG #4949](https://github.com/w3c/csswg-drafts/issues/4949#issuecomment-624705207)
resolved to develop a separate wrap-as control for treating atomics as text,
with details to follow. It did not adopt a rule turning every non-Auto
line-break value into atomic text emulation. Neither resolution establishes
mixed-style SHY suppression by a neighboring Anywhere box.

The existing [#75 determination](https://github.com/bj-data-eng/surgeist/issues/75)
adopts following-box ordinary ownership and either-side Anywhere for text/text
relations, preserving disappearing-character, hyphenation and atomic owners.
The [BreakAll determination](text-break-all-hyphenation--determination-20261024.md)
separates actual-owner hyphenation exclusion from independent ordinary offers.

### Pinned browser mechanisms

WebKit `73aa6c89e2cb77c46184a81aec944e4ab99d114d` is retained as
[complete Markdown source](webkit-text-source-gaps-hyphenation--73aa6c89e2cb.md).

- `InlineFormattingUtils.cpp`, SHA-256
  `b5bbeaa10f6b38b6e9757afa7d7e2d93f9c555245561323a7e89685745ccd527`:
  `isAtSoftWrapOpportunity` checks either-side Anywhere inside its text/text
  branch. Atomics use a separate branch. Its broad atomic return does not
  establish compliance with the current non-NBSP glue restriction; do not
  adopt that return as an exception to #8972.
- `TextUtil.cpp`, SHA-256
  `ef914f2a1941915eba05dcb311a6cf6f455184a290d88a46d63198d532270232`:
  `wordBreakBehavior` returns the Anywhere strategy before considering automatic
  hyphenation. `mayBreakInBetween` separately examines SHY's preceding style
  and the following text's ordinary rules. These mechanisms support distinct
  owners but do not prove the complete mixed-Anywhere alternative inventory.

Blink `daf70f9ddee4b45d4f8870f574fbde61c0db4ca7`,
[line_breaker.cc](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/line_breaker.cc),
SHA-256 `84d6fe431463f7d28423f810824b06251957b7e5ddad25a79ce8533fd3864380`:
`SetCurrentStyleForce` selects character breaking for Anywhere without itself
forcing hyphens:none; `CanBreakAfter` has a broad true return before ordinary
atomic inlines. These are not sufficient normative or interoperable oracles
for either disputed interaction. Do not import their omissions.

### Browser tests

Reviewed WPT `f4febe305ac97a2aa53435bf6bf98c0a78d8128f`, directory
[css/css-text/line-breaking](https://github.com/web-platform-tests/wpt/tree/f4febe305ac97a2aa53435bf6bf98c0a78d8128f/css/css-text/line-breaking).
The `line-breaking-atomic-001` through `019` tests establish baseline atomic
compatibility, NBSP exceptions, non-NBSP glue and punctuation/KeepAll behavior.
They do not set line-break:Anywhere, so they do not settle the override question.
Representative complete-file hashes:

| File | SHA-256 | Expected baseline |
| --- | --- | --- |
| line-breaking-atomic-001.html | `ba655f6bd03bbdf3e7ebcc95b8584511bc505ede720e3f29e8679d6b7357a6f1` | Break at NBSP before atomic. |
| line-breaking-atomic-002.html | `8e4608921de596c2186b95bc6512eb409e3df88b0457058f504909ada4f63034` | Break at NBSP after atomic. |
| line-breaking-atomic-003.html | `57329f5debc62dfc928c3061eece50adfcb2708501ea2e102977f0766cc0e9e2` | No break at preceding NNBSP. |
| line-breaking-atomic-005.html | `dac18b2b374b3e361f522c19e35f2aaa9d6049c541f3d3626f3a9df2e4eea45e` | No break at following WJ. |
| line-breaking-atomic-010.html | `ef8654b68ffb15d5b8f2f20fab42baee6c769efd0a741a3124b43b3e2024b8ae` | No break at following ZWJ. |
| line-breaking-atomic-018.html | `78a38c735530c30f513f591164a6e9f9814df94db44ae9c6cc9c029821202362` | No break at preceding FIGURE SPACE. |

These are reviewed test expectations, not executed pass results. The policies
below explicitly complete the source gaps; no browser consensus is claimed.

## Decision 1: hyphenation exclusion follows the actual owner

Apply line-break:Anywhere's exclusion to the actual hyphenation owner, exactly
as for its hyphens and BreakAll gates. Source SHY uses its own containing style;
an explicit discretionary control uses its captured style; Auto uses the
logical-preceding actual text contribution under the existing native policy.
Do not transfer that authority to the following ordinary text box or to a
neighboring inline wrapper. Nearest-common-ancestor wrapping permission and
resource language qualification remain separate gates.

For source `a + SHY + b`, at the authentic after-SHY cut:

| SHY line-break | Following text line-break | Otherwise eligible result |
| --- | --- | --- |
| Normal | Anywhere | Ordinary None variant and manual alternative coexist. |
| Anywhere | Normal | Ordinary None variant; manual alternative excluded. |
| Anywhere | Anywhere | Ordinary None variant; manual alternative excluded. |
| Normal | Normal | Existing manual eligibility and independent ordinary rules. |

This is explicit native mixed-owner policy. The first row does not hyphenate
an Anywhere-owned SHY: it retains a separately owned alternative. The ordinary
variant never inserts an indication. Reject the alternative interpretation
that any neighboring Anywhere suppresses every owner's alternative: it changes
established custody without a source mandate and makes independent permissions
improperly control one another.

An Anywhere-owned explicit discretionary control produces no hyphenation
alternative, even an empty one. A Normal-owned control may retain an eligible
alternative alongside a separately authenticated Anywhere text/text offer.
The control is not itself a text TCU or an Anywhere donor. Likewise, Auto with
a Normal preceding owner can coexist with following-Anywhere ordinary permission;
Auto with an Anywhere preceding owner is excluded even if following text is Normal.

An empty indication is still a hyphenation alternative, with its identity and
line-history semantics. Do not convert a suppressed empty alternative to None.
Retain factual source/resource/cut observations while exposing no excluded
alternative or continuation. Do not move SHY to another source endpoint or
promote every mixed cut sharing its numeric offset. Anywhere may independently
admit other real typography-safe edges, including before SHY when eligible;
that is its own authority, not a relocated SHY opportunity.

## Decision 2: Anywhere does not override atomic non-NBSP glue

Keep the atomic clause as the more specific owner at text/atomic and atomic/text
relations. An adjacent text style's Anywhere does not override its non-NBSP
GL/WJ/ZWJ prohibition, on either logical side. An atomic is not made into a TCU,
nor assigned an invented text style, by this operation. This precedence choice
is explicit bounded Surgeist policy: #8972 establishes the baseline restriction,
but does not explicitly settle its interaction with Anywhere.

For an actual in-flow atomic and its influencing adjacent text character:

| Adjacent character | NCA Wrap, text Normal or Anywhere | NCA NoWrap |
| --- | --- | --- |
| U+00A0 NBSP | Atomic compatibility opportunity on either side. | No atomic soft offer. |
| Other GL, including U+2007/U+202F | No atomic soft offer. | No atomic soft offer. |
| WJ, including U+2060/U+FEFF | No atomic soft offer. | No atomic soft offer. |
| ZWJ U+200D | No atomic soft offer. | No atomic soft offer. |
| Ordinary letter/punctuation | Existing atomic compatibility opportunity. | No atomic soft offer. |

Inside adjacent text, Anywhere still overrides the specified glue restrictions
at eligible text/text TCU edges. Do not extend the atomic veto to unrelated
text/text cuts, or use a text/text provider result across an atomic placeholder.
Neither an atomic's descendants nor transparent wrappers supply a donor style.
Use the actual mixed relation and existing influencing-neighbor traversal;
never scan through genuine text to select a more convenient neighbor.

An explicitly authored Wbr or forced control has its own exact occurrence and
authority. Denying an atomic-created offer does not erase that separately
qualified control, but its existence at a numeric offset does not authorize
all other cuts there. Out-of-flow and empty metadata create no permission.
Actual atomic breaks remain at their margin edges, with unchanged geometry.

## Shared precedence and acceptance

Forced breaks remain forced regardless of Anywhere or nowrap. Preserve mandatory
separator ordering (including no soft offer immediately before a hard separator,
and intact CRLF); Anywhere does not create duplicate soft alternatives at a
required cut. Never split a genuine typographic unit or an atomic box. NoWrap
continues to gate soft wrapping under each cause's existing authority, including
NCA authority for ordinary text/text and atomic relations. Disappearing-character
and explicit-control wrapping owners are not replaced by a blanket NCA test.

Test all four SHY matrix rows, both directions and nested styles; Normal/Anywhere
discretionary-control and Auto owners; empty and nonempty alternatives; precise
source/mixed-boundary identity and unaffected required/explicit offers. For
eligible coexistence assert that ordinary None has no indication/continuation
and remains an independent conditional-proof obligation.

Test each atomic matrix row in both logical orders and directions, differing
text/ancestor/wrapper styles, NCA Wrap and NoWrap, intervening transparent
metadata, and source U+FFFC versus an actual atomic participant. Contrast
GL/WJ/ZWJ text/text Anywhere admission with atomic rejection. Include combining
and emoji-ZWJ indivisibility, CRLF/hard separators, explicit Wbr and forced
controls, NBSP exception, margin-edge geometry and retained source-free identity.

Text owns classification, permission provenance and eligibility. Layout consumes
the issued inventory and real atomic geometry. No additional atomic style/API
or emulation mode is required. The implementation coordinator owns RED/fix,
contract integration and acceptance execution; the independent Unicode-context
fix need not wait on these decisions.
