Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Overflow Module Level 3](https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/).

Original copyright notice (from the matching exact HTML edition): Copyright © 2025 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Overflow Module Level 3

Source snapshot: https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/

Snapshot SHA-256: 090d7d316c74cd9f5f7438f7a8765e9d54ca64d43acce7ed0d25d78dfab122ee

Conversion: offline format conversion of the exact stored extracted-text; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- This stored input is an already extracted plain-text witness. It has no recoverable original HTML structure. The complete source text is preserved in a fenced block; no summary or specification regeneration was performed.

---

```text
CSS Overflow Module Level 3
CSS Overflow Module Level 3
W3C Working Draft
,
7 October 2025
More details about this document
This version:
https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/
Latest published version:
https://www.w3.org/TR/css-overflow-3/
Editor's Draft:
https://drafts.csswg.org/css-overflow-3/
Previous Versions:
https://www.w3.org/TR/2021/WD-css-overflow-3-20211223/
History:
https://www.w3.org/standards/history/css-overflow-3/
Feedback:
CSSWG Issues Repository
Inline In Spec
Editors:
Elika J. Etemad / fantasai
(
Apple
)
Florian Rivoal
(
On behalf of Bloomberg
)
Former Editor:
L. David Baron
(
Mozilla
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
This module contains the features of CSS relating to scrollable overflow handling in visual media. This level is focused on completing a precise specification for the existing overflow features, including the
overflow
property and its longhands; and the
text-overflow
property. A few additional features introduced in support of
[CSS-CONTAIN-1]
and
[CSS-CONTAIN-2]
are also defined:
overflow: clip
and the
overflow-clip-margin
property.
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
(preferred), including the spec code “css-overflow” in the title, like this: “[css-overflow]
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
The description of
overflow
and its longhands is considered significantly more complete and correct than previous working drafts or than
[CSS2]
, but a few questions and issues remain open.
overflow: clip
and
overflow-clip-margin
are rather new, and lack implementation experience.
text-overflow
is stable, is unchanged form its earlier definition in
[CSS-UI-3]
. While not yet fully validated by implementation experience, the design of
line-clamp
and its longhands is considered roughly complete. Earlier versions of this specification included experimental new ideas for handling overflow by creating new boxes using fragmentation. These ideas are not abandoned; they are merely
deferred until Level 4
. Work will resume on fragmented overflow once this level stabilizes completed.
The following features are at-risk, and may be dropped during the CR period:
the
max-lines
property
“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.
Table of Contents
1
Introduction
1.1
Value Definitions
1.2
Module Interactions
2
Overflow Concepts and Terminology
2.1
Ink Overflow
2.2
Scrollable Overflow
2.3
Scrolling Overflow
3
Scrolling and Clipping Overflow
3.1
Managing Overflow: the
overflow-x
,
overflow-y
, and
overflow
properties
3.1.1
Interaction of
visibility
and
overflow
3.1.2
Interaction of
border-radius
and
overflow
3.1.3
Overflow in Print and Other Static Media
3.2
Expanding Clipping Bounds: the
overflow-clip-margin
property
3.3
Overflow Viewport Propagation
3.4
Smooth Scrolling: the
scroll-behavior
Property
4
Scrollbars and Layout
4.1
Scrollbar Contributions to Sizing
4.2
Reserving space for the scrollbar: the
scrollbar-gutter
property
5
Automatic Ellipses
5.1
Overflow Ellipsis: the
text-overflow
property
Appendix A. Privacy Considerations
Appendix B. Security Considerations
Appendix C. Changes
Acknowledgments
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
In CSS Level 1
[CSS1]
, placing more content than would fit inside an element with a specified size was generally an authoring error. Doing so caused the content to extend outside the bounds of the element, which would likely cause that content to overlap with other elements.
CSS Level 2
[CSS2]
introduced the
overflow
property, which allows authors to have overflow be handled by scrolling, which means it is no longer an authoring error. It also allows authors to specify that overflow is handled by clipping, which makes sense when the author’s intent is that the content not be shown.
This specification introduces the long-standing de-facto
overflow-x
and
overflow-y
properties, adds a
clip
value, and defines overflow handling more fully.
[Something something
max-lines
.]
Note:
This specification also reproduces the definition of the
text-overflow
property previously defined in
[CSS-UI-3]
, with no addition or modification, in order to present
text-overflow
and
block-ellipsis
together.
1.1.
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
1.2.
Module Interactions
This module replaces (supersedes) and extends features defined in
[CSS2]
section
11.1 Overflow and clipping
and
[CSS-UI-3]
section
5.2. Overflow Ellipsis: the text-overflow property
.
2.
Overflow Concepts and Terminology
CSS uses the term
overflow
to describe the contents of a box that extend outside one of that box’s edges (i.e., its
content edge
,
padding edge
,
border edge
, or
margin edge
). The term might be interpreted as elements or features that cause this overflow, the non-rectangular region occupied by these features, or, more commonly, as the minimal rectangle that bounds that region. A box’s overflow is computed based on the layout and styling of the box itself and of all descendants whose
containing block chain
includes the box.
In most cases,
overflow
can be computed for any box from the bounds and properties of that box itself, plus the
overflow
of each of its children. However, this is not always the case; for example, when
transform-style: preserve-3d
[CSS3-TRANSFORMS]
is used on some of the children, any of their descendants with
transform-style: preserve-3d
must also be examined.
There are two different types of overflow, which are used for different purposes by the UA:
ink overflow
scrollable overflow
2.1.
Ink Overflow
The
ink overflow
of a box is the part of that box and its contents that creates a visual effect outside of the box’s border box. Ink overflow is the overflow of painting effects defined to not affect layout or otherwise extend the
scrollable overflow area
, such as
box shadows
,
border images
,
text decoration
, overhanging glyphs (with negative side bearings, or with ascenders/descenders extending outside the em box),
outlines
, etc.
Since some effects in CSS (for example, the blurs in
text-shadow
[CSS-TEXT-3]
and
box-shadow
[CSS-BACKGROUNDS-3]
, which are theoretically infinite) do not define what visual extent they cover, the extent of the
ink overflow
is undefined.
The
ink overflow area
is the non-rectangular area occupied by the
ink overflow
of a box and its contents, and the
ink overflow rectangle
is the minimal rectangle whose axes are aligned to the box’s axes and that contains the
ink overflow area
. Note that the
ink overflow rectangle
is a rectangle in the box’s coordinate system, but might be non-rectangular in other coordinate systems due to transforms.
[CSS3-TRANSFORMS]
Any overflow of
replaced
content is always
ink overflow
(as opposed to
scrollable overflow
).
2.2.
Scrollable Overflow
The
scrollable overflow
of a box is the set of things extending outside of that box’s padding edge for which a scrolling mechanism needs to be provided.
The
scrollable overflow area
is the non-rectangular region occupied by the
scrollable overflow
, and the
scrollable overflow rectangle
is the minimal rectangle whose axes are aligned to the box’s axes and that contains the
scrollable overflow area
.
The
scrollable overflow area
is the union of:
The
scroll container
’s own
padding box
.
All
line boxes
directly contained by the
scroll container
.
The border boxes of all boxes for which it is the containing block and whose border boxes are positioned not wholly in the
unreachable scrollable overflow region
, accounting for transforms by projecting each box onto the plane of the element that establishes its
3D rendering context
.
[CSS3-TRANSFORMS]
Is this description of handling transforms sufficiently accurate?
Border boxes with zero area do not affect the
scrollable overflow area
.
The margin areas of
grid item
and
flex item
boxes for which the box establishes a containing block.
The UA may
additionally
include the margin areas of other boxes for which the box establishes a containing block; however, the conditions under which such margin areas are included is undefined in this level.
This needs further testing and investigation; is therefore deferred in this draft.
The
scrollable overflow areas
of all of the above boxes (including zero-area boxes and accounting for transforms as described above), provided they themselves have
overflow: visible
(i.e. do not themselves trap the overflow) and that
scrollable overflow
is not already clipped (e.g. by the
clip
property or the
contain
property).
Note:
The
mask-*
properties
[CSS-MASKING-1]
do not affect the
scrollable overflow area
.
Need to evaluate what should/should not clip
scrollable overflow
.
[Issue #8607]
Additional padding added to the
scrollable overflow rectangle
as necessary to enable scroll positions that satisfy the requirements of both
place-content: start
and
place-content: end
alignment.
Note:
This padding represents, within the
scrollable overflow rectangle
, the box’s own padding so that when its content is scrolled to its end, there is padding between the edge of its in-flow (or floated) content and the border edge of the box. It typically ends up being exactly the same size as the box’s own padding, except in a few cases—​such as when an out-of-flow positioned element, or the visible overflow of a descendent, has already increased the size of the
scrollable overflow rectangle
outside the conceptual “content edge” of the
scroll container
’s content.
Issue: Replace this image with a proper SVG.
Additionally, due to Web-compatibility constraints (caused by authors exploiting legacy bugs to surreptitiously hide content from visual readers but not search engines and/or speech output), UAs must clip any content in the
unreachable scrollable overflow region
.
Note:
The
content-distribution properties
can
alter the unreachable scrollable overflow region
to ensure that a
scroll container
’s
alignment subject
is reachable after alignment.
[CSS-ALIGN-3]
Note:
The
scrollable overflow rectangle
is always a rectangle in the box’s own coordinate system, but might be non-rectangular in other coordinate systems due to transforms
[CSS3-TRANSFORMS]
. This means scrollbars can sometimes appear when not actually necessary.
2.3.
Scrolling Overflow
A box’s
overflow
can be visible or clipped. CSS also allows a box to be a
scroll container
that allows clipped parts of its
scrollable overflow area
to be scrolled into view. The visual “viewport” of a
scroll container
(through which the
scrollable overflow area
can be viewed) coincides with its padding box, and is called the
scrollport
. A box’s
nearest scrollport
is the
scrollport
of its nearest
scroll container
ancestor.
Scrolling operations can be initiated by the user (for example, by manipulating a scrollbar, swiping a touchscreen, or using keyboard controls) or by script (
for example, by the
scrollIntoView()
or
focus()
APIs
). The initial position of the
scrollable overflow rectangle
within the
scrollport
before any scrolling operations take effect is the
initial scroll position
. The
initial scroll position
is typically dependent on the
scroll container
’s
writing mode
, and, unless otherwise specified, coincides with its
scroll origin position
. For example,
scroll-initial-target
property can be used to change the
initial scroll position
.
[CSS-SCROLL-SNAP-2]
A
scroll position
is a particular alignment of the
scrollable overflow rectangle
within its
scrollport
. It is associated with a
scroll offset
which is its distance from the
scroll origin
.
The
scroll origin
is the anchor coordinate of the
scrollable overflow rectangle
, from which the
scrollable overflow rectangle
expands. Unless otherwise specified, it is the
block-start
inline-start
corner of the
scrollable overflow rectangle
. (For example, in a
flex container
it is the
main-start
cross-start
corner.) Unless otherwise adjusted (e.g.
by content alignment
[css-align-3]
), the area beyond the
scroll origin
in either axis is considered the
unreachable scrollable overflow region
: content rendered here is not accessible to the reader, see
§ 2.2 Scrollable Overflow
. A
scroll container
is said to be scrolled to its
scroll origin
when its
scroll origin
coincides with the corresponding corner of its
scrollport
. This
scroll position
, the
scroll origin position
, usually, but not always, coincides with the
initial scroll position
.
For example,
scroll snapping
[CSS-SCROLL-SNAP-1]
can change the
initial scroll position
away from the
scroll origin position
.
Check whether things like
baseline alignment
depend on the
initial scroll position
or the
scroll origin position
.
This doesn’t define a coordinate system for
scroll offsets
. Whether they increase downward/rightward, block/inline-axis endward, or away from the
scroll origin
is not defined. Should each API define its coordinate model?
The root viewport, which scrolls the page
canvas
, uses the
principal writing mode
for determining its
scroll origin
and
initial scroll position
.
Note:
In the case where a
scroll container
(or one of its ancestors) is the target of a graphical transform, the UA might need to take this transform into account when mapping user inputs to scrolling operations. For instance, on a touch screen where the user scrolls by directly dragging the content, the transform would be expected to be taken into account to match the direction of scrolling to the gesture. On the other hand, other user inputs (such as the Page Down key, or a 1D scroll wheel) might be more naturally interpreted ignoring the transform. Choosing the appropriate behavior for each scrolling mechanism is the responsibility of the UA.
3.
Scrolling and Clipping Overflow
3.1.
Managing Overflow: the
overflow-x
,
overflow-y
, and
overflow
properties
These properties specify whether a box’s
overflow
is clipped, and if so, whether it is a
scroll container
.
Name:
overflow-x
,
overflow-y
,
overflow-block
,
overflow-inline
Value:
visible
|
hidden
|
clip
|
scroll
|
auto
Initial:
visible
Applies to:
block containers
[CSS2]
, flex containers
[CSS3-FLEXBOX]
, grid containers
[CSS3-GRID-LAYOUT]
Inherited:
no
Percentages:
N/A
Computed value:
usually specified value, but see text
Canonical order:
per grammar
Animation type:
discrete
Logical property group:
overflow
The
overflow-x
property specifies the handling of
overflow
in the horizontal axis (i.e., overflow from the left and right sides of the box), and the
overflow-y
property specifies the handling of
overflow
in the vertical axis (i.e., overflow from the top and bottom sides of the box).
The
overflow-block
and
overflow-inline
properties likewise specify the handling of
overflow
in the
block
and
inline
axis, respectively
These four properties form a
logical property group
together with the
overflow
shorthand
, and interact as defined in
CSS Logical Properties 1
§ 4 Flow-Relative Box Model Properties
.
Name:
overflow
Value:
<'overflow-block'>
{1,2}
Initial:
visible
Applies to:
block containers
[CSS2]
, flex containers
[CSS3-FLEXBOX]
, and grid containers
[CSS3-GRID-LAYOUT]
Inherited:
no
Percentages:
N/A
Computed value:
see individual properties
Animation type:
discrete
Canonical order:
per grammar
The
overflow
property is a
shorthand property
that sets the specified values of
overflow-x
and
overflow-y
in that order. If the second value is omitted, it is copied from the first.
Values have the following meanings:
visible
There is no special handling of overflow, that is, the box’s content is rendered outside the box if positioned there. The box is not a
scroll container
.
hidden
This value indicates that the box’s content is clipped to its
padding box
and that the UA must not provide any scrolling user interface to view the content outside the clipping region, nor allow scrolling by direct intervention of the user, such as dragging on a touch screen or using the scrolling wheel on a mouse. However, the content must still be scrollable programmatically, for example using the mechanisms defined in
[CSSOM-VIEW]
, and the box is therefore still a
scroll container
.
clip
This value indicates that the box’s content is clipped to its
overflow clip edge
and that no scrolling user interface should be provided by the UA to view the content outside the clipping region. In addition, unlike
overflow: hidden
which still allows programmatic scrolling,
overflow: clip
forbids scrolling entirely, through any mechanism, and therefore the box is not a
scroll container
.
Unlike
hidden
, this value
does not
cause the element to establish a new formatting context.
Note:
Authors who also want the box to establish a formatting context may use
display: flow-root
together with
overflow: clip
.
scroll
This value indicates that the content is clipped to the
padding box
, but can be scrolled into view (and therefore the box is a
scroll container
). Furthermore, if the user agent uses a scrolling mechanism that is visible on the screen (such as a scroll bar or a panner), that mechanism should be displayed whether or not any of its content is clipped. This avoids any problem with scrollbars appearing and disappearing in a dynamic environment. When the target medium is
print
, overflowing content may be printed; it is not defined where it may be printed.
auto
Like
scroll
when the box has
scrollable overflow
; like
hidden
otherwise. Thus, if the user agent uses a scrolling mechanism that is visible on the screen (such as a scroll bar or a panner), that mechanism will only be displayed if there is overflow.
The
scroll
,
auto
, and
hidden
values are known as the
scrollable values
of
overflow
. The
visible
and
clip
values are known as the
non-scrollable values
.
The
visible
/
clip
values of
overflow
compute to
auto
/
hidden
(respectively) if one of
overflow-x
or
overflow-y
is neither
visible
nor
clip
. On
replaced elements
, a
computed
hidden
value further resolves to a
used value
of
clip
.
If the computed value of
overflow
on a
block box
is neither
visible
nor
clip
nor a combination thereof, it
establishes an independent formatting context
for its contents.
User agents must also support the
overlay
keyword as a
legacy value alias
of
auto
.
3.1.1.
Interaction of
visibility
and
overflow
If the computed value of the
visibility
property is
hidden
(or
collapse
when it has the same effect as
hidden
), and
overflow
is either
scroll
or
auto
, then:
The user agent must not make any scrolling mechanism visible. To the extent that the scrolling mechanism that would normally be visible in the absence of
visibility: hidden
affects layout, it continues to do so, but is not painted.
As would be the case with
overflow: hidden
, scrolling directly triggered by user interactions is disabled, but programmatic scrolling continues to take effect.
The lack of interactive direct scrolling is enforced even if the user interacts (e.g. with a mouse scrolling wheel) with a descendent of the
visibility: hidden
scroll container
that is itself set to
visibility: visible
.
3.1.2.
Interaction of
border-radius
and
overflow
As mentioned in
CSS Backgrounds 3
§ 4.3 Corner Clipping
, the clipping region established by
overflow
can be rounded:
When
overflow-x
and
overflow-y
compute to
hidden
,
scroll
, or
auto
, the clipping region is rounded based on the border radius, adjusted to the
padding edge
, as described in
CSS Backgrounds 3
§ 4.2 Corner Shaping
.
When both
overflow-x
and
overflow-y
compute to
clip
, the clipping region is rounded as described in
§ 3.2 Expanding Clipping Bounds: the overflow-clip-margin property
.
However, when one of
overflow-x
or
overflow-y
computes to
clip
and the other computes to
visible
, the clipping region is not rounded.
3.1.3.
Overflow in Print and Other Static Media
Since scrolling is not possible in static media (such as print) authors should be careful to make content accessible in such media, for example by using
@media print, (update: none) { … }
to adjust layout such that all relevant content is simultaneously visible.
On
scroll containers
in non-interactive media with an
overflow
value of
auto
or
scroll
(but not
hidden
) UAs may display an indication of any scrollable overflow, such as by displaying scrollbars or an ellipsis.
Note:
Not all
paged media
is non-interactive: for example, e-book readers paginate content, but are interactive.
3.2.
Expanding Clipping Bounds: the
overflow-clip-margin
property
Name:
overflow-clip-margin
Value:
<visual-box>
||
<length [0,∞]>
Initial:
0px
Applies to:
boxes to which
overflow
applies
Inherited:
no
Percentages:
n/a
Computed value:
the computed
<length>
and a
<visual-box>
keyword
Canonical order:
per grammar
Animation type:
per computed value if the
<visual-box>
values match; otherwise discrete
This property defines the
overflow clip edge
of the box, i.e. precisely
how far
outside its bounds the box’s content is allowed to paint before being clipped by effects (such as
overflow: clip
, above) that are defined to clip to the box’s
overflow clip edge
.
Values are defined as follows:
<visual-box>
Specifies the box edge to use as the
overflow clip edge
origin, i.e. when the specified offset is zero.
If omitted, defaults to
padding-box
.
<length [0,∞]>
The specified offset dictates how much the
overflow clip edge
is expanded from the specified box edge Negative values are invalid. Defaults to zero if omitted.
The
overflow clip edge
is shaped in the corners exactly the same way as an
outer box-shadow
with a spread radius of the same cumulative offset from the box’s
border edge
. See
CSS Backgrounds 3
§ 4.2 Corner Shaping
and
CSS Backgrounds 3
§ 6.1.1 Shadow Shape, Spread, and Knockout
, noting in particular the formula for outsets beyond the
border edge
.
Note:
This property has no effect on boxes with
overflow: hidden
or
overflow: scroll
, which are not defined to use the
overflow clip edge
.
3.3.
Overflow Viewport Propagation
UAs must apply the
overflow-*
values set on the root element to the
viewport
when the root element’s
display
value is not
none
. However, when the root element is an
[HTML]
html
element (including
XML syntax for HTML
) whose
overflow
value is
visible
(in both axes), and that element has as a child a
body
element whose
display
value is also not
none
, user agents must instead apply the
overflow-*
values of the first such child element to the viewport. The element from which the value is propagated must then have a used
overflow
value of
visible
.
Note:
Using
containment
on the HTML
html
or
body
elements disables this special handling of the HTML
body
element. See the
CSS Containment 1
§ 2 Strong Containment: the contain property
for details.
Note:
overflow: hidden
on the root element might not clip everything outside the
Initial Containing Block
if the ICB is smaller than the viewport, which can happen on mobile.
If
visible
is applied to the viewport, it must be interpreted as
auto
. If
clip
is applied to the viewport, it must be interpreted as
hidden
.
3.4.
Smooth Scrolling: the
scroll-behavior
Property
Name:
scroll-behavior
Value:
auto
|
smooth
Initial:
auto
Applies to:
scroll containers
Inherited:
no
Percentages:
n/a
Computed value:
specified value
Canonical order:
per grammar
Animation type:
not animatable
The
scroll-behavior
property specifies the scrolling behavior for a
scroll container
, when scrolling happens due to navigation, scrolling APIs
[CSSOM-VIEW]
, or scroll snapping operations not initiated by the user
[CSS-SCROLL-SNAP-1]
. Any other scrolls, e.g. those that are performed by the user, are not affected by this property. When this property is specified on the root element, it applies to the
viewport
instead.
Note:
The
scroll-behavior
property of the HTML
body
element is
not
propagated to the viewport.
auto
The
scroll container
is scrolled in an
instant
fashion.
smooth
The
scroll container
is scrolled in a
smooth
fashion using a user-agent-defined timing function over a user-agent-defined period of time. User agents should follow platform conventions, if any.
User agents may ignore this property.
4.
Scrollbars and Layout
4.1.
Scrollbar Contributions to Sizing
When reserving space for a scrollbar placed at the edge of an element’s box, the reserved space is inserted between the inner border edge and the outer padding edge. For the purpose of the
background positioning area
and
background painting area
however, this reserved space is considered to be part of the
padding box
.
In the following document fragment, both an absolutely-positioned element and a background image are positioned to the top right of the box.
<
style
>
article
{
background
:
top
right
no-repeat
url
(
circle.png
);
position
:
relative
;
overflow
:
auto
;
}
aside
{
position
:
absolute
;
top
:
0
;
right
:
0
;
}
</
style
>
<
article
>
<
aside
>
×
</
aside
>
</
article
>
If no scrollbars are present on
<article>
, they will both coincide in the top right padding edge corner. However, if scrollbars are present then
<aside>
will be completely visible, on the right padding-box edge adjacent to the scrollbars; whereas the background image will be tucked underneath the scrollbars, in the top right corner of the scrollbar-extended
background positioning area
.
When the box is intrinsically sized, this reserved space is added to the size of its contents. It is otherwise subtracted from space allotted to the
content area
. To the extent that the presence of scrollbars can affect sizing, UAs must start with the assumption that no scrollbars are needed, and recalculate sizes if it turns out they are.
In the following document fragment, the outer
<article>
has
height: auto
, but
max-height: 5em
. The inner
<section>
has large margins and would normally just fit:
... article { overflow: auto; max-height: 5em; width: max-content; } section { margin: 2em; line-height: 1 } ...
<
article
>
<
section
>
This section has big margins.
</
section
>
</
article
>
If we assumed that
<article>
needed scrollbars, then the height of
<section>
, including the single line of text and twice 2em of margins, adds up to 5em plus a scrollbar. Since that is greater than 5em, the maximum allowed height, it seems we made the right assumption and d1 indeed needs scrollbars.
However, we should have started by assuming that no scrollbars are needed. In that case the content height of
<article>
is exactly the maximum height of 5em, proving that the assumption was correct and
<article>
indeed should not have scrollbars.
4.2.
Reserving space for the scrollbar: the
scrollbar-gutter
property
The space between the inner border edge and the outer padding edge which user agents may reserve to display the scrollbar is called the
scrollbar gutter
.
The
scrollbar-gutter
property gives control to the author over the presence of
scrollbar gutters
separately from the ability to control the presence of scrollbars provided by the
overflow
property.
Name:
scrollbar-gutter
Value:
auto
|
stable
&&
both-edges
?
Initial:
auto
Applies to:
scroll containers
Inherited:
no
Percentages:
n/a
Computed value:
specified keyword(s)
Canonical order:
per grammar
Animation type:
discrete
This property affects the presence of
scrollbar gutters
placed at the
inline start
edge or
inline end
edge of the box.
The presence of a
scrollbar gutter
at the
block start
edge and
block end
edge of the box cannot be controlled in this level, and is determined the same way as the presence of
scrollbar gutters
placed at the
inline start
edge or
inline end
edge of the box when
scrollbar-gutter
is
auto
.
Scrollbars which by default are placed over the content box and do not cause
scrollbar gutters
to be created are called
overlay scrollbars
. Such scrollbars are usually partially transparent, revealing the content behind them if any. Their appearance and size may vary based on whether and how the user is interacting with them.
Scrollbars which are always placed in a
scrollbar gutter
, consuming space when present, are called
classic scrollbars
. Such scrollbars are usually opaque.
Whether
classic scrollbars
or
overlay scrollbars
are used, the appearance and size of the scrollbar, and whether scrollbars appear on the start or end edge of the box, is UA defined.
Note:
Which side a scrollbar appears on may depend on operating system conventions,
bidirectionality
, or other ergonomic considerations.
In the case of
classic scrollbars
, the width of the
scrollbar gutter
, if present (see below), is the same as the width of the scrollbar. In the case of
overlay scrollbars
, no
scrollbar gutter
is present.
Note:
There are known use cases that could be addressed by enabling
scrollbar gutters
for
overlay scrollbars
, but no satisfactory design has been agreed to so far. This could be addressed by future extensions of this property. See
CSS Overflow 4
§ B Possible extensions for scrollbar-gutter
.
The values of this property have the following meaning:
auto
Classic scrollbars
consume space by creating a
scrollbar gutter
when
overflow
is
scroll
, or when
overflow
is
auto
and the box is overflowing.
Overlay scrollbars
do not consume space.
stable
The
scrollbar gutter
is present for
classic scrollbars
when
overflow
is
hidden
,
scroll
, or
auto
, regardless of whether the box is actually overflowing.
Overlay scrollbars
do not consume space.
Note:
This does not change whether the scrollbar itself is visible, only the presence of a gutter is affected.
both-edges
If a
scrollbar gutter
would be present on one of the inline start edge or the inline end edge of the box, another
scrollbar gutter
must be present on the opposite edge as well.
When the
scrollbar gutter
is present but the scrollbar is not, or the scrollbar is transparent or otherwise does not fully obscure the
scrollbar gutter
, the background of the
scrollbar gutter
must be painted as an extension of the padding.
As for the
overflow
property, when
scrollbar-gutter
is set on the root element, the user agent must apply it to the viewport instead, and the used value on the root element itself is
scrollbar-gutter: auto
. However, unlike the
overflow
property, the user agent must not propagate
scrollbar-gutter
from the HTML
body
element.
Note: The following table summarizes the interaction of
overflow
and
scrollbar-gutter
showing in which case space is reserved for a
classic scrollbar
’s
scrollbar gutter
.
Should a
classic scrollbar
’s
scrollbar gutter
be present?
overflow
scrollbar-gutter
Overflowing
Not overflowing
scroll
auto
yes
yes
stable
yes
yes
auto
auto
yes
stable
yes
yes
hidden
auto
stable
yes
yes
visible
,
clip
auto
stable
5.
Automatic Ellipses
5.1.
Overflow Ellipsis: the
text-overflow
property
Name:
text-overflow
Value:
clip
|
ellipsis
Initial:
clip
Applies to:
block containers
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
This property specifies rendering when inline content overflows its
end
line box edge in the inline progression direction of its block container element ("the block") that has
overflow
other than
visible
.
Text can overflow for example when it is prevented from wrapping (e.g. due to
white-space
:
nowrap
or a single word is too long to fit). Values have the following meanings:
clip
Clip inline content that overflows its block container element. Characters may be only partially rendered.
ellipsis
Render an ellipsis character (U+2026) to represent clipped inline content. Implementations may substitute a more language, script, or writing-mode appropriate ellipsis character, or three dots "..." if the ellipsis character is unavailable.
The term "character" is used in this property definition for better readability and means "grapheme cluster"
[UAX29]
for implementation purposes.
For the ellipsis value implementations must hide characters and
atomic inline-level elements
at the
end
edge of the line as necessary to fit the ellipsis, and place the ellipsis immediately adjacent to the
end
edge of the remaining inline content. The first character or
atomic inline-level element
on a line must be clipped rather than ellipsed.
Bidi ellipsis examples
These examples demonstrate which characters get hidden to make room for the ellipsis in a bidi situation: those visually at the end edge of the line.
Sample CSS:
div
{
font-family
:
monospace
;
white-space
:
pre
;
overflow
:
hidden
;
width
:
9
ch
;
text-overflow
:
ellipsis
;
}
Sample HTML fragments, renderings, and your browser:
HTML
Reference rendering
Your Browser
<
div
>
שלום 123456
</
div
>
123456 ם…
שלום 123456
<
div
dir
=
rtl
>
שלום 123456
</
div
>
…456 שלום
שלום 123456
ellipsing details
Ellipsing only affects rendering and must not affect layout nor dispatching of pointer events: The UA should dispatch any pointer event on the ellipsis to the elided element, as if
text-overflow
had been
none
.
The ellipsis is styled and baseline-aligned according to the block.
Ellipsing occurs after relative positioning and other graphical transformations.
If there is insufficient space for the ellipsis, then clip the rendering of the ellipsis itself (on the same side that neutral characters on the line would have otherwise been clipped with the
text-overflow:clip
value).
user interaction with ellipsis
When the user is interacting with content (e.g. editing, selecting, scrolling), the user agent may treat
text-overflow: ellipsis
as
text-overflow: clip
.
Selecting the ellipsis should select the ellipsed text. If all of the ellipsed text is selected, UAs should show selection of the ellipsis. Behavior of partially-selected ellipsed text is up to the UA.
Example(s):
text-overflow examples
These examples demonstrate setting the text-overflow of a block container element that has text which overflows its dimensions:
sample CSS for a div:
div
{
font-family
:
Helvetica
,
sans-serif
;
line-height
:
1.1
;
width
:
3.1
em
;
padding
:
.2
em
;
border
:
solid
.1
em
black
;
margin
:
1
em
0
;
}
sample HTML fragments, renderings, and your browser:
HTML
sample rendering
your browser
<
div
>
CSS IS AWESOME, YES
</
div
>
First, a box with text drawing outside of it.
CSS IS AWESOME, YES
<
div
style
=
"
text-overflow:clip;
overflow:hidden"
>
CSS IS AWESOME, YES
</
div
>
Second, a similar box with the text clipped outside the box.
CSS IS AWESOME, YES
<
div
style
=
"
text-overflow:ellipsis;
overflow:hidden"
>
CSS IS AWESOME, YES
</
div
>
Third, a similar box with an ellipsis representing the clipped text.
CSS IS AWESOME, YES
<
div
style
=
"
text-overflow:ellipsis;
overflow:hidden"
>
NESTED
<
p
>
PARAGRAPH
</
p
>
WON’T ELLIPSE.
</
div
>
Fourth, a box with a nested paragraph demonstrating anonymous block boxes equivalency and non-inheritance into a nested element.
NESTED
PARAGRAPH
WON’T ELLIPSE.
Note:
the side of the line that the ellipsis is placed depends on the
direction
of the block. E.g. an overflow hidden right-to-left (
direction
:
rtl
) block clips inline content on the
left
side, thus would place a text-overflow ellipsis on the
left
to represent that clipped content.
ellipsis interaction with scrolling interfaces
This section applies to elements with text-overflow other than
text-overflow:clip
(non-clip text-overflow) and overflow:scroll.
When an element with non-clip text-overflow has overflow of scroll in the inline progression dimension of the text, and the browser provides a mechanism for scrolling (e.g. a scrollbar on the element, or a touch interface to swipe-scroll, etc.), there are additional implementation details that provide a better user experience:
When an element is scrolled (e.g. by the user, DOM manipulation), more of the element’s content is shown. The value of text-overflow should not affect whether more of the element’s content is shown or not. If a non-clip text-overflow is set, then as more content is scrolled into view, implementations should show whatever additional content fits, only truncating content which would otherwise be clipped (or is necessary to make room for the ellipsis/string), until the element is scrolled far enough to display the edge of the content at which point that content should be displayed rather than an ellipsis/string.
This example uses text-overflow on an element with overflow scroll to demonstrate the above described behavior.
sample CSS:
div.crawlbar
{
text-overflow
:
ellipsis
;
height
:
2
em
;
overflow
:
scroll
;
white-space
:
nowrap
;
width
:
15
em
;
border
:
1
em
solid black
;
}
sample HTML fragment:
<
div
class
=
"crawlbar"
>
CSS is awesome, especially when you can scroll to see extra text instead of just having it overlap other text by default.
</
div
>
demonstration of sample CSS and HTML:
CSS is awesome, especially when you can scroll to see extra text instead of just having it overlap other text by default.
While the content is being scrolled, implementations may adjust their rendering of ellipses (e.g. align to the box edge rather than line edge).
Appendix A. Privacy Considerations
This specification introduces no new privacy concerns.
Appendix B. Security Considerations
This specification introduces no new security concerns.
Appendix C. Changes
This appendix is
informative
.
Significant changes since the
29 March 2023 Working Draft
:
Define that
overflow: hidden
is treated as
overflow: clip
on
replaced elements
. (
Issue 7714
)
Redefine the interaction of overflowing
content distribution
and
scroll containers
to not impact layout, but to only affect the extent of the
scrollable overflow area
. (
Issue 4957
)
Defined some more terminology to help other specs connect to concepts in this one.
See also
Previous Changes
.
Acknowledgments
Thanks especially to the feedback from Rossen Atanassov, Tab Atkins, Bert Bos, Tantek Çelik, John Daggett, Daniel Glazman, Vincent Hardy, Ian Kilpatrick, Håkon Wium Lie, Peter Linss, Robert O’Callahan, Florian Rivoal, Alan Stearns, Steve Zilles, and all the rest of the
www-style
community.
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
auto
value for overflow, overflow-x, overflow-y
, in § 3.1
value for scroll-behavior
, in § 3.4
value for scrollbar-gutter
, in § 4.2
both-edges
, in § 4.2
classic scrollbars
, in § 4.2
clip
value for overflow, overflow-x, overflow-y
, in § 3.1
value for text-overflow
, in § 5.1
ellipsis
, in § 5.1
hidden
, in § 3.1
initial scroll position
, in § 2.3
ink overflow
, in § 2.1
ink overflow area
, in § 2.1
ink overflow rectangle
, in § 2.1
ink overflow region
, in § 2.1
<length [0,∞]>
, in § 3.2
nearest scrollport
, in § 2.3
non-scrollable overflow value
, in § 3.1
overflow
(property)
, in § 3.1
definition of
, in § 2
overflow-block
, in § 3.1
overflow clip edge
, in § 3.2
overflow-clip-margin
, in § 3.2
overflow-inline
, in § 3.1
overflow-x
, in § 3.1
overflow-y
, in § 3.1
overlay
, in § 3.1
overlay scrollbars
, in § 4.2
scroll
, in § 3.1
scrollable overflow
, in § 2.2
scrollable overflow area
, in § 2.2
scrollable overflow rectangle
, in § 2.2
scrollable overflow region
, in § 2.2
scrollable overflow value
, in § 3.1
scrollbar gutter
, in § 4.2
scrollbar-gutter
, in § 4.2
scroll-behavior
, in § 3.4
scroll container
, in § 2.3
scroll offset
, in § 2.3
scroll origin
, in § 2.3
scroll origin position
, in § 2.3
scrollport
, in § 2.3
scroll position
, in § 2.3
smooth
, in § 3.4
stable
, in § 4.2
text-overflow
, in § 5.1
unreachable scrollable overflow region
, in § 2.3
visible
, in § 3.1
<visual-box>
, in § 3.2
Terms defined by reference
[CSS-ALIGN-3]
defines the following terms:
alignment subject
baseline alignment
content distribution
content-distribution properties
place-content
[CSS-BACKGROUNDS-3]
defines the following terms:
background painting area
background positioning area
border-radius
box-shadow
outer box-shadow
[CSS-BOX-4]
defines the following terms:
<visual-box>
border edge
content area
padding box
padding edge
[CSS-CASCADE-5]
defines the following terms:
computed value
legacy value alias
shorthand
shorthand property
used value
[CSS-CONTAIN-2]
defines the following terms:
contain
containment
[CSS-DISPLAY-3]
defines the following terms:
display
[CSS-DISPLAY-4]
defines the following terms:
block box
collapse
containing block chain
establishes an independent formatting context
hidden
initial containing block
none
replaced
replaced element
visibility
[CSS-GRID-2]
defines the following terms:
grid item
[CSS-INLINE-3]
defines the following terms:
line box
[CSS-LOGICAL-1]
defines the following terms:
logical property group
[CSS-MASKING-1]
defines the following terms:
clip
[CSS-OVERFLOW-4]
defines the following terms:
block-ellipsis
line-clamp
max-lines
[CSS-SCROLL-SNAP-1]
defines the following terms:
scroll snap
[CSS-SCROLL-SNAP-2]
defines the following terms:
scroll-initial-target
[CSS-SIZING-3]
defines the following terms:
height
max-height
[CSS-TEXT-DECOR-4]
defines the following terms:
text-shadow
[CSS-TRANSFORMS-2]
defines the following terms:
3D rendering context
transform-style
[CSS-VALUES-4]
defines the following terms:
&&
<length>
?
CSS-wide keywords
{A,B}
|
||
[CSS-WRITING-MODES-3]
defines the following terms:
direction
left
[CSS-WRITING-MODES-4]
defines the following terms:
bidirectionality
block axis
block end
block start
block-start
end
inline axis
inline end
inline start
inline-start
principal writing mode
writing mode
[CSS2]
defines the following terms:
canvas
viewport
[CSS3-FLEXBOX]
defines the following terms:
cross-start
flex container
flex item
main-start
[CSSOM-VIEW]
defines the following terms:
instant scroll
scrollIntoView()
smooth scroll
[HTML]
defines the following terms:
body
focus()
html
[MEDIAQUERIES-5]
defines the following terms:
paged media
References
Normative References
[CSS-ALIGN-3]
Elika Etemad; Tab Atkins Jr..
CSS Box Alignment Module Level 3
. 11 March 2025. WD. URL:
https://www.w3.org/TR/css-align-3/
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
[CSS-CASCADE-5]
Elika Etemad; Miriam Suzanne; Tab Atkins Jr..
CSS Cascading and Inheritance Level 5
. 13 January 2022. CR. URL:
https://www.w3.org/TR/css-cascade-5/
[CSS-CONTAIN-2]
Tab Atkins Jr.; Florian Rivoal; Vladimir Levin.
CSS Containment Module Level 2
. 17 September 2022. WD. URL:
https://www.w3.org/TR/css-contain-2/
[CSS-DISPLAY-3]
Elika Etemad; Tab Atkins Jr..
CSS Display Module Level 3
. 30 March 2023. CR. URL:
https://www.w3.org/TR/css-display-3/
[CSS-DISPLAY-4]
Elika Etemad; Tab Atkins Jr..
CSS Display Module Level 4
. 19 December 2024. FPWD. URL:
https://www.w3.org/TR/css-display-4/
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
[CSS-MASKING-1]
Dirk Schulze; Brian Birtles; Tab Atkins Jr..
CSS Masking Module Level 1
. 5 August 2021. CRD. URL:
https://www.w3.org/TR/css-masking-1/
[CSS-OVERFLOW-4]
David Baron; Florian Rivoal; Elika Etemad.
CSS Overflow Module Level 4
. 21 March 2023. WD. URL:
https://www.w3.org/TR/css-overflow-4/
[CSS-SCROLL-SNAP-1]
Matt Rakow; et al.
CSS Scroll Snap Module Level 1
. 11 March 2021. CR. URL:
https://www.w3.org/TR/css-scroll-snap-1/
[CSS-SCROLL-SNAP-2]
Elika Etemad; Tab Atkins Jr.; Adam Argyle.
CSS Scroll Snap Module Level 2
. 23 July 2024. FPWD. URL:
https://www.w3.org/TR/css-scroll-snap-2/
[CSS-TEXT-DECOR-4]
Elika Etemad; Koji Ishii.
CSS Text Decoration Module Level 4
. 4 May 2022. WD. URL:
https://www.w3.org/TR/css-text-decor-4/
[CSS-TRANSFORMS-2]
Tab Atkins Jr.; et al.
CSS Transforms Module Level 2
. 9 November 2021. WD. URL:
https://www.w3.org/TR/css-transforms-2/
[CSS-UI-3]
Tantek Çelik; Florian Rivoal.
CSS Basic User Interface Module Level 3 (CSS3 UI)
. 21 June 2018. REC. URL:
https://www.w3.org/TR/css-ui-3/
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
[CSS3-FLEXBOX]
Tab Atkins Jr.; et al.
CSS Flexible Box Layout Module Level 1
. 19 November 2018. CR. URL:
https://www.w3.org/TR/css-flexbox-1/
[CSS3-GRID-LAYOUT]
Tab Atkins Jr.; et al.
CSS Grid Layout Module Level 1
. 26 March 2025. CRD. URL:
https://www.w3.org/TR/css-grid-1/
[CSS3-TRANSFORMS]
Simon Fraser; et al.
CSS Transforms Module Level 1
. 14 February 2019. CR. URL:
https://www.w3.org/TR/css-transforms-1/
[CSSOM-VIEW]
Simon Fraser; Emilio Cobos Álvarez.
CSSOM View Module
. 16 September 2025. WD. URL:
https://www.w3.org/TR/cssom-view-1/
[HTML]
Anne van Kesteren; et al.
HTML Standard
. Living Standard. URL:
https://html.spec.whatwg.org/multipage/
[RFC2119]
S. Bradner.
Key words for use in RFCs to Indicate Requirement Levels
. March 1997. Best Current Practice. URL:
https://datatracker.ietf.org/doc/html/rfc2119
[UAX29]
Josh Hadley.
Unicode Text Segmentation
. 17 August 2025. Unicode Standard Annex #29. URL:
https://www.unicode.org/reports/tr29/tr29-47.html
Informative References
[CSS-CONTAIN-1]
Tab Atkins Jr.; Florian Rivoal.
CSS Containment Module Level 1
. 25 June 2024. REC. URL:
https://www.w3.org/TR/css-contain-1/
[CSS-SIZING-3]
Tab Atkins Jr.; Elika Etemad.
CSS Box Sizing Module Level 3
. 17 December 2021. WD. URL:
https://www.w3.org/TR/css-sizing-3/
[CSS-TEXT-3]
Elika Etemad; Koji Ishii; Florian Rivoal.
CSS Text Module Level 3
. 30 September 2024. CRD. URL:
https://www.w3.org/TR/css-text-3/
[CSS-WRITING-MODES-3]
Elika Etemad; Koji Ishii.
CSS Writing Modes Level 3
. 10 December 2019. REC. URL:
https://www.w3.org/TR/css-writing-modes-3/
[CSS1]
Håkon Wium Lie; Bert Bos.
Cascading Style Sheets, level 1
. 13 September 2018. REC. URL:
https://www.w3.org/TR/CSS1/
[MEDIAQUERIES-5]
Dean Jackson; et al.
Media Queries Level 5
. 18 December 2021. WD. URL:
https://www.w3.org/TR/mediaqueries-5/
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
overflow
<'overflow-block'>{1,2}
visible
block containers [CSS2], flex containers [CSS3-FLEXBOX], and grid containers [CSS3-GRID-LAYOUT]
no
N/A
discrete
per grammar
see individual properties
overflow-block
visible | hidden | clip | scroll | auto
visible
block containers [CSS2], flex containers [CSS3-FLEXBOX], grid containers [CSS3-GRID-LAYOUT]
no
N/A
discrete
per grammar
usually specified value, but see text
overflow
overflow-clip-margin
<visual-box> || <length [0,∞]>
0px
boxes to which overflow applies
no
n/a
per computed value if the <visual-box> values match; otherwise discrete
per grammar
the computed <length> and a <visual-box> keyword
overflow-inline
visible | hidden | clip | scroll | auto
visible
block containers [CSS2], flex containers [CSS3-FLEXBOX], grid containers [CSS3-GRID-LAYOUT]
no
N/A
discrete
per grammar
usually specified value, but see text
overflow
overflow-x
visible | hidden | clip | scroll | auto
visible
block containers [CSS2], flex containers [CSS3-FLEXBOX], grid containers [CSS3-GRID-LAYOUT]
no
N/A
discrete
per grammar
usually specified value, but see text
overflow
overflow-y
visible | hidden | clip | scroll | auto
visible
block containers [CSS2], flex containers [CSS3-FLEXBOX], grid containers [CSS3-GRID-LAYOUT]
no
N/A
discrete
per grammar
usually specified value, but see text
overflow
scroll-behavior
auto | smooth
auto
scroll containers
no
n/a
not animatable
per grammar
specified value
scrollbar-gutter
auto | stable && both-edges?
auto
scroll containers
no
n/a
discrete
per grammar
specified keyword(s)
text-overflow
clip | ellipsis
clip
block containers
no
N/A
discrete
per grammar
specified keyword
Issues Index
Is this description of handling transforms sufficiently accurate?
↵
This needs further testing and investigation; is therefore deferred in this draft.
↵
Need to evaluate what should/should not clip
scrollable overflow
.
[Issue #8607]
↵
Check whether things like
baseline alignment
depend on the
initial scroll position
or the
scroll origin position
.
↵
This doesn’t define a coordinate system for
scroll offsets
. Whether they increase downward/rightward, block/inline-axis endward, or away from the
scroll origin
is not defined. Should each API define its coordinate model?
↵
```
