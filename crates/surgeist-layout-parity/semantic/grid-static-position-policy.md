# Grid static-position qualification

Policy `grid-absolute-child-content-edges`, edition
`css-grid-1-20250326-native-1`, covers the original
[border-and-padding alignment](../tests/layout/browser_parity/html/grid/grid_absolute_layout_within_border_static.html)
and [resolved inset](../tests/layout/browser_parity/html/grid/grid_absolute_resolved_insets.html)
sources in both box modes and directions.

The [Grid 1 static-position rule](../../../references/css-grid-1--CRD-css-grid-1-20250326--53a47980a217.md#static-position)
aligns a child with two auto insets in the parent's used content rectangle on
that axis. This rectangle uses the final allocated content edges after border,
padding and scrollbar reservations. Static alignment treats auto margins as
zero. Explicit insets and percentage sizing use the actual containing rectangle
selected by [Grid 1 §9.1](../../../references/css-grid-1--CRD-css-grid-1-20250326--53a47980a217.md#abspos-items).

[Positioned Layout 3 §3.5.3](https://drafts.csswg.org/css-position-3/#staticpos-rect)
still gives the grid area precedence when the grid establishes the containing
block. Layout follows the Grid-specific rule and the
[CSSWG resolution on containing-block independence](https://github.com/w3c/csswg-drafts/issues/7661#issuecomment-1591609408).
The [Chromium compatibility report](https://issues.chromium.org/issues/40272496)
records deployment concerns. The
[pinned Blink capture implementation](https://github.com/chromium/chromium/blob/149.0.7827.115/third_party/blink/renderer/core/layout/grid/grid_layout_algorithm.cc#L2566)
uses the containing grid area for the static position when the grid establishes
the containing block.

The first source retains eight coordinate differences per variant. The 50px
boxes align against the content edges 20px inside the outer box, with the
authored 10px margin applied to the last two children. The second source
retains four coordinate differences per variant, two for each zero-size
auto-inset child. Its content start is 35px from the corresponding outer start
edge. The 15px classic gutters are on the logical end sides in this capture.
The remaining explicit-inset rows are equality controls.
