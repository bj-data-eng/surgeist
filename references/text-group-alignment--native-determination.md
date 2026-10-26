# Native text-group alignment

This determination owns the native group-alignment policies required by
[#1478](https://github.com/bj-data-eng/surgeist/issues/1478) under
[#68](https://github.com/bj-data-eng/surgeist/issues/68). It specifies the
Text/Layout composition contract; it does not certify feature completion or
table-character alignment. Implementations and tests must preserve these
decisions unless their owning requirement is explicitly revised.

## Source and operation order

The pinned [Text4 §7.6](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#text-group-align-property)
requires common padding before ordinary text alignment. Its Appendix A instead
lists justification, alignment and then group alignment. These orders conflict.
The native profile adopts §7.6's explicit operation: measure natural remaining
space, resolve group padding, then justify and align within the reduced content
frame. The [CSSWG proposal](https://github.com/w3c/csswg-drafts/issues/1975#issuecomment-378765521)
describes leaving the longest line natural and justifying shorter lines to its
width; the [resolution](https://github.com/w3c/csswg-drafts/issues/1975#issuecomment-380123672)
adopts the outline with open questions. Neither establishes the nested or
negative-remainder policies below.

Pinned WebKit `73aa6c89e2cb77c46184a81aec944e4ab99d114d` has flagged property
parsing/storage, but the directly contained inline-formatting files inspected
provide no group-alignment algorithm. Pinned Blink
`daf70f9ddee4b45d4f8870f574fbde61c0db4ca7` has no property entry in
`css_properties.json5`. Parsing support and basic single-container WPT examples
do not establish justification, nested-owner or mixed-direction behavior.

## Membership and residual composition

A group's members are its actual descendant in-flow formatted line boxes in
the same block formatting context. Skip independent-context interiors, including
floats and out-of-flow boxes. An atomic inline's exterior remains real content
of its parent line. Exclude phantom/nonformatted completions; include genuine
Forced formatted empty lines. Use actual dispatch and normalized context facts,
not source ancestry or overflow alone, to determine boundaries.

Process non-`none` group owners from outer to inner. For each member line `l`,
retain its original usable alignment capacity `W_l - indent_l`, natural
pre-justification alignment advance `N_l`, and the total padding already assigned
by ancestors. The owner's new padding amount is:

```text
C_l = W_l - indent_l - ancestor_line_left_l - ancestor_line_right_l
P = max(0, min over actual members of (C_l - N_l))
```

An empty member set adds nothing. Apply `P` once to every member on the selected
side or sides. A child `none` adds no operation and does not erase ancestor
effects. Independent contexts form separate collections. Disjoint sibling
traversal order cannot affect the result. Derive each evaluation from immutable
natural measurements, never from its previous expanded or padded output.

Outer-to-inner residual composition is explicit Surgeist policy. Independent
full-frame reductions could spend the same slack twice; a child override or
descendant exclusion would discard the ancestor's specified membership.

If any member has a negative remainder, this owner adds zero padding. Clamp
only the new amount `P`; preserve signed advances, indentation, overflow and
ancestor insets. A descendant subset may still have positive residual slack.
This nonnegative reduction is explicit policy for a source gap: negative padding
would enlarge the content frame instead of reducing it. Ordinary post-
justification start-overflow behavior remains operative.

## Per-line sides

Resolve sides individually from each participating line's actual line-box
direction and writing axes:

| Group value | Added padding |
| --- | --- |
| `none` | No new operation |
| `start` | `P` at that line's end |
| `end` | `P` at that line's start |
| `left` | `P` at line-right |
| `right` | `P` at line-left |
| `center` | `P / 2` at each side |

Under Plaintext, retain the authentic resolved direction, including the accepted
preceding-line/containing fallback. Under normal bidi, use the line's containing
block direction. Body glyph levels do not supply this alignment basis. The
[Text4 line-box rule](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#bidi-linebox)
defines line sides generally without expressly listing this property. Applying
those sides to each member completes the mixed-direction case as native policy,
without an interoperability claim. Logical `start` can select different physical
edges for different members; `left`/`right` select line-relative physical sides.

## Phase ownership and publication

Layout owns group membership, natural-slack collection, padding resolution,
spacing allocation, numeric placement and coherent replay/publication. Text
owns the normalized alignment-only finalization frame and the one finalizer
that positions its glyphs, boxes, controls, fragments and interaction geometry.
Consumers own their direct adapters.

Keep wrapping, source/termination custody, original float bands, indentation,
tab grids and natural Phase-II/hanging measurements unchanged. Text's outer
`available_inline` and metric frame retain their original meaning. Resolve all
group insets before allocating genuine justification against the reduced
content capacity. Do not reuse full-frame expansion amounts, move only painted
glyphs or invent spacing opportunities for numeric lines.

The dependency basis includes actual owners and membership, natural measures,
bands, indentation, effective directions and accumulated insets. A changed
limiting sibling invalidates its peers. Trial float/measurement state contributes
only when accepted. Verify convergence before publishing; failure preserves
prior immutable output. Missing required context or unrepresentable consumed
geometry must fail precisely rather than inventing source, metrics or clamps.

## Independent acceptance examples

These are derived expectations, not claims of executed measurements. Use genuine
numeric boxes or registered Ahem text with real spacing/termination facts.

For width100 and natural line advances60/40 in LTR:

| Group | Ordinary alignment | Insets left/right | Content starts |
| --- | --- | --- | --- |
| `none` | center | 0/0 | 20/30 |
| `left` | center | 0/40 | 0/10 |
| `right` | center | 40/0 | 40/50 |
| `center` | start | 20/20 | 20/20 |
| `center` | justified with eligible spaces | 20/20 | Both intervals20..80 |

The justified lines expand by0/20, not40/60. Use effective last-line alignment
that actually requests justification. Bands100/80 with natural60/50 give `P=30`
and, for center plus start, local content starts15/15 without moving the float.

For nested groups with width100, a parent center group includes an outside
line80 and child lines60/40. The parent adds10/10. A child `end` group sees
capacity80 and adds20 at each child's line-start: total LTR insets30/10,
start-aligned child content at30, outside content at10. A child containing the
parent's limiting80 line adds zero. Widths120/40 give zero new group padding;
ordinary start overflow and the short line's ordinary alignment remain.

Qualify both scalars, supported writing modes, opposing descendant/Plaintext
directions, nested/none/independent owners, real Forced versus phantom outcomes,
signed indent and margins, tabs, hanging/trailing whitespace, source/visual
identity, unchanged cuts and nominal metrics, actual atomics/endpoints, and
warm/cold changed-limiter output. Completion requires independent composed
review of these behaviors; this determination alone establishes no test result.
