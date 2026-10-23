# Blink formatting-unit rectangle reference

This metadata-only implementation witness supports the distinction between Text
source slots, DOM Range rectangles, caret positions and Range-unit advance
allocation. It supplements the
[selected Range policy](../docs/text-layout-contract.md#nonempty-text-range-fragments)
without supplying a normative zero-control endpoint rule.

- Upstream: [Chromium Blink](https://www.chromium.org/blink/).
- Version: Chrome for Testing `149.0.7827.115`; resolved Chromium commit
  `daf70f9ddee4b45d4f8870f574fbde61c0db4ca7`.
- Original path: `third_party/blink/renderer/core/layout/inline/fragment_item.cc`.
- [Immutable source](https://chromium.googlesource.com/chromium/src/+/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/fragment_item.cc).
- Retrieved: 2026-10-22 from official Gitiles TEXT, base64-decoded.
- Complete original: 50,492 bytes; SHA-256
  `8e04daf333796b99e850bf46e1ec6d3d64a2dfc2e09bd28ecd5c9ac0367913a6`;
  Git blob `089ba6bfb4fa74c4f117ad39b2005e92bcd944a9`.
- Representation: source identity, bounded locators and factual synopsis only;
  no source excerpt is redistributed, compiled or ingested.
- Attribution: copyright 2019 The Chromium Authors; BSD-style project license.
  The pinned root license is byte-identical to the existing
  [Chromium license](../licenses/chromium/LICENSE.txt), SHA-256
  `368cca1106be99d39ecd32a38d8305585d802a475effb66380b91ffc9bcf709b`.
  See [reference attribution](../licenses/chromium/NOTICE.md#formatting-unit-range-reference).

## Bounded observations

[`FragmentItem::LocalRect`, original lines 1201–1233](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/fragment_item.cc#L1201-L1233)
uses whole-fragment dimensions when requested offsets exactly match the stored
item. Otherwise it obtains offset positions through `LineLeftAndRightForOffsets`
and maps them according to writing mode. Its dimensions account for SVG where
applicable.

[`PositionForPointInText`, original lines 1258–1283](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/fragment_item.cc#L1258-L1283)
is a separate point-to-caret operation with conditional bidi hit-test adjustment.
It is not a universal Range endpoint rule.

The authenticated PDI-only browser observation identifies the actual DOM Text
node and selected offsets, but not Blink's internal item offsets or the branch
taken by `LocalRect`. This witness does not attribute that branch to the PDI or
claim a complete trace of the DOM Range caller. The retained CSSOM View edition
remains the normative source; Surgeist's explicit policy addresses its missing
zero-width formatting-unit endpoint convention.

## Range-unit advance allocation

- Additional original path:
  `third_party/blink/renderer/platform/fonts/shaping/shape_result.cc`.
- [Immutable source](https://chromium.googlesource.com/chromium/src/+/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/platform/fonts/shaping/shape_result.cc).
- Retrieved: 2026-10-23 UTC from official Gitiles TEXT, base64-decoded;
  92,478 bytes, 2,505 lines; SHA-256
  `fc66a4c3f0cb927b9a24d2c73e5780eea2e87de53eb373c11de145e84227caa5`;
  Git blob `e80c6c3fa919b9965af854d48e6bd4400436327f`, verified against Gitiles.
- Attribution: copyright (c) 2012 Google Inc.; copyright (C) 2013 BlackBerry
  Limited. The [exact file-level three-clause BSD notice](../licenses/chromium/shape-result-cc-range-license-header.txt)
  remains distinct from the Chromium project license. No source excerpt is
  redistributed, compiled or ingested.

[`FragmentItem::LineLeftAndRightForOffsets`, lines 1132–1164](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/core/layout/inline/fragment_item.cc#L1132-L1164)
uses the shaped result's offset-position operation with start/end adjustments
on the rectangle path. [`ShapeResult::CaretPositionForOffset`, lines 709–714](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/platform/fonts/shaping/shape_result.cc#L709-L714)
first establishes grapheme information. [`XPositionForOffset`, lines 202–331](https://github.com/chromium/chromium/blob/daf70f9ddee4b45d4f8870f574fbde61c0db4ca7/third_party/blink/renderer/platform/fonts/shaping/shape_result.cc#L202-L331),
particularly 278–309, allocates a glyph sequence's advance by its grapheme count
and accounts for direction before adjusting endpoints within a grapheme.
This supports distinguishing Range units from provider clusters; it does not
make equal shares a CSS requirement or prove all scripts and profiles use them.
Surgeist's contract separately preserves already-owned spacing adjustments.
