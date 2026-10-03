Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Positioned Layout Module Level 3](https://www.w3.org/TR/2025/WD-css-position-3-20251007/).

Original copyright notice (from the matching exact HTML edition): Copyright © 2025 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Positioned Layout Module Level 3

Source snapshot: https://www.w3.org/TR/2025/WD-css-position-3-20251007/

Snapshot SHA-256: b75e4eb4bbe40ebe07ae3f4f14635cb218cb3ce63bbd2ec8af3fc75a25c6852b

Conversion: offline format conversion of the exact stored extracted-text; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- This stored input is an already extracted plain-text witness. It has no recoverable original HTML structure. The complete source text is preserved in a fenced block; no summary or specification regeneration was performed.

---

```text
CSS Positioned Layout Module Level 3
CSS Positioned Layout Module Level 3
W3C Working Draft
,
7 October 2025
More details about this document
This version:
https://www.w3.org/TR/2025/WD-css-position-3-20251007/
Latest published version:
https://www.w3.org/TR/css-position-3/
Editor's Draft:
https://drafts.csswg.org/css-position-3/
History:
https://www.w3.org/standards/history/css-position-3/
Feedback:
CSSWG Issues Repository
Inline In Spec
in Wiki
Editors:
Elika J. Etemad / fantasai
(
Apple
)
Tab Atkins Jr.
(
Google
)
Former Editors:
Rossen Atanassov
(
Microsoft
)
Arron Eicholz
(
Microsoft
)
Suggest an Edit for this Spec:
GitHub Editor
Copyright
© 2025
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
This module contains defines coordinate-based positioning and offsetting schemes of
CSS
:
relative positioning
,
sticky positioning
,
absolute positioning
, and
fixed positioning
.
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
(preferred), including the spec code “css-position” in the title, like this: “[css-position]
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
Module Interactions
1.2
Value Definitions
2
Choosing A Positioning Scheme:
position
property
2.1
Containing Blocks of Positioned Boxes
2.1.1
Further Adjustments to the Containing Block
2.2
Painting Order and Stacking Contexts
3
Positioning Coordinates
3.1
Box Insets: the
top
,
right
,
bottom
,
left
,
inset-block-start
,
inset-inline-start
,
inset-block-end
, and
inset-inline-end
properties
3.2
Box Insets Shorthands: the
inset-block
,
inset-inline
, and
inset
properties
3.3
Relative Positioning
3.4
Sticky positioning
3.4.1
Scroll Position of Sticky-Positioned Boxes
3.5
Absolute (and Fixed) Positioning
3.5.1
Resolvings Insets for the “Inset-Modified Containing Block”
3.5.2
Calculating the Static Position and the “Static-Position Rectangle”
3.5.3
Fragmenting Absolutely-positioned Elements
4
Absolute Positioning Layout Model
4.1
Automatic Sizes of Absolutely-Positioned Boxes
4.2
Auto Margins of Absolutely-Positioned Boxes
5
Self-Alignment of Absolutely Positioned Boxes
6
Old Absolute Positioning Layout Model
6.1
The Width of Absolutely-Positioned, Non-Replaced Elements
6.2
The width of absolute or fixed positioned, replaced elements
6.3
The Height Of Absolutely Positioned, Non-Replaced Elements
6.4
The Height Of Absolutely Positioned, Replaced Elements
7
Informative Comparison of Normal Flow, Floats, and Positioning
7.1
Normal Flow Example
7.2
Relative Positioning Example
7.3
Floating Example
7.4
Absolute Positioning Example
8
Acknowledgments
Changes
Privacy Considerations
Security Considerations
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
The CSS layout algorithms, by default, size and position boxes in relation to each other so that nothing overlaps.
This specification defines several ways to violate these assumptions when needed, moving elements around in ways that can make them overlap other content:
Relative positioning
, which visually shifts a box relative to its laid-out location.
Sticky positioning
, which visually shifts a box relative to its laid-out location in order to keep it visible when a scrollable ancestor would otherwise scroll it out of sight.
Absolute positioning
, which ignores normal layout entirely, pulling the element
out of flow
and positioning it relative to its
containing block
with no regard for other content.
Fixed positioning
, which absolutely positions the box and affixes it to the viewport or page frame so that it is always visible.
These
positioning schemes
, controlled by the
position
property and the
inset properties
, are powerful but easy to misuse. With appropriate care, they allow many interesting and useful layouts that couldn’t otherwise be achieved with standard layout rules; without, they allow a page to be laid out in an unusable overlapping jumble of content.
1.1.
Module Interactions
This module replaces and extends the
positioning scheme
features defined in
[CSS2]
sections:
9.1.2 Containing blocks
9.3 Positioning schemes
9.4.3 Relative positioning
9.6 Absolute positioning
9.7 Relationships between display, position, and float
9.8 Comparison of normal flow, floats, and absolute positioning
10.1 Definition of "containing block"
10.3.7 Absolutely positioned, non-replaced elements
10.3.8 Absolutely positioned, replaced elements
10.6.4 Absolutely positioned, non-replaced elements
10.6.5 Absolutely positioned replaced elements
It also replaces and supersedes the
inset*
property definitions in
[CSS-LOGICAL-1]
(
CSS Logical Properties 1
§ 4.3 Flow-relative Offsets: the inset-block-start, inset-block-end, inset-inline-start, inset-inline-end properties and inset-block, inset-inline, and inset shorthands
).
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
Choosing A Positioning Scheme:
position
property
Name:
position
Value:
static
|
relative
|
absolute
|
sticky
|
fixed
Initial:
static
Applies to:
all elements except table-column-group and table-column
Inherited:
no
Percentages:
N/A
Computed value:
specified keyword
Canonical order:
per grammar
Animation type:
discrete
The
position
property determines which of the
positioning schemes
is used to calculate the position of a box. Values other than
static
make the box a
positioned box
, and cause it to establish an
absolute positioning containing block
for its descendants. Values have the following meanings:
static
The box is not a
positioned box
, and is laid out according to the rules of its parent
formatting context
. The
inset properties
do not apply.
relative
The box is laid out as for
static
, then offset from the resulting position. This offsetting is a purely visual effect, and, unless otherwise specified, does not affect the size or position of any other non-descendant box except insofar as it increases the
scrollable overflow area
of its ancestors. This
positioning scheme
is called
relative positioning
.
sticky
Identical to
relative
, except that its offsets are automatically adjusted in reference to the nearest ancestor
scroll container’s
scrollport
(as modified by the
inset properties
) in whichever axes the
inset properties
are not both
auto
, to try to keep the box in view within its
containing block
as the user scrolls. This
positioning scheme
is called
sticky positioning
.
absolute
The box is taken
out of flow
such that it has no impact on the size or position of its siblings and ancestors, and does not participate in its parent’s
formatting context
.
Instead, the box is positioned and sized solely in reference to its
absolute positioning containing block
, as modified by the box’s
inset properties
, see
§ 4 Absolute Positioning Layout Model
. It can overlap
in-flow
content or other
absolutely positioned
elements, and is included in the
scrollable overflow area
of the box that generates is
containing block
. This
positioning scheme
is called
absolute positioning
.
fixed
Same as
absolute
, except the box is positioned and sized relative to a
fixed positioning containing block
(usually the
viewport
in
continuous media
, or the
page area
in
paged media
). The box’s position is fixed with respect to this reference rectangle: when attached to the
viewport
it does not move when the document is scrolled, and when attached to the
page area
is replicated on every page when the document is paginated. This
positioning scheme
is called
fixed positioning
and is considered a subset of
absolute positioning
.
Authors may wish to specify
fixed
in a media-dependent way. For instance, an author may want a box to remain at the top of the
viewport
on the screen, but not at the top of each printed page. The two specifications may be separated by using an
'@media'
rule, as in:
@media
screen
{
h1#first
{
position
:
fixed
}
}
@media
print
{
h1#first
{
position
:
static
}
}
A
position
value of
absolute
or
fixed
blockifies
the box, causes
float
to compute to
none
, and forces the box to
establish an independent formatting context
.
2.1.
Containing Blocks of Positioned Boxes
The
containing block
of a
static
,
relative
, or
sticky
box
is as defined by its
formatting context
. For
fixed
and
absolute
boxes, it is defined as follows:
If the box has
position: absolute
:
The
containing block
is established by the nearest ancestor box that establishes an
absolute positioning containing block
, in the following way:
If the ancestor is not an
inline box
,
the
containing block
is formed by the
padding edge
of the ancestor, unless otherwise specified (for example, see
CSS Grid Layout 1
§ 9.1 With a Grid Container as Containing Block
).
If the ancestor is an
inline box
, using the
writing mode
of that box,
the
containing block
is formed by forming a rectangle from the
start
-most
content edges
(in both axes) of the first
box fragment
of the ancestor, and the
end
-most
content edges
of the
end
-most
box fragment
(s) of the ancestor in each axis. If there are multiple fragments on the same line (e.g. due to
bidi reordering
), take the
start
-most fragment as the first fragment.
What is a useful containing block to form when the box is fragmented across multiple lines?
[Issue #8284]
Note:
The
containing block
formed by a fragmented
inline box
was undefined in
[CSS2]
.
If no ancestor establishes one, the
absolute positioning containing block
is the
initial containing block
.
Note:
Properties that can cause a box to establish an
absolute positioning containing block
include
position
,
transform
,
will-change
,
contain
…
If the box has
position: fixed
:
The
containing block
is established by the nearest ancestor box that establishes an
fixed positioning containing block
, with the bounds of the
containing block
determined identically to the
absolute positioning containing block
.
Note:
Properties that can cause a box to establish a
fixed positioning containing block
include
transform
,
will-change
,
contain
…
If no ancestor establishes one, the box’s
fixed positioning containing block
is the
initial fixed containing block
:
in
continuous media
, the
layout viewport
(whose size matches the
dynamic viewport size
); as a result,
fixed
boxes do not move when the document is scrolled.
Note:
In this respect, they are similar to
fixed background images
(
background-attachment: fixed
).
in
paged media
, the
page area
of each page;
fixed positioned
boxes
are thus replicated on every page. (They are fixed with respect to the page box only, and are not affected by being seen through a
viewport
; as in the case of print preview, for example.)
Note:
As a result, parts of
fixed-positioned boxes
that extend outside the
layout viewport
/
page area
cannot be scrolled to and will not print.
The
initial fixed containing block
is the parent of the
initial containing block
in the
containing block chain
.
With no positioning, the containing blocks (C.B.) in the following document:
<!DOCTYPE html>
<
html
>
<
head
>
<
title
>
Illustration of containing blocks
</
title
>
</
head
>
<
body
id
=
"body"
>
<
div
id
=
"div1"
>
<
p
id
=
"p1"
>
This is text in the first paragraph...
</
p
>
<
p
id
=
"p2"
>
This is text
<
em
id
=
"em1"
>
in the
<
strong
id
=
"strong1"
>
second
</
strong
>
paragraph.
</
em
></
p
>
</
div
>
</
body
>
</
html
>
are established as follows:
For box generated by
C.B. is established by
html
initial C.B. (UA-dependent)
body
html
div1
body
p1
div1
p2
div1
em1
p2
strong1
p2
If we position "div1":
#div1
{
position
:
absolute
;
left
:
50
px
;
top
:
50
px
}
its containing block is no longer "body"; it becomes the
initial containing block
(since there are no other positioned ancestor boxes).
If we position "em1" as well:
#div1
{
position
:
absolute
;
left
:
50
px
;
top
:
50
px
}
#em1
{
position
:
absolute
;
left
:
100
px
;
top
:
100
px
}
the table of containing blocks becomes:
For box generated by
C.B. is established by
html
initial C.B. (UA-dependent)
body
html
div1
initial C.B.
p1
div1
p2
div1
em1
div1
strong1
em1
By positioning "em1", its containing block becomes the nearest positioned ancestor box (i.e., that generated by "div1").
2.1.1.
Further Adjustments to the Containing Block
Some features can alter the effective
containing block
rectangle of
absolutely positioned
boxes. These are applied in the following order, with earlier steps modifying the
containing block
that later steps see:
The
grid-placement properties
on an
absolutely positioned box
whose
containing block
is generated by a
grid container
can change the
containing block
rectangle to a specified
grid area
. See
CSS Grid Layout 1
§ 9.1 With a Grid Container as Containing Block
.
The
position-area
and
position-try
properties can change the
containing block
rectangle to a specified area of an
position-area grid
. See
CSS Anchor Positioning
§ 3.1 The position-area Property
.
The element’s
original containing block
is its
containing block
before applying any of these effects.
2.2.
Painting Order and Stacking Contexts
The
z-index
property applies to all
positioned boxes
. When
z-index
is
auto
:
Fixed
and
sticky
positioned boxes
nonetheless form a
stacking context
.
Relative
and
absolute
positioned boxes
do not form a
stacking context
, but are painted as if those elements did generated new
stacking contexts
, except that their
positioned
descendants and any would-be child
stacking contexts
take part in the current
stacking context
.
Note:
The
root element
always forms a
stacking context
regardless.
See
CSS2 § 9.9 Layered presentation
and
Appendix E: Elaborate description of Stacking Contexts
for details about
z-index
,
stacking contexts
, and painting order.
3.
Positioning Coordinates
The precise location of a
positioned box
is controlled by the
inset properties
: the
physical
inset properties
top
,
right
,
bottom
,
left
; the
flow-relative
inset properties
inset-block-start
,
inset-inline-start
,
inset-block-end
, and
inset-inline-end
; and their
shorthands
,
inset-block
,
inset-inline
, and
inset
.
The interpretation of these
inset properties
varies by
positioning scheme
:
for
absolute positioning
, they represent insets from the containing block.
for
relative positioning
, they represent insets from the box’s original margin edge.
for
sticky positioning
, they represent insets from the
scrollport
edge.
3.1.
Box Insets: the
top
,
right
,
bottom
,
left
,
inset-block-start
,
inset-inline-start
,
inset-block-end
, and
inset-inline-end
properties
Name:
top
,
right
,
bottom
,
left
,
inset-block-start
,
inset-inline-start
,
inset-block-end
,
inset-inline-end
Value:
auto
|
<length-percentage>
Initial:
auto
Applies to:
positioned elements
Inherited:
no
Percentages:
refer to size of
containing block
; see prose
Computed value:
the keyword
auto
or a computed
<length-percentage>
value
Canonical order:
per grammar
Animation type:
by computed value type
Logical property group:
inset
These
inset properties
represent an inward “inset” on the corresponding side of the box (with respect to the box’s own
writing mode
; see
CSS Writing Modes 3
§ 6 Abstract Box Terminology
). For example,
top
represents a downward inset of the top edge. The
physical
and
flow-relative
properties interact as defined in
[CSS-LOGICAL-1]
. Values have the following meanings:
<length>
The inset is a fixed distance from the reference edge. Negative values are allowed.
<percentage>
The inset is a percentage relative to the
containing block
’s size in the corresponding axis (e.g. width for
left
or
right
, height for
top
and
bottom
). For
sticky positioned
boxes, the inset is instead relative to the relevant
scrollport
’s size. Negative values are allowed.
auto
Represents an unconstrained inset; the exact meaning depends on the
positioning scheme
.
Note:
For
fixed positioned
elements, using large values or negative values can easily move elements outside the
viewport
and make the contents unreachable through scrolling or other means.
3.2.
Box Insets Shorthands: the
inset-block
,
inset-inline
, and
inset
properties
Name:
inset-block
,
inset-inline
Value:
<'top'>
{1,2}
Initial:
auto
Applies to:
positioned elements
Inherited:
no
Percentages:
see individual properties
Computed value:
see individual properties
Canonical order:
per grammar
Animation type:
by computed value type
The
inset-block
and
inset-inline
properties are
shorthand properties
for setting
inset-block-start
+
inset-block-end
or
inset-inline-start
+
inset-inline-end
, respectively, in a single declaration. The first component value sets the
start
side, the second sets the
end
; if omitted, the second value defaults to the first.
Name:
inset
Value:
<'top'>
{1,4}
Initial:
auto
Applies to:
positioned elements
Inherited:
no
Percentages:
see individual properties
Computed value:
see individual properties
Canonical order:
per grammar
Animation type:
by computed value type
The
inset
property is a
shorthand property
that sets all of the
inset properties
in a single declaration, assigning values to the longhands representing each side exactly as the
margin
property does for its longhands.
By default, the
inset
property values are assigned to the corresponding
physical
longhand properties
—​
top
,
right
,
bottom
, and
left
—​which for historical reasons do not have an
inset-
prefix. This matches the behavior of other "4 values assigned to sides" properties, such as
margin
.
Allowing properties such as this to resolve to the
flow-relative
longhands
is under discussion in
[CSS-LOGICAL-1]
.
Yes, we understand it’s a little confusing that
inset
doesn’t expand to any
inset-*
properties.
3.3.
Relative Positioning
For a
relatively positioned
box, the
inset properties
move the box inward from the respective edge, without changing its size.
left
moves the box to the right,
right
moves it to the left, etc. Since boxes are not split or stretched as a result of
relative positioning
opposing
used values
in a given axis must be negations of each other:
If opposing
inset properties
in an axis both compute to
auto
(their
initial values
), their
used values
are zero (i.e., the boxes stay in their original position in that axis).
If only one is
auto
, its
used value
becomes the negation of the other, and the box is shifted by the specified amount.
If neither is
auto
, the position is over-constrained; (with respect to the
writing mode
of its
containing block
) the
computed
end
side value is ignored, and its
used value
becomes the negation of the
start
side.
The following three rules are equivalent, and shift the box
1em
to the left:
div.a8
{
position
:
relative
;
direction
:
ltr
;
left
:
-1
em
;
right
:
auto
}
div.a8
{
position
:
relative
;
direction
:
ltr
;
left
:
auto
;
right
:
1
em
}
div.a8
{
position
:
relative
;
direction
:
ltr
;
left
:
-1
em
;
right
:
5
em
}
If specified on a
table-row-group
,
table-header-group
,
table-footer-group
, or
table-row
box
the shift affects all the contents of the box, including all
table cells
that originate in the affected row, but not those that don’t.
Note:
Since
position
does not apply to
table-column-group
or
table-column
boxes, they are not affected by
relative positioning
.
3.4.
Sticky positioning
Sticky positioning
is similar to
relative positioning
except the offsets are automatically calculated in reference to the
nearest scrollport
.
For a
sticky positioned
box
, the
inset properties
represent insets from the respective edges of the
nearest scrollport
, defining the
sticky view rectangle
used to constrain the box’s position. (For this purpose an
auto
value represents a zero inset.) If this results in a
sticky view rectangle
size in any axis less than the size of the
border box
of the
sticky
box in that axis, then the effective
end
-edge inset in the affected axis is reduced (possibly becoming negative) to bring the
sticky view rectangle
’s size up to the size of the
border box
in that axis (where
end
is interpreted relative to the
writing mode
of the
containing block
).
For example, if the
nearest scrollport
is
300px
tall, the
sticky
box’s
border box
is
200px
tall, and it has
top: 20px
, then the top-edge inset of the
nearest scrollport
is
20px
, and the bottom-edge inset is
0px
, yielding a
sticky view rectangle
that is
280px
tall.
But if the
nearest scrollport
were only
100px
tall, then the effective bottom-edge inset becomes
-120px
, resulting in a
sticky view rectangle
that’s
200px
tall, enough to fully contain the
margin box
of the
sticky
box.
For each side of the box, if the corresponding
inset property
is not
auto
, and the corresponding
border edge
of the box would be outside the corresponding edge of the
sticky view rectangle
, then the box must be visually shifted (
as for relative positioning
) to be inward of that
sticky view rectangle
edge, insofar as it can while its
position box
remains contained within its
containing block
. The
position box
is its
margin box
, except that for any side for which the distance between its
margin edge
and the corresponding edge of its
containing block
is less than its corresponding
margin
, that distance is used in place of that
margin
.
Note:
A sticky positioned element with a non-
auto
top
value and an
auto
bottom
value will only ever be pushed down by sticky positioning; it will never be offset upwards.
Note:
Multiple
sticky positioned
boxes in the same container are offset independently, and therefore might overlap.
3.4.1.
Scroll Position of Sticky-Positioned Boxes
For the purposes of any operation targeting the scroll position of a sticky positioned element (or one of its descendants), the sticky positioned element must be considered to be at its offsetted position.
For example, if a user clicks a link targeting a
sticky-positioned
element, if the element’s
nearest scrollport
is currently scrolled such that the
sticky positioned
element is offset from its initial position, the
scroll container
will be scrolled back only the minimum necessary to bring it into its desired position in the
scrollport
(rather than scrolling all the way back to target its original, non-offsetted position).
3.5.
Absolute (and Fixed) Positioning
For an
absolutely positioned
box, the
inset properties
effectively reduce the
containing block
into which it is sized and positioned by the specified amounts. The resulting rectangle is called the
inset-modified containing block
. (For disambiguation, the actual
containing block
of an
absolutely positioned box
can also be called the
absolute-position containing block
.)
3.5.1.
Resolvings Insets for the “Inset-Modified Containing Block”
If only one
inset property
in a given axis is
auto
, it is set to zero. If both
inset properties
in a given axis are
auto
, then, depending on the box’s
self-alignment property
in the relevant axis:
for
self-start
alignment or its equivalent
Set its start-edge
inset property
to the
static position
, and its end-edge
inset property
to zero.
for
self-end
alignment or its equivalent
Set its end-edge
inset property
to the
static position
, and its start-edge
inset property
to zero.
for
center
alignment
Let
start distance
be the distance from the center of its
static-position rectangle
to the start edge of its
containing block
, and
end distance
be the distance from the center of its
static-position rectangle
to the end edge of its
containing block
. If
start distance
is less than or equal to
end distance
, then set the start-edge
inset property
to zero, and set the end-edge
inset property
to (
containing block size
- 2 × |
start distance
|); otherwise, set the end-edge
inset property
to zero and the start-edge
inset property
to (
containing block size
- 2 × |
end distance
|).
For the rules above, ignore
overflow alignment
, and treat
normal
as
start
and any
baseline
or
stretch
alignment value as its fallback alignment.
If these adjustments result in an effective
containing block
size in any axis less than zero, then the
weaker inset
in the affected axis is reduced (possibly becoming negative) to bring that size up to zero. In the case that only one inset is
auto
, that is the
weaker inset
(whose opposite inset is the
stronger inset
); otherwise the
weaker inset
is the inset of the
end
edge (where
end
is interpreted relative to the
writing mode
of the
containing block
).
Note:
Sizing and positioning of the
absolutely positioned box
into this
inset-modified containing block
is as described in
§ 4 Absolute Positioning Layout Model
.
If its
self-alignment property
in an axis is
normal
, then the
resolved value
of its
weaker inset
in that axis is the value necessary to match that edge of its
inset-modified containing block
to the corresponding edge of its
margin box
after
layout
. (Otherwise the
resolved value
is the
used value
described above.)
3.5.2.
Calculating the Static Position and the “Static-Position Rectangle”
When both
inset properties
in a given axis are
auto
, they are resolved into a
static position
by aligning the box into its
static-position rectangle
, an
alignment container
derived from the
formatting context
the box would have participated in if it were
position: static
(independent of its actual containing block). The
static position
represents an approximation of the position the box would have had if it were
position: static
.
Block Layout
The
static positions
of a
block-level box
are defined in
[CSS2]
Chapter 10. The
static-position rectangle
is a zero-thickness rectangle spanning between the inline-axis sides of the box’s
static-position containing block
and positioned at its
block-start
static position
(see
CSS2§10.6.4
).
Note:
In block layout the
static-position rectangle
corresponds to the position of the “hypothetical box” described in
CSS2.1§10.3.7
. Since it has no alignment properties, CSS2.1 always uses a
block-start
inline-start
alignment of the absolutely-positioned box within the
static-position rectangle
.
Inline Layout
The
static positions
of an
inline-level box
are defined in
[CSS2]
Chapter 10. The
static-position rectangle
is a zero-thickness rectangle spanning between the
line-over
/
line-under
sides of the
line box
that would have contained its “hypothetical box” (see
CSS2§10.3.7
); and positioned at its
inline-start
static position
.
Flex Layout
The
static-position rectangle
of the child of a
flex container
corresponds to the
content edges
of the
flex container
in the
cross axis
, and to the
outer edges
of its hypothetical position in the
main axis
. See
static position of a flex container child
in
[CSS-FLEXBOX-1]
.
Grid Layout
By default, the
static-position rectangle
of the child of a
grid container
corresponds to the
content edges
of the
grid container
. However, if that
grid container
also establishes the box’s actual
containing block
, then the
grid area
specified by the
grid-placement properties
establishes its
static-position rectangle
instead. See the
static position of a grid container child
in
[CSS-GRID-1]
.
Finding the
static position
and the
static-position rectangle
assumes that both
float
and
clear
(as well as
position
) have their
initial values
, and can consequently require assuming a different hypothetical value for
display
as well. (The
containing block
the element would have had under these conditions is the
static-position containing block
.) To the extent the
box alignment properties
have an effect, they use the
static-position containing block
as the effective
containing block
, including using its
writing mode
for resolving alignment axes and directions. Additionally, the
containing block
of
fixed positioned
elements is assumed to be the
initial containing block
instead of the
viewport
, and all
scroll containers
should be assumed to be scrolled to their
initial scroll position
. Lastly, all
auto
margins on the box itself are treated as zero.
Boxes in the
top layer
always use the
initial containing block
as their
static-position rectangle
.
3.5.3.
Fragmenting Absolutely-positioned Elements
In a
fragmented flow
, an
absolutely positioned box
is positioned relative to its
containing block
ignoring any
fragmentation breaks
(as if the flow were continuous). The box may subsequently be broken over several
fragmentation containers
.
For absolutely positioned content in
paged media
that resolves to a position on a page other than the page being laid out (the current page), or resolves to a position on the current page that has already been rendered for printing, printers may place the content:
on the current page,
on a subsequent page, or
may omit it altogether.
Note:
A
block-level
element that is split over several pages can have a different width on each page, and there may be device-specific limits.
User agents must not paginate the content of
fixed-positioned boxes
.
Note:
User agents might print invisible content in other ways. See
CSS Paged Media 3
§ 3.2 Content outside the page box
.
4.
Absolute Positioning Layout Model
Absolute positioning
not only takes a box
out of flow
, but also lays it out in its
containing block
(after the final size of the
containing block
has been determined) according to the
absolute positioning layout model
:
First, its
inset-modified containing block
is calculated, defining its
available space
. (See
§ 3.5 Absolute (and Fixed) Positioning
.)
Because an
absolutely positioned box
does not affect the size of its
containing block
, its
available space
is always
definite
.
Next, its width and height are resolved against this
definite
available space
, as its
preferred size
capped by its
maximum size
(if any), floored by its
minimum size
. See
§ 4.1 Automatic Sizes of Absolutely-Positioned Boxes
. Percentages, however, are resolved against the original
containing block
size.
Then, the value of any
auto
margins are calculated, see
§ 4.2 Auto Margins of Absolutely-Positioned Boxes
.
Lastly, its
margin box
is aligned within the
inset-modified containing block
, see
§ 5 Self-Alignment of Absolutely Positioned Boxes
.
4.1.
Automatic Sizes of Absolutely-Positioned Boxes
The
automatic size
of an
absolutely positioned box
is resolved against its
inset-modified containing block
as follows:
If its
self-alignment property
in the relevant axis is
stretch
and neither of its
inset properties
nor
margins
in that axis are
auto
Or if it is
normal
and the box is
non-replaced
, not a
table wrapper box
, and has no
auto
inset
in the relevant axis
Its
automatic size
is its
stretch-fit size
.
Otherwise
Its
automatic size
is its
fit-content size
.
However, if the box has an aspect-ratio, then an
automatic size
in the
ratio-dependent axis
is instead resolved as a
max-content size
. When both axes have an
automatic size
, if only one axis has an
auto
inset
then that axis is the
ratio-dependent axis
, else the
block axis
is the
ratio-dependent axis
. An
automatic size
in the
ratio-determining axis
is determined as above.
For the purpose of computing these sizes, any
auto
margins
are treated as zero.
The
automatic minimum size
of an absolutely-positioned box is always zero.
Note:
To the extent that form controls can be resized (and are not directly representing
replaced elements
such as images), they are expected to be treated as
non-replaced
here. In HTML, all form controls other than
<input type=image>
are treated as
non-replaced
.
4.2.
Auto Margins of Absolutely-Positioned Boxes
If either
inset property
in the relevant axis is
auto
, then any
auto
margins
resolve to zero.
Otherwise, the
remaining space
is calculated as the size of its
inset-modified containing block
in the relevant axis minus the box’s used size in the relevant axis, and this
remaining space
is divided among any
auto
margins in the relevant axis. However, (all with respect to the
writing mode
of the
containing block
), if in the
inline axis
the
remaining space
is negative and both margins are
auto
, the
start
margin resolves to zero and the
end
margin receives the
remaining space
.
Note:
Unlike typical
in-flow
layout, the space distributed to
auto
margins
can be negative in
absolute positioning
.
What should happen in “negatively-sized” containing blocks? CSS2.1 and this draft currently conflict.
[Issue #11478]
5.
Self-Alignment of Absolutely Positioned Boxes
If the
computed value
of either
inset property
in an axis is
auto
, then its
margin box
is aligned to the edge of the
inset-modified containing block
corresponding to its
stronger inset
(even if this would overflow its
containing block
).
Otherwise, if either
margin
is
auto
, its position is resolved according to
§ 4.2 Auto Margins of Absolutely-Positioned Boxes
.
Otherwise, the box is aligned as specified by its
self-alignment property
in the relevant axis (as defined by the
writing mode
of the
containing block
), using its
margin box
as the
alignment subject
and the
inset-modified containing block
as the
alignment container
. However, if an explicit
overflow alignment
is not specified and its
margin box
overflows the
inset-modified containing block
, its alignment is adjusted to minimize overflow as specified in
CSS Box Alignment 3
§ 4.4.1.2 Self-Alignment for Absolutely Positioned Boxes
.
6.
Old Absolute Positioning Layout Model
This section is being replaced with the new
§ 4 Absolute Positioning Layout Model
section. It is preserved here for comparison: both models should yield the same result in
horizontal writing modes
when the box’s
self-alignment
is
normal
.
6.1.
The Width of Absolutely-Positioned, Non-Replaced Elements
The constraint that determines the used values for these elements is:
left
+
margin-left
+
border-left-width
+
padding-left
+
width
+
padding-right
+
border-right-width
+
margin-right
+
right
= width of containing block
If all three of
left
,
width
, and
right
are
auto
: First set any
auto
values for
margin-left
and
margin-right
to
0
. Then, if the
direction
property of the element establishing the static-position
containing block
is
ltr
set
left
to the static position and apply rule number
three
below; otherwise, set
right
to the static-position and apply rule number
one
below.
If none of the three is
auto
: If both
margin-left
and
margin-right
are
auto
, solve the equation under the extra constraint that the two margins get equal values, unless this would make them negative, in which case when direction of the containing block is
ltr
(
rtl
), set
margin-left
(
margin-right
) to
0
and solve for
margin-right
(
margin-left
). If one of
margin-left
or
margin-right
is
auto
, solve the equation for that value. If the values are over-constrained, ignore the value for
left
(in case the
direction
property of the
containing block
is
rtl
) or
right
(in case
direction
is
ltr
) and solve for that value.
Otherwise, set
auto
values for
margin-left
and
margin-right
to
0
, and pick one of the following six rules that apply.
If
left
and
width
are
auto
and
right
is not
auto
, then the width is shrink-to-fit. Then solve for
left
.
If
left
and
right
are
auto
and
width
is not
auto
, then if the
direction
property of the element establishing the static-position
containing block
is
ltr
set
left
to the static-position, otherwise set
right
to the static-position. Then solve for
left
(if
direction
is
rtl
) or
right
(if
direction
is
ltr
).
If
width
and
right
are
auto
and
left
is not
auto
, then the width is shrink-to-fit. Then solve for
right
.
If
left
is
auto
,
width
and
right
are not
auto
, then solve for
left
.
If
width
is
auto
,
left
and
right
are not
auto
, then solve for
width
.
If
right
is
auto
,
left
and
width
are not
auto
, then solve for
right
.
Summary of rules for
dir=ltr
in horizontal writing modes
Is auto?
Result
left
width
right
margin-left
margin-right
✔
✔
✔
any
auto margins → zero
left → static pos
width → shrink-to-fit
right → solve
✘
✘
✘
✔
✘
auto margin → free space
✘
✔
✔
✔
margins split positive free space
right margin gets negative free space
✘
✘
treat
right
as
auto
✔
✘
✔
any
auto margins → zero
left → static pos
width → as specified
right → solve
✔
✔
✘
any
auto margins → zero
left → solve
width → shrink-to-fit
right → as specified
✘
✔
✔
any
auto margins → zero
left → as specified
width → shrink-to-fit
right → solve
✔
✘
✘
any
auto margins → zero
solve for auto
✘
✘
✔
✘
✔
✘
6.2.
The width of absolute or fixed positioned, replaced elements
If
height
and
width
both have computed values of
auto
and the element also has an intrinsic width, then that intrinsic width is the used value of
width
.
If
height
and
width
both have computed values of
auto
and the element has no intrinsic width, but does have an intrinsic height and intrinsic ratio; or if
width
has a computed value of
auto
,
height
has some other computed value, and the element does have an intrinsic ratio; then the used value of
width
is:
(used height) * (intrinsic ratio)
If
height
and
width
both have computed values of
auto
, the element has an intrinsic ratio but no intrinsic height or width, and the
containing block’s
width does not itself depend on the replaced element’s width, then the used value of
width
is calculated from the constraint equation used for
block-level, non-replaced elements in
normal flow
.
Otherwise, if
width
has a computed value of
auto
, and the element has an intrinsic width, then that intrinsic width is the used value of
width
.
Otherwise, if
width
has a computed value of
auto
, but none of the conditions above are met, and then the used value of
width
becomes
300px
. If
300px
is too wide to fit the device, user agents should use the width of the largest rectangle that has a 2:1 ratio and fits the device instead.
After establishing the
width
, in order to position the replaced element, apply the following rules as appropriate.
If both
left
and
right
have the value
auto
, and if the
direction
property of the element establishing the static-position
containing block
is
ltr
, set
left
to the static position and solve for
right
; else if
direction
is
rtl
, set
right
to the static position and solve for
left
.
If
left
is
auto
and
right
is not
auto
, replace any
auto
on
margin-left
or
margin-right
with
0
, then solve for
left
.
If
right
is
auto
and
left
is not
auto
, replace any
auto
on
margin-left
or
margin-right
with
0
, then solve for
right
.
If at this point both
margin-left
and
margin-right
are still
auto
, solve the equation under the extra constraint that the two margins must get equal values, unless this would make them negative, in which case when the direction of the
containing block
is
ltr
(
rtl
), set
margin-left
(
margin-right
) to
0
and solve for
margin-right
(
margin-left
).
If at this point there is an
auto
remaining, solve the equation for that value.
If at this point the values are over-constrained, ignore the value for either
left
(in case the
direction
property of the
containing block
is
rtl
) or
right
(in case
direction
is
ltr
) and solve for that value.
6.3.
The Height Of Absolutely Positioned, Non-Replaced Elements
For absolutely positioned elements, the used values of the vertical dimensions must satisfy this constraint:
top
+
margin-top
+
border-top-width
+
padding-top
+
height
+
padding-bottom
+
border-bottom-width
+
margin-bottom
+
bottom
=
height of containing block
If all three of
top
,
height
, and
bottom
are
auto
: First set any
auto
values for
margin-top
and
margin-bottom
to
0
, then set
top
to the static position, and finally apply rule number
three
below.
If none of the three are
auto
: If both
margin-top
and
margin-bottom
are
auto
, solve the equation under the extra constraint that the two margins get equal values. If one of
margin-top
or
margin-bottom
is
auto
, solve the equation for that value. If the values are over-constrained, ignore the value for
bottom
and solve for that value.
Otherwise, set
auto
values for
margin-top
and
margin-bottom
to
0
, and pick one of the following six rules that apply.
If
top
and
height
are
auto
and
bottom
is not
auto
, then the height is based on the
Auto heights for block formatting context roots
, and solve for
top
.
If
top
and
bottom
are
auto
and
height
is not
auto
, then set
top
to the static position, then solve for
bottom
.
If
height
and
bottom
are
auto
and
top
is not
auto
, then the height is based on the
Auto heights for block formatting context roots
, and solve for
bottom
.
If
top
is
auto
,
height
and
bottom
are not
auto
, then solve for
top
.
If
height
is
auto
,
top
and
bottom
are not
auto
, then solve for
height
.
If
bottom
is
auto
,
top
and
height
are not
auto
, then solve for
bottom
.
6.4.
The Height Of Absolutely Positioned, Replaced Elements
If
height
and
width
both have computed values of
auto
and the element also has an intrinsic height, then that intrinsic height is the used value of
height
.
Otherwise, if
height
has a computed value of
auto
and the element has an intrinsic ratio then the used value of
height
is:
(used width) / (intrinsic ratio)
Otherwise, if
height
has a computed value of
auto
and the element has an intrinsic height, then that intrinsic height is the used value of
height
.
Otherwise, if
height
has a computed value of
auto
, but none of the conditions above are met, then the used value of
height
must be set to the height of the largest rectangle that has a 2:1 ratio, has a height not greater than
150px
, and has a width not greater than the device width.
After establishing the
height
, in order to position the replaced element, apply the following rules as appropriate.
If both
top
and
bottom
have the value
auto
, replace
top
with the element’s static position.
If
bottom
is
auto
, replace any
auto
on
margin-top
or
margin-bottom
with
0
.
If at this point both
margin-top
and
margin-bottom
are still
auto
, solve the equation under the extra constraint that the two margins must get equal values.
If at this point there is only one
auto
remaining, solve the equation for that value.
If at this point the values are over-constrained, ignore the value for
bottom
and solve for that value.
7.
Informative Comparison of Normal Flow, Floats, and Positioning
This section is not normative.
To illustrate the differences between
normal flow
,
relative positioning
,
floats
, and
absolute positioning
, we provide a series of examples based on the following HTML:
<!DOCTYPE html>
<
html
>
<
head
>
<
title
>
Comparison of positioning schemes
</
title
>
<
style
>
body
{
display
:
block
;
font-size
:
12
px
;
line-height
:
200
%
;
width
:
400
px
;
height
:
400
px
}
p
{
display
:
block
}
span
{
display
:
inline
}
</
style
>
</
head
>
<
body
>
<
p
>
Beginning of p contents.
<
span
id
=
"outer"
>
Start of outer contents.
<
span
id
=
"inner"
>
Inner contents.
</
span
>
End of outer contents.
</
span
>
End of p contents.
</
p
>
</
body
>
</
html
>
The final positions of boxes generated by the
outer
and
inner
elements vary in each example. In each illustration, the numbers to the left of the illustration indicate the
normal flow
position of the double-spaced (for clarity) lines.
Note:
The diagrams in this section are illustrative and not to scale. They are meant to highlight the differences between the various
positioning schemes
, and are not intended to be reference renderings of the examples given.
7.1.
Normal Flow Example
Consider the following CSS declarations for
outer
and
inner
that do not alter the
normal flow
of boxes:
#outer
{
color
:
red
}
#inner
{
color
:
blue
}
The P element contains all inline content:
anonymous inline text
and two SPAN elements. Therefore, all of the content will be laid out in an inline formatting context, within a
containing block
established by the P element, producing something like:
All of the text within the P’s containing block flows together as continuous text, even though it’s located in separated nested elements.
7.2.
Relative Positioning Example
To see the effect of
relative positioning
, we specify:
#outer
{
position
:
relative
;
top
:
-12
px
;
color
:
red
}
#inner
{
position
:
relative
;
top
:
12
px
;
color
:
blue
}
Text flows normally up to the
outer
element. The
outer
text is then flowed into its
normal flow
position and dimensions at the end of line 1. Then, the inline boxes containing the text (distributed over three lines) are shifted as a unit by
-12px
(upwards).
The contents of
inner
, as a child of
outer
, would normally flow immediately after the words "of outer contents" (on line 1.5). However, the
inner
contents are themselves offset relative to the
outer
contents by
12px
(downwards), back to their original position on line 2.
Note that the content following
outer
is not affected by the relative positioning of
outer
.
The result is identical to normal flow, except that the "outer" text is shifted 12px upward, without affecting the flow of the "body" or "inner" text.
Note also that had the offset of
outer
been
-24px
, the text of
outer
and the body text would have overlapped.
7.3.
Floating Example
Now consider the effect of
floating
the
inner
element’s text to the right by means of the following rules:
#outer
{
color
:
red
}
#inner
{
float
:
right
;
width
:
130
px
;
color
:
blue
}
Text flows normally up to the
inner
box, which is pulled out of the flow and
floated
to the right margin (its
width
has been assigned explicitly). Line boxes to the left of the float are shortened, and the document’s remaining text flows into them.
The "inner" text lays out in an independent box on the right, causing the remaining "body" and "outer" text to flow around it.
To show the effect of the
clear
property, we add a
sibling
element to the example:
<!DOCTYPE html>
<
html
>
<
head
>
<
title
>
Comparison of positioning schemes II
</
title
>
<
style
>
#
inner
{
float
:
right
;
width
:
130
px
;
color
:
blue
}
#
sibling
{
color
:
red
}
</
style
>
</
head
>
<
body
>
<
p
>
Beginning of p contents.
<
span
id
=
"outer"
>
Start of outer contents.
<
span
id
=
"inner"
>
Inner contents.
</
span
>
<
span
id
=
"sibling"
>
Sibling contents.
</
span
>
End of outer contents.
</
span
>
End of p contents.
</
p
>
</
body
>
</
html
>
These styles cause the
inner
box to float to the right, as before, and the document’s remaining text to flow into the vacated space:
Identical to the previous example, save that there is now "sibling" text flowing with the "body" and "outer" text.
However, if the
clear
property on the
sibling
element is set to
right
(i.e., the generated
sibling
box will not accept a position next to
floating
boxes to its right), the
sibling
content begins to flow below the float:
#inner
{
float
:
right
;
width
:
130
px
;
color
:
blue
}
#sibling
{
clear
:
right
;
color
:
red
}
Now the "sibling" text moves down to below the "inner" text’s box, leaving blank space behind. The text following the "sibling" text flows after it as normal.
7.4.
Absolute Positioning Example
Next, we consider the effect of absolute positioning. Consider the following CSS declarations for
outer
and
inner
:
#outer
{
position
:
absolute
;
top
:
200
px
;
left
:
200
px
;
width
:
200
px
;
color
:
red
;
}
#inner
{
color
:
blue
}
which cause the top of the
outer
box to be positioned with respect to its
containing block
. The
containing block
for a positioned box is established by the nearest positioned ancestor (or, if none exists, the
initial containing block
, as in our example). The top side of the
outer
box is
200px
below the top of the
containing block
and the left side is
200px
from the left side. The child box of
outer
is flowed normally with respect to its parent.
All of the text within #outer (the "outer" and "inner" text) moves down to an independent box in the lower right corner. The two halves of "body" text flow together.
The following example shows an absolutely positioned box that is a child of a relatively positioned box. Although the parent
outer
box is not actually offset, setting its
position
property to
relative
means that its box may serve as the containing block for positioned descendants. Since the
outer
box is an inline box that is split across several lines, the first inline box’s top and left edges (depicted by thick dashed lines in the illustration below) serve as references for
top
and
left
offsets.
#outer
{
position
:
relative
;
color
:
red
}
#inner
{
position
:
absolute
;
top
:
200
px
;
left
:
-100
px
;
height
:
130
px
;
width
:
130
px
;
color
:
blue
;
}
This results in something like the following:
The "inner" text is positioned in an independent box, relative to the top-left corner of the start of the "outer" text.
If we do not position the
outer
box:
#outer
{
color
:
red
}
#inner
{
position
:
absolute
;
top
:
200
px
;
left
:
-100
px
;
height
:
130
px
;
width
:
130
px
;
color
:
blue
;
}
the
containing block
for
inner
becomes the
initial containing block
(in our example). The following illustration shows where the
inner
box would end up in this case.
Same as before, except now the "inner text" is positioned relative to the top-left corner of the page itself.
Relative and absolute positioning may be used to implement change bars, as shown in the following example. The following fragment:
<
p
style
=
"position: relative; margin-right: 10px; left: 10px;"
>
I used two red hyphens to serve as a change bar. They will "float" to the left of the line containing THIS
<
span
style
=
"position: absolute; top: auto; left: -1em; color: red;"
>
--
</
span
>
word.
</
p
>
might result in something like:
The two red hyphens, indicating a change, sit in the left margin of the page on the line containing the word "THIS", regardless of what line that ends up being.
First, the paragraph (whose
containing block
sides are shown in the illustration) is flowed normally. Then it is offset
10px
from the left edge of the
containing block
(thus, a right margin of
10px
has been reserved in anticipation of the offset). The two hyphens acting as change bars are taken out of the flow and positioned at the current line (due to 'top: auto'),
-1em
from the left edge of its containing block (established by the P in its final position). The result is that the change bars seem to "float" to the left of the current line.
8.
Acknowledgments
This module would not have been possible without input and support from many helpful people. Thanks to Rossen Atanassov, Bert Bos, Oriol Brufau, Tantek Çelik, Arron Eicholz Sylvain Galineau, John Jansen, Chris Jones, Ian Kilpatrick, Anton Prowse.
Changes
Significant changes since the
11 March 2025 Working Draft
:
Defined the
initial fixed containing block
and its relationship to the
initial containing block
, to give this concept a name.
See also
Previous Changes
.
Privacy Considerations
This specification introduces no new privacy considerations.
Security Considerations
If an attacker is able to inject arbitrary CSS, positioned layout can make it easier to position elements the attacker has control of over arbitrary other elements of the page, potentially tricking users of the page. (There are many routes to this attack: negative
margin
,
transform
, etc. Don’t let people apply arbitrary CSS to bits of your page.)
position: fixed
can allow a page to emulate modal dialogs, potentially tricking a user into thinking they’re interacting with the user agent and entering in sensitive information that they page can then capture. User agents must ensure that their native dialogs are positioned in ways that the page cannot emulate; in particular, that at least some of the dialog is outside the "poisoned pixels" that web content can paint to.
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
absolute
dfn for
, in § 2
value for position
, in § 2
absolutely
, in § 2
absolutely position
, in § 2
absolutely-positioned
, in § 2
absolutely positioned box
, in § 2
absolutely positioned element
, in § 2
absolute position
, in § 2
absolute-position containing block
, in § 3.5
absolute positioning containing block
, in § 2
absolute positioning layout
, in § 4
absolute positioning layout model
, in § 4
auto
, in § 3.1
bottom
, in § 3.1
fixed
dfn for
, in § 2
value for position
, in § 2
fixed position
, in § 2
fixed-positioned
, in § 2
fixed-positioned box
, in § 2
fixed positioning containing block
, in § 2.1
initial fixed containing block
, in § 2.1
inset
, in § 3.2
inset-block
, in § 3.2
inset-block-end
, in § 3.1
inset-block-start
, in § 3.1
inset-inline
, in § 3.2
inset-inline-end
, in § 3.1
inset-inline-start
, in § 3.1
inset-modified containing block
, in § 3.5
inset properties
, in § 3
left
, in § 3.1
<length>
, in § 3.1
original containing block
, in § 2.1.1
<percentage>
, in § 3.1
position
, in § 2
position box
, in § 3.4
positioned
, in § 2
positioned box
, in § 2
positioning scheme
, in § 2
relative
dfn for
, in § 2
value for position
, in § 2
relatively
, in § 2
relatively position
, in § 2
relatively-positioned
, in § 2
relatively positioned box
, in § 2
relative position
, in § 2
right
, in § 3.1
static
, in § 2
static position
, in § 3.5.2
static-position containing block
, in § 3.5.2
static-position rectangle
, in § 3.5.2
sticky
dfn for
, in § 2
value for position
, in § 2
sticky position
, in § 2
sticky-positioned
, in § 2
sticky-positioned box
, in § 2
sticky view rectangle
, in § 3.4
stronger inset
, in § 3.5.1
top
, in § 3.1
weaker inset
, in § 3.5.1
Terms defined by reference
[CSS-ALIGN-3]
defines the following terms:
alignment container
alignment subject
baseline alignment
box alignment properties
center
normal
overflow alignment
self-alignment
self-alignment properties
self-end
self-start
start
stretch
[CSS-ANCHOR-POSITION-1]
defines the following terms:
position-area
position-area grid
position-try
[CSS-BACKGROUNDS-3]
defines the following terms:
background-attachment
border-bottom-width
border-left-width
border-right-width
border-top-width
[CSS-BOX-4]
defines the following terms:
border box
border edge
content edge
margin
margin box
margin edge
margin-bottom
margin-left
margin-right
margin-top
outer edge
padding edge
padding-bottom
padding-left
padding-right
padding-top
[CSS-BREAK-4]
defines the following terms:
box fragment
fragmentation break
fragmentation container
fragmented flow
[CSS-CASCADE-5]
defines the following terms:
computed value
initial value
longhand
longhand property
shorthand
shorthand property
used value
[CSS-CONTAIN-2]
defines the following terms:
contain
[CSS-DISPLAY-4]
defines the following terms:
block-level
block-level box
blockify
box
containing block
containing block chain
display
establish an independent formatting context
formatting context
in-flow
initial containing block
inline box
inline-level box
non-replaced
out of flow
replaced element
root element
table-cell
table-column
table-column-group
table-footer-group
table-header-group
table-row
table-row-group
[CSS-FLEXBOX-1]
defines the following terms:
cross axis
flex container
main axis
[CSS-GRID-2]
defines the following terms:
grid area
grid container
grid-placement property
[CSS-INLINE-3]
defines the following terms:
line box
[CSS-OVERFLOW-3]
defines the following terms:
initial scroll position
nearest scrollport
scroll container
scrollable overflow area
scrollport
[CSS-PAGE-3]
defines the following terms:
page area
[CSS-PAGE-FLOATS-3]
defines the following terms:
float
[CSS-POSITION-4]
defines the following terms:
top layer
[CSS-SIZING-3]
defines the following terms:
auto
automatic minimum size
automatic size
available space
definite
fit-content size
height
max-content size
maximum size
minimum size
preferred size
stretch-fit size
width
[CSS-SIZING-4]
defines the following terms:
ratio-dependent axis
ratio-determining axis
[CSS-TABLES-3]
defines the following terms:
table wrapper box
[CSS-TRANSFORMS-1]
defines the following terms:
transform
[CSS-VALUES-4]
defines the following terms:
<length-percentage>
CSS-wide keywords
dynamic viewport size
{A,B}
|
[CSS-WILL-CHANGE-1]
defines the following terms:
will-change
[CSS-WRITING-MODES-3]
defines the following terms:
direction
[CSS-WRITING-MODES-4]
defines the following terms:
block axis
block-start
end
flow-relative
horizontal writing mode
inline axis
inline-end
inline-start
line-over
line-under
ltr
physical
rtl
start
writing mode
[CSS2]
defines the following terms:
auto
clear
float
none
normal flow
stacking context
viewport
z-index
[CSSOM-1]
defines the following terms:
resolved value
[CSSOM-VIEW-1]
defines the following terms:
layout viewport
[MEDIAQUERIES-5]
defines the following terms:
continuous media
paged media
References
Normative References
[CSS-ALIGN-3]
Elika Etemad; Tab Atkins Jr..
CSS Box Alignment Module Level 3
. 11 March 2025. WD. URL:
https://www.w3.org/TR/css-align-3/
[CSS-ANCHOR-POSITION-1]
Tab Atkins Jr.; Elika Etemad; Ian Kilpatrick.
CSS Anchor Positioning
. 9 May 2025. WD. URL:
https://www.w3.org/TR/css-anchor-position-1/
[CSS-BACKGROUNDS-3]
Elika Etemad; Brad Kemper.
CSS Backgrounds and Borders Module Level 3
. 11 March 2024. CRD. URL:
https://www.w3.org/TR/css-backgrounds-3/
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
. 19 December 2024. FPWD. URL:
https://www.w3.org/TR/css-display-4/
[CSS-FLEXBOX-1]
Tab Atkins Jr.; et al.
CSS Flexible Box Layout Module Level 1
. 19 November 2018. CR. URL:
https://www.w3.org/TR/css-flexbox-1/
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
[CSS-INLINE-3]
Elika Etemad.
CSS Inline Layout Module Level 3
. 18 December 2024. WD. URL:
https://www.w3.org/TR/css-inline-3/
[CSS-LOGICAL-1]
Rossen Atanassov; Elika Etemad.
CSS Logical Properties and Values Level 1
. 27 August 2018. WD. URL:
https://www.w3.org/TR/css-logical-1/
[CSS-OVERFLOW-3]
Elika Etemad; Florian Rivoal.
CSS Overflow Module Level 3
. 29 March 2023. WD. URL:
https://www.w3.org/TR/css-overflow-3/
[CSS-PAGE-3]
Elika Etemad.
CSS Paged Media Module Level 3
. 14 September 2023. WD. URL:
https://www.w3.org/TR/css-page-3/
[CSS-PAGE-FLOATS-3]
Johannes Wilm.
CSS Page Floats
. 15 September 2015. FPWD. URL:
https://www.w3.org/TR/css-page-floats-3/
[CSS-POSITION-4]
CSS Positioned Layout Module Level 4
. 8 July 2025. FPWD. URL:
https://www.w3.org/TR/css-position-4/
[CSS-SIZING-3]
Tab Atkins Jr.; Elika Etemad.
CSS Box Sizing Module Level 3
. 17 December 2021. WD. URL:
https://www.w3.org/TR/css-sizing-3/
[CSS-SIZING-4]
Tab Atkins Jr.; Elika Etemad; Jen Simmons.
CSS Box Sizing Module Level 4
. 20 May 2021. WD. URL:
https://www.w3.org/TR/css-sizing-4/
[CSS-TABLES-3]
François Remy; Greg Whitworth; David Baron.
CSS Table Module Level 3
. 27 July 2019. WD. URL:
https://www.w3.org/TR/css-tables-3/
[CSS-TRANSFORMS-1]
Simon Fraser; et al.
CSS Transforms Module Level 1
. 14 February 2019. CR. URL:
https://www.w3.org/TR/css-transforms-1/
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
[CSS-WRITING-MODES-3]
Elika Etemad; Koji Ishii.
CSS Writing Modes Level 3
. 10 December 2019. REC. URL:
https://www.w3.org/TR/css-writing-modes-3/
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
[CSSOM-1]
Daniel Glazman; Emilio Cobos Álvarez.
CSS Object Model (CSSOM)
. 26 August 2021. WD. URL:
https://www.w3.org/TR/cssom-1/
[CSSOM-VIEW-1]
Simon Fraser; Emilio Cobos Álvarez.
CSSOM View Module
. 16 September 2025. WD. URL:
https://www.w3.org/TR/cssom-view-1/
[MEDIAQUERIES-5]
Dean Jackson; et al.
Media Queries Level 5
. 18 December 2021. WD. URL:
https://www.w3.org/TR/mediaqueries-5/
[RFC2119]
S. Bradner.
Key words for use in RFCs to Indicate Requirement Levels
. March 1997. Best Current Practice. URL:
https://datatracker.ietf.org/doc/html/rfc2119
Informative References
[CSS-CONTAIN-2]
Tab Atkins Jr.; Florian Rivoal; Vladimir Levin.
CSS Containment Module Level 2
. 17 September 2022. WD. URL:
https://www.w3.org/TR/css-contain-2/
[CSS-WILL-CHANGE-1]
Tab Atkins Jr..
CSS Will Change Module Level 1
. 5 May 2022. CRD. URL:
https://www.w3.org/TR/css-will-change-1/
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
Logical property group
bottom
auto | <length-percentage>
auto
positioned elements
no
refer to size of containing block; see prose
by computed value type
per grammar
the keyword auto or a computed <length-percentage> value
inset
inset
<'top'>{1,4}
auto
positioned elements
no
see individual properties
by computed value type
per grammar
see individual properties
inset-block
<'top'>{1,2}
auto
positioned elements
no
see individual properties
by computed value type
per grammar
see individual properties
inset-block-end
auto | <length-percentage>
auto
positioned elements
no
refer to size of containing block; see prose
by computed value type
per grammar
the keyword auto or a computed <length-percentage> value
inset
inset-block-start
auto | <length-percentage>
auto
positioned elements
no
refer to size of containing block; see prose
by computed value type
per grammar
the keyword auto or a computed <length-percentage> value
inset
inset-inline
<'top'>{1,2}
auto
positioned elements
no
see individual properties
by computed value type
per grammar
see individual properties
inset-inline-end
auto | <length-percentage>
auto
positioned elements
no
refer to size of containing block; see prose
by computed value type
per grammar
the keyword auto or a computed <length-percentage> value
inset
inset-inline-start
auto | <length-percentage>
auto
positioned elements
no
refer to size of containing block; see prose
by computed value type
per grammar
the keyword auto or a computed <length-percentage> value
inset
left
auto | <length-percentage>
auto
positioned elements
no
refer to size of containing block; see prose
by computed value type
per grammar
the keyword auto or a computed <length-percentage> value
inset
position
static | relative | absolute | sticky | fixed
static
all elements except table-column-group and table-column
no
N/A
discrete
per grammar
specified keyword
right
auto | <length-percentage>
auto
positioned elements
no
refer to size of containing block; see prose
by computed value type
per grammar
the keyword auto or a computed <length-percentage> value
inset
top
auto | <length-percentage>
auto
positioned elements
no
refer to size of containing block; see prose
by computed value type
per grammar
the keyword auto or a computed <length-percentage> value
inset
Issues Index
What is a useful containing block to form when the box is fragmented across multiple lines?
[Issue #8284]
↵
What should happen in “negatively-sized” containing blocks? CSS2.1 and this draft currently conflict.
[Issue #11478]
↵
This section is being replaced with the new
§ 4 Absolute Positioning Layout Model
section. It is preserved here for comparison: both models should yield the same result in
horizontal writing modes
when the box’s
self-alignment
is
normal
.
↵
```
