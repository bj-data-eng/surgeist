# Grid min-content and fit-content qualification

Policy `grid-min-fit-10`, edition `css-grid-1-20250326-native-1`, covers the
[visible min-content and fit-content(10px) fixture](https://github.com/bj-data-eng/surgeist/blob/main/crates/surgeist-layout/tests/layout/browser_parity/html/grid/grid_span_2_min_content_fit_content_10px_indefinite.html)
in both box modes and directions. It requires the captured Ahem 10px leaf with
two four-character runs separated by U+200B. Each run advances 40px, and the
whole leaf advances 80px.

[CSS Grid 1 §11.5](https://github.com/bj-data-eng/surgeist/blob/main/references/css-grid-1--CRD-css-grid-1-20250326--53a47980a217.md#algo-content)
supplies the intrinsic track-sizing phases. The min-content and max-content
candidates are 40px and 80px. The absolute root's available inline extent is
800px, giving a shrink-to-fit width of 80px. The final Normal track pass
assigns 30px to the first track and 10px to the second. One fixed 40px row
sets each box's height.

| Source path | Source index | LTR x | RTL x | Width |
| --- | ---: | ---: | ---: | ---: |
| `/` | 0 | 0 | 0 | 80 |
| `/0` | 0 | 0 | 50 | 30 |
| `/1` | 1 | 30 | 40 | 10 |
| `/2` | 2 | 0 | 40 | 40 |

Both packet channels use these integral positions and sizes. Margins, borders,
and padding are zero. The Grid and analytic text leaf have unknown named
baselines. The two empty Flex items establish named baseline absence. Each
box has local border, padding, content, scrollport, overflow, and optimal-viewing
rectangles with the same extent. Visible overflow yields zero scroll ranges
and scrollbar reservation. The target has zero scroll margins, no snap
alignment, and Normal snap stop. The independent
[Native Grid tests](https://github.com/bj-data-eng/surgeist/blob/main/crates/surgeist-layout/src/grid_tests/tracks_intrinsic/native_min_fit_content.rs)
cover the intrinsic candidates, the final pass, and scroll geometry.

The captured browser uses a 40px root and two 20px tracks. Qualification
requires the complete Layout and DOM integer observations and both Native
packet channels. The raw result retains the exact browser differences.
