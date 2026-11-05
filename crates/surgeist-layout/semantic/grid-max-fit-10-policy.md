# Grid max-content and fit-content(10px)

Policy `grid-max-fit-10`, edition `css-grid-1-20250326-native-1`, covers
the [visible max-content and fit-content(10px) source](https://github.com/bj-data-eng/surgeist/blob/main/crates/surgeist-layout/tests/layout/browser_parity/html/grid/grid_span_2_max_content_fit_content_10px_indefinite.html)
in both box modes and directions. The Ahem leaf consists of two 40px runs
separated by U+200B and advances 80px in full.

[CSS Grid 1 §11.5](https://github.com/bj-data-eng/surgeist/blob/main/references/css-grid-1--CRD-css-grid-1-20250326--53a47980a217.md#algo-content)
orders intrinsic minimum, content-based minimum, and max-content minimum
growth. The 40px minimum distributes across both tracks, with 10px going
to the capped fit-content track. The max-content minimum then adds 40px
to the first track, giving 70px and 10px. Both intrinsic candidates and
the final absolute root width are 80px. The browser captures tracks of
60px and 20px within the same 80px root.

Each box is 40px high with zero margins, borders, and padding. The leaf
spans both tracks. LTR places the second item at x=70px, and RTL places
the first item at x=10px. Root and leaf start at zero. The Grid and leaf
have unknown named baselines. The empty Flex items establish their
absence. The complete Layout and DOM integer comparisons and both Native
packet channels are required. The raw comparison retains every difference.
