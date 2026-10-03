Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Grid Layout Module Level 3](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/).

Original copyright notice (from the matching exact HTML edition): Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Grid Layout Module Level 3

Source snapshot: https://www.w3.org/TR/2026/WD-css-grid-3-20260121/

Snapshot SHA-256: 56617abd29a30ec0552b33864e706e0f1422fdea34252336cce89cebe55ad290

Conversion: offline format conversion of the exact stored extracted-text; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- This stored input is an already extracted plain-text witness. It has no recoverable original HTML structure. The complete source text is preserved in a fenced block; no summary or specification regeneration was performed.

---

```text
CSS Grid Layout Module Level 3
CSS Grid Layout Module Level 3
W3C Working Draft
,
21 January 2026
More details about this document
This version:
https://www.w3.org/TR/2026/WD-css-grid-3-20260121/
Latest published version:
https://www.w3.org/TR/css-grid-3/
Editor's Draft:
https://drafts.csswg.org/css-grid-3/
History:
https://www.w3.org/standards/history/css-grid-3/
Feedback:
CSSWG Issues Repository
Inline In Spec
Editors:
Tab Atkins Jr.
(
Google
)
Elika J. Etemad / fantasai
(
Apple
)
Jen Simmons
(
Apple
)
Brandon Stewart
(
Apple
)
Former Editor:
Mats Palmgren
(
Mozilla
)
Suggest an Edit for this Spec:
GitHub Editor
Copyright
© 2026
World Wide Web Consortium
.
W3C
®
liability
,
trademark
and
permissive document license
rules apply.
Abstract
This module introduces a one-dimensional grid layout mode for
CSS Grid
containers.
CSS
is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.
Status of this document
This section describes the status of this document at the time of its publication. A list of current W3C publications and the latest revision of this technical report can be found in the
W3C standards and drafts index.
This document was published by the
CSS Working Group
as a
Working Draft
using the
Recommendation track
. Publication as a Working Draft does not imply endorsement by
W3C
and its Members.
This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.
Please send feedback by
filing issues in GitHub
(preferred), including the spec code “css-grid” in the title, like this: “[css-grid]
…summary of comment…
”. All issues and comments are
archived
. Alternately, feedback can be sent to the (
archived
) public mailing list
www-style@w3.org
.
This document is governed by the
18 August 2025 W3C Process Document
.
This document was produced by a group operating under the
W3C Patent Policy
. W3C maintains a
public list of any patent disclosures
made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains
Essential Claim(s)
must disclose the information in accordance with
section 6 of the W3C Patent Policy
.
Table of Contents
1
Introduction
1.1
Background and Motivation
1.1.1
Waterfall Layout with Auto-placed Items
1.1.2
One-dimensional Grid Layout
1.2
Value Definitions
2
Grid Lanes Layout Model
2.1
Reordering and Accessibility
2.2
Establishing Grid Lanes Layout
2.3
Orienting Grid Lanes Layout
3
Grid Lanes Track Specification
3.1
Declaring Grid Lanes Track Templates: the
grid-template-*
properties
3.1.1
Intrinsic Tracks and repeat()
3.2
Subgrids
3.3
Track Repetition: the
repeat()
notation
3.3.1
repeat(auto-fit)
3.4
Grid Axis Track Sizing
3.4.1
Subgrid Item Contributions
3.4.2
Optimized Track Sizing
4
Grid Lanes Item Placement
4.1
Specifying Grid Axis Item Placement: the
grid-column-*
and
grid-row-*
properties
4.2
Placement Precision: the
flow-tolerance
property
4.3
Dense Placement: the
dense
keyword
4.4
Grid Lanes Layout and Placement Algorithm
4.4.1
Containing Block
4.4.2
Placement and Writing Modes
5
Sizing Grid Containers
6
Alignment and Spacing
6.1
Gutters: the
row-gap
,
column-gap
, and
gap
properties
6.2
Grid-axis Alignment: the
align-content
/
justify-content
,
align-self
/
justify-self
, and
align-items
/
justify-items
properties
6.3
Stacking-axis Content Distribution: the
align-content
/
justify-content
properties
6.4
Stacking-axis Self Alignment: the
align-self
/
justify-self
and
align-items
/
justify-items
properties
6.5
Baseline Alignment
7
Fragmentation
7.1
Fragmentation in the stacking axis
7.2
Fragmentation in the Grid Axis
8
Absolute Positioning
9
Graceful Degradation
Appendix A: Generic Layout Item Flow Controls: the
item-*
properties
Item Flow Axis:
item-track
/
item-direction
Item Cross Axis Placement Mode:
item-cross
/
item-wrap
Item Placement Packing Mode: the
item-pack
property
Item Placement Shorthand: the
item-flow
shorthand
10
Acknowledgements
11
Security Considerations
12
Privacy Considerations
Changes
Additions Since Level 2
Recent Changes
Conformance
Document conventions
Conformance classes
Partial implementations
Implementations of Unstable and Proprietary Features
Non-experimental implementations
Index
Terms defined by this specification
Terms defined by reference
References
Normative References
Informative References
Property Index
Issues Index
1.
Introduction
This section is not normative.
Grid Layout is a layout model for CSS that has powerful abilities to control the sizing and positioning of boxes and their contents. Grid Layout is optimized for 2-dimensional layouts: those in which alignment of content is desired in both dimensions.
Representative Grid layout example
Although many layouts can be expressed with regular Grid Layout, restricting items into a grid in both axes also makes it impossible to express some common layouts on the Web.
This module defines a variant of
Grid Layout
that removes that restriction so that items can be placed into Grid-like tracks in one axis while stacking one after another in the other. Items not explicitly placed into specific tracks are placed into the column (or row) with the most remaining space based on the layout size of the items placed so far.
1.1.
Background and Motivation
1.1.1.
Waterfall Layout with Auto-placed Items
Grid lanes layout
, sometimes also called “masonry layout” or “waterfall layout”, is a common Web design pattern where a number of items—​commonly images or short article summaries—​are placed one by one into columns in a way that loosely resembles masonry construction. Unlike
multi-column layout
and
multi-line column flex layout
, where content is placed vertically in the first column until it must spills over to the second column,
automatic placement
in
grid lanes layout
selects a column for each new item such that it is generally closer to the top of the layout than items placed later.
The traditional Pinterest search results page exemplifies this layout:
Representative grid lanes layout example
Here, each item has a different height (depending on the content and the width of the column), and inspecting the DOM reveals (as the visual content itself gives no indication of ordering) that each item has been placed into the column with the smallest height so far.
This layout superficially looks similar to multi-column layout; but it has the advantage that scrolling down will naturally lead to “later” items in the layout (such as those less relevant in the search results).
It’s not possible to achieve this layout using earlier CSS layout models, unless you know up-front how tall each item will be, or use JavaScript for content measurement or placement.
Using
display: grid-lanes
together with
grid-template-columns
yields this type of
grid lanes layout
1.1.2.
One-dimensional Grid Layout
Grid layout
allows for powerful track sizing and explicit placement in two axes, but sometimes a layout only needs alignment of its items in one dimension.
Using
grid lanes layout
together with explicitly-positioned items allows for this type of one-dimensional grid layout.
This example
by Douglas Graham
uses explicit positioning to place each item into its assigned column; but there are no rows. Instead, items in each column stack one after the other. This layout can’t be duplicated in
grid layout
because the “spanning” relationships among the items in adjacent columns is not fixed: it depends on their relative heights and whether the optional banner or advertisement items are included. It also can’t be duplicated in
flex layout
because the source order of the items (which is used for reading, sequential navigation, and one-column mobile phone layout) goes back and forth between the two columns.
Comparison of one-column and two-column variants of a one-dimensional grid layout.
1.2.
Value Definitions
This specification follows the
CSS property definition conventions
from
[CSS2]
using the
value definition syntax
from
[CSS-VALUES-3]
. Value types not defined in this specification are defined in CSS Values & Units
[CSS-VALUES-3]
. Combination with other CSS modules may expand the definitions of these value types.
In addition to the property-specific values listed in their definitions, all properties defined in this specification also accept the
CSS-wide keywords
as their property value. For readability they have not been repeated explicitly.
2.
Grid Lanes Layout Model
Grid lanes layout
lays out items into pre-defined tracks similar to
grid layout
in one axis (called the
grid axis
), but flows them freely similar to
flex layout
in the other (called the
stacking axis
). Similar to
grid layout
and unlike
flex layout
,
grid lanes layout
’s auto-placement distributes items across the tracks to keep the lengths of those tracks as similar as possible.
Grid items
are formed and
blockified
exactly the same as in a regular
grid container
.
All CSS properties work the same as in a regular
grid container
unless otherwise specified by this specification. For example,
order
can be used to specify a different layout order for the items.
Note:
Subgrid items are supported, but subgridding only occurs in the
grid axis
; see
§ 3.2 Subgrids
for details.
A
grid lanes container
is a
grid container
whose contents participate in
grid lanes layout
. A
grid lanes container
creates column
tracks
if its
stacking axis
is the
block axis
, or row
tracks
if its
stacking axis
is the
inline axis
.
Comparing Grid Lanes Containers
Column Lanes
grid-template-columns: 1fr 2fr 3fr;
Row Lanes
grid-template-rows: 1fr 2fr 3fr;
2.1.
Reordering and Accessibility
Although
grid lanes layout
generally progresses in a forwards fashion (placing the next item endward of the current item in at least one axis, matching the natural “reading order”), it can switch between endward in the inline or block axis in a seemingly arbitrary manner. In simple cases, the
flow-tolerance
property can help reduce the feeling of backtracking due to small sizing differences in the
stacking axis
when laying out auto-placed items. But when
auto-placement
is mixed with
explicit placement
or spanning items, some amount of backtracking may occur.
For example, in the following markup sample, the fourth item is a spanner that doesn’t fit in the remaining empty column on the first line. It ends up positioned into the into the first column, which is the highest available space into which it will fit. The next few items, which have a span of 1, end up laying out “above” it in the empty column, violating the natural reading order.
<
section
class
=
masonry
>
<
div
class
=
item
>
1
</
div
>
<
div
class
=
item
>
2
</
div
>
<
div
class
=
"item tall"
>
3
</
div
>
<
div
class
=
"item wide"
>
4
</
div
>
<
div
class
=
item
>
5
</
div
>
<
div
class
=
item
>
6
</
div
>
<
div
class
=
item
>
7
</
div
>
</
section
>
<
style
>
.
masonry
{
display
:
grid
-
lanes
;
grid-template-columns
:
repeat
(
5
,
auto
);
}
.
item
{
height
:
50
px
;
}
.
item
.
wide
{
grid-column
:
span
3
;
}
.
item
.
tall
{
height
:
90
px
;
}
</
style
>
Auto-placed grid lanes layout with mixed-height items and mixed span sizes
Similarly, items explicitly placed into specific tracks can leave gaps behind them, into which subsequent auto-placed items can be placed visually out-of-order.
Authors should be aware of these possibilities and design layouts where such backtracking is minimized so that focus and reading order can be more easily followed. Alternatively, if the items do not have an inherent order, use the
reading-flow
property to allow the UA to re-order the items for reading and linear navigation.
Or should reordering be the default behavior for auto-placed items here?
Techniques for reducing backtracking include:
Using appropriate values for
flow-tolerance
, i.e. values large enough to avoid gratuitous differentiation among similarly-sized tracks, but not so large that meaningful differences get ignored.
Using explicit placement in ways that help group related items together, rather than ways that disrupt the natural order of items
Avoiding the combination of mixed span sizes in the
grid axis
and disparate item sizes in the
stacking axis
, which can cause items to get pulled out of order (see example above).
As with
grid layout
and
flex layout
authors can use the
order
property to re-order items; the same caveats apply. See
CSS Grid Layout 2
§ 4 Reordering and Accessibility
and
CSS Display 4
§ 3.1 Reordering and Accessibility
.
2.2.
Establishing Grid Lanes Layout
Name:
display
New values:
grid-lanes
|
inline-grid-lanes
grid-lanes
This value causes an element to generate a
block-level
grid lanes container
box.
inline-grid-lanes
This value causes an element to generate an
inline-level
grid lanes container
box.
A
grid lanes container
that is not
subgridded
in its
grid axis
establishes an
independent formatting context
for its contents.
ISSUE(12820): Write up how grid/masonry formatting contexts work more formally?
2.3.
Orienting Grid Lanes Layout
The orientation of a
grid lanes container
, i.e. whether its
grid axis
is the
inline axis
(establishing columns) or the
block axis
(establishing rows) is determined by the
TBD
property.
The
initial value
of this property is
normal
, which determines the orientation from the
grid-template-*
properties:
If
grid-template-columns
is
none
and
grid-template-rows
is not
none
, the
grid container
’s
block axis
is the
grid axis
(establishing rows).
Otherwise (thus by default), the
grid container
’s
inline axis
is the
grid axis
(establishing columns).
The following code will create a 2-column (“waterfall style”)
grid lanes container
that grows downward:
.container { display: grid-lanes; grid-template-
columns
: 100px 200px; }
while the following code will create a 2-row (“brick wall style”)
grid lanes container
that grows horizontally:
.container { display: grid-lanes; grid-template-
rows
: 100px 200px; }
Figure out whether we are re-using
grid-auto-flow
here (and what it’s values mean) or defining a new property like
grid-lanes-direction
.
[Issue #12803]
3.
Grid Lanes Track Specification
In the
grid axis
, the full power of
grid layout
is available for track specification:
Track sizes, line names, and areas can be specified on the
grid lanes container
’s
grid axis
, just like in
grid layout
.
The
explicit grid
and
implicit grid
are formed in the same way as for a regular
grid container
.
Items can be
placed
against these grid templates just as in
grid layout
.
However, auto-placed items contribute sizing to all tracks, not just the track into which they are ultimately placed; see
§ 3.4 Grid Axis Track Sizing
.
Note:
This is because auto-placed items must be laid out
as
they are placed, so that each track knows how “full” it is (and therefore which track should receive the next auto-placed item); thus, the tracks themselves must already have a definite size so that the items know their
available space
during layout.
3.1.
Declaring Grid Lanes Track Templates: the
grid-template-*
properties
The
grid-template-*
and
grid-auto-rows
/
grid-auto-columns
properties (and their shorthands) apply in the
grid axis
of the
grid lanes container
and establish tracks just as on regular
grid containers
. (They are ignored in the
stacking axis
.)
3.1.1.
Intrinsic Tracks and repeat()
Level 3 extends the
repeat()
notation to allow repetitions where neither the
min track sizing function
nor the
max track sizing function
is
definite
; in other words, the syntax for
<auto-repeat>
is relaxed to the following:
<auto-repeat>
= repeat( [ auto-fill
|
auto-fit ]
,
[
<line-names>
?
<track-size>
]
+
<line-names>
?
)
In order to resolve the number of repetitions, a hypothetical size is calculated for such tracks by initializing the track sizes (per
CSS Grid Layout 2
§ 12.4 Initialize Track Sizes
) and resolving intrinsic track sizes (per
CSS Grid Layout 2
§ 12.5 Resolve Intrinsic Track Sizes
) in accordance with
§ 3.4 Grid Axis Track Sizing
with the following assumptions:
Ignore explicit item placement. (That is, assume all items have an
automatic position
.)
Do not
collapse
any tracks.
Expand the repeated track listing to capture all possible
automatic placements
of each item, i.e. repeat the track listing
2 + (
largest span
- 2)/(
number of tracks in repeat()
)
times, rounded
down
to a whole number.
The hypothetical size of each track in the
repeat()
listing is given by the largest track corresponding to that entry (by index) in the listing.
For example, given a template of
repeat(auto-fill, 50px auto auto)
and a largest spanner of 2, you need to repeat the track listing twice, giving
50px auto auto 50px auto auto
. After doing the hypothetical layout, the 2nd and 5th tracks are maxed together to provide a hypothetical size for the first
auto
and the 3rd and 6th tracks are maxed together to provide a hypothetical size for the second
auto
. Those concrete sizes are then used to determine how many repetitions will fill the container.
Should this work also in Grid Layout somehow, or fall back to a single repetition? If so, how?
[Issue #10915]
Note:
This simplified layout heuristic is defined to be "good enough", while remaining fast and consistent. Ignoring placement is required just to make the concept coherent; before you know how many repetitions you need, you can’t tell what track an item (even one with a definite placement) will end up in.
Technically, if your explicit placement does not enter or cross over the repetition, we can know its placement prior to resolving the reptition. Do we want to adjust the algorithm above to account for this? That is, given
auto repeat(auto-fill, ...) auto
, if you explicitly place an item against line 1 or -1, we could let it only affect that track (rather than pretending it’ll go in every track).
3.2.
Subgrids
Subgridding
allows nested
grid lanes containers
(and
grid containers
) to share track sizes. If the parent’s corresponding axis is a
grid axis
, the subgridded axis is taken from the parent container
as specified for grid containers
; if the parent’s corresponding axis is a
stacking axis
, the subgridded axis also acts as a
stacking axis
.
What if this conflicts with the lanes orientation, or results in both axes stacking?
In
grid lanes layout
, auto-placed
subgrids
don’t inherit any line names from their parent grid, because that would make the placement of the item dependent on layout results; but the subgrid’s tracks are still aligned to the parent’s tracks as usual.
Here’s a subgrid
example
:
<style> .grid
{
display
:
inline-grid-lanes
;
grid-template-rows
:
auto auto
100
px
;
align-content
:
center
;
height
:
300
px
;
border
:
1
px
solid
;
}
.grid > *
{
margin
:
5
px
;
background
:
silver
;
}
.grid > :nth-child
(
2
n
)
{
background
:
pink
;
}
.grid subgrid
{
display
:
grid
;
grid
:
subgrid / subgrid
;
grid-row
:
2
/ span
2
;
grid-gap
:
30
px
;
}
.grid subgrid > *
{
background
:
cyan
;
}
</style>
<
div
class
=
"grid"
>
<
item
>
1
</
item
>
<
item
>
2
</
item
>
<
item
>
3
</
item
>
<
subgrid
>
<
item
style
=
"height:100px"
>
subgrid.1
</
item
>
<
item
>
sub.2
</
item
>
<
item
>
s.3
</
item
>
</
subgrid
>
<
item
>
4
</
item
>
<
item
>
5
</
item
>
<
item
style
=
"width: 80px"
>
6
</
item
>
<
item
>
7
</
item
>
</
div
>
The rendering of the subgrid example above.
Note how the subgrid’s first item ("subgrid.1") contributes to the intrinsic size of the 2nd row in the parent grid. This is possible since the subgrid specified a definite position so we know which tracks it will occupy. Note also that trying to subgrid the parent’s
stacking axis
results in the subgrid converting to a
grid lanes container
with its
inline axis
as the
stacking axis
.
A
subgrid
that is a
grid lanes container
can be referred to as a
grid lanes subgrid
.
3.3.
Track Repetition: the
repeat()
notation
This specification introduces new keywords and grid-lanes–specific behavior for the
repeat()
notation.
3.3.1.
repeat(auto-fit)
In
grid lanes containers
(as in regular
grid containers
)
auto-fit
acts like
auto-fill
, but with empty tracks
collapsed
. However, because placement occurs after track sizing,
grid lanes containers
use a heuristic to determine if a track will be occupied:
All tracks occupied by explicitly placed items are considered occupied.
With the sum of the spans of all auto-placed items as
N
, all unoccupied tracks up to the
N
th such track are considered occupied.
All tracks produced by the
auto-fit
repetition and considered unoccupied by this heuristic are assumed “empty” and are
collapsed
. A
collapsed grid track
cannot accept placement of auto-placed items.
Note:
It is possible for an auto-placed item to be placed in a track when
auto-fill
is used that would be collapsed if
auto-fit
is used if there are auto-placed items with a span greater than 1 mixed with explicitly-placed items that leave gaps too small for the auto-placed items.
3.4.
Grid Axis Track Sizing
Track sizing works the same as in
CSS Grid
, except that when considering which items contribute to intrinsic sizes:
All items explicitly placed in that track contribute, and
All items with an
automatic grid position
contribute (regardless of whether they are ultimately placed in that track).
For example, suppose there are two columns in the
grid axis
and that
Items A, B, and C have no explicit position.
Item D is explicitly placed into the first column.
In this case, items A, B, C, and D all contribute to sizing the first column, while only A, B, and C (and not D) contribute to the second column.
In the case of spanning items with an
automatic grid position
, they are assumed to be placed at every possible start position, and contribute accordingly.
For example, suppose there are 5 columns in the
grid axis
, with the middle having a fixed size of
100px
and the other two being
auto
-sized. For the purpose of track sizing, an item that spans 2 tracks and has an intrinsic contribution of 220px is essentially copied and assumed to exist:
At grid line 1, contributing 110px to each of the first two tracks.
At grid line 2, contributing 120px to the second track.
At grid line 3, contributing 120px to the fourth track.
At grid line 4, contributing 110px to the fourth and fifth tracks.
Note:
This algorithm ensures that each track is at least big enough to accommodate every item that is ultimately placed in it, and does not create dependency cycles between placement and track sizing. However, depending on the variation in sizes, tracks could be larger than necessary: an exact fit is only guaranteed if all items are explicitly placed in the
grid axis
or all items are the same size (or matching multiples of that size, in the case of spanning items).
3.4.1.
Subgrid Item Contributions
When sizing the tracks of either a regular
grid container
or a
grid lanes container
, a
grid lanes subgrid
has special handling of items that have an
automatic grid position
:
Any such item is placed into every possible grid track that could be spanned by the
grid lanes subgrid
. (If the subgrid has a
definite grid position
, thus only the spanned tracks; if it has an
automatic grid position
, then all tracks in the parent grid.)
Any such item receives the largest margin/border/padding contribution of each edge at which it could hypothetically be placed. If the item spans the entire subgrid, it receives both. (See
CSS Grid Layout §9
.)
3.4.2.
Optimized Track Sizing
Track sizing can be optimized by aggregating items that have the same span size and placement into a single virtual item as follows:
Separate all the
grid items
into
item groups
, according to the following properties:
the span of the item
the placement of the item, i.e. which tracks it is allowed to be placed in
the item’s
baseline-sharing group
Note:
For example, an item with span 2 placed in the second track will be in a different group than an item with span 2 that has an
automatic grid position
.
For each
item group
, synthesize a
virtual grid item
that has the maximum of every intrinsic size contribution among the items in that group.
If the items apply
baseline alignment
, determine the baselines of the
virtual grid item
by placing all of its items into a single hypothetical grid track and finding their shared baseline(s) and shims. Increase the group’s intrinsic size contributions accordingly.
Place hypothetical copies of each
virtual grid item
into the
grid axis
tracks in every position that the item could potentially occupy, and run the
track sizing algorithm
with those items. The resulting track sizes are the
grid lanes container’s
track sizes.
Note:
This optimization should give the same results as the track sizing description
above
; if not this is an error, please
report it to the CSSWG
.
4.
Grid Lanes Item Placement
In the
grid axis
, items can be
explicitly placed
into tracks and span them using the familiar
grid-placement properties
’ syntax. Auto-placement, however, uses the
§ 4.4 Grid Lanes Layout and Placement Algorithm
, placing each item with an
automatic grid position
into the “shortest” track available.
Here’s a grid lanes layout demonstrating explicitly placed and spanning items:
.container
{
grid-template-columns
:
repeat
(
3
,
auto
);
}
.container > :nth-child
(
2
)
{
/* auto-placed, but spanning. */
grid-column: span
2
;
}
.container > :nth-child
(
3
)
{
/* manually placed */
grid-column:
3
;
}
/* all other children are auto-placed */
Rendering of the example above.
4.1.
Specifying Grid Axis Item Placement: the
grid-column-*
and
grid-row-*
properties
The
grid-column-*
and
grid-row-*
properties (and their shorthands) apply in the
grid axis
of the items and establish placement just as in regular
grid layout
.
4.2.
Placement Precision: the
flow-tolerance
property
Name:
flow-tolerance
Value:
normal
|
<length-percentage>
|
infinite
Initial:
normal
Applies to:
grid lanes containers
Inherited:
no
Percentages:
relative to the
grid-axis
content box
size of the
grid lanes container
Computed value:
a computed
<length-percentage>
value
Canonical order:
per grammar
Animation type:
as length
Grid lanes containers
are filled by placing each
grid item
in whichever
grid track
is currently the least filled. When multiple tracks are tied for least-filled, placing the items in order looks good. But if tracks are only
very slightly
different heights, it can look strange to have them not fill in order, as the height differences aren’t perceived as
meaningfully
different.
The
flow-tolerance
property specifies what the threshold is for considering tracks to be “the same height”, causing them to fill in order.
<length-percentage>
Specifies the
tie threshold
for the
grid lanes container
. Placement positions are considered to be equally good (“tied”) if they are within the specified distance from the shortest position.
Note:
The initial value is a “small” distance (
1em
) that is probably appropriate to represent “close enough”.
normal
Resolves to a
used value
of
1em
in
grid lanes layout
and a
used value
of
0
in all other layout modes.
infinite
Specifies an infinite
tie threshold
. This makes items distribute themselves strictly in order, without considering the length of the tracks at all.
Note:
This value can result in consecutive items being placed in dramatically different positions in the
stacking axis
, which can be confusing to readers. If the initial value (`1em`) is too small, consider a larger value (such as `10em` or `50vh`) instead of `infinite`.
In the following example, when placing the 5th item, the fourth column is the shortest, but the first column is
almost
as short.
With the default tolerance of
1em
, both the first and fourth columns are considered to be "tied", and so the first is chosen.
If, instead, the tolerance is set to
0px
, then there is no tie; only the fourth column is a possible placement.
Note:
We expect to apply this property to
flex layout
in the future also, see
discussions
on how that might work.
4.3.
Dense Placement: the
dense
keyword
In
grid layout
,
grid-auto-flow: dense
allows backtracking during the
grid item placement algorithm
. The
dense
keyword similarly allows backtracking during the
grid lanes placement algorithm
. However, because item placement and sizing are intertwined in
grid lanes layout
, an item can only backtrack into a compatible empty slot if the total used size of that slot’s tracks matches the used size of the item’s normal-placement tracks.
Note:
This restriction avoids laying out the item more than once.
4.4.
Grid Lanes Layout and Placement Algorithm
For each of the tracks in the
grid axis
, keep a
running position
initialized to zero. Maintain also a
auto-placement cursor
, initially pointing to the first line.
For each item in
order-modified document order
:
If the item has a
definite grid position
in the
grid axis
, use that placement.
Should this also update the placement cursor?
Otherwise, resolve its
grid axis
placement using these substeps:
Starting at the first
grid axis
line in the
implicit grid
, find the largest
running position
of the
grid axis
tracks that the item would span if it were placed at this line, and call this position
max_pos
.
Repeat the previous step for each successive line number until the item would no longer fit inside the grid.
Let
possible lines
be the line that resulted in the smallest
max_pos
, and all lines that result in a
max_pos
within the
tie threshold
of this
max_pos
.
Choose the first line in
possible lines
greater than or equal to the
auto-placement cursor
as the item’s position in the
grid axis
; or if there are none such, choose the first one.
Update the
auto-placement cursor
to point to item’s last line.
Place the item in its
grid axis
tracks at the maximum of the
running position
s of the tracks it spans.
Calculate the size of the item’s
containing block
and then layout the item. Set the
running position
of the spanned
grid axis
tracks to
max_pos
+
outer size
+
grid-gap
.
For this purpose, the
outer sizes
of the box are floored at zero.
Note:
That is, if the box has sufficiently large negative margins, it won’t somehow count as a negative
size
here. The
running position
of a track never decreases, so a negative-sized box won’t cause future normally-sized items placed in the track to overlap previous items.
If the
grid lanes container
uses
dense
packing, and there exists skipped spaces in the layout (e.g. due to spanning items) into which the item, as it is sized now, could have fit if it were placed earlier, and where the spanned tracks have the same total used size as the tracks into which it is currently placed, then instead place it into the highest such space. If there are multiple valid spaces within the
tie threshold
of the highest space, place it in the
start
-most of them. Rewind the
auto-placement cursor
and the
running position
to their values before this item’s placement.
Note:
Items that
visually
intrude into preceding empty spaces (via negative margins,
position: relative
, transforms, etc.), do not affect the size of those empty spaces. Later items can get placed in those spaces and visually overlap these previous items.
Note:
Dense packing both ignores the
auto-placement cursor
when backfilling, and does not update it after placement. If there aren’t any acceptable placement gaps to backfill, though, it places items exactly as when
dense
were not specified.
Note:
This algorithm chooses the track that would result in the item being placed as highly as possible. If there are ties, it chooses the earliest such track,
after
the most recently placed item if possible (ensuring that it always “moves forward” even in the presence of ties).
4.4.1.
Containing Block
The
containing block
for a
grid item
participating in
grid lanes layout
is formed by its
grid area
in the
grid axis
and the
grid container
’s
content box
in the
stacking axis
.
4.4.2.
Placement and Writing Modes
Note:
Like all of
grid layout
, grid lanes layout and placement is sensitive to the
writing mode
. For example, for
direction: rtl
, items are placed right-to-left rather than left-to-right, whether the inline axis is a
grid axis
or a
stacking axis
.
Here’s a simple
example
using
direction: rtl
in the
grid axis
:
<style> .grid
{
display
:
inline-grid-lanes
;
direction
:
rtl
;
grid-template-columns
:
repeat
(
4
,
2
ch
);
border
:
1
px
solid
;
}
item
{
background
:
silver
}
item:nth-child
(
2
n
+1
)
{
background
:
pink
;
height
:
4
em
;
}
</style>
<
div
class
=
"grid"
>
<
item
>
1
</
item
>
<
item
style
=
"grid-column:span 2"
>
2
</
item
>
<
item
>
3
</
item
>
<
item
>
4
</
item
>
</
div
>
Rendering of the
direction: rtl
example above.
Here’s a simple
example
using
direction: rtl
in the
stacking axis
:
<style> .grid
{
display
:
inline-grid-lanes
;
direction
:
rtl
;
width
:
10
ch
;
column-gap
:
1
ch
;
grid-template-rows
:
repeat
(
4
,
2
em
);
border
:
1
px
solid
;
}
item
{
background
:
silver
}
item:nth-child
(
2
n
+1
)
{
background
:
pink
;
width
:
4
ch
;
}
</style>
<
div
class
=
"grid"
>
<
item
>
1
</
item
>
<
item
style
=
"grid-row:span 2"
>
2
</
item
>
<
item
>
3
</
item
>
<
item
>
4
</
item
>
</
div
>
Rendering of the
direction: rtl
example above.
5.
Sizing Grid Containers
Sizing Grid Containers
works the same as for regular
grid containers
but with the following replacement for the
stacking axis
: The
max-content size
(
min-content size
) of a
grid container
in the
stacking axis
is the size of the
stacking range
when the
grid lanes container
is sized under a
max-content constraint
(
min-content constraint
) in that axis. The
stacking range
is the range between the startmost
outer edge
among the first items of each track and the endmost
outer edge
among the last items of each track.
Here’s a simple
example
:
<style> .grid
{
display
:
inline-grid-lanes
;
grid-template-columns
:
50
px
100
px
auto
;
grid-gap
:
10
px
;
border
:
1
px
solid
;
}
item
{
background
:
silver
;
margin
:
5
px
;
}
</style>
<
div
class
=
"grid"
>
<
item
style
=
"border:10px solid"
>
1
</
item
>
<
item
>
2
</
item
>
<
item
>
3
</
item
>
<
item
style
=
"height:50px"
>
4
</
item
>
<
item
>
5
</
item
>
<
item
>
6
</
item
>
</
div
>
Rendering of the
grid container
intrinsic sizing example above.
6.
Alignment and Spacing
6.1.
Gutters: the
row-gap
,
column-gap
, and
gap
properties
Gutters
are supported in both axes using the
row-gap
and
column-gap
properties (and their
gap
shorthand
). In the
grid axis
they work the same as in a regular
grid container
; see
CSS Grid Layout 2
§ 11. Alignment and Spacing
. In the
stacking axis
, the gap is applied between the margin boxes of each pair of adjacent items. Margins do not collapse in either axis.
6.2.
Grid-axis Alignment: the
align-content
/
justify-content
,
align-self
/
justify-self
, and
align-items
/
justify-items
properties
In the
grid axis
, the
box alignment properties
work the same as in a regular
grid container
. See
CSS Grid Layout 2
§ 11. Alignment and Spacing
and
CSS Box Alignment Level 3
.
6.3.
Stacking-axis Content Distribution: the
align-content
/
justify-content
properties
In the
stacking axis
,
content-distribution
is applied to the content as a whole, similarly to how it behaves in block containers. More specifically, the
alignment subject
is the
stacking range
.
The extent of the alignment subject is indicated by the dashed border, while the alignment container is the whole container (indicated by the height of the column dividers). It defaults to start-aligning, but depending on (in this case)
align-content
, the items can move down, as a block, to center- or end-align vertically.
Note:
There is only ever one
alignment subject
for these properties in the
stacking axis
, so the unique
align-content
/
justify-content
values boil down to
start
,
center
,
end
, and
baseline alignment
. (The behavior of
normal
and
stretch
is identical to
start
, and the
distributed alignment
values behave as their
fallback alignments
.) If the
grid items
overflow the
grid container
’s
content box
in the
stacking axis
, then the
stacking range
will be larger than the
grid container
’s
content box
.
6.4.
Stacking-axis Self Alignment: the
align-self
/
justify-self
and
align-items
/
justify-items
properties
In the
stacking axis
, the
self-alignment properties
only apply to items that are adjacent to a gap in the layout, i.e. placed in their
grid track
(s) either immediately before a spanning item or as the last item. The
alignment subject
is the item’s
margin box
, and the
alignment container
is that box plus the adjacent gap.
The three highlighted items are the only ones with "gaps" following them, so they’re the only ones that will respond to
align-items
. Their
alignment containers
are indicated by the dashed areas.
Is this a reasonable definition for how the
self-alignment properties
should work in the
stacking axis
?
[Issue #10275]
6.5.
Baseline Alignment
Item
baseline alignment
inside the
grid axis
tracks works as usual for a regular
grid container
, and the
grid container
’s baseline is determined the same as for a regular
grid container
in that axis.
Baseline alignment
is not supported in the
stacking axis
. The first baseline set of the
grid container
in this axis is generated from the highest
alignment baseline
among the
grid items
placed first in each track, and the last baseline set from the lowest
alignment baseline
among the
grid items
placed last in each track.
We could support baseline alignment in the first row. Do we want to?
7.
Fragmentation
7.1.
Fragmentation in the stacking axis
Each
grid axis
track is fragmented independently in the
stacking axis
. If a
grid item
is fragmented, or has a
forced break
before/after it, then the
running position
for the tracks that it spans in the
grid axis
are set to the size of the
fragmentainer
so that no further items will be placed in those tracks. An item that is split into multiple fragments retains its placement in the
grid axis
for all its fragments. A grid item that is pushed, however, is placed again by the next
grid container
fragment. Placement continues until all items are placed or pushed to a new fragment.
Here’s an
example
illustrating fragmentation of grid lanes layout with
stacking
in its
block axis
. It renders like this:
Visualization of fragmentation in a
block-axis
grid lanes layout
.
7.2.
Fragmentation in the Grid Axis
Fragmentation in the
grid axis
of a
grid lanes container
is also supported. In this case the fragmentation behaves more like in a regular
grid container
; however, there’s a separate step to determine which
grid-axis
track each item is placed into, before fragmentation occurs.
Here’s an
example
illustrating fragmentation of grid lanes layout with
stacking
in its
inline axis
. In this case the breaks occurs between the
grid-axis
rows. It renders like this:
Visualization of fragmentation in the
block axis
with
inline-axis
grid lanes layout
.
8.
Absolute Positioning
Grid-aligned absolute-positioned descendants
are supported in
grid lanes containers
just as for regular
grid containers
; however, in the
stacking axis
there exist only two lines (in addition to the
auto
lines) for placement:
line 1 (line -2) corresponds to the start edge of the
stacking range
line 2 (line -1) corresponds to the end edge of the
stacking range
It might be useful to define a static position in the
stacking axis
. Maybe it could defined as the max (or min?) current
running position
of the
grid-axis
tracks at that point? Or the end of the item before it?
9.
Graceful Degradation
Typically, a grid lanes design can be expected to degrade quite nicely in a UA that supports
grid layout
but not
grid lanes layout
.
Here’s an
example
to illustrate this.
display
:
grid
;
display
:
grid-lanes
;
/* ignored in UAs that don't support grid lanes layout */
grid-template-columns:
150
px
100
px
50
px
;
This creates a layout with three columns, but will have "more gaps" in the
block axis
if the UA doesn’t support
grid lanes layout
. Here’s what it looks like with grid lanes support for comparison:
Rendering of the example in a UA with grid lanes support.
Appendix A: Generic Layout Item Flow Controls: the
item-*
properties
This section is likely to move to another spec, such as
[css-display-4]
, since it affects multiple display types. It is also still under discussion as to whether this is a good idea.
Multiple layout modes in CSS place their children as atomic "items" organized into rows and/or colummns in their container, and allow the author to configure their ordering and placement. The
item-*
properties provide generic controls for these ordering and placement options, encapsulating the layout-specific
flex-flow
and
grid-auto-flow
properties.
Overview of Item Flow Controls
Flow-oriented Proposal
Track-oriented Proposal
Value Space
Description
Existing flex property
Existing grid property
item-direction
item-track
auto | row | column | row-reverse | column-reverse
Controls whether items are placed into rows or columns, and whether within those tracks they are ordered start-to-end or end-to-start.
flex-direction
grid-auto-flow
item-wrap
item-cross
[ auto | nowrap | wrap ] || [ normal | reverse ] | wrap-reverse
Controls whether items are wrapped in the axis opposite to that controled by the row/column property, and if so if they are placed in start-to-end or end-to-start order.
flex-wrap
Introduced into
grid-auto-flow
in this level.
item-pack
item-pack
normal | dense || balance
Configures how items are packed into their tracks.
grid-auto-flow
flow-tolerance
flow-tolerance
normal |
<length-percentage>
| infinite
Defines a layout-specific amount of “slack” in placement decisions.
The CSSWG is still figuring out how these properties should be named and fit together.
[Issue #11480]
Item Flow Axis:
item-track
/
item-direction
Name:
item-direction
,
item-track
Value:
auto
|
row
|
column
|
row-reverse
|
column-reverse
Initial:
auto
Applies to:
flex containers
,
grid containers
,
grid lanes containers
Inherited:
no
Percentages:
N/A
Computed value:
as specified
Canonical order:
per grammar
Animation type:
discrete
This property controls whether items are placed as rows or columns, and whether within those tracks they are ordered start-to-end or end-to-start.
There two open debates on this property: a) what should it be called and b) does it describe the primary direction of placement, or the orientation of the tracks into which items are placed; in other words, is the
primary axis
defined by this property the
primary placement axis
or the
primary track axis
. These are identical for
flex layout
and
grid layout
, but differ for
grid lanes layout
whose primary placement direction is across its tracks. See
latest discussion
.
[Issue #11480]
auto
Computes to either
row
or
column
depending on the layout mode:
On
flex containers
and
grid containers
, computes to
row
.
On
grid lanes containers
, if
grid-template-rows
is not
none
and
grid-template-columns
is
none
, computes to the value representing row tracks; otherwise computes to the value representing column tracks.
row
Track-oriented Option
Represents placement into rows, i.e. tracks or lines parallel to the
inline axis
. Items fill those rows in start-to-end order.
Flow-oriented Option
Represents row-primary item placement, i.e. placing items start-to-end in the
inline axis
, producing flex row lines in
flex layout
, and column grid tracks in
grid lanes layout
.
column
Track-oriented Option
Represents placement into columns, i.e. tracks or lines parallel to the
block axis
. Items fill those columns in start-to-end order.
Flow-oriented Option
Represents column-primary item placement, i.e. placing items start-to-end in the
block axis
, producing flex column lines in
flex layout
, and row grid tracks in
grid lanes layout
.
row-reverse
Same as
row
, but using end-to-start placement order.
column-reverse
Same as
column
, but using end-to-start placement order.
Item Cross Axis Placement Mode:
item-cross
/
item-wrap
Name:
item-wrap
,
item-cross
Value:
[ auto
|
nowrap
|
wrap ]
||
[ normal
|
reverse ]
|
wrap-reverse
Initial:
auto
Applies to:
flex containers
,
grid containers
,
grid lanes containers
Inherited:
no
Percentages:
N/A
Computed value:
as specified
Canonical order:
per grammar
Animation type:
discrete
Controls placement in the axis opposite to the
primary axis
.
auto
Computes to
nowrap
on
flex containers
, and
wrap
on everything else.
nowrap
Items are placed in the
primary placement axis
forever, even if they run out of room. In
flex layout
this creates a
single-line flex container
; in
grid layout
this creates
implicit tracks
in the
primary placement axis
as necessary.
wrap
Items wrap when the
primary placement axis
runs out of space. In
flex layout
this creates a
multi-line flex container
; in
grid layout
auto-placement algorithm moves to the next row/column when it runs out of explicit tracks in the
primary placement axis
.
normal
Items are placed in start-to-end order in the axis opposite to the
primary track axis
.
In
flex layout
and
grid layout
, this controls the direction that new tracks (flex lines or grid tracks) are placed in.
In
grid lanes layout
,
for track-oriented syntax this controls which track is selected when several are tied for equal height
;
for flow-oriented syntax this controls which direction items fill their track in.
reverse
Items are placed in end-to-start order in the axis opposite to the
primary track axis
.
wrap-reverse
Computes to
wrap reverse
.
Note:
This value exists for consistency with the existing
flex-wrap
value.
The interpretation and naming of this property depends on the interpretation of axes for
item-direction
/
item-track
.
[Issue #11480]
Item Placement Packing Mode: the
item-pack
property
Name:
item-pack
Value:
normal
|
dense
||
balance
Initial:
normal
Applies to:
flex containers
,
grid containers
,
grid lanes containers
Inherited:
no
Percentages:
N/A
Computed value:
as specified
Canonical order:
per grammar
Animation type:
discrete
This property controls how items are distributed among the tracks in a layout-specific way.
normal
Uses the default packing strategy for the layout mode.
dense
Allows backtracking to place items in earlier spaces that were skipped. (Such spaces can exist because earlier items were too big for those spaces.)
For example, in
flex layout
this allows placing items on earlier lines that still have enough empty space left over.
balance
In
flex layout
, this value balances the amount of content on each line (including the last line), similar to
text-wrap-style: balance
.
Item Placement Shorthand: the
item-flow
shorthand
Name:
item-flow
Value:
<'item-direction'>
||
<'item-wrap'>
||
<'item-pack'>
||
<'flow-tolerance'>
Initial:
see individual properties
Applies to:
see individual properties
Inherited:
see individual properties
Percentages:
see individual properties
Computed value:
see individual properties
Animation type:
see individual properties
Canonical order:
per grammar
This
shorthand property
sets all its
item-*
longhand properties
in a single declaration.
10.
Acknowledgements
Special thanks goes to Cameron McCormack who wrote a masonry layout explainer document (from which was lifted the Background chapter) and presented it to the CSSWG, and to Mats Palmgren who developed the original version of this specification alongside a prototype implementation in Firefox. Thanks also to everyone who provided feedback on the
initial proposal
for this feature as well as the
many subsequent issues
, and to Noam Rosenthal particularly for
naming it
.
11.
Security Considerations
As a layout specification, this spec introduces no new security considerations beyond that exposed by CSS layout in general.
12.
Privacy Considerations
As a layout specification, this spec introduces no new privacy considerations beyond that exposed by CSS layout in general.
Changes
Additions Since Level 2
The following features have been added since
Level 2
:
Added
grid lanes layout
.
Expanded
repeat(auto-fill)
and
repeat(auto-fit)
to accept indefinite track sizing functions. (
Issue 9321
,
Issue 12899
)
Introduced the
flow-tolerance
property.
Introduced the
item-flow
property and its longhands, as generic controls for item ordering and placement. See
Appendix A: Generic Layout Item Flow Controls: the item-* properties
. Note: This is still under discussion. (
Issue 11480
)
Recent Changes
The following changes have been made since the
23 December 2025 Working Draft
:
Renamed
item-tolerance
to
flow-tolerance
. (
Issue 10884
)
The following changes have been made since the
16 December 2025 Working Draft
:
Defined
normal
as the initial value for
setting the track orientation
.
Defined the
self-alignment properties
’ effect in the
stacking axis
. See
§ 6.4 Stacking-axis Self Alignment: the align-self/justify-self and align-items/justify-items properties
.
The following changes have been made since the
17 September 2025 Working Draft
:
Defined a new inner display type,
grid-lanes
, to establish
grid lanes layout
, and updated spec vocabulary to match. (
Issue 12022
)
Dropped the unnecessary
auto-areas
value from
repeat()
. (
Issue 10854
)
Adjusted heuristic for
repeat(auto-fill)
and
repeat(auto-fit)
with indefinite track sizing functions. (
Issue 12899
)
Floored the contribution of items to a track’s
running position
at zero. (
Issue 12918
)
Renamed
item-tolerance
to
flow-tolerance
. (
Issue 10884
)
See also
earlier changes
.
Conformance
Document conventions
Conformance requirements are expressed with a combination of descriptive assertions and RFC 2119 terminology. The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”, “SHOULD NOT”, “RECOMMENDED”, “MAY”, and “OPTIONAL” in the normative parts of this document are to be interpreted as described in RFC 2119. However, for readability, these words do not appear in all uppercase letters in this specification.
All of the text of this specification is normative except sections explicitly marked as non-normative, examples, and notes.
[RFC2119]
Examples in this specification are introduced with the words “for example” or are set apart from the normative text with
class="example"
, like this:
This is an example of an informative example.
Informative notes begin with the word “Note” and are set apart from the normative text with
class="note"
, like this:
Note, this is an informative note.
Advisements are normative sections styled to evoke special attention and are set apart from other normative text with
<strong class="advisement">
, like this:
UAs MUST provide an accessible alternative.
Tests
Tests relating to the content of this specification may be documented in “Tests” blocks like this one. Any such block is non-normative.
Conformance classes
Conformance to this specification is defined for three conformance classes:
style sheet
A
CSS style sheet
.
renderer
A
UA
that interprets the semantics of a style sheet and renders documents that use them.
authoring tool
A
UA
that writes a style sheet.
A style sheet is conformant to this specification if all of its statements that use syntax defined in this module are valid according to the generic CSS grammar and the individual grammars of each feature defined in this module.
A renderer is conformant to this specification if, in addition to interpreting the style sheet as defined by the appropriate specifications, it supports all the features defined by this specification by parsing them correctly and rendering the document accordingly. However, the inability of a UA to correctly render a document due to limitations of the device does not make the UA non-conformant. (For example, a UA is not required to render color on a monochrome monitor.)
An authoring tool is conformant to this specification if it writes style sheets that are syntactically correct according to the generic CSS grammar and the individual grammars of each feature in this module, and meet all other conformance requirements of style sheets as described in this module.
Partial implementations
So that authors can exploit the forward-compatible parsing rules to assign fallback values, CSS renderers
must
treat as invalid (and
ignore as appropriate
) any at-rules, properties, property values, keywords, and other syntactic constructs for which they have no usable level of support. In particular, user agents
must not
selectively ignore unsupported component values and honor supported values in a single multi-value property declaration: if any value is considered invalid (as unsupported values must be), CSS requires that the entire declaration be ignored.
Implementations of Unstable and Proprietary Features
To avoid clashes with future stable CSS features, the CSSWG recommends
following best practices
for the implementation of
unstable
features and
proprietary extensions
to CSS.
Non-experimental implementations
Once a specification reaches the Candidate Recommendation stage, non-experimental implementations are possible, and implementors should release an unprefixed implementation of any CR-level feature they can demonstrate to be correctly implemented according to spec.
To establish and maintain the interoperability of CSS across implementations, the CSS Working Group requests that non-experimental CSS renderers submit an implementation report (and, if necessary, the testcases used for that implementation report) to the W3C before releasing an unprefixed implementation of any CSS features. Testcases submitted to W3C are subject to review and correction by the CSS Working Group.
Further information on submitting testcases and implementation reports can be found from on the CSS Working Group’s website at
https://www.w3.org/Style/CSS/Test/
. Questions should be directed to the
public-css-testsuite@w3.org
mailing list.
Index
Terms defined by this specification
auto
value for item-direction, item-track
, in § Unnumbered section
value for item-wrap, item-cross
, in § Unnumbered section
auto-placement cursor
, in § 4.4
<auto-repeat>
, in § 3.1.1
balance
, in § Unnumbered section
column
, in § Unnumbered section
column-reverse
, in § Unnumbered section
dense
, in § Unnumbered section
flow-tolerance
, in § 4.2
grid axis
, in § 2
grid-axis
, in § 2
grid-lanes
, in § 2.2
grid lanes container
, in § 2
Grid lanes layout
, in § 2
grid lanes subgrid
, in § 3.2
infinite
, in § 4.2
inline-grid-lanes
, in § 2.2
item-cross
, in § Unnumbered section
item-direction
, in § Unnumbered section
item-flow
, in § Unnumbered section
item groups
, in § 3.4.2
item-pack
, in § Unnumbered section
item-track
, in § Unnumbered section
item-wrap
, in § Unnumbered section
<length-percentage>
, in § 4.2
normal
value for flow-tolerance
, in § 4.2
value for grid-auto-flow
, in § 2.3
value for item-pack
, in § Unnumbered section
value for item-wrap, item-cross
, in § Unnumbered section
nowrap
, in § Unnumbered section
primary axis
, in § Unnumbered section
primary placement axis
, in § Unnumbered section
primary track axis
, in § Unnumbered section
reverse
, in § Unnumbered section
row
, in § Unnumbered section
row-reverse
, in § Unnumbered section
running position
, in § 4.4
stacking axis
, in § 2
stacking-axis
, in § 2
stacking range
, in § 5
tie threshold
, in § 4.2
virtual grid item
, in § 3.4.2
wrap
, in § Unnumbered section
wrap-reverse
, in § Unnumbered section
Terms defined by reference
[CSS-ALIGN-3]
defines the following terms:
align-content
align-items
align-self
alignment baseline
alignment container
alignment subject
baseline alignment
baseline-sharing group
box alignment properties
center
column-gap
distributed alignment
end
fallback alignment
gap
grid-gap
justify-content
justify-items
justify-self
normal
row-gap
self-alignment properties
start
stretch
[CSS-BOX-4]
defines the following terms:
content box
margin box
outer edge
[CSS-BREAK-4]
defines the following terms:
forced break
fragmentainer
[CSS-CASCADE-5]
defines the following terms:
initial value
longhand property
shorthand
shorthand property
used value
[CSS-DISPLAY-4]
defines the following terms:
block-level
blockify
containing block
display
independent formatting context
inline-level
order
order-modified document order
reading-flow
[CSS-FLEXBOX-1]
defines the following terms:
flex container
flex layout
flex-flow
flex-wrap
multi-line flex container
single-line flex container
[CSS-GAPS-1]
defines the following terms:
repeat()
[CSS-GRID-2]
defines the following terms:
<line-names>
<track-size>
auto-fill
auto-fit
automatic grid position
automatic placement
automatic position
collapsed grid track
definite grid position
definite position
explicit grid
grid area
grid container
grid item
grid layout
grid track
grid-auto-columns
grid-auto-flow
grid-auto-rows
grid-placement property
grid-template-columns
grid-template-rows
implicit grid
implicit grid track
max track sizing function
min track sizing function
none
subgrid
[CSS-POSITION-3]
defines the following terms:
position
[CSS-SIZING-3]
defines the following terms:
available space
definite
max-content constraint
max-content size
min-content constraint
min-content size
outer size
[CSS-TEXT-4]
defines the following terms:
text-wrap-style
[CSS-VALUES-4]
defines the following terms:
+
,
<length-percentage>
?
CSS-wide keywords
|
||
[CSS-WRITING-MODES-3]
defines the following terms:
direction
[CSS-WRITING-MODES-4]
defines the following terms:
block axis
block-axis
inline axis
inline-axis
start
writing mode
References
Normative References
[CSS-ALIGN-3]
Elika Etemad; Tab Atkins Jr..
CSS Box Alignment Module Level 3
. 11 March 2025. WD. URL:
https://www.w3.org/TR/css-align-3/
[CSS-BOX-4]
Elika Etemad.
CSS Box Model Module Level 4
. 4 August 2024. WD. URL:
https://www.w3.org/TR/css-box-4/
[CSS-BREAK-4]
Rossen Atanassov; Elika Etemad.
CSS Fragmentation Module Level 4
. 18 December 2018. FPWD. URL:
https://www.w3.org/TR/css-break-4/
[CSS-CASCADE-5]
Elika Etemad; Miriam Suzanne; Tab Atkins Jr..
CSS Cascading and Inheritance Level 5
. 13 January 2022. CR. URL:
https://www.w3.org/TR/css-cascade-5/
[CSS-DISPLAY-4]
Elika Etemad; Tab Atkins Jr..
CSS Display Module Level 4
. 6 November 2025. WD. URL:
https://www.w3.org/TR/css-display-4/
[CSS-FLEXBOX-1]
Elika Etemad; Tab Atkins Jr.; Rossen Atanassov.
CSS Flexible Box Layout Module Level 1
. 14 October 2025. CRD. URL:
https://www.w3.org/TR/css-flexbox-1/
[CSS-GAPS-1]
Kevin Babbitt.
CSS Gap Decorations Module Level 1
. 17 April 2025. FPWD. URL:
https://www.w3.org/TR/css-gaps-1/
[CSS-GRID-1]
Tab Atkins Jr.; et al.
CSS Grid Layout Module Level 1
. 26 March 2025. CRD. URL:
https://www.w3.org/TR/css-grid-1/
[CSS-GRID-2]
Tab Atkins Jr.; et al.
CSS Grid Layout Module Level 2
. 26 March 2025. CRD. URL:
https://www.w3.org/TR/css-grid-2/
[CSS-SIZING-3]
Tab Atkins Jr.; Elika Etemad.
CSS Box Sizing Module Level 3
. 17 December 2021. WD. URL:
https://www.w3.org/TR/css-sizing-3/
[CSS-TEXT-4]
Elika Etemad; et al.
CSS Text Module Level 4
. 29 May 2024. WD. URL:
https://www.w3.org/TR/css-text-4/
[CSS-VALUES-3]
Tab Atkins Jr.; Elika Etemad.
CSS Values and Units Module Level 3
. 22 March 2024. CRD. URL:
https://www.w3.org/TR/css-values-3/
[CSS-VALUES-4]
Tab Atkins Jr.; Elika Etemad.
CSS Values and Units Module Level 4
. 12 March 2024. WD. URL:
https://www.w3.org/TR/css-values-4/
[CSS-WRITING-MODES-4]
Elika Etemad; Koji Ishii.
CSS Writing Modes Level 4
. 30 July 2019. CR. URL:
https://www.w3.org/TR/css-writing-modes-4/
[CSS2]
Bert Bos; et al.
Cascading Style Sheets Level 2 Revision 1 (CSS 2.1) Specification
. 7 June 2011. REC. URL:
https://www.w3.org/TR/CSS2/
[RFC2119]
S. Bradner.
Key words for use in RFCs to Indicate Requirement Levels
. March 1997. Best Current Practice. URL:
https://datatracker.ietf.org/doc/html/rfc2119
Informative References
[CSS-MULTICOL-1]
Florian Rivoal; Rachel Andrew.
CSS Multi-column Layout Module Level 1
. 16 May 2024. CR. URL:
https://www.w3.org/TR/css-multicol-1/
[CSS-POSITION-3]
Elika Etemad; Tab Atkins Jr..
CSS Positioned Layout Module Level 3
. 7 October 2025. WD. URL:
https://www.w3.org/TR/css-position-3/
[CSS-WRITING-MODES-3]
Elika Etemad; Koji Ishii.
CSS Writing Modes Level 3
. 10 December 2019. REC. URL:
https://www.w3.org/TR/css-writing-modes-3/
Property Index
Name
Value
Initial
Applies to
Inh.
%ages
Anim­ation type
Canonical order
Com­puted value
flow-tolerance
normal | <length-percentage> | infinite
normal
grid lanes containers
no
relative to the grid-axis content box size of the grid lanes container
as length
per grammar
a computed <length-percentage> value
item-cross
[ auto | nowrap | wrap ] || [ normal | reverse ] | wrap-reverse
auto
flex containers, grid containers, grid lanes containers
no
N/A
discrete
per grammar
as specified
item-direction
auto | row | column | row-reverse | column-reverse
auto
flex containers, grid containers, grid lanes containers
no
N/A
discrete
per grammar
as specified
item-flow
<'item-direction'> || <'item-wrap'> || <'item-pack'> || <'flow-tolerance'>
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
item-pack
normal | dense || balance
normal
flex containers, grid containers, grid lanes containers
no
N/A
discrete
per grammar
as specified
item-track
auto | row | column | row-reverse | column-reverse
auto
flex containers, grid containers, grid lanes containers
no
N/A
discrete
per grammar
as specified
item-wrap
[ auto | nowrap | wrap ] || [ normal | reverse ] | wrap-reverse
auto
flex containers, grid containers, grid lanes containers
no
N/A
discrete
per grammar
as specified
Issues Index
Or should reordering be the default behavior for auto-placed items here?
↵
ISSUE(12820): Write up how grid/masonry formatting contexts work more formally?
↵
TBD
↵
Figure out whether we are re-using
grid-auto-flow
here (and what it’s values mean) or defining a new property like
grid-lanes-direction
.
[Issue #12803]
↵
Should this work also in Grid Layout somehow, or fall back to a single repetition? If so, how?
[Issue #10915]
↵
Technically, if your explicit placement does not enter or cross over the repetition, we can know its placement prior to resolving the reptition. Do we want to adjust the algorithm above to account for this? That is, given
auto repeat(auto-fill, ...) auto
, if you explicitly place an item against line 1 or -1, we could let it only affect that track (rather than pretending it’ll go in every track).
↵
What if this conflicts with the lanes orientation, or results in both axes stacking?
↵
Should this also update the placement cursor?
↵
Is this a reasonable definition for how the
self-alignment properties
should work in the
stacking axis
?
[Issue #10275]
↵
We could support baseline alignment in the first row. Do we want to?
↵
It might be useful to define a static position in the
stacking axis
. Maybe it could defined as the max (or min?) current
running position
of the
grid-axis
tracks at that point? Or the end of the item before it?
↵
This section is likely to move to another spec, such as
[css-display-4]
, since it affects multiple display types. It is also still under discussion as to whether this is a good idea.
↵
The CSSWG is still figuring out how these properties should be named and fit together.
[Issue #11480]
↵
There two open debates on this property: a) what should it be called and b) does it describe the primary direction of placement, or the orientation of the tracks into which items are placed; in other words, is the
primary axis
defined by this property the
primary placement axis
or the
primary track axis
. These are identical for
flex layout
and
grid layout
, but differ for
grid lanes layout
whose primary placement direction is across its tracks. See
latest discussion
.
[Issue #11480]
↵
The interpretation and naming of this property depends on the interpretation of axes for
item-direction
/
item-track
.
[Issue #11480]
↵
```
