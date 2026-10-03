Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Display Module Level 3](https://www.w3.org/TR/2026/CRD-css-display-3-20260605/).

Original copyright notice (from the matching exact HTML edition): Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Display Module Level 3

Source snapshot: https://www.w3.org/TR/2026/CRD-css-display-3-20260605/

Snapshot SHA-256: 3cdbd7b195395cd751d6cb78099001884b72cfaa4623b69defad47956d483c5e

Conversion: offline format conversion of the exact stored extracted-text; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- This stored input is an already extracted plain-text witness. It has no recoverable original HTML structure. The complete source text is preserved in a fenced block; no summary or specification regeneration was performed.

---

```text
CSS Display Module Level 3
CSS Display Module Level 3
W3C Candidate Recommendation Draft
,
5 June 2026
More details about this document
This version:
https://www.w3.org/TR/2026/CRD-css-display-3-20260605/
Latest published version:
https://www.w3.org/TR/css-display-3/
Editor's Draft:
https://drafts.csswg.org/css-display/
Previous Versions:
https://www.w3.org/TR/2023/CR-css-display-3-20230330/
https://www.w3.org/TR/2021/CRD-css-display-3-20210903/
https://www.w3.org/TR/2020/CRD-css-display-3-20201218/
https://www.w3.org/TR/2020/CR-css-display-3-20200519/
https://www.w3.org/TR/2019/CR-css-display-3-20190711/
https://www.w3.org/TR/2018/CR-css-display-3-20180828/
https://www.w3.org/TR/2018/WD-css-display-3-20180809/
https://www.w3.org/TR/2018/WD-css-display-3-20180420/
https://www.w3.org/TR/2017/WD-css-display-3-20170720/
https://www.w3.org/TR/2017/WD-css-display-3-20170126/
https://www.w3.org/TR/2015/WD-css-display-3-20151015/
https://www.w3.org/TR/2015/WD-css-display-3-20150721/
https://www.w3.org/TR/2014/WD-css-display-3-20140911/
History:
https://www.w3.org/standards/history/css-display-3/
Implementation Report:
https://wpt.fyi/results/css/css-display?label=master&label=experimental&aligned
Feedback:
CSSWG Issues Repository
Editors:
Tab Atkins Jr.
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
https://wpt.fyi/results/css/css-display/
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
This module describes how the CSS formatting box tree is generated from the document element tree and defines the
display
property that controls it.
CSS
is a language for describing the rendering of structured documents (such as HTML and XML) on screen, on paper, etc.
Status of this document
This section describes the status of this document at the time of its publication. A list of current W3C publications and the latest revision of this technical report can be found in the
W3C standards and drafts index.
This document was published by the
CSS Working Group
as a
Candidate Recommendation Draft
using the
Recommendation track
. Publication as a Candidate Recommendation does not imply endorsement by
W3C
and its Members. A Candidate Recommendation Draft integrates changes from the previous Candidate Recommendation that the Working Group intends to include in a subsequent Candidate Recommendation Snapshot.
This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.
Please send feedback by
filing issues in GitHub
(preferred), including the spec code “css-display” in the title, like this: “[css-display]
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
A
preliminary implementation report
is available. Further tests will be added during the CR period.
The following features are at-risk, and may be dropped during the CR period:
Application of
::first-letter
in the presence of run-ins
display: run-in
All multi-keyword values of
display
“At-risk” is a W3C Process term-of-art, and does not necessarily imply that the feature is in danger of being dropped or delayed. It means that the WG believes the feature may have difficulty being interoperably implemented in a timely manner, and marking it as such allows the WG to drop the feature if necessary when transitioning to the Proposed Rec stage, without having to publish a new Candidate Rec without the feature first.
Table of Contents
1
Introduction
1.1
Module interactions
1.2
Value Definitions
2
Box Layout Modes: the
display
property
2.1
Outer Display Roles for Flow Layout: the
block
,
inline
, and
run-in
keywords
2.2
Inner Display Layout Models: the
flow
,
flow-root
,
table
,
flex
,
grid
, and
ruby
keywords
2.3
Generating Marker Boxes: the
list-item
keyword
2.4
Layout-Internal Display Types: the
table-*
and
ruby-*
keywords
2.5
Box Generation: the
none
and
contents
keywords
2.6
Precomposed Inline-level Display Values
2.7
Automatic Box Type Transformations
2.8
The Root Element’s Principal Box
3
Display Order: the
order
property
3.1
Reordering and Accessibility
4
Invisibility: the
visibility
property
5
Run-In Layout
Appendix A: Glossary
Appendix B: Effects of
display: contents
on Unusual Elements
HTML Elements
SVG Elements
MathML Elements
Appendix C: Box Construction Guidelines for Spec Authors
Acknowledgments
Changes
Changes Since 2023 Candidate Recommendation Snapshot
Changes Since 2020 Candidate Recommendation
Changes Since 2019 Candidate Recommendation
Changes Prior to Candidate Recommendation Status
Privacy Considerations
Security Considerations
Self-Review Questionnaire
Conformance
Document conventions
Conformance classes
Partial implementations
Implementations of Unstable and Proprietary Features
Non-experimental implementations
CR exit criteria
Index
Terms defined by this specification
Terms defined by reference
References
Normative References
Non-Normative References
Property Index
1.
Introduction
This section is normative.
CSS takes a source document organized as a
tree
of
elements
(which can contain a mix of other
elements
and
text nodes
) and
text nodes
(which can contain text), and renders it onto a
canvas
such as your screen, a piece of paper, or an audio stream. Although any such source document can be rendered with CSS, the most commonly used type is the DOM.
[DOM]
(Some of these more complex tree types might have additional types of nodes, such as the comment nodes in the DOM. For the purposes of CSS, all of these additional types of nodes are ignored, as if they didn’t exist.)
To do this, it generates an intermediary structure, the
box tree
, which represents the formatting structure of the rendered document. Each
box
in the
box tree
represents its corresponding
element
(or
pseudo-element
) in space and/or time on the canvas, while each
text sequence
in the
box tree
likewise represents the corresponding contents of its
text nodes
.
To create the
box tree
, CSS first uses
cascading and inheritance
, to assign a
computed value
for each CSS property to each
element
and
text node
in the source tree. (See
[CSS-CASCADE-3]
.)
Then, for each
element
, CSS generates zero or more
boxes
as specified by that element’s
display
property. Typically, an element generates a single
box
, the
principal box
, which represents itself and contains its contents in the
box tree
. However, some
display
values (e.g.
display: list-item
) generate more than one box (e.g. a
principal block box
and a child
marker box
). And some values (such as
none
or
contents
) cause the
element
and/or its descendants to not generate any
boxes
at all.
Boxes
are often referred to by their
display
type—​e.g. a
box
generated by an element with
display: block
is called a “block box” or just a “block”.
A
box
is assigned the same styles as its generating
element
, unless otherwise indicated. In general,
inherited properties
are assigned to the
principal box
, and then inherit through the box tree to any other boxes generated by the same element. Non-inherited properties default to applying to the
principal box
, but when the element generates multiple boxes, are sometimes defined to apply to a different box: for example, the
border
properties applied to a table element are applied to its
table grid box
, not to its
principal
table wrapper box
. If the value computation process alters the styles of those boxes, and the element’s style is requested (such as through
getComputedStyle()
), the element reflects, for each property, the value from the box to which that property was applied.
Similarly, each contiguous sequence of sibling
text nodes
generates a
text sequence
containing their text contents, which is assigned the same styles as the generating
text nodes
. If the sequence contains no text, however, it does not generate a
text sequence
.
In constructing the box tree, boxes generated by an element are descendants of the principal box of any ancestor elements. In the general case, the direct
parent box
of an element’s
principal box
is the
principal box
of its nearest ancestor element that generates a
box
; however, there are some exceptions, such as for
run-in
boxes, display types (like tables) that generate multiple container boxes, and intervening
anonymous boxes
.
An
anonymous box
is a box that is not associated with any element.
Anonymous boxes
are generated in certain circumstances to fix up the
box tree
when it requires a particular nested structure that is not provided by the boxes generated from the
element tree
. For example, a
table cell box
requires a particular type of parent box (the
table row box
), and will generate an
anonymous
table row box
around itself if its parent is not a
table row box
. (See
[CSS2]
§
17.2.1
.) Unlike element-generated boxes, whose styles inherit strictly through the element tree, anonymous boxes (which only exist in the
box tree
)
inherit
through their
box tree
parentage.
In the course of layout,
boxes
and
text sequences
can be broken into multiple
fragments
. This happens, for example, when an
inline box
and/or
text sequence
is broken across lines, or when a
block box
is broken across pages or columns, in a process called
fragmentation
. It can also happen due to bidi reordering of text (see
Applying the Bidirectional Reordering Algorithm
in
CSS Writing Modes
) or higher-level
display type
box splitting, e.g.
block-in-inline splitting
(see
CSS2§9.2
) or
column-spanner-in-block
splitting (see
CSS Multi-column Layout
). A
box
therefore consists of one or more
box fragments
, and a
text sequence
consists of one or more
text fragments
. See
[CSS-BREAK-3]
for more information on
fragmentation
.
Note:
Many of the CSS specs were written before this terminology was ironed out, or refer to things incorrectly, so view older specs with caution when they’re using these terms. It should be possible to infer from context which term they really mean. Please
report errors
in specs when you find them, so they can be corrected.
Note:
Further information on the “aural” box tree and its interaction with the
display
property can be found in the
CSS Speech Module
.
[CSS-SPEECH-1]
Tests
empty-text-baseline-001.html
(live test)
(source)
empty-text-baseline-002.html
(live test)
(source)
1.1.
Module interactions
This module replaces and extends the definition of the
display
property defined in
[CSS2]
section 9.2.4.
None of the properties in this module apply to the
::first-line
or
::first-letter
pseudo-elements.
Tests
display-first-letter-001.html
(live test)
(source)
display-first-line-001.html
(live test)
(source)
display-first-line-002.html
(live test)
(source)
display-inline-dynamic-001.html
(live test)
(source)
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
Box Layout Modes: the
display
property
Name:
display
Value:
[
<display-outside>
||
<display-inside>
]
|
<display-listitem>
|
<display-internal>
|
<display-box>
|
<display-legacy>
Initial:
inline
Applies to:
all elements
Inherited:
no
Percentages:
n/a
Computed value:
a pair of keywords representing the
inner
and
outer
display types plus optional
list-item
flag, or a
<display-internal>
or
<display-box>
keyword; see prose in a variety of specs for computation rules
Canonical order:
per grammar
Animation type:
not animatable
User agents are expected to support this property on all media, including non-visual ones. The
display
property defines an element’s
display type
, which consists of the two basic qualities of how an element generates boxes:
the
inner display type
, which defines (if it is a
non-replaced element
) the kind of
formatting context
it generates, dictating how its descendant boxes are laid out. (The inner display of a
replaced element
is outside the scope of CSS.)
the
outer display type
, which dictates how the
principal box
itself participates in
flow layout
.
Text sequences
have no
display type
.
Some
display
values have additional side-effects: such as
list-item
, which also generates a
::marker
pseudo-element, and
none
, which causes the element’s entire subtree to be left out of the box tree.
The
display
property has no effect on an element’s semantics: these are defined by the
document language
and
are not affected by CSS
. Aside from the
none
value, which also affects the aural/speech output
[CSS-SPEECH-1]
and interactivity of an element and its descendants, the
display
property only affects visual layout: its purpose is to allow designers freedom to change the layout behavior of an element
without
affecting the underlying document semantics.
Values are defined as follows:
<display-outside>
= block
|
inline
|
run-in
<display-inside>
= flow
|
flow-root
|
table
|
flex
|
grid
|
ruby
<display-listitem>
=
<display-outside>
?
&&
[ flow
|
flow-root ]
?
&&
list-item
<display-internal>
= table-row-group
|
table-header-group
|
table-footer-group
|
table-row
|
table-cell
|
table-column-group
|
table-column
|
table-caption
|
ruby-base
|
ruby-text
|
ruby-base-container
|
ruby-text-container
<display-box>
= contents
|
none
<display-legacy>
= inline-block
|
inline-table
|
inline-flex
|
inline-grid
The following informative table summarizes the values of
display
:
Short
display
Full
display
Generated box
none
—
subtree omitted from
box tree
contents
—
element replaced by contents in
box tree
block
block flow
block-level
block container
aka
block box
flow-root
block flow-root
block-level
block container
that establishes a new
block formatting context
(
BFC
)
inline
inline flow
inline box
inline-block
inline flow-root
inline-level
block container
aka
inline block
run-in
run-in flow
run-in box
(
inline box
with special box-tree-munging rules)
list-item
block flow list-item
block box
with additional
marker box
inline list-item
inline flow list-item
inline box
with additional
marker box
flex
block flex
block-level
flex container
inline-flex
inline flex
inline-level
flex container
grid
block grid
block-level
grid container
inline-grid
inline grid
inline-level
grid container
ruby
inline ruby
inline-level
ruby container
block ruby
block ruby
block box
containing
ruby container
table
block table
block-level
table wrapper box
containing
table grid box
inline-table
inline table
inline-level
table wrapper box
containing
table grid box
<display-internal>
types
—
layout-specific internal box
Note:
Following the precedence rules of “most backwards-compatible, then shortest”, serialization of equivalent
display
values uses the “Short
display
” column.
[CSSOM]
Tests
display-interpolation.html
(live test)
(source)
display-math-on-non-mathml-elements.html
(live test)
(source)
display-math-on-pseudo-elements-001.html
(live test)
(source)
display-math-on-pseudo-elements-002.html
(live test)
(source)
display-none-root-hit-test-crash.html
(live test)
(source)
inheritance.html
(live test)
(source)
display-computed.html
(live test)
(source)
display-invalid.html
(live test)
(source)
display-valid.html
(live test)
(source)
select-4-option-optgroup-display-none.html
(live test)
(source)
textarea-display.html
(live test)
(source)
2.1.
Outer Display Roles for Flow Layout: the
block
,
inline
, and
run-in
keywords
The
<display-outside>
keywords specify the element’s
outer display type
, which is essentially its
principal box’s
role in
flow layout
. They are defined as follows:
block
The element generates a box that is
block-level
when placed in
flow layout
.
[CSS2]
inline
The element generates a box that is
inline-level
when placed in
flow layout
.
[CSS2]
run-in
The element generates an
run-in box
, which is a type of
inline-level box
with special behavior that attempts to merge it into a subsequent block container. See
§ 5 Run-In Layout
for details.
Note:
Outer display types
do affect
replaced elements
.
If a
<display-outside>
value is specified but
<display-inside>
is omitted, the element’s
inner display type
defaults to
flow
.
Tests
display-change-iframe.html
(live test)
(source)
display-change-object-iframe.html
(live test)
(source)
after-content-display-004.xht (visual test)
(source)
anonymous-box-generation-002.xht (visual test)
(source)
background-applies-to-011.xht (visual test)
(source)
background-attachment-applies-to-011.xht (visual test)
(source)
background-color-applies-to-011.xht (visual test)
(source)
background-image-applies-to-011.xht (visual test)
(source)
background-position-applies-to-011.xht (visual test)
(source)
background-repeat-applies-to-011.xht (visual test)
(source)
before-content-display-004.xht (visual test)
(source)
border-applies-to-011.xht (visual test)
(source)
border-bottom-applies-to-011.xht (visual test)
(source)
border-bottom-color-applies-to-011.xht (visual test)
(source)
border-bottom-style-applies-to-011.xht (visual test)
(source)
border-bottom-width-applies-to-011.xht (visual test)
(source)
border-collapse-applies-to-004.xht (visual test)
(source)
border-color-applies-to-011.xht (visual test)
(source)
border-left-applies-to-011.xht (visual test)
(source)
border-left-color-applies-to-011.xht (visual test)
(source)
border-left-style-applies-to-011.xht (visual test)
(source)
border-left-width-applies-to-011.xht (visual test)
(source)
border-right-applies-to-011.xht (visual test)
(source)
border-right-color-applies-to-011.xht (visual test)
(source)
border-right-style-applies-to-011.xht (visual test)
(source)
border-right-width-applies-to-011.xht (visual test)
(source)
border-spacing-applies-to-004.xht (visual test)
(source)
border-style-applies-to-011.xht (visual test)
(source)
border-top-applies-to-011.xht (visual test)
(source)
border-top-color-applies-to-011.xht (visual test)
(source)
border-top-style-applies-to-011.xht (visual test)
(source)
border-top-width-applies-to-011.xht (visual test)
(source)
border-width-applies-to-011.xht (visual test)
(source)
bottom-applies-to-011.xht (visual test)
(source)
caption-side-applies-to-004.xht (visual test)
(source)
clear-applies-to-011.xht (visual test)
(source)
clear-runin-001.xht (visual test)
(source)
color-applies-to-011.xht (visual test)
(source)
counter-increment-applies-to-011.xht
(live test)
(source)
counter-reset-applies-to-011.xht
(live test)
(source)
cursor-applies-to-011.xht (manual test)
(source)
direction-applies-to-011.xht (visual test)
(source)
display-004.xht (visual test)
(source)
empty-cells-applies-to-004.xht (visual test)
(source)
first-line-pseudo-009.xht (visual test)
(source)
float-applies-to-011.xht (visual test)
(source)
font-applies-to-004.xht (visual test)
(source)
font-family-applies-to-004.xht (visual test)
(source)
font-size-applies-to-004.xht (visual test)
(source)
font-style-applies-to-004.xht
(live test)
(source)
font-variant-applies-to-004.xht
(live test)
(source)
font-weight-applies-to-004.xht
(live test)
(source)
height-applies-to-011.xht
(live test)
(source)
left-applies-to-011.xht (visual test)
(source)
letter-spacing-applies-to-004.xht
(live test)
(source)
line-height-applies-to-011.xht (visual test)
(source)
list-style-applies-to-011.xht
(live test)
(source)
list-style-image-applies-to-011.xht (visual test)
(source)
list-style-position-applies-to-011.xht (visual test)
(source)
list-style-type-applies-to-011.xht
(live test)
(source)
margin-applies-to-011.xht (visual test)
(source)
margin-bottom-applies-to-011.xht (visual test)
(source)
margin-left-applies-to-011.xht (visual test)
(source)
margin-right-applies-to-011.xht (visual test)
(source)
margin-top-applies-to-011.xht (visual test)
(source)
max-height-applies-to-011.xht
(live test)
(source)
max-width-applies-to-011.xht
(live test)
(source)
min-height-applies-to-011.xht
(live test)
(source)
min-width-applies-to-011.xht
(live test)
(source)
outline-applies-to-011.xht (visual test)
(source)
outline-color-applies-to-011.xht (visual test)
(source)
outline-style-applies-to-011.xht (visual test)
(source)
outline-width-applies-to-011.xht (visual test)
(source)
overflow-applies-to-011.xht (visual test)
(source)
padding-applies-to-011.xht (visual test)
(source)
padding-bottom-applies-to-011.xht (visual test)
(source)
padding-left-applies-to-011.xht (visual test)
(source)
padding-right-applies-to-011.xht (visual test)
(source)
padding-top-applies-to-011.xht (visual test)
(source)
position-applies-to-011.xht (visual test)
(source)
quotes-applies-to-011.xht
(live test)
(source)
right-applies-to-011.xht (visual test)
(source)
run-in-001.xht (visual test)
(source)
run-in-002.xht (visual test)
(source)
run-in-003.xht (visual test)
(source)
run-in-004.xht (visual test)
(source)
run-in-005.xht (visual test)
(source)
run-in-006.xht (visual test)
(source)
run-in-007.xht (visual test)
(source)
run-in-008.xht (visual test)
(source)
run-in-009.xht (visual test)
(source)
run-in-010.xht (visual test)
(source)
run-in-011.xht (visual test)
(source)
run-in-012.xht (visual test)
(source)
run-in-013.xht (visual test)
(source)
run-in-abspos-between-001.xht
(live test)
(source)
run-in-abspos-between-002.xht
(live test)
(source)
run-in-abspos-between-003.xht
(live test)
(source)
run-in-basic-001.xht
(live test)
(source)
run-in-basic-002.xht
(live test)
(source)
run-in-basic-003.xht
(live test)
(source)
run-in-basic-004.xht
(live test)
(source)
run-in-basic-005.xht
(live test)
(source)
run-in-basic-006.xht
(live test)
(source)
run-in-basic-007.xht
(live test)
(source)
run-in-basic-008.xht
(live test)
(source)
run-in-basic-009.xht
(live test)
(source)
run-in-basic-010.xht
(live test)
(source)
run-in-basic-011.xht
(live test)
(source)
run-in-basic-012.xht
(live test)
(source)
run-in-basic-013.xht
(live test)
(source)
run-in-basic-014.xht
(live test)
(source)
run-in-basic-015.xht
(live test)
(source)
run-in-basic-016.xht
(live test)
(source)
run-in-basic-017.xht
(live test)
(source)
run-in-basic-018.xht
(live test)
(source)
run-in-block-between-001.xht
(live test)
(source)
run-in-block-between-002.xht
(live test)
(source)
run-in-block-between-003.xht
(live test)
(source)
run-in-breaking-001.xht
(live test)
(source)
run-in-breaking-002.xht
(live test)
(source)
run-in-clear-001.xht
(live test)
(source)
run-in-clear-002.xht
(live test)
(source)
run-in-contains-abspos-001.xht
(live test)
(source)
run-in-contains-block-001.xht
(live test)
(source)
run-in-contains-block-002.xht
(live test)
(source)
run-in-contains-block-003.xht
(live test)
(source)
run-in-contains-block-004.xht
(live test)
(source)
run-in-contains-block-005.xht
(live test)
(source)
run-in-contains-block-inside-inline-001.xht
(live test)
(source)
run-in-contains-block-inside-inline-002.xht
(live test)
(source)
run-in-contains-block-inside-inline-003.xht
(live test)
(source)
run-in-contains-float-001.xht
(live test)
(source)
run-in-contains-inline-001.xht
(live test)
(source)
run-in-contains-inline-002.xht
(live test)
(source)
run-in-contains-inline-003.xht
(live test)
(source)
run-in-contains-inline-004.xht
(live test)
(source)
run-in-contains-inline-005.xht
(live test)
(source)
run-in-contains-inline-006.xht
(live test)
(source)
run-in-contains-inline-007.xht
(live test)
(source)
run-in-contains-inline-block-001.xht
(live test)
(source)
run-in-contains-inline-table-001.xht
(live test)
(source)
run-in-contains-relpos-block-001.xht
(live test)
(source)
run-in-contains-relpos-block-002.xht
(live test)
(source)
run-in-contains-relpos-block-003.xht
(live test)
(source)
run-in-contains-run-in-001.xht
(live test)
(source)
run-in-contains-run-in-002.xht
(live test)
(source)
run-in-contains-run-in-003.xht
(live test)
(source)
run-in-contains-table-001.xht
(live test)
(source)
run-in-contains-table-002.xht
(live test)
(source)
run-in-contains-table-003.xht
(live test)
(source)
run-in-contains-table-caption-001.xht
(live test)
(source)
run-in-contains-table-cell-001.xht
(live test)
(source)
run-in-contains-table-column-001.xht
(live test)
(source)
run-in-contains-table-column-group-001.xht
(live test)
(source)
run-in-contains-table-inside-inline-001.xht
(live test)
(source)
run-in-contains-table-inside-inline-002.xht
(live test)
(source)
run-in-contains-table-inside-inline-003.xht
(live test)
(source)
run-in-contains-table-row-001.xht
(live test)
(source)
run-in-contains-table-row-group-001.xht
(live test)
(source)
run-in-display-none-between-001.xht
(live test)
(source)
run-in-display-none-between-002.xht
(live test)
(source)
run-in-display-none-between-003.xht
(live test)
(source)
run-in-fixedpos-between-001.xht
(live test)
(source)
run-in-fixedpos-between-002.xht
(live test)
(source)
run-in-fixedpos-between-003.xht
(live test)
(source)
run-in-float-between-001.xht
(live test)
(source)
run-in-float-between-002.xht
(live test)
(source)
run-in-float-between-003.xht
(live test)
(source)
run-in-inherit-001.xht
(live test)
(source)
run-in-inheritance-001.xht (visual test)
(source)
run-in-inline-between-001.xht
(live test)
(source)
run-in-inline-between-002.xht
(live test)
(source)
run-in-inline-between-003.xht
(live test)
(source)
run-in-inline-block-between-001.xht
(live test)
(source)
run-in-inline-block-between-002.xht
(live test)
(source)
run-in-inline-block-between-003.xht
(live test)
(source)
run-in-inline-table-between-001.xht
(live test)
(source)
run-in-inline-table-between-002.xht
(live test)
(source)
run-in-inline-table-between-003.xht
(live test)
(source)
run-in-linebox-001.xht (visual test)
(source)
run-in-linebox-002.xht (visual test)
(source)
run-in-listitem-between-001.xht
(live test)
(source)
run-in-listitem-between-002.xht
(live test)
(source)
run-in-listitem-between-003.xht
(live test)
(source)
run-in-relpos-between-001.xht
(live test)
(source)
run-in-relpos-between-002.xht
(live test)
(source)
run-in-relpos-between-003.xht
(live test)
(source)
run-in-replaced-001.xht
(live test)
(source)
run-in-restyle-001.xht
(live test)
(source)
run-in-restyle-002.xht
(live test)
(source)
run-in-restyle-003.xht
(live test)
(source)
run-in-run-in-between-001.xht
(live test)
(source)
run-in-run-in-between-002.xht
(live test)
(source)
run-in-run-in-between-003.xht
(live test)
(source)
run-in-run-in-between-004.xht
(live test)
(source)
run-in-run-in-between-005.xht
(live test)
(source)
run-in-run-in-between-006.xht
(live test)
(source)
run-in-run-in-between-007.xht
(live test)
(source)
run-in-run-in-between-008.xht
(live test)
(source)
run-in-table-between-001.xht
(live test)
(source)
run-in-table-between-002.xht
(live test)
(source)
run-in-table-between-003.xht
(live test)
(source)
run-in-table-cell-between-001.xht
(live test)
(source)
run-in-table-cell-between-002.xht
(live test)
(source)
run-in-table-cell-between-003.xht
(live test)
(source)
run-in-table-row-between-001.xht
(live test)
(source)
run-in-table-row-between-002.xht
(live test)
(source)
run-in-table-row-between-003.xht
(live test)
(source)
run-in-text-between-001.xht
(live test)
(source)
run-in-text-between-002.xht
(live test)
(source)
run-in-text-between-003.xht
(live test)
(source)
run-in-text-between-004.xht
(live test)
(source)
run-in-text-between-005.xht
(live test)
(source)
table-anonymous-block-001.xht (visual test)
(source)
table-layout-applies-to-004.xht (visual test)
(source)
text-align-applies-to-004.xht (visual test)
(source)
text-decoration-applies-to-004.xht
(live test)
(source)
text-indent-applies-to-004.xht (visual test)
(source)
text-transform-applies-to-004.xht
(live test)
(source)
top-applies-to-011.xht (visual test)
(source)
unicode-bidi-applies-to-011.xht (visual test)
(source)
vertical-align-applies-to-011.xht (visual test)
(source)
visibility-applies-to-011.xht (visual test)
(source)
white-space-applies-to-004.xht (visual test)
(source)
width-applies-to-011.xht
(live test)
(source)
word-spacing-applies-to-004.xht (visual test)
(source)
z-index-applies-to-011.xht (visual test)
(source)
2.2.
Inner Display Layout Models: the
flow
,
flow-root
,
table
,
flex
,
grid
, and
ruby
keywords
The
<display-inside>
keywords specify the element’s
inner display type
, which defines the type of formatting context that lays out its contents (assuming it is a
non-replaced element
). They are defined as follows:
flow
The element lays out its contents using
flow layout
(
block-and-inline layout
).
If its
outer display type
is
inline
or
run-in
, and it is participating in a
block
or
inline
formatting context, then it generates an
inline box
.
Otherwise it generates a
block container
box.
Depending on the value of other properties (such as
position
,
float
, or
overflow
) and whether it is itself participating in a block or inline
formatting context
, it either establishes a new
block formatting context
for its contents or integrates its contents into its parent
formatting context
. See
CSS2.1 Chapter 9
.
[CSS2]
A
block container
that establishes a new
block formatting context
is considered to have a used
inner display type
of
flow-root
.
flow-root
The element generates a
block container
box, and lays out its contents using
flow layout
. It always establishes a new
block formatting context
for its contents.
[CSS2]
table
The element generates a principal
table wrapper box
that establishes a
block formatting context
, and which contains an additionally-generated
table grid box
that establishes a
table formatting context
.
[CSS2]
flex
The element generates a principal
flex container
box and establishes a
flex formatting context
.
[CSS-FLEXBOX-1]
grid
The element generates a principal
grid container
box, and establishes a
grid formatting context
.
[CSS-GRID-1]
(Grids using
subgrid
might not generate a new
grid formatting context
; see
[CSS-GRID-2]
for details.)
ruby
The element generates a
ruby container
box and establishes a
ruby formatting context
in addition to integrating its base-level contents into its parent
formatting context
(if it is inline) or generating a wrapper box of the appropriate
outer display type
(if it is not).
[CSS-RUBY-1]
If a
<display-inside>
value is specified but
<display-outside>
is omitted, the element’s
outer display type
defaults to
block
—​except for
ruby
, which defaults to
inline
.
Tests
display-flow-root-001.html
(live test)
(source)
display-flow-root-002.html
(live test)
(source)
display-none-inline-img.html
(live test)
(source)
2.3.
Generating Marker Boxes: the
list-item
keyword
The
list-item
keyword causes the element to generate a
::marker
pseudo-element
[CSS-PSEUDO-4]
with the content specified by its
list-style
properties (
CSS 2.1§12.5 Lists
)
[CSS2]
together with a principal box of the specified type for its own contents.
If no
inner display type
value is specified, the principal box’s
inner display type
defaults to
flow
. If no
outer display type
value is specified, the principal box’s
outer display type
defaults to
block
.
Note:
In this level, as restricted in the grammar, list-items are limited to the
Flow Layout
display types (
block
/
inline
/
run-in
with
flow
/
flow-root
inner types). This restriction may be relaxed in a future level of this module.
Tests
display-flow-root-list-item-001.html
(live test)
(source)
display-list-item-height-after-dom-change.html
(live test)
(source)
2.4.
Layout-Internal Display Types: the
table-*
and
ruby-*
keywords
Some layout models, such as
table
and
ruby
, have a complex internal structure, with several different roles that their children and descendants can fill. This section defines those “
layout-internal
”
display
values, which only have meaning within that particular layout mode.
Unless otherwise specified, both the
inner display type
and the
outer display type
of elements using these
display
values are set to the given keyword.
When the
display
property of a
replaced element
computes to one of the
layout-internal
values, it is handled as having a used value of
inline
. White space collapsing and anonymous box generation must happen around those replaced elements based on that
inline
value, as if they never had a
layout-internal
display value applied to them.
Authors should not assign a
layout-internal
display value to
replaced elements
.
The
<display-internal>
keywords are defined as follows:
table-row-group
,
table-header-group
,
table-footer-group
,
table-row
,
table-cell
,
table-column-group
,
table-column
The element is an
internal table element
. It generates the appropriate
internal table box
which participates in a
table formatting context
. See
CSS2§17.2
[CSS2]
.
table-cell
boxes have a
flow-root
inner display type
.
table-caption
The element generates a
table caption box
, which is a
block box
with special behavior with respect to table and table wrapper boxes. See
CSS2§17.2
[CSS2]
.
table-caption
boxes have a
flow-root
inner display type
.
ruby-base
,
ruby-text
,
ruby-base-container
,
ruby-text-container
The element is an
internal ruby element
. It generates the appropriate
internal ruby box
which participates in a
ruby formatting context
.
[CSS-RUBY-1]
ruby-base
and
ruby-text
have a
flow
inner display type
.
Boxes with layout-specific display types generate anonymous wrapper boxes around themselves when placed in an incompatible parent, as defined by their respective specifications.
For example, Table Layout requires that a
table-cell
box must have a
table-row
parent box.
If it is misparented, like so:
<
div
style
=
"display:block;"
>
<
div
style
=
"display:table-cell"
>
...
</
div
>
</
div
>
It will generate wrapper boxes around itself, producing a structure like:
block box └anonymous table box └anonymous table-row-group box └anonymous table-row box └table-cell box
Even if the parent is another
internal table element
, if it’s not the
correct
one, wrapper boxes will be generated. For example, in the following markup:
<
div
style
=
"display:table;"
>
<
div
style
=
"display:table-row"
>
<
div
style
=
"display:table-cell"
>
...
</
div
>
</
div
>
</
div
>
Anonymous wrapper box generation will produce:
table box └anonymous table-row-group box └table-row box └table-cell box
This "fix-up" ensures that table layout has a predictable structure to operate on.
Tests
This section lacks tests.
2.5.
Box Generation: the
none
and
contents
keywords
While
display
can control the
types
of boxes an element will generate, it can also control whether an element will generate any boxes at all.
The
<display-box>
keywords are defined as follows:
contents
The element itself does not generate any boxes, but its children and pseudo-elements still generate
boxes
and
text sequences
as normal. For the purposes of box generation and layout, the element must be treated as if it had been replaced in the
element tree
by its contents (including both its source-document children and its pseudo-elements, such as
::before
and
::after
pseudo-elements, which are generated before/after the element’s children as normal).
Note:
As only the box tree is affected, any semantics based on the document tree, such as selector-matching, event handling, and property
inheritance
, are not affected.
As of writing, however,
this is not implemented correctly in major browsers
, so using this feature on the Web must be done with care as it can prevent accessibility tools from accessing the element’s semantics.
This value computes to
display: none
on replaced elements and other elements whose rendering is not entirely controlled by CSS; see
Appendix B: Effects of display: contents on Unusual Elements
for details.
Note:
Replaced elements and form controls are treated specially because removing only the element’s own generating box is a more-or-less undefined operation. As this behavior may be refined if use cases (and more precise rendering models) develop, authors should use
display: none
rather than
display: contents
on such elements for forward-compatibility.
none
The
element
and its descendants generate no
boxes
or
text sequences
.
Similarly, if a
text node
is defined to behave as
display: none
, it generates no
text sequences
.
Elements with either of these values do not have
inner
or
outer display types
, because they don’t generate any boxes at all.
Note:
As these values cause affected elements to not generate a box, anonymous box generation rules will ignore the elided elements entirely, as if they did not exist in the box tree.
Markup-based relationships, however, are not affected by these values, as they are solely rendering-time effects.
For example, although they may affect which table cell
appears
in a column, they do not affect which table cell is associated with a particular column
element
. Similarly, they cannot affect which HTML
summary
element is associated with a particular table or whether a
legend
is considered to be labelling the contents of a particular
fieldset
.
Tests
display-contents-001-crash.html
(live test)
(source)
display-contents-alignment-001.html
(live test)
(source)
display-contents-alignment-002.html
(live test)
(source)
display-contents-before-after-001.html
(live test)
(source)
display-contents-before-after-002.html
(live test)
(source)
display-contents-before-after-003.html
(live test)
(source)
display-contents-block-001.html
(live test)
(source)
display-contents-block-002.html
(live test)
(source)
display-contents-blockify-dynamic.html
(live test)
(source)
display-contents-button.html
(live test)
(source)
display-contents-computed-style.html
(live test)
(source)
display-contents-details-001.html
(live test)
(source)
display-contents-details.html
(live test)
(source)
display-contents-dynamic-before-after-001.html
(live test)
(source)
display-contents-dynamic-before-after-first-letter-001.html
(live test)
(source)
display-contents-dynamic-fieldset-legend-001.html
(live test)
(source)
display-contents-dynamic-flex-001-inline.html
(live test)
(source)
display-contents-dynamic-flex-001-none.html
(live test)
(source)
display-contents-dynamic-flex-002-inline.html
(live test)
(source)
display-contents-dynamic-flex-002-none.html
(live test)
(source)
display-contents-dynamic-flex-003-inline.html
(live test)
(source)
display-contents-dynamic-flex-003-none.html
(live test)
(source)
display-contents-dynamic-generated-content-fieldset-001.html
(live test)
(source)
display-contents-dynamic-inline-flex-001-inline.html
(live test)
(source)
display-contents-dynamic-inline-flex-001-none.html
(live test)
(source)
display-contents-dynamic-list-001-inline.html
(live test)
(source)
display-contents-dynamic-list-001-none.html
(live test)
(source)
display-contents-dynamic-multicol-001-inline.html
(live test)
(source)
display-contents-dynamic-multicol-001-none.html
(live test)
(source)
display-contents-dynamic-pseudo-insertion-001.html
(live test)
(source)
display-contents-dynamic-table-001-inline.html
(live test)
(source)
display-contents-dynamic-table-001-none.html
(live test)
(source)
display-contents-dynamic-table-002-inline.html
(live test)
(source)
display-contents-dynamic-table-002-none.html
(live test)
(source)
display-contents-fieldset-002.html
(live test)
(source)
display-contents-fieldset-nested-legend.html
(live test)
(source)
display-contents-fieldset.html
(live test)
(source)
display-contents-first-letter-001.html
(live test)
(source)
display-contents-first-letter-002.html
(live test)
(source)
display-contents-first-line-001.html
(live test)
(source)
display-contents-first-line-002.html
(live test)
(source)
display-contents-flex-001.html
(live test)
(source)
display-contents-flex-002.html
(live test)
(source)
display-contents-flex-003.html
(live test)
(source)
display-contents-float-001.html
(live test)
(source)
display-contents-focusable-001.html
(live test)
(source)
display-contents-inline-001.html
(live test)
(source)
display-contents-inline-002.html
(live test)
(source)
display-contents-inline-flex-001.html
(live test)
(source)
display-contents-line-height.html
(live test)
(source)
display-contents-list-001.html
(live test)
(source)
display-contents-multicol-001.html
(live test)
(source)
display-contents-oof-001.html
(live test)
(source)
display-contents-oof-002.html
(live test)
(source)
display-contents-parsing-001.html
(live test)
(source)
display-contents-pseudo-click-target.html
(live test)
(source)
display-contents-root-background.html
(live test)
(source)
display-contents-shadow-dom-1.html
(live test)
(source)
display-contents-shadow-host-whitespace.html
(live test)
(source)
display-contents-sharing-001.html
(live test)
(source)
display-contents-slot-attach-whitespace.html
(live test)
(source)
display-contents-state-change-001.html
(live test)
(source)
display-contents-suppression-dynamic-001.html
(live test)
(source)
display-contents-svg-anchor-child.html
(live test)
(source)
display-contents-svg-elements.html
(live test)
(source)
display-contents-svg-switch-child.html
(live test)
(source)
display-contents-table-001.html
(live test)
(source)
display-contents-table-002.html
(live test)
(source)
display-contents-td-001.html
(live test)
(source)
display-contents-text-inherit-002.html
(live test)
(source)
display-contents-text-inherit.html
(live test)
(source)
display-contents-text-only-001.html
(live test)
(source)
display-contents-tr-001.html
(live test)
(source)
display-contents-unusual-html-elements-none.html
(live test)
(source)
display-contents-whitespace-inside-inline.html
(live test)
(source)
2.6.
Precomposed Inline-level Display Values
CSS level 2 used a single-keyword syntax for
display
, requiring separate keywords for block-level and inline-level variants of the same layout mode. These
<display-legacy>
keywords map as follows:
inline-block
Computes to
inline flow-root
.
inline-table
Computes to
inline table
.
inline-flex
Computes to
inline flex
.
inline-grid
Computes to
inline grid
.
Note:
Although these keywords and their equivalents compute to the same value, their
specified values
remain distinct.
Note:
The
getComputedStyle()
serialization rules will always output these precomposed keywords rather than the equivalent two-keyword pairs due to the
shortest, most backwards-compatible serialization principle
.
Tests
This section lacks tests.
2.7.
Automatic Box Type Transformations
Some layout effects require
blockification
or
inlinification
of the box type, which sets the box’s
computed
outer display type
to
block
or
inline
(respectively). (This has no effect on
display types
that generate no box at all, such as
none
or
contents
.) Additionally:
If a
block box
(
block flow
) is
inlinified
, its
inner display type
is set to
flow-root
so that it remains a block container.
If an
inline box
(
inline flow
) is
inlinified
, it recursively
inlinifies
all of its
in-flow
children, so that no block-level descendants break up the inline formatting context in which it participates.
For legacy reasons, if an
inline block box
(
inline flow-root
) is
blockified
, it becomes a
block
box (losing its
flow-root
nature). For consistency, a
run-in flow-root
box also
blockifies
to a
block
box.
If a
layout-internal
box is
blockified
, its
inner display type
converts to
flow
so that it becomes a
block container
.
Inlinification
has no effect on
layout-internal
boxes. (However, placement in such an inline context will typically cause them to be wrapped in an appropriately-typed anonymous inline-level box.)
Note:
There are two methods used to fix up box types when a box is mismatched to its context. One is transformation of the
computed value
of
display
, such as
blockification
and
inlinification
described here. The other, taking place during
box tree construction
(after computed values have been determined), is the creation of intermediary anonymous boxes, such as happens in
tables
,
ruby
, and
flow
layout.
Some examples of computed-value fixup include:
Absolute positioning or floating an element
blockifies
the box’s display type.
[CSS2]
Containment in a
ruby container
inlinifies
the box’s display type, as described in
[CSS-RUBY-1]
.
A parent with a
grid
or
flex
display
value
blockifies
the box’s display type.
[CSS-GRID-1]
[CSS-FLEXBOX-1]
2.8.
The Root Element’s Principal Box
The
root element
’s display type is always
blockified
, and its
principal box
always establishes an
independent formatting context
. This box’s
containing block
is the
initial containing block
.
Additionally, a
display
of
contents
computes to
block
on the root element.
3.
Display Order: the
order
property
Name:
order
Value:
<integer>
Initial:
0
Applies to:
flex items
and
grid items
Inherited:
no
Percentages:
n/a
Computed value:
specified integer
Canonical order:
per grammar
Animation type:
by computed value type
Tests
flexible-order.html
(live test)
(source)
Boxes are generally displayed and laid out in the same order as they appear in the source document. In some
formatting contexts
, the
order
property can be used to rearrange the order of boxes to deliberately create a divergence of the logical order of elements and their spatial arrangement on the 2D visual canvas. (See
§ 3.1 Reordering and Accessibility
.)
Specifically, the
order
property controls the order in which
flex items
or
grid items
appear within their container, by assigning them to ordinal groups. It takes a single
<integer>
value, which specifies which ordinal group the item belongs to.
Here’s an example of a catalog item card which has a title, a photo, and a description. Within each entry, the source document content is ordered logically with the title first, followed by the description and the photo. This provides a sensible ordering for speech rendering and in non-CSS browsers. For a more compelling visual presentation, however,
order
is used to pull the image up from later in the content to the top of the card.
article.sale-item
{
display
:
flex
;
flex-flow
:
column
;
}
article.sale-item > img
{
order
:
-1
;
/* Shift image before other content (in layout order) */
align-self: center
;
}
<
article
class
=
"sale-item"
>
<
h3
>
Computer Starter Kit
</
h3
>
<
p
>
This is the best computer money can buy, if you don’t have much money.
<
ul
>
<
li
>
Computer
<
li
>
Monitor
<
li
>
Keyboard
<
li
>
Mouse
</
ul
>
<
img
src
=
"images/computer.jpg"
alt
=
"You get: a white desktop computer with matching peripherals."
width
=
"250"
height
=
"188"
>
</
article
>
Computer Starter Kit
This is the best computer money can buy, if you don’t have much money.
Computer
Monitor
Keyboard
Mouse
An example rendering of the code above.
Flex
and
grid containers
lay out their contents in
order-modified document order
, starting from the lowest numbered ordinal group and going up. Items with the same ordinal group are laid out in the order they appear in the source document. This also affects the
painting order
[CSS2]
, exactly as if the
flex
/
grid items
were reordered in the source document. Absolutely-positioned children of a
flex
/
grid container
are treated as having
order: 0
for the purpose of determining their painting order relative to
flex
/
grid items
.
Unless otherwise specified by a future specification, this property has no effect on boxes that are not
flex items
or
grid items
.
3.1.
Reordering and Accessibility
The
order
property
does not
affect ordering in non-visual media (such as
speech
). Likewise,
order
does not affect the default traversal order of sequential navigation modes (such as cycling through links, see e.g.
tabindex
[HTML]
).
Authors
must
use
order
only for spatial, not logical, reordering of content. Style sheets that use
order
to perform logical reordering are non-conforming.
Note:
This is so that non-visual media and non-CSS UAs, which typically present content linearly, can rely on a logical source order, while
order
is used to tailor the layout order. (Since visual perception is two-dimensional and non-linear, the desired layout order is not always logical.)
In order to preserve the author’s intended ordering in all presentation modes, authoring tools—​including WYSIWYG editors as well as Web-based authoring aids—​must reorder the underlying document source and not use
order
to perform reordering unless the author has explicitly indicated that the spatial order should be
out-of-sync
with the underlying document order (which determines speech and navigation order).
For example, a tool might offer both drag-and-drop reordering of flex items as well as handling of media queries for alternate layouts per screen size range.
Since most of the time, reordering should affect all screen ranges as well as navigation and speech order, the tool would perform drag-and-drop reordering at the DOM layer. In some cases, however, the author may want different layouts per screen size. The tool could offer this functionality by using
order
together with media queries, but also tie the smallest screen size’s ordering to the underlying DOM order (since this is most likely to be a logical linear presentation order) while using
order
to define the visual presentation order in other size ranges.
This tool would be conformant, whereas a tool that only ever used
order
to handle drag-and-drop reordering (however convenient it might be to implement it that way) would be non-conformant.
Note:
User agents, including browsers, accessible technology, and extensions, may offer spatial navigation features. This section does not preclude respecting the
order
property when determining element ordering in such spatial navigation modes; indeed it would need to be considered for such a feature to work. But
order
is not the only (or even the primary) CSS property that would need to be considered for such a spatial navigation feature. A well-implemented spatial navigation feature would need to consider all the layout features of CSS that modify spatial relationships.
4.
Invisibility: the
visibility
property
Name:
visibility
Value:
visible
|
hidden
|
collapse
Initial:
visible
Applies to:
all elements
Inherited:
yes
Percentages:
N/A
Computed value:
as specified
Canonical order:
per grammar
Animation type:
discrete
Media:
visual
The
visibility
property specifies whether the box is rendered.
Invisible
boxes still affect layout. (Set the
display
property to
none
to suppress box generation altogether.). Values have the following meanings:
visible
The generated box is visible, as normal.
hidden
Any boxes generated by the element are
invisible
. Descendants of the element can, however, be visible if they have
visibility: visible
.
collapse
Indicates that the box is
collapsed
, which can cause it to take up less space than otherwise in a formatting-context–specific way. See
dynamic row and column effects in tables
[CSS2]
and
collapsed flex items
in flex layout
[CSS-FLEXBOX-1]
. In all other cases, however, (i.e. unless otherwise specified) this simply makes the box
invisible
, just like
hidden
.
Note:
Currently, many user agents and/or accessibility tools don’t correctly implement the accessibility implications of visible elements with semantic relationships to
invisible
elements, so, for example, making parent elements with special roles (such as table rows)
invisible
while leaving child elements with special roles (such as table cells) visible can be problematic for users of those tools. Authors should avoid creating these situations until the tooling situation improves.
Invisible boxes
are not rendered (as if they were fully transparent), cannot be interacted with (and behave as if they had
pointer-events: none
), are removed from navigation (similar to
display: none
), and are also not rendered to speech (except when
speak
is
always
[CSS-SPEECH-1]
). However, as with
display: contents
, their semantic role as a container is not affected, to ensure that any
visible
descendants are properly interpreted.
Note:
If
speak
is
always
, an otherwise
invisible
box
is
rendered to speech, and may be interacted with using non-visual/spatial methods.
While temporarily hiding things with
display: none
is often sufficient, doing so removes the elements from layout entirely, possibly causing unwanted movement or reflow of the page when an element is hidden or shown.
visibility: hidden
can instead be used to keep the page’s layout stable as something is hidden and displayed.
For example, here is a (deliberately simplified) possible implementation of a "spoiler" element that can be revealed by clicking on the hidden text:
<
p
>
The symbolism earlier in the movie becomes obvious at the end, when it's revealed that
<
spoiler-text
><
span
>
Luke is his own father
</
span
></
spoiler-text
>
, making the wizard's cryptic riddles meaningful.
<
style
>
spoiler-text
{
border-bottom
:
1
px
solid
;
}
spoiler-text
>
span
{
visibility
:
hidden
;
}
spoiler-text
.
shown
>
span
{
visibility
:
visible
;
}
</
style
>
<
script
>
[...
document
.
querySelectorAll
(
"spoiler-text"
)].
forEach
(
el
=>{
el
.
addEventListener
(
"click"
,
e
=>
el
.
classList
.
toggle
(
"shown"
));
});
</
script
>
This example is deliberately significantly simplified. It is missing a number of accessibility and UX features that a well-designed spoiler element would have to show off the
visibility
usage more plainly. Don’t copy this code for a real site.
Tests
This section lacks tests.
5.
Run-In Layout
A
run-in box
is a box that
merges into
a block that comes after it, inserting itself at the beginning of that block’s inline-level content. This is useful for formatting compact headlines, definitions, and other similar things, where the appropriate DOM structure is to have a headline preceding the following prose, but the desired display is an inline headline laying out with the text.
For example, dictionary definitions are often formatted so that the word is inline with the definition:
<dl class='dict'> <dt>dictionary <dd>a book that lists the words of a language in alphabetical order and gives their meaning, or that gives the equivalent words in a different language. <dt>glossary <dd>an alphabetical list of terms or words found in or relating to a specific subject, text, or dialect, with explanations; a brief dictionary. </dl> <style> .dict > dt { display: run-in; } .dict > dt::after { content: ": " } </style>
Which is formatted as:
dictionary:
a book that lists the words of a language in alphabetical order and explains their meaning.
glossary:
an alphabetical list of terms or words found in or relating to a specific subject, text, or dialect, with explanations; a brief dictionary.
A
run-in box
behaves exactly as any other
inline-level box
, except:
A
run-in box
with a
flow
inner display type
inlinifies
its contents.
If a
run-in sequence
is immediately followed by a block box that does not establish a new
block formatting context
, it is inserted as direct children of that block box: after its
::marker
pseudo-element’s boxes (if any), but preceding any other boxes generated by the contents of the block (including the box generated by the
::before
pseudo-element, if any). This re-parenting recurses if possible (so that the run-in effectively becomes part of the deepest subsequent “paragraph” in its formatting context, collecting newly-adjacent run-ins as it goes).
The reparented content is then formatted as if originally parented there.
Note that only layout is affected, not inheritance, because property inheritance for non-anonymous boxes is based only on the element tree.
Otherwise (if the
run-in sequence
is
not
followed by such a block), an anonymous block box is generated around the
run-in sequence
and all immediately following inline-level content (up to, but not including, the next
run-in sequence
, if any).
A
run-in sequence
is a maximal sequence of consecutive sibling
run-in boxes
and intervening
white space
and/or
out-of-flow
boxes.
Note:
This statement implies that out-of-flow boxes are reparented if they are between two run-in boxes. Another alternative would be to leave behind the intervening out-of-flow boxes, or to have out-of-flow boxes impede the running-in of earlier boxes. Implementers and authors are encouraged to contact the CSSWG if they have a preferred behavior, as this one was picked somewhat at random.
This fixup occurs before the anonymous block and inline box fixup described in
CSS2§9.2
, and affects the determination of the
first formatted line
of the affected elements as if the
run-in sequence
were originally in its final location in the box tree.
Note:
As the earliest run-in represents the first text on the
first formatted line
of its containing block, a
::first-letter
pseudo-element applied to that block element selects the first letter of the run-in, rather than the first letter of its own contents.
Note:
This run-in model is slightly different from the one proposed in earlier revisions of
[CSS2]
.
Appendix A: Glossary
The following terms are defined here for convenience:
root element
The
element
at the root of the
document tree
. In a
document tree
produced under the DOM, this is the
document element
; in HTML it is the
html
element.
[DOM]
[HTML]
principal box
When an
element
generates one or more
boxes
, one of them is the
principal box
, which contains its descendant boxes and generated content, and is also the box involved in any positioning scheme.
Some elements may generate additional boxes in addition to the principal box (such as
list-item
elements, which generate an additional marker box, or
table
elements, which generate a
principal
table wrapper box
and an additional
table grid box
). These additional boxes are placed with respect to the principal box.
inline-level
Content that participates in inline layout. Specifically, inline-level boxes and
text sequences
.
block-level
Content that participates in
block layout
. Specifically, block-level boxes.
inline box
A non-replaced
inline-level
box whose
inner display type
is
flow
. The contents of an inline box participate in the same inline formatting context as the inline box itself.
inline
Used as a shorthand for
inline box
or
inline-level box
where unambiguous, or as an adjective meaning
inline-level
. The latter usage is deprecated.
atomic inline
An inline-level box that is replaced (such as an image) or that establishes a new formatting context (such as an
inline-block
or
inline-table
) and cannot split across lines (as
inline boxes
and
ruby containers
can).
Any inline-level box whose
inner display type
is not
flow
establishes a new formatting context of the specified
inner display type
.
block container
A block container either contains only inline-level boxes participating in an
inline formatting context
, or contains only block-level boxes participating in a
block formatting context
(possibly generating anonymous block boxes to ensure this constraint, as defined in
CSS2§9.2.1.1
).
A block container that contains only inline-level content establishes a new
inline formatting context
. The element then also generates a
root inline box
which wraps all of its inline content.
Note, this
root inline box
concept effectively replaces the "anonymous inline element" concept introduced in
CSS2§9.2.2.1
.
A block container establishes a new
block formatting context
if its parent formatting context is
not
a
block formatting context
; otherwise, when participating in a
block formatting context
itself, it either establishes a new
block formatting context
for its contents or continues the one in which it participates, as determined by the constraints of other properties (such as
overflow
or
align-content
).
Note:
A block container box can both establish a block formatting context
and
an inline formatting context simultaneously.
block box
A
block-level box
that is also a
block container
.
Note:
Not all
block container
boxes are
block-level
boxes: non-replaced
inline blocks
and non-replaced table cells, for example, are block containers but not block-level boxes. Similarly, not all block-level boxes are block containers: block-level replaced elements (
display: block
) and flex containers (
display: flex
), for example, are not block containers.
block
Used as a shorthand for
block box
,
block-level box
, or
block container box
, where unambiguous.
replaced element
An element whose content is outside the scope of the CSS formatting model, such as an image or embedded document.
For example, the content of the HTML
img
element is often replaced by the image that its
src
attribute designates.
Replaced elements often have
natural dimensions
. For example, a bitmap image has a natural width and a natural height specified in absolute units (from which the natural ratio can obviously be determined). On the other hand, other objects may not have any natural dimensions (for example, a blank HTML document). See
[css-images-3]
.
User agents may consider a
replaced element
to not have any
natural dimensions
if it is believed that those dimensions could leak sensitive information to a third party. For example, if an HTML document changed natural size depending on the user’s bank balance, then the UA might want to act as if that resource had no
natural dimensions
.
The content of replaced elements is not considered in the CSS formatting model; however, their
natural dimensions
are used in various layout calculations. Replaced elements always establish an
independent formatting context
.
A
non-replaced
element is one that is not
replaced
, i.e. whose rendering is dictated by the CSS model.
containing block
A rectangle that forms the basis of sizing and positioning for the
boxes
associated with it. Notably, a
containing block
is
not a
box
(it is a rectangle), however it is often derived from the dimensions of a
box
. Each
box
is given a position with respect to its
containing block
, but it is not confined by this
containing block
; it can
overflow
. The phrase “a box’s containing block” means “the
containing block
in which the box lives,” not the one it generates.
In general, the
edges
of a
box
act as the
containing block
for descendant boxes; we say that a box “establishes” the
containing block
for its descendants. If properties of a
containing block
are referenced, they reference the values on the
box
that generated the
containing block
. (For the
initial containing block
, values are taken from the root element unless otherwise specified.)
See
[CSS2]
Section 9.1.2
and
Section 10.1
and
CSS Positioned Layout 3
§ 2.1 Containing Blocks of Positioned Boxes
for details.
containing block chain
A sequence of successive
containing blocks
that form an ancestor-descendant chain through the
containing block
relation. For example, an
inline box
’s containing block is the content box of its closest
block container
ancestor; if that block container is an
in-flow
block
, then
its
containing block is formed by its parent
block container
; if that grandparent
block container
is
absolutely positioned
, then
its
containing block is the padding edges of its closest
positioned
ancestor (not necessarily its parent), and so on up to the
initial containing block
.
initial containing block
The
containing block
of the
root element
. The
initial containing block
establishes a
block formatting context
, and takes the
principal writing mode
of the document (see
CSS Writing Modes 4
§ 8.1 Propagation to the Initial Containing Block
). In
continuous media
, it has the dimensions of the
small viewport size
and is anchored at the canvas origin. In
paged media
, see
[CSS-PAGE-3]
for its position and dimensions.
formatting context
A
formatting context
is the environment into which a set of related boxes are laid out. Different
formatting contexts
lay out their boxes according to different rules. For example, a
flex formatting context
lays out boxes according to the
flex layout
rules
[CSS-FLEXBOX-1]
, whereas a
block formatting context
lays out boxes according to the block-and-inline layout rules
[CSS2]
. Additionally, some types of
formatting contexts
interleave and co-exist: for example, an
inline formatting context
exists within and interacts with the
block formatting context
of the element that establishes it, and a
ruby container
overlays a
ruby formatting context
over the
inline formatting context
in which its
ruby base container
participates.
A box either establishes a new
independent formatting context
or continues the
formatting context
of its containing block. In some cases, it might additionally establish a new (non-independent) co-existing formatting context. Unless otherwise specified, however, establishing a new
formatting context
creates an
independent formatting context
. The type of formatting context established by the box is determined by its
inner display type
. E.g. a
grid container
establishes a new
grid formatting context
, a
ruby container
establishes a new
ruby formatting context
, and a
block container
can establish a new
block formatting context
and/or a new
inline formatting context
. See the
display
property.
independent formatting context
When a box establishes an independent formatting context (whether that
formatting context
is of the same type as its parent or not), it essentially creates a new, independent layout environment: except through the sizing of the box itself, the layout of its descendants is (generally) not affected by the rules and contents of the
formatting context
outside the box, and vice versa.
For example, in a
block formatting context
, floated boxes affect the layout of surrounding boxes. But their effects do not escape their
formatting context
: the box establishing their
formatting context
grows to fully contain them, and floats from outside that box are not allowed to protrude into and affect the contents inside the box.
As another example, margins do not collapse across
formatting context
boundaries.
Exclusions are able to affect content across
independent formatting context
boundaries. (At time of writing, they are the only layout feature that can.)
[CSS3-EXCLUSIONS]
Certain properties can force a box to
establish an independent formatting context
in cases where it wouldn’t ordinarily. For example, making a box
out-of-flow
causes it to
blockify
as well as to
establish an independent formatting context
. As another example, certain values of the
contain
property can cause a box to
establish an independent formatting context
. Turning a block into a
scroll container
will cause it to
establish an independent formatting context
; however turning a
subgrid
into a
scroll container
will not—​it continues to act as a subgrid, with its contents participating in the layout of its parent
grid container
.
A
block box
that
establishes an independent formatting context
establishes a new
block formatting context
for its contents. In most other cases, forcing a box to
establish an independent formatting context
is a no-op—​either the box already establishes an
independent formatting context
(e.g.
flex containers
), or it’s not possible to establish a totally independent new formatting context on that type of box (e.g. non-replaced
inline boxes
).
block formatting context
inline formatting context
Block
and
inline formatting contexts
are defined in
CSS 2.1 Section 9.4
.
Inline formatting contexts
exist within (are part of their containing)
block formatting contexts
; for example, line boxes belonging to the
inline formatting context
interact with floats belonging to the
block formatting context
.
block layout
The layout of
block-level boxes
, performed within a
block formatting context
.
block formatting context root
A
block container
that establishes a new
block formatting context
.
BFC
Abbreviation for
block formatting context
or
block formatting context root
. Has various informal definitions referring to boxes which contain internal floats, exclude external floats, and suppress margin collapsing, and may therefore refer specifically to one of:
a
block container
that establishes a new
block formatting context
for its contents
a
block box
(i.e. a
block-level
block container) that establishes a
block formatting context
for its contents (as distinguished from a block box which does not)
(very loosely) any block-level box that establishes a new
formatting context
(other than an inline formatting context)
out-of-flow
in-flow
A box is
out-of-flow
if it is extracted from its expected position and interaction with surrounding content and laid out using a different paradigm outside the normal flow of content in its parent formatting context. This occurs if the box is floated (via
float
) or
absolutely positioned
(via
position
). A box is
in-flow
if it is not
out-of-flow
.
Note:
Some
formatting contexts
inhibit floating, so that an element with
float: left
is not necessarily
out-of-flow
.
document order
The order in which boxes or content occurs in the document (which can be different from the order in which it appears for rendering). For the purpose of determining the relative order of pseudo-elements, the box-tree order is used, see
CSS Pseudo-Elements 4
§ 4 Tree-Abiding Pseudo-elements
.
See
[CSS2]
Chapter 9
for a fuller definition of these terms.
Appendix B: Effects of
display: contents
on Unusual Elements
This section is (currently) non-normative.
Some elements aren’t rendered purely by CSS box concepts; for example, replaced elements (such as
img
), many form controls (such as
input
), and SVG elements.
This appendix defines how they interact with
display: contents
.
HTML Elements
br
wbr
meter
progress
canvas
embed
object
audio
iframe
img
video
frame
frameset
input
textarea
select
display: contents
computes to
display: none
.
legend
Per HTML, a
legend
with
display: contents
is not a
rendered legend
, so it does not have magical display behavior. (Thus, it reacts to
display: contents
normally.)
button
details
fieldset
These elements don’t have any special behavior;
display: contents
simply removes their
principal box
, and their contents render as normal.
any other HTML element
Behaves as normal for
display: contents
.
SVG Elements
An
svg
element that has CSS box layout (this includes all
svg
whose parent is an HTML element, as well as document root elements)
display: contents
computes to
display: none
.
All other SVG
container elements
that are also
renderable elements
SVG
text content child elements
use
display: contents
strips the element from the formatting tree, and hoists its contents up to display in its place. These contents include the shadow-DOM content for
use
.
any other SVG elements
display: contents
computes to
display: none
.
The intention here is that the
display: none
behavior applies whenever the "rendering context" inside the element is different than the context outside of it. If the element’s child elements would not be valid children of the element’s parent, you cannot simply hoist them up the formatting tree.
For example, text content and text formatting elements in SVG require a
text
element context; if you remove a
text
, its child text content and elements are no longer valid. For that reason,
display: contents
on
text
prevents the entire text element from being rendered. In contrast, any valid content inside a
tspan
or
textPath
is also valid content inside the parent text formatting context, so the hoisting behavior applies for these elements.
Similarly, if hoisting would convert the children from
non-rendered elements
(e.g., a shape inside a
pattern
or
symbol
) to
rendered elements
(e.g., a shape that is a direct child of the
svg
), that is an invalid change of rendering context. Never-rendered container elements therefore cannot be "un-boxed" by
display: contents
.
When an element is stripped from the formatting tree, then any SVG attributes on that element that control layout and visual formatting are ignored when rendering the contents. However, SVG
presentation attributes
—​which map to CSS properties—​continue to affect value processing and inheritance
[CSS-CASCADE-3]
; thus such attributes can affect the layout and visual formatting of the element’s descendants by influencing the values of such properties on those descendants.
MathML Elements
For all MathML elements,
display: contents
computes to
display: none
.
Appendix C: Box Construction Guidelines for Spec Authors
This section is non-normative guidance for specification authors.
A box cannot be
blockified
and
inlinified
at the same time; if such a thing would occur, define which wins over the other.
Non-principal non-anonymous boxes can’t be
blockified
: blockification affects the element’s computed values and thus determines the type of its
principal box
.
Boxes which
blockify
their contents can’t directly contain
inline-level
content; any boxes or text sequences generated within such an element must be
blockified
or wrapped in an
anonymous
block container
.
Boxes which
inlinify
their contents can’t directly contain
block-level
boxes; any boxes generated within such an element must be
inline-level
.
Boxes that fundamentally cannot establish an
independent formatting context
(such as non-replaced inlines) must not be asked to
establish an independent formatting context
. Blockify them first, or otherwise change their box type to one that can establish an
independent formatting context
.
Acknowledgments
We would like to thank the many people who have attempted to separate out the disparate details of box generation over the years, most particularly Bert Bos, whose last attempt with
display-model
and
display-role
didn’t get anywhere, but primed us for the current spec; Anton Prowse, whose relentless assault on CSS2.1 Chapter 9 forced some order out of the chaos; and Oriol Brufau, who teased apart dozens of fine distinctions and errors in this spec. Honorable mentions also go to David Baron, Mats Palmgren, Ilya Streltsyn, and Boris Zbarsky for their feedback.
Changes
Changes Since 2023 Candidate Recommendation Snapshot
Changes since the
30 March 2023 Candidate Recommendation Snapshot
include:
Inlined the CSS2.1 definition of the
initial containing block
for
continuous media
, cross-referenced its
writing mode
propagation as defined in
[CSS-WRITING-MODES-4]
, and clarified that it refers to the
small viewport size
. (
Issue 6453
)
Changes Since 2020 Candidate Recommendation
A
Disposition of Comments
since the
19 May 2020 Candidate Recommendation
is available.
Changes since the
19 May 2020 Candidate Recommendation
include:
Renamed “text run” to “
text sequence
”. (
Issue 7768
)
Defined
root element
and clarified its layout as a
block-level
in
§ 2.8 The Root Element’s Principal Box
and the definition of
initial containing block
. (
PR 8095
,
Issue 7207
,
Issue 6480
,
Issue 7786
)
Pulled in the
order
property definition from
[CSS-FLEXBOX-1]
, since it also applies to
grid items
. (
Issue 5865
)
Imported the
visibility
property definition from
[CSS2]
and updated it to be more thorough and complete. (
Issue 6123
)
Defined that
replaced elements
with a
layout-internal
display
are treated as
display: inline
. (
Issue 6000
)
Clarified that the
<display-legacy>
values actually
compute
to the same value as their two-keyword equivalents. (
Issue 5575
)
inline-…
Behaves as
Computes to
inline …
.
Note: Although these keywords and their equivalents compute to the same value, their
specified values
remain distinct.
Note: The
getComputedStyle()
serialization rules will always output these precomposed keywords rather than the equivalent two-keyword pairs due to the
shortest, most backwards-compatible serialization principle
.
Clarified that
blockification
and
inlinification
are
computed value
changes. (
Issue 6251
)
Some layout effects require
blockification
or
inlinification
of the box type, which sets the box’s
computed
outer display type
to
block
or
inline
(respectively).
Added definition for
block layout
to glossary, for convenience.
Updated references to
CSS Grid Layout
.
Various minor editorial clarifications and cross-linking improvements.
Changes Since 2019 Candidate Recommendation
Changes since the
11 July 2019 Candidate Recommendation
include:
Merged in additional prose from
[CSS2]
into the definition of
containing block
.
Clarified that, for the purpose of CSS, DOM nodes other than elements and text are ignored.
(Some source documents start from more complex trees, such as the DOM, which can have comment nodes and other types of things. For the purposes of CSS, all of these additional types of nodes are ignored, as if they didn’t exist.)
Improved/moved glossary definitions relating to
absolute positioning
.
Changes since the
28 August 2018 Candidate Recommendation
include:
Defined box tree parentage; see
parent box
.
Added definition of
absolutely positioned
to the glossary;
copied from CSS2
for easier referencing.
Added cross-references to various forms of fragmentation in
§ 1 Introduction
.
Renamed “table box” to “table grid box” to more easily distinguish from “table wrapper box”.
Added “unless otherwise specified” to root → initial containing block propagation, since there are some regrettable special exceptions for the HTML
body
element.
Changes Prior to Candidate Recommendation Status
A
Disposition of Comments
is available.
Changes since the
20 April 2018 Working Draft
include:
Elements with
display: contents
that behave as
display: none
now
compute to
display: none
. (
Issue 2755
)
Distinguished “new”
formatting context
from
independent formatting context
since certain formatting contexts layer. (
Issue 2597
,
Issue 1457
)
Defined that
block boxes
that establish an
independent formatting context
have a used
display
of
flow-root
, to provide an easier point of reference. (
Issue 1550
)
Clarified that
display
is not animatable (as opposed to discretely animatable). (
Issue 2938
)
Minor editorial fixes.
Changes since the
20 July 2017 Working Draft
include:
Tightened up rules for the
blockification
of
inline-block
/
inline flow-root
to ensure compatibility with CSS2. (
Issue 1246
) Updated handling of
run-in flow-root
to match. (
Issue 1715
)
Adjusted grammar of
display
to list the
list-item
keyword last. This affects the expected serialization order. (
Issue 1621
)
Added better definition of interaction between
block formatting contexts
and
inline formatting contexts
in block containers that establish inline formatting contexts. (
Issue 1553
)
More clearly defined the way property values are reflected between an element and its boxes (in the case of an element generating multiple boxes). (
Issue 1643
)
Clarified that empty text objects are ignored for CSS rendering. (
Issue 1808
)
Clarified that
display
has no effect on document semantics, since this is a common bug in UAs. (
Issue 2355
)
Fixed error in definition of
display
’s
computed value
(which is definitely not “as specified”, due to
blockification
and
inlinification
rules triggered by various properties). (
Issue 1716
)
Added definition for
document order
.
Added missing SVG elements to
Appendix B
’s details on
display: contents
(
Issue 2118
), clarified effect of SVG attributes (
Issue 2502
), and defined behavior for MathML (
Issue 2167
).
Added some
guidance
to future spec authors of anonymous box construction rules. (
Issue 1643
)
Pushed the section about “becoming a formatting context” back to
CSS Containment
where it is used.
Various minor wording fixes and clarifications.
Changes since the
26 January 2017 Working Draft
include:
Remove
inline-list-item
value that is equivalent to
inline list-item
.
Added the notion of “
text nodes
” to the
element tree
, and “
text runs
” to the
box tree
to define behavior in the context of
display: contents
. (
Issue 19
,
Issue 32
)
Defined that the root element is “in flow”. (
Issue 3
)
Defined interaction of
::first-line
/
::first-letter
and
run-in
. (
Issue 5
,
Issue 42
)
Clarified that block/inline/run-in only dictates behavior in
flow layout
; it is ignored in other contexts.
Run-ins are a type of inline box, not just "like" an inline box.
Fixed the lack of recursion in of run-in’s box-tree munging. (
Issue 45
)
Added an appendix on how
display: contents
works on “unusual elements”. (
Issue 8
,
Issue 18
)
Fix
blockification
and
inlinification
rules, particularly handling of layout-internal types. (
Issue 35
,
Issue 57
)
Clarified interaction of various box tree fixups. (
Issue 38
,
Issue 48
)
Added the definition of “becoming a formatting context”.
Miscellaneous minor fixes and minor clarifications.
A
Disposition of Comments
is also available.
Changes since the
15 October 2015 Working Draft
include:
Deferred the
box-suppress/display-or-not
property to the next level of Display, in order to provide time for further discussion of use cases.
Specified the effects of
display: contents
on unusual elements such as replaced elements and form controls.
Clarified that an element’s
::before
and
::after
pseudo-elements still exist if its own box is not generated due to
display: contents
.
Clarified that event bubbling is not affected by
display: contents
.
Clarified interaction of run-ins with out-of-flow elements and
::first-letter
.
Switched
table-caption
and
table-cell
to use
flow-root
as their
inner display type
, since they always form a formatting context root.
Closed off remaining issues and added at-risk list.
Changes since the
21 July 2015 Working Draft
include:
Added definitions for
in-flow
and
out-of-flow
to glossary.
Changes since the
11 September 2014 Working Draft
include:
Removed
display-inside
,
display-outside
, and
display-extras
longhands, in favor of just making
display
multi-value. (This was done to impose constraints on what can be combined. Future levels of this specification may relax some or all of those restrictions if they become unnecessary or unwanted.)
Created the
flow
and
flow-root
inner display types
to better express flow layout
display types
and to create an explicit switch for making an element a
BFC
root. (This should eliminate the need for hacks like
::after { clear: both; }
and
overflow: hidden
that are intended to accomplish this purpose.)
Privacy Considerations
This specification introduces no new privacy considerations.
Security Considerations
This specification introduces no new security considerations.
Self-Review Questionnaire
Per the
Self-Review Questionnaire: Security and Privacy: Questions to Consider
Does this specification deal with personally-identifiable information?
No.
Does this specification deal with high-value data?
No.
Does this specification introduce new state for an origin that persists across browsing sessions?
No.
Does this specification expose persistent, cross-origin state to the web?
No.
Does this specification expose any other data to an origin that it doesn’t currently have access to?
No.
Does this specification enable new script execution/loading mechanisms?
No.
Does this specification allow an origin access to a user’s location?
No.
Does this specification allow an origin access to sensors on a user’s device?
No.
Does this specification allow an origin access to aspects of a user’s local computing environment?
No.
Does this specification allow an origin access to other devices?
No.
Does this specification allow an origin some measure of control over a user agent’s native UI?
No.
Does this specification expose temporary identifiers to the web?
No.
Does this specification distinguish between behavior in first-party and third-party contexts?
No.
How should this specification work in the context of a user agent’s "incognito" mode?
No differently.
Does this specification persist data to a user’s local device?
No.
Does this specification have a "Security Considerations" and "Privacy Considerations" section?
Yes.
Does this specification allow downgrading default security characteristics?
No.
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
CR exit criteria
For this specification to be advanced to Proposed Recommendation, there must be at least two independent, interoperable implementations of each feature. Each feature may be implemented by a different set of products, there is no requirement that all features be implemented by a single product. For the purposes of this criterion, we define the following terms:
independent
each implementation must be developed by a different party and cannot share, reuse, or derive from code used by another qualifying implementation. Sections of code that have no bearing on the implementation of this specification are exempt from this requirement.
interoperable
passing the respective test case(s) in the official CSS test suite, or, if the implementation is not a Web browser, an equivalent test. Every relevant test in the test suite should have an equivalent test created if such a user agent (UA) is to be used to claim interoperability. In addition if such a UA is to be used to claim interoperability, then there must one or more additional UAs which can also pass those equivalent tests in the same way for the purpose of interoperability. The equivalent tests must be made publicly available for the purposes of peer review.
implementation
a user agent which:
implements the specification.
is available to the general public. The implementation may be a shipping product or other publicly available version (i.e., beta version, preview release, or "nightly build"). Non-shipping product releases must have implemented the feature(s) for a period of at least one month in order to demonstrate stability.
is not experimental (i.e., a version specifically designed to pass the test suite and is not intended for normal usage going forward).
The specification will remain Candidate Recommendation for at least six months.
Index
Terms defined by this specification
anonymous
, in § 1
anonymous box
, in § 1
atomic inline
, in § Unnumbered section
atomic inline box
, in § Unnumbered section
BFC
, in § Unnumbered section
block
definition of
, in § Unnumbered section
value for display, <display-outside>
, in § 2.1
block box
, in § Unnumbered section
block container
, in § Unnumbered section
block container box
, in § Unnumbered section
block formatting context
, in § Unnumbered section
block formatting context root
, in § Unnumbered section
blockification
, in § 2.7
blockify
, in § 2.7
block layout
, in § Unnumbered section
block-level
, in § Unnumbered section
block-level box
, in § 2.1
block-level content
, in § Unnumbered section
box
, in § 1
box tree
, in § 1
collapse
, in § 4
collapsed
, in § 4
containing block
, in § Unnumbered section
containing block chain
, in § Unnumbered section
contents
, in § 2.5
display
, in § 2
<display-box>
, in § 2
<display-inside>
, in § 2
<display-internal>
, in § 2
<display-legacy>
, in § 2
<display-listitem>
, in § 2
<display-outside>
, in § 2
display type
, in § 2
document order
, in § Unnumbered section
element
, in § 1
element tree
, in § 1
establish an independent formatting context
, in § Unnumbered section
established an independent formatting context
, in § Unnumbered section
establishes an independent formatting context
, in § Unnumbered section
establishing an independent formatting context
, in § Unnumbered section
flex
, in § 2.2
flow
, in § 2.2
flow layout
, in § 2.2
flow-root
, in § 2.2
formatting context
, in § Unnumbered section
grid
, in § 2.2
hidden
, in § 4
independent formatting context
, in § Unnumbered section
in flow
, in § Unnumbered section
in-flow
, in § Unnumbered section
in-flow box
, in § Unnumbered section
initial containing block
, in § Unnumbered section
inline
definition of
, in § Unnumbered section
value for display, <display-outside>
, in § 2.1
inline block
, in § 2
inline-block
, in § 2.6
inline block box
, in § 2
inline box
, in § Unnumbered section
inline-flex
, in § 2.6
inline formatting context
, in § Unnumbered section
inline-grid
, in § 2.6
inline-level
, in § Unnumbered section
inline-level box
, in § 2.1
inline-level content
, in § Unnumbered section
inline-table
, in § 2.6
inlinification
, in § 2.7
inlinify
, in § 2.7
inner
, in § 2
inner display type
, in § 2
<integer>
, in § 3
internal ruby box
, in § 2.4
internal ruby element
, in § 2.4
internal table box
, in § 2.4
internal table element
, in § 2.4
invisible
, in § 4
invisible box
, in § 4
layout-internal
, in § 2.4
list-item
, in § 2.3
none
, in § 2.5
non-replaced
, in § Unnumbered section
non-replaced element
, in § Unnumbered section
order
, in § 3
order-modified document order
, in § 3
outer
, in § 2
outer display type
, in § 2
out of flow
, in § Unnumbered section
out-of-flow
, in § Unnumbered section
out-of-flow box
, in § Unnumbered section
parent box
, in § 1
principal box
, in § Unnumbered section
replaced
, in § Unnumbered section
replaced element
, in § Unnumbered section
root element
, in § Unnumbered section
ruby
, in § 2.2
ruby-base
, in § 2.4
ruby-base-container
, in § 2.4
ruby-text
, in § 2.4
ruby-text-container
, in § 2.4
run-in
definition of
, in § 5
value for display, <display-outside>
, in § 2.1
run-in box
, in § 5
run-in sequence
, in § 5
table
, in § 2.2
table-caption
, in § 2.4
table caption box
, in § 2.4
table-cell
, in § 2.4
table-column
, in § 2.4
table-column-group
, in § 2.4
table-footer-group
, in § 2.4
table-header-group
, in § 2.4
table-row
, in § 2.4
table-row-group
, in § 2.4
text node
, in § 1
text sequence
, in § 1
visibility
, in § 4
visible
, in § 4
Terms defined by reference
[CSS-ALIGN-3]
defines the following terms:
align-content
[CSS-BACKGROUNDS-3]
defines the following terms:
border
[CSS-BOX-4]
defines the following terms:
box edge
[CSS-BREAK-3]
defines the following terms:
fragment
[CSS-BREAK-4]
defines the following terms:
box fragment
fragmentation
[CSS-CASCADE-5]
defines the following terms:
computed value
inheritance
inherited property
specified value
[CSS-CONTAIN-2]
defines the following terms:
contain
[CSS-FLEXBOX-1]
defines the following terms:
flex container
flex formatting context
flex item
flex layout
[CSS-GRID-2]
defines the following terms:
grid container
grid formatting context
grid item
subgrid
subgrid
(for grid-template-rows)
[CSS-IMAGES-3]
defines the following terms:
natural dimension
[CSS-INLINE-3]
defines the following terms:
root inline box
[CSS-OVERFLOW-3]
defines the following terms:
overflow
scroll container
[CSS-POSITION-3]
defines the following terms:
absolute position
absolutely position
position
[CSS-PSEUDO-4]
defines the following terms:
::after
::before
::first-letter
::first-line
::marker
first formatted line
[CSS-RUBY-1]
defines the following terms:
ruby base container
ruby container
ruby formatting context
[CSS-SPEECH-1]
defines the following terms:
always
speak
[CSS-TABLES-3]
defines the following terms:
table grid box
table wrapper box
[CSS-TEXT-4]
defines the following terms:
white space
[CSS-UI-4]
defines the following terms:
pointer-events
[CSS-VALUES-4]
defines the following terms:
&&
<integer>
?
CSS-wide keywords
small viewport size
|
||
[CSS-WRITING-MODES-4]
defines the following terms:
principal writing mode
writing mode
[CSS2]
defines the following terms:
float
list-style
overflow
[CSSOM]
defines the following terms:
getComputedStyle(elt)
[DOM]
defines the following terms:
document element
document tree
[HTML]
defines the following terms:
audio
body
br
button
canvas
details
embed
fieldset
frame
frameset
html
iframe
img
input
legend
meter
object
progress
rendered legend
select
src
summary
textarea
video
wbr
[MEDIAQUERIES-5]
defines the following terms:
continuous media
paged media
[SELECTORS-4]
defines the following terms:
document language
pseudo-elements
[SVG2]
defines the following terms:
container element
non-rendered element
pattern
presentation attributes
renderable element
rendered element
svg
symbol
text
text content child element
textPath
tspan
use
References
Normative References
[CSS-ALIGN-3]
Elika Etemad; Tab Atkins Jr..
CSS Box Alignment Module Level 3
. 30 January 2026. WD. URL:
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
[CSS-BREAK-3]
Rossen Atanassov; Elika Etemad.
CSS Fragmentation Module Level 3
. 4 December 2018. CR. URL:
https://www.w3.org/TR/css-break-3/
[CSS-BREAK-4]
Rossen Atanassov; Elika Etemad.
CSS Fragmentation Module Level 4
. 18 December 2018. FPWD. URL:
https://www.w3.org/TR/css-break-4/
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
[CSS-CONTAIN-2]
Tab Atkins Jr.; Florian Rivoal; Vladimir Levin.
CSS Containment Module Level 2
. 17 September 2022. WD. URL:
https://www.w3.org/TR/css-contain-2/
[CSS-FLEXBOX-1]
Elika Etemad; Tab Atkins Jr.; Rossen Atanassov.
CSS Flexible Box Layout Module Level 1
. 14 October 2025. CRD. URL:
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
[CSS-OVERFLOW-3]
Elika Etemad; Florian Rivoal.
CSS Overflow Module Level 3
. 7 October 2025. WD. URL:
https://www.w3.org/TR/css-overflow-3/
[CSS-PAGE-3]
Elika Etemad.
CSS Paged Media Module Level 3
. 14 September 2023. WD. URL:
https://www.w3.org/TR/css-page-3/
[CSS-POSITION-3]
Elika Etemad; Tab Atkins Jr..
CSS Positioned Layout Module Level 3
. 7 October 2025. WD. URL:
https://www.w3.org/TR/css-position-3/
[CSS-PSEUDO-4]
Elika Etemad; Alan Stearns.
CSS Pseudo-Elements Module Level 4
. 27 June 2025. WD. URL:
https://www.w3.org/TR/css-pseudo-4/
[CSS-RUBY-1]
Elika Etemad; et al.
CSS Ruby Annotation Layout Module Level 1
. 31 December 2022. WD. URL:
https://www.w3.org/TR/css-ruby-1/
[CSS-SPEECH-1]
Léonie Watson; Elika Etemad.
CSS Speech Module Level 1
. 14 February 2023. CRD. URL:
https://www.w3.org/TR/css-speech-1/
[CSS-TABLES-3]
Keith Cirkel.
CSS Table Module Level 3
. 16 December 2025. WD. URL:
https://www.w3.org/TR/css-tables-3/
[CSS-TEXT-4]
Elika Etemad; et al.
CSS Text Module Level 4
. 29 May 2024. WD. URL:
https://www.w3.org/TR/css-text-4/
[CSS-UI-4]
Tab Atkins Jr.; Florian Rivoal.
CSS Basic User Interface Module Level 4
. 20 January 2026. WD. URL:
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
[DOM]
Anne van Kesteren.
DOM Standard
. Living Standard. URL:
https://dom.spec.whatwg.org/
[HTML]
Anne van Kesteren; et al.
HTML Standard
. Living Standard. URL:
https://html.spec.whatwg.org/multipage/
[MEDIAQUERIES-5]
Tab Atkins Jr.; et al.
Media Queries Level 5
. 19 February 2026. WD. URL:
https://www.w3.org/TR/mediaqueries-5/
[RFC2119]
S. Bradner.
Key words for use in RFCs to Indicate Requirement Levels
. March 1997. Best Current Practice. URL:
https://datatracker.ietf.org/doc/html/rfc2119
[SELECTORS-4]
Elika Etemad; Tab Atkins Jr..
Selectors Level 4
. 22 January 2026. WD. URL:
https://www.w3.org/TR/selectors-4/
[SVG2]
Amelia Bellamy-Royds; et al.
Scalable Vector Graphics (SVG) 2
. 4 October 2018. CR. URL:
https://www.w3.org/TR/SVG2/
Non-Normative References
[CSS3-EXCLUSIONS]
Rossen Atanassov; Vincent Hardy; Alan Stearns.
CSS Exclusions Module Level 1
. 15 January 2015. WD. URL:
https://www.w3.org/TR/css3-exclusions/
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
Media
display
[ <display-outside> || <display-inside> ] | <display-listitem> | <display-internal> | <display-box> | <display-legacy>
inline
all elements
no
n/a
not animatable
per grammar
a pair of keywords representing the inner and outer display types plus optional list-item flag, or a <display-internal> or <display-box> keyword; see prose in a variety of specs for computation rules
order
<integer>
0
flex items and grid items
no
n/a
by computed value type
per grammar
specified integer
visibility
visible | hidden | collapse
visible
all elements
yes
N/A
discrete
per grammar
as specified
visual
```
