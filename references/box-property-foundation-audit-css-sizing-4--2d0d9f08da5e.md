Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

This software or document includes material copied from or derived from [CSS Box Sizing Module Level 4](https://www.w3.org/TR/2026/WD-css-sizing-4-20260904/).

Original copyright notice (from the matching exact HTML edition): Copyright © 2026 World Wide Web Consortium. W3C® liability, trademark and permissive document license rules apply.

License: [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: CSS Box Sizing Module Level 4

Source snapshot: https://www.w3.org/TR/2026/WD-css-sizing-4-20260904/

Snapshot SHA-256: 2d0d9f08da5eb657339f485745cd889006972dee00564ccb35d1694c8b6e7059

Conversion: offline format conversion of the exact stored extracted-text; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- This stored input is an already extracted plain-text witness. It has no recoverable original HTML structure. The complete source text is preserved in a fenced block; no summary or specification regeneration was performed.

---

```text
CSS Box Sizing Module Level 4
CSS Box Sizing Module Level 4
W3C Working Draft
,
4 September 2026
More details about this document
This version:
https://www.w3.org/TR/2026/WD-css-sizing-4-20260904/
Latest published version:
https://www.w3.org/TR/css-sizing-4/
Editor's Draft:
https://drafts.csswg.org/css-sizing-4/
Previous Versions:
https://www.w3.org/TR/2021/WD-css-sizing-4-20210520/
History:
https://www.w3.org/standards/history/css-sizing-4/
Test Suites:
https://wpt.fyi/results/css/css-sizing
https://wpt.fyi/results/css/css-sizing/
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
Jen Simmons
(
Apple
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
This module extends the CSS sizing properties with keywords that represent content-based "intrinsic" sizes and context-based "extrinsic" sizes, allowing CSS to more easily describe boxes that fit their content or fit into a particular layout context. This is a delta spec over CSS Sizing Level 3.
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
Table of Contents
1
Introduction
1.1
Module interactions
1.2
Value Definitions
2
Terminology
3
Specifying Box Sizes
3.1
Sizing Properties
3.2
New Sizing Values: the
contain
,
fit-content()
, and
calc-size()
values
4
Aspect Ratios
4.1
Preferred Aspect Ratios: the
aspect-ratio
property
4.2
Effects of Preferred Aspect Ratio on Automatic Sizes
4.2.1
Margin-collapsing
4.3
Automatic Content-based Minimum Sizes
4.4
Min/Max Size Transfers
5
Intrinsic Size Determination
5.1
Intrinsic Sizes
5.2
Overriding Contained Intrinsic Sizes: the
contain-intrinsic-*
properties
5.2.1
Last Remembered Size
5.2.2
Interaction With
overflow: auto
5.3
Responsively-sized iframes: the
frame-sizing
property
5.3.1
HTML
iframe
Details
5.3.2
Extensions to the
Window
Interface
5.4
Intrinsic Size Contributions
5.5
Zeroing Min-Content Size Contributions: the
min-intrinsic-sizing
property
5.6
New Column Sizing Values: the
stretch
,
min-content
,
max-content
,
fit-content
, and
fit-content()
values
6
Extrinsic Size Determination
6.1
Contain-fit Sizing: stretching while maintaining an aspect ratio
6.2
Percentage Sizing
Changes
Changes since 5 May 2021 Working Draft
Changes since 20 October 2020 Working Draft
Additions Since Level 3
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
IDL Index
Issues Index
1.
Introduction
This is a diff spec over
CSS Sizing Level 3
. It is currently an Exploratory Working Draft: if you are implementing anything, please use Level 3 as a reference. We will merge the Level 3 text into this draft once it reaches CR.
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
CSS Sizing 3
§ 2 Terminology
3.
Specifying Box Sizes
CSS Sizing 3
§ 3 Specifying Box Sizes
3.1.
Sizing Properties
CSS Sizing 3
§ 3.1 Sizing Properties
Name:
size
Value:
<'width'>
<'height'>
?
Initial:
auto
Applies to:
all elements
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
The
size
property is a
shorthand
that sets
width
and
height
in a single declaration. If the second value is omitted, it is copied from the first.
The
size
shorthand cannot be used in
@page
contexts, as
@page
already has an unrelated
size
descriptor for setting the page size.
The
size
property needs to be omitted from the
preferred shorthand order
in CSSOM, to avoid compat problems and conflicts with
@page
.
Name:
min-size
Value:
<'min-width'>
<'min-height'>
?
Initial:
auto
Applies to:
all elements
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
The
min-size
property is a
shorthand
that sets
min-width
and
min-height
in a single declaration. If the second value is omitted, it is copied from the first.
Name:
max-size
Value:
<'max-width'>
<'max-height'>
?
Initial:
none
Applies to:
all elements
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
The
max-size
property is a
shorthand
that sets
max-width
and
max-height
in a single declaration. If the second value is omitted, it is copied from the first.
3.2.
New Sizing Values: the
contain
,
fit-content()
, and
calc-size()
values
CSS Sizing 3
§ 3.2 Sizing Values: the <length-percentage>, auto | none, min-content, max-content, and fit-content() values
Level 4 adds the ability to use the
contain
,
fit-content()
, and
calc-size()
values in the
sizing properties
, thus expanding the
<box-size>
production as follows:
<box-size>
=
<length-percentage>
|
stretch
|
contain
|
min-content
|
max-content
|
fit-content
|
fit-content()
|
calc-size()
Name:
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
New values:
contain
|
fit-content(
<length-percentage [0,∞]>
)
fit-content(
<length-percentage [0,∞]>
)
Use the fit-content formula with the
available space
replaced by the specified argument, i.e.
min(
max-content
, max(
min-content
,
<length-percentage>
))
, where the
<length-percentage>
argument is resolved exactly as for
<length-percentage>
values standing alone.
Negative
<length-percentage>
values are invalid.
Tests
auto-scrollbar-inside-stf-abspos.html
(live test)
(source)
block-fit-content-as-initial.html
(live test)
(source)
fit-content-length-percentage-001.html
(live test)
(source)
fit-content-length-percentage-002.html
(live test)
(source)
fit-content-length-percentage-003.html
(live test)
(source)
fit-content-length-percentage-004.html
(live test)
(source)
fit-content-length-percentage-005.html
(live test)
(source)
fit-content-length-percentage-006.html
(live test)
(source)
fit-content-length-percentage-007.html
(live test)
(source)
fit-content-length-percentage-008.html
(live test)
(source)
fit-content-length-percentage-009.html
(live test)
(source)
fit-content-length-percentage-010.html
(live test)
(source)
fit-content-length-percentage-011.html
(live test)
(source)
fit-content-length-percentage-012.html
(live test)
(source)
fit-content-length-percentage-013.html
(live test)
(source)
fit-content-length-percentage-014.html
(live test)
(source)
fit-content-length-percentage-015.html
(live test)
(source)
fit-content-length-percentage-016.html
(live test)
(source)
contain
If the box has a
preferred aspect ratio
, applies
contain-fit sizing
, attempting to fit into the box’s constraints while maintaining its
preferred aspect ratio
insofar as possible. See
§ 6.1 Contain-fit Sizing: stretching while maintaining an aspect ratio
.
If the box has no
preferred aspect ratio
, applies
stretch-fit sizing
.
calc-size()
See
CSS Values 5
§ 10. Calculating With Intrinsic Sizes: the calc-size() function
.
Note:
The
none
keyword is not usable within
calc-size()
.
Note:
These new values add to the set of values that the definition of
<calc-size()>
refers to as “allowed in the context”.
4.
Aspect Ratios
Images often have a
natural aspect ratio
, which the CSS layout algorithms attempt to preserve as they resize the element.
The
aspect-ratio
property allows specifying this behavior for non-replaced elements, and for altering the effective aspect ratio of replaced elements.
We are still working through the details of this section. If there is any behavior specified here that would cause
replaced elements
with a
preferred aspect ratio
to behave differently than they would under the requirements of the
CSS2
,
Flex Layout
, and
Grid Layout
specs combined (without this specification in effect),
this is an error and should be
reported
to the CSSWG
. There is a
list of open aspect-ratio issues
.
4.1.
Preferred Aspect Ratios: the
aspect-ratio
property
Name:
aspect-ratio
Value:
auto
||
<ratio>
Initial:
auto
Applies to:
all elements except
inline boxes
and internal ruby or table boxes
Inherited:
no
Percentages:
n/a
Computed value:
specified keyword or a pair of numbers
Canonical order:
per grammar
Animation type:
by computed value
Tests
aspect-ratio-interpolation.html
(live test)
(source)
abspos-001.html
(live test)
(source)
abspos-002.html
(live test)
(source)
abspos-003.html
(live test)
(source)
abspos-004.html
(live test)
(source)
abspos-005.html
(live test)
(source)
abspos-006.html
(live test)
(source)
abspos-007.html
(live test)
(source)
abspos-008.html
(live test)
(source)
abspos-009.html
(live test)
(source)
abspos-010.html
(live test)
(source)
abspos-011.html
(live test)
(source)
abspos-012.html
(live test)
(source)
abspos-013.html
(live test)
(source)
abspos-014.html
(live test)
(source)
abspos-015.html
(live test)
(source)
abspos-016.html
(live test)
(source)
abspos-017.html
(live test)
(source)
abspos-018.html
(live test)
(source)
abspos-019.html
(live test)
(source)
abspos-020.html
(live test)
(source)
abspos-021.html
(live test)
(source)
auto-margins-001.html
(live test)
(source)
block-aspect-ratio-001.html
(live test)
(source)
block-aspect-ratio-002.html
(live test)
(source)
block-aspect-ratio-003.html
(live test)
(source)
block-aspect-ratio-004.html
(live test)
(source)
block-aspect-ratio-005.html
(live test)
(source)
block-aspect-ratio-006.html
(live test)
(source)
block-aspect-ratio-007.html
(live test)
(source)
block-aspect-ratio-008.html
(live test)
(source)
block-aspect-ratio-009.html
(live test)
(source)
block-aspect-ratio-010.html
(live test)
(source)
block-aspect-ratio-011.html
(live test)
(source)
block-aspect-ratio-012.html
(live test)
(source)
block-aspect-ratio-013.html
(live test)
(source)
block-aspect-ratio-014.html
(live test)
(source)
block-aspect-ratio-015.html
(live test)
(source)
block-aspect-ratio-016.html
(live test)
(source)
block-aspect-ratio-017.html
(live test)
(source)
block-aspect-ratio-018.html
(live test)
(source)
block-aspect-ratio-019.html
(live test)
(source)
block-aspect-ratio-020.html
(live test)
(source)
block-aspect-ratio-021.html
(live test)
(source)
block-aspect-ratio-022.html
(live test)
(source)
block-aspect-ratio-023.html
(live test)
(source)
block-aspect-ratio-024.html
(live test)
(source)
block-aspect-ratio-025.html
(live test)
(source)
block-aspect-ratio-026.html
(live test)
(source)
block-aspect-ratio-027.html
(live test)
(source)
block-aspect-ratio-028.html
(live test)
(source)
block-aspect-ratio-029-crash.html
(live test)
(source)
block-aspect-ratio-030.html
(live test)
(source)
block-aspect-ratio-031.html
(live test)
(source)
block-aspect-ratio-032.html
(live test)
(source)
block-aspect-ratio-033.html
(live test)
(source)
block-aspect-ratio-034.html
(live test)
(source)
block-aspect-ratio-035.html
(live test)
(source)
block-aspect-ratio-036.html
(live test)
(source)
block-aspect-ratio-037.html
(live test)
(source)
block-aspect-ratio-051-crash.html
(live test)
(source)
block-aspect-ratio-052.html
(live test)
(source)
block-aspect-ratio-053.html
(live test)
(source)
block-aspect-ratio-054.html
(live test)
(source)
block-aspect-ratio-055.html
(live test)
(source)
block-aspect-ratio-056.html
(live test)
(source)
block-aspect-ratio-058.html
(live test)
(source)
block-aspect-ratio-with-margin-collapsing-001.html
(live test)
(source)
block-aspect-ratio-with-margin-collapsing-002.html
(live test)
(source)
box-sizing-dimensions.html
(live test)
(source)
box-sizing-squashed.html
(live test)
(source)
flex-aspect-ratio-001.html
(live test)
(source)
flex-aspect-ratio-003.html
(live test)
(source)
flex-aspect-ratio-005.html
(live test)
(source)
flex-aspect-ratio-006.html
(live test)
(source)
flex-aspect-ratio-007.html
(live test)
(source)
flex-aspect-ratio-008.html
(live test)
(source)
flex-aspect-ratio-009.html
(live test)
(source)
flex-aspect-ratio-010.html
(live test)
(source)
flex-aspect-ratio-011.html
(live test)
(source)
flex-aspect-ratio-012.html
(live test)
(source)
flex-aspect-ratio-013.html
(live test)
(source)
flex-aspect-ratio-014.html
(live test)
(source)
flex-aspect-ratio-015.html
(live test)
(source)
flex-aspect-ratio-016.html
(live test)
(source)
flex-aspect-ratio-017.html
(live test)
(source)
flex-aspect-ratio-018.html
(live test)
(source)
flex-aspect-ratio-019.html
(live test)
(source)
flex-aspect-ratio-020.html
(live test)
(source)
flex-aspect-ratio-021.html
(live test)
(source)
flex-aspect-ratio-022.html
(live test)
(source)
flex-aspect-ratio-023.html
(live test)
(source)
flex-aspect-ratio-024.html
(live test)
(source)
flex-aspect-ratio-027.html
(live test)
(source)
flex-aspect-ratio-028.html
(live test)
(source)
flex-aspect-ratio-029.html
(live test)
(source)
flex-aspect-ratio-030.html
(live test)
(source)
flex-aspect-ratio-032.html
(live test)
(source)
flex-aspect-ratio-033.html
(live test)
(source)
flex-aspect-ratio-034.html
(live test)
(source)
flex-aspect-ratio-035.html
(live test)
(source)
flex-aspect-ratio-036.html
(live test)
(source)
flex-aspect-ratio-037.html
(live test)
(source)
flex-aspect-ratio-038.html
(live test)
(source)
flex-aspect-ratio-045.html
(live test)
(source)
flex-aspect-ratio-046.html
(live test)
(source)
flex-aspect-ratio-047.html
(live test)
(source)
flex-aspect-ratio-048.html
(live test)
(source)
flex-aspect-ratio-049.html
(live test)
(source)
flex-aspect-ratio-050.html
(live test)
(source)
flex-aspect-ratio-051.html
(live test)
(source)
flex-aspect-ratio-052.html
(live test)
(source)
flex-aspect-ratio-053.html
(live test)
(source)
flex-aspect-ratio-054.html
(live test)
(source)
flex-aspect-ratio-055.html
(live test)
(source)
floats-aspect-ratio-001.html
(live test)
(source)
fractional-aspect-ratio.html
(live test)
(source)
grid-aspect-ratio-001.html
(live test)
(source)
grid-aspect-ratio-002.html
(live test)
(source)
grid-aspect-ratio-003.html
(live test)
(source)
grid-aspect-ratio-004.html
(live test)
(source)
grid-aspect-ratio-005.html
(live test)
(source)
grid-aspect-ratio-006.html
(live test)
(source)
grid-aspect-ratio-007.html
(live test)
(source)
grid-aspect-ratio-008.html
(live test)
(source)
grid-aspect-ratio-009.html
(live test)
(source)
grid-aspect-ratio-010.html
(live test)
(source)
grid-aspect-ratio-011.html
(live test)
(source)
grid-aspect-ratio-012.html
(live test)
(source)
grid-aspect-ratio-014.html
(live test)
(source)
grid-aspect-ratio-015.html
(live test)
(source)
grid-aspect-ratio-016.html
(live test)
(source)
grid-aspect-ratio-017.html
(live test)
(source)
grid-aspect-ratio-018.html
(live test)
(source)
grid-aspect-ratio-019.html
(live test)
(source)
grid-aspect-ratio-020.html
(live test)
(source)
grid-aspect-ratio-021.html
(live test)
(source)
grid-aspect-ratio-022.html
(live test)
(source)
grid-aspect-ratio-023.html
(live test)
(source)
grid-aspect-ratio-024.html
(live test)
(source)
grid-aspect-ratio-025.html
(live test)
(source)
grid-aspect-ratio-026.html
(live test)
(source)
grid-aspect-ratio-027.html
(live test)
(source)
grid-aspect-ratio-028.html
(live test)
(source)
grid-aspect-ratio-029.html
(live test)
(source)
grid-aspect-ratio-030.html
(live test)
(source)
grid-aspect-ratio-031.html
(live test)
(source)
grid-aspect-ratio-032.html
(live test)
(source)
grid-aspect-ratio-033.html
(live test)
(source)
grid-aspect-ratio-034.html
(live test)
(source)
grid-aspect-ratio-035.html
(live test)
(source)
grid-aspect-ratio-036.html
(live test)
(source)
grid-aspect-ratio-037.html
(live test)
(source)
grid-aspect-ratio-038.html
(live test)
(source)
grid-aspect-ratio-040.html
(live test)
(source)
grid-aspect-ratio-041.html
(live test)
(source)
inheritance.html
(live test)
(source)
intrinsic-size-001.html
(live test)
(source)
intrinsic-size-002.html
(live test)
(source)
intrinsic-size-003.html
(live test)
(source)
intrinsic-size-004.html
(live test)
(source)
intrinsic-size-005.html
(live test)
(source)
intrinsic-size-006.html
(live test)
(source)
intrinsic-size-007.html
(live test)
(source)
intrinsic-size-008.html
(live test)
(source)
intrinsic-size-009.html
(live test)
(source)
intrinsic-size-011.html
(live test)
(source)
intrinsic-size-012.html
(live test)
(source)
intrinsic-size-013.html
(live test)
(source)
intrinsic-size-014.html
(live test)
(source)
intrinsic-size-015.html
(live test)
(source)
intrinsic-size-016.html
(live test)
(source)
intrinsic-size-017.html
(live test)
(source)
intrinsic-size-018.html
(live test)
(source)
intrinsic-size-019.html
(live test)
(source)
intrinsic-size-020.html
(live test)
(source)
intrinsic-size-021.html
(live test)
(source)
intrinsic-size-022.html
(live test)
(source)
intrinsic-size-023.html
(live test)
(source)
intrinsic-size-024.html
(live test)
(source)
intrinsic-size-025.html
(live test)
(source)
large-aspect-ratio-crash.html
(live test)
(source)
aspect-ratio-computed.html
(live test)
(source)
aspect-ratio-invalid.html
(live test)
(source)
aspect-ratio-valid.html
(live test)
(source)
percentage-resolution-001.html
(live test)
(source)
percentage-resolution-002.html
(live test)
(source)
percentage-resolution-003.html
(live test)
(source)
percentage-resolution-004.html
(live test)
(source)
percentage-resolution-005.html
(live test)
(source)
quirks-mode-001.html
(live test)
(source)
quirks-mode-002.html
(live test)
(source)
quirks-mode-003.html
(live test)
(source)
replaced-element-001.html
(live test)
(source)
replaced-element-002.html
(live test)
(source)
replaced-element-003.html
(live test)
(source)
replaced-element-004.html
(live test)
(source)
replaced-element-005.html
(live test)
(source)
replaced-element-006.html
(live test)
(source)
replaced-element-007.html
(live test)
(source)
replaced-element-008.html
(live test)
(source)
replaced-element-009.html
(live test)
(source)
replaced-element-010.html
(live test)
(source)
replaced-element-011.html
(live test)
(source)
replaced-element-012.html
(live test)
(source)
replaced-element-013.html
(live test)
(source)
replaced-element-014.html
(live test)
(source)
replaced-element-015.html
(live test)
(source)
replaced-element-016.html
(live test)
(source)
replaced-element-017.html
(live test)
(source)
replaced-element-018.html
(live test)
(source)
replaced-element-019.html
(live test)
(source)
replaced-element-020.html
(live test)
(source)
replaced-element-021.html
(live test)
(source)
replaced-element-022.html
(live test)
(source)
replaced-element-023.html
(live test)
(source)
replaced-element-024.html
(live test)
(source)
replaced-element-025.html
(live test)
(source)
replaced-element-026.html
(live test)
(source)
replaced-element-027.html
(live test)
(source)
replaced-element-028.html
(live test)
(source)
replaced-element-029.html
(live test)
(source)
replaced-element-030.html
(live test)
(source)
replaced-element-031.html
(live test)
(source)
replaced-element-034.html
(live test)
(source)
replaced-element-035.html
(live test)
(source)
replaced-element-036.html
(live test)
(source)
replaced-element-037.html
(live test)
(source)
replaced-element-042.html
(live test)
(source)
replaced-element-045.html
(live test)
(source)
replaced-element-046.html
(live test)
(source)
replaced-element-049.html
(live test)
(source)
replaced-element-dynamic-aspect-ratio.html
(live test)
(source)
select-element-001.html
(live test)
(source)
sign-function-aspect-ratio.html
(live test)
(source)
small-aspect-ratio-crash.html
(live test)
(source)
table-element-001.html
(live test)
(source)
zero-or-infinity-001.html
(live test)
(source)
zero-or-infinity-002.html
(live test)
(source)
zero-or-infinity-003.html
(live test)
(source)
zero-or-infinity-004.html
(live test)
(source)
zero-or-infinity-005.html
(live test)
(source)
zero-or-infinity-006.html
(live test)
(source)
zero-or-infinity-007.html
(live test)
(source)
zero-or-infinity-008.html
(live test)
(source)
zero-or-infinity-009.html
(live test)
(source)
zero-or-infinity-010.html
(live test)
(source)
This property sets a
preferred aspect ratio
for the box, which will be used in the calculation of
auto
sizes and some other layout functions.
auto
Replaced elements
with a
natural aspect ratio
use that aspect ratio; otherwise the box has no
preferred aspect ratio
. Size calculations involving the aspect ratio work with the
content box
dimensions always.
<ratio>
The box’s
preferred aspect ratio
is the specified ratio of
width
/
height
. Size calculations involving the aspect ratio work with the dimensions of the box specified by
box-sizing
.
If the
<ratio>
is
degenerate
, the property instead behaves as
auto
.
auto &&
<ratio>
If both
auto
and a
<ratio>
are specified together, the
preferred aspect ratio
is the specified ratio of
width
/
height
unless it is a
replaced element
with a
natural aspect ratio
, in which case that aspect ratio is used instead. In all cases, size calculations involving the aspect ratio work with the
content box
dimensions always.
If the
<ratio>
is
degenerate
, the property instead behaves as
auto
.
Tests
block-aspect-ratio-050.html
(live test)
(source)
Note:
Having a
preferred aspect ratio
does not make a box into a
replaced element
; layout rules specific to
replaced elements
do not generally apply to
non-replaced
boxes with a
preferred aspect ratio
. For example, a
non-replaced
absolutely-positioned
box treats
justify-self: normal
as
stretch
, not as
start
(
CSS Box Alignment 3
§ 6.1.2 Absolutely-Positioned Boxes
), even if it has a
preferred aspect ratio
CSS2.1 does not cleanly differentiate between replaced elements vs. elements with an aspect ratio; need to figure out specific cases that are unclear and define them, either in the appropriate Level 3 spec or here.
This example sets each item in the grid to render as a square, determining the number of items and their widths by the available space.
<
ul
>
<
li
>
…
<
li
>
…
<
li
>
…
<
li
>
…
</
ul
>
ul
{
display
:
grid
;
grid-template-columns
:
repeat
(
auto-fill
,
minmax
(
12
em
,
1
fr
));
}
li
{
aspect-ratio
:
1
/
1
;
overflow
:
auto
;
}
This example uses the
iframe
element’s
width
and
height
attributes to set the
aspect-ratio
property, giving the iframe an aspect ratio to use for sizing so that it behaves exactly like an image with that aspect ratio.
<
iframe
src
=
"https://www.youtube.com/embed/0Gr1XSyxZy0"
width
=
560
height
=
315
>
@supports
(
aspect-ratio:
attr
(
width number
)
/
1
)
{
iframe
{
aspect-ratio
:
attr
(
width number
)
/
attr
(
height number
);
width
:
100
%
;
height
:
auto
;
}
}
If a replaced element’s only
natural dimension
is a
natural width
or a
natural height
, giving it a
preferred aspect ratio
also gives it an
natural
height or width, whichever was missing, by transferring the existing size through the
preferred aspect ratio
.
4.2.
Effects of Preferred Aspect Ratio on Automatic Sizes
When a box has a
preferred aspect ratio
, its
automatic sizes
are calculated the same as for a
replaced element
with a
natural aspect ratio
and no
natural size
in that axis, see e.g.
CSS2 § 10
and
CSS Flexible Box Model Level 1 § 9.2
. The axis in which the
preferred size
calculation depends on this aspect ratio is called the
ratio-dependent axis
, and the resulting size is
definite
if its input sizes are also
definite
. The opposite axis (on which the
ratio-dependent axis
size depends) is the
ratio-determining axis
.
Note:
A
preferred aspect ratio
only ever has an effect if at least one of the box’s sizes is
automatic
. If neither
width
nor
height
is an
automatic size
, it can have no effect on its
preferred sizes
.
When we move all the sizing information here, rather than crowbar-ing our way into 2.1, then the core principle here is just: the resolved
preferred size
in the ratio-determining axis (before applying min/max) gets transferred thru the ratio. Min/max constraints get transferred afterwards, and then applied to each axis independently without regards to aspect-ratio.
Tests
flex-aspect-ratio-031.html
(live test)
(source)
grid-aspect-ratio-align-items-center.html
(live test)
(source)
4.2.1.
Margin-collapsing
For the purpose of margin collapsing (
CSS 2
§ 8.3.1 Collapsing margins
), if the
block axis
is the
ratio-dependent axis
, it is not considered to have a
computed
block-size
of
auto
.
4.3.
Automatic Content-based Minimum Sizes
In order to avoid unintentional overflow, the
automatic minimum size
in the
ratio-dependent axis
of a box with a
preferred aspect ratio
that is neither a
replaced element
, nor a
scroll container
in that axis, is its
min-content size
capped by its
maximum size
.
In the following example, the box is as wide as the container (as usual), and its height is as tall as needed to contain its content but at least as tall as it is wide.
div
{
aspect-ratio
:
1
/
1
;
/* 'width' and 'height' both default to 'auto' */
}
+----------+ +----------+ +----------+ | ~~~~~~~~ | | ~~~~~~~~ | | ~~~~~~~~ | | ~~~~~~~~ | | ~~~~~~~~ | | ~~~~~~~~ | | ~~~~~~~ | | ~~~~~~~~ | | ~~~~~~~~ | | | | ~~~ | | ~~~~~~~~ | +----------+ +----------+ | ~~~~~~~~ | | ~~~~~~ | +----------+
When
overflow: auto
is specified, however, even the box with excess content maintains the 1:1 aspect ratio (and handles overflow by becoming scrollable instead, as usual).
div
{
overflow
:
auto
;
aspect-ratio
:
1
/
1
;
}
+----------+ +----------+ +----------+ | ~~~~~~~~ | | ~~~~~~~~ | | ~~~~~~~~^| | ~~~~~~~~ | | ~~~~~~~~ | | ~~~~~~~~ | | ~~~~~~~ | | ~~~~~~~~ | | ~~~~~~~~ | | | | ~~~ | | ~~~~~~~~v| +----------+ +----------+ +----------+
Overriding the
min-height
property also maintains the 1:1 aspect ratio, but will result in content overflowing the box if it is not otherwise handled.
div
{
aspect-ratio
:
1
/
1
;
min-height
:
0
;
}
+----------+ +----------+ +----------+ | ~~~~~~~~ | | ~~~~~~~~ | | ~~~~~~~~ | | ~~~~~~~~ | | ~~~~~~~~ | | ~~~~~~~~ | | ~~~~~~~ | | ~~~~~~~~ | | ~~~~~~~~ | | | | ~~~ | | ~~~~~~~~ | +----------+ +----------+ +-~~~~~~~~-+ ~~~~~~
This automatic minimum operates in both axes. Consider this example:
<
div
style
=
"height: 100px; aspect-ratio: 1/1;"
>
<
span
style
=
"display: inline-block; width: 50px;"
></
span
>
<
span
style
=
"display: inline-block; width: 150px;"
></
span
>
</
div
>
The
width
of the container, being
auto
, resolves through the aspect ratio to 100px. However, its
min-width
, being
auto
, resolves to 150px. The resulting width of the container is thus 150px. To ignore the contents when sizing the container,
min-width: 0
can be specified.
Tests
block-aspect-ratio-038.html
(live test)
(source)
block-aspect-ratio-039.html
(live test)
(source)
fieldset-element-001.html
(live test)
(source)
fieldset-element-002.html
(live test)
(source)
flex-aspect-ratio-002.html
(live test)
(source)
flex-aspect-ratio-004.html
(live test)
(source)
flex-aspect-ratio-025.html
(live test)
(source)
flex-aspect-ratio-026.html
(live test)
(source)
flex-aspect-ratio-040.html
(live test)
(source)
flex-aspect-ratio-041.html
(live test)
(source)
flex-aspect-ratio-042.html
(live test)
(source)
flex-aspect-ratio-043.html
(live test)
(source)
flex-aspect-ratio-044.html
(live test)
(source)
grid-aspect-ratio-039.html
(live test)
(source)
grid-aspect-ratio-042.html
(live test)
(source)
intrinsic-size-010.html
(live test)
(source)
4.4.
Min/Max Size Transfers
Sizing constraints in either axis (the
origin
axis) are transferred through the
preferred aspect ratio
and applied to any
indefinite
minimum
,
maximum
, or
preferred
size in the other axis (the
destination
axis) as follows:
First, any
definite
minimum size
is converted and transferred from the
origin
to
destination
axis. This transferred minimum is capped by any
definite
preferred
or
maximum size
in the
destination
axis.
Then, any
definite
maximum size
is converted and transferred from the
origin
to
destination
. This transferred maximum is floored by any
definite
preferred
or
minimum size
in the
destination
axis as well as by the transferred minimum, if any.
Note:
Thus, any definite sizes are completely unaffected by a transferred constraint; and a transferred minimum will never cause an element to exceed a definite preferred/maximum size, nor will a transferred maximum cause an element to violate its preferred/minimum size.
Note:
The basic principle is that sizing constraints transfer through the aspect-ratio to the other side to preserve the aspect ratio to the extent that they can without violating any sizes specified explicitly on that affected axis. (This is the principle that drove the contents of the
constraint table in CSS2 Section 10.4
.)
In the following example:
<
div
id
=
container
style
=
"height: 100px; float: left;"
>
<
div
id
=
item
style
=
"height: 100%; aspect-ratio: 1/1;"
>
content
</
div
>
</
div
>
Since the height of the
#item
is a percentage that resolves against a definite container, the width of the item resolves to 100px for both its intrinsic size contributions as well as for final layout, so the container also sizes to a width of 100px.
<
div
id
=
container
style
=
"height: auto; float: left;"
>
<
div
id
=
item
style
=
"height: 100%; aspect-ratio: 1/1;"
>
content
</
div
>
</
div
>
In this next example, the percentage height of the item cannot be resolved and
behaves as auto
(see
CSS 2
§ 10.5 Content height: the 'height' property
). Since both axes now have an
automatic size
, the height becomes the
ratio-dependent axis
. Calculating the
intrinsic size contributions
of the box produces a width derived from its content, and a height calculated from that width and the aspect ratio, yielding a square box (and a container) sized to the width of the word “content”.
Tests
replaced-element-032.html
(live test)
(source)
replaced-element-033.html
(live test)
(source)
block-aspect-ratio-040.html
(live test)
(source)
block-aspect-ratio-041.html
(live test)
(source)
block-aspect-ratio-042.html
(live test)
(source)
block-aspect-ratio-043.html
(live test)
(source)
block-aspect-ratio-044.html
(live test)
(source)
block-aspect-ratio-045.html
(live test)
(source)
block-aspect-ratio-046.html
(live test)
(source)
block-aspect-ratio-047.html
(live test)
(source)
block-aspect-ratio-048.html
(live test)
(source)
block-aspect-ratio-049.html
(live test)
(source)
flex-aspect-ratio-039.html
(live test)
(source)
replaced-element-039.html
(live test)
(source)
replaced-element-040.html
(live test)
(source)
replaced-element-041.html
(live test)
(source)
replaced-element-043.html
(live test)
(source)
replaced-element-044.html
(live test)
(source)
image-max-width-and-height-behaves-as-auto.html
(live test)
(source)
min-max-content-orthogonal-flow-crash-001.html
(live test)
(source)
This section might not be written correctly.
[Issue #6071]
5.
Intrinsic Size Determination
5.1.
Intrinsic Sizes
CSS Sizing 3
§ 5.1 Intrinsic Sizes
5.2.
Overriding Contained Intrinsic Sizes: the
contain-intrinsic-*
properties
Name:
contain-intrinsic-width
,
contain-intrinsic-height
,
contain-intrinsic-block-size
,
contain-intrinsic-inline-size
Value:
auto
?
[ none
|
<length [0,∞]>
]
Initial:
none
Applies to:
elements with
size containment
Inherited:
no
Percentages:
n/a
Computed value:
as specified, with
<length>
values computed
Canonical order:
per grammar
Animation type:
by computed value type
Logical property group:
contain-intrinsic-size
Tests
auto-014.html
(live test)
(source)
auto-015.html
(live test)
(source)
auto-016.html
(live test)
(source)
auto-017.html
(live test)
(source)
auto-018.html
(live test)
(source)
contain-intrinsic-size-001.html
(live test)
(source)
contain-intrinsic-size-002.html
(live test)
(source)
contain-intrinsic-size-003.html
(live test)
(source)
contain-intrinsic-size-004.html
(live test)
(source)
contain-intrinsic-size-005.html
(live test)
(source)
contain-intrinsic-size-006.html
(live test)
(source)
contain-intrinsic-size-007.html
(live test)
(source)
contain-intrinsic-size-008.html
(live test)
(source)
contain-intrinsic-size-009.html
(live test)
(source)
contain-intrinsic-size-010.html
(live test)
(source)
contain-intrinsic-size-011.html
(live test)
(source)
contain-intrinsic-size-012.html
(live test)
(source)
contain-intrinsic-size-013.html
(live test)
(source)
contain-intrinsic-size-014.html
(live test)
(source)
contain-intrinsic-size-015.html
(live test)
(source)
contain-intrinsic-size-016.html
(live test)
(source)
contain-intrinsic-size-017.html
(live test)
(source)
contain-intrinsic-size-018.html
(live test)
(source)
contain-intrinsic-size-019.html
(live test)
(source)
contain-intrinsic-size-020.html
(live test)
(source)
contain-intrinsic-size-021.html
(live test)
(source)
contain-intrinsic-size-022.html
(live test)
(source)
contain-intrinsic-size-023.html
(live test)
(source)
contain-intrinsic-size-024.html
(live test)
(source)
contain-intrinsic-size-025.html
(live test)
(source)
contain-intrinsic-size-026.html
(live test)
(source)
contain-intrinsic-size-027.html
(live test)
(source)
contain-intrinsic-size-028.html
(live test)
(source)
contain-intrinsic-size-029.html
(live test)
(source)
contain-intrinsic-size-030.html
(live test)
(source)
contain-intrinsic-size-031.html
(live test)
(source)
contain-intrinsic-size-032.html
(live test)
(source)
contain-intrinsic-size-logical-001.html
(live test)
(source)
contain-intrinsic-size-logical-002.html
(live test)
(source)
contain-intrinsic-size-logical-003.html
(live test)
(source)
forget-on-disconnect-in-iframe.html
(live test)
(source)
contain-intrinsic-size-computed.html
(live test)
(source)
contain-intrinsic-size-invalid.html
(live test)
(source)
contain-intrinsic-size-valid.html
(live test)
(source)
These properties allow elements with
size containment
to specify an
explicit intrinsic inner size
, causing the box to size as if its
in-flow
content totals to a width and height matching the specified
explicit intrinsic inner size
(rather than sizing as if it were empty).
Note:
This is not always equivalent to laying out as if the element had one child of the specified
explicit intrinsic inner size
. For example, a
grid container
with one child of the specified size would still size according to the specified grid, usually ending up with a larger content size than specified.
Tests
contain-intrinsic-size-033.html
(live test)
(source)
none
|
<length>
If no other
contain-intrinsic-size
value (such as
auto
) is providing an
explicit intrinsic inner size
, the corresponding axis either doesn’t have an
explicit intrinsic inner size
(if
none
is specified) or has an
explicit intrinsic inner size
of the specified
<length>
.
auto
If
auto
is specified and the element has a
last remembered size
and is currently
skipping its contents
, its
explicit intrinsic inner size
in the corresponding axis is the
last remembered size
in that axis.
Note:
This occurs, for example, when an element with
content-visibility: auto
is off-screen.
Tests
content-visibility-058.html
(live test)
(source)
If an element has an
explicit intrinsic inner size
in an axis, then after laying out the element as normal for
size containment
, the size of the contents in that axis are instead treated as being the
explicit intrinsic inner size
instead of what was calculated in layout, and layout is performed again if necessary. (If it has an
explicit intrinsic inner size
in both axises, this implies the first layout can be skipped.)
These four properties are part of a
logical property group
.
Note:
An element with
size containment
is laid out as if it had no contents
[CSS-CONTAIN-1]
, which in many cases this will cause the element to collapse to zero inner height. This can be corrected with an explicit
height
chosen to show the expected contents, but that can have unintended effects in some layout systems, such as Flex and Grid Layout, which treat an explicit
height
as a stronger command than an implicit content-based height. The element thus might lay out substantially differently than it would have were it simply filled with content up to that height. Providing an
explicit intrinsic inner size
for the element preserves the performance benefits of ignoring its contents for layout while still allowing it to size as if it had content.
Name:
contain-intrinsic-size
Value:
[ auto
?
[ none
|
<length [0,∞]>
] ]
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
contain-intrinsic-size-interpolation.html
(live test)
(source)
contain-intrinsic-size
is a shorthand property that sets the
contain-intrinsic-width
and
contain-intrinsic-height
properties.
The first value represents the
contain-intrinsic-width
value, and the second represents the
contain-intrinsic-height
value. If only one value is given, it applies to both properties.
5.2.1.
Last Remembered Size
Size containment
is very valuable for ensuring a page can render efficiently, restricting the scope of layout work that can happen as a result of an element changing its rendering. However, it’s also very restrictive for the author, requiring them to correctly predict what the size of the element will be; if this guess is incorrect, even slightly, it can cause unsightly scrollbars or accidentally-hidden content.
The
auto
keyword of
contain-intrinsic-size
allows a middle-ground: if an element is ever
not
size-contained
, this value causes the element to remember its size (calculated as normal by layout); then, if the element gains
size containment
later, it will use the remembered size, offering the performance benefits of
size containment
while
probably
sizing accurately to its contents.
Only elements capable of being
ResizeObserver
targets can have a
last remembered size
.
The
last remembered size
of an element is determined by:
At the time that
ResizeObserver
events are determined and delivered, if an element has a
auto
keyword in
contain-intrinsic-size
property, is capable of being a
ResizeObserver
target, but does not have
size containment
, record the current inner dimensions of its
principal box
as its
last remembered size
.
At the time that
ResizeObserver
events are determined and delivered, if an element has a
last remembered size
but does
not
have
auto
keyword in
contain-intrinsic-size
property, remove its
last remembered size
.
Note:
The
last remembered size
is state attached to the
element
, not any particular box generated by the element. So long as the element retains
auto
keyword in
contain-intrinsic-size
property, it will remember its
last remembered size
even across changes such as going to/from
display: none
.
Tests
auto-001.html
(live test)
(source)
auto-002.html
(live test)
(source)
auto-003.html
(live test)
(source)
auto-004.html
(live test)
(source)
auto-005.html
(live test)
(source)
auto-006.html
(live test)
(source)
auto-007.html
(live test)
(source)
auto-008.html
(live test)
(source)
auto-009.html
(live test)
(source)
auto-010.html
(live test)
(source)
auto-011.html
(live test)
(source)
auto-012.html
(live test)
(source)
auto-013.html
(live test)
(source)
5.2.2.
Interaction With
overflow: auto
The
contain-intrinsic-size
property provides an estimate of how large the author expects the content of an element to be, but this estimate is not actual content and does not represent anything that needs to be shown to the user. Therefore, an element with
overflow: auto
must not generate scrollbars as a consequence of
contain-intrinsic-size
.
However, if
contain-intrinsic-size
indicates a size large enough that the element would generate scrollbars if it contained actual content of that size, then the element must be
sized
as if it generated those scrollbar(s) in accordance with such hypothetical content.
In the following example code:
div
{
width
:
max-content
;
contain-intrinsic-size
:
100
px
100
px
;
overflow
:
auto
;
}
The element ends up being
100px
wide and
100px
tall:
contain-intrinsic-size
provides the max-content width, and also the height.
If the element then ended up with content that was
150px
tall, it would show a vertical scrollbar; if the scrollbar is not overlay, it will take up some of that
100px
width, leaving a smaller amount (roughly
84px
, typically) for the content to flow into. (See
CSS Overflow 3
§ 4 Scrollbars and Layout
.)
Even though there’s now less than
100px
of horizontal space available for the content, it will not generate a horizontal scrollbar just because
contain-intrinsic-size
indicates a
100px
width; that would only happen if the actual content had something unbreakable and wider than the remaining space.
In contrast, in the following example code:
div
{
width
:
max-content
;
contain-intrinsic-size
:
100
px
100
px
;
height
:
50
px
;
overflow
:
auto
;
}
The element has a fixed
50px
height, but
contain-intrinsic-size
indicates a
100px
“estimated content height”. The element thus assumes that it will need a vertical scrollbar when it’s filled with actual content, resulting in a max-content width a little more than
100px
(roughly
116px
, typically), to accommodate the estimated
100px
of max-content width from
contain-intrinsic-size
, and as well as the vertical scrollbar width (roughly
16px
, typically).
However, even though the element reserves space on the assumption of needing a scrollbar, it will not actually generate one unless the actual content overflows: if it ends up containing content that’s less than 50px tall, no vertical scrollbar will be generated at all, but the element will still be
116px
wide.
5.3.
Responsively-sized iframes: the
frame-sizing
property
Name:
frame-sizing
Value:
auto
|
content-width
|
content-height
|
content-block-size
|
content-inline-size
Initial:
auto
Applies to:
replaced elements (but see below for details)
Inherited:
no
Percentages:
n/a
Computed value:
as specified
Canonical order:
per grammar
Animation type:
discrete
Some
replaced elements
can contain "normal" flowed content, such as HTML
iframe
s. For privacy and security reasons, these elements do not, by default, expose any information about their internal contents' sizing to the outside page, instead just using a static, predetermined intrinsic size, and making their contents scrollable. The
frame-sizing
property allows these elements to opt into exposing their actual content size, known as their
internal layout intrinsic size
. Values have the following meaning:
auto
The element’s
internal layout intrinsic size
, if any, is ignored.
content-width
content-height
content-block-size
content-inline-size
If the element has an
internal layout intrinsic size
, its
intrinsic size
takes the corresponding dimension (either width or height) from the
internal layout intrinsic size
. (The other dimension is determined normally.)
Logical directions resolve based on the
writing mode
of the element. (Not the embedded document.)
Which elements can have an
internal layout intrinsic size
, and how it’s determined, are decided by the
document language
. In HTML, only
iframe
elements can have an
internal layout intrinsic size
, and further, only when the contained document has also opted in via a
<
meta
name
=
responsive-embedded-sizing
>
element. (See
§ 5.3.1 HTML iframe Details
.)
When the embedded document has the following HTML:
<
meta
name
=
"responsive-embedded-sizing"
>
<
div
style
=
"height: 500px"
></
div
>
and the embedding document has the following CSS:
iframe
{
frame-sizing
:
content-height
;
}
The height of the
iframe
will initially be the default iframe height (typically
150px
), but will be updated to
500px
once the iframe’s document loads.
In addition, the internal document can call
window.requestResize()
to update its
internal layout intrinsic size
later.
5.3.1.
HTML
iframe
Details
In HTML, only the
iframe
element can have an
internal layout intrinsic size
, and only when the
iframe
’s embedded document has opted in appropriately.
An HTML
Document
object has an
responsive embedded sizing flag
associated with it, which is initially unset. It is set to true or false during the initial document parse, depending on which of the following first occurs:
If a
<
meta
name
=
responsive-embedded-sizing
>
element is encountered, it is set to true.
If the
body
element is opened (explicitly or implicitly), or inserted into the document by the parser, it is set to false.
Once set to either true or false, the flag will not change its value again for the lifetime of the
Document
.
Note:
Currently, the HTML
meta
element appearing in the
head
of an HTML document is the only way for an embedded document to opt into this feature in HTML. This means that SVG documents in
iframe
cannot do so.
Note:
Due to quirks of the HTML parser, a
meta
element can technically appear
between
the
head
and
body
elements, and will be reparented into the
head
. This is still a valid location for the
<meta name=responsive-embedded-sizing>
element, as the
body
has not yet been opened. Note that most HTML elements will implicitly open a
body
element even if the
body
start tag is not present, and if the document completely lacks such an element, an empty one is automatically generated and inserted.
The
internal layout intrinsic size
is first set by the results of the embedded document’s first layout after it fires its
DOMContentLoaded
event, and again when the
load
event is fired at the
Window
. Subsequent changes to content, styling or layout of the embedded document do not automatically affect the
internal layout intrinsic size
, but see
requestResize()
.
An embedded document can additionally have a
locked embedded ICB size
, which is initially null. If the document’s
responsive embedded sizing flag
is true when it performs a layout, and its
locked embedded ICB size
is null, it records its current
initial containing block
size as its
locked embedded ICB size
. On subsequent layouts, it uses its
locked embedded ICB size
instead of calculating its
initial containing block
size normally.
Note:
This freezing of the ICB reduces the chance of layout loops (an iframe document sizing to slightly larger than its ICB, then the parent iframe changing size to match, and the next layout again making it slightly larger than the ICB, etc to infinity).
Do we want to have a way to force an iframe to forget its
locked embedded ICB size
? Maybe turning
frame-sizing
off and on again?
Navigating the iframe’s document causes it to forget its
locked embedded ICB size
. It
does not
forget its
internal layout intrinsic size
as long as the new document’s
responsive embedded sizing flag
is unset. Once the flag is set (to true
or
false), the
internal layout intrinsic size
is forgotten; if the flag is set to true, both are then freshly computed as normal.
5.3.2.
Extensions to the
Window
Interface
partial
interface
Window
{
undefined
requestResize
(); };
When
requestResize()
is invoked:
Let
navigable
be the
navigable
that has
this
as its
content window
. If
navigable
is not a
child navigable
,
throw
a
NotAllowedError
DOMException
.
Let
host element
be the
navigable container
whose
content navigable
is
navigable
. If
host element
is not an HTML
iframe
element,
throw
a
NotAllowedError
DOMException
.
Let
document
be
this’s
active document
. If
document
’s
responsive embedded sizing flag
is unset or false,
throw
a
NotAllowedError
DOMException
.
If
document
has pending style or layout changes, perform them.
Set the
internal layout intrinsic size
of
host element
to the width and height of the
scrolling area
of
this
.
5.4.
Intrinsic Size Contributions
CSS Sizing 3
§ 5.2 Intrinsic Contributions
5.5.
Zeroing Min-Content Size Contributions: the
min-intrinsic-sizing
property
Name:
min-intrinsic-sizing
Value:
legacy
|
zero-if-scroll
||
zero-if-extrinsic
Initial:
legacy
Applies to:
all elements except
inline boxes
Inherited:
no
Percentages:
n/a
Computed value:
as specified
Canonical order:
per grammar
Animation type:
discrete
This property seriously needs some name bikeshedding.
This property defines whether the
min-content contribution
of a
non-replaced
box is “compressed” under certain circumstances. Values have the following meanings:
legacy
The box’s
min-content contribution
is handled as normal.
zero-if-scroll
The box’s
min-content contribution
is “compressed” if it is a
scroll container
in that axis.
zero-if-extrinsic
The box’s
min-content contribution
is “compressed” if has an
extrinsic
preferred
or
maximum
size.
Note:
This is the default behavior of most
replaced elements
.
The following rule will make all
scroll containers
essentially ignore their contents when passing up their size contributions (unless they specifically requested a content-based size):
*
,
::before
,
::after
{
min-intrinsic-sizing
:
zero-if-scroll
;
}
This prevents the
scroll container
from blowing up the size of its ancestors if it contains large items such as a table or long lines of unbreakable text. Meanwhile, it allows boxes that are not scroll containers to continue influencing the
min-content size
of their ancestors.
Note:
The behavior of
zero-if-scroll
would have been a better default, but due to Web-compat, it cannot be the initial value. :(
The “compressed”
min-content contributions
is calculated by pretending the box were empty, except when factoring in sizing constraints imposed by explicit
min-content
,
max-content
, and
fit-content
values of the
sizing properties
.
5.6.
New Column Sizing Values: the
stretch
,
min-content
,
max-content
,
fit-content
, and
fit-content()
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
fit-content(
<length-percentage>
)
Specifies the optimal column width as
min(
max-content size
, max(
min-content size
,
<length-percentage>
))
Note:
The column width never varies by column. When the column width is informed by the multi-column container’s contents (as in the keywords above), all of its contents are taken under consideration and the calculated width is shared by all the columns.
6.
Extrinsic Size Determination
CSS Sizing 3
§ 4 Extrinsic Size Determination
6.1.
Contain-fit Sizing: stretching while maintaining an aspect ratio
Contain-fit sizing essentially applies stretch-fit sizing, but reduces the size of the box in one axis to maintain the box’s
preferred aspect ratio
, similar to the
contain
keyword of the
object-fit
and
background-size
properties.
First, a target rectangle is determined:
The initial target rectangle is the size of the box’s containing block, with any indefinite size assumed as infinity. If both dimensions are indefinite, the initial target rectangle is set to match the outer edges of the box were it
stretch-fit sized
.
Next, if the box has a non-
none
max-width
or
max-height
, the target rectangle is clamped in the affected dimension to less than or equal to the “maximum size” of the box’s margin box, i.e. the size its margin box would be if the box was sized at its max-width/height. (Note that, consistent with normal
box-sizing rules
, this “maximum size” is floored by the effects of the box’s
min-width
/
min-height
.)
Last, the target rectangle is reduced in one dimension by the minimum necessary for it to match the box’s
preferred aspect ratio
.
The contain-fit size in each dimension is the size that would result from stretch-fitting into the target rectangle.
Copy whatever stretch-fit ends up doing wrt margin collapsing.
If there is a minimum size in one dimension that would cause overflow of the target rectangle if the aspect ratio were honored, do we honor the aspect ratio or skew the image? If the former, we need a step similar to #2 that applies the relevant minimums.
6.2.
Percentage Sizing
…
Changes
Changes since 5 May 2021 Working Draft
Significant changes since the
5 May 2021 Working Draft
include:
Added the
size
,
min-size
, and
max-size
shorthand properties
. (
Issue 820
)
Added
calc-size()
to the
sizing properties
. (
Issue 6265
)
Added
frame-sizing
property definition. (
1771
)
Imported
fit-content()
from Level 3, graduated
fit-content
and
stretch
back down to Level 3. (
Issue 10601
)
Introduced
<box-size>
grammar production to consolidate sizing values. (
Issue 13478
)
Disallowed re-ordering of values in
contain-intrinsic-size
. (
Issue 6391
).
Only used the last remembered size for the
auto
value of the
contain-intrinsic-*
properties when the element is skipping its contents to avoid phantom sizes. (
Issue 6308
).
Clarify which elements get a last remembered size and when. (
Issue 6220
)
Added logical property group for
contain-intrinsic-size
properties. (
Issue 2822
).
Changed syntax of
contain-intrinsic-size
to allow for more flexible combinations of values. (
Issue 8407
).
Disallowed negative lengths in
contain-intrinsic-*
properties. (
Issue 11945
).
Added
responsively-sized iframes
via 'contain-intrinsic-size/from-element' value. (
Issue 1771
).
Clarified that Flexbox’s
stretch-fit sizing
behavior is slightly different from
stretch
. (
Issue 11784
).
Added Web Platform Tests coverage.
Various other minor editorial fixes and improvements.
Changes since 20 October 2020 Working Draft
Significant changes since the
20 October 2020 Working Draft
include:
Drafted
min-intrinsic-sizing
property, to better control the min-content contributions of scroll containers. (
Issue 1865
,
Issue 4585
)
Added longhands to
contain-intrinsic-size
for controlling each axis independently. (
Issue 5432
)
Drafted
auto
value to
contain-intrinsic-size
to allow “remembering” the previously-calculated size. (
Issue 5668
,
Issue 5815
)
Defined handling of degenerate ratios in
aspect-ratio
. (
Issue 5557
)
Defined how
aspect-ratio
impacts a replaced element’s natural sizes. (
Issue 5306
)
Fixed some errors in the
§ 4.4 Min/Max Size Transfers
section, aligning the behavior to not conflict with behavior defined by CSS2 / CSS Flex Layout / etc. (
Issue 6071
)
Added
contain-intrinsic-size: auto none
syntax.
Significant changes since the
26 May 2020 First Public Working Draft
include:
Define
ratio-determining axis
as a term.
Define that min/max sizing constraints are transferred across an aspect-ratio, (
Issue 5257
)
Additionally, sizing constraints in either axis (the
origin
axis) are transferred through the
preferred aspect ratio
to the other axis (the
destination
axis) as follows:
First, any
definite
minimum size
is converted and transferred from the
origin
to
destination
axis. This transferred minimum is capped by any
definite
preferred
or
maximum size
in the
destination
axis.
Then, any
definite
maximum size
is converted and transferred from the
origin
to
destination
. This transferred maximum is floored by any
definite
preferred
or
minimum size
in the
destination
axis as well as by the transferred minimum, if any.
Note:
The basic principle is that sizing constraints transfer through the aspect-ratio to the other side to preserve the aspect ratio to the extent that they can without violating any sizes specified explicitly on that affected axis.
Clarify that
aspect-ratio
on a
replaced element
with only one
natural size
determines the other dimension. (
Issue 5306
)
If a replaced element’s only natural dimension is a natural width or a natural height, giving it a
preferred aspect ratio
also gives it a natural height or width, whichever was missing, by transferring the existing size through the
preferred aspect ratio
.
Define that
aspect-ratio
inhibits margin self-collapsing (
Issue 5328
)
For the purpose of margin collapsing (
CSS 2
§ 8.3.1 Collapsing margins
), if the
block axis
is the
ratio-dependent axis
, it is not considered to have a
computed
block-size
of
auto
.
Additions Since Level 3
Added
fit-content()
function and
contain
keyword for sizing properties.
Added
aspect-ratio
property.
Added
contain-intrinsic-width
,
contain-intrinsic-height
,
contain-intrinsic-block-size
, and
contain-intrinsic-inline-size
properties and their shorthand
contain-intrinsic-size
.
Added
min-intrinsic-sizing
property.
Added
frame-sizing
property.
Acknowledgments
Special thanks go to Aaron Gustafson, L. David Baron for their contributions to this module.
Privacy Considerations
This specification introduces no new privacy considerations.
Security Considerations
This specification introduces no new security considerations.
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
aspect-ratio
, in § 4.1
auto
value for aspect-ratio
, in § 4.1
value for contain-intrinsic-width, contain-intrinsic-height, contain-intrinsic-block-size, contain-intrinsic-inline-size, contain-intrinsic-size
, in § 5.2
value for frame-sizing
, in § 5.3
auto && <ratio>
, in § 4.1
<box-size>
, in § 3.2
calc-size()
, in § 3.2
contain
, in § 3.2
contain-fit sizing
, in § 6
contain-intrinsic-block-size
, in § 5.2
contain-intrinsic-height
, in § 5.2
contain-intrinsic-inline-size
, in § 5.2
contain-intrinsic-size
, in § 5.2
contain-intrinsic-width
, in § 5.2
content-block-size
, in § 5.3
content-height
, in § 5.3
content-inline-size
, in § 5.3
content-width
, in § 5.3
explicit intrinsic inner size
, in § 5.2
fit-content
, in § 5.6
fit-content()
, in § 3.2
fit-content(<length-percentage>)
, in § 5.6
frame-sizing
, in § 5.3
internal layout intrinsic size
, in § 5.3
last remembered size
, in § 5.2
legacy
, in § 5.5
<length>
, in § 5.2
locked embedded ICB size
, in § 5.3.1
max-content
, in § 5.6
max-size
, in § 3.1
min-content
, in § 5.6
min-intrinsic-sizing
, in § 5.5
min-size
, in § 3.1
none
, in § 5.2
preferred aspect ratio
, in § 4.1
<ratio>
, in § 4.1
ratio-dependent axis
, in § 4.2
ratio-determining axis
, in § 4.2
requestResize()
, in § 5.3.2
responsive embedded sizing flag
, in § 5.3.1
responsively-sized iframe
, in § 5.2.2
size
, in § 3.1
stretch
, in § 5.6
zero-if-extrinsic
, in § 5.5
zero-if-scroll
, in § 5.5
Terms defined by reference
[CSS-ALIGN-3]
defines the following terms:
justify-self
start
stretch
(for align-self)
stretch
(for justify-self)
[CSS-BACKGROUNDS-3]
defines the following terms:
background-size
[CSS-BOX-4]
defines the following terms:
content box
[CSS-CASCADE-5]
defines the following terms:
computed value
shorthand
shorthand property
[CSS-CONTAIN-2]
defines the following terms:
content-visibility
size containment
skipping its contents
[CSS-DISPLAY-4]
defines the following terms:
display
in-flow
initial containing block
inline box
non-replaced
principal box
replaced element
[CSS-GRID-2]
defines the following terms:
grid container
[CSS-IMAGES-3]
defines the following terms:
natural aspect ratio
natural dimension
natural height
natural size
natural width
[CSS-IMAGES-4]
defines the following terms:
contain
object-fit
[CSS-LOGICAL-1]
defines the following terms:
block-size
logical property group
[CSS-MULTICOL-2]
defines the following terms:
column-width
[CSS-OVERFLOW-3]
defines the following terms:
overflow
scroll container
[CSS-PAGE-3]
defines the following terms:
@page
size
[CSS-POSITION-3]
defines the following terms:
absolutely-positioned
[CSS-SIZING-3]
defines the following terms:
auto
automatic minimum size
automatic size
available space
behaves as auto
box-sizing
definite
extrinsic sizing
height
indefinite
intrinsic size
intrinsic size contribution
max-content
max-content inline size
max-content size
maximum size
min-content
min-content contribution
min-content inline size
min-content size
minimum size
none
preferred size
sizing property
stretch-fit inline size
stretch-fit size
width
[CSS-SIZING-4]
defines the following terms:
fit-content
[CSS-VALUES-4]
defines the following terms:
<length-percentage>
<length>
<ratio>
?
CSS-wide keywords
degenerate ratio
{A,B}
|
||
[CSS-WRITING-MODES-4]
defines the following terms:
block axis
writing mode
[CSS2]
defines the following terms:
document language
max-height
max-width
min-height
min-width
[CSSOM-VIEW-1]
defines the following terms:
scrolling area
[DOM]
defines the following terms:
Document
[HTML]
defines the following terms:
DOMContentLoaded
Window
active document
body
child navigable
content navigable
content window
head
iframe
load
meta
navigable
navigable container
[RESIZE-OBSERVER-1]
defines the following terms:
ResizeObserver
[WEBIDL]
defines the following terms:
DOMException
NotAllowedError
this
throw
undefined
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
[CSS-CASCADE-5]
Elika Etemad; Miriam Suzanne; Tab Atkins Jr..
CSS Cascading and Inheritance Level 5
. 13 January 2022. CR. URL:
https://www.w3.org/TR/css-cascade-5/
[CSS-CONTAIN-1]
Tab Atkins Jr.; Florian Rivoal.
CSS Containment Module Level 1
. 25 June 2024. REC. URL:
https://www.w3.org/TR/css-contain-1/
[CSS-CONTAIN-2]
Tab Atkins Jr.; Florian Rivoal; Vladimir Levin.
CSS Containment Module Level 2
. 17 September 2022. WD. URL:
https://www.w3.org/TR/css-contain-2/
[CSS-DISPLAY-4]
Elika Etemad; Tab Atkins Jr..
CSS Display Module Level 4
. 6 November 2025. WD. URL:
https://www.w3.org/TR/css-display-4/
[CSS-IMAGES-3]
Tab Atkins Jr.; Elika Etemad; Lea Verou.
CSS Images Module Level 3
. 18 December 2023. CRD. URL:
https://www.w3.org/TR/css-images-3/
[CSS-IMAGES-4]
Elika Etemad; Tab Atkins Jr.; Lea Verou.
CSS Images Module Level 4
. 30 September 2025. WD. URL:
https://www.w3.org/TR/css-images-4/
[CSS-LOGICAL-1]
Elika Etemad; Rossen Atanassov.
CSS Logical Properties and Values Module Level 1
. 4 December 2025. WD. URL:
https://www.w3.org/TR/css-logical-1/
[CSS-MULTICOL-2]
Florian Rivoal; Rachel Andrew.
CSS Multi-column Layout Module Level 2
. 19 December 2024. FPWD. URL:
https://www.w3.org/TR/css-multicol-2/
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
[CSS-VALUES-5]
Tab Atkins Jr.; Elika Etemad; Miriam Suzanne.
CSS Values and Units Module Level 5
. 11 November 2024. WD. URL:
https://www.w3.org/TR/css-values-5/
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
[CSSOM-VIEW-1]
Simon Fraser; Emilio Cobos Álvarez.
CSSOM View Module
. 16 September 2025. WD. URL:
https://www.w3.org/TR/cssom-view-1/
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
[RESIZE-OBSERVER-1]
Aleks Totic; Greg Whitworth.
Resize Observer
. 11 February 2020. FPWD. URL:
https://www.w3.org/TR/resize-observer-1/
[RFC2119]
S. Bradner.
Key words for use in RFCs to Indicate Requirement Levels
. March 1997. Best Current Practice. URL:
https://datatracker.ietf.org/doc/html/rfc2119
[WEBIDL]
Edgar Chen; Timothy Gu.
Web IDL Standard
. Living Standard. URL:
https://webidl.spec.whatwg.org/
Non-Normative References
[CSS-GRID-2]
Tab Atkins Jr.; et al.
CSS Grid Layout Module Level 2
. 26 March 2025. CRD. URL:
https://www.w3.org/TR/css-grid-2/
[CSS-POSITION-3]
Elika Etemad; Tab Atkins Jr..
CSS Positioned Layout Module Level 3
. 7 October 2025. WD. URL:
https://www.w3.org/TR/css-position-3/
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
aspect-ratio
auto || <ratio>
auto
all elements except inline boxes and internal ruby or table boxes
no
n/a
by computed value
per grammar
specified keyword or a pair of numbers
contain-intrinsic-block-size
auto? [ none | <length [0,∞]> ]
none
elements with size containment
no
n/a
by computed value type
per grammar
as specified, with <length> values computed
contain-intrinsic-size
contain-intrinsic-height
auto? [ none | <length [0,∞]> ]
none
elements with size containment
no
n/a
by computed value type
per grammar
as specified, with <length> values computed
contain-intrinsic-size
contain-intrinsic-inline-size
auto? [ none | <length [0,∞]> ]
none
elements with size containment
no
n/a
by computed value type
per grammar
as specified, with <length> values computed
contain-intrinsic-size
contain-intrinsic-size
[ auto? [ none | <length [0,∞]> ] ]{1,2}
see individual properties
see individual properties
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
contain-intrinsic-width
auto? [ none | <length [0,∞]> ]
none
elements with size containment
no
n/a
by computed value type
per grammar
as specified, with <length> values computed
contain-intrinsic-size
frame-sizing
auto | content-width | content-height | content-block-size | content-inline-size
auto
replaced elements (but see below for details)
no
n/a
discrete
per grammar
as specified
max-size
<'max-width'> <'max-height'>?
none
all elements
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
min-intrinsic-sizing
legacy | zero-if-scroll || zero-if-extrinsic
legacy
all elements except inline boxes
no
n/a
discrete
per grammar
as specified
min-size
<'min-width'> <'min-height'>?
auto
all elements
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
size
<'width'> <'height'>?
auto
all elements
see individual properties
see individual properties
see individual properties
per grammar
see individual properties
IDL Index
partial
interface
Window
{
undefined
requestResize
(); };
Issues Index
This is a diff spec over
CSS Sizing Level 3
. It is currently an Exploratory Working Draft: if you are implementing anything, please use Level 3 as a reference. We will merge the Level 3 text into this draft once it reaches CR.
↵
CSS Sizing 3
§ 2 Terminology
↵
CSS Sizing 3
§ 3 Specifying Box Sizes
↵
CSS Sizing 3
§ 3.1 Sizing Properties
↵
The
size
property needs to be omitted from the
preferred shorthand order
in CSSOM, to avoid compat problems and conflicts with
@page
.
↵
CSS Sizing 3
§ 3.2 Sizing Values: the <length-percentage>, auto | none, min-content, max-content, and fit-content() values
↵
We are still working through the details of this section. If there is any behavior specified here that would cause
replaced elements
with a
preferred aspect ratio
to behave differently than they would under the requirements of the
CSS2
,
Flex Layout
, and
Grid Layout
specs combined (without this specification in effect),
this is an error and should be
reported
to the CSSWG
. There is a
list of open aspect-ratio issues
.
↵
CSS2.1 does not cleanly differentiate between replaced elements vs. elements with an aspect ratio; need to figure out specific cases that are unclear and define them, either in the appropriate Level 3 spec or here.
↵
When we move all the sizing information here, rather than crowbar-ing our way into 2.1, then the core principle here is just: the resolved
preferred size
in the ratio-determining axis (before applying min/max) gets transferred thru the ratio. Min/max constraints get transferred afterwards, and then applied to each axis independently without regards to aspect-ratio.
↵
This section might not be written correctly.
[Issue #6071]
↵
CSS Sizing 3
§ 5.1 Intrinsic Sizes
↵
Do we want to have a way to force an iframe to forget its
locked embedded ICB size
? Maybe turning
frame-sizing
off and on again?
↵
CSS Sizing 3
§ 5.2 Intrinsic Contributions
↵
This property seriously needs some name bikeshedding.
↵
CSS Sizing 3
§ 4 Extrinsic Size Determination
↵
Copy whatever stretch-fit ends up doing wrt margin collapsing.
↵
If there is a minimum size in one dimension that would cause overflow of the target rectangle if the aspect ratio were honored, do we honor the aspect ratio or skew the image? If the former, we need a step similar to #2 that applies the relevant minimums.
↵
```
