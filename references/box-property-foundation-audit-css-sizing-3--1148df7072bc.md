Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Box Sizing Module Level 3](https://www.w3.org/TR/2026/WD-css-sizing-3-20260904/).

Original copyright notice (from the matching exact HTML edition): Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Box Sizing Module Level 3

Source snapshot: https://www.w3.org/TR/2026/WD-css-sizing-3-20260904/

Snapshot SHA-256: 1148df7072bca9e2788c3ce0ca0e5187b21029f432ef3e378b6a33d0f6c17e82

Conversion: offline format conversion of the exact stored extracted-text; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- This stored input is an already extracted plain-text witness. It has no recoverable original HTML structure. The complete source text is preserved in a fenced block; no summary or specification regeneration was performed.

---

```text
CSS Box Sizing Module Level 3
CSS Box Sizing Module Level 3
W3C Working Draft
,
4 September 2026
More details about this document
This version:
https://www.w3.org/TR/2026/WD-css-sizing-3-20260904/
Latest published version:
https://www.w3.org/TR/css-sizing-3/
Editor's Draft:
https://drafts.csswg.org/css-sizing-3/
Previous Versions:
https://www.w3.org/TR/2021/WD-css-sizing-3-20210317/
https://www.w3.org/TR/2020/WD-css-sizing-3-20201218/
https://www.w3.org/TR/2020/WD-css-sizing-3-20201023/
https://www.w3.org/TR/2019/WD-css-sizing-3-20190522/
https://www.w3.org/TR/2018/WD-css-sizing-3-20180304/
https://www.w3.org/TR/2017/WD-css-sizing-3-20170207/
History:
https://www.w3.org/standards/history/css-sizing-3/
Feedback:
CSSWG Issues Repository
CSSWG GitHub
Inline In Spec
Editors:
Tab Atkins
(
Google
)
Elika J. Etemad / fantasai
(
Apple
)
Suggest an Edit for this Spec:
GitHub Editor
Test Suite:
https://wpt.fyi/results/css/css-sizing/
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
This module extends the CSS sizing properties with keywords that represent content-based "intrinsic" sizes and context-based "extrinsic" sizes, allowing CSS to more easily describe boxes that fit their content or fit into a particular layout context.
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
(preferred), including the spec code “css-sizing” in the title, like this: “[css-sizing]
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
The following features are at-risk, and may be dropped during the CR period:
Additions to
column-width
“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.
Table of Contents
1
Introduction
1.1
Module interactions
1.2
Value Definitions
2
Terminology
2.1
Auto Box Sizes
2.2
Intrinsic Size Contributions
2.3
Intrinsic Size Constraints
3
Specifying Box Sizes
3.1
Sizing Properties
3.1.1
Preferred Size Properties: the
width
,
height
,
inline-size
, and
block-size
properties
3.1.2
Minimum Size Properties: the
min-width
,
min-height
,
min-inline-size
, and
min-block-size
properties
3.1.3
Maximum Size Properties: the
max-width
,
max-height
,
max-inline-size
, and
max-block-size
properties
3.2
Sizing Values: the
<length-percentage>
,
auto
|
none
,
stretch
,
min-content
,
max-content
, and
fit-content
values
3.2.1
“Behaving as
auto
”
3.2.2
Containing or Excluding Floats
3.3
Box Edges for Sizing: the
box-sizing
property
3.4
New Column Sizing Values: the
stretch
,
min-content
,
max-content
, and
fit-content
values
4
Extrinsic Size Determination
4.1
Percentage Sizing
4.2
Stretch-fit Sizing: filling the containing block
5
Intrinsic Size Determination
5.1
Intrinsic Sizes
5.2
Intrinsic Contributions
5.2.1
Intrinsic Contributions of Percentage-Sized Boxes
5.2.2
Compressible Replaced Elements
Changes
Recent Changes
Additions since CSS Level 2
Acknowledgments
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
Non-Normative References
Property Index
Issues Index
1.
Introduction
This section is not normative.
CSS layout has several different concepts of automatic sizing that are used in various layout calculations. This section defines some more precise terminology to help connect the layout behaviors of this spec to the calculations used in other modules, and some new keywords for the
width
and
height
properties to allow authors to assign elements the dimensions resulting from these size calculations.
Tests
General sizing tests
dynamic-available-size-iframe.html
(live test)
(source)
dynamic-change-inline-size-001.html
(live test)
(source)
dynamic-change-inline-size-002.html
(live test)
(source)
dynamic-change-inline-size-003.html
(live test)
(source)
dynamic-change-inline-size-004.html
(live test)
(source)
frameset-intrinsic-crash.html
(live test)
(source)
inheritance-001.html
(live test)
(source)
inheritance-002.html
(live test)
(source)
min-width-max-width-precedence.html
(live test)
(source)
min-width-max-width-precedence.html
(live test)
(source)
replaced-max-size-saturation.html
(live test)
(source)
responsive-iframe-no-match-element.html
(live test)
(source)
textarea-large-padding-crash.html
(live test)
(source)
This spec needs illustrations! See
issue
.
1.1.
Module interactions
This module extends the
width
,
height
,
min-width
,
min-height
,
max-width
,
max-height
, and
column-width
features defined in
[CSS2]
chapter 10 and in
[CSS3COL]
The definition of the
box-sizing
property in this module supersedes the one in
[CSS-UI-3]
.
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
Terminology
Some key terminology related to coordinate axises and dimensions is defined in
CSS Writing Modes 3
§ 6 Abstract Box Terminology
.
size
A one- or two-dimensional measurement: a
block size
and/or
inline size
; alternatively a
width
and/or
height
.
Whether the
width
or
height
corresponds to an
inline size
or
block size
depends on the
writing mode
.
inner size
The
content-box
size
of a
box
.
Inner size
outer size
The
margin-box
size
of a
box
.
Outer size
definite size
A size that can be determined without performing layout; that is, a
<length>
, a measure of text (without consideration of line-wrapping), a size of the
initial containing block
, or a
<percentage>
or other formula (such the
“stretch-fit” sizing of non-replaced blocks
[CSS2]
) that is resolved solely against
definite
sizes.
Additionally, the size of the
containing block
of an absolutely positioned element is always
definite
with respect to that element.
indefinite size
A size that is not
definite
.
Indefinite
available space
is essentially infinite.
Note:
intrinsic sizing
keywords such as
max-content
are indefinite, even if they can be determined without laying out the children e.g. due to
size containment
or lack of children.
available space
A size representing the space into which a box is laid out, as determined by the rules of the formatting context in which it participates. The space available to a box is usually either a measurement of its
containing block
(if that is
definite
) or an infinite size (when it is
indefinite
).
Available space
can alternatively be either a
min-content constraint
or a
max-content constraint
, which forces boxes laid into it to be laid out under that constraint.
Tests
available-height-for-replaced-content-001.html
(live test)
(source)
table-percentage-max-width-beside-float.html
(live test)
(source)
table-percentage-min-width-below-float.html
(live test)
(source)
table-percentage-min-width-beside-float.html
(live test)
(source)
stretch fit
The
stretch fit
into a given size is that size, minus the box’s computed margins (not collapsed, treating
auto
as zero), border, and padding in the given dimension (such that the outer size is a perfect fit), and flooring at zero (so that the inner size is not negative).
Note: This is the formula used to calculate the
auto
widths of non-replaced blocks in normal flow in
CSS2.1§10.3.3
.
fallback size
Some sizing algorithms do not work well with an infinite size. In these cases, the
fallback size
is used instead. Unless otherwise specified, this is the size of the
initial containing block
.
2.1.
Auto Box Sizes
There are four types of automatically-determined sizes in CSS (sizes resulting from
auto
sizing rules, depending on context):
stretch-fit size
stretch-fit inline size
stretch-fit block size
The
size
a box would take if its
outer size
filled the
available space
in the given axis; in other words, the
stretch fit
into the
available space
, if that is
definite
. Undefined if the
available space
is
indefinite
.
Note:
For the
inline axis
, this is called the “available width” in
CSS2.1§10.3.5
and computed by the rules in
CSS2.1§10.3.3
.
Note:
Calculations involving this size need to specify a fallback behavior for when the
available space
is
indefinite
if that happens to be possible.
max-content size
A box’s “ideal”
size
in a given axis when given infinite available space. Usually this is the smallest
size
the box could take in that axis while still fitting around its contents, i.e. minimizing unfilled space while avoiding overflow.
max-content inline size
The box’s “ideal”
size
in the
inline axis
. Usually the narrowest
inline size
it could take while fitting around its contents if
none
of the soft wrap opportunities within the box were taken. (See
§ 5 Intrinsic Size Determination
.)
Note:
This is called the “preferred width” in
CSS2.1§10.3.5
and the “maximum cell width” in
CSS2.1§17.5.2.2
.
max-content block size
The box’s “ideal”
size
in the
block axis
. Usually the
block size
of the content after layout.
If the ideal
max-content size
would be smaller than the
min-content size
(e.g. due to the use of negative margins) the effective
max-content size
is floored by the
min-content size
.
min-content size
Nominally, the smallest
size
a box could take that doesn’t lead to overflow that could be avoided by choosing a larger
size
. Formally, the size of the box when sized under a
min-content constraint
, see
§ 5 Intrinsic Size Determination
.
min-content inline size
The
min-content size
in the
inline axis
. Typically, the
inline size
that would fit around its contents if
all
soft wrap opportunities within the box were taken.
Note:
This is called the “preferred minimum width” in
CSS2.1§10.3.5
and the “minimum content width” in
CSS2.1§17.5.2.2
.
min-content block size
The
min-content size
in the
block axis
. For
block containers
, tables, and
inline boxes
, this is equivalent to the
max-content block size
.
fit-content size
fit-content inline size
fit-content block size
If the
available space
in a given axis is
definite
, equal to
clamp(
min-content size
,
stretch-fit size
,
max-content size
)
(i.e.
max(
min-content size
, min(
max-content size
,
stretch-fit size
))
). When sizing under a
min-content constraint
, equal to the
min-content size
. Otherwise, equal to the
max-content size
in that axis.
Note:
This is called the “shrink-to-fit” width in
CSS2.1§10.3.5
and
CSS Multi-column Layout § 3.4
.
intrinsic size
A
max-content size
or
min-content size
, i.e. a size arising primarily from the size of the content. (Some uses of this term may refer also to sizes derived primarily from one of these two sizes.)
Replaced elements
frequently derive their
intrinsic size
from their
natural dimensions
.
The following example applies
fit-content sizing
to the width of the inner
<div>
, and places it within
containing blocks
of varying sizes:
In the narrowest containing block (
2ch
), the box takes its
min-content size
—​overflowing its containing block in order to fully contain all its content, which has wrapped as narrowly as possible.
In the middle containing block (
9ch
), the box takes its
stretch-fit size
—​filling the containing block exactly while its content wraps to fit inside.
In the widest containing block (
16ch
), the box takes its
max-content size
—​smaller than the containing block, but fitting around its unwrapped contents exactly.
<!DOCTYPE html>
<
style
>
div
>
div
{
width
:
fit-content
;
}</
style
>
<
div
style
=
"width: 2ch;"
>
<
div
>
abc def ehg
</
div
>
</
div
>
<
div
style
=
"width: 9ch;"
>
<
div
>
abc def ehg
</
div
>
</
div
>
<
div
style
=
"width: 16ch;"
>
<
div
>
abc def ehg
</
div
>
</
div
>
Using the
min-width
or
max-width
property in place of
width
here would apply these same sizes as the
minimum width
or
maximum width
(respectively), constraining the
preferred width
to find the
used width
. See
§ 3 Specifying Box Sizes
and
§ 3.2 Sizing Values: the <length-percentage>, auto | none, stretch, min-content, max-content, and fit-content values
.
2.2.
Intrinsic Size Contributions
max-content contribution
The size that a box contributes to its
containing block
’s
max-content size
.
min-content contribution
The size that a box contributes to its
containing block
’s
min-content size
.
intrinsic size contribution
A
max-content contribution
,
min-content contribution
, or similarly-calculated content-based size contribution.
Intrinsic size contributions are based on the
outer size
of the box; for this purpose
auto
margins are treated as zero.
If the ideal
max-content contribution
would be smaller than the
min-content contribution
(e.g. due to the use of negative margins) the effective
max-content contribution
is floored by the
min-content contribution
.
2.3.
Intrinsic Size Constraints
max-content constraint
A sizing constraint imposed by the box’s
containing block
that causes it to produce its
max-content contribution
.
min-content constraint
A sizing constraint imposed by the box’s
containing block
that causes it to produce its
min-content contribution
.
preferred aspect ratio
A width:height ratio inherent to a box, which biases various sizing algorithms to produce a size consistent with that aspect ratio insofar as possible while honoring other sizing inputs. Unless otherwise specified, a box’s
preferred aspect ratio
is its
natural aspect ratio
if it has one and is applied to its
content box
. Most boxes do not have a
preferred aspect ratio
.
Tests
replaced-fractional-height-from-aspect-ratio.html
(live test)
(source)
3.
Specifying Box Sizes
3.1.
Sizing Properties
This section defines the
sizing properties
, which specify the
preferred
,
minimum
, and
maximum
sizes of the box to which they are applied. Their potential values are defined in the next section,
§ 3.2 Sizing Values: the <length-percentage>, auto | none, stretch, min-content, max-content, and fit-content values
.
The
flow-relative
variants (
inline-size
,
block-size
, etc.) are mapped using the
writing mode
of the element itself, and interact with their
physical
counterparts (
width
,
height
, etc.) as defined in
[CSS-LOGICAL-1]
. See also
CSS Writing Modes 3
§ 6 Abstract Box Terminology
.
In CSS, while the
minimum
and
maximum
sizes both constrain the
used size
, the
minimum size
constraint is always the strongest constraint. Furthermore, the
inner size
is always floored at zero.
3.1.1.
Preferred Size Properties: the
width
,
height
,
inline-size
, and
block-size
properties
Name:
width
,
height
,
inline-size
,
block-size
Value:
auto
|
<box-size>
Initial:
auto
Applies to:
all elements except
non-replaced
inlines
Inherited:
no
Percentages:
relative to width/height of
containing block
Computed value:
as specified, with
<length-percentage>
values computed
Canonical order:
per grammar
Animation type:
by computed value type, recursing into
fit-content()
Logical property group:
size
Tests
height-composition.html
(live test)
(source)
height-interpolation.html
(live test)
(source)
height-no-interpolation.html
(live test)
(source)
width-composition.html
(live test)
(source)
width-interpolation.html
(live test)
(source)
percentage-height-replaced-content-in-auto-cb.html
(live test)
(source)
height-invalid.html
(live test)
(source)
height-valid.html
(live test)
(source)
width-invalid.html
(live test)
(source)
width-valid.html
(live test)
(source)
The
width
and
height
(
physical
) and
inline-size
and
block-size
(
flow-relative
) are
sizing properties
that specify the
preferred
width
and
height
(
physical
) or
inline size
and
block size
(
flow-relative
) of the box, respectively.
3.1.2.
Minimum Size Properties: the
min-width
,
min-height
,
min-inline-size
, and
min-block-size
properties
Name:
min-width
,
min-height
,
min-inline-size
,
min-block-size
Value:
auto
|
<box-size>
Initial:
auto
Applies to:
all elements that accept
width
or
height
Inherited:
no
Percentages:
relative to width/height of
containing block
Computed value:
as specified, with
<length-percentage>
values computed
Canonical order:
per grammar
Animation type:
by computed value, recursing into
fit-content()
Logical property group:
min-size
Tests
min-height-composition.html
(live test)
(source)
min-height-interpolation.html
(live test)
(source)
min-width-composition.html
(live test)
(source)
min-width-interpolation.html
(live test)
(source)
button-min-width.html
(live test)
(source)
grid-item-image-percentage-min-height-computes-as-0.html
(live test)
(source)
min-height-computed.html
(live test)
(source)
min-height-invalid.html
(live test)
(source)
min-height-valid.html
(live test)
(source)
min-width-computed.html
(live test)
(source)
min-width-invalid.html
(live test)
(source)
min-width-valid.html
(live test)
(source)
The
min-width
and
min-height
(
physical
) and
min-inline-size
and
min-block-size
(
flow-relative
) are
sizing properties
that specify the
minimum width
(“min width”) and
minimum height
(“min height”) or
minimum inline size
(“min inline size”) and
minimum block size
(“min block size”) of the box, respectively.
Note:
The initial value of
auto
is new; in
[CSS2]
the initial value was zero.
3.1.3.
Maximum Size Properties: the
max-width
,
max-height
,
max-inline-size
, and
max-block-size
properties
Name:
max-width
,
max-height
,
max-inline-size
,
max-block-size
Value:
none
|
<box-size>
Initial:
none
Applies to:
all elements that accept
width
or
height
Inherited:
no
Percentages:
relative to width/height of
containing block
Computed value:
as specified, with
<length-percentage>
values computed
Canonical order:
per grammar
Animation type:
by computed value, recursing into
fit-content()
Logical property group:
max-size
Tests
max-height-composition.html
(live test)
(source)
max-height-interpolation.html
(live test)
(source)
max-width-composition.html
(live test)
(source)
max-width-interpolation.html
(live test)
(source)
block-image-percentage-max-height-inside-inline.html
(live test)
(source)
image-percentage-max-height-in-anonymous-block.html
(live test)
(source)
nested-flexbox-image-percentage-max-height-computes-as-none.html
(live test)
(source)
max-height-computed.html
(live test)
(source)
max-height-invalid.html
(live test)
(source)
max-height-valid.html
(live test)
(source)
max-width-computed.html
(live test)
(source)
max-width-invalid.html
(live test)
(source)
max-width-valid.html
(live test)
(source)
The
max-width
and
max-height
(
physical
) and
max-inline-size
and
max-block-size
(
flow-relative
) are
sizing properties
that specify the
maximum width
(“max width”) and
maximum height
(“max height”) or
maximum inline size
(“max inline size) and
maximum block size
(“max block size) of the box, respectively.
3.2.
Sizing Values: the
<length-percentage>
,
auto
|
none
,
stretch
,
min-content
,
max-content
, and
fit-content
values
The following values are used in the
sizing properties
. Values other than
auto
and
none
are grouped under the
<box-size>
production:
<box-size>
=
<length-percentage>
|
stretch
|
min-content
|
max-content
|
fit-content
Note:
The equivalent of
<box-size>
for Level 2 would be just
<length-percentage>
alone.
<length-percentage [0,∞]>
Specifies the size of the box using
<length>
and/or
<percentage>
. The
box-sizing
property indicates whether the
content box
or
border box
is measured.
Percentages are resolved against the width/height, as appropriate, of the box’s
containing block
. If, in a particular axis, the
containing block’s
size depends on the box’s size, see the relevant layout module for special rules on how to resolve percentages.
Negative values are invalid.
Tests
thin-element-render.html
(live test)
(source)
auto
For
width
/
height
, specifies an
automatic size
(
automatic
block size
/
automatic
inline size
). See the relevant layout module for how to calculate this.
For
min-width
/
min-height
, specifies an
automatic minimum size
. Unless otherwise defined by the relevant layout module, however, it resolves to a used value of
0
. For backwards-compatibility, the
resolved value
of this keyword is zero for boxes of all
[CSS2]
display types
(block and inline boxes, inline blocks, and all table display types) when
aspect-ratio
is
auto
. It also resolves to zero when no box is generated.
none
No limit on the size of the box.
stretch
Applies
stretch-fit sizing
, attempting to match the size of the box’s
margin box
to the size of its
containing block
. See
§ 4.2 Stretch-fit Sizing: filling the containing block
.
Tests
block-height-001.html
(live test)
(source)
flex-line-001.html
(live test)
(source)
flex-line-002.html
(live test)
(source)
flex-line-003.html
(live test)
(source)
flex-line-004.html
(live test)
(source)
flex-line-005.html
(live test)
(source)
min-width-1.html
(live test)
(source)
parsing.html
(live test)
(source)
positioned-non-replaced-1.html
(live test)
(source)
positioned-replaced-1.html
(live test)
(source)
positioned-replaced-2.html
(live test)
(source)
positioned-replaced-3.html
(live test)
(source)
stretch-block-size-001.html
(live test)
(source)
stretch-block-size-002.html
(live test)
(source)
stretch-block-size-003.html
(live test)
(source)
stretch-inline-size-001.html
(live test)
(source)
stretch-inline-size-002.html
(live test)
(source)
stretch-inline-size-003.html
(live test)
(source)
stretch-max-block-size-001.html
(live test)
(source)
stretch-max-inline-size-001.html
(live test)
(source)
stretch-min-block-size-001.html
(live test)
(source)
stretch-min-inline-size-001.html
(live test)
(source)
stretch-quirk-001.html
(live test)
(source)
min-content
Use the
min-content size
in the relevant axis; for a box’s
block size
, unless otherwise specified, this is equivalent to its
automatic size
.
Tests
clone-intrinsic-size.html
(live test)
(source)
clone-nowrap-intrinsic-size-bidi.html
(live test)
(source)
clone-nowrap-intrinsic-size.html
(live test)
(source)
min-content-negative-margin-crash.html
(live test)
(source)
replaced-max-height-min-content.html
(live test)
(source)
replaced-max-size-saturation.html
(live test)
(source)
replaced-max-width-min-content.html
(live test)
(source)
replaced-min-height-min-content.html
(live test)
(source)
replaced-min-width-min-content.html
(live test)
(source)
shrink-to-fit-sizing-max-width-min-content.html
(live test)
(source)
slice-intrinsic-size.html
(live test)
(source)
slice-nowrap-intrinsic-size-bidi.html
(live test)
(source)
slice-nowrap-intrinsic-size.html
(live test)
(source)
svg-no-ar-max-height-min-content.html
(live test)
(source)
svg-no-ar-min-height-min-content.html
(live test)
(source)
max-content
Use the
max-content size
in the relevant axis; for a box’s
block size
, unless otherwise specified, this is equivalent to its
automatic size
.
fit-content
Use the
fit-content size
in the relevant axis, i.e.
min(
max-content
, max(
min-content
,
stretch
))
.
Tests
fit-content-block-size-abspos.html
(live test)
(source)
fit-content-block-size-fixedpos.html
(live test)
(source)
fit-content-contribution-001.html
(live test)
(source)
fit-content-min-inline-size.html
(live test)
(source)
fit-content-percentage-padding.html
(live test)
(source)
float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-001.html
(live test)
(source)
float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-002.html
(live test)
(source)
float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-003.html
(live test)
(source)
float-clearance-with-margin-collapse-and-fit-content-and-padding-percentage-004.html
(live test)
(source)
Tests
block-size-with-min-or-max-content-1a.html
(live test)
(source)
block-size-with-min-or-max-content-1b.html
(live test)
(source)
block-size-with-min-or-max-content-2.html
(live test)
(source)
block-size-with-min-or-max-content-3.html
(live test)
(source)
block-size-with-min-or-max-content-4.html
(live test)
(source)
block-size-with-min-or-max-content-5.html
(live test)
(source)
block-size-with-min-or-max-content-6.html
(live test)
(source)
block-size-with-min-or-max-content-7.html
(live test)
(source)
block-size-with-min-or-max-content-table-1a.html
(live test)
(source)
block-size-with-min-or-max-content-table-1b.html
(live test)
(source)
hori-block-size-small-or-larger-than-container-with-min-or-max-content-1.html
(live test)
(source)
hori-block-size-small-or-larger-than-container-with-min-or-max-content-2a.html
(live test)
(source)
hori-block-size-small-or-larger-than-container-with-min-or-max-content-2b.html
(live test)
(source)
image-min-max-content-intrinsic-size-change-001.html
(live test)
(source)
image-min-max-content-intrinsic-size-change-002.html
(live test)
(source)
image-min-max-content-intrinsic-size-change-003.html
(live test)
(source)
image-min-max-content-intrinsic-size-change-004.html
(live test)
(source)
image-min-max-content-intrinsic-size-change-005.html
(live test)
(source)
image-min-max-content-intrinsic-size-change-006.html
(live test)
(source)
image-min-max-content-intrinsic-size-change-007.html
(live test)
(source)
image-min-max-content-intrinsic-size-change-008.html
(live test)
(source)
keyword-sizes-for-intrinsic-contributions.html
(live test)
(source)
keyword-sizes-for-intrinsic-contributions-002.html
(live test)
(source)
keyword-sizes-on-abspos.html
(live test)
(source)
keyword-sizes-on-flex-item-001.html
(live test)
(source)
keyword-sizes-on-flex-item-002.html
(live test)
(source)
keyword-sizes-on-floated-element.html
(live test)
(source)
keyword-sizes-on-inline-block.html
(live test)
(source)
keyword-sizes-on-replaced-element.html
(live test)
(source)
min-content-min-width-000.html
(live test)
(source)
percentage-min-width.html
(live test)
(source)
vert-block-size-small-or-larger-than-container-with-min-or-max-content-1.html
(live test)
(source)
vert-block-size-small-or-larger-than-container-with-min-or-max-content-2a.html
(live test)
(source)
vert-block-size-small-or-larger-than-container-with-min-or-max-content-2b.html
(live test)
(source)
In all cases, the used value is floored to preserve a non-negative
inner size
.
Note:
The
stretch
,
min-content
,
max-content
, and
fit-content
values are new in Level 3.
Note:
The
flex-basis
property hereby also gains these new keywords, as its values are defined by reference to
<'width'>
.
3.2.1.
“Behaving as
auto
”
To have a common term for both when
width
/
height
computes to
auto
and when it is defined to behave as if
auto
were specified (as in the case of
block percentage heights
resolving against an
indefinite
size, see
CSS2§10.5
), the property is said to
behave as auto
in both of these cases.
Note:
Legacy spec prose defining layout behavior, particularly in
[CSS2]
, might explicitly refer to
width
/
height
having a computed value of
auto
as a condition; some of these cases should be interpreted as meaning
behaves as auto
, and reported to the CSSWG for updating.
Tests
margin-collapse-with-indefinite-block-size-001.html
(live test)
(source)
margin-collapse-with-indefinite-block-size-002.html
(live test)
(source)
margin-collapse-with-indefinite-block-size-003.html
(live test)
(source)
margin-collapse-with-indefinite-block-size-004.html
(live test)
(source)
margin-collapse-with-indefinite-block-size-005.html
(live test)
(source)
Replace this section with references to the new term
automatic size
.
3.2.2.
Containing or Excluding Floats
This section is non-normative.
Although
block box
boundaries are typically pervious to floats, sometimes an author needs them to contain their own (descendant) floats or to exclude floats from outside. For Block layout, specifying
display: flow-root
will make the box a
formatting context
root, which has this behavior.
Note:
Boxes participating in Flex, Grid, or Table layout will automatically have this behavior.
3.3.
Box Edges for Sizing: the
box-sizing
property
Name:
box-sizing
Value:
content-box
|
border-box
Initial:
content-box
Applies to:
all elements that accept
width
or
height
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
Tests
box-sizing-replaced-001.xht
(live test)
(source)
box-sizing-replaced-002.xht
(live test)
(source)
box-sizing-replaced-003.xht
(live test)
(source)
box-sizing-computed.html
(live test)
(source)
box-sizing-invalid.html
(live test)
(source)
box-sizing-valid.html
(live test)
(source)
The
box-sizing
property defines whether fixed sizes (such as
<length>
s and
<percentage>
s) are assigned to the
content box
or to the
border box
. It affects the interpretation of all
sizing properties
, including
flex-basis
.
Values have the following meanings:
content-box
Sizes specified on
sizing properties
as
<length-percentage>
represent the box’s
inner sizes
, excluding the margins/border/padding: they are applied to the
content box
. The padding and border of the box are laid out and drawn
outside
the specified
width
and
height
.
Note:
This is the behavior of width and height as specified by CSS2.1, and is thus the default.
Tests
box-sizing-content-box-001.xht
(live test)
(source)
box-sizing-content-box-002.xht
(live test)
(source)
box-sizing-content-box-003.xht
(live test)
(source)
border-box
Any
<length-percentage>
values in the
sizing properties
are applied to the
border box
, thus representing the box’s visually-apparent sizes: the
padding
and
border
of the box (but not its
margins
) are essentially laid out and drawn
inside
the used
width
and
height
, with the
content box
sized to fill the remaining space. More specifically, the
content box
width
and
height
are calculated by subtracting the
border
and
padding
in the corresponding axis from the specified
<length-percentage>
, and flooring the result at zero (as the
inner size
of a box cannot be negative).
Used values of the
sizing properties
, as exposed for instance through
getComputedStyle()
, also refer to the
border box
.
Tests
border-box-and-max-content-001.html
(live test)
(source)
border-box-and-max-content-002.html
(live test)
(source)
border-box-and-max-content-003.html
(live test)
(source)
box-sizing-border-box-001.xht
(live test)
(source)
box-sizing-border-box-002.xht
(live test)
(source)
box-sizing-border-box-003.xht
(live test)
(source)
box-sizing-border-box-004.xht
(live test)
(source)
table-child-percentage-height-with-border-box.html
(live test)
(source)
Values affected by
box-sizing
include both raw
<length-percentage>
values and those used in functional notations such as
fit-content()
[css-sizing-4]
. In contrast, non-quantitative values such as
auto
and
min-content
are not influenced by the
box-sizing
property (unless otherwise specified).
For example, the following properties set the content-box size of the box to
100px
, with the border-box size calculating to
120px
:
.box
{
box-sizing
:
content-box
;
/* default */
width:
100
px
;
padding-left
:
10
px
;
border-left
:
10
px
solid
;
}
On the other hand, by changing to
border-box
, the border-box is set to
100px
, with the content-box size calculating to
80px
:
.box
{
box-sizing
:
border-box
;
width
:
100
px
;
padding-left
:
10
px
;
border-left
:
10
px
solid
;
}
The
inner size
can’t be less than zero, so if the
padding
+
border
is greater than the specified border-box size, the box will end up larger than specified. In this case, the content-box size will floor at
0px
so the border-box size ends up at
120px
, even though
width: 100px
is specified for the border box:
.box
{
box-sizing
:
border-box
;
width
:
100
px
;
padding-left
:
60
px
;
border-left
:
60
px
solid
;
/* padding + border = 120px */
}
This example uses box-sizing to evenly horizontally split two divs with fixed size borders inside a div container, which would otherwise require additional markup.
sample CSS:
div.container
{
width
:
38
em
;
border
:
1
em
solid black
;
}
div.split
{
box-sizing
:
border-box
;
width
:
50
%
;
border
:
1
em
silver ridge
;
float
:
left
;
}
sample HTML fragment:
<
div
class
=
"container"
>
<
div
class
=
"split"
>
This div occupies the left half.
</
div
>
<
div
class
=
"split"
>
This div occupies the right half.
</
div
>
</
div
>
demonstration of sample CSS and HTML:
This div should occupy the left half.
This div should occupy the right half.
The two divs above should appear side by side, each (including borders) 50% of the content width of their container. If instead they are stacked one on top of the other then your browser does not support
box-sizing
.
Note:
Certain HTML elements, such as
button
, default to
border-box
behavior. See HTML for details on which elements have this behavior.
In legacy CSS specifications, the terms
width
,
height
,
minimum (min) width
,
minimum (min) height
,
maximum (max) width
, and
maximum (max) height
generally refer to the
inner
size (
content-box
size) of a
box
unless otherwise indicated.
Refer to
CSS User Interface 3
§ 3.1 Changing the Box Model: the box-sizing property
for an explicit disambiguation of these terms for the
Visual formatting model details
section of
[CSS2]
.
To avoid ambiguities, specification authors should avoid ambiguous uses of terms such as width or height without further qualification, and should explicitly refer and link to the
inner
size, the
outer
size, the size of the
border-box
, the
computed value
of the
sizing properties
, etc, as appropriate for each case.
3.4.
New Column Sizing Values: the
stretch
,
min-content
,
max-content
, and
fit-content
values
Name:
column-width
New values:
<box-size>
Computed value:
as specified, with
<length-percentage>
values computed
Animation type:
by computed value type
When used as values for
column-width
, the new keywords specify the optimal column width:
stretch
Specifies the optimal column width as the
stretch-fit inline size
of the multi-column container.
min-content
Specifies the optimal column width as the
min-content inline size
of the multi-column container’s contents.
max-content
Specifies the optimal column width as the
max-content inline size
of the multi-column container’s contents.
fit-content
Specifies the optimal column width as
min(
max-content inline size
, max(
min-content inline size
,
stretch-fit inline size
))
.
Note:
The column width never varies by column. When the column width is informed by the multi-column container’s contents (as in the keywords above), all of its contents are taken under consideration and the calculated width is shared by all the columns.
4.
Extrinsic Size Determination
Extrinsic sizing
determines sizes based on the context of an element, without regard for its contents.
4.1.
Percentage Sizing
Percentages specify sizing of a box with respect to the box’s
containing block
.
For example, in the following markup:
<
article
style
=
"height: 60em"
>
<
figure
style
=
"height: 50%;"
>
<
img
style
=
"height: 50%;"
>
</
figure
>
</
article
>
the
<figure>
would be
30em
tall = 50% of the
definite
60em
height of the
<article>
the
<img>
would be
15em
tall = 50% of the
<figure>
’s height (which is itself
definite
because it’s a percentage resolved against a
definite
length)
See
§ 5.2.1 Intrinsic Contributions of Percentage-Sized Boxes
for details on how to resolve percentages when the size of the
containing block
depends on the size of its content.
Tests
percentage-height-in-flexbox.html
(live test)
(source)
range-percent-intrinsic-size-1.html
(live test)
(source)
range-percent-intrinsic-size-2.html
(live test)
(source)
range-percent-intrinsic-size-2a.html
(live test)
(source)
4.2.
Stretch-fit Sizing: filling the containing block
Stretch-fit sizing tries to set the box’s used size to the length necessary to make its
principal box
’s
outer size
as close to filling the
containing block
as possible while still respecting the constraints imposed by
min-height
/
min-width
/
max-height
/
max-width
.
If used in an axis where percentage sizes can resolve to a
definite
value
Sizes the
margin box
to fill the
containing block
exactly, treating
auto
margins as zero. (If this would make the
inner size
negative, it instead sizes the
margin box
so that the
inner size
is zero.)
For
in-flow
block-level boxes
that form an
independent formatting context
, use the space available to
line boxes
(i.e. excluding floats) in place of the
containing block
’s
inline size
.
For
in-flow
block-level boxes
when resolving the
block axis
size: if the ancestor element percentages resolve against does not have a
block-start
border
or
padding
and is not an
independent formatting context
, treat the element’s
block-start
margin as zero for the purpose of calculating this size. Do the same for the
block-end
margin, analogously.
Note:
This simulates the effect of margins collapsing with the parent’s margin. It doesn’t actually suppress the margins, so if anything prevents the element from actually collapsing with its parent, the
stretch-fit size
might actually be too large to fit in the parent perfectly.
Otherwise
In a
preferred size property
,
behaves as auto
. In a
min size property
, behaves as
0
. In a
max size property
, behaves as
none
.
For example, given the following HTML representing two
block boxes
:
<div class="parent"> <div class="child">text</div> </div>
In the following case, the
outer height
of the child box will exactly match the height of the parent box (200px), but its
inner height
will be 20px less, to account for its margins.
.parent { height: 200px; border: solid; } .child { height: stretch; margin: 10px; }
On the other hand, in this case we can assume that the child’s margins will collapse with the parent, so the inner box will be
200px
tall, exactly filling the parent.
.outer { height: 200px; margin: 0; } .inner { height: stretch; margin: 10px; }
(The top margins will in fact collapse, but the bottom margins do not collapse, because the bottom margin of a box is not adjoining to the bottom margin of a parent with a non-
auto
height, see
CSS 2
§ 8.3.1 Collapsing margins
. Luckily, an overflowing bottom margin doesn’t have any visible effect.)
Similarly,
width: stretch
causes the box to fill its container, being 20px narrower than the width of "some more text" (due to the 10px margin):
<div class="parent"> <div class="child">text</div> </div> some more text
.parent { float: left; margin: 0; } .child { width: stretch; margin: 10px; }
On the other hand, in this example the container’s height is indefinite, which would cause a percentage height on the child to
behave as auto
, so
height: stretch
behaves as auto
as well.
.parent { height: auto; margin: 0; } .child { height: stretch; margin: 10px; }
Tests
abspos-1.html
(live test)
(source)
abspos-2.html
(live test)
(source)
aspect-ratio-1.html
(live test)
(source)
aspect-ratio-2.html
(live test)
(source)
auto-margins-1.html
(live test)
(source)
auto-margins-2.html
(live test)
(source)
bfc-next-to-float-1.html
(live test)
(source)
bfc-next-to-float-2.html
(live test)
(source)
block-height-002.html
(live test)
(source)
block-height-003.html
(live test)
(source)
block-height-004.html
(live test)
(source)
block-height-005.html
(live test)
(source)
block-height-006.html
(live test)
(source)
block-height-007.html
(live test)
(source)
block-height-008.html
(live test)
(source)
block-height-009.html
(live test)
(source)
block-height-010.html
(live test)
(source)
cache-miss-001.html
(live test)
(source)
cache-miss-002.html
(live test)
(source)
content-contribution-001.html
(live test)
(source)
fixed-table-1.html
(live test)
(source)
flex-basis-1.html
(live test)
(source)
flexbox-auto-minimum-001.html
(live test)
(source)
flexbox-auto-minimum-002.html
(live test)
(source)
flexbox-flex-base-size-001.html
(live test)
(source)
flexbox-flex-base-size-002.html
(live test)
(source)
flexbox-stretch-minimum-001.html
(live test)
(source)
flexbox-stretch-minimum-002.html
(live test)
(source)
indefinite-1.html
(live test)
(source)
indefinite-2.html
(live test)
(source)
indefinite-3.html
(live test)
(source)
indefinite-4.html
(live test)
(source)
replaced-next-to-float-1.html
(live test)
(source)
replaced-next-to-float-2.html
(live test)
(source)
stretch-table-001.html
(live test)
(source)
Note:
The
stretch-fit size
is usually, but not always, equivalent to
stretch
self-alignment
. For example, in
multi-line
flex layout
, the
stretch-fit size
resolves against the
containing block
directly, and contributes that size to the
cross size
of the line—​whereas
stretch
contributes the
fit-content size
—​before finally resolving against the
cross size
of the line.
5.
Intrinsic Size Determination
Intrinsic sizing
determines sizes based on the contents of an element, without regard for its context.
Tests
canvas-intrinsic-dynamic.html
(live test)
(source)
intrinsic-percent-replaced-dynamic-001.html
(live test)
(source)
intrinsic-percent-replaced-dynamic-002.html
(live test)
(source)
intrinsic-percent-replaced-dynamic-003.html
(live test)
(source)
intrinsic-percent-replaced-dynamic-004.html
(live test)
(source)
intrinsic-percent-replaced-dynamic-005.html
(live test)
(source)
intrinsic-percent-replaced-dynamic-006.html
(live test)
(source)
intrinsic-percent-replaced-dynamic-007.html
(live test)
(source)
intrinsic-percent-replaced-dynamic-008.html
(live test)
(source)
intrinsic-percent-replaced-dynamic-009.html
(live test)
(source)
intrinsic-percent-replaced-dynamic-010.html
(live test)
(source)
intrinsic-percent-replaced-dynamic-011.html
(live test)
(source)
intrinsic-percent-replaced-dynamic-012.html
(live test)
(source)
intrinsic-ratio-replaced-box-sizing.html
(live test)
(source)
intrinsic-size-fallback-replaced.html
(live test)
(source)
5.1.
Intrinsic Sizes
The
min-content size
of a box in each axis is the size it would have if it was a float given an
auto
preferred size
in that axis (and no
minimum
or
maximum size
in that axis) and if its containing block was
zero
-sized in that axis. (In other words, the minimum size it has when sized as “shrink-to-fit”.)
The
max-content size
of a box in each axis is the size it would have if it was a float given an
auto
preferred size
in that axis (and no
minimum
or
maximum size
in that axis), and if its containing block was
infinitely
-sized in that axis. (In other words, the maximum size it has when sized as “shrink-to-fit”.)
The
min-content size
and
max-content size
are collectively referred to as the
intrinsic sizes
.
Note:
When the box has a
preferred aspect ratio
, size constraints in the opposite dimension will transfer through and can affect the
auto
size in the considered one. See
CSS2§10
.
This specification does not define how to determine the sizes of floats. Please refer to
[CSS2]
. However, the
intrinsic sizes
of
replaced elements
without
natural sizes
are defined below:
If it has a non-degenerate
preferred aspect ratio
:
For the
min-content size
, use zero.
For the
max-content size
:
If the
available space
is
definite
in the
inline axis
, use the
stretch fit
into that size for the inline size and calculate the block size using the aspect ratio.
Otherwise if the box has a
<length>
as its
computed value
for
min-width
or
min-height
, use that size and calculate the other dimension using the aspect ratio; if both dimensions have a
<length>
minimum, choose the one that results in the larger overall size.
Note:
This case was previous calculated from a 300x150 default size, rather than the box’s min size. This is believed to be a better behavior, and likely to be Web-compatible, but please send feedback to the CSSWG if there are any problems.
Tests
replaced-fractional-height-from-aspect-ratio-2.html
(live test)
(source)
Otherwise use an
inline size
matching the corresponding dimension of the
initial containing block
and calculate the other dimension using the aspect ratio.
If it has no
preferred aspect ratio
:
For both the
min-content size
and
max-content size
:
If the box has a
<length>
as its
computed
minimum size
(
min-width
/
min-height
) in that dimension, use that size.
Note:
This author-controllable behavior is made possible by the new
auto
value for the
min size properties
. This is believed to be a better behavior, but it is not yet clear if it is Web-compatible, so please send feedback to the CSSWG if there are any problems.
Otherwise, use
300px
for the width and/or
150px
for the height as needed.
Note:
This does not imply an aspect ratio.
Since a block-level or inline-level replaced element whose
height
or
width
behaves as auto
is effectively defined to use its
max-content size
(
CSS2§10.3.2
), this specification applies the rules above to the undefined case of a replaced element whose
height
and
width
both
behave as auto
.
Note:
This specification does not define how to determine the size of a float. Please refer to
[CSS2]
, the relevant CSS specification for that display type, and/or existing implementations for further details. A future specification will define this in detail, replacing the CSS2 “definition”, such as it is.
Although the
auto
size of text input controls such as HTML’s
<input type=text>
and
<textarea>
elements is typically a fixed size, the contents of such elements can be used to determine a content-based
intrinsic size
, as for non-replaced block containers. The
min-content
and
max-content
keywords of the
sizing properties
thus represent content-based sizes for form controls which render their value as text contained within their box, allowing such controls to size to fit their visible contents similarly to regular non-replaced elements.
The content in this case is defined to be the input control’s values (the
raw value
in the case of
textarea
, or the
value
in the case of
input
), possibly transformed to a more human-readable and/or localized display format, which is then treated as child
text sequences
of the input control, allowing
soft wrap opportunities
only where the input control would actually allow wrapping (whether keyed off of CSS properties or other, UA-internal constraints). If the input control has designated placeholder text to be overlaid in its value display area, then that text is also measured for the purpose of calculating the content-based size—​whether or not the placeholder text is visible at the moment. (Thus the content-based
intrinsic size
of the input control is the larger of the size to fit the placeholder text and the size to fit the value.)
The UA may enforce a minimum (such as the size required to contain a single zero-width character, or the smallest usable size of a touch target) on the form control’s
min-content
and
max-content sizes
to ensure sufficient space for the caret and otherwise maintain usability of the form control.
Note:
This might be extended to
iframe
or other content-containing replaced elements (see
discussion
), but text inputs are a major use-case; and being document-internal, have the least additional complications.
Tests
aspect-ratio-affects-container-width-when-height-changes.html
(live test)
(source)
calc-margins-block.html
(live test)
(source)
calc-margins-fieldset-content.html
(live test)
(source)
calc-margins-fieldset-legend.html
(live test)
(source)
calc-margins-flex.html
(live test)
(source)
calc-margins-table-caption.html
(live test)
(source)
image-fractional-height-with-wide-aspect-ratio.html
(live test)
(source)
intrinsic-fixed-width-with-max-content-height.html
(live test)
(source)
intrinsic-fixed-width-with-min-content-height.html
(live test)
(source)
intrinsic-percent-replaced-001.html
(live test)
(source)
intrinsic-percent-replaced-002.html
(live test)
(source)
intrinsic-percent-replaced-003.html
(live test)
(source)
intrinsic-percent-replaced-004.html
(live test)
(source)
intrinsic-percent-replaced-005.html
(live test)
(source)
intrinsic-percent-replaced-006.html
(live test)
(source)
intrinsic-percent-replaced-007.html
(live test)
(source)
intrinsic-percent-replaced-008.html
(live test)
(source)
intrinsic-percent-replaced-009.html
(live test)
(source)
intrinsic-percent-replaced-010.html
(live test)
(source)
intrinsic-percent-replaced-011.html
(live test)
(source)
intrinsic-percent-replaced-012.html
(live test)
(source)
intrinsic-percent-replaced-013.html
(live test)
(source)
intrinsic-percent-replaced-014.html
(live test)
(source)
intrinsic-percent-replaced-015.html
(live test)
(source)
intrinsic-percent-replaced-016.html
(live test)
(source)
intrinsic-percent-replaced-017.html
(live test)
(source)
intrinsic-percent-replaced-018.html
(live test)
(source)
intrinsic-percent-replaced-019.html
(live test)
(source)
intrinsic-percent-replaced-020.html
(live test)
(source)
intrinsic-percent-replaced-021.html
(live test)
(source)
intrinsic-percent-replaced-022.html
(live test)
(source)
intrinsic-percent-replaced-023.html
(live test)
(source)
intrinsic-percent-replaced-024.html
(live test)
(source)
intrinsic-percent-replaced-025.html
(live test)
(source)
intrinsic-percent-replaced-026.html
(live test)
(source)
intrinsic-percent-replaced-027.html
(live test)
(source)
intrinsic-size-fallback-video.html
(live test)
(source)
max-content-input-001.html
(live test)
(source)
ortho-writing-mode-001.html
(live test)
(source)
orthogonal-writing-mode-float-in-inline.html
(live test)
(source)
replaced-aspect-ratio-intrinsic-size-001.html
(live test)
(source)
replaced-aspect-ratio-intrinsic-size-002.html
(live test)
(source)
replaced-aspect-ratio-stretch-fit-001.html
(live test)
(source)
replaced-aspect-ratio-stretch-fit-002.html
(live test)
(source)
replaced-aspect-ratio-stretch-fit-003.html
(live test)
(source)
svg-intrinsic-size-001.html
(live test)
(source)
svg-intrinsic-size-002.html
(live test)
(source)
svg-intrinsic-size-003.html
(live test)
(source)
svg-intrinsic-size-004.html
(live test)
(source)
svg-intrinsic-size-005.html
(live test)
(source)
svg-intrinsic-size-006.html
(live test)
(source)
whitespace-and-break.html
(live test)
(source)
5.2.
Intrinsic Contributions
A box’s
min-content contribution
/
max-content contribution
in each axis is the size of the content box of a hypothetical
auto
-sized float that contains only that box, if that hypothetical float’s containing block is zero-sized/infinitely-sized.
Note:
This specification does not define precisely how to determine these sizes. Please refer to
[CSS2]
, the relevant CSS specification for that display type, the
rules for handling percentages
(below), and/or existing implementations for further details.
Tests
intrinsic-percent-non-replaced-001.html
(live test)
(source)
intrinsic-percent-non-replaced-002.html
(live test)
(source)
intrinsic-percent-non-replaced-003.html
(live test)
(source)
intrinsic-percent-non-replaced-004.html
(live test)
(source)
intrinsic-percent-non-replaced-005.html
(live test)
(source)
For this purpose,
stretch
self-alignment
and
stretch-fit sizing
(when they are able to resolve extrinsically) are considered definite the same way as resolveable percentages.
5.2.1.
Intrinsic Contributions of Percentage-Sized Boxes
Sometimes the size of a percentage-sized box’s
containing block
depends on the
intrinsic size contribution
of the box itself, creating a cyclic dependency. When calculating the
intrinsic size contribution
of such a box (including any calculations for a content-based
automatic minimum size
), a percentage value that resolves against a size in the same axis as the
intrinsic size contribution
(a
cyclic percentage size
) is resolved specially:
If the box is
non-replaced
, then the entire value of any
max size property
or
preferred size property
(
width
/
max-width
/
height
/
max-height
) specified as an expression
containing a percentage
(such as
10%
or
calc(10px + 0%)
) that is
cyclic
is treated
for the purpose of calculating the box’s
intrinsic size contributions
only
as that property’s
initial value
. For example, given a box with
width: calc(20px + 50%)
, its max-content contribution is calculated as if its
width
were
auto
. (The percentage is honored as usual, however, during the actual sizing of the box itself; see below.)
Likewise, if the box is
replaced
, then the entire value of any
max size property
or
preferred size property
specified as an expression containing a percentage that is
cyclic
is treated
for the purpose of calculating the box’s
max-content contributions
only
as that property’s
initial value
.
If the box is
replaced
, a
cyclic percentage
in the value of any
max size property
or
preferred size property
(
width
/
max-width
/
height
/
max-height
), is resolved against zero when calculating the
min-content contribution
in the corresponding axis. (See
§ 5.2.2 Compressible Replaced Elements
for a list of which elements in HTML this applies to.) If the box also has a
preferred aspect ratio
, then this
min-content contribution
is floored by any
<length-percentage>
minimum size
from the opposite axis—​resolving any such percentage against zero—​transferred through the
preferred aspect ratio
.
Should we resolve transferred percentages against their containing block instead of zero before transferring them? See
discussion
.
The UA may additionally floor the
min-content contribution
based on UI considerations, such as ensuring certain UI elements remain visible (for example, the dropdown arrow on a
select
).
Note:
The
min-content contribution
is, as always, also floored by the
minimum size
in its own axis.
This rule also applies when calculating a content-based
automatic minimum size
or its corresponding size contribution, yielding a
definite
“specified size suggestion”.
Tests
intrinsic-percent-replaced-028.html
(live test)
(source)
For example, an
input
assigned
width: calc(50% + 50px)
has a
min-content contribution
of
50px
, plus any horizontal margin/border/padding.
For the
min size properties
, as well as for
margins
and
paddings
(and
gutters
), a
cyclic percentage
is resolved against zero for determining
intrinsic size contributions
.
Tests
inline-intrinsic-size-calc.html
(live test)
(source)
intrinsic-percent-non-replaced-006.html
(live test)
(source)
Summary of the Cyclic-Percentage Intrinsic Size Contribution Rules (Above)
Element Type
Replaced
Non-replaced
Contribution Type
min-content
max-content
min-content
max-content
min size
&
margin
/
padding
zeroᵈ
zeroᵈ
zeroᵈ
zeroᵈ
max
&
preferred size
zeroᶜ
initialᵇ
initialᵃ
initialᵃ
Then, unless otherwise specified, when calculating the used sizes and positions of the containing block’s
contents
:
If the cyclic dependency was introduced due to a
block-axis
size other than a
minimum size
on the containing block (i.e. a
block-size
or
max-block-size
in most layout modes, or a
flex-basis
in
flex layout
) that causes it to depend on the size of its contents, the box’s percentage is not resolved and instead
behaves as auto
.
Note:
Grid items
in both axes,
flex items
in the
main axis
, and children of
flex items
in both axes do allow percentages to resolve in this case.
Otherwise, the percentage is resolved against the containing block’s size. (The containing block’s size is not re-resolved based on the resulting size of the box; the contents might thus overflow or underflow the containing block).
Note:
These rules specify the previously-undefined behavior of this cyclic case in
CSS2§10.2
,
CSS2§8.3
, and
CSS2§8.4
. Note also, the behavior in
CSS2§10.5
is superseded in their respective specifications for layout modes (such as
flex layout
) not described in CSS2.
For example, in the following markup:
<
article
style
=
"width: min-content"
>
<
aside
style
=
"width: 50%;"
>
LOOOOOOOOOOOOOOOOOOOONG
</
aside
>
</
article
>
When calculating the width of the outer
<article>
, the inner
<aside>
behaves as
width: auto
, so the
<article>
sets itself to the width of the long word. Since the
<article>
’s width didn’t depend on "real" layout, though, it’s treated as
definite
for resolving the
<aside>
, whose width resolves to half that of the
<article>
.
In this example,
<
article
style
=
"height:auto"
>
<
aside
style
=
"height: 50%;"
>
<
div
class
=
block
style
=
"height: 150px;"
></
div
>
</
aside
>
<
section
style
=
"height: 30px;"
></
section
>
</
article
>
because the percentage
block size
(
height
, in this case) on block-level elements is defined to not resolve inside content-sized containing blocks, the percentage height on the
<aside>
is ignored, that is, it behaves exactly as if
auto
were specified.
Letting percentages still resolve against a definite
height
when the min-height is intrinsic is an open issue. (CSS2 has a general statement about "height depending on contents", which this technically is, even though CSS2 didn’t have content-dependent keywords for
min-height
. Since this is new, we think we could have this different behavior.)
The following examples illustrate how block-axis percentages resolve against a containing block whose size depends on its contents.
<
article
style
=
"height:100px; min-height: min-content;"
>
<
aside
style
=
"height: 50%;"
>
<
div
style
=
"height: 150px;"
></
div
>
</
aside
>
<
section
style
=
"height: 30px;"
></
section
>
</
article
>
The initial height of the
<article>
is 100px, as specified, which would make the
<aside>
50px tall when it resolved its percentage. However, we must calculate the min-height, by substituting it in for
height
. This causes the percentage on the
<aside>
to
behave as auto
, so the
<aside>
ends up 150px tall. The total height of the contents is thus 180px. This is larger than the specified 100px height, so the
<article>
gets adjusted to 180px tall.
Then, since the percentage could
originally
resolve against the (100px) height, it now resolves against the 180px height, so the
<aside>
ends up being 90px tall.
<
article
style
=
"height:auto; min-height: min-content;"
>
<
aside
style
=
"height: 50%;"
>
<
div
class
=
block
style
=
"height: 150px;"
></
div
>
</
aside
>
<
section
style
=
"height: 30px;"
></
section
>
</
article
>
In this case, the percentage on the
<aside>
won’t normally resolve, because the containing block’s height is
auto
(and thus depends on the size of its contents). Instead it
behaves as auto
, resulting in a height of 150px for the
<aside>
, and an initial height of 180px for the
<article>
The
min-height
doesn’t change this;
height: min-content;
acts similarly to
height: auto;
and results in the same sizes.
<
article
style
=
"height:100px; min-height: min-content;"
>
<
aside
style
=
"height: 200%;"
>
<
div
style
=
"height: 150px;"
></
div
>
</
aside
>
<
section
style
=
"height: 30px;"
></
section
>
</
article
>
This is a variation on the first code block, and follows a similar path; the
<aside>
initially wants to compute to 200px tall (200% of the 100px containing block height). When we calculate the effects of
min-height
, the percentage
behaves as auto
, causing it to become 150px tall, and the total
min-content
height of the containing block to be 180px tall. Since this is larger than 100px, the
<article>
gets clamped to 180px, the percentage resolves against this new height, and the
<aside>
ends up being 360px tall, overflowing the
<article>
5.2.2.
Compressible Replaced Elements
In addition to the
replaced elements
listed in
HTML§14.4
[HTML]
, the following HTML elements are also considered to be
replaced elements
for the purpose of the
percentage-sized replaced element rule
above, and can have their
min-content contribution
compressed when their
width
/
height
or
max-width
/
max-height
is expressed with a cyclic percentage size:
input
with any
type
that is not "button-like"; this can vary depending on the UA.
A type is "button-like" in a particular UA if it displays similar to a
button
element, where it can contains actual content that determines the layout of the element. In most UAs, the "button", "reset", "submit", and "color" types are button-like; the "file" type is also partially button-like in some UAs, when it’s displayed as a combination of a text input (shrinkable) and a button (button-like, and thus not shrinkable).
select
,
textarea
,
progress
,
meter
,
marquee
.
Tracking web-compat & implementation progress of applying this to max-width/height in
Issue 6348
.
[Issue #6348]
Changes
Recent Changes
Changes since the
17 December 2021 Working Draft
include:
Moved
fit-content()
to Level 4, pulled
fit-content
and
stretch
from Level 4. (
Issue 10601
)
Imported definitions of the
flow-relative
sizing properties
from
[CSS-LOGICAL-1]
. (
Issue 10189
)
Clarified details for
stretch-fit sizing
. (
Issue 11044
,
Issue 11006
,
Issue 11076
,
Issue 4028
,
Issue 13260
,
Issue 11489
)
Introduced
<box-size>
grammar production to consolidate sizing values. (
Issue 13478
)
Clarified that rules for finding the intrinsic size of replaced elements with a
preferred aspect ratio
only apply when the aspect ratio is non-degenerate. (
Issue 12612
)
Defined that
aspect-ratio
[css-sizing-4]
preserves the
resolved value
of an
auto
minimum size
as
auto
. (
Issue 11716
)
Clarified that the
max-content size
and
max-content contribution
are floored by the
min-content size
/
min-content contribution
. (
Issue 12076
)
Clarified handling of cyclic percentages in intrinsic size contribution calculations. (
Issue 6822
)
Clarified that stretch sizes are handled similarly to percentages for the purpose of intrinsic size calculations. (
Issue 10619
)
Clarified indefiniteness of intrinsic size keywords. (
Issue 7206
)
Added logical property groups to the
sizing properties
. (
Issue 2822
)
Added Web Platform Tests coverage.
Various other minor editorial fixes and improvements.
Changes since the
18 December 2020 Working Draft
include:
Fixed the order of
contain-intrinsic-size
values when
auto
is combined with other values so that parsing is unambiguous. (
Issue 6391
)
Clarified which elements are allowed to not have a last remembered size for
contain-intrinsic-size: auto
. (
Issue 6220
)
Limited
contain-intrinsic-size: auto
to when
content-visibility
is
auto
. (
Issue 6308
)
Changes since the
18 December 2020 Working Draft
include:
Fixed various errors in definition of max-content sizes of replaced elements in
§ 5.1 Intrinsic Sizes
. (
Issue 6072
)
Added missing statement handling
min-content constraint
to definition of
fit-content size
.
Renamed replaced element “intrinsic” dimensions to “natural” dimensions in order to avoid confusion with
intrinsic sizes
(see
Issue 4961
).
Various other minor editorial fixes and improvements.
Major changes since the
22 May 2019 Working Draft
include:
Defined that
min-content
and
max-content
do not necessarily behave the same as the property’s initial value if otherwise specified (by the relevant layout module). (
Issue 3973
)
Switched intrinsic contribution of
fit-content()
to treat its argument as that argument would be treated alone for intrinsic contribution calculations and resolve the fit-content formula accordingly, rather than having special behavior for
fit-content()
resolution when calculating intrinsic contributions. (
Issue 3731
)
Changed the
max-content size
of replaced boxes without an intrinsic size to use their
minimum size
in place of ICB or 300px×150px only when it is a
<length>
, see
§ 5.1 Intrinsic Sizes
. (
Issue 4217
)
Switched default sizing of an object with a natural aspect ratio to use the ICB size instead of 300px×150px. (
Issue 4218
)
Defined
preferred aspect ratio
and used it in place of “intrinsic aspect ratio” where appropriate.
Miscellaneous minor / editorial fixes.
Major changes since the
4 March 2018 Working Draft
include:
Imported the
box-sizing
definition from
CSS UI Level 3
.
More rigorously specified handling of
cyclic percentages
. (
#1132
,
#2384
,
#2297
,
#2674
)
Changed the
*-content
values applied to the bock axis to not compute to the property’s initial value, but to rather “behave as” the property’s initial value. (
#2708
)
Fixed miscellaneous trivial errors.
Major changes since the
7 February 2017 Working Draft
include:
More accurate definition of min-content and max-content sizes for replaced elements.
Compute new keywords to the initial value, not to a potentially non-existent
auto
, when applied to the block axis.
Specify that percent sizes on replaced elements zero out their min-content contribution.
Fix confusing/wrong definition of percentage sizes resolved against a dependent containing block. (This may require further work.)
Deferred the
stretch
and
fit-content
keywords to Level 4 to allow for further consideration of their behavior in
indefinite
containing blocks.
Pulled in full definitions for all of the sizing properties (rather than diffing them):
width
,
height
,
min-width
,
min-height
, max-width',
max-height
, and
box-sizing
.
Additions since CSS Level 2
In addition to substantially more detail to the various automatic and content-based sizing algorithms, the following new features have been added since
[CSS2]
:
The
box-sizing
property (originally defined in
[CSS-UI-3]
, then moved here).
The
stretch
,
min-content
,
max-content
, and
fit-content
values of the
sizing properties
.
The
auto
initial value of the
min-width
and
min-height
properties (originally defined in
[CSS-FLEXBOX-1]
, then moved here).
Acknowledgments
Special thanks go to L. David Baron, Aaron Gustafson, Daniel Holbert, and Mats Palmgren for their contributions to this module.
Privacy Considerations
In order to support automatic layout, CSS sizes boxes to fit their contents. In conjunction with various
[DOM]
and
[CSSOM]
APIs which can return the size of those boxes to script, this can expose information about those contents. However, this information is more directly and easily available by inspecting the DOM for the contents, rather than indirecting through the box’s size. Containers that can’t have their contents inspected (such as cross-origin
iframe
s) also do not expose sizing information to the outer page, except insofar as
replaced elements
such as images expose their natural size and/or aspect ratio.
Security Considerations
No new security considerations have been reported on this specification.
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
, in § 3.2
automatic block size
, in § 3.2
automatic inline size
, in § 3.2
automatic minimum size
, in § 3.2
automatic size
, in § 3.2
available
, in § 2
available block space
, in § 2
available inline space
, in § 2
available space
, in § 2
behave as auto
, in § 3.2.1
behaves as auto
, in § 3.2.1
behaving as auto
, in § 3.2.1
block size
, in § 3.1.1
block-size
, in § 3.1.1
border-box
, in § 3.3
<box-size>
, in § 3.2
box-sizing
, in § 3.3
content-box
, in § 3.3
cyclic percentage
, in § 5.2.1
cyclic percentage size
, in § 5.2.1
definite
, in § 2
definite size
, in § 2
Extrinsic sizing
, in § 4
fallback
, in § 2
fallback size
, in § 2
fit-content
value for column-width
, in § 3.4
value for width, min-width, max-width, height, min-height, max-height
, in § 3.2
fit-content block size
, in § 2.1
fit-content inline size
, in § 2.1
fit-content size
, in § 2.1
height
(property)
, in § 3.1.1
definition of
, in § 3.1.1
indefinite
, in § 2
indefinite size
, in § 2
inline size
, in § 3.1.1
inline-size
, in § 3.1.1
inner block size
, in § 2
inner height
, in § 2
inner inline size
, in § 2
inner size
, in § 2
inner width
, in § 2
intrinsic size
, in § 2.1
intrinsic size constraint
, in § 2.2
intrinsic size contribution
, in § 2.2
Intrinsic sizing
, in § 5
<length-percentage [0,∞]>
, in § 3.2
max block size
, in § 3.1.3
max-block-size
, in § 3.1.3
max-content
definition of
, in § 2.1
value for column-width
, in § 3.4
value for width, min-width, max-width, height, min-height, max-height
, in § 3.2
max-content block size
, in § 2.1
max-content block-size contribution
, in § 2.2
max-content constraint
, in § 2.3
max-content contribution
, in § 2.2
max-content inline size
, in § 2.1
max-content inline-size contribution
, in § 2.2
max-content size
, in § 2.1
max height
, in § 3.1.3
max-height
, in § 3.1.3
maximum block size
, in § 3.1.3
maximum height
, in § 3.1.3
maximum inline size
, in § 3.1.3
maximum size
, in § 3.1.3
maximum width
, in § 3.1.3
max inline size
, in § 3.1.3
max-inline-size
, in § 3.1.3
max size
, in § 3.1.3
max size property
, in § 3.1.2
max width
, in § 3.1.3
max-width
, in § 3.1.3
min block size
, in § 3.1.2
min-block-size
, in § 3.1.2
min-content
definition of
, in § 2.1
value for column-width
, in § 3.4
value for width, min-width, max-width, height, min-height, max-height
, in § 3.2
min-content block size
, in § 2.1
min-content block-size contribution
, in § 2.2
min-content constraint
, in § 2.3
min-content contribution
, in § 2.2
min-content inline size
, in § 2.1
min-content inline-size contribution
, in § 2.2
min-content size
, in § 2.1
min height
, in § 3.1.2
min-height
, in § 3.1.2
minimum block size
, in § 3.1.2
minimum height
, in § 3.1.2
minimum inline size
, in § 3.1.2
minimum size
, in § 3.1.2
minimum width
, in § 3.1.2
min inline size
, in § 3.1.2
min-inline-size
, in § 3.1.2
min size
, in § 3.1.2
min size property
, in § 3.1.1
min width
, in § 3.1.2
min-width
, in § 3.1.2
none
, in § 3.2
outer block size
, in § 2
outer height
, in § 2
outer inline size
, in § 2
outer size
, in § 2
outer width
, in § 2
preferred aspect ratio
, in § 2.3
preferred block size
, in § 3.1.1
preferred height
, in § 3.1.1
preferred inline size
, in § 3.1.1
preferred size
, in § 3.1.1
preferred size property
, in § 3.1
preferred width
, in § 3.1.1
size
, in § 2
sizing property
, in § 3.1
stretch
value for column-width
, in § 3.4
value for width, min-width, max-width, height, min-height, max-height
, in § 3.2
stretch fit
, in § 2
stretch-fit block size
, in § 2.1
stretch-fit inline size
, in § 2.1
stretch-fit size
, in § 2.1
width
(property)
, in § 3.1.1
definition of
, in § 3.1.1
Terms defined by reference
[CSS-ALIGN-3]
defines the following terms:
self-alignment
stretch
[CSS-BORDERS-4]
defines the following terms:
border
[CSS-BOX-4]
defines the following terms:
border
border box
content box
margin
margin box
margin properties
padding
padding properties
[CSS-CASCADE-5]
defines the following terms:
computed value
initial value
used value
[CSS-CONTAIN-2]
defines the following terms:
auto
size containment
[CSS-DISPLAY-3]
defines the following terms:
box
display
[CSS-DISPLAY-4]
defines the following terms:
block box
block container
block-level box
containing block
display type
formatting context
in-flow
independent formatting context
initial containing block
inline
inline box
non-replaced
principal box
replaced
replaced element
text sequence
[CSS-FLEXBOX-1]
defines the following terms:
cross size
flex item
flex layout
flex-basis
main axis
multi-line flex container
[CSS-GAPS-1]
defines the following terms:
gutter
[CSS-GRID-2]
defines the following terms:
grid item
[CSS-IMAGES-3]
defines the following terms:
natural aspect ratio
natural dimension
natural size
[CSS-INLINE-3]
defines the following terms:
line box
[CSS-MULTICOL-2]
defines the following terms:
column-width
[CSS-SIZING-4]
defines the following terms:
aspect-ratio
auto
(for aspect-ratio)
auto
(for contain-intrinsic-width)
contain-intrinsic-size
fit-content()
size
[CSS-TEXT-4]
defines the following terms:
soft wrap opportunity
[CSS-VALUES-4]
defines the following terms:
<length-percentage>
<length>
<percentage>
contain a percentage
CSS-wide keywords
|
[CSS-WRITING-MODES-4]
defines the following terms:
block axis
block-axis
block-end
block-start
flow-relative
inline axis
physical
writing mode
[CSSOM]
defines the following terms:
getComputedStyle(elt)
resolved value
[HTML]
defines the following terms:
button
iframe
input
marquee
meter
progress
raw value
select
textarea
type
References
Normative References
[CSS-ALIGN-3]
Elika Etemad; Tab Atkins Jr..
CSS Box Alignment Module Level 3
. 30 January 2026. WD. URL:
https://www.w3.org/TR/css-align-3/
[CSS-BORDERS-4]
Elika Etemad; et al.
CSS Borders and Box Decorations Module Level 4
. 16 December 2025. WD. URL:
https://www.w3.org/TR/css-borders-4/
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
Tab Atkins Jr.; Elika Etemad.
CSS Display Module Level 3
. 5 June 2026. CRD. URL:
https://www.w3.org/TR/css-display-3/
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
CSS Gaps Module Level 1
. 24 June 2026. WD. URL:
https://www.w3.org/TR/css-gaps-1/
[CSS-IMAGES-3]
Tab Atkins Jr.; Elika Etemad; Lea Verou.
CSS Images Module Level 3
. 18 December 2023. CRD. URL:
https://www.w3.org/TR/css-images-3/
[CSS-INLINE-3]
Elika Etemad.
CSS Inline Layout Module Level 3
. 18 December 2024. WD. URL:
https://www.w3.org/TR/css-inline-3/
[CSS-MULTICOL-2]
Florian Rivoal; Rachel Andrew.
CSS Multi-column Layout Module Level 2
. 19 December 2024. FPWD. URL:
https://www.w3.org/TR/css-multicol-2/
[CSS-SIZING-4]
Tab Atkins Jr.; Elika Etemad; Jen Simmons.
CSS Box Sizing Module Level 4
. 20 May 2021. WD. URL:
https://www.w3.org/TR/css-sizing-4/
[CSS-TEXT-4]
Elika Etemad; et al.
CSS Text Module Level 4
. 14 August 2026. WD. URL:
https://www.w3.org/TR/css-text-4/
[CSS-UI-3]
Tantek Çelik; Florian Rivoal.
CSS Basic User Interface Module Level 3 (CSS3 UI)
. 7 April 2026. REC. URL:
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
[CSS3COL]
Florian Rivoal; Rachel Andrew.
CSS Multi-column Layout Module Level 1
. 16 May 2024. CR. URL:
https://www.w3.org/TR/css-multicol-1/
[CSSOM]
Daniel Glazman; Emilio Cobos Álvarez.
CSS Object Model (CSSOM)
. 26 August 2021. WD. URL:
https://www.w3.org/TR/cssom-1/
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
Non-Normative References
[CSS-GRID-2]
Tab Atkins Jr.; et al.
CSS Grid Layout Module Level 2
. 26 March 2025. CRD. URL:
https://www.w3.org/TR/css-grid-2/
[CSS-LOGICAL-1]
Elika Etemad; Rossen Atanassov.
CSS Logical Properties and Values Module Level 1
. 4 December 2025. WD. URL:
https://www.w3.org/TR/css-logical-1/
[DOM]
Anne van Kesteren.
DOM Standard
. Living Standard. URL:
https://dom.spec.whatwg.org/
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
block-size
auto | <box-size>
auto
all elements except non-replaced inlines
no
relative to width/height of containing block
by computed value type, recursing into fit-content()
per grammar
as specified, with <length-percentage> values computed
size
box-sizing
content-box | border-box
content-box
all elements that accept width or height
no
N/A
discrete
per grammar
specified keyword
height
auto | <box-size>
auto
all elements except non-replaced inlines
no
relative to width/height of containing block
by computed value type, recursing into fit-content()
per grammar
as specified, with <length-percentage> values computed
size
inline-size
auto | <box-size>
auto
all elements except non-replaced inlines
no
relative to width/height of containing block
by computed value type, recursing into fit-content()
per grammar
as specified, with <length-percentage> values computed
size
max-block-size
none | <box-size>
none
all elements that accept width or height
no
relative to width/height of containing block
by computed value, recursing into fit-content()
per grammar
as specified, with <length-percentage> values computed
max-size
max-height
none | <box-size>
none
all elements that accept width or height
no
relative to width/height of containing block
by computed value, recursing into fit-content()
per grammar
as specified, with <length-percentage> values computed
max-size
max-inline-size
none | <box-size>
none
all elements that accept width or height
no
relative to width/height of containing block
by computed value, recursing into fit-content()
per grammar
as specified, with <length-percentage> values computed
max-size
max-width
none | <box-size>
none
all elements that accept width or height
no
relative to width/height of containing block
by computed value, recursing into fit-content()
per grammar
as specified, with <length-percentage> values computed
max-size
min-block-size
auto | <box-size>
auto
all elements that accept width or height
no
relative to width/height of containing block
by computed value, recursing into fit-content()
per grammar
as specified, with <length-percentage> values computed
min-size
min-height
auto | <box-size>
auto
all elements that accept width or height
no
relative to width/height of containing block
by computed value, recursing into fit-content()
per grammar
as specified, with <length-percentage> values computed
min-size
min-inline-size
auto | <box-size>
auto
all elements that accept width or height
no
relative to width/height of containing block
by computed value, recursing into fit-content()
per grammar
as specified, with <length-percentage> values computed
min-size
min-width
auto | <box-size>
auto
all elements that accept width or height
no
relative to width/height of containing block
by computed value, recursing into fit-content()
per grammar
as specified, with <length-percentage> values computed
min-size
width
auto | <box-size>
auto
all elements except non-replaced inlines
no
relative to width/height of containing block
by computed value type, recursing into fit-content()
per grammar
as specified, with <length-percentage> values computed
size
Issues Index
This spec needs illustrations! See
issue
.
↵
Replace this section with references to the new term
automatic size
.
↵
Should we resolve transferred percentages against their containing block instead of zero before transferring them? See
discussion
.
↵
Letting percentages still resolve against a definite
height
when the min-height is intrinsic is an open issue. (CSS2 has a general statement about "height depending on contents", which this technically is, even though CSS2 didn’t have content-dependent keywords for
min-height
. Since this is new, we think we could have this different behavior.)
↵
Tracking web-compat & implementation progress of applying this to max-width/height in
Issue 6348
.
[Issue #6348]
↵
```
