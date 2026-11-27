# Sideways Flex collapse qualification

The Layout policy `flexbox-collapse-two-round-struts`, edition
`css-flexbox-1-20251014-native-1`, governs the collapsed subtree in
[the sideways composition fixture](../tests/layout/browser_parity/html/flex/fri07_flex_composition.html).
[Flexbox §4.4 and the line algorithm](../../../references/css-flexbox-1--CRD-css-flexbox-1-20251014--b0e3b097bc6d.md)
define the collapse behavior owned by [#245](https://github.com/bj-data-eng/surgeist/issues/245).

The first round distributes 18 px of cross space across three lines and
records a 36 px strut. Replay starts a line at the collapsed slot because
the 116 px occupied by the preceding line and the 6 px gap exceed the
120 px main extent. The replay stretches each line by 27 px. Native
publishes zero box geometry for the collapsed subtree and omits its
scroll geometry.

The exact comparison qualifies the orthogonal child's y position,
the collapsed box's y, width and height, its two scroll spans against
Native absence, and its descendant's x, width and height. The
[scrollbar edge policy](flex-composition-scrollbar-policy.md) owns the
visible scroller's child offset.
