# execCommand: bounded editability definitions

Source: [https://github.com/w3c/editing/blob/85de9ff4a9816139d06fc2dc39d5dacdc456fddc/docs/execCommand/index.html](https://github.com/w3c/editing/blob/85de9ff4a9816139d06fc2dc39d5dacdc456fddc/docs/execCommand/index.html); retrieved 2026-10-11. Complete source SHA-256 `da222a4ea70349eba9d9036a20708f1fa63690118f6c657da3f7f06c56fa78cf`. Incomplete draft; only the HTML-delegated definitions below are selected, not command execution. HTML editing-host semantics come from HTML revision 877d6146487f15b15bc8dd0e3c625063a37b456a.

Conversion/extraction by Surgeist on 2026-10-11 using Pandoc 3.1.11.1: selected paragraphs, absolute links, no script execution. Unexpanded Bikeshed tokens remain as authored. No semantic changes or endorsement.

## Original document notice

Copyright © 2015 This specification may be used under the terms of the CC0 1.0 Universal License and the W3C Community Contributor License Agreement (CLA). Thus anyone may reuse, modify, and redistribute them without restriction. W3C Editing APIs Community Group & W3C® (MIT, ERCIM, Keio, Beihang). W3C liability, trademark and document use rules apply.

Repository LICENSE.md at the same revision licenses all documents under the [W3C Software and Document License](https://www.w3.org/Consortium/Legal/copyright-software). The linked notice is reproduced below.

## Selected definitions

<div>

An <span id="html-element" class="dfn">HTML element</span> is an {{Element}} whose \[=Element/namespace=\] is the <a href="http://dom.spec.whatwg.org/#html-namespace" class="external" data-anolis-spec="dom">HTML namespace</a>.

Something is <span id="editable" class="dfn">editable</span> if it is a <a href="http://dom.spec.whatwg.org/#concept-node" class="external" data-anolis-spec="dom" title="concept-node">node</a>; it is not an \[=editing host=\]; it does not have a ` `[`contenteditable`](http://www.whatwg.org/specs/web-apps/current-work/multipage/editing.html#attr-contenteditable) attribute set to the false state; its \[=tree/parent=\] is an \[=editing host=\] or [editable](https://w3c.github.io/editing/docs/execCommand/#editable); and either it is an [HTML element](https://w3c.github.io/editing/docs/execCommand/#html-element), or it is an ` `[`svg`](http://www.whatwg.org/specs/web-apps/current-work/multipage/the-map-element.html#svg) or ` `[`math`](http://www.whatwg.org/specs/web-apps/current-work/multipage/the-map-element.html#math) element, or it is not an {{Element}} and its \[=tree/parent=\] is an [HTML element](https://w3c.github.io/editing/docs/execCommand/#html-element).

An [editable](https://w3c.github.io/editing/docs/execCommand/#editable) node cannot be a \[=document=\] or {{DocumentFragment}}, its \[=tree/parent=\] cannot be null, and it must descend from either an {{Element}} or a \[=document=\].

The <span id="editing-host-of" class="dfn">editing host of</span> `node` is null if `node` is neither [editable](https://w3c.github.io/editing/docs/execCommand/#editable) nor an \[=editing host=\]; `node` itself, if `node` is an \[=editing host=\]; or the nearest \[=tree/ancestor=\] of `node` that is an \[=editing host=\], if `node` is [editable](https://w3c.github.io/editing/docs/execCommand/#editable).

</div>

## W3C license notice

<div>

<div id="main" role="main">

<div class="content">

# Software and Document license - 2023 version

<div class="l-sidebar">

<div>

<div class="sidebar">

<span lang="en">Read this page in:</span>

</div>

<div class="not-sidebar">

<div class="l-cluster">

- <a href="https://www.w3.org/zh-hans/copyright/software-license-2023/" hreflang="zh-Hans" lang="zh-Hans" dir="auto">中文（简体）</a>
- <a href="https://www.w3.org/ja/copyright/software-license-2023/" hreflang="ja" lang="ja" dir="auto">日本語</a>

</div>

</div>

</div>

</div>

**Status:** This document is in effect since 1 January 2023.

------------------------------------------------------------------------

This work is being provided by the copyright holders under the following license.

## License

By obtaining and/or copying this work, you (the licensee) agree that you have read, understood, and will comply with the following terms and conditions.

Permission to copy, modify, and distribute this work, with or without modification, for any purpose and without fee or royalty is hereby granted, provided that you include the following on ALL copies of the work or portions thereof, including modifications:

- The full text of this NOTICE in a location viewable to users of the redistributed or derivative work.
- Any pre-existing intellectual property disclaimers, notices, or terms and conditions. If none exist, the [W3C software and document short notice](https://www.w3.org/Consortium/Legal/2023/copyright-software-short-notice.html) should be included.
- Notice of any changes or modifications, through a copyright statement on the new code or document such as "This software or document includes material copied from or derived from \[title and URI of the W3C document\]. Copyright © \[\$year-of-document\] [World Wide Web Consortium](https://www.w3.org/). <https://www.w3.org/copyright/software-license-2023/>"

## Disclaimers

THIS WORK IS PROVIDED "AS IS," AND COPYRIGHT HOLDERS MAKE NO REPRESENTATIONS OR WARRANTIES, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO, WARRANTIES OF MERCHANTABILITY OR FITNESS FOR ANY PARTICULAR PURPOSE OR THAT THE USE OF THE SOFTWARE OR DOCUMENT WILL NOT INFRINGE ANY THIRD PARTY PATENTS, COPYRIGHTS, TRADEMARKS OR OTHER RIGHTS.

COPYRIGHT HOLDERS WILL NOT BE LIABLE FOR ANY DIRECT, INDIRECT, SPECIAL OR CONSEQUENTIAL DAMAGES ARISING OUT OF ANY USE OF THE SOFTWARE OR DOCUMENT.

The name and trademarks of copyright holders may NOT be used in advertising or publicity pertaining to the work without specific, written prior permission. Title to copyright in this work will at all times remain with copyright holders.

## Versions

- This version: <https://www.w3.org/copyright/software-license-2023/>
- Latest version: <https://www.w3.org/copyright/software-license/>
- Previous version: <https://www.w3.org/copyright/software-license-2015/>

### Changes since the previous document

- The 2023 version updates identification of the copyright holder.

<div class="related">

## In this section

- [BSD test suite license](https://www.w3.org/copyright/3-clause-bsd-license-2008/)
- [Document license - 2002 version](https://www.w3.org/copyright/document-license-2002/)
- [Document license - 2015 version](https://www.w3.org/copyright/document-license-2015/)
- [Document license - 2023 version](https://www.w3.org/copyright/document-license-2023/)
- [Intellectual rights](https://www.w3.org/copyright/intellectual-rights/)
- [Software and Document license - 2002 version](https://www.w3.org/copyright/software-license-2002/)
- [Software and Document license - 2015 version](https://www.w3.org/copyright/software-license-2015/)
- <a href="https://www.w3.org/copyright/software-license-2023/" aria-current="page">Software and Document license - 2023 version</a>
- [Test suites licenses](https://www.w3.org/copyright/test-suites-licenses/)
- [W3C test suite license - 2008 version](https://www.w3.org/copyright/test-suite-license-2008/)
- [W3C test suite license - 2023 version](https://www.w3.org/copyright/test-suite-license-2023/)

</div>

</div>

</div>

</div>
