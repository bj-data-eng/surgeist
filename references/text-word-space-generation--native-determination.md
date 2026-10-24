# Separator-generated SPACE and full-width

This native determination applies to #63/#72. The owning
[transformation contract](../docs/text-layout-contract.md#text-transformations-and-correspondence)
defines execution and custody. This reference distinguishes the selected
interpretation from explicit draft text and browser qualification.

The pinned [CSS Text 4 §2.2](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#word-space-transform)
replaces eligible U+200B, genuine wbr and virtual expandable separators.
[§2.3](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#text-transform-order)
places word-space replacement after Phase I and before casing/full-width.
Full-width maps SPACE within preserved whitespace; the
[preserved definition](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#preserved-white-space)
does not expressly classify a newly generated SPACE for every collapse mode.

Select the following native interpretation: an eligible separator-generated
U+0020 survives the already completed Phase I and qualifies for subsequent
full-width conversion in every mode, including Collapse and Discard. Retain
the generation fact separately from original SPACE preservation. Do not rerun
Phase I, collapse adjacent generated spaces, or alter authored-SPACE rules.
Original text and plaintext remain unchanged.

Full-width eligibility is not Phase-II immunity. Later trimming/hanging uses
the actual emitted U+0020 or U+3000 and actual governing style. Suppress
forced-break-adjacent replacement before emission, under §2.2. This is a
source-supported native completion of an ambiguity, not standards consensus
or a claim that the draft explicitly enumerates every mode.

## Surviving Discard characters in Phase II

The [recorded #74 clarification](https://github.com/bj-data-eng/surgeist/issues/74#issuecomment-6379092206)
selects ordinary advance for surviving Discard characters. Pinned Phase II
does not enumerate Discard hanging; #74's earlier Preserve-equivalent extension
applies only to PreserveSpaces. Do not infer Preserve semantics from a generic
non-Collapse implementation branch.

Original U+3000, separator-generated U+0020 and its full-width U+3000 result
retain ordinary advance at soft, forced and block ends, with or without wrapping.
Discard supplies neither conditional nor unconditional whitespace-rule hanging
and requires no terminal/available-edge fact solely for that hanging. Independent
applicable rules remain separate. This is native policy for the source omission,
not CSSWG consensus or browser parity. Removing CSS document whitespace in Phase I
does not remove surviving other-space separators or later generated characters.

## Supporting WPT source and limits

Use WPT revision
[`69f2bba61ce7e9469ccbce53de4644a89f6b7dff`](https://github.com/web-platform-tests/wpt/tree/69f2bba61ce7e9469ccbce53de4644a89f6b7dff/css/css-text/word-space-transform).
These are static source observations, not browser results or acquired test code.

- [007](https://github.com/web-platform-tests/wpt/blob/69f2bba61ce7e9469ccbce53de4644a89f6b7dff/css/css-text/word-space-transform/word-space-transform-007.html)
  asserts that generated spaces remain distinct from neighboring spaces after
  Phase I. Its ordinary collapsing-space
  [reference](https://github.com/web-platform-tests/wpt/blob/69f2bba61ce7e9469ccbce53de4644a89f6b7dff/css/css-text/word-space-transform/reference/word-space-transform-007-ref.html)
  cannot qualify the asserted multiple gaps. Retain the processing-order
  evidence without adopting that visual oracle.
- [009](https://github.com/web-platform-tests/wpt/blob/69f2bba61ce7e9469ccbce53de4644a89f6b7dff/css/css-text/word-space-transform/word-space-transform-009.html)
  combines word-space Space and full-width under default collapsing whitespace,
  supporting generated-SPACE conversion outside preserving modes. Its
  [reference](https://github.com/web-platform-tests/wpt/blob/69f2bba61ce7e9469ccbce53de4644a89f6b7dff/css/css-text/word-space-transform/reference/word-space-transform-009-ref.html)
  also converts original collapsible SPACE and conflates adjacent original and
  generated gaps. Those outputs do not override the pinned preserved-SPACE rule
  or establish exact multiplicity.

The four immutable source objects are respectively Git blobs
`5d021684de477711b2599b3d3d38fe47198e3aca`,
`b0cb8913084fee3c86ff549fec616f19086630f9`,
`5c0df04d1dabed892d4815fad51f1a134ecce881` and
`815fe800e3de5cc663fa892284ca1947f3e115d1`. These are upstream Git object
identifiers, not raw SHA-256 captures. Independent literal native tests must
qualify actual execution, original/Phase-I custody and Phase-II behavior.
