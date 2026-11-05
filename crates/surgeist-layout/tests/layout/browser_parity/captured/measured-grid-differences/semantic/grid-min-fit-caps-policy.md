# Grid min-content and fit-content shrink-to-fit

Policy `grid-min-fit-caps`, edition `css-grid-1-20250326-native-1`, covers
[min-content and fit-content(10px) with Hidden overflow](../tests/layout/browser_parity/html/grid/grid_span_2_min_content_fit_content_10px_indefinite_hidden.html),
[min-content and fit-content(30px) with Visible overflow](../tests/layout/browser_parity/html/grid/grid_span_2_min_content_fit_content_30px_indefinite.html),
and [min-content and fit-content(30px) with Hidden overflow](../tests/layout/browser_parity/html/grid/grid_span_2_min_content_fit_content_30px_indefinite_hidden.html).
Each source runs in both box modes and directions. The captured Ahem 10px
leaf contains two 40px runs separated by U+200B and advances 80px in full.

[CSS Grid 1 §11.5](../../../references/css-grid-1--CRD-css-grid-1-20250326--53a47980a217.md#algo-content)
supplies the intrinsic track-sizing phases. The Native min-content and
max-content candidates are 40px and 80px. The absolute root has 800px
available inline, giving a shrink-to-fit width of 80px. The Normal track
pass resolves the columns. The fixed row gives every box height 40px and
vertical position zero.

| Cap | Leaf overflow | Native tracks | Native span | Browser root | Browser tracks |
| ---: | --- | --- | ---: | ---: | --- |
| 10px | Hidden | 40px, 10px | 50px | 50px | 40px, 10px |
| 30px | Visible | 20px, 30px | 50px | 50px | 20px, 30px |
| 30px | Hidden | 40px, 30px | 70px | 70px | 40px, 30px |

For track widths A and B, the LTR inline positions of the two items and
the leaf are 0, A, and 0. The RTL positions are 80 minus A, 80 minus A
minus B, and 80 minus A minus B. The root remains at zero in both
directions. The independent
[Native Grid tests](../src/grid_tests/tracks_intrinsic/native_min_fit_content.rs)
cover intrinsic candidates, final placement, and scroll geometry in both
packet channels.

Margins, borders, padding, scroll ranges, and scrollbar reservation are
zero. Each box's local border, padding, content, scrollport, overflow, and
optimal-viewing rectangles have its extent. Hidden clips the leaf to its
span and row. The Grid and leaf have unknown named baselines. The two empty
Flex items establish named baseline absence. Scroll targets have zero
margins, no snap alignment, and Normal snap stop.

Qualification compares complete Layout geometry and DOM integer sizes in
both directions. Hidden also requires the two captured Layout scroll-range
spans. The raw comparison retains each browser difference.
