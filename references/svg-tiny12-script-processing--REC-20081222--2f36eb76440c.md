# SVG Tiny 1.2: HTML-delegated script processing

This bounded source reference supports implementation of the selected ordinary
offline Disabled HTML construction operation. It is not a replacement technical
specification or evidence of SVG execution, fetching or whole SVG conformance.
The original English Recommendation remains authoritative. The separately
recorded Surgeist reconciliation controls suppression of acquisition/execution;
no suppression step has been inserted into the original algorithm below.

Original: [SVG Tiny 1.2, Scripting §15.2.1](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptContentProcessing),
W3C Recommendation 22 December 2008. Complete chapter SHA-256:
`2f36eb76440c913cb28d706afb50b72056008f3a959eb2d7ef9ea2658ec3fb67`.
The HTML-linked alias was byte-identical when captured on 2026-10-12.
The original edition page SHA-256 is
`9b859abf015408649bfe6f74524db15cd4bd8a03ab54c7e33ce6a1cb55e082e1`.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document
includes material copied from or derived from
[Scalable Vector Graphics (SVG) Tiny 1.2 Specification](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/).
This is the required implementation-support derivative notice under the
[W3C Document License 2015](../licenses/w3c/document-license-2015.txt);
it does not replace the exact original 2008 copyright below.
The [historical 2002 document-use terms](../licenses/w3c/document-license-2002.txt)
remain available. The [5 February 2015 official license update](https://www.w3.org/news/2015/w3c-updates-general-document-license/)
applies its derivative-support permission to existing publications.
No project software license is changed.

Changes: bounded extraction; Markdown presentation; preservation of selected
heading IDs; replacement of relative source links with dated absolute source
URLs; normalized inter-element/line-break whitespace. Original prose, conditions,
algorithm order and link labels are unchanged. Raw ranges are edition index
15–49 (title, editors and original copyright), index 74–146 (complete original
status), and script 91–198 (processing algorithm, notification qualification and
following modification rules). The remaining script chapter, including language
execution, attributes, examples and §15.7 content delivery, is retained in the raw
capture but not transcribed here. No embedded executable example is included.

The [source-edition mapping](SOURCE-EDITIONS.md#offline-disabled-svg-script-and-clone-sources)
records the separate offline and clone determinations. The retained algorithm
below is the original source, with the conversion bounds stated above.

## Original edition notice and status

<a id="pagetitle"></a>
### Scalable Vector Graphics (SVG) Tiny 1.2 Specification

<a id="pagesubtitle"></a>
### W3C Recommendation *22 December 2008*

This version:

[http://www.w3.org/TR/2008/REC-SVGTiny12-20081222/](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/)

Latest version:

[http://www.w3.org/TR/SVGTiny12/](https://www.w3.org/TR/SVGTiny12/)

Previous version:

[http://www.w3.org/TR/2008/PR-SVGTiny12-20081117/](https://www.w3.org/TR/2008/PR-SVGTiny12-20081117/)

Editors:

Ola Andersson (Ikivo) &lt;[ola.andersson@ikivo.com](mailto:ola.andersson@ikivo.com)&gt;

Robin Berjon (Expway) &lt;[robin.berjon@expway.fr](mailto:robin.berjon@expway.fr)&gt;

Erik Dahlström (Opera Software) &lt;ed@opera.com&gt;

Andrew Emmons (BitFlash) &lt;[andrew.emmons@bitflash.com](mailto:andrew.emmons@bitflash.com)&gt;

Jon Ferraiolo (Adobe Systems until May 2006) &lt;[jon.ferraiolo@adobe.com](mailto:jon.ferraiolo@adobe.com)&gt;

Anthony Grasso (Canon, Inc.) &lt;[anthony.grasso@cisra.canon.com.au](mailto:anthony.grasso@cisra.canon.com.au)&gt;

Vincent Hardy (Sun Microsystems, Inc.) &lt;[vincent.hardy@sun.com](mailto:vincent.hardy@sun.com)&gt;

Scott Hayman (Research In Motion Limited)

[Dean Jackson](https://www.w3.org/People/Dean/) (W3C) &lt;[dean@w3.org](mailto:dean@w3.org)&gt;

[Chris Lilley](https://www.w3.org/People/chris/) (W3C) &lt;[chris@w3.org](mailto:chris@w3.org)&gt;

[Cameron McCormack](http://mcc.id.au/) (Invited Expert) &lt;[cam@mcc.id.au](mailto:cam@mcc.id.au)&gt;

Andreas Neumann (ETH Zurich)

Craig Northway (Canon, Inc.) &lt;[craign@cisra.canon.com.au](mailto:craign@cisra.canon.com.au)&gt;

Antoine Quint (Invited Expert) &lt;[aq@fuchsia-design.com](mailto:aq@fuchsia-design.com)&gt;

Nandini Ramani (Sun Microsystems)

[Doug Schepers](https://www.w3.org/People/Schepers/) (W3C) &lt;[schepers@w3.org](mailto:schepers@w3.org)&gt;

Andrew Shellshear (Canon, Inc.)

Authors:

See [author list](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/#AuthorList).

Please refer to the [**errata**](https://www.w3.org/2008/12/REC-SVGTiny12-20081222-errata.html) for this document, which may include some normative corrections.

This document is also available in these non-normative formats: a [single-page version](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/single-page.html), a [zip archive of HTML](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/REC-SVGTiny12-20081222.zip) (without external dependencies), and a [PDF](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/REC-SVGTiny12-20081222.pdf). See also [**translations**](http://www.w3.org/2003/03/Translations/byTechnology?technology=SVGTiny12), noting that the English version of this specification is the only normative version.

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2008 [W3C](https://www.w3.org/)® ([MIT](http://www.csail.mit.edu/), [ERCIM](http://www.ercim.org/), [Keio](http://www.keio.ac.jp/)), All Rights Reserved. W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [document use](https://www.w3.org/Consortium/Legal/copyright-documents) rules apply.

<a id="status"></a>
### Status of this document

*This section describes the status of this document at the time of its publication. Other documents may supersede this document. A list of current W3C publications and the latest revision of this technical report can be found in the [W3C technical reports index](https://www.w3.org/TR/) at http://www.w3.org/TR/.*

This is the 22 December 2008 Recommendation of SVG Tiny 1.2.

This document has been reviewed by W3C Members, by software developers, and by other W3C groups and interested parties, and is endorsed by the Director as a W3C Recommendation. It is a stable document and may be used as reference material or cited from another document. W3C's role in making the Recommendation is to draw attention to the specification and to promote its widespread deployment. This enhances the functionality and interoperability of the Web.

The [SVG Working Group](https://www.w3.org/Graphics/SVG/) working closely with the developer community, has produced an [implementation report](https://www.w3.org/Graphics/SVG/1.2/Tiny/ImpReport.html) to prove the implementability of this specification. Previous drafts for this specification resulted in a number of comments which have been addressed by the [SVG Working Group](https://www.w3.org/Graphics/SVG/), with a [Disposition of Comments](https://www.w3.org/Graphics/SVG/1.2/Tiny/dc.html) available on the W3C SVG site. A list of changes made since the Proposed Recommendation Working Draft is available in [Appendix T](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/changes.html).

As described in the abstract, this specification represents the core for a set of modular extensions, but is named SVG Tiny for historical reasons, as a profile for mobile devices. Future versions of this specification will maintain backwards compatibility with previous versions of the language, in a continuing line of technology, but will bear the name "SVG Core" to represent this relationship.

Please send questions or comments regarding the SVG 1.2 Tiny specification to [www-svg@w3.org](mailto:www-svg@w3.org), the public email list for issues related to SVG. This list is [archived](http://lists.w3.org/Archives/Public/www-svg/) and acceptance of this archiving policy is requested automatically upon first post. To subscribe to this list send an email to [www-svg-request@w3.org](mailto:www-svg-request@w3.org) with the word "subscribe" in the subject line.

This document has been produced by the [SVG Working Group](https://www.w3.org/Graphics/SVG/) as part of the W3C [Graphics Activity](https://www.w3.org/Graphics/Activity), following the procedures set out for the W3C [Process](https://www.w3.org/Consortium/Process/). The authors of this document are listed at the end in the [Author List](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/#AuthorList) section.

This document was produced by a group operating under the [5 February 2004 W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20040205/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/2004/01/pp-impl/19480/status) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20040205/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20040205/#sec-Disclosure).

## Original processing clauses

<a id="ScriptContentProcessing"></a>
### 15.2.1 Script processing

Execution of a given ['script'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptElement) element occurs at most once. There is a conceptual flag associated with each ['script'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptElement) element (referred to here as the "already processed" flag) that enforces this behavior. When a ['script'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptElement) element is executed depends on the method by which the element was inserted into the document.

One way for a ['script'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptElement) element to be inserted into the document is if it was inserted while parsing the document. As mentioned in [Progressive rendering](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/struct.html#ProgressiveRendering), as the document is parsed if a ['script'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptElement) element is encountered then it will be processed just after its *end element* event occurs, but before any more of the document is parsed and further nodes inserted into the document. (See below for a description of what it means for a ['script'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptElement) element to be processed.) Once processed, parsing of the document resumes.

The other way a ['script'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptElement) element can be inserted into the document is if it was inserted by something other than the parser (such as by other script executing). In this case, as soon as one or more ['script'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptElement) elements are inserted into the document, they must be processed one by one in document order.

A ['script'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptElement) element is processed as follows:

1. If the ['script'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptElement) element's "already processed" flag is true or if the element is not in the document tree, then no action is performed and these steps are ended.

2. If the ['script'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptElement) element references external script content, then the external script content using the current value of the ['xlink:href'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptElementHrefAttribute) attribute is fetched. Further processing of the ['script'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptElement) element is dependent on the external script content, and will block here until the resource has been fetched or is determined to be an [invalid IRI reference](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermInvalidIRIReference).

3. The ['script'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptElement) element's "already processed" flag is set to true.

4. If the script content is inline, or if it is external and was fetched successfully, then the script is executed. Note that at this point, these steps may be re-entrant if the execution of the script results in further ['script'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptElement) elements being inserted into the document.

Note that a [`load`](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/interact.html#EventsLoad) event is dispatched on a ['script'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptElement) element once it has been processed, unless it referenced external script content with an [invalid IRI reference](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/intro.html#TermInvalidIRIReference) and ['externalResourcesRequired'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/struct.html#ExternalResourcesRequiredAttribute) was set to 'true'.

Modifying or removing the ['script'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptElement) element (or content) after the script has started its execution must have no effect on the script execution.

Modifying a ['script'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptElement) element's ['xlink:href'](https://www.w3.org/TR/2008/REC-SVGTiny12-20081222/script.html#ScriptElementHrefAttribute) attribute after its "already processed" flag is set to true will not cause any new script content to be fetched or executed.

