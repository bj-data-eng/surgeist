Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Box Model Module Level 3](https://www.w3.org/TR/2024/REC-css-box-3-20240411/).

Original copyright notice (from the matching exact HTML edition): Copyright © 2024 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Box Model Module Level 3

Source snapshot: https://www.w3.org/TR/2024/REC-css-box-3-20240411/

Snapshot SHA-256: be3732bfdaa640b3951fbecb7afece39d142f5a15a1b8a7c9af1b72f815120ac

Conversion: offline format conversion of the exact stored extracted-text; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- This stored input is an already extracted plain-text witness. It has no recoverable original HTML structure. The complete source text is preserved in a fenced block; no summary or specification regeneration was performed.

---

```text
CSS Box Model Module Level 3
CSS Box Model Module Level 3
W3C Recommendation
,
11 April 2024
More details about this document
This version:
https://www.w3.org/TR/2024/REC-css-box-3-20240411/
Latest published version:
https://www.w3.org/TR/css-box-3/
Editor's Draft:
https://drafts.csswg.org/css-box-3/
Previous Versions:
https://www.w3.org/TR/2022/CRD-css-box-3-20221103/
History:
https://www.w3.org/standards/history/css-box-3/
Implementation Report:
https://wpt.fyi/results/css/css-box
Feedback:
CSSWG Issues Repository
CSSWG GitHub
Editor:
Elika J. Etemad / fantasai
(
Apple
)
Suggest an Edit for this Spec:
GitHub Editor
Errata:
https://www.w3.org/Style/2024/REC-css-box-3-20240411-errata.html
Copyright
© 2024
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
This specification describes the margin and padding properties, which create spacing in and around a CSS box.
CSS
is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.
Status of this document
This section describes the status of this document at the time of its publication. A list of current W3C publications and the latest revision of this technical report can be found in the
W3C technical reports index at https://www.w3.org/TR/.
This document was published by the
CSS Working Group
as a Recommendation using the
Recommendation track
.
A W3C Recommendation is a specification that, after extensive consensus-building, is endorsed by
W3C
and its Members, and has commitments from Working Group members to
royalty-free licensing
for implementations.
W3C recommends the wide deployment of this specification as a standard for the Web.
Please send feedback by
filing issues in GitHub
(preferred), including the spec code “css-box” in the title, like this: “[css-box]
…summary of comment…
”. All issues and comments are
archived
. Alternately, feedback can be sent to the (
archived
) public mailing list
www-style@w3.org
.
This document is governed by the
03 November 2023 W3C Process Document
.
This document was produced by a group operating under the
W3C Patent Policy
. W3C maintains a
public list of any patent disclosures
made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains
Essential Claim(s)
must disclose the information in accordance with
section 6 of the W3C Patent Policy
.
Table of Contents
1
Introduction
1.1
Value Definitions
1.2
Module Interactions
2
The CSS Box Model
2.1
Boxes and Edges
2.2
Fragmentation
2.3
Box-edge Keywords
3
Margins
3.1
Page-relative (Physical) Margin Properties: the
margin-top
,
margin-right
,
margin-bottom
, and
margin-left
properties
3.2
Margin Shorthand: the
margin
property
4
Padding
4.1
Page-relative (Physical) Padding Properties: the
padding-top
,
padding-right
,
padding-bottom
, and
padding-left
properties
4.2
Padding Shorthand: the
padding
property
5
Borders
6
Changes
6.1
Recent Changes
6.2
Changes Since CSS Level 2
7
Privacy Considerations
8
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
1.
Introduction
This subsection is not normative.
CSS describes how each element and each string of text in a source document is laid out by transforming the
element tree
into a set of
boxes
, whose size, position, and stacking level on the
canvas
depend on the values of their CSS properties.
Note:
CSS Cascading and Inheritance
describes how properties are assigned to elements in the box tree, while
CSS Display 3
§ 1 Introduction
describes how the
element tree
is transformed into the
box tree
.
Each CSS
box
has a rectangular content area, a band of padding around the content, a border around the padding, and a margin outside the border. The
sizing properties
[css-sizing-3]
, together with various other properties that control layout, define the size of the content area. The box styling properties—​
padding
and its longhands,
border
and its longhands, and
margin
and its longhands—​define the sizes of these other areas.
Margins
and
padding
are defined in this module.
Borders
are similarly defined in
[css-backgrounds-3]
.
Note:
This module only defines the
physical
per-side
longhand
properties. Additional,
flow-relative
longhand
properties are defined in
[css-logical-1]
. Note that both sets of
longhand
properties control the same
margins
/
padding
: they are just different ways of indexing each side.
History of CSS Box module and the definition of Block Layout
This module
originally contained
the CSS Level 3 specification prose relating to box generation (now defined in
[css-display-3]
), the box model (defined here), as well as block layout (now only defined in
[CSS2]
Chapters 9 and 10). Since its maintenance was put aside during the development of CSS2.1, its prose was severely outdated by the time CSS2 Revision 1 was finally completed. Therefore, the block layout portion of the prose has been retired, to be re-synched to
CSS2
and updated as input to a new Block Layout module at some point in the future. It is being split apart from this module and from the
CSS Display Module
both because of the practical concern that it would be a huge amount of work and also in recognition that CSS now has multiple layout models (
Flex Layout
,
Grid Layout
,
Positioned Layout
, and
Table Layout
, in addition to Block Layout) which each deserve their own parallel module.
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
This module replaces the definitions of the margin and padding properties defined in
[CSS2]
sections 8.1, 8.2, 8.3 (but not 8.3.1), and 8.4.
All properties in this module apply to the
::first-line
and
::first-letter
pseudo-elements.
2.
The CSS Box Model
Each box has a
content area
(which contains its content—​text, descendant boxes, an image or other
replaced element
content, etc.) and optional surrounding
padding
,
border
, and
margin areas
; the size of each area is specified by corresponding properties, and can be zero (or in the case of margins, negative). The following diagram shows how these areas relate and the terminology used to refer to the various parts of the box:
The various areas and edges of a typical box.
The background of the content, padding, and border areas of a box is specified by its
background
properties. The border area can additionally be painted with a border style using the
border
properties. Margins are always transparent. See
[css-backgrounds-3]
.
The margin, border, and padding can be broken down into top, right, bottom, and left segments, each of which can be controlled independently by its corresponding property.
2.1.
Boxes and Edges
The perimeter of each of the four areas (content, padding, border, and margin) is called an
edge
, and each
edge
can be broken down into a top, right, bottom, and left side. Thus each
box
has four
edges
each composed of four sides:
content edge
or
inner edge
The content edge surrounds the rectangle given by the width and height of the box, which often depend on the element’s content and/or its
containing block
size. The four sides of the
content edge
together define the box’s
content box
.
padding edge
The padding edge surrounds the box’s padding. If the padding has zero width on a given side, the padding edge coincides with the content edge on that side. The four sides of the
padding edge
together define the box’s
padding box
, which contains both the
content
and
padding areas
.
border edge
The border edge surrounds the box’s border. If the border has zero width on a given side, the border edge coincides with the padding edge on that side. The four sides of the
border edge
together define the box’s
border box
, which contains the box’s
content
,
padding
, and
border areas
.
margin edge
or
outer edge
The margin edge surrounds the box’s margin. If the margin has zero width on a given side, the margin edge coincides with the border edge on that side. The four sides of the
margin edge
together define the box’s
margin box
, which contains the all of the box’s
content
,
padding
,
border
, and
margin areas
.
2.2.
Fragmentation
When a box
fragments
—​is broken, as across lines or across pages, into separate
box fragments
—​each of its boxes (
content box
,
padding box
,
border box
,
margin box
) also fragments. How the content/padding/border/margin areas react to fragmentation is specified in
[css-break-3]
and controlled by the
box-decoration-break
property.
2.3.
Box-edge Keywords
The following
<box>
CSS keywords are defined for use in properties (such as
transform-box
and
background-clip
) that need to refer to various box edges:
content-box
Refers to the
content box
or
content edge
. (In an SVG context, treated as
fill-box
.)
padding-box
Refers to the
padding box
or
padding edge
. (In an SVG context, treated as
fill-box
.)
border-box
Refers to the
border box
or
border edge
. (In an SVG context, treated as
stroke-box
.)
margin-box
Refers to the
margin box
or
margin edge
. (In an SVG context, treated as
stroke-box
.)
fill-box
Refers to the
object bounding box
or its edges. (In a CSS box context, treated as
content-box
.)
stroke-box
Refers to the
stroke bounding box
or its edges. (In a CSS box context, treated as
border-box
.)
view-box
Refers to the nearest
SVG viewport
element’s
origin box
, which is a rectangle with the width and height of the initial SVG
user coordinate system
established by the
viewBox
attribute for that element, positioned such that its top left corner is anchored at the coordinate system origin. (In a CSS box context, treated as
border-box
.)
Note:
When the
viewBox
includes non-zero
min-x
or
min-y
offsets, this
origin box
does not actually correspond to the visible region rectangle defined by the
viewBox
!
For convenience, the following value types are defined to represents commonly used subsets of
<box>
:
<visual-box>
= content-box
|
padding-box
|
border-box
<layout-box>
=
<visual-box>
|
margin-box
<paint-box>
=
<visual-box>
|
fill-box
|
stroke-box
<coord-box>
=
<paint-box>
|
view-box
3.
Margins
Margins
surround the border edge of a box, providing spacing between boxes. The
margin properties
specify the thickness of the
margin area
of a box. The
margin
shorthand property
sets the margin for all four sides while the margin
longhand properties
only set their respective side. This section defines the
physical
margin
longhands
. (Additional
flow-relative
margin
longhands
are defined in
[css-logical-1]
.)
Note:
Adjoining margins in
block layout
can
collapse
. See
CSS2§8.3.1 Collapsing Margins
for details. Also, margins adjoining a
fragmentation break
are sometimes truncated. See
CSS Fragmentation 3
§ 5.2 Adjoining Margins at Breaks
for details.
3.1.
Page-relative (Physical) Margin Properties: the
margin-top
,
margin-right
,
margin-bottom
, and
margin-left
properties
Name:
margin-top
,
margin-right
,
margin-bottom
,
margin-left
Value:
<length-percentage>
|
auto
Initial:
0
Applies to:
all elements except
internal table elements
Inherited:
no
Percentages:
refer to
logical width
of containing block
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
margin
These properties set the top, right, bottom, and left
margin
of a
box
, respectively.
Negative values for margin properties are allowed, but there may be implementation-specific limits.
Note:
Application to
internal ruby boxes
[CSS-RUBY-1]
is not defined in this specification.
3.2.
Margin Shorthand: the
margin
property
Name:
margin
Value:
<'margin-top'>
{1,4}
Initial:
0
Applies to:
see individual properties
Inherited:
no
Percentages:
refer to
logical width
of containing block
Computed value:
see individual properties
Canonical order:
per grammar
Animation type:
by computed value type
The
margin
property is a shorthand property for setting
margin-top
,
margin-right
,
margin-bottom
, and
margin-left
in a single declaration.
If there is only one component value, it applies to all sides. If there are two values, the top and bottom margins are set to the first value and the right and left margins are set to the second. If there are three values, the top is set to the first value, the left and right are set to the second, and the bottom is set to the third. If there are four values they apply to the top, right, bottom, and left, respectively.
The following code demonstrates some possible
margin
declarations.
body { margin: 2em } /* all margins set to 2em */ body { margin: 1em 2em } /* top & bottom = 1em, right & left = 2em */ body { margin: 1em 2em 3em } /* top=1em, right=2em, bottom=3em, left=2em */
The last rule of the example above is equivalent to the example below:
body { margin-top: 1em; margin-right: 2em; margin-bottom: 3em; margin-left: 2em; /* copied from opposite side (right) */ }
4.
Padding
Padding
is inserted between the content edge and the padding edge of a box, providing spacing between the content and the border. The
padding properties
specify the thickness of the
padding area
of a box. The
padding
shorthand property
sets the padding for all four sides while the padding
longhand properties
only set their respective side. This section defines the
physical
padding
longhands
. (Additional
flow-relative
padding
longhands
are defined in
[css-logical-1]
.)
Note:
Backgrounds specified on the box are by default laid out and painted within the padding edges. (They are additionally painted underneath the border, in the
border area
.) This behavior can be adjusted using the
background-origin
and
background-clip
properties.
4.1.
Page-relative (Physical) Padding Properties: the
padding-top
,
padding-right
,
padding-bottom
, and
padding-left
properties
Name:
padding-top
,
padding-right
,
padding-bottom
,
padding-left
Value:
<length-percentage [0,∞]>
Initial:
0
Applies to:
all elements except:
internal table elements
other than table cells
Inherited:
no
Percentages:
refer to
logical width
of containing block
Computed value:
a computed
<length-percentage>
value
Canonical order:
per grammar
Animation type:
by computed value type
Logical property group:
padding
These properties set the top, right, bottom, and left
padding
of a
box
, respectively.
Negative values for padding properties are invalid.
Note:
Application to
internal ruby boxes
[CSS-RUBY-1]
is not defined in this specification.
4.2.
Padding Shorthand: the
padding
property
Name:
padding
Value:
<'padding-top'>
{1,4}
Initial:
0
Applies to:
see individual properties
Inherited:
no
Percentages:
refer to
logical width
of containing block
Computed value:
see individual properties
Canonical order:
per grammar
Animation type:
by computed value type
The
padding
property is a shorthand property for setting
padding-top
,
padding-right
,
padding-bottom
, and
padding-left
in a single declaration.
If there is only one component value, it applies to all sides. If there are two values, the top and bottom padding are set to the first value and the right and left padding are set to the second. If there are three values, the top is set to the first value, the left and right are set to the second, and the bottom is set to the third.
The following code demonstrates some possible
padding
declarations.
body { padding: 2em } /* all padding set to 2em */ body { padding: 1em 2em } /* top & bottom = 1em, right & left = 2em */ body { padding: 1em 2em 3em } /* top=1em, right=2em, bottom=3em, left=2em */
The last rule of the example above is equivalent to the example below:
body { padding-top: 1em; padding-right: 2em; padding-bottom: 3em; padding-left: 2em; /* copied from opposite side (right) */ }
5.
Borders
Borders
fill the
border area
, to visually delineate the edges of the box, The
border properties
specify the thickness of the
border area
of a box, as well as its drawing style and color. See
[css-backgrounds-3]
for the definition of these properties, including their
physical
longhands. (Additional
flow-relative
border longhands are defined in
[css-logical-1]
.)
6.
Changes
6.1.
Recent Changes
There have only been minor editorial adjustments since the
6 April 2023 Recommendation
:
Adding some subheadings to
§ 2 The CSS Box Model
and moving a paragraph accordingly.
Changing “document tree” to “element tree” in the
§ 1 Introduction
to align with the terminology used in
[css-display-3]
.
Adding the “Logical Property Group” line to the property definition tables.
6.2.
Changes Since CSS Level 2
The following changes have been made to this module since
CSS Level 2
:
Marked application to internal ruby boxes as undefined in this level.
Adapting the prose slightly to account for vertical
writing modes
.
Cross-linking to relevant concepts in
[css-break-3]
and
[css-backgrounds-3]
.
Providing a
centralized common definition
of keywords that reference the various box edges for use in properties defined outside this spec (e.g.
background-clip
) to avoid repetition (and the resulting inevitable synchronization errors).
7.
Privacy Considerations
No privacy considerations have been reported on this module.
8.
Security Considerations
No security considerations have been reported on this module.
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
border
, in § 5
border area
, in § 2
border box
, in § 2.1
border-box
, in § 2.3
border edge
, in § 2.1
border properties
, in § 5
<box>
, in § 2.3
box edge
, in § 2.1
content area
, in § 2
content box
, in § 2.1
content-box
, in § 2.3
content edge
, in § 2.1
<coord-box>
, in § 2.3
edge
, in § 2.1
fill-box
, in § 2.3
inner edge
, in § 2.1
<layout-box>
, in § 2.3
margin
(property)
, in § 3.2
definition of
, in § 3
margin area
, in § 2
margin-bottom
, in § 3.1
margin box
, in § 2.1
margin-box
, in § 2.3
margin edge
, in § 2.1
margin-left
, in § 3.1
margin properties
, in § 3
margin-right
, in § 3.1
margin-top
, in § 3.1
origin box
, in § 2.3
outer edge
, in § 2.1
Padding
, in § 4
padding
, in § 4.2
padding area
, in § 2
padding-bottom
, in § 4.1
padding box
, in § 2.1
padding-box
, in § 2.3
padding edge
, in § 2.1
padding-left
, in § 4.1
padding properties
, in § 4
padding-right
, in § 4.1
padding-top
, in § 4.1
<paint-box>
, in § 2.3
stroke-box
, in § 2.3
SVG viewport origin box
, in § 2.3
view-box
, in § 2.3
<visual-box>
, in § 2.3
Terms defined by reference
[CSS-BACKGROUNDS-3]
defines the following terms:
background
background-clip
background-origin
border
[CSS-BREAK-4]
defines the following terms:
box fragment
box-decoration-break
fragmentation break
[CSS-CASCADE-5]
defines the following terms:
longhand
longhand property
shorthand property
[CSS-DISPLAY-3]
defines the following terms:
block layout
box
box tree
containing block
element tree
internal table element
replaced element
[CSS-PSEUDO-4]
defines the following terms:
::first-letter
::first-line
[CSS-RUBY-1]
defines the following terms:
internal ruby boxes
[CSS-SIZING-3]
defines the following terms:
sizing property
[CSS-TRANSFORMS-1]
defines the following terms:
transform-box
[CSS-VALUES-4]
defines the following terms:
<length-percentage>
css-wide keywords
{a,b}
|
[CSS-WRITING-MODES-4]
defines the following terms:
flow-relative
logical width
physical
writing mode
[CSS2]
defines the following terms:
canvas
[SVG2]
defines the following terms:
object bounding box
stroke bounding box
svg viewports
user coordinate system
viewbox
References
Normative References
[CSS-BACKGROUNDS-3]
Elika Etemad; Brad Kemper.
CSS Backgrounds and Borders Module Level 3
. 11 March 2024. CR. URL:
https://www.w3.org/TR/css-backgrounds-3/
[CSS-BREAK-4]
Rossen Atanassov; Elika Etemad.
CSS Fragmentation Module Level 4
. 18 December 2018. WD. URL:
https://www.w3.org/TR/css-break-4/
[CSS-CASCADE-5]
Elika Etemad; Miriam Suzanne; Tab Atkins Jr..
CSS Cascading and Inheritance Level 5
. 13 January 2022. CR. URL:
https://www.w3.org/TR/css-cascade-5/
[CSS-DISPLAY-3]
Elika Etemad; Tab Atkins Jr..
CSS Display Module Level 3
. 30 March 2023. CR. URL:
https://www.w3.org/TR/css-display-3/
[CSS-PSEUDO-4]
Daniel Glazman; Elika Etemad; Alan Stearns.
CSS Pseudo-Elements Module Level 4
. 30 December 2022. WD. URL:
https://www.w3.org/TR/css-pseudo-4/
[CSS-SIZING-3]
Tab Atkins Jr.; Elika Etemad.
CSS Box Sizing Module Level 3
. 17 December 2021. WD. URL:
https://www.w3.org/TR/css-sizing-3/
[CSS-TRANSFORMS-1]
Simon Fraser; et al.
CSS Transforms Module Level 1
. 14 February 2019. CR. URL:
https://www.w3.org/TR/css-transforms-1/
[CSS-VALUES-3]
Tab Atkins Jr.; Elika Etemad.
CSS Values and Units Module Level 3
. 22 March 2024. CR. URL:
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
https://www.w3.org/TR/CSS21/
[RFC2119]
S. Bradner.
Key words for use in RFCs to Indicate Requirement Levels
. March 1997. Best Current Practice. URL:
https://datatracker.ietf.org/doc/html/rfc2119
[SVG2]
Amelia Bellamy-Royds; et al.
Scalable Vector Graphics (SVG) 2
. 4 October 2018. CR. URL:
https://www.w3.org/TR/SVG2/
Informative References
[CSS-BREAK-3]
Rossen Atanassov; Elika Etemad.
CSS Fragmentation Module Level 3
. 4 December 2018. CR. URL:
https://www.w3.org/TR/css-break-3/
[CSS-LOGICAL-1]
Rossen Atanassov; Elika Etemad.
CSS Logical Properties and Values Level 1
. 27 August 2018. WD. URL:
https://www.w3.org/TR/css-logical-1/
[CSS-RUBY-1]
Elika Etemad; et al.
CSS Ruby Annotation Layout Module Level 1
. 31 December 2022. WD. URL:
https://www.w3.org/TR/css-ruby-1/
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
margin
<'margin-top'>{1,4}
0
see individual properties
no
refer to logical width of containing block
by computed value type
per grammar
see individual properties
margin-bottom
<length-percentage> | auto
0
all elements except internal table elements
no
refer to logical width of containing block
by computed value type
per grammar
the keyword auto or a computed <length-percentage> value
margin
margin-left
<length-percentage> | auto
0
all elements except internal table elements
no
refer to logical width of containing block
by computed value type
per grammar
the keyword auto or a computed <length-percentage> value
margin
margin-right
<length-percentage> | auto
0
all elements except internal table elements
no
refer to logical width of containing block
by computed value type
per grammar
the keyword auto or a computed <length-percentage> value
margin
margin-top
<length-percentage> | auto
0
all elements except internal table elements
no
refer to logical width of containing block
by computed value type
per grammar
the keyword auto or a computed <length-percentage> value
margin
padding
<'padding-top'>{1,4}
0
see individual properties
no
refer to logical width of containing block
by computed value type
per grammar
see individual properties
padding-bottom
<length-percentage [0,∞]>
0
all elements except: internal table elements other than table cells
no
refer to logical width of containing block
by computed value type
per grammar
a computed <length-percentage> value
padding
padding-left
<length-percentage [0,∞]>
0
all elements except: internal table elements other than table cells
no
refer to logical width of containing block
by computed value type
per grammar
a computed <length-percentage> value
padding
padding-right
<length-percentage [0,∞]>
0
all elements except: internal table elements other than table cells
no
refer to logical width of containing block
by computed value type
per grammar
a computed <length-percentage> value
padding
padding-top
<length-percentage [0,∞]>
0
all elements except: internal table elements other than table cells
no
refer to logical width of containing block
by computed value type
per grammar
a computed <length-percentage> value
padding
```
