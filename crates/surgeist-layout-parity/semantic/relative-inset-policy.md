# Final-auto relative inset qualification

Policy `relative-insets-final-measured-basis`, edition
`css-position-3-20251007-native-1`, covers the original
[`block_inset_percentage` source](https://github.com/bj-data-eng/surgeist/blob/main/crates/surgeist-layout/tests/layout/browser_parity/html/block/block_inset_percentage.html)
in both box modes and directions.

[CSS Positioned Layout 3 §§3.1 and 3.3](https://www.w3.org/TR/2025/WD-css-position-3-20251007/#insets)
define corresponding-axis percentages and visual relative offsets. Layout
uses the final measured normal containing-block content size before corner
rounding, as specified in the
[Layout placement contract](https://github.com/bj-data-eng/surgeist/blob/main/docs/text-layout-contract.md#canonical-boxes-and-coherent-line-placement).
[CSSWG #9353](https://github.com/w3c/csswg-drafts/issues/9353) records the
open discrepancy for block-axis percentages against an initially auto size.
The pinned browser capture retains the positions with zero block-axis offset.

The parent is 50px wide. Its three 10px children occupy normal y positions
0px, 10px and 20px and establish a 30px block basis. Top 4% resolves to 1.2px
and bottom 8% resolves to 2.4px. Top wins the overconstrained block axis
for the third child. Layout publishes y positions 1px, 8px and 21px after
rounding. The browser records 0px, 10px and 20px. The horizontal offsets
use the 50px basis. The first two positions are 1px and −3px, and the third
is 1px in LTR and −3px in RTL. These horizontal observations and every box
extent are equality controls.
