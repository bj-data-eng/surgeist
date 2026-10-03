Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Logical Properties and Values Module Level 1](https://www.w3.org/TR/2025/WD-css-logical-1-20251204/).

Original copyright notice (from the matching exact HTML edition): Copyright © 2025 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Logical Properties and Values Module Level 1

Source snapshot: https://www.w3.org/TR/2025/WD-css-logical-1-20251204/

Snapshot SHA-256: a387e513984bccc9df98d906ccdc48ccec3873ed6ac2a5a66b0751f985fd6dc3

Conversion: offline format conversion of the exact stored extracted-text; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- This stored input is an already extracted plain-text witness. It has no recoverable original HTML structure. The complete source text is preserved in a fenced block; no summary or specification regeneration was performed.

---

```text
CSS Logical Properties and Values Module Level 1
CSS Logical Properties and Values Module Level 1
W3C Working Draft
,
4 December 2025
More details about this document
This version:
https://www.w3.org/TR/2025/WD-css-logical-1-20251204/
Latest published version:
https://www.w3.org/TR/css-logical-1/
Editor's Draft:
https://drafts.csswg.org/css-logical-1/
Previous Versions:
https://www.w3.org/TR/2018/WD-css-logical-1-20180827/
History:
https://www.w3.org/standards/history/css-logical-1/
Feedback:
CSSWG Issues Repository
Inline In Spec
Editors:
Rossen Atanassov
(
Microsoft
)
Elika J. Etemad / fantasai
(
Apple
)
Suggest an Edit for this Spec:
GitHub Editor
Test Suite:
https://wpt.fyi/results/css/css-logical/
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
This module introduces logical properties and values that provide the author with the ability to control layout through logical, rather than physical, direction and dimension mappings. The module defines logical properties and values for the features defined in
[CSS2]
. These properties are writing-mode relative equivalents of their corresponding physical properties.
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
(preferred), including the spec code “css-logical” in the title, like this: “[css-logical]
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
Value Definitions
2
Flow-Relative Values:
block-start
,
block-end
,
inline-start
,
inline-end
2.1
Logical Values for the
caption-side
Property
2.2
Flow-Relative Values for the
float
and
clear
Properties
2.3
Flow-Relative Values for the
text-align
Property
3
Flow-Relative Page Classifications
4
Flow-Relative Box Model Properties
4.1
Logical Height and Logical Width: the
block-size
/
inline-size
,
min-block-size
/
min-inline-size
, and
max-block-size
/
max-inline-size
properties
4.2
Flow-Relative Margins: the
margin-block-start
,
margin-block-end
,
margin-inline-start
,
margin-inline-end
properties and
margin-block
and
margin-inline
shorthands
4.3
Flow-Relative Offsets: the
inset-block-start
,
inset-block-end
,
inset-inline-start
,
inset-inline-end
properties and
inset-block
,
inset-inline
, and
inset
shorthands
4.4
Flow-Relative Padding: the
padding-block-start
,
padding-block-end
,
padding-inline-start
,
padding-inline-end
properties and
padding-block
and
padding-inline
shorthands
4.5
Flow-Relative Borders
4.5.1
Flow-Relative Border Widths: the
border-block-start-width
,
border-block-end-width
,
border-inline-start-width
,
border-inline-end-width
properties and
border-block-width
and
border-inline-width
shorthands
4.5.2
Flow-Relative Border Styles: the
border-block-start-style
,
border-block-end-style
,
border-inline-start-style
,
border-inline-end-style
properties and
border-block-style
and
border-inline-style
shorthands
4.5.3
Flow-Relative Border Colors: the
border-block-start-color
,
border-block-end-color
,
border-inline-start-color
,
border-inline-end-color
properties and
border-block-color
and
border-inline-color
shorthands
4.5.4
Flow-Relative Border Shorthands: the
border-block-start
,
border-block-end
,
border-inline-start
,
border-inline-end
properties and
border-block
and
border-inline
shorthands
4.6
Flow-Relative Corner Rounding: the
border-start-start-radius
,
border-start-end-radius
,
border-end-start-radius
,
border-end-end-radius
properties
4.7
Four-Directional Shorthand Properties: the
margin
,
padding
,
border-width
,
border-style
, and
border-color
shorthands
5
Acknowledgements
6
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
Property Index
Issues Index
1.
Introduction
Note:
See
[css-writing-modes-4]
for a proper introduction to writing modes; this module assumes familiarity with its terminology.
Because different writing systems are written in different directions, a variety of
writing modes
exist: left to right, top to bottom; right to left, top to bottom; bottom to top, right to left; etc. logical concepts like the “start” of a page or line map differently to physical concepts like the “top” of a line or “left edge” of a paragraph. Some aspects of a layout are actually relative to the writing directions, and thus will vary when the page is translated to a different system; others are inherently relative to the page’s physical orientation.
For example, lists, headings, and paragraphs are typically left-aligned in English; but actually they are start-aligned, because in Arabic the same constructs are right-aligned, and a multilingual document will need to accommodate both writing systems accordingly.
The following code exemplifies how using logical syntax can help you write code that works across different writing systems:
Rendering of the below code in a compatible browser
<
blockquote
dir
=
"auto"
>
Quotation in English
</
blockquote
>
<
blockquote
dir
=
"auto"
>
اقتباس في العربية
</
blockquote
>
blockquote
{
text-align
:
start
;
/* left in latin, right in arabic */
margin-inline-start:
0
px
;
/* margin-left in latin, margin-right in arabic */
border-inline-start:
5
px
solid gray
;
/* border-left in latin, border-right in arabic */
padding-inline-start:
5
px
;
/* padding-left in latin, padding-right in arabic */
}
Documents might need both logical and physical properties. For instance the drop shadows on buttons on a page must remain consistent throughout, so their offset will be chosen based on visual considerations and physical directions, and not vary by writing system.
Since CSS was originally designed with only physical coordinates in its controls, this module introduces text-flow–relative equivalents so that declarations in a CSS style sheet can be expressed in
flow-relative
terms. It defines the mapping and cascading of equivalent properties, some new properties and values equivalent to those in CSS2.1, and the principles used to derive their syntaxes. Future CSS specifications are expected to incorporate both sets of coordinates in their property and value definitions, so this module will not track the introduction of
flow-relative
variants of newer CSS features.
CSS Writing Modes
’
Abstract Box Terminology
section defines how to map between flow-relative and physical terms. This mapping, which depends on the
used values
of
writing-mode
,
direction
, and
text-orientation
, controls the interpretation of
flow-relative
keywords and properties.
Correspondence of physical and flow-relative terms in typical English text layout
Correspondence of physical and flow-relative terms in vertical Chinese text layout
Note:
Due to its interaction with
text-orientation: upright
, the
used
direction
depends on the
computed values
of
writing-mode
and
text-orientation
.
Tests
General tests for logical properties
getComputedStyle-listing.html
(live test)
(source)
inheritance.html
(live test)
(source)
logicalprops-quirklength.html
(live test)
(source)
Things That Are Unstable
Since implementation of parts of this module is effectively required for shipping an implementation of
CSS Writing Modes
on the Web (in order to correctly implement the default HTML styles), the CSSWG resolved that the requisite features in
§ 2 Flow-Relative Values: block-start, block-end, inline-start, inline-end
and
§ 4 Flow-Relative Box Model Properties
are approved for shipping. (See
FPWD announcement
for additional background.)
However, there are a few significant open issues:
The
logical
keyword on shorthands, because the name of the keyword may change or it may be replaced by some other syntactic marker. (This feature will be deferred from this level for further development if there is no clearly satisfactory mechanism proposed, see
Issue 1282
.)
Whether flow-relative longhands inherit from their namesake on the parent, or are mapped to a physical property and inherit from that property. (See
Issue 3029
.)
Whether shorthands like
margin
expand to both sets of longhands, or only the ones that were set. (See
Issue 3030
.)
Comments, suggestions, and use cases are welcome on these issues. Please file them in GitHub, tweet them to @csswg, or send them to www-style@w3.org.
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
2.
Flow-Relative Values:
block-start
,
block-end
,
inline-start
,
inline-end
Many CSS properties have historically accepted
directional keyword
values that are
physical
(
top
,
bottom
,
left
,
right
). This specification introduces
directional keyword
values that are
flow-relative
:
block-start
,
block-end
,
inline-start
,
inline-end
.
A property’s effect can be either 1-dimensional or 2-dimensional. When contextually constrained to one dimension, the flow-relative keywords are abbreviated (to
start
and
end
).
CSS Level 2 properties are here redefined to also accept
flow-relative
directional keywords. Such values can be used in place of the corresponding physical values. For properties that take multiple keywords, combinations of flow-relative and physical values are not allowed (unless otherwise specified in a future specification).
Note:
Newer CSS specifications are expected in most cases to define
flow-relative
or
line-relative
values instead of or in addition to any
physical
ones. In general, the mapping of such relative values are expected to use the
writing mode
of the
containing block
when affecting the box itself, and that of the box itself when affecting its contents. Regardless, which
writing modes
is used for the mapping needs to be explicitly defined.
2.1.
Logical Values for the
caption-side
Property
Name:
caption-side
New values:
inline-start
|
inline-end
Computed value:
specified keyword
Tests
caption-side-no-interpolation.html
(live test)
(source)
These two values are added only for implementations that support
left
and
right
values for
caption-side
. The
left
and
right
values themselves are defined to be
line-relative
.
The existing
top
and
bottom
values are idiosyncratically redefined as assigning to the
block-start
and
block-end
sides of the table, respectively.
The mapping on this property uses the
writing mode
of the caption’s
containing block
(that is, the
table wrapper box
).
2.2.
Flow-Relative Values for the
float
and
clear
Properties
Name:
float
,
clear
New values:
inline-start
|
inline-end
Computed value:
specified keyword
Tests
float-interpolation.html
(live test)
(source)
logical-values-float-clear-1.html
(live test)
(source)
logical-values-float-clear-2.html
(live test)
(source)
logical-values-float-clear-3.html
(live test)
(source)
logical-values-float-clear-4.html
(live test)
(source)
logical-values-float-clear-reftest.html
(live test)
(source)
logical-values-float-clear.html
(live test)
(source)
The mapping on these properties uses the
writing mode
of the element’s
containing block
.
Note:
These properties are 1-dimensional in CSS2, but are planned to be expanded to two dimensions, and therefore are given unabbreviated
flow-relative
keywords.
2.3.
Flow-Relative Values for the
text-align
Property
Name:
text-align
New values:
start
|
end
Computed value:
specified keyword
These values are normatively defined in
[css-text-3]
.
3.
Flow-Relative Page Classifications
In CSS, all pages are classified by user agents as either left pages or right pages.
[CSS2]
Which page is first in a spread, however, depends on whether the page progression is left-to-right or right-to-left.
To allow control of page breaking to the page that is on the earlier or later side of a spread, rather than to the left or right side of a spread, this module introduces the following additional keywords for the
page-break-after
and
page-break-before
properties
[CSS2]
:
recto
Equivalent to
right
in left-to-right page progressions and
left
in right-to-left page progressions.
verso
Equivalent to
left
in left-to-right page progressions and
right
in right-to-left page progressions.
These values are computed as specified and are further defined in
[css-break-3]
.
Although authors typically place page numbers using physical placements, the contents of headers often follows conventions depending on which page in the spread is earlier. Therefore the following flow-relative
page selectors
are also added to support flow-relative page selection:
:recto
Equivalent to ':right' in left-to-right page progressions and ':left' in right-to-left page progressions.
:verso
Equivalent to ':left' in left-to-right page progressions and ':right' in right-to-left page progressions.
The flow-relative page selectors have specificity equal to the ':left' and ':right' page selectors.
4.
Flow-Relative Box Model Properties
For many formatting effects, the axis or direction affected is encoded in the property name rather than in its value. The type of directional or axis mapping (
flow-relative
or
physical
) of each such property is called its
mapping logic
. Historically, all properties have been encoded in
physical
terms; this specification introduces new CSS properties that are
flow-relative
equivalents of CSS2’s
physical
box model properties.
Note:
Newer CSS specifications are expected in most cases to define
flow-relative
or
line-relative
properties instead of or in addition to any
physical
ones.
Each set of parallel
flow-relative
properties and
physical
properties (ignoring
shorthand
properties) related by setting equivalent styles on the various sides or dimensions of a box, forms a
logical property group
. For example, the
padding-*
properties form a single
logical property group
, the
margin-*
properties form a separate
logical property group
, the
border-*-style
properties form another
logical property group
, etc. (Each
longhand property
can belong to at most one
logical property group
.)
Within each
logical property group
, corresponding
flow-relative
and
physical
properties are paired using the element’s own
computed
writing mode
. Although the
specified value
of each property remains distinct, paired properties share a
computed value
. This shared value is determined by
cascading
the declarations of both properties together as one; in other words, the
computed value
of both properties in the pair is derived from the
specified value
of the property declared with higher priority in the CSS
cascade
.
[CSS-CASCADE-3]
Note that this requires implementations to maintain relative order of declarations within a CSS declaration block, which was not previously required for CSS cascading. It also requires that
writing-mode
,
direction
, and
text-orientation
be computed as a prerequisite for cascading together the
flow-relative
and
physical
declarations of a
logical property group
to find their
computed values
.
For example, given the following rule:
p
{
margin-inline-start
:
1
px
;
margin-left
:
2
px
;
margin-inline-end
:
3
px
;
}
In a paragraph with computed
writing-mode
being
horizontal-tb
and computed
direction
being
ltr
, the computed value of
margin-left
is
2px
, since for that
writing-mode
and
direction
,
margin-inline-start
and
margin-left
share a computed value, and the declaration of
margin-left
is after the declaration of
margin-inline-start
. However, if the computed
direction
were instead
rtl
, the computed value of
margin-left
is
3px
, since
margin-inline-end
and
margin-left
share a computed value, and the declaration of
margin-inline-end
is after the declaration of
margin-left
.
[CSSOM]
APIs that return computed values (such as
getComputedStyle()
) must return the same value for each individual property in such a pair.
Note:
Depending on the element’s own
writing mode
for mapping every
flow-relative
property to its
physical
equivalent simplifies the cascading calculations and gives a straightforward model for authors to reason about. However, it is problematic in many cases, see for example
this discussion
. Authors may need to use nested elements to get the correct mapping behavior when changing an element’s
writing mode
from its parent.
Inheritance of each property is from its corresponding property on the parent. For example, although the
inline-start
margin of an
rtl
box is its right margin,
margin-inline-start
on this box will inherit the
margin-inline-start
of an
ltr
parent even though that happens to be the parent’s
left
margin.
Unless otherwise specified,
shorthand properties
that encompass both logical and physical longhands (such as the
all
shorthand) set their physical longhands last. For example,
all: inherit
will set all of the
margin
properties to
inherit
, but since the physical longhands are set last, the child’s margins will inherit from their physical counterparts in the parent.
Tests
animation-001.html
(live test)
(source)
animation-002.html
(live test)
(source)
animation-004.html
(live test)
(source)
logicalprops-with-deferred-writing-mode.html
(live test)
(source)
logicalprops-with-variables.html
(live test)
(source)
4.1.
Logical Height and Logical Width: the
block-size
/
inline-size
,
min-block-size
/
min-inline-size
, and
max-block-size
/
max-inline-size
properties
Name:
block-size
,
inline-size
Value:
<'width'>
Initial:
auto
Applies to:
Same as
height
and
width
Inherited:
no
Percentages:
As for the corresponding physical property
Computed value:
Same as
height
,
width
Canonical order:
per grammar
Animation type:
by computed value type
Logical property group:
size
Tests
cascading-001.html
(live test)
(source)
logical-box-size.html
(live test)
(source)
logicalprops-block-size-vlr.html
(live test)
(source)
logicalprops-block-size.html
(live test)
(source)
logicalprops-inline-size-vlr.html
(live test)
(source)
logicalprops-inline-size.html
(live test)
(source)
block-size-computed.html
(live test)
(source)
block-size-invalid.html
(live test)
(source)
block-size-valid.html
(live test)
(source)
inline-size-computed.html
(live test)
(source)
inline-size-invalid.html
(live test)
(source)
inline-size-valid.html
(live test)
(source)
These properties correspond to the
height
and
width
properties. The mapping depends on the element’s
writing-mode
.
Name:
min-block-size
,
min-inline-size
Value:
<'min-width'>
Initial:
0
Applies to:
same as
height
and
width
Inherited:
no
Percentages:
As for the corresponding physical property
Computed value:
Same as
min-height
,
min-width
Canonical order:
per grammar
Animation type:
by computed value type
Logical property group:
min-size
Tests
min-block-size-computed.html
(live test)
(source)
min-block-size-invalid.html
(live test)
(source)
min-block-size-valid.html
(live test)
(source)
min-inline-size-computed.html
(live test)
(source)
min-inline-size-invalid.html
(live test)
(source)
min-inline-size-valid.html
(live test)
(source)
These properties correspond to the
min-height
and
min-width
properties. The mapping depends on the element’s
writing-mode
.
Name:
max-block-size
,
max-inline-size
Value:
<'max-width'>
Initial:
none
Applies to:
same as
height
and
width
Inherited:
no
Percentages:
As for the corresponding physical property
Computed value:
Same as
max-height
,
max-width
Canonical order:
per grammar
Animation type:
by computed value type
Logical property group:
max-size
Tests
max-block-size-computed.html
(live test)
(source)
max-block-size-invalid.html
(live test)
(source)
max-block-size-valid.html
(live test)
(source)
max-inline-size-computed.html
(live test)
(source)
max-inline-size-invalid.html
(live test)
(source)
max-inline-size-valid.html
(live test)
(source)
These properties correspond to the
max-height
and
max-width
properties. The mapping depends on the element’s
writing-mode
.
4.2.
Flow-Relative Margins: the
margin-block-start
,
margin-block-end
,
margin-inline-start
,
margin-inline-end
properties and
margin-block
and
margin-inline
shorthands
Name:
margin-block-start
,
margin-block-end
,
margin-inline-start
,
margin-inline-end
Value:
<'margin-top'>
Initial:
0
Applies to:
Same as
margin-top
Inherited:
no
Percentages:
As for the corresponding physical property
Computed value:
Same as corresponding
margin-*
properties
Canonical order:
per grammar
Animation type:
by computed value type
Logical property group:
margin
Tests
logical-box-margin.html
(live test)
(source)
margin-block-inline-computed.html
(live test)
(source)
margin-block-inline-invalid.html
(live test)
(source)
margin-block-inline-shorthand.html
(live test)
(source)
margin-block-inline-valid.html
(live test)
(source)
These properties correspond to the
margin-top
,
margin-bottom
,
margin-left
, and
margin-right
properties. The mapping depends on the element’s
writing-mode
,
direction
, and
text-orientation
.
Name:
margin-block
,
margin-inline
Value:
<'margin-top'>
{1,2}
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
Tests
margin-block-interpolation.html
(live test)
(source)
margin-inline-interpolation.html
(live test)
(source)
These two
shorthand properties
set the
margin-block-start
&
margin-block-end
and
margin-inline-start
&
margin-inline-end
, respectively. The first value represents the
start
edge style, and the second value represents the
end
edge style. If only one value is given, it applies to both the
start
and
end
edges.
4.3.
Flow-Relative Offsets: the
inset-block-start
,
inset-block-end
,
inset-inline-start
,
inset-inline-end
properties and
inset-block
,
inset-inline
, and
inset
shorthands
The
top
,
left
,
bottom
,
right
physical properties, their
inset-block-start
,
inset-block-end
,
inset-inline-start
,
inset-inline-end
flow-relative correspondents, and the
inset-block
,
inset-inline
, and
inset
shorthands, are collectively known as the
inset properties
.
Name:
inset-block-start
,
inset-block-end
,
inset-inline-start
,
inset-inline-end
Value:
<'top'>
Initial:
auto
Applies to:
positioned elements
Inherited:
no
Percentages:
As for the corresponding physical property
Computed value:
Same as corresponding
top
/
right
/
bottom
/
left
properties
Canonical order:
per grammar
Animation type:
by computed value type
Logical property group:
inset
Tests
logical-box-inset.html
(live test)
(source)
inset-block-inline-computed.html
(live test)
(source)
inset-block-inline-invalid.html
(live test)
(source)
inset-block-inline-shorthand.html
(live test)
(source)
inset-block-inline-valid.html
(live test)
(source)
These properties correspond to the
top
,
bottom
,
left
, and
right
properties. The mapping depends on the element’s
writing-mode
,
direction
, and
text-orientation
.
Name:
inset-block
,
inset-inline
Value:
<'top'>
{1,2}
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
These two
shorthand properties
set the
inset-block-start
&
inset-block-end
and
inset-inline-start
&
inset-inline-end
, respectively. The first value represents the
start
edge style, and the second value represents the
end
edge style. If only one value is given, it applies to both the
start
and
end
edges.
Name:
inset
Value:
<'top'>
{1,4}
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
Tests
inset-computed.html
(live test)
(source)
inset-invalid.html
(live test)
(source)
inset-shorthand.html
(live test)
(source)
inset-valid.html
(live test)
(source)
This
shorthand property
sets the
top
,
right
,
bottom
, and
left
properties. Values are assigned to its
sub-properties
as for
margin
.
4.4.
Flow-Relative Padding: the
padding-block-start
,
padding-block-end
,
padding-inline-start
,
padding-inline-end
properties and
padding-block
and
padding-inline
shorthands
Name:
padding-block-start
,
padding-block-end
,
padding-inline-start
,
padding-inline-end
Value:
<'padding-top'>
Initial:
0
Applies to:
Same as
padding-top
Inherited:
no
Percentages:
As for the corresponding physical property
Computed value:
Same as corresponding
padding-*
properties
Canonical order:
per grammar
Animation type:
by computed value type
Logical property group:
padding
Tests
logical-box-padding.html
(live test)
(source)
padding-block-inline-computed.html
(live test)
(source)
padding-block-inline-invalid.html
(live test)
(source)
padding-block-inline-shorthand.html
(live test)
(source)
padding-block-inline-valid.html
(live test)
(source)
These properties correspond to the
padding-top
,
padding-bottom
,
padding-left
, and
padding-right
properties. The mapping depends on the element’s
writing-mode
,
direction
, and
text-orientation
.
Name:
padding-block
,
padding-inline
Value:
<'padding-top'>
{1,2}
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
Tests
padding-block-interpolation.html
(live test)
(source)
padding-inline-interpolation.html
(live test)
(source)
These two
shorthand properties
set the
padding-block-start
&
padding-block-end
and
padding-inline-start
&
padding-inline-end
, respectively. The first value represents the
start
edge style, and the second value represents the
end
edge style. If only one value is given, it applies to both the
start
and
end
edges.
4.5.
Flow-Relative Borders
4.5.1.
Flow-Relative Border Widths: the
border-block-start-width
,
border-block-end-width
,
border-inline-start-width
,
border-inline-end-width
properties and
border-block-width
and
border-inline-width
shorthands
Name:
border-block-start-width
,
border-block-end-width
,
border-inline-start-width
,
border-inline-end-width
Value:
<'border-top-width'>
Initial:
medium
Applies to:
Same as
border-top-width
Inherited:
no
Percentages:
n/a
Computed value:
Same as corresponding
border-*-width
properties
Canonical order:
per grammar
Animation type:
by computed value type
Logical property group:
border-width
Tests
logical-box-border-width.html
(live test)
(source)
These properties correspond to the
border-top-width
,
border-bottom-width
,
border-left-width
, and
border-right-width
properties. The mapping depends on the element’s
writing-mode
,
direction
, and
text-orientation
.
Name:
border-block-width
,
border-inline-width
Value:
<'border-top-width'>
{1,2}
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
Tests
border-block-width-computed.html
(live test)
(source)
border-block-width-invalid.html
(live test)
(source)
border-block-width-valid.html
(live test)
(source)
border-inline-width-computed.html
(live test)
(source)
border-inline-width-invalid.html
(live test)
(source)
border-inline-width-valid.html
(live test)
(source)
These two
shorthand properties
set the
border-block-start-width
&
border-block-end-width
and
border-inline-start-width
&
border-inline-end-width
, respectively. The first value represents the
start
edge width, and the second value represents the
end
edge width. If only one value is given, it applies to both the
start
and
end
edges.
4.5.2.
Flow-Relative Border Styles: the
border-block-start-style
,
border-block-end-style
,
border-inline-start-style
,
border-inline-end-style
properties and
border-block-style
and
border-inline-style
shorthands
Name:
border-block-start-style
,
border-block-end-style
,
border-inline-start-style
,
border-inline-end-style
Value:
<'border-top-style'>
Initial:
none
Applies to:
Same as
border-top-style
Inherited:
no
Percentages:
n/a
Computed value:
Same as corresponding
border-*-style
properties
Canonical order:
per grammar
Animation type:
discrete
Logical property group:
border-style
Tests
logical-box-border-style.html
(live test)
(source)
border-block-style-computed.html
(live test)
(source)
border-block-style-invalid.html
(live test)
(source)
border-block-style-valid.html
(live test)
(source)
border-inline-style-computed.html
(live test)
(source)
border-inline-style-invalid.html
(live test)
(source)
border-inline-style-valid.html
(live test)
(source)
These properties correspond to the
border-top-style
,
border-bottom-style
,
border-left-style
, and
border-right-style
properties. The mapping depends on the element’s
writing-mode
,
direction
, and
text-orientation
.
Name:
border-block-style
,
border-inline-style
Value:
<'border-top-style'>
{1,2}
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
These two
shorthand properties
set the
border-block-start-style
&
border-block-end-style
and
border-inline-start-style
&
border-inline-end-style
, respectively. The first value represents the
start
edge style, and the second value represents the
end
edge style. If only one value is given, it applies to both the
start
and
end
edges.
4.5.3.
Flow-Relative Border Colors: the
border-block-start-color
,
border-block-end-color
,
border-inline-start-color
,
border-inline-end-color
properties and
border-block-color
and
border-inline-color
shorthands
Name:
border-block-start-color
,
border-block-end-color
,
border-inline-start-color
,
border-inline-end-color
Value:
<'border-top-color'>
Initial:
currentcolor
Applies to:
Same as
border-top-color
Inherited:
no
Percentages:
n/a
Computed value:
Same as corresponding
border-*-color
properties
Canonical order:
per grammar
Animation type:
by computed value type
Logical property group:
border-color
Tests
logical-box-border-color.html
(live test)
(source)
border-block-color-computed.html
(live test)
(source)
border-block-color-invalid.html
(live test)
(source)
border-block-color-valid.html
(live test)
(source)
border-inline-color-computed.html
(live test)
(source)
border-inline-color-invalid.html
(live test)
(source)
border-inline-color-valid.html
(live test)
(source)
These properties correspond to the
border-top-color
,
border-bottom-color
,
border-left-color
, and
border-right-color
properties. The mapping depends on the element’s
writing-mode
,
direction
, and
text-orientation
.
Name:
border-block-color
,
border-inline-color
Value:
<'border-top-color'>
{1,2}
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
These two
shorthand properties
set the
border-block-start-color
&
border-block-end-color
and
border-inline-start-color
&
border-inline-end-color
, respectively. The first value represents the
start
edge color, and the second value represents the
end
edge color. If only one value is given, it applies to both the
start
and
end
edges.
4.5.4.
Flow-Relative Border Shorthands: the
border-block-start
,
border-block-end
,
border-inline-start
,
border-inline-end
properties and
border-block
and
border-inline
shorthands
Name:
border-block-start
,
border-block-end
,
border-inline-start
,
border-inline-end
Value:
<'border-top-width'>
||
<'border-top-style'>
||
<color>
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
Tests
logical-box-border-shorthands.html
(live test)
(source)
border-block-valid.html
(live test)
(source)
These properties correspond to the
border-top
,
border-bottom
,
border-left
, and
border-right
properties. The mapping depends on the element’s
writing-mode
,
direction
, and
text-orientation
.
Name:
border-block
,
border-inline
Value:
<'border-block-start'>
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
Tests
border-inline-valid.html
(live test)
(source)
These two
shorthand properties
set the
border-block-start
&
border-block-end
or
border-inline-start
&
border-inline-end
, respectively, both to the same style.
4.6.
Flow-Relative Corner Rounding: the
border-start-start-radius
,
border-start-end-radius
,
border-end-start-radius
,
border-end-end-radius
properties
Name:
border-start-start-radius
,
border-start-end-radius
,
border-end-start-radius
,
border-end-end-radius
Value:
<'border-top-left-radius'>
Initial:
Same as
border-top-left-radius
Applies to:
Same as
border-top-left-radius
Inherited:
no
Percentages:
Same as
border-top-left-radius
Computed value:
Same as corresponding physical
border-*-radius
properties
Canonical order:
per grammar
Animation type:
by computed value type
Logical property group:
border-radius
Tests
logical-box-border-radius.html
(live test)
(source)
These properties correspond to the
border-top-left-radius
,
border-bottom-left-radius
,
border-top-right-radius
, and
border-bottom-right-radius
properties. The mapping depends on the element’s
writing-mode
,
direction
, and
text-orientation
, with the first start/end giving the block axis side, and the second the inline-axis side (i.e. patterned as 'border-
block
-
inline
-radius').
4.7.
Four-Directional Shorthand Properties: the
margin
,
padding
,
border-width
,
border-style
, and
border-color
shorthands
The shorthand properties for margin, padding, and border set values for physical properties by default. But authors can specify the
logical
keyword at the beginning of the property value to indicate that the values map to the flow-relative properties instead of the physical ones.
The proposed syntax for this feature is
under discussion
and is almost guaranteed to change from what is described here. This section remains in the draft to promote discussion of alternatives, to document the affected properties, and to specify the expected impact on the interpretation of whatever syntactic switch is ultimately chosen.
The following
[CSS2]
shorthand properties accept the
logical
keyword:
inset
margin
padding
border-width
border-style
border-color
scroll-padding
scroll-margin
The syntax for these properties is effectively changed by replacing
<
value-type
>
{1,4}
with
logical
?
<
value-type
>
{1,4}
When the
logical
keyword is present in the value, the values that follow are assigned to its flow-relative longhands as follows:
If only one value is set, the value applies to all four flow-relative
longhands
.
If two values are set, the first is for block-start and block-end, the second is for inline-start and inline-end.
If three values are set, the first is for block-start, the second is for inline-start and inline-end, and the third is for block-end.
If four values are set, they apply to the block-start, inline-start, block-end, and inline-end sides in that order.
In the following example, the two rules are equivalent:
blockquote
{
margin
:
logical
1
em
2
em
3
em
4
em
;
}
blockquote
{
margin-block-start
:
1
em
;
margin-inline-start
:
2
em
;
margin-block-end
:
3
em
;
margin-inline-end
:
4
em
;
}
5.
Acknowledgements
Cameron McCormack, David Baron, Oriol Brufau, Shinyu Murakami, Tab Atkins
6.
Changes
Changes since the
27 August 2018 Working Draft
include:
Defined inheritance of flow-relative properties.
Moved flow-relative values of
resize
to
[CSS-UI-4]
.
Editorially rewrote large parts of the specification for clarity.
Added
text-orientation
to list of mapping dependencies.
Clarified that
border-inline
and
border-block
set both affected sides to the same value.
Clarified the mapping for border-radius.
Referred to physical margin, border, and padding properties in the elements they apply to, so ruby base containers and ruby annotation containers are excluded.
Made margin and padding properties refer to
*-top
properties for consistency.
Defined the term
inset properties
.
Miscellaneous minor clarifications.
Added Web Platform Tests coverage.
Changes between the earlier Editors Drafts and the
18 May 2017 First Public Working Draft
include:
Making all properties cascade using the
writing mode
specified on the element, not on its parent.
Making the ordering of longhands within
margin
-like shorthands put inline-start before inline-end.
Adding the
*-inline
and
*-block
shorthand forms for margins/borders/padding.
Renaming the
offset-*
properties to
inset-*
and marking an issue for discussion.
Adding an Introduction section.
Updating to current terminology of CSS Writing Modes.
Miscellaneous prose cleanup.
Privacy Considerations
No new privacy considerations have been reported on this specification.
Security Considerations
No new security considerations have been reported on this specification.
Tests
Features from other specs
logical-values-resize.html
(live test)
(source)
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
block-size
, in § 4.1
border-block
, in § 4.5.4
border-block-color
, in § 4.5.3
border-block-end
, in § 4.5.4
border-block-end-color
, in § 4.5.3
border-block-end-style
, in § 4.5.2
border-block-end-width
, in § 4.5.1
border-block-start
, in § 4.5.4
border-block-start-color
, in § 4.5.3
border-block-start-style
, in § 4.5.2
border-block-start-width
, in § 4.5.1
border-block-style
, in § 4.5.2
border-block-width
, in § 4.5.1
border-end-end-radius
, in § 4.6
border-end-start-radius
, in § 4.6
border-inline
, in § 4.5.4
border-inline-color
, in § 4.5.3
border-inline-end
, in § 4.5.4
border-inline-end-color
, in § 4.5.3
border-inline-end-style
, in § 4.5.2
border-inline-end-width
, in § 4.5.1
border-inline-start
, in § 4.5.4
border-inline-start-color
, in § 4.5.3
border-inline-start-style
, in § 4.5.2
border-inline-start-width
, in § 4.5.1
border-inline-style
, in § 4.5.2
border-inline-width
, in § 4.5.1
border-start-end-radius
, in § 4.6
border-start-start-radius
, in § 4.6
directional keyword
, in § 2
inline-size
, in § 4.1
inset
, in § 4.3
inset-block
, in § 4.3
inset-block-end
, in § 4.3
inset-block-start
, in § 4.3
inset-inline
, in § 4.3
inset-inline-end
, in § 4.3
inset-inline-start
, in § 4.3
inset properties
, in § 4.3
logical
, in § 4.7
logical property group
, in § 4
mapping logic
, in § 4
margin-block
, in § 4.2
margin-block-end
, in § 4.2
margin-block-start
, in § 4.2
margin-inline
, in § 4.2
margin-inline-end
, in § 4.2
margin-inline-start
, in § 4.2
max-block-size
, in § 4.1
max-inline-size
, in § 4.1
min-block-size
, in § 4.1
min-inline-size
, in § 4.1
padding-block
, in § 4.4
padding-block-end
, in § 4.4
padding-block-start
, in § 4.4
padding-inline
, in § 4.4
padding-inline-end
, in § 4.4
padding-inline-start
, in § 4.4
:recto
, in § 3
recto
, in § 3
:verso
, in § 3
verso
, in § 3
Terms defined by reference
[CSS-BACKGROUNDS-3]
defines the following terms:
border-bottom
border-bottom-color
border-bottom-left-radius
border-bottom-right-radius
border-bottom-style
border-bottom-width
border-color
border-left
border-left-color
border-left-style
border-left-width
border-radius
border-right
border-right-color
border-right-style
border-right-width
border-style
border-top
border-top-color
border-top-left-radius
border-top-right-radius
border-top-style
border-top-width
border-width
[CSS-BOX-4]
defines the following terms:
margin
margin-bottom
margin-left
margin-right
margin-top
padding
padding-bottom
padding-left
padding-right
padding-top
[CSS-CASCADE-5]
defines the following terms:
all
computed value
inherit
longhand
longhand property
shorthand
shorthand property
specified value
sub-property
used value
[CSS-CASCADE-6]
defines the following terms:
cascade
[CSS-COLOR-5]
defines the following terms:
<color>
[CSS-DISPLAY-4]
defines the following terms:
containing block
[CSS-POSITION-3]
defines the following terms:
bottom
left
right
top
[CSS-SCROLL-SNAP-1]
defines the following terms:
scroll-margin
scroll-padding
[CSS-SIZING-3]
defines the following terms:
height
width
[CSS-TABLES-3]
defines the following terms:
table wrapper box
[CSS-TEXT-3]
defines the following terms:
text-align
[CSS-UI-4]
defines the following terms:
resize
[CSS-VALUES-4]
defines the following terms:
?
CSS-wide keywords
{A,B}
|
||
[CSS-WRITING-MODES-3]
defines the following terms:
direction
end
start
[CSS-WRITING-MODES-4]
defines the following terms:
block-end
block-start
flow-relative
horizontal-tb
inline-start
line-relative
ltr
physical
rtl
text-orientation
writing mode
writing-mode
[CSS2]
defines the following terms:
caption-side
clear
float
max-height
max-width
min-height
min-width
page-break-after
page-break-before
References
Normative References
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
[CSS-BREAK-3]
Rossen Atanassov; Elika Etemad.
CSS Fragmentation Module Level 3
. 4 December 2018. CR. URL:
https://www.w3.org/TR/css-break-3/
[CSS-CASCADE-3]
Elika Etemad; Tab Atkins Jr..
CSS Cascading and Inheritance Level 3
. 11 February 2021. REC. URL:
https://www.w3.org/TR/css-cascade-3/
[CSS-CASCADE-5]
Elika Etemad; Miriam Suzanne; Tab Atkins Jr..
CSS Cascading and Inheritance Level 5
. 13 January 2022. CR. URL:
https://www.w3.org/TR/css-cascade-5/
[CSS-CASCADE-6]
Elika Etemad; Miriam Suzanne; Tab Atkins Jr..
CSS Cascading and Inheritance Level 6
. 6 September 2024. WD. URL:
https://www.w3.org/TR/css-cascade-6/
[CSS-COLOR-5]
Chris Lilley; et al.
CSS Color Module Level 5
. 18 March 2025. WD. URL:
https://www.w3.org/TR/css-color-5/
[CSS-DISPLAY-4]
Elika Etemad; Tab Atkins Jr..
CSS Display Module Level 4
. 6 November 2025. WD. URL:
https://www.w3.org/TR/css-display-4/
[CSS-POSITION-3]
Elika Etemad; Tab Atkins Jr..
CSS Positioned Layout Module Level 3
. 7 October 2025. WD. URL:
https://www.w3.org/TR/css-position-3/
[CSS-SCROLL-SNAP-1]
Matt Rakow; et al.
CSS Scroll Snap Module Level 1
. 11 March 2021. CR. URL:
https://www.w3.org/TR/css-scroll-snap-1/
[CSS-SIZING-3]
Tab Atkins Jr.; Elika Etemad.
CSS Box Sizing Module Level 3
. 17 December 2021. WD. URL:
https://www.w3.org/TR/css-sizing-3/
[CSS-TABLES-3]
François Remy; Greg Whitworth; David Baron.
CSS Table Module Level 3
. 27 July 2019. WD. URL:
https://www.w3.org/TR/css-tables-3/
[CSS-TEXT-3]
Elika Etemad; Koji Ishii; Florian Rivoal.
CSS Text Module Level 3
. 30 September 2024. CRD. URL:
https://www.w3.org/TR/css-text-3/
[CSS-UI-4]
Florian Rivoal.
CSS Basic User Interface Module Level 4
. 16 March 2021. WD. URL:
https://www.w3.org/TR/css-ui-4/
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
[CSSOM]
Daniel Glazman; Emilio Cobos Álvarez.
CSS Object Model (CSSOM)
. 26 August 2021. WD. URL:
https://www.w3.org/TR/cssom-1/
[RFC2119]
S. Bradner.
Key words for use in RFCs to Indicate Requirement Levels
. March 1997. Best Current Practice. URL:
https://datatracker.ietf.org/doc/html/rfc2119
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
<'width'>
auto
Same as height and width
no
As for the corresponding physical property
by computed value type
per grammar
Same as height, width
size
border-block
<'border-block-start'>
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
border-block-color
<'border-top-color'>{1,2}
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
border-block-end
<'border-top-width'> || <'border-top-style'> || <color>
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
border-block-end-color
<'border-top-color'>
currentcolor
Same as border-top-color
no
n/a
by computed value type
per grammar
Same as corresponding border-*-color properties
border-color
border-block-end-style
<'border-top-style'>
none
Same as border-top-style
no
n/a
discrete
per grammar
Same as corresponding border-*-style properties
border-style
border-block-end-width
<'border-top-width'>
medium
Same as border-top-width
no
n/a
by computed value type
per grammar
Same as corresponding border-*-width properties
border-width
border-block-start
<'border-top-width'> || <'border-top-style'> || <color>
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
border-block-start-color
<'border-top-color'>
currentcolor
Same as border-top-color
no
n/a
by computed value type
per grammar
Same as corresponding border-*-color properties
border-color
border-block-start-style
<'border-top-style'>
none
Same as border-top-style
no
n/a
discrete
per grammar
Same as corresponding border-*-style properties
border-style
border-block-start-width
<'border-top-width'>
medium
Same as border-top-width
no
n/a
by computed value type
per grammar
Same as corresponding border-*-width properties
border-width
border-block-style
<'border-top-style'>{1,2}
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
border-block-width
<'border-top-width'>{1,2}
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
border-end-end-radius
<'border-top-left-radius'>
Same as border-top-left-radius
Same as border-top-left-radius
no
Same as border-top-left-radius
by computed value type
per grammar
Same as corresponding physical border-*-radius properties
border-radius
border-end-start-radius
<'border-top-left-radius'>
Same as border-top-left-radius
Same as border-top-left-radius
no
Same as border-top-left-radius
by computed value type
per grammar
Same as corresponding physical border-*-radius properties
border-radius
border-inline
<'border-block-start'>
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
border-inline-color
<'border-top-color'>{1,2}
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
border-inline-end
<'border-top-width'> || <'border-top-style'> || <color>
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
border-inline-end-color
<'border-top-color'>
currentcolor
Same as border-top-color
no
n/a
by computed value type
per grammar
Same as corresponding border-*-color properties
border-color
border-inline-end-style
<'border-top-style'>
none
Same as border-top-style
no
n/a
discrete
per grammar
Same as corresponding border-*-style properties
border-style
border-inline-end-width
<'border-top-width'>
medium
Same as border-top-width
no
n/a
by computed value type
per grammar
Same as corresponding border-*-width properties
border-width
border-inline-start
<'border-top-width'> || <'border-top-style'> || <color>
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
border-inline-start-color
<'border-top-color'>
currentcolor
Same as border-top-color
no
n/a
by computed value type
per grammar
Same as corresponding border-*-color properties
border-color
border-inline-start-style
<'border-top-style'>
none
Same as border-top-style
no
n/a
discrete
per grammar
Same as corresponding border-*-style properties
border-style
border-inline-start-width
<'border-top-width'>
medium
Same as border-top-width
no
n/a
by computed value type
per grammar
Same as corresponding border-*-width properties
border-width
border-inline-style
<'border-top-style'>{1,2}
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
border-inline-width
<'border-top-width'>{1,2}
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
border-start-end-radius
<'border-top-left-radius'>
Same as border-top-left-radius
Same as border-top-left-radius
no
Same as border-top-left-radius
by computed value type
per grammar
Same as corresponding physical border-*-radius properties
border-radius
border-start-start-radius
<'border-top-left-radius'>
Same as border-top-left-radius
Same as border-top-left-radius
no
Same as border-top-left-radius
by computed value type
per grammar
Same as corresponding physical border-*-radius properties
border-radius
inline-size
<'width'>
auto
Same as height and width
no
As for the corresponding physical property
by computed value type
per grammar
Same as height, width
size
inset
<'top'>{1,4}
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
inset-block
<'top'>{1,2}
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
inset-block-end
<'top'>
auto
positioned elements
no
As for the corresponding physical property
by computed value type
per grammar
Same as corresponding top/right/bottom/left properties
inset
inset-block-start
<'top'>
auto
positioned elements
no
As for the corresponding physical property
by computed value type
per grammar
Same as corresponding top/right/bottom/left properties
inset
inset-inline
<'top'>{1,2}
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
inset-inline-end
<'top'>
auto
positioned elements
no
As for the corresponding physical property
by computed value type
per grammar
Same as corresponding top/right/bottom/left properties
inset
inset-inline-start
<'top'>
auto
positioned elements
no
As for the corresponding physical property
by computed value type
per grammar
Same as corresponding top/right/bottom/left properties
inset
margin-block
<'margin-top'>{1,2}
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
margin-block-end
<'margin-top'>
0
Same as margin-top
no
As for the corresponding physical property
by computed value type
per grammar
Same as corresponding margin-* properties
margin
margin-block-start
<'margin-top'>
0
Same as margin-top
no
As for the corresponding physical property
by computed value type
per grammar
Same as corresponding margin-* properties
margin
margin-inline
<'margin-top'>{1,2}
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
margin-inline-end
<'margin-top'>
0
Same as margin-top
no
As for the corresponding physical property
by computed value type
per grammar
Same as corresponding margin-* properties
margin
margin-inline-start
<'margin-top'>
0
Same as margin-top
no
As for the corresponding physical property
by computed value type
per grammar
Same as corresponding margin-* properties
margin
max-block-size
<'max-width'>
none
same as height and width
no
As for the corresponding physical property
by computed value type
per grammar
Same as max-height, max-width
max-size
max-inline-size
<'max-width'>
none
same as height and width
no
As for the corresponding physical property
by computed value type
per grammar
Same as max-height, max-width
max-size
min-block-size
<'min-width'>
0
same as height and width
no
As for the corresponding physical property
by computed value type
per grammar
Same as min-height, min-width
min-size
min-inline-size
<'min-width'>
0
same as height and width
no
As for the corresponding physical property
by computed value type
per grammar
Same as min-height, min-width
min-size
padding-block
<'padding-top'>{1,2}
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
padding-block-end
<'padding-top'>
0
Same as padding-top
no
As for the corresponding physical property
by computed value type
per grammar
Same as corresponding padding-* properties
padding
padding-block-start
<'padding-top'>
0
Same as padding-top
no
As for the corresponding physical property
by computed value type
per grammar
Same as corresponding padding-* properties
padding
padding-inline
<'padding-top'>{1,2}
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
padding-inline-end
<'padding-top'>
0
Same as padding-top
no
As for the corresponding physical property
by computed value type
per grammar
Same as corresponding padding-* properties
padding
padding-inline-start
<'padding-top'>
0
Same as padding-top
no
As for the corresponding physical property
by computed value type
per grammar
Same as corresponding padding-* properties
padding
Issues Index
Things That Are Unstable
Since implementation of parts of this module is effectively required for shipping an implementation of
CSS Writing Modes
on the Web (in order to correctly implement the default HTML styles), the CSSWG resolved that the requisite features in
§ 2 Flow-Relative Values: block-start, block-end, inline-start, inline-end
and
§ 4 Flow-Relative Box Model Properties
are approved for shipping. (See
FPWD announcement
for additional background.)
However, there are a few significant open issues:
The
logical
keyword on shorthands, because the name of the keyword may change or it may be replaced by some other syntactic marker. (This feature will be deferred from this level for further development if there is no clearly satisfactory mechanism proposed, see
Issue 1282
.)
Whether flow-relative longhands inherit from their namesake on the parent, or are mapped to a physical property and inherit from that property. (See
Issue 3029
.)
Whether shorthands like
margin
expand to both sets of longhands, or only the ones that were set. (See
Issue 3030
.)
Comments, suggestions, and use cases are welcome on these issues. Please file them in GitHub, tweet them to @csswg, or send them to www-style@w3.org.
↵
The proposed syntax for this feature is
under discussion
and is almost guaranteed to change from what is described here. This section remains in the draft to promote discussion of alternatives, to document the affected properties, and to specify the expected impact on the interpretation of whatever syntactic switch is ultimately chosen.
↵
```
