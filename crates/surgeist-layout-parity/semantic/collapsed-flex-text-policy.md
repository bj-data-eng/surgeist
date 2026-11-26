# Collapsed Flex text qualification

Policy `flexbox-collapse-text-strut`, edition `css-flexbox-1-20251014-native-1`,
covers the four box-mode and direction variants of the original
[collapsed Flex source](https://github.com/bj-data-eng/surgeist/blob/main/crates/surgeist-layout/tests/layout/browser_parity/html/flex/fri07_collapsed_strut_single_line.html).
The comparison uses a Normal absolute-root-local request and the original
10 px Ahem WOFF2 face embedded in the base stylesheet.

[Flexbox §4.4 and §9.4](https://github.com/bj-data-eng/surgeist/blob/main/references/css-flexbox-1--CRD-css-flexbox-1-20251014--b0e3b097bc6d.md)
define the two-round collapse algorithm. The 180 px container has one
first-round line with child cross sizes 20, 40, 25 and 60 px. After
restarting line collection, the 60 px collapsed strut establishes the
line height and the two visible items occupy 60 px in the main axis.
RTL mirrors the visible item intervals within 180 px.

Each visible source box contains a 40 px anonymous Flex item. Each
four-letter word advances 40 px because all four Ahem advances are
1,000 font units at 1,000 units per em. The 10 px line has ascent
8 px, descent 2 px and zero leading. The two visible source boxes
share the physical baseline at y=8. Each source range contributes
four 10×10 px fragments in its anonymous item's local frame. The
anonymous item starts at x=0 in LTR and x=−10 in RTL. The text leaf
starts at (0,0) in that item. Physical baseline alignment leaves
named baseline facts Unknown.

| Source path | LTR x | RTL x | Border size | Content size |
| --- | ---: | ---: | --- | --- |
| `/` | 0 | 0 | 180×60 | 180×60 |
| `/0` | 0 | 150 | 30×20 | 40×20 |
| `/1` | 0 | 0 | 0×0 | 0×0 |
| `/2` | 30 | 120 | 30×25 | 40×25 |
| `/3` | 0 | 0 | 0×0 | 0×0 |

Both packet channels carry all original boxes, anonymous items and
text leaves. Geometry has integral coordinates and the channels
agree. Every box edge is zero. Collapsed boxes, their anonymous
items and their text leaves retain their source indices, zero
geometry and absent scroll geometry. Visible text leaves have
40×10 px size and content size with absent scroll geometry.

The visible box scrollports match their local border, padding and
content boxes. The root overflow equals its 180×60 px box. Each
visible source child's overflow is 40 px wide and begins at x=0
in LTR or x=−10 in RTL. Its anonymous item's overflow equals
its own 40 px wide box. Visible overflow yields zero physical
ranges, no clips or gutters, and zero scrollbar reservation.
Scroll padding and target margins are zero. Target flow matches
box flow, snap alignments and snap type are None, and snap stop
is Normal.

The policy binds exact browser and Native difference pairs for all
five original elements and full packet facts for all 13 Native
participants. The pinned browser retains nonzero collapsed geometry,
which also displaces the second visible item. Each variant has seven
Layout differences and eight DOM size differences. The raw verdict
reports all 15 differences. The qualified verdict records the Native
policy and its complete observation set. Input drift, packet drift or
changed difference pairs require requalification.
