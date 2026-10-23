# Blink formatting-unit rectangle reference

This metadata-only implementation witness supports the distinction between Text
source slots, DOM Range rectangles and caret positions. It supplements the
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
