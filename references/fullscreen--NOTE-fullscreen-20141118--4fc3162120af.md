Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Fullscreen](https://www.w3.org/TR/2014/NOTE-fullscreen-20141118/).

Original copyright notice: Copyright © 2012 W3C® (MIT, ERCIM, Keio, Beihang), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Fullscreen

Source snapshot: https://www.w3.org/TR/2014/NOTE-fullscreen-20141118/

Snapshot SHA-256: 4fc3162120af3e4d11c986a397d5285cdaeeb224e3804e00b2961bd89611cff1

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.

---

# Fullscreen

[Copyright](https://www.w3.org/Consortium/Legal/ipr-notice#Copyright) © 2012 [W3C](https://www.w3.org/)<sup>®</sup> ([MIT](http://www.csail.mit.edu/), [ERCIM](http://www.ercim.org/), [Keio](http://www.keio.ac.jp/), [Beihang](http://ev.buaa.edu.cn/)), All Rights Reserved. W3C [liability](https://www.w3.org/Consortium/Legal/ipr-notice#Legal_Disclaimer), [trademark](https://www.w3.org/Consortium/Legal/ipr-notice#W3C_Trademarks) and [document use](https://www.w3.org/Consortium/Legal/copyright-documents) rules apply.

------------------------------------------------------------------------

## <a id="abstract"></a>Abstract

Fullscreen defines the fullscreen API for the web platform.

## <a id="status-of-this-document"></a><a id="sotd"></a>Status of this Document

<i>This section describes the status of this document at the time of its
publication. Other documents may supersede this document. A list of current W3C
publications and the latest revision of this technical report can be found in
the <a href="https://www.w3.org/TR/">W3C technical reports index</a> at
http&#58;//www&#46;w3&#46;org/TR/.</i>

This is the 18 November 2014 W3C Working Group Note of Fullscreen.

This document was jointly produced by the [Web Applications Working Group](https://www.w3.org/2008/webapps/) and the [CSS Working Group](https://www.w3.org/Style/CSS/members). The Web Applications Working Group is part of the [Rich Web Clients Activity](https://www.w3.org/2006/rwc/Activity) and the CSS Working Group is part of the [Style Activity](https://www.w3.org/Style/). Both of these Working Groups are part of the the W3C [Interaction Domain](https://www.w3.org/Interaction/).

<strong>Work on this document has been discontinued and it should not be 
 referenced or used as a basis for implementation. At the time this document
 was published, the WHATWG was working on a 
 <a href="https://fullscreen.spec.whatwg.org">Fullscreen API</a> specification.</strong>

Comments related to the API part of this document should be sent to the [public-webapps](mailto:public-webapps@w3.org?subject=%5Bfullscreen-api%5D%20) mail list ([archived](http://lists.w3.org/Archives/Public/public-webapps/)) with a subject header of `[fullscreen]` and comments related to the rendering part of this document should be sent to the [www-style](mailto:www-style@w3.org?subject=%5Bfullscreen-css%5D%20) mail list ([archived](http://lists.w3.org/Archives/Public/www-style/)) with a subject header of `[fullscreen]`. Alternatively, [file a bug](https://www.w3.org/Bugs/Public/enter_bug.cgi?product=WebAppsWG&component=Fullscreen).

Publication as a Working Group Note does not imply endorsement by the W3C Membership. This is a draft document and may be updated, replaced or obsoleted by other documents at any time. It is inappropriate to cite this document as other than work in progress.

This document was produced by groups operating under the [5 February 2004 W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20040205/). W3C maintains a [public list of any patent disclosures (WebApps)](https://www.w3.org/2004/01/pp-impl/42538/status) and a [public list of any patent disclosures (CSS)](https://www.w3.org/2004/01/pp-impl/32061/status) made in connection with the deliverables of each group; these pages also include instructions for disclosing a patent. An individual who has actual knowledge of a patent which the individual believes contains [Essential Claim(s)](https://www.w3.org/Consortium/Patent-Policy-20040205/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/Consortium/Patent-Policy-20040205/#sec-Disclosure).

<a id="w3c_process_revision"></a>

This document is governed by the [14 October 2005 W3C Process Document](https://www.w3.org/2005/10/Process-20051014/).
