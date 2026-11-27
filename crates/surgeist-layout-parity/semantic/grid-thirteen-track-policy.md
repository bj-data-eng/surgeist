# Grid thirteen-track intrinsic sizing

Policy `grid-thirteen-track`, edition `css-grid-1-20250326-native-1`, covers
the [Visible source](https://github.com/bj-data-eng/surgeist/blob/main/crates/surgeist-layout/tests/layout/browser_parity/html/grid/grid_span_13_most_non_flex_with_minmax_indefinite.html)
and [Hidden source](https://github.com/bj-data-eng/surgeist/blob/main/crates/surgeist-layout/tests/layout/browser_parity/html/grid/grid_span_13_most_non_flex_with_minmax_indefinite_hidden.html)
in both box modes and directions. The captured Ahem leaf consists of two
160px runs separated by U+200B and advances 320px in full.

[CSS Grid 1 §11.5.1](https://github.com/bj-data-eng/surgeist/blob/main/references/css-grid-1--CRD-css-grid-1-20250326--53a47980a217.md#distribute-extra-space)
distributes space to nonaffected tracks up to their growth limits before
affected tracks exceed their limits. The 2px to 4px track takes its remaining
2px during intrinsic distribution. The intrinsic sizes are 160px and 320px.
The absolute root accepts 320px, and the Normal pass resolves the 20% track
to 64px. The browser captures a 322px root and span.

In the Normal pass, the fixed bases total 84px after the 2px increase. The
remaining 76px of the minimum contribution distributes over seven tracks
for Visible or five tracks for Hidden. The two max-content minimum tracks
then receive the remaining max-content contribution. Binary32 widths and
logical prefixes determine physical positions. Publication rounds physical
edges independently. DOM integer sizes use the unrounded local
scroll geometry. The
[Native Grid tests](https://github.com/bj-data-eng/surgeist/blob/main/crates/surgeist-layout/src/grid_tests/tracks_intrinsic/native_thirteen_tracks.rs)
cover the intrinsic and Normal operations, including both packet channels.

The root is 80px high. The measured leaf spans all columns in the first
40px row. The 13 empty markers occupy the second 40px row. Each box has
zero margins, borders, padding, scroll ranges, and scrollbar reservation.
Hidden clips the leaf to its span and row. Qualification retains both
Native packet channels, complete Layout and DOM integer comparisons, and
both Layout scroll-range spans on the Hidden leaf.
