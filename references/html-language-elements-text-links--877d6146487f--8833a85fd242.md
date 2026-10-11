# HTML Standard: elements text links

Attribution and reformatting notice added for Surgeist on 2026-10-11. Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Original English standard and normative/informative labels remain authoritative. This partial format conversion supports the selected HTML language baseline; it is not a replacement specification or an implementation-conformance claim. No endorsement is implied.

Source: [HTML Standard commit snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/). Source revision `877d6146487f15b15bc8dd0e3c625063a37b456a` (9 October 2026); retrieved 11 October 2026 UTC. This immutable snapshot is the selected language baseline, independently of the html5ever implementation ingestion revision.

The exact [WHATWG LICENSE](../licenses/whatwg/LICENSE), also present in the HTML repository at this revision, has SHA-256 `85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`. Specification prose is Creative Commons Attribution 4.0 International; the BSD 3-Clause scope applies when portions are incorporated into code, not as an alternative license for this Markdown.

Complete original HTML: 14,930,614 bytes, SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`. Complete-section extraction input: 1,129,965 bytes, SHA-256 `8833a85fd24263a69dd6754816901c85725f3f44b89b23ad430a9af20cea6c96`.

Retained complete heading sections, including all descendant clauses and bounded by the next equal-or-higher-level heading: `sections`, `grouping-content`, `text-level-semantics`, `links`, `edits`. Original snapshot header is retained. The [edition map](SOURCE-EDITIONS.md#pinned-html-language-baseline) records the ten companion captures and bounded ownership. Every §4 element definition is retained across the collection. Omitted browser runtime chapters remain linked to this immutable snapshot; retaining JavaScript interfaces, networking, media, script execution or custom-element algorithms as source evidence does not assign their implementation to the native HTML crate. HTML syntax, author conformance and required parsing recovery remain distinct; malformed input is not rejected solely for an author-conformance error.

Conversion uses the repository preparation/filter, approved Pandoc 3.1.11.1 and parsed-GFM comparison. Source scripts/styles are omitted without execution. Source headings, IDs, links, prose, inline code, variables, literal blocks, images and table cells/spans are checked. Selected complex tables and inline SVG diagrams remain literal semantic HTML/SVG, preserving source structure without flattening. Cross-capture links retain the same pinned edition.

---

<!-- captured-body-start -->

<a id="head"></a>

[![WHATWG](https://resources.whatwg.org/logo.svg)](https://whatwg.org/)

# HTML

<a id="living&#45;standard"></a>Commit Snapshot — Last Updated 9 October 2026

<a id="commit&#45;snapshot&#45;link"></a>[<strong>One&#45;Page Version</strong> <code>html&#46;spec&#46;whatwg&#46;org</code>](https://html.spec.whatwg.org/) <a id="multipage&#45;link"></a>[<strong>Multipage Version</strong> <code>&#47;multipage</code>](https://html.spec.whatwg.org/multipage/) [<strong>Version for Web Devs</strong> <code>&#47;dev</code>](https://html.spec.whatwg.org/dev/) [<strong>PDF Version</strong> <code>&#47;print&#46;pdf</code>](https://html.spec.whatwg.org/print.pdf) [<strong>Translations</strong> <code>日本語 • 简体中文</code>](https://github.com/whatwg/html/wiki/Translations)

[<strong>FAQ</strong> <code>on GitHub</code>](https://github.com/whatwg/html/blob/main/FAQ.md) [<strong>Chat</strong> <code>on Matrix</code>](https://whatwg.org/chat)

[<strong>Contribute on GitHub</strong> <code>whatwg&#47;html repository</code>](https://github.com/whatwg/html) [<strong>Commits</strong> <code>on GitHub</code>](https://github.com/whatwg/html/commits)

[<strong>Open Issues</strong> <code>filed on GitHub</code>](https://github.com/whatwg/html/issues) [<strong>Open an Issue</strong> <code>whatwg&#46;org&#47;newbug</code>](https://whatwg.org/newbug) [<strong>Tests</strong> <code>web&#45;platform&#45;tests html&#47;</code>](https://github.com/web-platform-tests/wpt/tree/master/html) [<strong>Issues for Tests</strong> <code>ongoing work</code>](https://github.com/web-platform-tests/wpt/labels/html)

### <a id="sections"></a>4&#46;3 Sections[](#sections)

<strong>✔</strong>MDN

[Introduction&#95;to&#95;HTML&#47;Document&#95;and&#95;website&#95;structure&#35;HTML&#95;for&#95;structuring&#95;content](https://developer.mozilla.org/en-US/docs/Web/HTML/Introduction_to_HTML/Document_and_website_structure#HTML_for_structuring_content)

Support in all current engines&#46;

Firefox4&#43;Safari5&#43;Chrome5&#43;

------------------------------------------------------------------------

Opera11&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer9&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android11&#46;1&#43;

#### <a id="the&#45;body&#45;element"></a>4&#46;3&#46;1 The <strong><code>body</code></strong> element[](#the-body-element)

<strong>✔</strong>MDN

[Element&#47;body](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/body)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong>✔</strong>MDN

[HTMLBodyElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLBodyElement)

Support in all current engines&#46;

Firefox1&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer4&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

<strong><a id="the&#45;body&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

None&#46;

<strong><a id="the&#45;body&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

As the second element in an <a id="the&#45;body&#45;element&#58;the&#45;html&#45;element"></a><code>[html](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-html-element)</code> element&#46;

<strong><a id="the&#45;body&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;body&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<strong><a id="the&#45;body&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

A <a id="the&#45;body&#45;element&#58;the&#45;body&#45;element"></a><code>[body](#the-body-element)</code> element&#39;s <a id="the&#45;body&#45;element&#58;syntax&#45;start&#45;tag"></a>[start tag](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#syntax-start-tag) can be omitted if the element is empty&#44; or if the first thing inside the <a id="the&#45;body&#45;element&#58;the&#45;body&#45;element&#45;2"></a><code>[body](#the-body-element)</code> element is not <a id="the&#45;body&#45;element&#58;space&#45;characters"></a>[ASCII whitespace](https://infra.spec.whatwg.org/#ascii-whitespace) or a <a id="the&#45;body&#45;element&#58;syntax&#45;comments"></a>[comment](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#syntax-comments)&#44; except if the first thing inside the <a id="the&#45;body&#45;element&#58;the&#45;body&#45;element&#45;3"></a><code>[body](#the-body-element)</code> element is a <a id="the&#45;body&#45;element&#58;the&#45;meta&#45;element"></a><code>[meta](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-meta-element)</code>&#44; <a id="the&#45;body&#45;element&#58;the&#45;noscript&#45;element"></a><code>[noscript](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-noscript-element)</code>&#44; <a id="the&#45;body&#45;element&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code>&#44; <a id="the&#45;body&#45;element&#58;the&#45;script&#45;element"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code>&#44; <a id="the&#45;body&#45;element&#58;the&#45;style&#45;element"></a><code>[style](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-style-element)</code>&#44; or <a id="the&#45;body&#45;element&#58;the&#45;template&#45;element"></a><code>[template](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-template-element)</code> element&#46;

A <a id="the&#45;body&#45;element&#58;the&#45;body&#45;element&#45;4"></a><code>[body](#the-body-element)</code> element&#39;s <a id="the&#45;body&#45;element&#58;syntax&#45;end&#45;tag"></a>[end tag](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#syntax-end-tag) can be omitted if the <a id="the&#45;body&#45;element&#58;the&#45;body&#45;element&#45;5"></a><code>[body](#the-body-element)</code> element is not immediately followed by a <a id="the&#45;body&#45;element&#58;syntax&#45;comments&#45;2"></a>[comment](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#syntax-comments)&#46;

<strong><a id="the&#45;body&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;body&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<a id="the&#45;body&#45;element&#58;handler&#45;window&#45;onafterprint"></a><code>[onafterprint](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-window-onafterprint)</code>

<a id="the&#45;body&#45;element&#58;handler&#45;window&#45;onbeforeprint"></a><code>[onbeforeprint](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-window-onbeforeprint)</code>

<a id="the&#45;body&#45;element&#58;handler&#45;window&#45;onbeforeunload"></a><code>[onbeforeunload](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-window-onbeforeunload)</code>

<a id="the&#45;body&#45;element&#58;handler&#45;window&#45;onhashchange"></a><code>[onhashchange](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-window-onhashchange)</code>

<a id="the&#45;body&#45;element&#58;handler&#45;window&#45;onlanguagechange"></a><code>[onlanguagechange](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-window-onlanguagechange)</code>

<a id="the&#45;body&#45;element&#58;handler&#45;window&#45;onmessage"></a><code>[onmessage](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-window-onmessage)</code>

<a id="the&#45;body&#45;element&#58;handler&#45;window&#45;onmessageerror"></a><code>[onmessageerror](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-window-onmessageerror)</code>

<a id="the&#45;body&#45;element&#58;handler&#45;window&#45;onoffline"></a><code>[onoffline](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-window-onoffline)</code>

<a id="the&#45;body&#45;element&#58;handler&#45;window&#45;ononline"></a><code>[ononline](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-window-ononline)</code>

<a id="the&#45;body&#45;element&#58;handler&#45;window&#45;onpageswap"></a><code>[onpageswap](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-window-onpageswap)</code>

<a id="the&#45;body&#45;element&#58;handler&#45;window&#45;onpagehide"></a><code>[onpagehide](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-window-onpagehide)</code>

<a id="the&#45;body&#45;element&#58;handler&#45;window&#45;onpagereveal"></a><code>[onpagereveal](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-window-onpagereveal)</code>

<a id="the&#45;body&#45;element&#58;handler&#45;window&#45;onpageshow"></a><code>[onpageshow](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-window-onpageshow)</code>

<a id="the&#45;body&#45;element&#58;handler&#45;window&#45;onpopstate"></a><code>[onpopstate](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-window-onpopstate)</code>

<a id="the&#45;body&#45;element&#58;handler&#45;window&#45;onrejectionhandled"></a><code>[onrejectionhandled](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-window-onrejectionhandled)</code>

<a id="the&#45;body&#45;element&#58;handler&#45;window&#45;onstorage"></a><code>[onstorage](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-window-onstorage)</code>

<a id="the&#45;body&#45;element&#58;handler&#45;window&#45;onunhandledrejection"></a><code>[onunhandledrejection](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-window-onunhandledrejection)</code>

<a id="the&#45;body&#45;element&#58;handler&#45;window&#45;onunload"></a><code>[onunload](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-window-onunload)</code>

<strong><a id="the&#45;body&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-body)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-body)&#46;

<strong><a id="the&#45;body&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;body&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;body&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

<a id="htmlbodyelement"></a><a id="the&#45;body&#45;element&#58;htmlelement"></a><a id="the&#45;body&#45;element&#58;htmlconstructor"></a><a id="the&#45;body&#45;element&#58;htmlbodyelement"></a><a id="the&#45;body&#45;element&#58;windoweventhandlers"></a>

``` text
[Exposed=Window]
interface HTMLBodyElement : HTMLElement {
  [HTMLConstructor] constructor();

  // also has obsolete members
};

HTMLBodyElement includes WindowEventHandlers;
```

The <a id="the&#45;body&#45;element&#58;the&#45;body&#45;element&#45;6"></a><code>[body](#the-body-element)</code> element <a id="the&#45;body&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) the contents of the document&#46;

In conforming documents&#44; there is only one <a id="the&#45;body&#45;element&#58;the&#45;body&#45;element&#45;7"></a><code>[body](#the-body-element)</code> element&#46; The <a id="the&#45;body&#45;element&#58;dom&#45;document&#45;body"></a><code>[document&#46;body](html-language-foundations--877d6146487f--b55e73c31978.md#dom-document-body)</code> IDL attribute provides scripts with easy access to a document&#39;s <a id="the&#45;body&#45;element&#58;the&#45;body&#45;element&#45;8"></a><code>[body](#the-body-element)</code> element&#46;

<strong>Note&#58;</strong>

Some DOM operations &#40;for example&#44; parts of the <a id="the&#45;body&#45;element&#58;dnd"></a>[drag and drop](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dnd) model&#41; are defined in terms of &#34;<a id="the&#45;body&#45;element&#58;the&#45;body&#45;element&#45;2&#45;2"></a>[the body element](html-language-foundations--877d6146487f--b55e73c31978.md#the-body-element-2)&#34;&#46; This refers to a particular element in the DOM&#44; as per the definition of the term&#44; and not any arbitrary <a id="the&#45;body&#45;element&#58;the&#45;body&#45;element&#45;9"></a><code>[body](#the-body-element)</code> element&#46;

The <a id="the&#45;body&#45;element&#58;the&#45;body&#45;element&#45;10"></a><code>[body](#the-body-element)</code> element exposes as <a id="the&#45;body&#45;element&#58;event&#45;handler&#45;content&#45;attributes"></a>[event handler content attributes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-handler-content-attributes) a number of the <a id="the&#45;body&#45;element&#58;event&#45;handlers"></a>[event handlers](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-handlers) of the <a id="the&#45;body&#45;element&#58;window"></a><code>[Window](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window)</code> object&#46; It also mirrors their <a id="the&#45;body&#45;element&#58;event&#45;handler&#45;idl&#45;attributes"></a>[event handler IDL attributes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-handler-idl-attributes)&#46;

The <a id="the&#45;body&#45;element&#58;event&#45;handlers&#45;2"></a>[event handlers](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-handlers) of the <a id="the&#45;body&#45;element&#58;window&#45;2"></a><code>[Window](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window)</code> object named by the <a id="the&#45;body&#45;element&#58;window&#45;reflecting&#45;body&#45;element&#45;event&#45;handler&#45;set"></a>[<code>Window</code>&#45;reflecting body element event handler set](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window-reflecting-body-element-event-handler-set)&#44; exposed on the <a id="the&#45;body&#45;element&#58;the&#45;body&#45;element&#45;11"></a><code>[body](#the-body-element)</code> element&#44; replace the generic <a id="the&#45;body&#45;element&#58;event&#45;handlers&#45;3"></a>[event handlers](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-handlers) with the same names normally supported by <a id="the&#45;body&#45;element&#58;html&#45;elements"></a>[HTML elements](html-language-foundations--877d6146487f--b55e73c31978.md#html-elements)&#46;

<strong>Example&#58;</strong>

Thus&#44; for example&#44; a bubbling <a id="the&#45;body&#45;element&#58;event&#45;error"></a><code>[error](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-error)</code> event dispatched on a child of <a id="the&#45;body&#45;element&#58;the&#45;body&#45;element&#45;2&#45;3"></a>[the body element](html-language-foundations--877d6146487f--b55e73c31978.md#the-body-element-2) of a <a id="the&#45;body&#45;element&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> would first trigger the <a id="the&#45;body&#45;element&#58;handler&#45;onerror"></a><code>[onerror](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-onerror)</code> <a id="the&#45;body&#45;element&#58;event&#45;handler&#45;content&#45;attributes&#45;2"></a>[event handler content attributes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-handler-content-attributes) of that element&#44; then that of the root <a id="the&#45;body&#45;element&#58;the&#45;html&#45;element&#45;2"></a><code>[html](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-html-element)</code> element&#44; and only <em>then</em> would it trigger the <a id="the&#45;body&#45;element&#58;handler&#45;onerror&#45;2"></a><code>[onerror](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handler-onerror)</code> <a id="the&#45;body&#45;element&#58;event&#45;handler&#45;content&#45;attributes&#45;3"></a>[event handler content attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-handler-content-attributes) on the <a id="the&#45;body&#45;element&#58;the&#45;body&#45;element&#45;12"></a><code>[body](#the-body-element)</code> element&#46; This is because the event would bubble from the target&#44; to the <a id="the&#45;body&#45;element&#58;the&#45;body&#45;element&#45;13"></a><code>[body](#the-body-element)</code>&#44; to the <a id="the&#45;body&#45;element&#58;the&#45;html&#45;element&#45;3"></a><code>[html](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-html-element)</code>&#44; to the <a id="the&#45;body&#45;element&#58;document&#45;2"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#44; to the <a id="the&#45;body&#45;element&#58;window&#45;3"></a><code>[Window](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window)</code>&#44; and the <a id="the&#45;body&#45;element&#58;event&#45;handlers&#45;4"></a>[event handler](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-handlers) on the <a id="the&#45;body&#45;element&#58;the&#45;body&#45;element&#45;14"></a><code>[body](#the-body-element)</code> is watching the <a id="the&#45;body&#45;element&#58;window&#45;4"></a><code>[Window](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window)</code> not the <a id="the&#45;body&#45;element&#58;the&#45;body&#45;element&#45;15"></a><code>[body](#the-body-element)</code>&#46; A regular event listener attached to the <a id="the&#45;body&#45;element&#58;the&#45;body&#45;element&#45;16"></a><code>[body](#the-body-element)</code> using <code>addEventListener&#40;&#41;</code>&#44; however&#44; would be run when the event bubbled through the <a id="the&#45;body&#45;element&#58;the&#45;body&#45;element&#45;17"></a><code>[body](#the-body-element)</code> and not when it reaches the <a id="the&#45;body&#45;element&#58;window&#45;5"></a><code>[Window](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window)</code> object&#46;

<strong>Example&#58;</strong>

This page updates an indicator to show whether or not the user is online&#58;

``` text
<!DOCTYPE HTML>
<html lang="en">
 <head>
  <title>Online or offline?</title>
  <script>
   function update(online) {
     document.getElementById('status').textContent =
       online ? 'Online' : 'Offline';
   }
  </script>
 </head>
 <body ononline="update(true)"
       onoffline="update(false)"
       onload="update(navigator.onLine)">
  <p>You are: <span id="status">(Unknown)</span></p>
 </body>
</html>
```

#### <a id="the&#45;article&#45;element"></a>4&#46;3&#46;2 The <strong><code>article</code></strong> element[](#the-article-element)

<strong>✔</strong>MDN

[Element&#47;article](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/article)

Support in all current engines&#46;

Firefox4&#43;Safari5&#43;Chrome5&#43;

------------------------------------------------------------------------

Opera11&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer9&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android11&#46;1&#43;

<strong><a id="the&#45;article&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;article&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;article&#45;element&#58;sectioning&#45;content&#45;2"></a>[Sectioning content](html-language-foundations--877d6146487f--b55e73c31978.md#sectioning-content-2)&#46;

<a id="the&#45;article&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;article&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;article&#45;element&#58;sectioning&#45;content&#45;2&#45;2"></a>[sectioning content](html-language-foundations--877d6146487f--b55e73c31978.md#sectioning-content-2) is expected&#46;

<strong><a id="the&#45;article&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;article&#45;element&#58;flow&#45;content&#45;2&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<strong><a id="the&#45;article&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;article&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;article&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;article&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-article)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-article)&#46;

<strong><a id="the&#45;article&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;article&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;article&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;article&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;article&#45;element&#58;the&#45;article&#45;element"></a><code>[article](#the-article-element)</code> element <a id="the&#45;article&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a complete&#44; or self&#45;contained&#44; composition in a document&#44; page&#44; application&#44; or site and that is&#44; in principle&#44; independently distributable or reusable&#44; e&#46;g&#46; in syndication&#46; This could be a forum post&#44; a magazine or newspaper article&#44; a blog entry&#44; a user&#45;submitted comment&#44; an interactive widget or gadget&#44; or any other independent item of content&#46;

When <a id="the&#45;article&#45;element&#58;the&#45;article&#45;element&#45;2"></a><code>[article](#the-article-element)</code> elements are nested&#44; the inner <a id="the&#45;article&#45;element&#58;the&#45;article&#45;element&#45;3"></a><code>[article](#the-article-element)</code> elements represent articles that are in principle related to the contents of the outer article&#46; For instance&#44; a blog entry on a site that accepts user&#45;submitted comments could represent the comments as <a id="the&#45;article&#45;element&#58;the&#45;article&#45;element&#45;4"></a><code>[article](#the-article-element)</code> elements nested within the <a id="the&#45;article&#45;element&#58;the&#45;article&#45;element&#45;5"></a><code>[article](#the-article-element)</code> element for the blog entry&#46;

Author information associated with an <a id="the&#45;article&#45;element&#58;the&#45;article&#45;element&#45;6"></a><code>[article](#the-article-element)</code> element &#40;q&#46;v&#46; the <a id="the&#45;article&#45;element&#58;the&#45;address&#45;element"></a><code>[address](#the-address-element)</code> element&#41; does not apply to nested <a id="the&#45;article&#45;element&#58;the&#45;article&#45;element&#45;7"></a><code>[article](#the-article-element)</code> elements&#46;

<strong>Note&#58;</strong>

When used specifically with content to be redistributed in syndication&#44; the <a id="the&#45;article&#45;element&#58;the&#45;article&#45;element&#45;8"></a><code>[article](#the-article-element)</code> element is similar in purpose to the <code>entry</code> element in Atom&#46; [&#91;ATOM&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsATOM)

<strong>Note&#58;</strong>

The schema&#46;org microdata vocabulary can be used to provide the publication date for an <a id="the&#45;article&#45;element&#58;the&#45;article&#45;element&#45;9"></a><code>[article](#the-article-element)</code> element&#44; using one of the CreativeWork subtypes&#46;

When the main content of the page &#40;i&#46;e&#46; excluding footers&#44; headers&#44; navigation blocks&#44; and sidebars&#41; is all one single self&#45;contained composition&#44; that content may be marked with an <a id="the&#45;article&#45;element&#58;the&#45;article&#45;element&#45;10"></a><code>[article](#the-article-element)</code>&#44; but it is technically redundant in that case &#40;since it&#39;s self&#45;evident that the page is a single composition&#44; as it is a single document&#41;&#46;

<a id="article&#45;example"></a>

<strong>Example&#58;</strong>

[](#article-example)

This example shows a blog post using the <a id="the&#45;article&#45;element&#58;the&#45;article&#45;element&#45;11"></a><code>[article](#the-article-element)</code> element&#44; with some schema&#46;org annotations&#58;

``` text
<article itemscope itemtype="http://schema.org/BlogPosting">
 <header>
  <h2 itemprop="headline">The Very First Rule of Life</h2>
  <p><time itemprop="datePublished" datetime="2009-10-09">3 days ago</time></p>
  <link itemprop="url" href="?comments=0">
 </header>
 <p>If there's a microphone anywhere near you, assume it's hot and
 sending whatever you're saying to the world. Seriously.</p>
 <p>...</p>
 <footer>
  <a itemprop="discussionUrl" href="?comments=1">Show comments...</a>
 </footer>
</article>
```

Here is that same blog post&#44; but showing some of the comments&#58;

``` text
<article itemscope itemtype="http://schema.org/BlogPosting">
 <header>
  <h2 itemprop="headline">The Very First Rule of Life</h2>
  <p><time itemprop="datePublished" datetime="2009-10-09">3 days ago</time></p>
  <link itemprop="url" href="?comments=0">
 </header>
 <p>If there's a microphone anywhere near you, assume it's hot and
 sending whatever you're saying to the world. Seriously.</p>
 <p>...</p>
 <section>
  <h1>Comments</h1>
  <article itemprop="comment" itemscope itemtype="http://schema.org/Comment" id="c1">
   <link itemprop="url" href="#c1">
   <footer>
    <p>Posted by: <span itemprop="creator" itemscope itemtype="http://schema.org/Person">
     <span itemprop="name">George Washington</span>
    </span></p>
    <p><time itemprop="dateCreated" datetime="2009-10-10">15 minutes ago</time></p>
   </footer>
   <p>Yeah! Especially when talking about your lobbyist friends!</p>
  </article>
  <article itemprop="comment" itemscope itemtype="http://schema.org/Comment" id="c2">
   <link itemprop="url" href="#c2">
   <footer>
    <p>Posted by: <span itemprop="creator" itemscope itemtype="http://schema.org/Person">
     <span itemprop="name">George Hammond</span>
    </span></p>
    <p><time itemprop="dateCreated" datetime="2009-10-10">5 minutes ago</time></p>
   </footer>
   <p>Hey, you have the same first name as me.</p>
  </article>
 </section>
</article>
```

Notice the use of <a id="the&#45;article&#45;element&#58;the&#45;footer&#45;element"></a><code>[footer](#the-footer-element)</code> to give the information for each comment &#40;such as who wrote it and when&#41;&#58; the <a id="the&#45;article&#45;element&#58;the&#45;footer&#45;element&#45;2"></a><code>[footer](#the-footer-element)</code> element <em>can</em> appear at the start of its section when appropriate&#44; such as in this case&#46; &#40;Using <a id="the&#45;article&#45;element&#58;the&#45;header&#45;element"></a><code>[header](#the-header-element)</code> in this case wouldn&#39;t be wrong either&#59; it&#39;s mostly a matter of authoring preference&#46;&#41;

<strong>Example&#58;</strong>

In this example&#44; <a id="the&#45;article&#45;element&#58;the&#45;article&#45;element&#45;12"></a><code>[article](#the-article-element)</code> elements are used to host widgets on a portal page&#46; The widgets are implemented as <a id="the&#45;article&#45;element&#58;customized&#45;built&#45;in&#45;element"></a>[customized built&#45;in elements](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#customized-built-in-element) in order to get specific styling and scripted behavior&#46;

``` text
<!DOCTYPE HTML>
<html lang=en>
<title>eHome Portal</title>
<script src="/scripts/widgets.js"></script>
<link rel=stylesheet href="/styles/main.css">
<article is="stock-widget">
 <h2>Stocks</h2>
 <table>
  <thead> <tr> <th> Stock <th> Value <th> Delta
  <tbody> <template> <tr> <td> <td> <td> </template>
 </table>
 <p> <input type=button value="Refresh" onclick="this.parentElement.refresh()">
</article>
<article is="news-widget">
 <h2>News</h2>
 <ul>
  <template>
   <li>
    <p><img> <strong></strong>
    <p>
  </template>
 </ul>
 <p> <input type=button value="Refresh" onclick="this.parentElement.refresh()">
</article>
```

#### <a id="the&#45;section&#45;element"></a>4&#46;3&#46;3 The <strong><code>section</code></strong> element[](#the-section-element)

<strong>✔</strong>MDN

[Element&#47;section](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/section)

Support in all current engines&#46;

Firefox4&#43;Safari5&#43;Chrome5&#43;

------------------------------------------------------------------------

Opera11&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer9&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android11&#46;1&#43;

<strong><a id="the&#45;section&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;section&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;section&#45;element&#58;sectioning&#45;content&#45;2"></a>[Sectioning content](html-language-foundations--877d6146487f--b55e73c31978.md#sectioning-content-2)&#46;

<a id="the&#45;section&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;section&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;section&#45;element&#58;sectioning&#45;content&#45;2&#45;2"></a>[sectioning content](html-language-foundations--877d6146487f--b55e73c31978.md#sectioning-content-2) is expected&#46;

<strong><a id="the&#45;section&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;section&#45;element&#58;flow&#45;content&#45;2&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<strong><a id="the&#45;section&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;section&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;section&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;section&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-section)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-section)&#46;

<strong><a id="the&#45;section&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;section&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;section&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;section&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;section&#45;element&#58;the&#45;section&#45;element"></a><code>[section](#the-section-element)</code> element <a id="the&#45;section&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a generic section of a document or application&#46; A section&#44; in this context&#44; is a thematic grouping of content&#44; typically with a heading&#46;

<strong>Example&#58;</strong>

Examples of sections would be chapters&#44; the various tabbed pages in a tabbed dialog box&#44; or the numbered sections of a thesis&#46; A web site&#39;s home page could be split into sections for an introduction&#44; news items&#44; and contact information&#46;

<strong>Note&#58;</strong>

Authors are encouraged to use the <a id="the&#45;section&#45;element&#58;the&#45;article&#45;element"></a><code>[article](#the-article-element)</code> element instead of the <a id="the&#45;section&#45;element&#58;the&#45;section&#45;element&#45;2"></a><code>[section](#the-section-element)</code> element when it would make sense to syndicate the contents of the element&#46;

<a id="use&#45;div&#45;for&#45;wrappers"></a>

<strong>Note&#58;</strong>

[](#use-div-for-wrappers)The <a id="the&#45;section&#45;element&#58;the&#45;section&#45;element&#45;3"></a><code>[section](#the-section-element)</code> element is not a generic container element&#46; When an element is needed only for styling purposes or as a convenience for scripting&#44; authors are encouraged to use the <a id="the&#45;section&#45;element&#58;the&#45;div&#45;element"></a><code>[div](#the-div-element)</code> element instead&#46; A general rule is that the <a id="the&#45;section&#45;element&#58;the&#45;section&#45;element&#45;4"></a><code>[section](#the-section-element)</code> element is appropriate only if the element&#39;s contents would be listed explicitly in the document&#39;s <a id="the&#45;section&#45;element&#58;outline"></a>[outline](#outline)&#46;

<strong>Example&#58;</strong>

In the following example&#44; we see an article &#40;part of a larger web page&#41; about apples&#44; containing two short sections&#46;

``` text
<article>
 <hgroup>
  <h2>Apples</h2>
  <p>Tasty, delicious fruit!</p>
 </hgroup>
 <p>The apple is the pomaceous fruit of the apple tree.</p>
 <section>
  <h3>Red Delicious</h3>
  <p>These bright red apples are the most common found in many
  supermarkets.</p>
 </section>
 <section>
  <h3>Granny Smith</h3>
  <p>These juicy, green apples make a great filling for
  apple pies.</p>
 </section>
</article>
```

<strong>Example&#58;</strong>

Here is a graduation programme with two sections&#44; one for the list of people graduating&#44; and one for the description of the ceremony&#46; &#40;The markup in this example features an uncommon style sometimes used to minimize the amount of <a id="the&#45;section&#45;element&#58;inter&#45;element&#45;whitespace"></a>[inter&#45;element whitespace](html-language-foundations--877d6146487f--b55e73c31978.md#inter-element-whitespace)&#46;&#41;

``` text
<!DOCTYPE Html>
<Html Lang=En
 ><Head
   ><Title
     >Graduation Ceremony Summer 2022</Title
   ></Head
 ><Body
   ><H1
     >Graduation</H1
   ><Section
     ><H2
       >Ceremony</H2
     ><P
       >Opening Procession</P
     ><P
       >Speech by Valedictorian</P
     ><P
       >Speech by Class President</P
     ><P
       >Presentation of Diplomas</P
     ><P
       >Closing Speech by Headmaster</P
   ></Section
   ><Section
     ><H2
       >Graduates</H2
     ><Ul
       ><Li
         >Molly Carpenter</Li
       ><Li
         >Anastasia Luccio</Li
       ><Li
         >Ebenezar McCoy</Li
       ><Li
         >Karrin Murphy</Li
       ><Li
         >Thomas Raith</Li
       ><Li
         >Susan Rodriguez</Li
     ></Ul
   ></Section
 ></Body
></Html>
```

<strong>Example&#58;</strong>

In this example&#44; a book author has marked up some sections as chapters and some as appendices&#44; and uses CSS to style the headers in these two classes of section differently&#46;

``` text
<style>
 section { border: double medium; margin: 2em; }
 section.chapter h2 { font: 2em Roboto, Helvetica Neue, sans-serif; }
 section.appendix h2 { font: small-caps 2em Roboto, Helvetica Neue, sans-serif; }
</style>
<header>
 <hgroup>
  <h1>My Book</h1>
  <p>A sample with not much content</p>
 </hgroup>
 <p><small>Published by Dummy Publicorp Ltd.</small></p>
</header>
<section class="chapter">
 <h2>My First Chapter</h2>
 <p>This is the first of my chapters. It doesn't say much.</p>
 <p>But it has two paragraphs!</p>
</section>
<section class="chapter">
 <h2>It Continues: The Second Chapter</h2>
 <p>Bla dee bla, dee bla dee bla. Boom.</p>
</section>
<section class="chapter">
 <h2>Chapter Three: A Further Example</h2>
 <p>It's not like a battle between brightness and earthtones would go
 unnoticed.</p>
 <p>But it might ruin my story.</p>
</section>
<section class="appendix">
 <h2>Appendix A: Overview of Examples</h2>
 <p>These are demonstrations.</p>
</section>
<section class="appendix">
 <h2>Appendix B: Some Closing Remarks</h2>
 <p>Hopefully this long example shows that you <em>can</em> style
 sections, so long as they are used to indicate actual sections.</p>
</section>
```

#### <a id="the&#45;nav&#45;element"></a>4&#46;3&#46;4 The <strong><code>nav</code></strong> element[](#the-nav-element)

<strong>✔</strong>MDN

[Element&#47;nav](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/nav)

Support in all current engines&#46;

Firefox4&#43;Safari5&#43;Chrome5&#43;

------------------------------------------------------------------------

Opera11&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer9&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android11&#46;1&#43;

<strong><a id="the&#45;nav&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;nav&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;nav&#45;element&#58;sectioning&#45;content&#45;2"></a>[Sectioning content](html-language-foundations--877d6146487f--b55e73c31978.md#sectioning-content-2)&#46;

<a id="the&#45;nav&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;nav&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;nav&#45;element&#58;sectioning&#45;content&#45;2&#45;2"></a>[sectioning content](html-language-foundations--877d6146487f--b55e73c31978.md#sectioning-content-2) is expected&#46;

<strong><a id="the&#45;nav&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;nav&#45;element&#58;flow&#45;content&#45;2&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<strong><a id="the&#45;nav&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;nav&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;nav&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;nav&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-nav)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-nav)&#46;

<strong><a id="the&#45;nav&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;nav&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;nav&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;nav&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;nav&#45;element&#58;the&#45;nav&#45;element"></a><code>[nav](#the-nav-element)</code> element <a id="the&#45;nav&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a section of a page that links to other pages or to parts within the page&#58; a section with navigation links&#46;

<strong>Note&#58;</strong>

Not all groups of links on a page need to be in a <a id="the&#45;nav&#45;element&#58;the&#45;nav&#45;element&#45;2"></a><code>[nav](#the-nav-element)</code> element — the element is primarily intended for sections that consist of major navigation blocks&#46; In particular&#44; it is common for footers to have a short list of links to various pages of a site&#44; such as the terms of service&#44; the home page&#44; and a copyright page&#46; The <a id="the&#45;nav&#45;element&#58;the&#45;footer&#45;element"></a><code>[footer](#the-footer-element)</code> element alone is sufficient for such cases&#59; while a <a id="the&#45;nav&#45;element&#58;the&#45;nav&#45;element&#45;3"></a><code>[nav](#the-nav-element)</code> element can be used in such cases&#44; it is usually unnecessary&#46;

<strong>Note&#58;</strong>

User agents &#40;such as screen readers&#41; that are targeted at users who can benefit from navigation information being omitted in the initial rendering&#44; or who can benefit from navigation information being immediately available&#44; can use this element as a way to determine what content on the page to initially skip or provide on request &#40;or both&#41;&#46;

<strong>Example&#58;</strong>

In the following example&#44; there are two <a id="the&#45;nav&#45;element&#58;the&#45;nav&#45;element&#45;4"></a><code>[nav](#the-nav-element)</code> elements&#44; one for primary navigation around the site&#44; and one for secondary navigation around the page itself&#46;

``` text
<body>
 <h1>The Wiki Center Of Exampland</h1>
 <nav>
  <ul>
   <li><a href="/">Home</a></li>
   <li><a href="/events">Current Events</a></li>
   ...more...
  </ul>
 </nav>
 <article>
  <header>
   <h2>Demos in Exampland</h2>
   <p>Written by A. N. Other.</p>
  </header>
  <nav>
   <ul>
    <li><a href="#public">Public demonstrations</a></li>
    <li><a href="#destroy">Demolitions</a></li>
    ...more...
   </ul>
  </nav>
  <div>
   <section id="public">
    <h2>Public demonstrations</h2>
    <p>...more...</p>
   </section>
   <section id="destroy">
    <h2>Demolitions</h2>
    <p>...more...</p>
   </section>
   ...more...
  </div>
  <footer>
   <p><a href="?edit">Edit</a> | <a href="?delete">Delete</a> | <a href="?Rename">Rename</a></p>
  </footer>
 </article>
 <footer>
  <p><small>© copyright 1998 Exampland Emperor</small></p>
 </footer>
</body>
```

<strong>Example&#58;</strong>

In the following example&#44; the page has several places where links are present&#44; but only one of those places is considered a navigation section&#46;

``` text
<body itemscope itemtype="http://schema.org/Blog">
 <header>
  <h1>Wake up sheeple!</h1>
  <p><a href="news.html">News</a> -
     <a href="blog.html">Blog</a> -
     <a href="forums.html">Forums</a></p>
  <p>Last Modified: <span itemprop="dateModified">2009-04-01</span></p>
  <nav>
   <h2>Navigation</h2>
   <ul>
    <li><a href="articles.html">Index of all articles</a></li>
    <li><a href="today.html">Things sheeple need to wake up for today</a></li>
    <li><a href="successes.html">Sheeple we have managed to wake</a></li>
   </ul>
  </nav>
 </header>
 <main>
  <article itemprop="blogPosts" itemscope itemtype="http://schema.org/BlogPosting">
   <header>
    <h2 itemprop="headline">My Day at the Beach</h2>
   </header>
   <div itemprop="articleBody">
    <p>Today I went to the beach and had a lot of fun.</p>
    ...more content...
   </div>
   <footer>
    <p>Posted <time itemprop="datePublished" datetime="2009-10-10">Thursday</time>.</p>
   </footer>
  </article>
  ...more blog posts...
 </main>
 <footer>
  <p>Copyright ©
   <span itemprop="copyrightYear">2010</span>
   <span itemprop="copyrightHolder">The Example Company</span>
  </p>
  <p><a href="about.html">About</a> -
     <a href="policy.html">Privacy Policy</a> -
     <a href="contact.html">Contact Us</a></p>
 </footer>
</body>
```

You can also see microdata annotations in the above example that use the schema&#46;org vocabulary to provide the publication date and other metadata about the blog post&#46;

<strong>Example&#58;</strong>

A <a id="the&#45;nav&#45;element&#58;the&#45;nav&#45;element&#45;5"></a><code>[nav](#the-nav-element)</code> element doesn&#39;t have to contain a list&#44; it can contain other kinds of content as well&#46; In this navigation block&#44; links are provided in prose&#58;

``` text
<nav>
 <h1>Navigation</h1>
 <p>You are on my home page. To the north lies <a href="/blog">my
 blog</a>, from whence the sounds of battle can be heard. To the east
 you can see a large mountain, upon which many <a
 href="/school">school papers</a> are littered. Far up thus mountain
 you can spy a little figure who appears to be me, desperately
 scribbling a <a href="/school/thesis">thesis</a>.</p>
 <p>To the west are several exits. One fun-looking exit is labeled <a
 href="https://games.example.com/">"games"</a>. Another more
 boring-looking exit is labeled <a
 href="https://isp.example.net/">ISP™</a>.</p>
 <p>To the south lies a dark and dank <a href="/about">contacts
 page</a>. Cobwebs cover its disused entrance, and at one point you
 see a rat run quickly out of the page.</p>
</nav>
```

<strong>Example&#58;</strong>

In this example&#44; <a id="the&#45;nav&#45;element&#58;the&#45;nav&#45;element&#45;6"></a><code>[nav](#the-nav-element)</code> is used in an email application&#44; to let the user switch folders&#58;

``` text
<p><input type=button value="Compose" onclick="compose()"></p>
<nav>
 <h1>Folders</h1>
 <ul>
  <li> <a href="/inbox" onclick="return openFolder(this.href)">Inbox</a> <span class=count></span>
  <li> <a href="/sent" onclick="return openFolder(this.href)">Sent</a>
  <li> <a href="/drafts" onclick="return openFolder(this.href)">Drafts</a>
  <li> <a href="/trash" onclick="return openFolder(this.href)">Trash</a>
  <li> <a href="/customers" onclick="return openFolder(this.href)">Customers</a>
 </ul>
</nav>
```

#### <a id="the&#45;aside&#45;element"></a>4&#46;3&#46;5 The <strong><code>aside</code></strong> element[](#the-aside-element)

<strong>✔</strong>MDN

[Element&#47;aside](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/aside)

Support in all current engines&#46;

Firefox4&#43;Safari5&#43;Chrome5&#43;

------------------------------------------------------------------------

Opera11&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer9&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android11&#46;1&#43;

<strong><a id="the&#45;aside&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;aside&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;aside&#45;element&#58;sectioning&#45;content&#45;2"></a>[Sectioning content](html-language-foundations--877d6146487f--b55e73c31978.md#sectioning-content-2)&#46;

<a id="the&#45;aside&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;aside&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;aside&#45;element&#58;sectioning&#45;content&#45;2&#45;2"></a>[sectioning content](html-language-foundations--877d6146487f--b55e73c31978.md#sectioning-content-2) is expected&#46;

<strong><a id="the&#45;aside&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;aside&#45;element&#58;flow&#45;content&#45;2&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<strong><a id="the&#45;aside&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;aside&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;aside&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;aside&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-aside)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-aside)&#46;

<strong><a id="the&#45;aside&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;aside&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;aside&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;aside&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;aside&#45;element&#58;the&#45;aside&#45;element"></a><code>[aside](#the-aside-element)</code> element <a id="the&#45;aside&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a section of a page that consists of content that is tangentially related to the content around the <a id="the&#45;aside&#45;element&#58;the&#45;aside&#45;element&#45;2"></a><code>[aside](#the-aside-element)</code> element&#44; and which could be considered separate from that content&#46; Such sections are often represented as sidebars in printed typography&#46;

The element can be used for typographical effects like pull quotes or sidebars&#44; for advertising&#44; for groups of <a id="the&#45;aside&#45;element&#58;the&#45;nav&#45;element"></a><code>[nav](#the-nav-element)</code> elements&#44; and for other content that is considered separate from the main content of the page&#46;

<strong>Note&#58;</strong>

It&#39;s not appropriate to use the <a id="the&#45;aside&#45;element&#58;the&#45;aside&#45;element&#45;3"></a><code>[aside](#the-aside-element)</code> element just for parentheticals&#44; since those are part of the main flow of the document&#46;

<strong>Example&#58;</strong>

The following example shows how an aside is used to mark up background material on Switzerland in a much longer news story on Europe&#46;

``` text
<aside>
 <h2>Switzerland</h2>
 <p>Switzerland, a land-locked country in the middle of geographic
 Europe, has not joined the geopolitical European Union, though it is
 a signatory to a number of European treaties.</p>
</aside>
```

<strong>Example&#58;</strong>

The following example shows how an aside is used to mark up a pull quote in a longer article&#46;

``` text
...

<p>He later joined a large company, continuing on the same work.
<q>I love my job. People ask me what I do for fun when I'm not at
work. But I'm paid to do my hobby, so I never know what to
answer. Some people wonder what they would do if they didn't have to
work... but I know what I would do, because I was unemployed for a
year, and I filled that time doing exactly what I do now.</q></p>

<aside>
 <q>People ask me what I do for fun when I'm not at work. But I'm
 paid to do my hobby, so I never know what to answer.</q>
</aside>

<p>Of course his work — or should that be hobby? —
isn't his only passion. He also enjoys other pleasures.</p>

...
```

<strong>Example&#58;</strong>

The following extract shows how <a id="the&#45;aside&#45;element&#58;the&#45;aside&#45;element&#45;4"></a><code>[aside](#the-aside-element)</code> can be used for blogrolls and other side content on a blog&#58;

``` text
<body>
 <header>
  <h1>My wonderful blog</h1>
  <p>My tagline</p>
 </header>
 <aside>
  <!-- this aside contains two sections that are tangentially related
  to the page, namely, links to other blogs, and links to blog posts
  from this blog -->
  <nav>
   <h2>My blogroll</h2>
   <ul>
    <li><a href="https://blog.example.com/">Example Blog</a>
   </ul>
  </nav>
  <nav>
   <h2>Archives</h2>
   <ol reversed>
    <li><a href="/last-post">My last post</a>
    <li><a href="/first-post">My first post</a>
   </ol>
  </nav>
 </aside>
 <aside>
  <!-- this aside is tangentially related to the page also, it
  contains twitter messages from the blog author -->
  <h1>Twitter Feed</h1>
  <blockquote cite="https://twitter.example.net/t31351234">
   I'm on vacation, writing my blog.
  </blockquote>
  <blockquote cite="https://twitter.example.net/t31219752">
   I'm going to go on vacation soon.
  </blockquote>
 </aside>
 <article>
  <!-- this is a blog post -->
  <h2>My last post</h2>
  <p>This is my last post.</p>
  <footer>
   <p><a href="/last-post" rel=bookmark>Permalink</a>
  </footer>
 </article>
 <article>
  <!-- this is also a blog post -->
  <h2>My first post</h2>
  <p>This is my first post.</p>
  <aside>
   <!-- this aside is about the blog post, since it's inside the
   <article> element; it would be wrong, for instance, to put the
   blogroll here, since the blogroll isn't really related to this post
   specifically, only to the page as a whole -->
   <h2>Posting</h2>
   <p>While I'm thinking about it, I wanted to say something about
   posting. Posting is fun!</p>
  </aside>
  <footer>
   <p><a href="/first-post" rel=bookmark>Permalink</a>
  </footer>
 </article>
 <footer>
  <p><a href="/archives">Archives</a> -
   <a href="/about">About me</a> -
   <a href="/copyright">Copyright</a></p>
 </footer>
</body>
```

#### <a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements"></a>4&#46;3&#46;6 The <a id="the&#45;h1&#45;element"></a><strong><code>h1</code></strong>&#44; <a id="the&#45;h2&#45;element"></a><strong><code>h2</code></strong>&#44; <a id="the&#45;h3&#45;element"></a><strong><code>h3</code></strong>&#44; <a id="the&#45;h4&#45;element"></a><strong><code>h4</code></strong>&#44; <a id="the&#45;h5&#45;element"></a><strong><code>h5</code></strong>&#44; and <a id="the&#45;h6&#45;element"></a><strong><code>h6</code></strong> elements[](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)

<strong>✔</strong>MDN

[Element&#47;Heading&#95;Elements](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/Heading_Elements)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[Element&#47;Heading&#95;Elements](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/Heading_Elements)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[Element&#47;Heading&#95;Elements](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/Heading_Elements)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[Element&#47;Heading&#95;Elements](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/Heading_Elements)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[Element&#47;Heading&#95;Elements](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/Heading_Elements)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[Element&#47;Heading&#95;Elements](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/Heading_Elements)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong>✔</strong>MDN

[HTMLHeadingElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLHeadingElement)

Support in all current engines&#46;

Firefox1&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

<strong><a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;heading&#45;content&#45;2"></a>[Heading content](html-language-foundations--877d6146487f--b55e73c31978.md#heading-content-2)&#46;

<a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

As a child of an <a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;the&#45;hgroup&#45;element"></a><code>[hgroup](#the-hgroup-element)</code> element&#46;

Where <a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;heading&#45;content&#45;2&#45;2"></a>[heading content](html-language-foundations--877d6146487f--b55e73c31978.md#heading-content-2) is expected&#46;

<strong><a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-h1-h6)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-h1-h6)&#46;

<strong><a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

<a id="htmlheadingelement"></a><a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;htmlelement"></a><a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;htmlconstructor"></a>

``` text
[Exposed=Window]
interface HTMLHeadingElement : HTMLElement {
  [HTMLConstructor] constructor();

  // also has obsolete members
};
```

These elements <a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;represents"></a>[represent](html-language-foundations--877d6146487f--b55e73c31978.md#represents) headings for their sections&#46;

The semantics and meaning of these elements are defined in the section on [headings and outlines](#headings-and-outlines)&#46;

These elements have a <a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;heading&#45;level"></a>[heading level](#heading-level) given by the number in their name&#46; The <a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;heading&#45;level&#45;2"></a>[heading level](#heading-level) corresponds to the levels of nested sections&#46; The <a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements"></a><code>[h1](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code> element is for a top&#45;level section&#44; <a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;2"></a><code>[h2](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code> for a subsection&#44; <a id="the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;3"></a><code>[h3](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code> for a sub&#45;subsection&#44; and so on&#46;

<strong>Example&#58;</strong>

As far as their respective document outlines &#40;their heading and section structures&#41; are concerned&#44; these two snippets are semantically equivalent&#58;

``` text
<body>
<h1>Let's call it a draw(ing surface)</h1>
<h2>Diving in</h2>
<h2>Simple shapes</h2>
<h2>Canvas coordinates</h2>
<h3>Canvas coordinates diagram</h3>
<h2>Paths</h2>
</body>
```

``` text
<body>
 <h1>Let's call it a draw(ing surface)</h1>
 <section>
  <h2>Diving in</h2>
 </section>
 <section>
  <h2>Simple shapes</h2>
 </section>
 <section>
  <h2>Canvas coordinates</h2>
  <section>
   <h3>Canvas coordinates diagram</h3>
  </section>
 </section>
 <section>
  <h2>Paths</h2>
 </section>
</body>
```

Authors might prefer the former style for its terseness&#44; or the latter style for its additional styling hooks&#46; Which is best is purely an issue of preferred authoring style&#46;

#### <a id="the&#45;hgroup&#45;element"></a>4&#46;3&#46;7 The <strong><code>hgroup</code></strong> element[](#the-hgroup-element)

<strong>✔</strong>MDN

[Element&#47;hgroup](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/hgroup)

Support in all current engines&#46;

Firefox4&#43;Safari5&#43;Chrome5&#43;

------------------------------------------------------------------------

Opera11&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer9&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android2&#46;2&#43;Samsung Internet&#63;Opera Android11&#46;1&#43;

<strong><a id="the&#45;hgroup&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;hgroup&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;hgroup&#45;element&#58;heading&#45;content&#45;2"></a>[Heading content](html-language-foundations--877d6146487f--b55e73c31978.md#heading-content-2)&#46;

<a id="the&#45;hgroup&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;hgroup&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;hgroup&#45;element&#58;heading&#45;content&#45;2&#45;2"></a>[heading content](html-language-foundations--877d6146487f--b55e73c31978.md#heading-content-2) is expected&#46;

<strong><a id="the&#45;hgroup&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

Zero or more <a id="the&#45;hgroup&#45;element&#58;the&#45;p&#45;element"></a><code>[p](#the-p-element)</code> elements&#44; followed by one <a id="the&#45;hgroup&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements"></a><code>[h1](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; <a id="the&#45;hgroup&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;2"></a><code>[h2](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; <a id="the&#45;hgroup&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;3"></a><code>[h3](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; <a id="the&#45;hgroup&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;4"></a><code>[h4](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; <a id="the&#45;hgroup&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;5"></a><code>[h5](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; or <a id="the&#45;hgroup&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;6"></a><code>[h6](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code> element&#44; followed by zero or more <a id="the&#45;hgroup&#45;element&#58;the&#45;p&#45;element&#45;2"></a><code>[p](#the-p-element)</code> elements&#44; optionally intermixed with <a id="the&#45;hgroup&#45;element&#58;script&#45;supporting&#45;elements&#45;2"></a>[script&#45;supporting elements](html-language-foundations--877d6146487f--b55e73c31978.md#script-supporting-elements-2)&#46;

<strong><a id="the&#45;hgroup&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;hgroup&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;hgroup&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;hgroup&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-hgroup)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-hgroup)&#46;

<strong><a id="the&#45;hgroup&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;hgroup&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;hgroup&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;hgroup&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;hgroup&#45;element&#58;the&#45;hgroup&#45;element"></a><code>[hgroup](#the-hgroup-element)</code> element <a id="the&#45;hgroup&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a heading and related content&#46; The element may be used to group an <a id="the&#45;hgroup&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;7"></a><code>[h1](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>–<a id="the&#45;hgroup&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;8"></a><code>[h6](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code> element with one or more <a id="the&#45;hgroup&#45;element&#58;the&#45;p&#45;element&#45;3"></a><code>[p](#the-p-element)</code> elements containing content representing a subheading&#44; alternative title&#44; or tagline&#46;

<strong>Example&#58;</strong>

Here are some examples of valid headings contained within an <a id="the&#45;hgroup&#45;element&#58;the&#45;hgroup&#45;element&#45;2"></a><code>[hgroup](#the-hgroup-element)</code> element&#46;

``` text
<hgroup>
 <h1>The reality dysfunction</h1>
 <p>Space is not the only void</p>
</hgroup>
```

``` text
<hgroup>
 <h1>Dr. Strangelove</h1>
 <p>Or: How I Learned to Stop Worrying and Love the Bomb</p>
</hgroup>
```

#### <a id="the&#45;header&#45;element"></a>4&#46;3&#46;8 The <strong><code>header</code></strong> element[](#the-header-element)

<strong>✔</strong>MDN

[Element&#47;header](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/header)

Support in all current engines&#46;

Firefox4&#43;Safari5&#43;Chrome5&#43;

------------------------------------------------------------------------

Opera11&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer9&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android11&#46;1&#43;

<strong><a id="the&#45;header&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;header&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;header&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;header&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;header&#45;element&#58;flow&#45;content&#45;2&#45;2"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2) is expected&#46;

<strong><a id="the&#45;header&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;header&#45;element&#58;flow&#45;content&#45;2&#45;3"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#44; but with no <a id="the&#45;header&#45;element&#58;the&#45;header&#45;element"></a><code>[header](#the-header-element)</code> or <a id="the&#45;header&#45;element&#58;the&#45;footer&#45;element"></a><code>[footer](#the-footer-element)</code> element descendants&#46;

<strong><a id="the&#45;header&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;header&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;header&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;header&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

If there is an ancestor <a id="the&#45;header&#45;element&#58;sectioning&#45;content&#45;2"></a>[sectioning content](html-language-foundations--877d6146487f--b55e73c31978.md#sectioning-content-2) element&#58; [for authors](https://w3c.github.io/html-aria/#el-header)&#59; [for implementers](https://w3c.github.io/html-aam/#el-header)&#46;

Otherwise&#58; [for authors](https://w3c.github.io/html-aria/#el-header)&#59; [for implementers](https://w3c.github.io/html-aam/#el-header-ancestorbody)&#46;

<strong><a id="the&#45;header&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;header&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;header&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;header&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;header&#45;element&#58;the&#45;header&#45;element&#45;2"></a><code>[header](#the-header-element)</code> element <a id="the&#45;header&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a group of introductory or navigational aids&#46;

<strong>Note&#58;</strong>

A <a id="the&#45;header&#45;element&#58;the&#45;header&#45;element&#45;3"></a><code>[header](#the-header-element)</code> element is intended to usually contain a heading &#40;an <a id="the&#45;header&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements"></a><code>[h1](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>–<a id="the&#45;header&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;2"></a><code>[h6](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code> element or an <a id="the&#45;header&#45;element&#58;the&#45;hgroup&#45;element"></a><code>[hgroup](#the-hgroup-element)</code> element&#41;&#44; but this is not required&#46; The <a id="the&#45;header&#45;element&#58;the&#45;header&#45;element&#45;4"></a><code>[header](#the-header-element)</code> element can also be used to wrap a section&#39;s table of contents&#44; a search form&#44; or any relevant logos&#46;

<strong>Example&#58;</strong>

Here are some sample headers&#46; This first one is for a game&#58;

``` text
<header>
 <p>Welcome to...</p>
 <h1>Voidwars!</h1>
</header>
```

The following snippet shows how the element can be used to mark up a specification&#39;s header&#58;

``` text
<header>
 <hgroup>
  <h1>Fullscreen API</h1>
  <p>Living Standard — Last Updated 19 October 2015<p>
 </hgroup>
 <dl>
  <dt>Participate:</dt>
  <dd><a href="https://github.com/whatwg/fullscreen">GitHub whatwg/fullscreen</a></dd>
  <dt>Commits:</dt>
  <dd><a href="https://github.com/whatwg/fullscreen/commits">GitHub whatwg/fullscreen/commits</a></dd>
 </dl>
</header>
```

<strong>Note&#58;</strong>

The <a id="the&#45;header&#45;element&#58;the&#45;header&#45;element&#45;5"></a><code>[header](#the-header-element)</code> element is not <a id="the&#45;header&#45;element&#58;sectioning&#45;content&#45;2&#45;2"></a>[sectioning content](html-language-foundations--877d6146487f--b55e73c31978.md#sectioning-content-2)&#59; it doesn&#39;t introduce a new section&#46;

<strong>Example&#58;</strong>

In this example&#44; the page has a page heading given by the <a id="the&#45;header&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;3"></a><code>[h1](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code> element&#44; and two subsections whose headings are given by <a id="the&#45;header&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;4"></a><code>[h2](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code> elements&#46; The content after the <a id="the&#45;header&#45;element&#58;the&#45;header&#45;element&#45;6"></a><code>[header](#the-header-element)</code> element is still part of the last subsection started in the <a id="the&#45;header&#45;element&#58;the&#45;header&#45;element&#45;7"></a><code>[header](#the-header-element)</code> element&#44; because the <a id="the&#45;header&#45;element&#58;the&#45;header&#45;element&#45;8"></a><code>[header](#the-header-element)</code> element doesn&#39;t take part in the <a id="the&#45;header&#45;element&#58;outline"></a>[outline](#outline) algorithm&#46;

``` text
<body>
 <header>
  <h1>Little Green Guys With Guns</h1>
  <nav>
   <ul>
    <li><a href="/games">Games</a>
    <li><a href="/forum">Forum</a>
    <li><a href="/download">Download</a>
   </ul>
  </nav>
  <h2>Important News</h2> <!-- this starts a second subsection -->
  <!-- this is part of the subsection entitled "Important News" -->
  <p>To play today's games you will need to update your client.</p>
  <h2>Games</h2> <!-- this starts a third subsection -->
 </header>
 <p>You have three active games:</p>
 <!-- this is still part of the subsection entitled "Games" -->
 ...
```

#### <a id="the&#45;footer&#45;element"></a>4&#46;3&#46;9 The <strong><code>footer</code></strong> element[](#the-footer-element)

<strong>✔</strong>MDN

[Element&#47;footer](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/footer)

Support in all current engines&#46;

Firefox4&#43;Safari5&#43;Chrome5&#43;

------------------------------------------------------------------------

Opera11&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer9&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android11&#46;1&#43;

<strong><a id="the&#45;footer&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;footer&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;footer&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;footer&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;footer&#45;element&#58;flow&#45;content&#45;2&#45;2"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2) is expected&#46;

<strong><a id="the&#45;footer&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;footer&#45;element&#58;flow&#45;content&#45;2&#45;3"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#44; but with no <a id="the&#45;footer&#45;element&#58;the&#45;header&#45;element"></a><code>[header](#the-header-element)</code> or <a id="the&#45;footer&#45;element&#58;the&#45;footer&#45;element"></a><code>[footer](#the-footer-element)</code> element descendants&#46;

<strong><a id="the&#45;footer&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;footer&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;footer&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;footer&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

If there is an ancestor <a id="the&#45;footer&#45;element&#58;sectioning&#45;content&#45;2"></a>[sectioning content](html-language-foundations--877d6146487f--b55e73c31978.md#sectioning-content-2) element&#58; [for authors](https://w3c.github.io/html-aria/#el-footer)&#59; [for implementers](https://w3c.github.io/html-aam/#el-footer)&#46;

Otherwise&#58; [for authors](https://w3c.github.io/html-aria/#el-footer)&#59; [for implementers](https://w3c.github.io/html-aam/#el-footer-ancestorbody)&#46;

<strong><a id="the&#45;footer&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;footer&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;footer&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;footer&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;footer&#45;element&#58;the&#45;footer&#45;element&#45;2"></a><code>[footer](#the-footer-element)</code> element <a id="the&#45;footer&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a footer for its nearest ancestor <a id="the&#45;footer&#45;element&#58;sectioning&#45;content&#45;2&#45;2"></a>[sectioning content](html-language-foundations--877d6146487f--b55e73c31978.md#sectioning-content-2) element&#44; or for <a id="the&#45;footer&#45;element&#58;the&#45;body&#45;element&#45;2"></a>[the body element](html-language-foundations--877d6146487f--b55e73c31978.md#the-body-element-2) if there is no such ancestor&#46; A footer typically contains information about its section such as who wrote it&#44; links to related documents&#44; copyright data&#44; and the like&#46;

When the <a id="the&#45;footer&#45;element&#58;the&#45;footer&#45;element&#45;3"></a><code>[footer](#the-footer-element)</code> element contains entire sections&#44; they <a id="the&#45;footer&#45;element&#58;represents&#45;2"></a>[represent](html-language-foundations--877d6146487f--b55e73c31978.md#represents) appendices&#44; indices&#44; long colophons&#44; verbose license agreements&#44; and other such content&#46;

<strong>Note&#58;</strong>

Contact information for the author or editor of a section belongs in an <a id="the&#45;footer&#45;element&#58;the&#45;address&#45;element"></a><code>[address](#the-address-element)</code> element&#44; possibly itself inside a <a id="the&#45;footer&#45;element&#58;the&#45;footer&#45;element&#45;4"></a><code>[footer](#the-footer-element)</code>&#46; Bylines and other information that could be suitable for both a <a id="the&#45;footer&#45;element&#58;the&#45;header&#45;element&#45;2"></a><code>[header](#the-header-element)</code> or a <a id="the&#45;footer&#45;element&#58;the&#45;footer&#45;element&#45;5"></a><code>[footer](#the-footer-element)</code> can be placed in either &#40;or neither&#41;&#46; The primary purpose of these elements is merely to help the author write self&#45;explanatory markup that is easy to maintain and style&#59; they are not intended to impose specific structures on authors&#46;

Footers don&#39;t necessarily have to appear at the <em>end</em> of a section&#44; though they usually do&#46;

When there is no ancestor <a id="the&#45;footer&#45;element&#58;sectioning&#45;content&#45;2&#45;3"></a>[sectioning content](html-language-foundations--877d6146487f--b55e73c31978.md#sectioning-content-2) element&#44; then it applies to the whole page&#46;

<strong>Note&#58;</strong>

The <a id="the&#45;footer&#45;element&#58;the&#45;footer&#45;element&#45;6"></a><code>[footer](#the-footer-element)</code> element is not itself <a id="the&#45;footer&#45;element&#58;sectioning&#45;content&#45;2&#45;4"></a>[sectioning content](html-language-foundations--877d6146487f--b55e73c31978.md#sectioning-content-2)&#59; it doesn&#39;t introduce a new section&#46;

<strong>Example&#58;</strong>

Here is a page with two footers&#44; one at the top and one at the bottom&#44; with the same content&#58;

``` text
<body>
 <footer><a href="../">Back to index...</a></footer>
 <hgroup>
  <h1>Lorem ipsum</h1>
  <p>The ipsum of all lorems</p>
 </hgroup>
 <p>A dolor sit amet, consectetur adipisicing elit, sed do eiusmod
 tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim
 veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex
 ea commodo consequat. Duis aute irure dolor in reprehenderit in
 voluptate velit esse cillum dolore eu fugiat nulla
 pariatur. Excepteur sint occaecat cupidatat non proident, sunt in
 culpa qui officia deserunt mollit anim id est laborum.</p>
 <footer><a href="../">Back to index...</a></footer>
</body>
```

<strong>Example&#58;</strong>

Here is an example which shows the <a id="the&#45;footer&#45;element&#58;the&#45;footer&#45;element&#45;7"></a><code>[footer](#the-footer-element)</code> element being used both for a site&#45;wide footer and for a section footer&#46;

``` text
<!DOCTYPE HTML>
<HTML LANG="en"><HEAD>
<TITLE>The Ramblings of a Scientist</TITLE>
<BODY>
<H1>The Ramblings of a Scientist</H1>
<ARTICLE>
 <H1>Episode 15</H1>
 <VIDEO SRC="/fm/015.ogv" CONTROLS PRELOAD>
  <P><A HREF="/fm/015.ogv">Download video</A>.</P>
 </VIDEO>
 <FOOTER> <!-- footer for article -->
  <P>Published <TIME DATETIME="2009-10-21T18:26-07:00">on 2009/10/21 at 6:26pm</TIME></P>
 </FOOTER>
</ARTICLE>
<ARTICLE>
 <H1>My Favorite Trains</H1>
 <P>I love my trains. My favorite train of all time is a Köf.</P>
 <P>It is fun to see them pull some coal cars because they look so
 dwarfed in comparison.</P>
 <FOOTER> <!-- footer for article -->
  <P>Published <TIME DATETIME="2009-09-15T14:54-07:00">on 2009/09/15 at 2:54pm</TIME></P>
 </FOOTER>
</ARTICLE>
<FOOTER> <!-- site wide footer -->
 <NAV>
  <P><A HREF="/credits.html">Credits</A> —
     <A HREF="/tos.html">Terms of Service</A> —
     <A HREF="/index.html">Blog Index</A></P>
 </NAV>
 <P>Copyright © 2009 Gordon Freeman</P>
</FOOTER>
</BODY>
</HTML>
```

<strong>Example&#58;</strong>

Some site designs have what is sometimes referred to as &#34;fat footers&#34; — footers that contain a lot of material&#44; including images&#44; links to other articles&#44; links to pages for sending feedback&#44; special offers&#46;&#46;&#46; in some ways&#44; a whole &#34;front page&#34; in the footer&#46;

This fragment shows the bottom of a page on a site with a &#34;fat footer&#34;&#58;

``` text
...
 <footer>
  <nav>
   <section>
    <h1>Articles</h1>
    <p><img src="images/somersaults.jpeg" alt=""> Go to the gym with
    our somersaults class! Our teacher Jim takes you through the paces
    in this two-part article. <a href="articles/somersaults/1">Part
    1</a> · <a href="articles/somersaults/2">Part 2</a></p>
    <p><img src="images/kindplus.jpeg"> Tired of walking on the edge of
    a clif<!-- sic -->? Our guest writer Lara shows you how to bumble
    your way through the bars. <a href="articles/kindplus/1">Read
    more...</a></p>
    <p><img src="images/crisps.jpeg"> The chips are down, now all
    that's left is a potato. What can you do with it? <a
    href="articles/crisps/1">Read more...</a></p>
   </section>
   <ul>
    <li> <a href="/about">About us...</a>
    <li> <a href="/feedback">Send feedback!</a>
    <li> <a href="/sitemap">Sitemap</a>
   </ul>
  </nav>
  <p><small>Copyright © 2015 The Snacker —
  <a href="/tos">Terms of Service</a></small></p>
 </footer>
</body>
```

#### <a id="the&#45;address&#45;element"></a>4&#46;3&#46;10 The <strong><code>address</code></strong> element[](#the-address-element)

<strong>✔</strong>MDN

[Element&#47;address](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/address)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;address&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;address&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;address&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;address&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;address&#45;element&#58;flow&#45;content&#45;2&#45;2"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2) is expected&#46;

<strong><a id="the&#45;address&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;address&#45;element&#58;flow&#45;content&#45;2&#45;3"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#44; but with no <a id="the&#45;address&#45;element&#58;heading&#45;content&#45;2"></a>[heading content](html-language-foundations--877d6146487f--b55e73c31978.md#heading-content-2) descendants&#44; no <a id="the&#45;address&#45;element&#58;sectioning&#45;content&#45;2"></a>[sectioning content](html-language-foundations--877d6146487f--b55e73c31978.md#sectioning-content-2) descendants&#44; and no <a id="the&#45;address&#45;element&#58;the&#45;header&#45;element"></a><code>[header](#the-header-element)</code>&#44; <a id="the&#45;address&#45;element&#58;the&#45;footer&#45;element"></a><code>[footer](#the-footer-element)</code>&#44; or <a id="the&#45;address&#45;element&#58;the&#45;address&#45;element"></a><code>[address](#the-address-element)</code> element descendants&#46;

<strong><a id="the&#45;address&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;address&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;address&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;address&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-address)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-address)&#46;

<strong><a id="the&#45;address&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;address&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;address&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;address&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;address&#45;element&#58;the&#45;address&#45;element&#45;2"></a><code>[address](#the-address-element)</code> element <a id="the&#45;address&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) the contact information for its nearest <a id="the&#45;address&#45;element&#58;the&#45;article&#45;element"></a><code>[article](#the-article-element)</code> or <a id="the&#45;address&#45;element&#58;the&#45;body&#45;element"></a><code>[body](#the-body-element)</code> element ancestor&#46; If that is <a id="the&#45;address&#45;element&#58;the&#45;body&#45;element&#45;2"></a>[the body element](html-language-foundations--877d6146487f--b55e73c31978.md#the-body-element-2)&#44; then the contact information applies to the document as a whole&#46;

<strong>Example&#58;</strong>

For example&#44; a page at the W3C web site related to HTML might include the following contact information&#58;

``` text
<ADDRESS>
 <A href="../People/Raggett/">Dave Raggett</A>,
 <A href="../People/Arnaud/">Arnaud Le Hors</A>,
 contact persons for the <A href="Activity">W3C HTML Activity</A>
</ADDRESS>
```

The <a id="the&#45;address&#45;element&#58;the&#45;address&#45;element&#45;3"></a><code>[address](#the-address-element)</code> element must not be used to represent arbitrary addresses &#40;e&#46;g&#46; postal addresses&#41;&#44; unless those addresses are in fact the relevant contact information&#46; &#40;The <a id="the&#45;address&#45;element&#58;the&#45;p&#45;element"></a><code>[p](#the-p-element)</code> element is the appropriate element for marking up postal addresses in general&#46;&#41;

The <a id="the&#45;address&#45;element&#58;the&#45;address&#45;element&#45;4"></a><code>[address](#the-address-element)</code> element must not contain information other than contact information&#46;

<strong>Example&#58;</strong>

For example&#44; the following is non&#45;conforming use of the <a id="the&#45;address&#45;element&#58;the&#45;address&#45;element&#45;5"></a><code>[address](#the-address-element)</code> element&#58;

``` text
<ADDRESS>Last Modified: 1999/12/24 23:37:50</ADDRESS>
```

Typically&#44; the <a id="the&#45;address&#45;element&#58;the&#45;address&#45;element&#45;6"></a><code>[address](#the-address-element)</code> element would be included along with other information in a <a id="the&#45;address&#45;element&#58;the&#45;footer&#45;element&#45;2"></a><code>[footer](#the-footer-element)</code> element&#46;

The contact information for a node <var>node</var> is a collection of <a id="the&#45;address&#45;element&#58;the&#45;address&#45;element&#45;7"></a><code>[address](#the-address-element)</code> elements defined by the first applicable entry from the following list&#58;

<strong>If <var>node</var> is an <a id="the&#45;address&#45;element&#58;the&#45;article&#45;element&#45;2"></a><code>[article](#the-article-element)</code> element</strong>

<strong>If <var>node</var> is a <a id="the&#45;address&#45;element&#58;the&#45;body&#45;element&#45;3"></a><code>[body](#the-body-element)</code> element</strong>

The contact information consists of all the <a id="the&#45;address&#45;element&#58;the&#45;address&#45;element&#45;8"></a><code>[address](#the-address-element)</code> elements that have <var>node</var> as an ancestor and do not have another <a id="the&#45;address&#45;element&#58;the&#45;body&#45;element&#45;4"></a><code>[body](#the-body-element)</code> or <a id="the&#45;address&#45;element&#58;the&#45;article&#45;element&#45;3"></a><code>[article](#the-article-element)</code> element ancestor that is a descendant of <var>node</var>&#46;

<strong>If <var>node</var> has an ancestor element that is an <a id="the&#45;address&#45;element&#58;the&#45;article&#45;element&#45;4"></a><code>[article](#the-article-element)</code> element</strong>

<strong>If <var>node</var> has an ancestor element that is a <a id="the&#45;address&#45;element&#58;the&#45;body&#45;element&#45;5"></a><code>[body](#the-body-element)</code> element</strong>

The contact information of <var>node</var> is the same as the contact information of the nearest <a id="the&#45;address&#45;element&#58;the&#45;article&#45;element&#45;5"></a><code>[article](#the-article-element)</code> or <a id="the&#45;address&#45;element&#58;the&#45;body&#45;element&#45;6"></a><code>[body](#the-body-element)</code> element ancestor&#44; whichever is nearest&#46;

<strong>If <var>node</var>&#39;s <a id="the&#45;address&#45;element&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document) has <a id="the&#45;address&#45;element&#58;the&#45;body&#45;element&#45;2&#45;2"></a>[a body element](html-language-foundations--877d6146487f--b55e73c31978.md#the-body-element-2)</strong>

The contact information of <var>node</var> is the same as the contact information of <a id="the&#45;address&#45;element&#58;the&#45;body&#45;element&#45;2&#45;3"></a>[the body element](html-language-foundations--877d6146487f--b55e73c31978.md#the-body-element-2) of the <a id="the&#45;address&#45;element&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#46;

<strong>Otherwise</strong>

There is no contact information for <var>node</var>&#46;

User agents may expose the contact information of a node to the user&#44; or use it for other purposes&#44; such as indexing sections based on the sections&#39; contact information&#46;

<strong>Example&#58;</strong>

In this example the footer contains contact information and a copyright notice&#46;

``` text
<footer>
 <address>
  For more details, contact
  <a href="mailto:js@example.com">John Smith</a>.
 </address>
 <p><small>© copyright 2038 Example Corp.</small></p>
</footer>
```

#### <a id="headings&#45;and&#45;outlines&#45;2"></a>4&#46;3&#46;11 <a id="headings&#45;and&#45;outlines"></a><a id="outlines"></a>Headings and outlines[](#headings-and-outlines-2)

<a id="headings&#45;and&#45;outlines&#45;2&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements"></a><code>[h1](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>–<a id="headings&#45;and&#45;outlines&#45;2&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;2"></a><code>[h6](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code> elements have a <a id="heading&#45;level"></a><strong>heading level</strong>&#44; which is given by <a id="headings&#45;and&#45;outlines&#45;2&#58;get&#45;an&#45;element&#39;s&#45;computed&#45;heading&#45;level"></a>[getting the element&#39;s computed heading level](#get-an-element's-computed-heading-level)&#46;

These elements <a id="headings&#45;and&#45;outlines&#45;2&#58;represents"></a>[represent](html-language-foundations--877d6146487f--b55e73c31978.md#represents) <a id="concept&#45;heading"></a><strong>headings</strong>&#46; The lower a <a id="headings&#45;and&#45;outlines&#45;2&#58;concept&#45;heading"></a>[heading](#concept-heading)&#39;s <a id="headings&#45;and&#45;outlines&#45;2&#58;heading&#45;level"></a>[heading level](#heading-level) is&#44; the fewer ancestor sections the <a id="headings&#45;and&#45;outlines&#45;2&#58;concept&#45;heading&#45;2"></a>[heading](#concept-heading) has&#46;

The <a id="outline"></a><strong>outline</strong> is all <a id="headings&#45;and&#45;outlines&#45;2&#58;concept&#45;heading&#45;3"></a>[headings](#concept-heading) in a document&#44; in <a id="headings&#45;and&#45;outlines&#45;2&#58;tree&#45;order"></a>[tree order](https://dom.spec.whatwg.org/#concept-tree-order)&#46;

The <a id="headings&#45;and&#45;outlines&#45;2&#58;outline"></a>[outline](#outline) should be used for generating document outlines&#44; for example when generating tables of contents&#46; When creating an interactive table of contents&#44; entries should jump the user to the relevant <a id="headings&#45;and&#45;outlines&#45;2&#58;concept&#45;heading&#45;4"></a>[heading](#concept-heading)&#46;

If a document has one or more <a id="headings&#45;and&#45;outlines&#45;2&#58;concept&#45;heading&#45;5"></a>[headings](#concept-heading)&#44; at least a single <a id="headings&#45;and&#45;outlines&#45;2&#58;concept&#45;heading&#45;6"></a>[heading](#concept-heading) within the <a id="headings&#45;and&#45;outlines&#45;2&#58;outline&#45;2"></a>[outline](#outline) should have a <a id="headings&#45;and&#45;outlines&#45;2&#58;heading&#45;level&#45;2"></a>[heading level](#heading-level) of 1&#46;

Each <a id="headings&#45;and&#45;outlines&#45;2&#58;concept&#45;heading&#45;7"></a>[heading](#concept-heading) following another <a id="headings&#45;and&#45;outlines&#45;2&#58;concept&#45;heading&#45;8"></a>[heading](#concept-heading) <var>lead</var> in the <a id="headings&#45;and&#45;outlines&#45;2&#58;outline&#45;3"></a>[outline](#outline) must have a <a id="headings&#45;and&#45;outlines&#45;2&#58;heading&#45;level&#45;3"></a>[heading level](#heading-level) that is less than&#44; equal to&#44; or 1 greater than <var>lead</var>&#39;s <a id="headings&#45;and&#45;outlines&#45;2&#58;heading&#45;level&#45;4"></a>[heading level](#heading-level)&#46;

<strong>Example&#58;</strong>

The following example is non&#45;conforming&#58;

``` text
<body>
 <h1>Apples</h1>
 <p>Apples are fruit.</p>
 <section>
  <h3>Taste</h3>
  <p>They taste lovely.</p>
 </section>
</body>
```

It could be written as follows and then it would be conforming&#58;

``` text
<body>
 <h1>Apples</h1>
 <p>Apples are fruit.</p>
 <section>
  <h2>Taste</h2>
  <p>They taste lovely.</p>
 </section>
</body>
```

##### <a id="heading&#45;levels&#45;&#38;&#45;offsets"></a>4&#46;3&#46;11&#46;1 Heading levels &#38; offsets[](#heading-levels-&-offsets)

The <a id="attr&#45;headingoffset"></a><strong><code>headingoffset</code></strong> content attribute allows authors to offset heading levels for descendants&#46;

If the <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;attr&#45;headingoffset"></a><code>[headingoffset](#attr-headingoffset)</code> attribute is specified&#44; it must have a value that is a <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;valid&#45;non&#45;negative&#45;integer"></a>[valid non&#45;negative integer](html-language-foundations--877d6146487f--b55e73c31978.md#valid-non-negative-integer) between 0 and 8&#44; inclusive&#46;

The <a id="attr&#45;headingreset"></a><strong><code>headingreset</code></strong> content attribute is a <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;boolean&#45;attribute"></a>[boolean attribute](html-language-foundations--877d6146487f--b55e73c31978.md#boolean-attribute)&#46; It allows authors to prevent a heading offset computation from traversing beyond the element with the attribute&#46;

To <a id="get&#45;an&#45;element&#39;s&#45;computed&#45;heading&#45;level"></a><strong>get an element&#39;s computed heading level</strong>&#44; given an element <var>element</var>&#58;

1.  Let <var>level</var> be 0&#46;

2.  If <var>element</var>&#39;s local name is <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements"></a><code>[h1](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; then set <var>level</var> to 1&#46;

3.  If <var>element</var>&#39;s local name is <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;2"></a><code>[h2](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; then set <var>level</var> to 2&#46;

4.  If <var>element</var>&#39;s local name is <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;3"></a><code>[h3](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; then set <var>level</var> to 3&#46;

5.  If <var>element</var>&#39;s local name is <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;4"></a><code>[h4](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; then set <var>level</var> to 4&#46;

6.  If <var>element</var>&#39;s local name is <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;5"></a><code>[h5](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; then set <var>level</var> to 5&#46;

7.  If <var>element</var>&#39;s local name is <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;6"></a><code>[h6](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; then set <var>level</var> to 6&#46;

8.  <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;assert"></a>[Assert](https://infra.spec.whatwg.org/#assert)&#58; <var>level</var> is not 0&#46;

9.  Increment <var>level</var> by the result of <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;get&#45;an&#45;element&#39;s&#45;computed&#45;heading&#45;offset"></a>[getting an element&#39;s computed heading offset](#get-an-element's-computed-heading-offset) given <var>element</var>&#46;

10. If <var>level</var> is greater than 9&#44; then return 9&#46;

11. Return <var>level</var>&#46;

To <a id="get&#45;an&#45;element&#39;s&#45;computed&#45;heading&#45;offset"></a><strong>get an element&#39;s computed heading offset</strong>&#44; given an element <var>element</var>&#44; perform the following steps&#46; They return a non&#45;negative integer&#46;

1.  Let <var>offset</var> be 0&#46;

2.  Let <var>inclusiveAncestor</var> be <var>element</var>&#46;

3.  While <var>inclusiveAncestor</var> is not null&#58;

    1.  Let <var>nextOffset</var> be 0&#46;

    2.  If <var>inclusiveAncestor</var> is an <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;html&#45;elements"></a>[HTML element](html-language-foundations--877d6146487f--b55e73c31978.md#html-elements) and has a <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;attr&#45;headingoffset&#45;2"></a><code>[headingoffset](#attr-headingoffset)</code> attribute&#44; then parse its value using the <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;rules&#45;for&#45;parsing&#45;non&#45;negative&#45;integers"></a>[rules for parsing non&#45;negative integers](html-language-foundations--877d6146487f--b55e73c31978.md#rules-for-parsing-non-negative-integers)&#46;

        If the result of parsing the value is not an error&#44; then set <var>nextOffset</var> to that value&#46;

    3.  Increment <var>offset</var> by <var>nextOffset</var>&#46;

    4.  If <var>inclusiveAncestor</var> is an <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;html&#45;elements&#45;2"></a>[HTML element](html-language-foundations--877d6146487f--b55e73c31978.md#html-elements) and has a <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;attr&#45;headingreset"></a><code>[headingreset](#attr-headingreset)</code> attribute&#44; then return <var>offset</var>&#46;

    5.  If <var>inclusiveAncestor</var>&#39;s parent is a <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;shadow&#45;root"></a>[shadow root](https://dom.spec.whatwg.org/#concept-shadow-root)&#44; then set <var>inclusiveAncestor</var> to that <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;shadow&#45;root&#45;2"></a>[shadow root](https://dom.spec.whatwg.org/#concept-shadow-root)&#39;s <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;concept&#45;documentfragment&#45;host"></a>[host](https://dom.spec.whatwg.org/#concept-documentfragment-host) and <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;continue"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue)&#46;

    6.  Set <var>inclusiveAncestor</var> to <var>inclusiveAncestor</var>&#39;s <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;parent&#45;element"></a>[parent element](https://dom.spec.whatwg.org/#parent-element)&#46;

4.  Return <var>offset</var>&#46;

<strong>Example&#58;</strong>

This example shows a combination of <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;attr&#45;headingoffset&#45;3"></a><code>[headingoffset](#attr-headingoffset)</code>&#44; <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;attr&#45;headingreset&#45;2"></a><code>[headingreset](#attr-headingreset)</code>&#44; and <a id="heading&#45;levels&#45;&#38;&#45;offsets&#58;attr&#45;aria&#45;level"></a><code>[aria&#45;level](https://w3c.github.io/aria/#aria-level)</code> attributes with comments demonstrating the respective heading levels&#46; This example illustrates the various combinations and is not a best practice example&#46;

``` text
<body>
  <main>
   <h1>This is a heading level 1</h1>
   <article headingoffset="1">
    <h1>This is a heading level 2</h1>
    <section headingoffset="1">
     <h1>This is a heading level 3</h1>
     <dialog headingreset>
      <h1>This is a heading level 1</h1>
     </dialog>
    </section>
   </article>
   <h1 aria-level="2">This is a heading level 2</h1>
  </main>
</body>
```

##### <a id="sample&#45;outlines"></a>4&#46;3&#46;11&#46;2 Sample outlines[](#sample-outlines)

<strong>Example&#58;</strong>

The following markup fragment&#58;

``` text
<body>
  <hgroup id="document-title">
    <h1>HTML: Living Standard</h1>
    <p>Last Updated 12 August 2016</p>
  </hgroup>
  <p>Some intro to the document.</p>
  <h2>Table of contents</h2>
  <ol id=toc>...</ol>
  <h2>First section</h2>
  <p>Some intro to the first section.</p>
</body>
```

&#46;&#46;&#46;results in 3 document headings&#58;

1.  <code>&#60;h1&#62;HTML&#58; Living Standard&#60;&#47;h1&#62;</code>

2.  <code>&#60;h2&#62;Table of contents&#60;&#47;h2&#62;</code>&#46;

3.  <code>&#60;h2&#62;First section&#60;&#47;h2&#62;</code>&#46;

A rendered view of the <a id="sample&#45;outlines&#58;outline"></a>[outline](#outline) might look like&#58;

![Top&#45;level section with the heading &#34;HTML&#58; Living Standard&#34; and two subsections&#59; &#34;Table of contents&#34; and &#34;First section&#34;&#46;](https://html.spec.whatwg.org/images/outline.svg)

<strong>Example&#58;</strong>

First&#44; here is a document&#44; which is a book with very short chapters and subsections&#58;

``` text
<!DOCTYPE HTML>
<html lang=en>
<title>The Tax Book (all in one page)</title>
<h1>The Tax Book</h1>
<h2>Earning money</h2>
<p>Earning money is good.</p>
<h3>Getting a job</h3>
<p>To earn money you typically need a job.</p>
<h2>Spending money</h2>
<p>Spending is what money is mainly used for.</p>
<h3>Cheap things</h3>
<p>Buying cheap things often not cost-effective.</p>
<h3>Expensive things</h3>
<p>The most expensive thing is often not the most cost-effective either.</p>
<h2>Investing money</h2>
<p>You can lend your money to other people.</p>
<h2>Losing money</h2>
<p>If you spend money or invest money, sooner or later you will lose money.
<h3>Poor judgement</h3>
<p>Usually if you lose money it's because you made a mistake.</p>
```

Its <a id="sample&#45;outlines&#58;outline&#45;2"></a>[outline](#outline) could be presented as follows&#58;

1.  The Tax Book
    1.  Earning money
        1.  Getting a job
    2.  Spending money
        1.  Cheap things
        2.  Expensive things
    3.  Investing money
    4.  Losing money
        1.  Poor judgement

Notice that the <a id="sample&#45;outlines&#58;the&#45;title&#45;element"></a><code>[title](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-title-element)</code> element is not a <a id="sample&#45;outlines&#58;concept&#45;heading"></a>[heading](#concept-heading)&#46;

<strong>Example&#58;</strong>

A document can contain multiple top&#45;level headings&#58;

``` text
<!DOCTYPE HTML>
<html lang=en>
<title>Alphabetic Fruit</title>
<h1>Apples</h1>
<p>Pomaceous.</p>
<h1>Bananas</h1>
<p>Edible.</p>
<h1>Carambola</h1>
<p>Star.</p>
```

The document&#39;s <a id="sample&#45;outlines&#58;outline&#45;3"></a>[outline](#outline) could be presented as follows&#58;

1.  Apples
2.  Bananas
3.  Carambola

<strong>Example&#58;</strong>

<a id="sample&#45;outlines&#58;the&#45;header&#45;element"></a><code>[header](#the-header-element)</code> elements do not influence the <a id="sample&#45;outlines&#58;outline&#45;4"></a>[outline](#outline) of a document&#58;

``` text
<!DOCTYPE HTML>
<html lang="en">
<title>We're adopting a child! — Ray's blog</title>
<h1>Ray's blog</h1>
<article>
 <header>
  <nav>
   <a href="?t=-1d">Yesterday</a>;
   <a href="?t=-7d">Last week</a>;
   <a href="?t=-1m">Last month</a>
  </nav>
  <h2>We're adopting a child!</h2>
 </header>
 <p>As of today, Janine and I have signed the papers to become
 the proud parents of baby Diane! We've been looking forward to
 this day for weeks.</p>
</article>
</html>
```

The document&#39;s <a id="sample&#45;outlines&#58;outline&#45;5"></a>[outline](#outline) could be presented as follows&#58;

1.  Ray&#39;s blog
    1.  We&#39;re adopting a child&#33;

------------------------------------------------------------------------

<strong>Example&#58;</strong>

The following example is conforming&#44; but not encouraged as it has no <a id="sample&#45;outlines&#58;concept&#45;heading&#45;2"></a>[heading](#concept-heading) whose <a id="sample&#45;outlines&#58;heading&#45;level"></a>[heading level](#heading-level) is 1&#58;

``` text
<!DOCTYPE HTML>
<html lang=en>
<title>Alphabetic Fruit</title>
<section>
 <h2>Apples</h2>
 <p>Pomaceous.</p>
</section>
<section>
 <h2>Bananas</h2>
 <p>Edible.</p>
</section>
<section>
 <h2>Carambola</h2>
 <p>Star.</p>
</section>
```

The document&#39;s <a id="sample&#45;outlines&#58;outline&#45;6"></a>[outline](#outline) could be presented as follows&#58;

1.  1.  Apples
    2.  Bananas
    3.  Carambola

<strong>Example&#58;</strong>

The following example is conforming&#44; but not encouraged as the first <a id="sample&#45;outlines&#58;concept&#45;heading&#45;3"></a>[heading](#concept-heading)&#39;s <a id="sample&#45;outlines&#58;heading&#45;level&#45;2"></a>[heading level](#heading-level) is not 1&#58;

``` text
<!DOCTYPE HTML>
<html lang=en>
<title>Feathers on The Site of Encyclopedic Knowledge</title>
 <h2>A plea from our caretakers</h2>
 <p>Please, we beg of you, send help! We're stuck in the server room!</p>
<h1>Feathers</h1>
<p>Epidermal growths.</p>
```

The document&#39;s <a id="sample&#45;outlines&#58;outline&#45;7"></a>[outline](#outline) could be presented as follows&#58;

1.  1.  A plea from our caretakers
2.  Feathers

##### <a id="exposing&#45;outlines&#45;to&#45;users"></a>4&#46;3&#46;11&#46;3 Exposing outlines to users[](#exposing-outlines-to-users)

User agents are encouraged to expose page <a id="exposing&#45;outlines&#45;to&#45;users&#58;outline"></a>[outlines](#outline) to users to aid in navigation&#46; This is especially true for non&#45;visual media&#44; e&#46;g&#46; screen readers&#46;

<strong>Example&#58;</strong>

For instance&#44; a user agent could map the arrow keys as follows&#58;

<strong>Shift &#43; ← Left</strong>

Go to previous heading

<strong>Shift &#43; → Right</strong>

Go to next heading

<strong>Shift &#43; ↑ Up</strong>

Go to next heading whose <a id="exposing&#45;outlines&#45;to&#45;users&#58;heading&#45;level"></a>[level](#heading-level) is one less than the current heading&#39;s level

<strong>Shift &#43; ↓ Down</strong>

Go to next heading whose <a id="exposing&#45;outlines&#45;to&#45;users&#58;heading&#45;level&#45;2"></a>[level](#heading-level) is the same as the current heading&#39;s level

#### <a id="usage&#45;summary&#45;2"></a>4&#46;3&#46;12 Usage summary[](#usage-summary-2)

<em>This section is non&#45;normative&#46;</em>

<table class="vertical-summary-table"><thead><tr><th rowspan="2">Element      </th><th>Purpose     </th></tr><tr><th>Example    </th></tr></thead><tbody><tr><td rowspan="2"><code id="usage-summary-2:the-body-element"><a href="#the-body-element">body</a></code>      </td><td>The contents of the document&#46;     </td></tr><tr><td><code class="example">&lt;&#33;DOCTYPE HTML&gt;&#10;&lt;html lang&#61;&#34;en&#34;&gt;&#10; &lt;head&gt; &lt;title&gt;Steve Hill&#39;s Home Page&lt;&#47;title&gt; &lt;&#47;head&gt;&#10; &lt;body&gt; &lt;p&gt;Hard Trance is My Life&#46;&lt;&#47;p&gt; &lt;&#47;body&gt;&#10;&lt;&#47;html&gt;</code>      </td></tr><tr><td rowspan="2"><code id="usage-summary-2:the-article-element"><a href="#the-article-element">article</a></code>      </td><td>A complete&#44; or self&#45;contained&#44;   composition in a document&#44; page&#44; application&#44; or site and that is&#44; in principle&#44; independently   distributable or reusable&#44; e&#46;g&#46; in syndication&#46; This could be a forum post&#44; a magazine or   newspaper article&#44; a blog entry&#44; a user&#45;submitted comment&#44; an interactive widget or gadget&#44; or any   other independent item of content&#46;     </td></tr><tr><td><code class="example">&lt;article&gt;&#10; &lt;img src&#61;&#34;&#47;tumblr&#95;masqy2s5yn1rzfqbpo1&#95;500&#46;jpg&#34; alt&#61;&#34;Yellow smiley face with the caption &#39;masif&#39;&#34;&gt;&#10; &lt;p&gt;My fave Masif tee so far&#33;&lt;&#47;p&gt;&#10; &lt;footer&gt;Posted 2 days ago&lt;&#47;footer&gt;&#10;&lt;&#47;article&gt;&#10;&lt;article&gt;&#10; &lt;img src&#61;&#34;&#47;tumblr&#95;m9tf6wSr6W1rzfqbpo1&#95;500&#46;jpg&#34; alt&#61;&#34;&#34;&gt;&#10; &lt;p&gt;Happy 2nd birthday Masif Saturdays&#33;&#33;&#33;&lt;&#47;p&gt;&#10; &lt;footer&gt;Posted 3 weeks ago&lt;&#47;footer&gt;&#10;&lt;&#47;article&gt;</code>      </td></tr><tr><td rowspan="2"><code id="usage-summary-2:the-section-element"><a href="#the-section-element">section</a></code>      </td><td>A generic section of a document or   application&#46; A section&#44; in this context&#44; is a thematic grouping of content&#44; typically with a   heading&#46;     </td></tr><tr><td><code class="example">&lt;h1&gt;Biography&lt;&#47;h1&gt;&#10;&lt;section&gt;&#10; &lt;h1&gt;The facts&lt;&#47;h1&gt;&#10; &lt;p&gt;1500&#43; shows&#44; 14&#43; countries&lt;&#47;p&gt;&#10;&lt;&#47;section&gt;&#10;&lt;section&gt;&#10; &lt;h1&gt;2010&#47;2011 figures per year&lt;&#47;h1&gt;&#10; &lt;p&gt;100&#43; shows&#44; 8&#43; countries&lt;&#47;p&gt;&#10;&lt;&#47;section&gt;</code>      </td></tr><tr><td rowspan="2"><code id="usage-summary-2:the-nav-element"><a href="#the-nav-element">nav</a></code>      </td><td>A section of a page that links to other   pages or to parts within the page&#58; a section with navigation links&#46;     </td></tr><tr><td><code class="example">&lt;nav&gt;&#10; &lt;p&gt;&lt;a href&#61;&#34;&#47;&#34;&gt;Home&lt;&#47;a&gt;&#10; &lt;p&gt;&lt;a href&#61;&#34;&#47;biog&#46;html&#34;&gt;Bio&lt;&#47;a&gt;&#10; &lt;p&gt;&lt;a href&#61;&#34;&#47;discog&#46;html&#34;&gt;Discog&lt;&#47;a&gt;&#10;&lt;&#47;nav&gt;</code>      </td></tr><tr><td rowspan="2"><code id="usage-summary-2:the-aside-element"><a href="#the-aside-element">aside</a></code>      </td><td>A section of a page that consists of   content that is tangentially related to the content around the <code id="usage-summary-2:the-aside-element-2"><a href="#the-aside-element">aside</a></code> element&#44; and   which could be considered separate from that content&#46; Such sections are often represented as   sidebars in printed typography&#46;     </td></tr><tr><td><code class="example">&lt;h1&gt;Music&lt;&#47;h1&gt;&#10;&lt;p&gt;As any burner can tell you&#44; the event has a lot of trance&#46;&lt;&#47;p&gt;&#10;&lt;aside&gt;You can buy the music we played at our &lt;a href&#61;&#34;buy&#46;html&#34;&gt;playlist page&lt;&#47;a&gt;&#46;&lt;&#47;aside&gt;&#10;&lt;p&gt;This year we played a kind of trance that originated in Belgium&#44; Germany&#44; and the Netherlands in the mid&#45;90s&#46;&lt;&#47;p&gt;</code>      </td></tr><tr><td rowspan="2"><code id="usage-summary-2:the-h1,-h2,-h3,-h4,-h5,-and-h6-elements"><a href="#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements">h1</a></code>–<code id="usage-summary-2:the-h1,-h2,-h3,-h4,-h5,-and-h6-elements-2"><a href="#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements">h6</a></code>      </td><td>A heading     </td></tr><tr><td><code class="example">&lt;h1&gt;The Guide To Music On The Playa&lt;&#47;h1&gt;&#10;&lt;h2&gt;The Main Stage&lt;&#47;h2&gt;&#10;&lt;p&gt;If you want to play on a stage&#44; you should bring one&#46;&lt;&#47;p&gt;&#10;&lt;h2&gt;Amplified Music&lt;&#47;h2&gt;&#10;&lt;p&gt;Amplifiers up to 300W or 90dB are welcome&#46;&lt;&#47;p&gt;</code>      </td></tr><tr><td rowspan="2"><code id="usage-summary-2:the-hgroup-element"><a href="#the-hgroup-element">hgroup</a></code>      </td><td>A heading and related content&#46; The   element may be used to group an <code id="usage-summary-2:the-h1,-h2,-h3,-h4,-h5,-and-h6-elements-3"><a href="#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements">h1</a></code>–<code id="usage-summary-2:the-h1,-h2,-h3,-h4,-h5,-and-h6-elements-4"><a href="#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements">h6</a></code> element with one or more   <code id="usage-summary-2:the-p-element"><a href="#the-p-element">p</a></code> elements containing content representing a subheading&#44; alternative title&#44; or   tagline&#46;     </td></tr><tr><td><code class="example">&lt;hgroup&gt;&#10; &lt;h1&gt;Burning Music&lt;&#47;h1&gt;&#10; &lt;p&gt;The Guide To Music On The Playa&lt;&#47;p&gt;&#10;&lt;&#47;hgroup&gt;&#10;&lt;section&gt;&#10; &lt;hgroup&gt;&#10;  &lt;h1&gt;Main Stage&lt;&#47;h1&gt;&#10;  &lt;p&gt;The Fiction Of A Music Festival&lt;&#47;p&gt;&#10; &lt;&#47;hgroup&gt;&#10; &lt;p&gt;If you want to play on a stage&#44; you should bring one&#46;&lt;&#47;p&gt;&#10;&lt;&#47;section&gt;&#10;&lt;section&gt;&#10; &lt;hgroup&gt;&#10;  &lt;h1&gt;Loudness&#33;&lt;&#47;h1&gt;&#10;  &lt;p&gt;Questions About Amplified Music&lt;&#47;p&gt;&#10; &lt;&#47;hgroup&gt;&#10; &lt;p&gt;Amplifiers up to 300W or 90dB are welcome&#46;&lt;&#47;p&gt;&#10;&lt;&#47;section&gt;</code>      </td></tr><tr><td rowspan="2"><code id="usage-summary-2:the-header-element"><a href="#the-header-element">header</a></code>      </td><td>A group of introductory or navigational   aids&#46;     </td></tr><tr><td><code class="example">&lt;article&gt;&#10; &lt;header&gt;&#10;  &lt;h1&gt;Hard Trance is My Life&lt;&#47;h1&gt;&#10;  &lt;p&gt;By DJ Steve Hill and Technikal&lt;&#47;p&gt;&#10; &lt;&#47;header&gt;&#10; &lt;p&gt;The album with the amusing punctuation has red artwork&#46;&lt;&#47;p&gt;&#10;&lt;&#47;article&gt;</code>      </td></tr><tr><td rowspan="2"><code id="usage-summary-2:the-footer-element"><a href="#the-footer-element">footer</a></code>      </td><td>A footer for its nearest ancestor   <a href="html-language-foundations--877d6146487f--b55e73c31978.md#sectioning-content-2" id="usage-summary-2:sectioning-content-2">sectioning content</a> element&#44; or for <a href="html-language-foundations--877d6146487f--b55e73c31978.md#the-body-element-2" id="usage-summary-2:the-body-element-2">the body element</a> if there is no such   ancestor&#46; A footer typically contains information about its section such as who wrote it&#44; links   to related documents&#44; copyright data&#44; and the like&#46;     </td></tr><tr><td><code class="example">&lt;article&gt;&#10; &lt;h1&gt;Hard Trance is My Life&lt;&#47;h1&gt;&#10; &lt;p&gt;The album with the amusing punctuation has red artwork&#46;&lt;&#47;p&gt;&#10; &lt;footer&gt;&#10;  &lt;p&gt;Artists&#58; DJ Steve Hill and Technikal&lt;&#47;p&gt;&#10; &lt;&#47;footer&gt;&#10;&lt;&#47;article&gt;</code>    </td></tr></tbody></table>

##### <a id="article&#45;or&#45;section"></a>4&#46;3&#46;12&#46;1 Article or section&#63;[](#article-or-section)

<em>This section is non&#45;normative&#46;</em>

A <a id="article&#45;or&#45;section&#58;the&#45;section&#45;element"></a><code>[section](#the-section-element)</code> forms part of something else&#46; An <a id="article&#45;or&#45;section&#58;the&#45;article&#45;element"></a><code>[article](#the-article-element)</code> is its own thing&#46; But how does one know which is which&#63; Mostly the real answer is &#34;it depends on author intent&#34;&#46;

For example&#44; one could imagine a book with a &#34;Granny Smith&#34; chapter that just said &#34;These juicy&#44; green apples make a great filling for apple pies&#46;&#34;&#59; that would be a <a id="article&#45;or&#45;section&#58;the&#45;section&#45;element&#45;2"></a><code>[section](#the-section-element)</code> because there&#39;d be lots of other chapters on &#40;maybe&#41; other kinds of apples&#46;

On the other hand&#44; one could imagine a tweet or reddit comment or tumblr post or newspaper classified ad that just said &#34;Granny Smith&#46; These juicy&#44; green apples make a great filling for apple pies&#46;&#34;&#59; it would then be <a id="article&#45;or&#45;section&#58;the&#45;article&#45;element&#45;2"></a><code>[article](#the-article-element)</code>s because that was the whole thing&#46;

A comment on an article is not part of the <a id="article&#45;or&#45;section&#58;the&#45;article&#45;element&#45;3"></a><code>[article](#the-article-element)</code> on which it is commenting&#44; therefore it is its own <a id="article&#45;or&#45;section&#58;the&#45;article&#45;element&#45;4"></a><code>[article](#the-article-element)</code>&#46;

### <a id="grouping&#45;content"></a>4&#46;4 Grouping content[](#grouping-content)

#### <a id="the&#45;p&#45;element"></a>4&#46;4&#46;1 The <strong><code>p</code></strong> element[](#the-p-element)

<strong>✔</strong>MDN

[Element&#47;p](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong>✔</strong>MDN

[HTMLParagraphElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLParagraphElement)

Support in all current engines&#46;

Firefox1&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

<strong><a id="the&#45;p&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;p&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;p&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;p&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;p&#45;element&#58;flow&#45;content&#45;2&#45;2"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2) is expected&#46;

As a child of an <a id="the&#45;p&#45;element&#58;the&#45;hgroup&#45;element"></a><code>[hgroup](#the-hgroup-element)</code> element&#46;

<strong><a id="the&#45;p&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;p&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;p&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

A <a id="the&#45;p&#45;element&#58;the&#45;p&#45;element"></a><code>[p](#the-p-element)</code> element&#39;s <a id="the&#45;p&#45;element&#58;syntax&#45;end&#45;tag"></a>[end tag](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#syntax-end-tag) can be omitted if the <a id="the&#45;p&#45;element&#58;the&#45;p&#45;element&#45;2"></a><code>[p](#the-p-element)</code> element is immediately followed by an <a id="the&#45;p&#45;element&#58;the&#45;address&#45;element"></a><code>[address](#the-address-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;article&#45;element"></a><code>[article](#the-article-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;aside&#45;element"></a><code>[aside](#the-aside-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;blockquote&#45;element"></a><code>[blockquote](#the-blockquote-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;details&#45;element"></a><code>[details](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-details-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;dialog&#45;element"></a><code>[dialog](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-dialog-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;div&#45;element"></a><code>[div](#the-div-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;dl&#45;element"></a><code>[dl](#the-dl-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;fieldset&#45;element"></a><code>[fieldset](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-fieldset-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;figcaption&#45;element"></a><code>[figcaption](#the-figcaption-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;figure&#45;element"></a><code>[figure](#the-figure-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;footer&#45;element"></a><code>[footer](#the-footer-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements"></a><code>[h1](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;2"></a><code>[h2](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;3"></a><code>[h3](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;4"></a><code>[h4](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;5"></a><code>[h5](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;6"></a><code>[h6](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;header&#45;element"></a><code>[header](#the-header-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;hgroup&#45;element&#45;2"></a><code>[hgroup](#the-hgroup-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;hr&#45;element"></a><code>[hr](#the-hr-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;main&#45;element"></a><code>[main](#the-main-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;menu&#45;element"></a><code>[menu](#the-menu-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;nav&#45;element"></a><code>[nav](#the-nav-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;ol&#45;element"></a><code>[ol](#the-ol-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;p&#45;element&#45;3"></a><code>[p](#the-p-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;pre&#45;element"></a><code>[pre](#the-pre-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;search&#45;element"></a><code>[search](#the-search-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;section&#45;element"></a><code>[section](#the-section-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;table&#45;element"></a><code>[table](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-table-element)</code>&#44; or <a id="the&#45;p&#45;element&#58;the&#45;ul&#45;element"></a><code>[ul](#the-ul-element)</code> element&#44; or if there is no more content in the parent element and the parent element is an <a id="the&#45;p&#45;element&#58;html&#45;elements"></a>[HTML element](html-language-foundations--877d6146487f--b55e73c31978.md#html-elements) that is not an <a id="the&#45;p&#45;element&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;audio&#45;element"></a><code>[audio](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-audio-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;del&#45;element"></a><code>[del](#the-del-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;ins&#45;element"></a><code>[ins](#the-ins-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;map&#45;element"></a><code>[map](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-map-element)</code>&#44; <a id="the&#45;p&#45;element&#58;the&#45;noscript&#45;element"></a><code>[noscript](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-noscript-element)</code>&#44; or <a id="the&#45;p&#45;element&#58;the&#45;video&#45;element"></a><code>[video](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-video-element)</code> element&#44; or an <a id="the&#45;p&#45;element&#58;autonomous&#45;custom&#45;element"></a>[autonomous custom element](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#autonomous-custom-element)&#46;

<strong><a id="the&#45;p&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;p&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;p&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-p)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-p)&#46;

<strong><a id="the&#45;p&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;p&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;p&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

<a id="htmlparagraphelement"></a><a id="the&#45;p&#45;element&#58;htmlelement"></a><a id="the&#45;p&#45;element&#58;htmlconstructor"></a>

``` text
[Exposed=Window]
interface HTMLParagraphElement : HTMLElement {
  [HTMLConstructor] constructor();

  // also has obsolete members
};
```

The <a id="the&#45;p&#45;element&#58;the&#45;p&#45;element&#45;4"></a><code>[p](#the-p-element)</code> element <a id="the&#45;p&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a <a id="the&#45;p&#45;element&#58;paragraph"></a>[paragraph](html-language-foundations--877d6146487f--b55e73c31978.md#paragraph)&#46;

<strong>Note&#58;</strong>

While paragraphs are usually represented in visual media by blocks of text that are physically separated from adjacent blocks through blank lines&#44; a style sheet or user agent would be equally justified in presenting paragraph breaks in a different manner&#44; for instance using inline pilcrows &#40;¶&#41;&#46;

<strong>Example&#58;</strong>

The following examples are conforming HTML fragments&#58;

``` text
<p>The little kitten gently seated herself on a piece of
carpet. Later in her life, this would be referred to as the time the
cat sat on the mat.</p>
```

``` text
<fieldset>
 <legend>Personal information</legend>
 <p>
   <label>Name: <input name="n"></label>
   <label><input name="anon" type="checkbox"> Hide from other users</label>
 </p>
 <p><label>Address: <textarea name="a"></textarea></label></p>
</fieldset>
```

``` text
<p>There was once an example from Femley,<br>
Whose markup was of dubious quality.<br>
The validator complained,<br>
So the author was pained,<br>
To move the error from the markup to the rhyming.</p>
```

The <a id="the&#45;p&#45;element&#58;the&#45;p&#45;element&#45;5"></a><code>[p](#the-p-element)</code> element should not be used when a more specific element is more appropriate&#46;

<strong>Example&#58;</strong>

The following example is technically correct&#58;

``` text
<section>
 <!-- ... -->
 <p>Last modified: 2001-04-23</p>
 <p>Author: fred@example.com</p>
</section>
```

However&#44; it would be better marked&#45;up as&#58;

``` text
<section>
 <!-- ... -->
 <footer>Last modified: 2001-04-23</footer>
 <address>Author: fred@example.com</address>
</section>
```

Or&#58;

``` text
<section>
 <!-- ... -->
 <footer>
  <p>Last modified: 2001-04-23</p>
  <address>Author: fred@example.com</address>
 </footer>
</section>
```

<strong>Note&#58;</strong>

List elements &#40;in particular&#44; <a id="the&#45;p&#45;element&#58;the&#45;ol&#45;element&#45;2"></a><code>[ol](#the-ol-element)</code> and <a id="the&#45;p&#45;element&#58;the&#45;ul&#45;element&#45;2"></a><code>[ul](#the-ul-element)</code> elements&#41; cannot be children of <a id="the&#45;p&#45;element&#58;the&#45;p&#45;element&#45;6"></a><code>[p](#the-p-element)</code> elements&#46; When a sentence contains a bulleted list&#44; therefore&#44; one might wonder how it should be marked up&#46;

<strong>Example&#58;</strong>

For instance&#44; this fantastic sentence has bullets relating to

- wizards&#44;
- faster&#45;than&#45;light travel&#44; and
- telepathy&#44;

and is further discussed below&#46;

The solution is to realize that a <a id="the&#45;p&#45;element&#58;paragraph&#45;2"></a><em>[paragraph](html-language-foundations--877d6146487f--b55e73c31978.md#paragraph)</em>&#44; in HTML terms&#44; is not a logical concept&#44; but a structural one&#46; In the fantastic example above&#44; there are actually <em>five</em> <a id="the&#45;p&#45;element&#58;paragraph&#45;3"></a>[paragraphs](html-language-foundations--877d6146487f--b55e73c31978.md#paragraph) as defined by this specification&#58; one before the list&#44; one for each bullet&#44; and one after the list&#46;

<strong>Example&#58;</strong>

The markup for the above example could therefore be&#58;

``` text
<p>For instance, this fantastic sentence has bullets relating to</p>
<ul>
 <li>wizards,
 <li>faster-than-light travel, and
 <li>telepathy,
</ul>
<p>and is further discussed below.</p>
```

Authors wishing to conveniently style such &#34;logical&#34; paragraphs consisting of multiple &#34;structural&#34; paragraphs can use the <a id="the&#45;p&#45;element&#58;the&#45;div&#45;element&#45;2"></a><code>[div](#the-div-element)</code> element instead of the <a id="the&#45;p&#45;element&#58;the&#45;p&#45;element&#45;7"></a><code>[p](#the-p-element)</code> element&#46;

<strong>Example&#58;</strong>

Thus for instance the above example could become the following&#58;

``` text
<div>For instance, this fantastic sentence has bullets relating to
<ul>
 <li>wizards,
 <li>faster-than-light travel, and
 <li>telepathy,
</ul>
and is further discussed below.</div>
```

This example still has five structural paragraphs&#44; but now the author can style just the <a id="the&#45;p&#45;element&#58;the&#45;div&#45;element&#45;3"></a><code>[div](#the-div-element)</code> instead of having to consider each part of the example separately&#46;

#### <a id="the&#45;hr&#45;element"></a>4&#46;4&#46;2 The <strong><code>hr</code></strong> element[](#the-hr-element)

<strong>✔</strong>MDN

[Element&#47;hr](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/hr)

Support in all current engines&#46;

Firefox1&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

<strong>✔</strong>MDN

[HTMLHRElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLHRElement)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

<strong><a id="the&#45;hr&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;hr&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<strong><a id="the&#45;hr&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;hr&#45;element&#58;flow&#45;content&#45;2&#45;2"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2) is expected&#46;

As a descendant of a <a id="the&#45;hr&#45;element&#58;the&#45;select&#45;element"></a><code>[select](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-select-element)</code> element&#46;

<strong><a id="the&#45;hr&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;hr&#45;element&#58;concept&#45;content&#45;nothing"></a>[Nothing](html-language-foundations--877d6146487f--b55e73c31978.md#concept-content-nothing)&#46;

<strong><a id="the&#45;hr&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

No <a id="the&#45;hr&#45;element&#58;syntax&#45;end&#45;tag"></a>[end tag](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#syntax-end-tag)&#46;

<strong><a id="the&#45;hr&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;hr&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;hr&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-hr)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-hr)&#46;

<strong><a id="the&#45;hr&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;hr&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;hr&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

<a id="htmlhrelement"></a><a id="the&#45;hr&#45;element&#58;htmlelement"></a><a id="the&#45;hr&#45;element&#58;htmlconstructor"></a>

``` text
[Exposed=Window]
interface HTMLHRElement : HTMLElement {
  [HTMLConstructor] constructor();

  // also has obsolete members
};
```

The <a id="the&#45;hr&#45;element&#58;the&#45;hr&#45;element"></a><code>[hr](#the-hr-element)</code> element <a id="the&#45;hr&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a <a id="the&#45;hr&#45;element&#58;paragraph"></a>[paragraph](html-language-foundations--877d6146487f--b55e73c31978.md#paragraph)&#45;level thematic break&#44; e&#46;g&#46;&#44; a scene change in a story&#44; or a transition to another topic within a section of a reference book&#59; alternatively&#44; it represents a separator between a set of options of a <a id="the&#45;hr&#45;element&#58;the&#45;select&#45;element&#45;2"></a><code>[select](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-select-element)</code> element&#46;

<strong>Example&#58;</strong>

The following fictional extract from a project manual shows two sections that use the <a id="the&#45;hr&#45;element&#58;the&#45;hr&#45;element&#45;2"></a><code>[hr](#the-hr-element)</code> element to separate topics within the section&#46;

``` text
<section>
 <h1>Communication</h1>
 <p>There are various methods of communication. This section
 covers a few of the important ones used by the project.</p>
 <hr>
 <p>Communication stones seem to come in pairs and have mysterious
 properties:</p>
 <ul>
  <li>They can transfer thoughts in two directions once activated
  if used alone.</li>
  <li>If used with another device, they can transfer one's
  consciousness to another body.</li>
  <li>If both stones are used with another device, the
  consciousnesses switch bodies.</li>
 </ul>
 <hr>
 <p>Radios use the electromagnetic spectrum in the meter range and
 longer.</p>
 <hr>
 <p>Signal flares use the electromagnetic spectrum in the
 nanometer range.</p>
</section>
<section>
 <h1>Food</h1>
 <p>All food at the project is rationed:</p>
 <dl>
  <dt>Potatoes</dt>
  <dd>Two per day</dd>
  <dt>Soup</dt>
  <dd>One bowl per day</dd>
 </dl>
 <hr>
 <p>Cooking is done by the chefs on a set rotation.</p>
</section>
```

There is no need for an <a id="the&#45;hr&#45;element&#58;the&#45;hr&#45;element&#45;3"></a><code>[hr](#the-hr-element)</code> element between the sections themselves&#44; since the <a id="the&#45;hr&#45;element&#58;the&#45;section&#45;element"></a><code>[section](#the-section-element)</code> elements and the <a id="the&#45;hr&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements"></a><code>[h1](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code> elements imply thematic changes themselves&#46;

<strong>Example&#58;</strong>

The following extract from Pandora&#39;s Star by Peter F&#46; Hamilton shows two paragraphs that precede a scene change and the paragraph that follows it&#46; The scene change&#44; represented in the printed book by a gap containing a solitary centered star between the second and third paragraphs&#44; is here represented using the <a id="the&#45;hr&#45;element&#58;the&#45;hr&#45;element&#45;4"></a><code>[hr](#the-hr-element)</code> element&#46;

``` text
<p>Dudley was ninety-two, in his second life, and fast approaching
time for another rejuvenation. Despite his body having the physical
age of a standard fifty-year-old, the prospect of a long degrading
campaign within academia was one he regarded with dread. For a
supposedly advanced civilization, the Intersolar Commonwealth could be
appallingly backward at times, not to mention cruel.</p>
<p><i>Maybe it won't be that bad</i>, he told himself. The lie was
comforting enough to get him through the rest of the night's
shift.</p>
<hr>
<p>The Carlton AllLander drove Dudley home just after dawn. Like the
astronomer, the vehicle was old and worn, but perfectly capable of
doing its job. It had a cheap diesel engine, common enough on a
semi-frontier world like Gralmond, although its drive array was a
thoroughly modern photoneural processor. With its high suspension and
deep-tread tyres it could plough along the dirt track to the
observatory in all weather and seasons, including the metre-deep snow
of Gralmond's winters.</p>
```

<strong>Note&#58;</strong>

The <a id="the&#45;hr&#45;element&#58;the&#45;hr&#45;element&#45;5"></a><code>[hr](#the-hr-element)</code> element does not affect the document&#39;s <a id="the&#45;hr&#45;element&#58;outline"></a>[outline](#outline)&#46;

#### <a id="the&#45;pre&#45;element"></a>4&#46;4&#46;3 The <strong><code>pre</code></strong> element[](#the-pre-element)

<strong>✔</strong>MDN

[Element&#47;pre](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/pre)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong>✔</strong>MDN

[HTMLPreElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLPreElement)

Support in all current engines&#46;

Firefox1&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer6&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

<strong><a id="the&#45;pre&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;pre&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;pre&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;pre&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;pre&#45;element&#58;flow&#45;content&#45;2&#45;2"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2) is expected&#46;

<strong><a id="the&#45;pre&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;pre&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;pre&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;pre&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;pre&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;pre&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-pre)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-pre)&#46;

<strong><a id="the&#45;pre&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;pre&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;pre&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

<a id="htmlpreelement"></a><a id="the&#45;pre&#45;element&#58;htmlelement"></a><a id="the&#45;pre&#45;element&#58;htmlconstructor"></a>

``` text
[Exposed=Window]
interface HTMLPreElement : HTMLElement {
  [HTMLConstructor] constructor();

  // also has obsolete members
};
```

The <a id="the&#45;pre&#45;element&#58;the&#45;pre&#45;element"></a><code>[pre](#the-pre-element)</code> element <a id="the&#45;pre&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a block of preformatted text&#44; in which structure is represented by typographic conventions rather than by elements&#46;

<strong>Note&#58;</strong>

In <a id="the&#45;pre&#45;element&#58;syntax"></a>[the HTML syntax](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#syntax)&#44; a leading newline character immediately following the <a id="the&#45;pre&#45;element&#58;the&#45;pre&#45;element&#45;2"></a><code>[pre](#the-pre-element)</code> element start tag is stripped&#46;

Some examples of cases where the <a id="the&#45;pre&#45;element&#58;the&#45;pre&#45;element&#45;3"></a><code>[pre](#the-pre-element)</code> element could be used&#58;

- Including an email&#44; with paragraphs indicated by blank lines&#44; lists indicated by lines prefixed with a bullet&#44; and so on&#46;
- Including fragments of computer code&#44; with structure indicated according to the conventions of that language&#46;
- Displaying ASCII art&#46;

<strong>Note&#58;</strong>

Authors are encouraged to consider how preformatted text will be experienced when the formatting is lost&#44; as will be the case for users of speech synthesizers&#44; braille displays&#44; and the like&#46; For cases like ASCII art&#44; it is likely that an alternative presentation&#44; such as a textual description&#44; would be more universally accessible to the readers of the document&#46;

To represent a block of computer code&#44; the <a id="the&#45;pre&#45;element&#58;the&#45;pre&#45;element&#45;4"></a><code>[pre](#the-pre-element)</code> element can be used with a <a id="the&#45;pre&#45;element&#58;the&#45;code&#45;element"></a><code>[code](#the-code-element)</code> element&#59; to represent a block of computer output the <a id="the&#45;pre&#45;element&#58;the&#45;pre&#45;element&#45;5"></a><code>[pre](#the-pre-element)</code> element can be used with a <a id="the&#45;pre&#45;element&#58;the&#45;samp&#45;element"></a><code>[samp](#the-samp-element)</code> element&#46; Similarly&#44; the <a id="the&#45;pre&#45;element&#58;the&#45;kbd&#45;element"></a><code>[kbd](#the-kbd-element)</code> element can be used within a <a id="the&#45;pre&#45;element&#58;the&#45;pre&#45;element&#45;6"></a><code>[pre](#the-pre-element)</code> element to indicate text that the user is to enter&#46;

<strong>Note&#58;</strong>

This element [has rendering requirements involving the bidirectional algorithm](html-language-foundations--877d6146487f--b55e73c31978.md#bidireq)&#46;

<strong>Example&#58;</strong>

In the following snippet&#44; a sample of computer code is presented&#46;

``` text
<p>This is the <code>Panel</code> constructor:</p>
<pre><code>function Panel(element, canClose, closeHandler) {
  this.element = element;
  this.canClose = canClose;
  this.closeHandler = function () { if (closeHandler) closeHandler() };
}</code></pre>
```

<strong>Example&#58;</strong>

In the following snippet&#44; <a id="the&#45;pre&#45;element&#58;the&#45;samp&#45;element&#45;2"></a><code>[samp](#the-samp-element)</code> and <a id="the&#45;pre&#45;element&#58;the&#45;kbd&#45;element&#45;2"></a><code>[kbd](#the-kbd-element)</code> elements are mixed in the contents of a <a id="the&#45;pre&#45;element&#58;the&#45;pre&#45;element&#45;7"></a><code>[pre](#the-pre-element)</code> element to show a session of Zork I&#46;

``` text
<pre><samp>You are in an open field west of a big white house with a boarded
front door.
There is a small mailbox here.

></samp> <kbd>open mailbox</kbd>

<samp>Opening the mailbox reveals:
A leaflet.

></samp></pre>
```

<strong>Example&#58;</strong>

The following shows a contemporary poem that uses the <a id="the&#45;pre&#45;element&#58;the&#45;pre&#45;element&#45;8"></a><code>[pre](#the-pre-element)</code> element to preserve its unusual formatting&#44; which forms an intrinsic part of the poem itself&#46;

``` text
<pre>                maxling

it is with a          heart
               heavy

that i admit loss of a feline
        so           loved

a friend lost to the
        unknown
                                (night)

~cdr 11dec07</pre>
```

#### <a id="the&#45;blockquote&#45;element"></a>4&#46;4&#46;4 The <strong><code>blockquote</code></strong> element[](#the-blockquote-element)

<strong>✔</strong>MDN

[Element&#47;blockquote](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/blockquote)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong>✔</strong>MDN

[HTMLQuoteElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLQuoteElement)

Support in all current engines&#46;

Firefox1&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer6&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android37&#43;Samsung Internet&#63;Opera Android12&#46;1&#43;

<strong><a id="the&#45;blockquote&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;blockquote&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;blockquote&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;blockquote&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;blockquote&#45;element&#58;flow&#45;content&#45;2&#45;2"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2) is expected&#46;

<strong><a id="the&#45;blockquote&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;blockquote&#45;element&#58;flow&#45;content&#45;2&#45;3"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<strong><a id="the&#45;blockquote&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;blockquote&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;blockquote&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<a id="the&#45;blockquote&#45;element&#58;attr&#45;blockquote&#45;cite"></a><code>[cite](#attr-blockquote-cite)</code> — Link to the source of the quotation or more information about the edit

<strong><a id="the&#45;blockquote&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-blockquote)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-blockquote)&#46;

<strong><a id="the&#45;blockquote&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;blockquote&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default) with attributes <a id="the&#45;blockquote&#45;element&#58;attr&#45;blockquote&#45;cite&#45;2"></a><code>[cite](#attr-blockquote-cite)</code>&#46;

<strong><a id="the&#45;blockquote&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

<a id="htmlquoteelement"></a><a id="the&#45;blockquote&#45;element&#58;htmlelement"></a><a id="the&#45;blockquote&#45;element&#58;htmlconstructor"></a><a id="the&#45;blockquote&#45;element&#58;cereactions"></a><a id="the&#45;blockquote&#45;element&#58;xattr&#45;reflecturl"></a><a id="dom&#45;quote&#45;cite"></a>

``` text
[Exposed=Window]
interface HTMLQuoteElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions, ReflectURL] attribute USVString cite;
};
```

<strong>Note&#58;</strong>

The <a id="the&#45;blockquote&#45;element&#58;htmlquoteelement"></a><code>[HTMLQuoteElement](#htmlquoteelement)</code> interface is also used by the <a id="the&#45;blockquote&#45;element&#58;the&#45;q&#45;element"></a><code>[q](#the-q-element)</code> element&#46;

The <a id="the&#45;blockquote&#45;element&#58;the&#45;blockquote&#45;element"></a><code>[blockquote](#the-blockquote-element)</code> element <a id="the&#45;blockquote&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a section that is quoted from another source&#46;

Content inside a <a id="the&#45;blockquote&#45;element&#58;the&#45;blockquote&#45;element&#45;2"></a><code>[blockquote](#the-blockquote-element)</code> must be quoted from another source&#44; whose address&#44; if it has one&#44; may be cited in the <a id="attr&#45;blockquote&#45;cite"></a><strong><code>cite</code></strong> attribute&#46;

If the <a id="the&#45;blockquote&#45;element&#58;attr&#45;blockquote&#45;cite&#45;3"></a><code>[cite](#attr-blockquote-cite)</code> attribute is present&#44; it must be a <a id="the&#45;blockquote&#45;element&#58;valid&#45;url&#45;potentially&#45;surrounded&#45;by&#45;spaces"></a>[valid URL potentially surrounded by spaces](html-language-foundations--877d6146487f--b55e73c31978.md#valid-url-potentially-surrounded-by-spaces)&#46; To obtain the corresponding citation link&#44; the value of the attribute must be <a id="the&#45;blockquote&#45;element&#58;encoding&#45;parsing&#45;a&#45;url"></a>[parsed](html-language-foundations--877d6146487f--b55e73c31978.md#encoding-parsing-a-url) relative to the element&#39;s <a id="the&#45;blockquote&#45;element&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46; User agents may allow users to follow such citation links&#44; but they are primarily intended for private use &#40;e&#46;g&#46;&#44; by server&#45;side scripts collecting statistics about a site&#39;s use of quotations&#41;&#44; not for readers&#46;

The content of a <a id="the&#45;blockquote&#45;element&#58;the&#45;blockquote&#45;element&#45;3"></a><code>[blockquote](#the-blockquote-element)</code> may be abbreviated or may have context added in the conventional manner for the text&#39;s language&#46;

<strong>Example&#58;</strong>

For example&#44; in English this is traditionally done using square brackets&#46; Consider a page with the sentence &#34;Jane ate the cracker&#46; She then said she liked apples and fish&#46;&#34;&#59; it could be quoted as follows&#58;

``` text
<blockquote>
 <p>[Jane] then said she liked [...] fish.</p>
</blockquote>
```

Attribution for the quotation&#44; if any&#44; must be placed outside the <a id="the&#45;blockquote&#45;element&#58;the&#45;blockquote&#45;element&#45;4"></a><code>[blockquote](#the-blockquote-element)</code> element&#46;

<strong>Example&#58;</strong>

For example&#44; here the attribution is given in a paragraph after the quote&#58;

``` text
<blockquote>
 <p>I contend that we are both atheists. I just believe in one fewer
 god than you do. When you understand why you dismiss all the other
 possible gods, you will understand why I dismiss yours.</p>
</blockquote>
<p>— Stephen Roberts</p>
```

The other examples below show other ways of showing attribution&#46;

<strong>Example&#58;</strong>

Here a <a id="the&#45;blockquote&#45;element&#58;the&#45;blockquote&#45;element&#45;5"></a><code>[blockquote](#the-blockquote-element)</code> element is used in conjunction with a <a id="the&#45;blockquote&#45;element&#58;the&#45;figure&#45;element"></a><code>[figure](#the-figure-element)</code> element and its <a id="the&#45;blockquote&#45;element&#58;the&#45;figcaption&#45;element"></a><code>[figcaption](#the-figcaption-element)</code> to clearly relate a quote to its attribution &#40;which is not part of the quote and therefore doesn&#39;t belong inside the <a id="the&#45;blockquote&#45;element&#58;the&#45;blockquote&#45;element&#45;6"></a><code>[blockquote](#the-blockquote-element)</code> itself&#41;&#58;

``` text
<figure>
 <blockquote>
  <p>The truth may be puzzling. It may take some work to grapple with.
  It may be counterintuitive. It may contradict deeply held
  prejudices. It may not be consonant with what we desperately want to
  be true. But our preferences do not determine what's true. We have a
  method, and that method helps us to reach not absolute truth, only
  asymptotic approaches to the truth — never there, just closer
  and closer, always finding vast new oceans of undiscovered
  possibilities. Cleverly designed experiments are the key.</p>
 </blockquote>
 <figcaption>Carl Sagan, in "<cite>Wonder and Skepticism</cite>", from
 the <cite>Skeptical Inquirer</cite> Volume 19, Issue 1 (January-February
 1995)</figcaption>
</figure>
```

<strong>Example&#58;</strong>

This next example shows the use of <a id="the&#45;blockquote&#45;element&#58;the&#45;cite&#45;element"></a><code>[cite](#the-cite-element)</code> alongside <a id="the&#45;blockquote&#45;element&#58;the&#45;blockquote&#45;element&#45;7"></a><code>[blockquote](#the-blockquote-element)</code>&#58;

``` text
<p>His next piece was the aptly named <cite>Sonnet 130</cite>:</p>
<blockquote cite="https://quotes.example.org/s/sonnet130.html">
  <p>My mistress' eyes are nothing like the sun,<br>
  Coral is far more red, than her lips red,<br>
  ...
```

<strong>Example&#58;</strong>

This example shows how a forum post could use <a id="the&#45;blockquote&#45;element&#58;the&#45;blockquote&#45;element&#45;8"></a><code>[blockquote](#the-blockquote-element)</code> to show what post a user is replying to&#46; The <a id="the&#45;blockquote&#45;element&#58;the&#45;article&#45;element"></a><code>[article](#the-article-element)</code> element is used for each post&#44; to mark up the threading&#46;

``` text
<article>
 <h1><a href="https://bacon.example.com/?blog=109431">Bacon on a crowbar</a></h1>
 <article>
  <header><strong>t3yw</strong> 12 points 1 hour ago</header>
  <p>I bet a narwhal would love that.</p>
  <footer><a href="?pid=29578">permalink</a></footer>
  <article>
   <header><strong>greg</strong> 8 points 1 hour ago</header>
   <blockquote><p>I bet a narwhal would love that.</p></blockquote>
   <p>Dude narwhals don't eat bacon.</p>
   <footer><a href="?pid=29579">permalink</a></footer>
   <article>
    <header><strong>t3yw</strong> 15 points 1 hour ago</header>
    <blockquote>
     <blockquote><p>I bet a narwhal would love that.</p></blockquote>
     <p>Dude narwhals don't eat bacon.</p>
    </blockquote>
    <p>Next thing you'll be saying they don't get capes and wizard
    hats either!</p>
    <footer><a href="?pid=29580">permalink</a></footer>
    <article>
     <article>
      <header><strong>boing</strong> -5 points 1 hour ago</header>
      <p>narwhals are worse than ceiling cat</p>
      <footer><a href="?pid=29581">permalink</a></footer>
     </article>
    </article>
   </article>
  </article>
  <article>
   <header><strong>fred</strong> 1 points 23 minutes ago</header>
   <blockquote><p>I bet a narwhal would love that.</p></blockquote>
   <p>I bet they'd love to peel a banana too.</p>
   <footer><a href="?pid=29582">permalink</a></footer>
  </article>
 </article>
</article>
```

<strong>Example&#58;</strong>

This example shows the use of a <a id="the&#45;blockquote&#45;element&#58;the&#45;blockquote&#45;element&#45;9"></a><code>[blockquote](#the-blockquote-element)</code> for short snippets&#44; demonstrating that one does not have to use <a id="the&#45;blockquote&#45;element&#58;the&#45;p&#45;element"></a><code>[p](#the-p-element)</code> elements inside <a id="the&#45;blockquote&#45;element&#58;the&#45;blockquote&#45;element&#45;10"></a><code>[blockquote](#the-blockquote-element)</code> elements&#58;

``` text
<p>He began his list of "lessons" with the following:</p>
<blockquote>One should never assume that his side of
the issue will be recognized, let alone that it will
be conceded to have merits.</blockquote>
<p>He continued with a number of similar points, ending with:</p>
<blockquote>Finally, one should be prepared for the threat
of breakdown in negotiations at any given moment and not
be cowed by the possibility.</blockquote>
<p>We shall now discuss these points...
```

<strong>Note&#58;</strong>

[Examples of how to represent a conversation](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#conversations) are shown in a later section&#59; it is not appropriate to use the <a id="the&#45;blockquote&#45;element&#58;the&#45;cite&#45;element&#45;2"></a><code>[cite](#the-cite-element)</code> and <a id="the&#45;blockquote&#45;element&#58;the&#45;blockquote&#45;element&#45;11"></a><code>[blockquote](#the-blockquote-element)</code> elements for this purpose&#46;

#### <a id="the&#45;ol&#45;element"></a>4&#46;4&#46;5 The <strong><code>ol</code></strong> element[](#the-ol-element)

<strong>✔</strong>MDN

[Element&#47;ol](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ol)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong>✔</strong>MDN

[HTMLOListElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLOListElement)

Support in all current engines&#46;

Firefox1&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

<strong><a id="the&#45;ol&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;ol&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

If the element&#39;s children include at least one <a id="the&#45;ol&#45;element&#58;the&#45;li&#45;element"></a><code>[li](#the-li-element)</code> element&#58; <a id="the&#45;ol&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;ol&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;ol&#45;element&#58;flow&#45;content&#45;2&#45;2"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2) is expected&#46;

<strong><a id="the&#45;ol&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

Zero or more <a id="the&#45;ol&#45;element&#58;the&#45;li&#45;element&#45;2"></a><code>[li](#the-li-element)</code> and <a id="the&#45;ol&#45;element&#58;script&#45;supporting&#45;elements&#45;2"></a>[script&#45;supporting](html-language-foundations--877d6146487f--b55e73c31978.md#script-supporting-elements-2) elements&#46;

<strong><a id="the&#45;ol&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;ol&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;ol&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<a id="the&#45;ol&#45;element&#58;attr&#45;ol&#45;reversed"></a><code>[reversed](#attr-ol-reversed)</code> — Number the list backwards

<a id="the&#45;ol&#45;element&#58;attr&#45;ol&#45;start"></a><code>[start](#attr-ol-start)</code> — <a id="the&#45;ol&#45;element&#58;concept&#45;ol&#45;start"></a>[Starting value](#concept-ol-start) of the list

<a id="the&#45;ol&#45;element&#58;attr&#45;ol&#45;type"></a><code>[type](#attr-ol-type)</code> — Kind of list marker

<strong><a id="the&#45;ol&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-ol)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-ol)&#46;

<strong><a id="the&#45;ol&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;ol&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default) with attributes <a id="the&#45;ol&#45;element&#58;attr&#45;ol&#45;reversed&#45;2"></a><code>[reversed](#attr-ol-reversed)</code>&#44; <a id="the&#45;ol&#45;element&#58;attr&#45;ol&#45;start&#45;2"></a><code>[start](#attr-ol-start)</code>&#44; <a id="the&#45;ol&#45;element&#58;attr&#45;ol&#45;type&#45;2"></a><code>[type](#attr-ol-type)</code>&#46;

<strong><a id="the&#45;ol&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

<a id="htmlolistelement"></a><a id="the&#45;ol&#45;element&#58;htmlelement"></a><a id="the&#45;ol&#45;element&#58;htmlconstructor"></a><a id="the&#45;ol&#45;element&#58;cereactions"></a><a id="the&#45;ol&#45;element&#58;xattr&#45;reflect"></a><a id="dom&#45;ol&#45;reversed"></a><a id="the&#45;ol&#45;element&#58;cereactions&#45;2"></a><a id="the&#45;ol&#45;element&#58;xattr&#45;reflect&#45;2"></a><a id="the&#45;ol&#45;element&#58;xattr&#45;reflectdefault"></a><a id="dom&#45;ol&#45;start"></a><a id="the&#45;ol&#45;element&#58;cereactions&#45;3"></a><a id="the&#45;ol&#45;element&#58;xattr&#45;reflect&#45;3"></a><a id="dom&#45;ol&#45;type"></a>

``` text
[Exposed=Window]
interface HTMLOListElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions, Reflect] attribute boolean reversed;
  [CEReactions, Reflect, ReflectDefault=1] attribute long start;
  [CEReactions, Reflect] attribute DOMString type;

  // also has obsolete members
};
```

The <a id="the&#45;ol&#45;element&#58;the&#45;ol&#45;element"></a><code>[ol](#the-ol-element)</code> element <a id="the&#45;ol&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a list of items&#44; where the items have been intentionally ordered&#44; such that changing the order would change the meaning of the document&#46;

The items of the list are the <a id="the&#45;ol&#45;element&#58;the&#45;li&#45;element&#45;3"></a><code>[li](#the-li-element)</code> element child nodes of the <a id="the&#45;ol&#45;element&#58;the&#45;ol&#45;element&#45;2"></a><code>[ol](#the-ol-element)</code> element&#44; in <a id="the&#45;ol&#45;element&#58;tree&#45;order"></a>[tree order](https://dom.spec.whatwg.org/#concept-tree-order)&#46;

<strong>✔</strong>MDN

[Element&#47;ol&#35;attr&#45;reversed](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ol#attr-reversed)

Support in all current engines&#46;

Firefox18&#43;Safari6&#43;Chrome18&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;≤79&#43;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

The <a id="attr&#45;ol&#45;reversed"></a><strong><code>reversed</code></strong> attribute is a <a id="the&#45;ol&#45;element&#58;boolean&#45;attribute"></a>[boolean attribute](html-language-foundations--877d6146487f--b55e73c31978.md#boolean-attribute)&#46; If present&#44; it indicates that the list is a descending list &#40;&#46;&#46;&#46;&#44; 3&#44; 2&#44; 1&#41;&#46; If the attribute is omitted&#44; the list is an ascending list &#40;1&#44; 2&#44; 3&#44; &#46;&#46;&#46;&#41;&#46;

The <a id="attr&#45;ol&#45;start"></a><strong><code>start</code></strong> attribute&#44; if present&#44; must be a <a id="the&#45;ol&#45;element&#58;valid&#45;integer"></a>[valid integer](html-language-foundations--877d6146487f--b55e73c31978.md#valid-integer)&#46; It is used to determine the <a id="the&#45;ol&#45;element&#58;concept&#45;ol&#45;start&#45;2"></a>[starting value](#concept-ol-start) of the list&#46;

An <a id="the&#45;ol&#45;element&#58;the&#45;ol&#45;element&#45;3"></a><code>[ol](#the-ol-element)</code> element has a <a id="concept&#45;ol&#45;start"></a><strong>starting value</strong>&#44; which is an integer determined as follows&#58;

1.  If the <a id="the&#45;ol&#45;element&#58;the&#45;ol&#45;element&#45;4"></a><code>[ol](#the-ol-element)</code> element has a <a id="the&#45;ol&#45;element&#58;attr&#45;ol&#45;start&#45;3"></a><code>[start](#attr-ol-start)</code> attribute&#58;

    1.  Let <var>parsed</var> be the result of <a id="the&#45;ol&#45;element&#58;rules&#45;for&#45;parsing&#45;integers"></a>[parsing the value of the attribute as an integer](html-language-foundations--877d6146487f--b55e73c31978.md#rules-for-parsing-integers)&#46;

    2.  If <var>parsed</var> is not an error&#44; then return <var>parsed</var>&#46;

2.  If the <a id="the&#45;ol&#45;element&#58;the&#45;ol&#45;element&#45;5"></a><code>[ol](#the-ol-element)</code> element has a <a id="the&#45;ol&#45;element&#58;attr&#45;ol&#45;reversed&#45;3"></a><code>[reversed](#attr-ol-reversed)</code> attribute&#44; then return the number of <a id="the&#45;ol&#45;element&#58;list&#45;owner"></a>[owned <code>li</code> elements](#list-owner)&#46;

3.  Return 1&#46;

The <a id="attr&#45;ol&#45;type"></a><strong><code>type</code></strong> attribute can be used to specify the kind of marker to use in the list&#44; in the cases where that matters &#40;e&#46;g&#46; because items are to be <a id="the&#45;ol&#45;element&#58;referenced"></a>[referenced](html-language-foundations--877d6146487f--b55e73c31978.md#referenced) by their number&#47;letter&#41;&#46; The attribute&#44; if specified&#44; must have a value that is <a id="the&#45;ol&#45;element&#58;identical&#45;to"></a>[identical to](https://infra.spec.whatwg.org/#string-is) one of the characters given in the first cell of one of the rows of the following table&#46; The <a id="the&#45;ol&#45;element&#58;attr&#45;ol&#45;type&#45;3"></a><code>[type](#attr-ol-type)</code> attribute represents the state given in the cell in the second column of the row whose first cell matches the attribute&#39;s value&#59; if none of the cells match&#44; or if the attribute is omitted&#44; then the attribute represents the <a id="the&#45;ol&#45;element&#58;attr&#45;ol&#45;type&#45;state&#45;decimal"></a>[decimal](#attr-ol-type-state-decimal) state&#46;

<table><thead><tr><th>Keyword      </th><th>State      </th><th>Description      </th><th colspan="8">Examples for values 1&#45;3 and 3999&#45;4001    </th></tr></thead><tbody><tr><td><dfn data-dfn-for="ol/type" id="attr-ol-type-keyword-decimal" data-dfn-type="attr-value"><code>1</code></dfn>      &#40;U&#43;0031&#41;      </td><td><dfn id="attr-ol-type-state-decimal">decimal</dfn>      </td><td>Decimal numbers      </td><td class="eg"><samp>1&#46;</samp> </td><td class="eg"><samp>2&#46;</samp> </td><td class="eg"><samp>3&#46;</samp> </td><td class="eg">&#46;&#46;&#46; </td><td class="eg"><samp>3999&#46;</samp> </td><td class="eg"><samp>4000&#46;</samp> </td><td class="eg"><samp>4001&#46;</samp> </td><td class="eg">&#46;&#46;&#46;     </td></tr><tr><td><dfn data-dfn-for="ol/type" id="attr-ol-type-keyword-lower-alpha" data-dfn-type="attr-value"><code>a</code></dfn> &#40;U&#43;0061&#41;      </td><td><dfn id="attr-ol-type-state-lower-alpha">lower&#45;alpha</dfn>      </td><td>Lowercase latin alphabet      </td><td class="eg"><samp>a&#46;</samp> </td><td class="eg"><samp>b&#46;</samp> </td><td class="eg"><samp>c&#46;</samp> </td><td class="eg">&#46;&#46;&#46; </td><td class="eg"><samp>ewu&#46;</samp> </td><td class="eg"><samp>ewv&#46;</samp> </td><td class="eg"><samp>eww&#46;</samp> </td><td class="eg">&#46;&#46;&#46;     </td></tr><tr><td><dfn data-dfn-for="ol/type" id="attr-ol-type-keyword-upper-alpha" data-dfn-type="attr-value"><code>A</code></dfn> &#40;U&#43;0041&#41;      </td><td><dfn id="attr-ol-type-state-upper-alpha">upper&#45;alpha</dfn>      </td><td>Uppercase latin alphabet      </td><td class="eg"><samp>A&#46;</samp> </td><td class="eg"><samp>B&#46;</samp> </td><td class="eg"><samp>C&#46;</samp> </td><td class="eg">&#46;&#46;&#46; </td><td class="eg"><samp>EWU&#46;</samp> </td><td class="eg"><samp>EWV&#46;</samp> </td><td class="eg"><samp>EWW&#46;</samp> </td><td class="eg">&#46;&#46;&#46;     </td></tr><tr><td><dfn data-dfn-for="ol/type" id="attr-ol-type-keyword-lower-roman" data-dfn-type="attr-value"><code>i</code></dfn> &#40;U&#43;0069&#41;       </td><td><dfn id="attr-ol-type-state-lower-roman">lower&#45;roman</dfn>      </td><td>Lowercase roman numerals      </td><td class="eg"><samp>i&#46;</samp> </td><td class="eg"><samp>ii&#46;</samp> </td><td class="eg"><samp>iii&#46;</samp> </td><td class="eg">&#46;&#46;&#46; </td><td class="eg"><samp>mmmcmxcix&#46;</samp> </td><td class="eg"><samp>i̅v̅&#46;</samp> </td><td class="eg"><samp>i̅v̅i&#46;</samp> </td><td class="eg">&#46;&#46;&#46;     </td></tr><tr><td><dfn data-dfn-for="ol/type" id="attr-ol-type-keyword-upper-roman" data-dfn-type="attr-value"><code>I</code></dfn> &#40;U&#43;0049&#41;      </td><td><dfn id="attr-ol-type-state-upper-roman">upper&#45;roman</dfn>      </td><td>Uppercase roman numerals      </td><td class="eg"><samp>I&#46;</samp> </td><td class="eg"><samp>II&#46;</samp> </td><td class="eg"><samp>III&#46;</samp> </td><td class="eg">&#46;&#46;&#46; </td><td class="eg"><samp>MMMCMXCIX&#46;</samp> </td><td class="eg"><samp>I̅V̅&#46;</samp> </td><td class="eg"><samp>I̅V̅I&#46;</samp> </td><td class="eg">&#46;&#46;&#46;   </td></tr></tbody></table>

User agents should render the items of the list in a manner consistent with the state of the <a id="the&#45;ol&#45;element&#58;attr&#45;ol&#45;type&#45;4"></a><code>[type](#attr-ol-type)</code> attribute of the <a id="the&#45;ol&#45;element&#58;the&#45;ol&#45;element&#45;6"></a><code>[ol](#the-ol-element)</code> element&#46; Numbers less than or equal to zero should always use the decimal system regardless of the <a id="the&#45;ol&#45;element&#58;attr&#45;ol&#45;type&#45;5"></a><code>[type](#attr-ol-type)</code> attribute&#46;

<strong>Note&#58;</strong>

For CSS user agents&#44; a mapping for this attribute to the <a id="the&#45;ol&#45;element&#58;&#39;list&#45;style&#45;type&#39;"></a>[&#39;list&#45;style&#45;type&#39;](https://drafts.csswg.org/css-lists/#propdef-list-style-type) CSS property is given [in the Rendering section](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#decohints) &#40;the mapping is straightforward&#58; the states above have the same names as their corresponding CSS values&#41;&#46;

<strong>Note&#58;</strong>

It is possible to redefine the default CSS list styles used to implement this attribute in CSS user agents&#59; doing so will affect how list items are rendered&#46;

<strong>Note&#58;</strong>

Due to <a id="the&#45;ol&#45;element&#58;xattr&#45;reflectdefault&#45;2"></a>[&#91;ReflectDefault&#93;](html-language-foundations--877d6146487f--b55e73c31978.md#xattr-reflectdefault) the <a id="the&#45;ol&#45;element&#58;dom&#45;ol&#45;start"></a><code>[start](#dom-ol-start)</code> IDL attribute does not necessarily match the list&#39;s <a id="the&#45;ol&#45;element&#58;concept&#45;ol&#45;start&#45;3"></a>[starting value](#concept-ol-start)&#44; in cases where the <a id="the&#45;ol&#45;element&#58;attr&#45;ol&#45;start&#45;4"></a><code>[start](#attr-ol-start)</code> content attribute is omitted and the <a id="the&#45;ol&#45;element&#58;attr&#45;ol&#45;reversed&#45;4"></a><code>[reversed](#attr-ol-reversed)</code> content attribute is specified&#46;

<strong>Example&#58;</strong>

The following markup shows a list where the order matters&#44; and where the <a id="the&#45;ol&#45;element&#58;the&#45;ol&#45;element&#45;7"></a><code>[ol](#the-ol-element)</code> element is therefore appropriate&#46; Compare this list to the equivalent list in the <a id="the&#45;ol&#45;element&#58;the&#45;ul&#45;element"></a><code>[ul](#the-ul-element)</code> section to see an example of the same items using the <a id="the&#45;ol&#45;element&#58;the&#45;ul&#45;element&#45;2"></a><code>[ul](#the-ul-element)</code> element&#46;

``` text
<p>I have lived in the following countries (given in the order of when
I first lived there):</p>
<ol>
 <li>Switzerland
 <li>United Kingdom
 <li>United States
 <li>Norway
</ol>
```

Note how changing the order of the list changes the meaning of the document&#46; In the following example&#44; changing the relative order of the first two items has changed the birthplace of the author&#58;

``` text
<p>I have lived in the following countries (given in the order of when
I first lived there):</p>
<ol>
 <li>United Kingdom
 <li>Switzerland
 <li>United States
 <li>Norway
</ol>
```

#### <a id="the&#45;ul&#45;element"></a>4&#46;4&#46;6 The <strong><code>ul</code></strong> element[](#the-ul-element)

<strong>✔</strong>MDN

[Element&#47;ul](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ul)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong>✔</strong>MDN

[HTMLUListElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLUListElement)

Support in all current engines&#46;

Firefox1&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

<strong><a id="the&#45;ul&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;ul&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

If the element&#39;s children include at least one <a id="the&#45;ul&#45;element&#58;the&#45;li&#45;element"></a><code>[li](#the-li-element)</code> element&#58; <a id="the&#45;ul&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;ul&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;ul&#45;element&#58;flow&#45;content&#45;2&#45;2"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2) is expected&#46;

<strong><a id="the&#45;ul&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

Zero or more <a id="the&#45;ul&#45;element&#58;the&#45;li&#45;element&#45;2"></a><code>[li](#the-li-element)</code> and <a id="the&#45;ul&#45;element&#58;script&#45;supporting&#45;elements&#45;2"></a>[script&#45;supporting](html-language-foundations--877d6146487f--b55e73c31978.md#script-supporting-elements-2) elements&#46;

<strong><a id="the&#45;ul&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;ul&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;ul&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;ul&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-ul)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-ul)&#46;

<strong><a id="the&#45;ul&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;ul&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;ul&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

<a id="htmlulistelement"></a><a id="the&#45;ul&#45;element&#58;htmlelement"></a><a id="the&#45;ul&#45;element&#58;htmlconstructor"></a>

``` text
[Exposed=Window]
interface HTMLUListElement : HTMLElement {
  [HTMLConstructor] constructor();

  // also has obsolete members
};
```

The <a id="the&#45;ul&#45;element&#58;the&#45;ul&#45;element"></a><code>[ul](#the-ul-element)</code> element <a id="the&#45;ul&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a list of items&#44; where the order of the items is not important — that is&#44; where changing the order would not materially change the meaning of the document&#46;

The items of the list are the <a id="the&#45;ul&#45;element&#58;the&#45;li&#45;element&#45;3"></a><code>[li](#the-li-element)</code> element child nodes of the <a id="the&#45;ul&#45;element&#58;the&#45;ul&#45;element&#45;2"></a><code>[ul](#the-ul-element)</code> element&#46;

<strong>Example&#58;</strong>

The following markup shows a list where the order does not matter&#44; and where the <a id="the&#45;ul&#45;element&#58;the&#45;ul&#45;element&#45;3"></a><code>[ul](#the-ul-element)</code> element is therefore appropriate&#46; Compare this list to the equivalent list in the <a id="the&#45;ul&#45;element&#58;the&#45;ol&#45;element"></a><code>[ol](#the-ol-element)</code> section to see an example of the same items using the <a id="the&#45;ul&#45;element&#58;the&#45;ol&#45;element&#45;2"></a><code>[ol](#the-ol-element)</code> element&#46;

``` text
<p>I have lived in the following countries:</p>
<ul>
 <li>Norway
 <li>Switzerland
 <li>United Kingdom
 <li>United States
</ul>
```

Note that changing the order of the list does not change the meaning of the document&#46; The items in the snippet above are given in alphabetical order&#44; but in the snippet below they are given in order of the size of their current account balance in 2007&#44; without changing the meaning of the document whatsoever&#58;

``` text
<p>I have lived in the following countries:</p>
<ul>
 <li>Switzerland
 <li>Norway
 <li>United Kingdom
 <li>United States
</ul>
```

#### <a id="the&#45;menu&#45;element"></a>4&#46;4&#46;7 The <a id="menus"></a><strong><code>menu</code></strong> element[](#the-menu-element)

<strong>✔</strong>MDN

[Element&#47;menu](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/menu)

Support in all current engines&#46;

Firefox1&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer6&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

<strong>✔</strong>MDN

[HTMLMenuElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMenuElement)

Support in all current engines&#46;

Firefox1&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer6&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

<strong><a id="the&#45;menu&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;menu&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

If the element&#39;s children include at least one <a id="the&#45;menu&#45;element&#58;the&#45;li&#45;element"></a><code>[li](#the-li-element)</code> element&#58; <a id="the&#45;menu&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;menu&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;menu&#45;element&#58;flow&#45;content&#45;2&#45;2"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2) is expected&#46;

<strong><a id="the&#45;menu&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

Zero or more <a id="the&#45;menu&#45;element&#58;the&#45;li&#45;element&#45;2"></a><code>[li](#the-li-element)</code> and <a id="the&#45;menu&#45;element&#58;script&#45;supporting&#45;elements&#45;2"></a>[script&#45;supporting](html-language-foundations--877d6146487f--b55e73c31978.md#script-supporting-elements-2) elements&#46;

<strong><a id="the&#45;menu&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;menu&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;menu&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;menu&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-menu)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-menu)&#46;

<strong><a id="the&#45;menu&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;menu&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;menu&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

<a id="htmlmenuelement"></a><a id="the&#45;menu&#45;element&#58;htmlelement"></a><a id="the&#45;menu&#45;element&#58;htmlconstructor"></a>

``` text
[Exposed=Window]
interface HTMLMenuElement : HTMLElement {
  [HTMLConstructor] constructor();

  // also has obsolete members
};
```

The <a id="the&#45;menu&#45;element&#58;the&#45;menu&#45;element"></a><code>[menu](#the-menu-element)</code> element <a id="the&#45;menu&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a toolbar consisting of its contents&#44; in the form of an unordered list of items &#40;represented by <a id="the&#45;menu&#45;element&#58;the&#45;li&#45;element&#45;3"></a><code>[li](#the-li-element)</code> elements&#41;&#44; each of which represents a command that the user can perform or activate&#46;

<strong>Note&#58;</strong>

The <a id="the&#45;menu&#45;element&#58;the&#45;menu&#45;element&#45;2"></a><code>[menu](#the-menu-element)</code> element is simply a semantic alternative to <a id="the&#45;menu&#45;element&#58;the&#45;ul&#45;element"></a><code>[ul](#the-ul-element)</code> to express an unordered list of commands &#40;a &#34;toolbar&#34;&#41;&#46;

<strong>Example&#58;</strong>

In this example&#44; a text&#45;editing application uses a <a id="the&#45;menu&#45;element&#58;the&#45;menu&#45;element&#45;3"></a><code>[menu](#the-menu-element)</code> element to provide a series of editing commands&#58;

``` text
<menu>
 <li><button onclick="copy()"><img src="copy.svg" alt="Copy"></button></li>
 <li><button onclick="cut()"><img src="cut.svg" alt="Cut"></button></li>
 <li><button onclick="paste()"><img src="paste.svg" alt="Paste"></button></li>
</menu>
```

Note that the styling to make this look like a conventional toolbar menu is up to the application&#46;

#### <a id="the&#45;li&#45;element"></a>4&#46;4&#46;8 The <strong><code>li</code></strong> element[](#the-li-element)

<strong>✔</strong>MDN

[Element&#47;li](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/li)

Support in all current engines&#46;

Firefox1&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

<strong>✔</strong>MDN

[HTMLLIElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLLIElement)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

<strong><a id="the&#45;li&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

None&#46;

<strong><a id="the&#45;li&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Inside <a id="the&#45;li&#45;element&#58;the&#45;ol&#45;element"></a><code>[ol](#the-ol-element)</code> elements&#46;

Inside <a id="the&#45;li&#45;element&#58;the&#45;ul&#45;element"></a><code>[ul](#the-ul-element)</code> elements&#46;

Inside <a id="the&#45;li&#45;element&#58;the&#45;menu&#45;element"></a><code>[menu](#the-menu-element)</code> elements&#46;

<strong><a id="the&#45;li&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;li&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<strong><a id="the&#45;li&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

An <a id="the&#45;li&#45;element&#58;the&#45;li&#45;element"></a><code>[li](#the-li-element)</code> element&#39;s <a id="the&#45;li&#45;element&#58;syntax&#45;end&#45;tag"></a>[end tag](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#syntax-end-tag) can be omitted if the <a id="the&#45;li&#45;element&#58;the&#45;li&#45;element&#45;2"></a><code>[li](#the-li-element)</code> element is immediately followed by another <a id="the&#45;li&#45;element&#58;the&#45;li&#45;element&#45;3"></a><code>[li](#the-li-element)</code> element or if there is no more content in the parent element&#46;

<strong><a id="the&#45;li&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;li&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

If the element is not a child of an <a id="the&#45;li&#45;element&#58;the&#45;ul&#45;element&#45;2"></a><code>[ul](#the-ul-element)</code> or <a id="the&#45;li&#45;element&#58;the&#45;menu&#45;element&#45;2"></a><code>[menu](#the-menu-element)</code> element&#58; <a id="the&#45;li&#45;element&#58;attr&#45;li&#45;value"></a><code>[value](#attr-li-value)</code> — <a id="the&#45;li&#45;element&#58;ordinal&#45;value"></a>[Ordinal value](#ordinal-value) of the list item

<strong><a id="the&#45;li&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-li)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-li)&#46;

<strong><a id="the&#45;li&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;li&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default) with attributes <a id="the&#45;li&#45;element&#58;attr&#45;li&#45;value&#45;2"></a><code>[value](#attr-li-value)</code>&#46;

<strong><a id="the&#45;li&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

<a id="htmllielement"></a><a id="the&#45;li&#45;element&#58;htmlelement"></a><a id="the&#45;li&#45;element&#58;htmlconstructor"></a><a id="the&#45;li&#45;element&#58;cereactions"></a><a id="the&#45;li&#45;element&#58;xattr&#45;reflect"></a><a id="dom&#45;li&#45;value"></a>

``` text
[Exposed=Window]
interface HTMLLIElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions, Reflect] attribute long value;

  // also has obsolete members
};
```

The <a id="the&#45;li&#45;element&#58;the&#45;li&#45;element&#45;4"></a><code>[li](#the-li-element)</code> element <a id="the&#45;li&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a list item&#46; If its parent element is an <a id="the&#45;li&#45;element&#58;the&#45;ol&#45;element&#45;2"></a><code>[ol](#the-ol-element)</code>&#44; <a id="the&#45;li&#45;element&#58;the&#45;ul&#45;element&#45;3"></a><code>[ul](#the-ul-element)</code>&#44; or <a id="the&#45;li&#45;element&#58;the&#45;menu&#45;element&#45;3"></a><code>[menu](#the-menu-element)</code> element&#44; then the element is an item of the parent element&#39;s list&#44; as defined for those elements&#46; Otherwise&#44; the list item has no defined list&#45;related relationship to any other <a id="the&#45;li&#45;element&#58;the&#45;li&#45;element&#45;5"></a><code>[li](#the-li-element)</code> element&#46;

The <a id="attr&#45;li&#45;value"></a><strong><code>value</code></strong> attribute&#44; if present&#44; must be a <a id="the&#45;li&#45;element&#58;valid&#45;integer"></a>[valid integer](html-language-foundations--877d6146487f--b55e73c31978.md#valid-integer)&#46; It is used to determine the <a id="the&#45;li&#45;element&#58;ordinal&#45;value&#45;2"></a>[ordinal value](#ordinal-value) of the list item&#44; when the <a id="the&#45;li&#45;element&#58;the&#45;li&#45;element&#45;6"></a><code>[li](#the-li-element)</code>&#39;s <a id="the&#45;li&#45;element&#58;list&#45;owner"></a>[list owner](#list-owner) is an <a id="the&#45;li&#45;element&#58;the&#45;ol&#45;element&#45;3"></a><code>[ol](#the-ol-element)</code> element&#46;

------------------------------------------------------------------------

Any element whose <a id="the&#45;li&#45;element&#58;computed&#45;value"></a>[computed value](https://drafts.csswg.org/css-cascade/#computed-value) of <a id="the&#45;li&#45;element&#58;&#39;display&#39;"></a>[&#39;display&#39;](https://drafts.csswg.org/css2/#display-prop) is &#39;list&#45;item&#39; has a <a id="list&#45;owner"></a><strong>list owner</strong>&#44; which is determined as follows&#58;

1.  If the element is not <a id="the&#45;li&#45;element&#58;being&#45;rendered"></a>[being rendered](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#being-rendered)&#44; return null&#59; the element has no <a id="the&#45;li&#45;element&#58;list&#45;owner&#45;2"></a>[list owner](#list-owner)&#46;

2.  Let <var>ancestor</var> be the element&#39;s parent&#46;

3.  If the element has an <a id="the&#45;li&#45;element&#58;the&#45;ol&#45;element&#45;4"></a><code>[ol](#the-ol-element)</code>&#44; <a id="the&#45;li&#45;element&#58;the&#45;ul&#45;element&#45;4"></a><code>[ul](#the-ul-element)</code>&#44; or <a id="the&#45;li&#45;element&#58;the&#45;menu&#45;element&#45;4"></a><code>[menu](#the-menu-element)</code> ancestor&#44; set <var>ancestor</var> to the closest such ancestor element&#46;

4.  Return the closest inclusive ancestor of <var>ancestor</var> that produces a <a id="the&#45;li&#45;element&#58;css&#45;box"></a>[CSS box](https://drafts.csswg.org/css-display/#css-box)&#46;

    <strong>Note&#58;</strong>

    Such an element will always exist&#44; as at the very least the <a id="the&#45;li&#45;element&#58;document&#45;element"></a>[document element](https://dom.spec.whatwg.org/#document-element) will always produce a <a id="the&#45;li&#45;element&#58;css&#45;box&#45;2"></a>[CSS box](https://drafts.csswg.org/css-display/#css-box)&#46;

To determine the <a id="ordinal&#45;value"></a><strong>ordinal value</strong> of each element owned by a given <a id="the&#45;li&#45;element&#58;list&#45;owner&#45;3"></a>[list owner](#list-owner) <var>owner</var>&#44; perform the following steps&#58;

1.  Let <var>i</var> be 1&#46;

2.  If <var>owner</var> is an <a id="the&#45;li&#45;element&#58;the&#45;ol&#45;element&#45;5"></a><code>[ol](#the-ol-element)</code> element&#44; let <var>numbering</var> be <var>owner</var>&#39;s <a id="the&#45;li&#45;element&#58;concept&#45;ol&#45;start"></a>[starting value](#concept-ol-start)&#46; Otherwise&#44; let <var>numbering</var> be 1&#46;

3.  <em>Loop</em>&#58; If <var>i</var> is greater than the number of <a id="the&#45;li&#45;element&#58;list&#45;owner&#45;4"></a>[list items that <var>owner</var> owns](#list-owner)&#44; then return&#59; all of <var>owner</var>&#39;s <a id="the&#45;li&#45;element&#58;list&#45;owner&#45;5"></a>[owned list items](#list-owner) have been assigned <a id="the&#45;li&#45;element&#58;ordinal&#45;value&#45;3"></a>[ordinal values](#ordinal-value)&#46;

4.  Let <var>item</var> be the <var>i</var>th of <var>owner</var>&#39;s <a id="the&#45;li&#45;element&#58;list&#45;owner&#45;6"></a>[owned list items](#list-owner)&#44; in <a id="the&#45;li&#45;element&#58;tree&#45;order"></a>[tree order](https://dom.spec.whatwg.org/#concept-tree-order)&#46;

5.  If <var>item</var> is an <a id="the&#45;li&#45;element&#58;the&#45;li&#45;element&#45;7"></a><code>[li](#the-li-element)</code> element that has a <a id="the&#45;li&#45;element&#58;attr&#45;li&#45;value&#45;3"></a><code>[value](#attr-li-value)</code> attribute&#58;

    1.  Let <var>parsed</var> be the result of <a id="the&#45;li&#45;element&#58;rules&#45;for&#45;parsing&#45;integers"></a>[parsing the value of the attribute as an integer](html-language-foundations--877d6146487f--b55e73c31978.md#rules-for-parsing-integers)&#46;

    2.  If <var>parsed</var> is not an error&#44; then set <var>numbering</var> to <var>parsed</var>&#46;

6.  The <a id="the&#45;li&#45;element&#58;ordinal&#45;value&#45;4"></a>[ordinal value](#ordinal-value) of <var>item</var> is <var>numbering</var>&#46;

7.  If <var>owner</var> is an <a id="the&#45;li&#45;element&#58;the&#45;ol&#45;element&#45;6"></a><code>[ol](#the-ol-element)</code> element&#44; and <var>owner</var> has a <a id="the&#45;li&#45;element&#58;attr&#45;ol&#45;reversed"></a><code>[reversed](#attr-ol-reversed)</code> attribute&#44; decrement <var>numbering</var> by 1&#59; otherwise&#44; increment <var>numbering</var> by 1&#46;

8.  Increment <var>i</var> by 1&#46;

9.  Go to the step labeled <em>loop</em>&#46;

------------------------------------------------------------------------

<strong>Example&#58;</strong>

The element&#39;s <a id="the&#45;li&#45;element&#58;dom&#45;li&#45;value"></a><code>[value](#dom-li-value)</code> IDL attribute does not directly correspond to its <a id="the&#45;li&#45;element&#58;ordinal&#45;value&#45;5"></a>[ordinal value](#ordinal-value)&#59; it simply <a id="the&#45;li&#45;element&#58;reflect"></a>[reflects](html-language-foundations--877d6146487f--b55e73c31978.md#reflect) the content attribute&#46; For example&#44; given this list&#58;

``` text
<ol>
 <li>Item 1
 <li value="3">Item 3
 <li>Item 4
</ol>
```

The <a id="the&#45;li&#45;element&#58;ordinal&#45;value&#45;6"></a>[ordinal values](#ordinal-value) are 1&#44; 3&#44; and 4&#44; whereas the <a id="the&#45;li&#45;element&#58;dom&#45;li&#45;value&#45;2"></a><code>[value](#dom-li-value)</code> IDL attributes return 0&#44; 3&#44; 0 on getting&#46;

<strong>Example&#58;</strong>

The following example&#44; the top ten movies are listed &#40;in reverse order&#41;&#46; Note the way the list is given a title by using a <a id="the&#45;li&#45;element&#58;the&#45;figure&#45;element"></a><code>[figure](#the-figure-element)</code> element and its <a id="the&#45;li&#45;element&#58;the&#45;figcaption&#45;element"></a><code>[figcaption](#the-figcaption-element)</code> element&#46;

``` text
<figure>
 <figcaption>The top 10 movies of all time</figcaption>
 <ol>
  <li value="10"><cite>Josie and the Pussycats</cite>, 2001</li>
  <li value="9"><cite lang="sh">Црна мачка, бели мачор</cite>, 1998</li>
  <li value="8"><cite>A Bug's Life</cite>, 1998</li>
  <li value="7"><cite>Toy Story</cite>, 1995</li>
  <li value="6"><cite>Monsters, Inc</cite>, 2001</li>
  <li value="5"><cite>Cars</cite>, 2006</li>
  <li value="4"><cite>Toy Story 2</cite>, 1999</li>
  <li value="3"><cite>Finding Nemo</cite>, 2003</li>
  <li value="2"><cite>The Incredibles</cite>, 2004</li>
  <li value="1"><cite>Ratatouille</cite>, 2007</li>
 </ol>
</figure>
```

The markup could also be written as follows&#44; using the <a id="the&#45;li&#45;element&#58;attr&#45;ol&#45;reversed&#45;2"></a><code>[reversed](#attr-ol-reversed)</code> attribute on the <a id="the&#45;li&#45;element&#58;the&#45;ol&#45;element&#45;7"></a><code>[ol](#the-ol-element)</code> element&#58;

``` text
<figure>
 <figcaption>The top 10 movies of all time</figcaption>
 <ol reversed>
  <li><cite>Josie and the Pussycats</cite>, 2001</li>
  <li><cite lang="sh">Црна мачка, бели мачор</cite>, 1998</li>
  <li><cite>A Bug's Life</cite>, 1998</li>
  <li><cite>Toy Story</cite>, 1995</li>
  <li><cite>Monsters, Inc</cite>, 2001</li>
  <li><cite>Cars</cite>, 2006</li>
  <li><cite>Toy Story 2</cite>, 1999</li>
  <li><cite>Finding Nemo</cite>, 2003</li>
  <li><cite>The Incredibles</cite>, 2004</li>
  <li><cite>Ratatouille</cite>, 2007</li>
 </ol>
</figure>
```

<strong>Note&#58;</strong>

While it is conforming to include heading elements &#40;e&#46;g&#46; <a id="the&#45;li&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements"></a><code>[h1](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#41; inside <a id="the&#45;li&#45;element&#58;the&#45;li&#45;element&#45;8"></a><code>[li](#the-li-element)</code> elements&#44; it likely does not convey the semantics that the author intended&#46; A heading starts a new section&#44; so a heading in a list implicitly splits the list into spanning multiple sections&#46;

#### <a id="the&#45;dl&#45;element"></a>4&#46;4&#46;9 The <strong><code>dl</code></strong> element[](#the-dl-element)

<strong>✔</strong>MDN

[Element&#47;dl](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dl)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong>✔</strong>MDN

[HTMLDListElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDListElement)

Support in all current engines&#46;

Firefox1&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

<strong><a id="the&#45;dl&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;dl&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

If the element&#39;s children include at least one name&#45;value group&#58; <a id="the&#45;dl&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;dl&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;dl&#45;element&#58;flow&#45;content&#45;2&#45;2"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2) is expected&#46;

<strong><a id="the&#45;dl&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

Either&#58; Zero or more groups each consisting of one or more <a id="the&#45;dl&#45;element&#58;the&#45;dt&#45;element"></a><code>[dt](#the-dt-element)</code> elements followed by one or more <a id="the&#45;dl&#45;element&#58;the&#45;dd&#45;element"></a><code>[dd](#the-dd-element)</code> elements&#44; optionally intermixed with <a id="the&#45;dl&#45;element&#58;script&#45;supporting&#45;elements&#45;2"></a>[script&#45;supporting elements](html-language-foundations--877d6146487f--b55e73c31978.md#script-supporting-elements-2)&#46;

Or&#58; One or more <a id="the&#45;dl&#45;element&#58;the&#45;div&#45;element"></a><code>[div](#the-div-element)</code> elements&#44; optionally intermixed with <a id="the&#45;dl&#45;element&#58;script&#45;supporting&#45;elements&#45;2&#45;2"></a>[script&#45;supporting elements](html-language-foundations--877d6146487f--b55e73c31978.md#script-supporting-elements-2)&#46;

<strong><a id="the&#45;dl&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;dl&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;dl&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;dl&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-dl)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-dl)&#46;

<strong><a id="the&#45;dl&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;dl&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;dl&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

<a id="htmldlistelement"></a><a id="the&#45;dl&#45;element&#58;htmlelement"></a><a id="the&#45;dl&#45;element&#58;htmlconstructor"></a>

``` text
[Exposed=Window]
interface HTMLDListElement : HTMLElement {
  [HTMLConstructor] constructor();

  // also has obsolete members
};
```

The <a id="the&#45;dl&#45;element&#58;the&#45;dl&#45;element"></a><code>[dl](#the-dl-element)</code> element <a id="the&#45;dl&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) an association list consisting of zero or more name&#45;value groups &#40;a description list&#41;&#46; A name&#45;value group consists of one or more names &#40;<a id="the&#45;dl&#45;element&#58;the&#45;dt&#45;element&#45;2"></a><code>[dt](#the-dt-element)</code> elements&#44; possibly as children of a <a id="the&#45;dl&#45;element&#58;the&#45;div&#45;element&#45;2"></a><code>[div](#the-div-element)</code> element child&#41; followed by one or more values &#40;<a id="the&#45;dl&#45;element&#58;the&#45;dd&#45;element&#45;2"></a><code>[dd](#the-dd-element)</code> elements&#44; possibly as children of a <a id="the&#45;dl&#45;element&#58;the&#45;div&#45;element&#45;3"></a><code>[div](#the-div-element)</code> element child&#41;&#44; ignoring any nodes other than <a id="the&#45;dl&#45;element&#58;the&#45;dt&#45;element&#45;3"></a><code>[dt](#the-dt-element)</code> and <a id="the&#45;dl&#45;element&#58;the&#45;dd&#45;element&#45;3"></a><code>[dd](#the-dd-element)</code> element children&#44; and <a id="the&#45;dl&#45;element&#58;the&#45;dt&#45;element&#45;4"></a><code>[dt](#the-dt-element)</code> and <a id="the&#45;dl&#45;element&#58;the&#45;dd&#45;element&#45;4"></a><code>[dd](#the-dd-element)</code> elements that are children of <a id="the&#45;dl&#45;element&#58;the&#45;div&#45;element&#45;4"></a><code>[div](#the-div-element)</code> element children&#46; Within a single <a id="the&#45;dl&#45;element&#58;the&#45;dl&#45;element&#45;2"></a><code>[dl](#the-dl-element)</code> element&#44; there should not be more than one <a id="the&#45;dl&#45;element&#58;the&#45;dt&#45;element&#45;5"></a><code>[dt](#the-dt-element)</code> element for each name&#46;

Name&#45;value groups may be terms and definitions&#44; metadata topics and values&#44; questions and answers&#44; or any other groups of name&#45;value data&#46;

The values within a group are alternatives&#59; multiple paragraphs forming part of the same value must all be given within the same <a id="the&#45;dl&#45;element&#58;the&#45;dd&#45;element&#45;5"></a><code>[dd](#the-dd-element)</code> element&#46;

The order of the list of groups&#44; and of the names and values within each group&#44; may be significant&#46;

In order to annotate groups with <a id="the&#45;dl&#45;element&#58;microdata"></a>[microdata](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#microdata) attributes&#44; or other <a id="the&#45;dl&#45;element&#58;global&#45;attributes&#45;2"></a>[global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes) that apply to whole groups&#44; or just for styling purposes&#44; each group in a <a id="the&#45;dl&#45;element&#58;the&#45;dl&#45;element&#45;3"></a><code>[dl](#the-dl-element)</code> element can be wrapped in a <a id="the&#45;dl&#45;element&#58;the&#45;div&#45;element&#45;5"></a><code>[div](#the-div-element)</code> element&#46; This does not change the semantics of the <a id="the&#45;dl&#45;element&#58;the&#45;dl&#45;element&#45;4"></a><code>[dl](#the-dl-element)</code> element&#46;

The name&#45;value groups of a <a id="the&#45;dl&#45;element&#58;the&#45;dl&#45;element&#45;5"></a><code>[dl](#the-dl-element)</code> element <var>dl</var> are determined using the following algorithm&#46; A name&#45;value group has a name &#40;a list of <a id="the&#45;dl&#45;element&#58;the&#45;dt&#45;element&#45;6"></a><code>[dt](#the-dt-element)</code> elements&#44; initially empty&#41; and a value &#40;a list of <a id="the&#45;dl&#45;element&#58;the&#45;dd&#45;element&#45;6"></a><code>[dd](#the-dd-element)</code> elements&#44; initially empty&#41;&#46;

1.  Let <var>groups</var> be an empty list of name&#45;value groups&#46;

2.  Let <var>current</var> be a new name&#45;value group&#46;

3.  Let <var>seenDd</var> be false&#46;

4.  Let <var>child</var> be <var>dl</var>&#39;s <a id="the&#45;dl&#45;element&#58;first&#45;child"></a>[first child](https://dom.spec.whatwg.org/#concept-tree-first-child)&#46;

5.  Let <var>grandchild</var> be null&#46;

6.  While <var>child</var> is not null&#58;

    1.  If <var>child</var> is a <a id="the&#45;dl&#45;element&#58;the&#45;div&#45;element&#45;6"></a><code>[div](#the-div-element)</code> element&#58;

        1.  Let <var>grandchild</var> be <var>child</var>&#39;s <a id="the&#45;dl&#45;element&#58;first&#45;child&#45;2"></a>[first child](https://dom.spec.whatwg.org/#concept-tree-first-child)&#46;

        2.  While <var>grandchild</var> is not null&#58;

            1.  <a id="the&#45;dl&#45;element&#58;process&#45;dt&#45;or&#45;dd"></a>[Process <code>dt</code> or <code>dd</code>](#process-dt-or-dd) for <var>grandchild</var>&#46;

            2.  Set <var>grandchild</var> to <var>grandchild</var>&#39;s <a id="the&#45;dl&#45;element&#58;next&#45;sibling"></a>[next sibling](https://dom.spec.whatwg.org/#concept-tree-next-sibling)&#46;

    2.  Otherwise&#44; <a id="the&#45;dl&#45;element&#58;process&#45;dt&#45;or&#45;dd&#45;2"></a>[process <code>dt</code> or <code>dd</code>](#process-dt-or-dd) for <var>child</var>&#46;

    3.  Set <var>child</var> to <var>child</var>&#39;s <a id="the&#45;dl&#45;element&#58;next&#45;sibling&#45;2"></a>[next sibling](https://dom.spec.whatwg.org/#concept-tree-next-sibling)&#46;

7.  If <var>current</var> is not empty&#44; then append <var>current</var> to <var>groups</var>&#46;

8.  Return <var>groups</var>&#46;

To <a id="process&#45;dt&#45;or&#45;dd"></a><strong>process <code>dt</code> or <code>dd</code></strong> for a node <var>node</var> means to follow these steps&#58;

1.  Let <var>groups</var>&#44; <var>current</var>&#44; and <var>seenDd</var> be the same variables as those of the same name in the algorithm that invoked these steps&#46;

2.  If <var>node</var> is a <a id="the&#45;dl&#45;element&#58;the&#45;dt&#45;element&#45;7"></a><code>[dt](#the-dt-element)</code> element&#58;

    1.  If <var>seenDd</var> is true&#44; then append <var>current</var> to <var>groups</var>&#44; set <var>current</var> to a new name&#45;value group&#44; and set <var>seenDd</var> to false&#46;

    2.  Append <var>node</var> to <var>current</var>&#39;s name&#46;

3.  Otherwise&#44; if <var>node</var> is a <a id="the&#45;dl&#45;element&#58;the&#45;dd&#45;element&#45;7"></a><code>[dd](#the-dd-element)</code> element&#44; then append <var>node</var> to <var>current</var>&#39;s value and set <var>seenDd</var> to true&#46;

<strong>Note&#58;</strong>

When a name&#45;value group has an empty list as name or value&#44; it is often due to accidentally using <a id="the&#45;dl&#45;element&#58;the&#45;dd&#45;element&#45;8"></a><code>[dd](#the-dd-element)</code> elements in the place of <a id="the&#45;dl&#45;element&#58;the&#45;dt&#45;element&#45;8"></a><code>[dt](#the-dt-element)</code> elements and vice versa&#46; Conformance checkers can spot such mistakes and might be able to advise authors how to correctly use the markup&#46;

<strong>Example&#58;</strong>

In the following example&#44; one entry &#40;&#34;Authors&#34;&#41; is linked to two values &#40;&#34;John&#34; and &#34;Luke&#34;&#41;&#46;

``` text
<dl>
 <dt> Authors
 <dd> John
 <dd> Luke
 <dt> Editor
 <dd> Frank
</dl>
```

<strong>Example&#58;</strong>

In the following example&#44; one definition is linked to two terms&#46;

``` text
<dl>
 <dt lang="en-US"> <dfn>color</dfn> </dt>
 <dt lang="en-GB"> <dfn>colour</dfn> </dt>
 <dd> A sensation which (in humans) derives from the ability of
 the fine structure of the eye to distinguish three differently
 filtered analyses of a view. </dd>
</dl>
```

<strong>Example&#58;</strong>

The following example illustrates the use of the <a id="the&#45;dl&#45;element&#58;the&#45;dl&#45;element&#45;6"></a><code>[dl](#the-dl-element)</code> element to mark up metadata of sorts&#46; At the end of the example&#44; one group has two metadata labels &#40;&#34;Authors&#34; and &#34;Editors&#34;&#41; and two values &#40;&#34;Robert Rothman&#34; and &#34;Daniel Jackson&#34;&#41;&#46; This example also uses the <a id="the&#45;dl&#45;element&#58;the&#45;div&#45;element&#45;7"></a><code>[div](#the-div-element)</code> element around the groups of <a id="the&#45;dl&#45;element&#58;the&#45;dt&#45;element&#45;9"></a><code>[dt](#the-dt-element)</code> and <a id="the&#45;dl&#45;element&#58;the&#45;dd&#45;element&#45;9"></a><code>[dd](#the-dd-element)</code> element&#44; to aid with styling&#46;

``` text
<dl>
 <div>
  <dt> Last modified time </dt>
  <dd> 2004-12-23T23:33Z </dd>
 </div>
 <div>
  <dt> Recommended update interval </dt>
  <dd> 60s </dd>
 </div>
 <div>
  <dt> Authors </dt>
  <dt> Editors </dt>
  <dd> Robert Rothman </dd>
  <dd> Daniel Jackson </dd>
 </div>
</dl>
```

<strong>Example&#58;</strong>

The following example shows the <a id="the&#45;dl&#45;element&#58;the&#45;dl&#45;element&#45;7"></a><code>[dl](#the-dl-element)</code> element used to give a set of instructions&#46; The order of the instructions here is important &#40;in the other examples&#44; the order of the blocks was not important&#41;&#46;

``` text
<p>Determine the victory points as follows (use the
first matching case):</p>
<dl>
 <dt> If you have exactly five gold coins </dt>
 <dd> You get five victory points </dd>
 <dt> If you have one or more gold coins, and you have one or more silver coins </dt>
 <dd> You get two victory points </dd>
 <dt> If you have one or more silver coins </dt>
 <dd> You get one victory point </dd>
 <dt> Otherwise </dt>
 <dd> You get no victory points </dd>
</dl>
```

<strong>Example&#58;</strong>

The following snippet shows a <a id="the&#45;dl&#45;element&#58;the&#45;dl&#45;element&#45;8"></a><code>[dl](#the-dl-element)</code> element being used as a glossary&#46; Note the use of <a id="the&#45;dl&#45;element&#58;the&#45;dfn&#45;element"></a><code>[dfn](#the-dfn-element)</code> to indicate the word being defined&#46;

``` text
<dl>
 <dt><dfn>Apartment</dfn>, n.</dt>
 <dd>An execution context grouping one or more threads with one or
 more COM objects.</dd>
 <dt><dfn>Flat</dfn>, n.</dt>
 <dd>A deflated tire.</dd>
 <dt><dfn>Home</dfn>, n.</dt>
 <dd>The user's login directory.</dd>
</dl>
```

<strong>Example&#58;</strong>

This example uses <a id="the&#45;dl&#45;element&#58;microdata&#45;2"></a>[microdata](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#microdata) attributes in a <a id="the&#45;dl&#45;element&#58;the&#45;dl&#45;element&#45;9"></a><code>[dl](#the-dl-element)</code> element&#44; together with the <a id="the&#45;dl&#45;element&#58;the&#45;div&#45;element&#45;8"></a><code>[div](#the-div-element)</code> element&#44; to annotate the ice cream desserts at a French restaurant&#46;

``` text
<dl>
 <div itemscope itemtype="http://schema.org/Product">
  <dt itemprop="name">Café ou Chocolat Liégeois
  <dd itemprop="offers" itemscope itemtype="http://schema.org/Offer">
   <span itemprop="price">3.50</span>
   <data itemprop="priceCurrency" value="EUR">€</data>
  <dd itemprop="description">
   2 boules Café ou Chocolat, 1 boule Vanille, sauce café ou chocolat, chantilly
 </div>

 <div itemscope itemtype="http://schema.org/Product">
  <dt itemprop="name">Américaine
  <dd itemprop="offers" itemscope itemtype="http://schema.org/Offer">
   <span itemprop="price">3.50</span>
   <data itemprop="priceCurrency" value="EUR">€</data>
  <dd itemprop="description">
   1 boule Crème brûlée, 1 boule Vanille, 1 boule Caramel, chantilly
 </div>
</dl>
```

Without the <a id="the&#45;dl&#45;element&#58;the&#45;div&#45;element&#45;9"></a><code>[div](#the-div-element)</code> element the markup would need to use the <a id="the&#45;dl&#45;element&#58;attr&#45;itemref"></a><code>[itemref](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-itemref)</code> attribute to link the data in the <a id="the&#45;dl&#45;element&#58;the&#45;dd&#45;element&#45;10"></a><code>[dd](#the-dd-element)</code> elements with the item&#44; as follows&#46;

``` text
<dl>
 <dt itemscope itemtype="http://schema.org/Product" itemref="1-offer 1-description">
  <span itemprop="name">Café ou Chocolat Liégeois</span>
 <dd id="1-offer" itemprop="offers" itemscope itemtype="http://schema.org/Offer">
  <span itemprop="price">3.50</span>
  <data itemprop="priceCurrency" value="EUR">€</data>
 <dd id="1-description" itemprop="description">
  2 boules Café ou Chocolat, 1 boule Vanille, sauce café ou chocolat, chantilly

 <dt itemscope itemtype="http://schema.org/Product" itemref="2-offer 2-description">
  <span itemprop="name">Américaine</span>
 <dd id="2-offer" itemprop="offers" itemscope itemtype="http://schema.org/Offer">
  <span itemprop="price">3.50</span>
  <data itemprop="priceCurrency" value="EUR">€</data>
 <dd id="2-description" itemprop="description">
  1 boule Crème brûlée, 1 boule Vanille, 1 boule Caramel, chantilly
</dl>
```

<strong>Note&#58;</strong>

The <a id="the&#45;dl&#45;element&#58;the&#45;dl&#45;element&#45;10"></a><code>[dl](#the-dl-element)</code> element is inappropriate for marking up dialogue&#46; See some [examples of how to mark up dialogue](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#conversations)&#46;

#### <a id="the&#45;dt&#45;element"></a>4&#46;4&#46;10 The <strong><code>dt</code></strong> element[](#the-dt-element)

<strong>✔</strong>MDN

[Element&#47;dt](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dt)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;dt&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

None&#46;

<strong><a id="the&#45;dt&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Before <a id="the&#45;dt&#45;element&#58;the&#45;dd&#45;element"></a><code>[dd](#the-dd-element)</code> or <a id="the&#45;dt&#45;element&#58;the&#45;dt&#45;element"></a><code>[dt](#the-dt-element)</code> elements inside <a id="the&#45;dt&#45;element&#58;the&#45;dl&#45;element"></a><code>[dl](#the-dl-element)</code> elements&#46;

Before <a id="the&#45;dt&#45;element&#58;the&#45;dd&#45;element&#45;2"></a><code>[dd](#the-dd-element)</code> or <a id="the&#45;dt&#45;element&#58;the&#45;dt&#45;element&#45;2"></a><code>[dt](#the-dt-element)</code> elements inside <a id="the&#45;dt&#45;element&#58;the&#45;div&#45;element"></a><code>[div](#the-div-element)</code> elements that are children of a <a id="the&#45;dt&#45;element&#58;the&#45;dl&#45;element&#45;2"></a><code>[dl](#the-dl-element)</code> element&#46;

<strong><a id="the&#45;dt&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;dt&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#44; but with no <a id="the&#45;dt&#45;element&#58;the&#45;header&#45;element"></a><code>[header](#the-header-element)</code>&#44; <a id="the&#45;dt&#45;element&#58;the&#45;footer&#45;element"></a><code>[footer](#the-footer-element)</code>&#44; <a id="the&#45;dt&#45;element&#58;sectioning&#45;content&#45;2"></a>[sectioning content](html-language-foundations--877d6146487f--b55e73c31978.md#sectioning-content-2)&#44; or <a id="the&#45;dt&#45;element&#58;heading&#45;content&#45;2"></a>[heading content](html-language-foundations--877d6146487f--b55e73c31978.md#heading-content-2) descendants&#46;

<strong><a id="the&#45;dt&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

A <a id="the&#45;dt&#45;element&#58;the&#45;dt&#45;element&#45;3"></a><code>[dt](#the-dt-element)</code> element&#39;s <a id="the&#45;dt&#45;element&#58;syntax&#45;end&#45;tag"></a>[end tag](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#syntax-end-tag) can be omitted if the <a id="the&#45;dt&#45;element&#58;the&#45;dt&#45;element&#45;4"></a><code>[dt](#the-dt-element)</code> element is immediately followed by another <a id="the&#45;dt&#45;element&#58;the&#45;dt&#45;element&#45;5"></a><code>[dt](#the-dt-element)</code> element or a <a id="the&#45;dt&#45;element&#58;the&#45;dd&#45;element&#45;3"></a><code>[dd](#the-dd-element)</code> element&#46;

<strong><a id="the&#45;dt&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;dt&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;dt&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-dt)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-dt)&#46;

<strong><a id="the&#45;dt&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;dt&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;dt&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;dt&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;dt&#45;element&#58;the&#45;dt&#45;element&#45;6"></a><code>[dt](#the-dt-element)</code> element <a id="the&#45;dt&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) the term&#44; or name&#44; part of a term&#45;description group in a description list &#40;<a id="the&#45;dt&#45;element&#58;the&#45;dl&#45;element&#45;3"></a><code>[dl](#the-dl-element)</code> element&#41;&#46;

<strong>Note&#58;</strong>

The <a id="the&#45;dt&#45;element&#58;the&#45;dt&#45;element&#45;7"></a><code>[dt](#the-dt-element)</code> element itself&#44; when used in a <a id="the&#45;dt&#45;element&#58;the&#45;dl&#45;element&#45;4"></a><code>[dl](#the-dl-element)</code> element&#44; does not indicate that its contents are a term being defined&#44; but this can be indicated using the <a id="the&#45;dt&#45;element&#58;the&#45;dfn&#45;element"></a><code>[dfn](#the-dfn-element)</code> element&#46;

<strong>Example&#58;</strong>

This example shows a list of frequently asked questions &#40;a FAQ&#41; marked up using the <a id="the&#45;dt&#45;element&#58;the&#45;dt&#45;element&#45;8"></a><code>[dt](#the-dt-element)</code> element for questions and the <a id="the&#45;dt&#45;element&#58;the&#45;dd&#45;element&#45;4"></a><code>[dd](#the-dd-element)</code> element for answers&#46;

``` text
<article>
 <h1>FAQ</h1>
 <dl>
  <dt>What do we want?</dt>
  <dd>Our data.</dd>
  <dt>When do we want it?</dt>
  <dd>Now.</dd>
  <dt>Where is it?</dt>
  <dd>We are not sure.</dd>
 </dl>
</article>
```

#### <a id="the&#45;dd&#45;element"></a>4&#46;4&#46;11 The <strong><code>dd</code></strong> element[](#the-dd-element)

<strong>✔</strong>MDN

[Element&#47;dd](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dd)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android4&#43;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;dd&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

None&#46;

<strong><a id="the&#45;dd&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

After <a id="the&#45;dd&#45;element&#58;the&#45;dt&#45;element"></a><code>[dt](#the-dt-element)</code> or <a id="the&#45;dd&#45;element&#58;the&#45;dd&#45;element"></a><code>[dd](#the-dd-element)</code> elements inside <a id="the&#45;dd&#45;element&#58;the&#45;dl&#45;element"></a><code>[dl](#the-dl-element)</code> elements&#46;

After <a id="the&#45;dd&#45;element&#58;the&#45;dt&#45;element&#45;2"></a><code>[dt](#the-dt-element)</code> or <a id="the&#45;dd&#45;element&#58;the&#45;dd&#45;element&#45;2"></a><code>[dd](#the-dd-element)</code> elements inside <a id="the&#45;dd&#45;element&#58;the&#45;div&#45;element"></a><code>[div](#the-div-element)</code> elements that are children of a <a id="the&#45;dd&#45;element&#58;the&#45;dl&#45;element&#45;2"></a><code>[dl](#the-dl-element)</code> element&#46;

<strong><a id="the&#45;dd&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;dd&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<strong><a id="the&#45;dd&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

A <a id="the&#45;dd&#45;element&#58;the&#45;dd&#45;element&#45;3"></a><code>[dd](#the-dd-element)</code> element&#39;s <a id="the&#45;dd&#45;element&#58;syntax&#45;end&#45;tag"></a>[end tag](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#syntax-end-tag) can be omitted if the <a id="the&#45;dd&#45;element&#58;the&#45;dd&#45;element&#45;4"></a><code>[dd](#the-dd-element)</code> element is immediately followed by another <a id="the&#45;dd&#45;element&#58;the&#45;dd&#45;element&#45;5"></a><code>[dd](#the-dd-element)</code> element or a <a id="the&#45;dd&#45;element&#58;the&#45;dt&#45;element&#45;3"></a><code>[dt](#the-dt-element)</code> element&#44; or if there is no more content in the parent element&#46;

<strong><a id="the&#45;dd&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;dd&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;dd&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-dd)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-dd)&#46;

<strong><a id="the&#45;dd&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;dd&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;dd&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;dd&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;dd&#45;element&#58;the&#45;dd&#45;element&#45;6"></a><code>[dd](#the-dd-element)</code> element <a id="the&#45;dd&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) the description&#44; definition&#44; or value&#44; part of a term&#45;description group in a description list &#40;<a id="the&#45;dd&#45;element&#58;the&#45;dl&#45;element&#45;3"></a><code>[dl](#the-dl-element)</code> element&#41;&#46;

<strong>Example&#58;</strong>

A <a id="the&#45;dd&#45;element&#58;the&#45;dl&#45;element&#45;4"></a><code>[dl](#the-dl-element)</code> can be used to define a vocabulary list&#44; like in a dictionary&#46; In the following example&#44; each entry&#44; given by a <a id="the&#45;dd&#45;element&#58;the&#45;dt&#45;element&#45;4"></a><code>[dt](#the-dt-element)</code> with a <a id="the&#45;dd&#45;element&#58;the&#45;dfn&#45;element"></a><code>[dfn](#the-dfn-element)</code>&#44; has several <a id="the&#45;dd&#45;element&#58;the&#45;dd&#45;element&#45;7"></a><code>[dd](#the-dd-element)</code>s&#44; showing the various parts of the definition&#46;

``` text
<dl>
 <dt><dfn>happiness</dfn></dt>
 <dd class="pronunciation">/ˈhæpinəs/</dd>
 <dd class="part-of-speech"><i><abbr>n.</abbr></i></dd>
 <dd>The state of being happy.</dd>
 <dd>Good fortune; success. <q>Oh <b>happiness</b>! It worked!</q></dd>
 <dt><dfn>rejoice</dfn></dt>
 <dd class="pronunciation">/rɪˈdʒɔɪs/</dd>
 <dd><i class="part-of-speech"><abbr>v.intr.</abbr></i> To be delighted oneself.</dd>
 <dd><i class="part-of-speech"><abbr>v.tr.</abbr></i> To cause one to be delighted.</dd>
</dl>
```

#### <a id="the&#45;figure&#45;element"></a>4&#46;4&#46;12 The <strong><code>figure</code></strong> element[](#the-figure-element)

<strong>✔</strong>MDN

[Element&#47;figure](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/figure)

Support in all current engines&#46;

Firefox4&#43;Safari5&#46;1&#43;Chrome8&#43;

------------------------------------------------------------------------

Opera11&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer9&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android11&#43;

<strong><a id="the&#45;figure&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;figure&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;figure&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;figure&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;figure&#45;element&#58;flow&#45;content&#45;2&#45;2"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2) is expected&#46;

<strong><a id="the&#45;figure&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

Either&#58; one <a id="the&#45;figure&#45;element&#58;the&#45;figcaption&#45;element"></a><code>[figcaption](#the-figcaption-element)</code> element followed by <a id="the&#45;figure&#45;element&#58;flow&#45;content&#45;2&#45;3"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

Or&#58; <a id="the&#45;figure&#45;element&#58;flow&#45;content&#45;2&#45;4"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2) followed by one <a id="the&#45;figure&#45;element&#58;the&#45;figcaption&#45;element&#45;2"></a><code>[figcaption](#the-figcaption-element)</code> element&#46;

Or&#58; <a id="the&#45;figure&#45;element&#58;flow&#45;content&#45;2&#45;5"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<strong><a id="the&#45;figure&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;figure&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;figure&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;figure&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-figure)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-figure)&#46;

<strong><a id="the&#45;figure&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;figure&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;figure&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;figure&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;figure&#45;element&#58;the&#45;figure&#45;element"></a><code>[figure](#the-figure-element)</code> element <a id="the&#45;figure&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) some <a id="the&#45;figure&#45;element&#58;flow&#45;content&#45;2&#45;6"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#44; optionally with a caption&#44; that is self&#45;contained &#40;like a complete sentence&#41; and is typically <a id="the&#45;figure&#45;element&#58;referenced"></a>[referenced](html-language-foundations--877d6146487f--b55e73c31978.md#referenced) as a single unit from the main flow of the document&#46;

<strong>Note&#58;</strong>

&#34;Self&#45;contained&#34; in this context does not necessarily mean independent&#46; For example&#44; each sentence in a paragraph is self&#45;contained&#59; an image that is part of a sentence would be inappropriate for <a id="the&#45;figure&#45;element&#58;the&#45;figure&#45;element&#45;2"></a><code>[figure](#the-figure-element)</code>&#44; but an entire sentence made of images would be fitting&#46;

The element can thus be used to annotate illustrations&#44; diagrams&#44; photos&#44; code listings&#44; etc&#46;

<a id="figure&#45;note&#45;about&#45;references"></a>

<strong>Note&#58;</strong>

[](#figure-note-about-references)

When a <a id="the&#45;figure&#45;element&#58;the&#45;figure&#45;element&#45;3"></a><code>[figure](#the-figure-element)</code> is referred to from the main content of the document by identifying it by its caption &#40;e&#46;g&#46;&#44; by figure number&#41;&#44; it enables such content to be easily moved away from that primary content&#44; e&#46;g&#46;&#44; to the side of the page&#44; to dedicated pages&#44; or to an appendix&#44; without affecting the flow of the document&#46;

If a <a id="the&#45;figure&#45;element&#58;the&#45;figure&#45;element&#45;4"></a><code>[figure](#the-figure-element)</code> element is <a id="the&#45;figure&#45;element&#58;referenced&#45;2"></a>[referenced](html-language-foundations--877d6146487f--b55e73c31978.md#referenced) by its relative position&#44; e&#46;g&#46;&#44; &#34;in the photograph above&#34; or &#34;as the next figure shows&#34;&#44; then moving the figure would disrupt the page&#39;s meaning&#46; Authors are encouraged to consider using labels to refer to figures&#44; rather than using such relative references&#44; so that the page can easily be restyled without affecting the page&#39;s meaning&#46;

The first <a id="the&#45;figure&#45;element&#58;the&#45;figcaption&#45;element&#45;3"></a><code>[figcaption](#the-figcaption-element)</code> element child of the element&#44; if any&#44; represents the caption of the <a id="the&#45;figure&#45;element&#58;the&#45;figure&#45;element&#45;5"></a><code>[figure](#the-figure-element)</code> element&#39;s contents&#46; If there is no child <a id="the&#45;figure&#45;element&#58;the&#45;figcaption&#45;element&#45;4"></a><code>[figcaption](#the-figcaption-element)</code> element&#44; then there is no caption&#46;

A <a id="the&#45;figure&#45;element&#58;the&#45;figure&#45;element&#45;6"></a><code>[figure](#the-figure-element)</code> element&#39;s contents are part of the surrounding flow&#46; If the purpose of the page is to display the figure&#44; for example a photograph on an image sharing site&#44; the <a id="the&#45;figure&#45;element&#58;the&#45;figure&#45;element&#45;7"></a><code>[figure](#the-figure-element)</code> and <a id="the&#45;figure&#45;element&#58;the&#45;figcaption&#45;element&#45;5"></a><code>[figcaption](#the-figcaption-element)</code> elements can be used to explicitly provide a caption for that figure&#46; For content that is only tangentially related&#44; or that serves a separate purpose than the surrounding flow&#44; the <a id="the&#45;figure&#45;element&#58;the&#45;aside&#45;element"></a><code>[aside](#the-aside-element)</code> element should be used &#40;and can itself wrap a <a id="the&#45;figure&#45;element&#58;the&#45;figure&#45;element&#45;8"></a><code>[figure](#the-figure-element)</code>&#41;&#46; For example&#44; a pull quote that repeats content from an <a id="the&#45;figure&#45;element&#58;the&#45;article&#45;element"></a><code>[article](#the-article-element)</code> would be more appropriate in an <a id="the&#45;figure&#45;element&#58;the&#45;aside&#45;element&#45;2"></a><code>[aside](#the-aside-element)</code> than in a <a id="the&#45;figure&#45;element&#58;the&#45;figure&#45;element&#45;9"></a><code>[figure](#the-figure-element)</code>&#44; because it isn&#39;t part of the content&#44; it&#39;s a repetition of the content for the purposes of enticing readers or highlighting key topics&#46;

<strong>Example&#58;</strong>

This example shows the <a id="the&#45;figure&#45;element&#58;the&#45;figure&#45;element&#45;10"></a><code>[figure](#the-figure-element)</code> element to mark up a code listing&#46;

``` text
<p>In <a href="#l4">listing 4</a> we see the primary core interface
API declaration.</p>
<figure id="l4">
 <figcaption>Listing 4. The primary core interface API declaration.</figcaption>
 <pre><code>interface PrimaryCore {
 boolean verifyDataLine();
 undefined sendData(sequence&lt;byte> data);
 undefined initSelfDestruct();
}</code></pre>
</figure>
<p>The API is designed to use UTF-8.</p>
```

<strong>Example&#58;</strong>

Here we see a <a id="the&#45;figure&#45;element&#58;the&#45;figure&#45;element&#45;11"></a><code>[figure](#the-figure-element)</code> element to mark up a photo that is the main content of the page &#40;as in a gallery&#41;&#46;

``` text
<!DOCTYPE HTML>
<html lang="en">
<title>Bubbles at work — My Gallery™</title>
<figure>
 <img src="bubbles-work.jpeg"
      alt="Bubbles, sitting in his office chair, works on his
           latest project intently.">
 <figcaption>Bubbles at work</figcaption>
</figure>
<nav><a href="19414.html">Prev</a> — <a href="19416.html">Next</a></nav>
```

<strong>Example&#58;</strong>

In this example&#44; we see an image that is <em>not</em> a figure&#44; as well as an image and a video that are&#46; The first image is literally part of the example&#39;s second sentence&#44; so it&#39;s not a self&#45;contained unit&#44; and thus <a id="the&#45;figure&#45;element&#58;the&#45;figure&#45;element&#45;12"></a><code>[figure](#the-figure-element)</code> would be inappropriate&#46;

``` text
<h2>Malinko's comics</h2>

<p>This case centered on some sort of "intellectual property"
infringement related to a comic (see Exhibit A). The suit started
after a trailer ending with these words:

<blockquote>
 <img src="promblem-packed-action.png" alt="ROUGH COPY! Promblem-Packed Action!">
</blockquote>

<p>...was aired. A lawyer, armed with a Bigger Notebook, launched a
preemptive strike using snowballs. A complete copy of the trailer is
included with Exhibit B.

<figure>
 <img src="ex-a.png" alt="Two squiggles on a dirty piece of paper.">
 <figcaption>Exhibit A. The alleged <cite>rough copy</cite> comic.</figcaption>
</figure>

<figure>
 <video src="ex-b.mov"></video>
 <figcaption>Exhibit B. The <cite>Rough Copy</cite> trailer.</figcaption>
</figure>

<p>The case was resolved out of court.
```

<strong>Example&#58;</strong>

Here&#44; a part of a poem is marked up using <a id="the&#45;figure&#45;element&#58;the&#45;figure&#45;element&#45;13"></a><code>[figure](#the-figure-element)</code>&#46;

``` text
<figure>
 <p>'Twas brillig, and the slithy toves<br>
 Did gyre and gimble in the wabe;<br>
 All mimsy were the borogoves,<br>
 And the mome raths outgrabe.</p>
 <figcaption><cite>Jabberwocky</cite> (first verse). Lewis Carroll, 1832-98</figcaption>
</figure>
```

<strong>Example&#58;</strong>

In this example&#44; which could be part of a much larger work discussing a castle&#44; nested <a id="the&#45;figure&#45;element&#58;the&#45;figure&#45;element&#45;14"></a><code>[figure](#the-figure-element)</code> elements are used to provide both a group caption and individual captions for each figure in the group&#58;

``` text
<figure>
 <figcaption>The castle through the ages: 1423, 1858, and 1999 respectively.</figcaption>
 <figure>
  <figcaption>Etching. Anonymous, ca. 1423.</figcaption>
  <img src="castle1423.jpeg" alt="The castle has one tower, and a tall wall around it.">
 </figure>
 <figure>
  <figcaption>Oil-based paint on canvas. Maria Towle, 1858.</figcaption>
  <img src="castle1858.jpeg" alt="The castle now has two towers and two walls.">
 </figure>
 <figure>
  <figcaption>Film photograph. Peter Jankle, 1999.</figcaption>
  <img src="castle1999.jpeg" alt="The castle lies in ruins, the original tower all that remains in one piece.">
 </figure>
</figure>
```

<strong>Example&#58;</strong>

The previous example could also be more succinctly written as follows &#40;using <a id="the&#45;figure&#45;element&#58;attr&#45;title"></a><code>[title](html-language-foundations--877d6146487f--b55e73c31978.md#attr-title)</code> attributes in place of the nested <a id="the&#45;figure&#45;element&#58;the&#45;figure&#45;element&#45;15"></a><code>[figure](#the-figure-element)</code>&#47;<a id="the&#45;figure&#45;element&#58;the&#45;figcaption&#45;element&#45;6"></a><code>[figcaption](#the-figcaption-element)</code> pairs&#41;&#58;

``` text
<figure>
 <img src="castle1423.jpeg" title="Etching. Anonymous, ca. 1423."
      alt="The castle has one tower, and a tall wall around it.">
 <img src="castle1858.jpeg" title="Oil-based paint on canvas. Maria Towle, 1858."
      alt="The castle now has two towers and two walls.">
 <img src="castle1999.jpeg" title="Film photograph. Peter Jankle, 1999."
      alt="The castle lies in ruins, the original tower all that remains in one piece.">
 <figcaption>The castle through the ages: 1423, 1858, and 1999 respectively.</figcaption>
</figure>
```

<strong>Example&#58;</strong>

The figure is sometimes <a id="the&#45;figure&#45;element&#58;referenced&#45;3"></a>[referenced](html-language-foundations--877d6146487f--b55e73c31978.md#referenced) only implicitly from the content&#58;

``` text
<article>
 <h1>Fiscal negotiations stumble in Congress as deadline nears</h1>
 <figure>
  <img src="obama-reid.jpeg" alt="Obama and Reid sit together smiling in the Oval Office.">
  <figcaption>Barack Obama and Harry Reid. White House press photograph.</figcaption>
 </figure>
 <p>Negotiations in Congress to end the fiscal impasse sputtered on Tuesday, leaving both chambers
 grasping for a way to reopen the government and raise the country's borrowing authority with a
 Thursday deadline drawing near.</p>
 ...
</article>
```

#### <a id="the&#45;figcaption&#45;element"></a>4&#46;4&#46;13 The <strong><code>figcaption</code></strong> element[](#the-figcaption-element)

<strong>✔</strong>MDN

[Element&#47;figcaption](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/figcaption)

Support in all current engines&#46;

Firefox4&#43;Safari5&#46;1&#43;Chrome8&#43;

------------------------------------------------------------------------

Opera11&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer9&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android11&#43;

<strong><a id="the&#45;figcaption&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

None&#46;

<strong><a id="the&#45;figcaption&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

As the first or last child of a <a id="the&#45;figcaption&#45;element&#58;the&#45;figure&#45;element"></a><code>[figure](#the-figure-element)</code> element&#46;

<strong><a id="the&#45;figcaption&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;figcaption&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<strong><a id="the&#45;figcaption&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;figcaption&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;figcaption&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;figcaption&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-figcaption)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-figcaption)&#46;

<strong><a id="the&#45;figcaption&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;figcaption&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;figcaption&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;figcaption&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;figcaption&#45;element&#58;the&#45;figcaption&#45;element"></a><code>[figcaption](#the-figcaption-element)</code> element <a id="the&#45;figcaption&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a caption or legend for the rest of the contents of the <a id="the&#45;figcaption&#45;element&#58;the&#45;figcaption&#45;element&#45;2"></a><code>[figcaption](#the-figcaption-element)</code> element&#39;s parent <a id="the&#45;figcaption&#45;element&#58;the&#45;figure&#45;element&#45;2"></a><code>[figure](#the-figure-element)</code> element&#44; if any&#46;

<strong>Example&#58;</strong>

The element can contain additional information about the source&#58;

``` text
<figcaption>
 <p>A duck.</p>
 <p><small>Photograph courtesy of 🌟 News.</small></p>
</figcaption>
```

``` text
<figcaption>
 <p>Average rent for 3-room apartments, excluding non-profit apartments</p>
 <p>Zürich’s Statistics Office — <time datetime=2017-11-14>14 November 2017</time></p>
</figcaption>
```

#### <a id="the&#45;main&#45;element"></a>4&#46;4&#46;14 The <strong><code>main</code></strong> element[](#the-main-element)

<strong>✔</strong>MDN

[Element&#47;main](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/main)

Support in all current engines&#46;

Firefox21&#43;Safari7&#43;Chrome26&#43;

------------------------------------------------------------------------

Opera16&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;main&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;main&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;main&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;main&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;main&#45;element&#58;flow&#45;content&#45;2&#45;2"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2) is expected&#44; but only if it is a <a id="the&#45;main&#45;element&#58;hierarchically&#45;correct&#45;main&#45;element"></a>[hierarchically correct <code>main</code> element](#hierarchically-correct-main-element)&#46;

<strong><a id="the&#45;main&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;main&#45;element&#58;flow&#45;content&#45;2&#45;3"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<strong><a id="the&#45;main&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;main&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;main&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;main&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-main)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-main)&#46;

<strong><a id="the&#45;main&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;main&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;main&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;main&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;main&#45;element&#58;the&#45;main&#45;element"></a><code>[main](#the-main-element)</code> element <a id="the&#45;main&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) the dominant contents of the document&#46;

A document must not have more than one <a id="the&#45;main&#45;element&#58;the&#45;main&#45;element&#45;2"></a><code>[main](#the-main-element)</code> element that does not have the <a id="the&#45;main&#45;element&#58;attr&#45;hidden"></a><code>[hidden](html-language-host-facts--877d6146487f--2dd822df9d5e.md#attr-hidden)</code> attribute specified&#46;

A <a id="hierarchically&#45;correct&#45;main&#45;element"></a><strong>hierarchically correct <code>main</code> element</strong> is one whose ancestor elements are limited to <a id="the&#45;main&#45;element&#58;the&#45;html&#45;element"></a><code>[html](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-html-element)</code>&#44; <a id="the&#45;main&#45;element&#58;the&#45;body&#45;element"></a><code>[body](#the-body-element)</code>&#44; <a id="the&#45;main&#45;element&#58;the&#45;div&#45;element"></a><code>[div](#the-div-element)</code>&#44; <a id="the&#45;main&#45;element&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> without an <a id="the&#45;main&#45;element&#58;concept&#45;accessible&#45;name"></a>[accessible name](https://w3c.github.io/accname/#dfn-accessible-name)&#44; and <a id="the&#45;main&#45;element&#58;autonomous&#45;custom&#45;element"></a>[autonomous custom elements](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#autonomous-custom-element)&#46; Each <a id="the&#45;main&#45;element&#58;the&#45;main&#45;element&#45;3"></a><code>[main](#the-main-element)</code> element must be a <a id="the&#45;main&#45;element&#58;hierarchically&#45;correct&#45;main&#45;element&#45;2"></a>[hierarchically correct <code>main</code> element](#hierarchically-correct-main-element)&#46;

<strong>Example&#58;</strong>

In this example&#44; the author has used a presentation where each component of the page is rendered in a box&#46; To wrap the main content of the page &#40;as opposed to the header&#44; the footer&#44; the navigation bar&#44; and a sidebar&#41;&#44; the <a id="the&#45;main&#45;element&#58;the&#45;main&#45;element&#45;4"></a><code>[main](#the-main-element)</code> element is used&#46;

``` text
<!DOCTYPE html>
<html lang="en">
<title>RPG System 17</title>
<style>
 header, nav, aside, main, footer {
   margin: 0.5em; border: thin solid; padding: 0.5em;
   background: #EFF; color: black; box-shadow: 0 0 0.25em #033;
 }
 h1, h2, p { margin: 0; }
 nav, main { float: left; }
 aside { float: right; }
 footer { clear: both; }
</style>
<header>
 <h1>System Eighteen</h1>
</header>
<nav>
 <a href="../16/">← System 17</a>
 <a href="../18/">RPXIX →</a>
</nav>
<aside>
 <p>This system has no HP mechanic, so there's no healing.
</aside>
<main>
 <h2>Character creation</h2>
 <p>Attributes (magic, strength, agility) are purchased at the cost of one point per level.</p>
 <h2>Rolls</h2>
 <p>Each encounter, roll the dice for all your skills. If you roll more than the opponent, you win.</p>
</main>
<footer>
 <p>Copyright © 2013
</footer>
</html>
```

In the following example&#44; multiple <a id="the&#45;main&#45;element&#58;the&#45;main&#45;element&#45;5"></a><code>[main](#the-main-element)</code> elements are used and script is used to make navigation work without a server roundtrip and to set the <a id="the&#45;main&#45;element&#58;attr&#45;hidden&#45;2"></a><code>[hidden](html-language-host-facts--877d6146487f--2dd822df9d5e.md#attr-hidden)</code> attribute on those that are not current&#58;

``` text
<!doctype html>
<html lang=en-CA>
<meta charset=utf-8>
<title> … </title>
<link rel=stylesheet href=spa.css>
<script src=spa.js async></script>
<nav>
 <a href=/>Home</a>
 <a href=/about>About</a>
 <a href=/contact>Contact</a>
</nav>
<main>
 <h1>Home</h1>
 …
</main>
<main hidden>
 <h1>About</h1>
 …
</main>
<main hidden>
 <h1>Contact</h1>
 …
</main>
<footer>Made with ❤️ by <a href=https://example.com/>Example 👻</a>.</footer>
```

#### <a id="the&#45;search&#45;element"></a>4&#46;4&#46;15 The <strong><code>search</code></strong> element[](#the-search-element)

<strong>⚠</strong>MDN

[Element&#47;search](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/search)

No support in current engines&#46;

FirefoxNoSafariNoChromeNo

------------------------------------------------------------------------

OperaNoEdgeNo

------------------------------------------------------------------------

Edge &#40;Legacy&#41;NoInternet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome AndroidNoWebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;search&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;search&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;search&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;search&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;search&#45;element&#58;flow&#45;content&#45;2&#45;2"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2) is expected&#46;

<strong><a id="the&#45;search&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;search&#45;element&#58;flow&#45;content&#45;2&#45;3"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<strong><a id="the&#45;search&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;search&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;search&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;search&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-search)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-search)&#46;

<strong><a id="the&#45;search&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;search&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;search&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;search&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;search&#45;element&#58;the&#45;search&#45;element"></a><code>[search](#the-search-element)</code> element <a id="the&#45;search&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a part of a document or application that contains a set of form controls or other content related to performing a search or filtering operation&#46; This could be a search of the web site or application&#59; a way of searching or filtering search results on the current web page&#59; or a global or Internet&#45;wide search function&#46;

<strong>Note&#58;</strong>

It&#39;s not appropriate to use the <a id="the&#45;search&#45;element&#58;the&#45;search&#45;element&#45;2"></a><code>[search](#the-search-element)</code> element just for presenting search results&#44; though suggestions and links as part of &#34;quick search&#34; results can be included as part of a search feature&#46; Rather&#44; a returned web page of search results would instead be expected to be presented as part of the main content of that web page&#46;

<strong>Example&#58;</strong>

In the following example&#44; the author is including a search form within the <a id="the&#45;search&#45;element&#58;the&#45;header&#45;element"></a><code>[header](#the-header-element)</code> of the web page&#58;

``` text
<header>
  <h1><a href="/">My fancy blog</a></h1>
  ...
  <search>
    <form action="search.php">
      <label for="query">Find an article</label>
      <input id="query" name="q" type="search">
      <button type="submit">Go!</button>
    </form>
  </search>
</header>
```

<strong>Example&#58;</strong>

In this example&#44; the author has implemented their web application&#39;s search functionality entirely with JavaScript&#46; There is no use of the <a id="the&#45;search&#45;element&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> element to perform server&#45;side submission&#44; but the containing <a id="the&#45;search&#45;element&#58;the&#45;search&#45;element&#45;3"></a><code>[search](#the-search-element)</code> element semantically identifies the purpose of the descendant content as representing search capabilities&#46;

``` text
<search>
  <label>
    Find and filter your query
    <input type="search" id="query">
  </label>
  <label>
    <input type="checkbox" id="exact-only">
    Exact matches only
  </label>

  <section>
    <h3>Results found:</h3>
    <ul id="results">
      <li>
        <p><a href="services/consulting">Consulting services</a></p>
        <p>
          Find out how can we help you improve your business with our integrated consultants, Bob and Bob.
        </p>
      </li>
      ...
    </ul>
    <!--
      when a query returns or filters out all results
      render the no results message here
    -->
    <output id="no-results"></output>
  </section>
</search>
```

<strong>Example&#58;</strong>

In the following example&#44; the page has two search features&#46; The first is located in the web page&#39;s <a id="the&#45;search&#45;element&#58;the&#45;header&#45;element&#45;2"></a><code>[header](#the-header-element)</code> and serves as a global mechanism to search the web site&#39;s content&#46; Its purpose is indicated by its specified <a id="the&#45;search&#45;element&#58;the&#45;title&#45;element"></a><code>[title](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-title-element)</code> attribute&#46; The second is included as part of the main content of the page&#44; as it represents a mechanism to search and filter the content of the current page&#46; It contains a heading to indicate its purpose&#46;

``` text
<body>
  <header>
    ...
    <search title="Website">
      ...
    </search>
  </header>
  <main>
    <h1>Hotels near your location</h1>
     <search>
       <h2>Filter results</h2>
       ...
     </search>
     <article>
      <!-- search result content -->
    </article>
  </main>
</body>
```

#### <a id="the&#45;div&#45;element"></a>4&#46;4&#46;16 The <strong><code>div</code></strong> element[](#the-div-element)

<strong>✔</strong>MDN

[Element&#47;div](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/div)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong>✔</strong>MDN

[HTMLDivElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDivElement)

Support in all current engines&#46;

Firefox1&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

<strong><a id="the&#45;div&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;div&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;div&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;div&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;div&#45;element&#58;flow&#45;content&#45;2&#45;2"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2) is expected&#46;

As a child of a <a id="the&#45;div&#45;element&#58;the&#45;dl&#45;element"></a><code>[dl](#the-dl-element)</code> element&#46;

As a descendant of an <a id="the&#45;div&#45;element&#58;the&#45;option&#45;element"></a><code>[option](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-option-element)</code> element&#44; an <a id="the&#45;div&#45;element&#58;the&#45;optgroup&#45;element"></a><code>[optgroup](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-optgroup-element)</code> element&#44; or a <a id="the&#45;div&#45;element&#58;the&#45;select&#45;element"></a><code>[select](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-select-element)</code> element&#46;

<strong><a id="the&#45;div&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

If the element is a child of a <a id="the&#45;div&#45;element&#58;the&#45;dl&#45;element&#45;2"></a><code>[dl](#the-dl-element)</code> element&#58; One or more <a id="the&#45;div&#45;element&#58;the&#45;dt&#45;element"></a><code>[dt](#the-dt-element)</code> elements followed by one or more <a id="the&#45;div&#45;element&#58;the&#45;dd&#45;element"></a><code>[dd](#the-dd-element)</code> elements&#44; optionally intermixed with <a id="the&#45;div&#45;element&#58;script&#45;supporting&#45;elements&#45;2"></a>[script&#45;supporting elements](html-language-foundations--877d6146487f--b55e73c31978.md#script-supporting-elements-2)&#46;

Otherwise&#44; if the element is a descendant of an <a id="the&#45;div&#45;element&#58;the&#45;option&#45;element&#45;2"></a><code>[option](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-option-element)</code> element&#44; an <a id="the&#45;div&#45;element&#58;the&#45;optgroup&#45;element&#45;2"></a><code>[optgroup](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-optgroup-element)</code> element&#44; or a <a id="the&#45;div&#45;element&#58;the&#45;select&#45;element&#45;2"></a><code>[select](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-select-element)</code> element&#58; <a id="the&#45;div&#45;element&#58;transparent"></a>[transparent](html-language-foundations--877d6146487f--b55e73c31978.md#transparent)&#46;

Otherwise&#58; <a id="the&#45;div&#45;element&#58;flow&#45;content&#45;2&#45;3"></a>[flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<strong><a id="the&#45;div&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;div&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;div&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;div&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-div)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-div)&#46;

<strong><a id="the&#45;div&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;div&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;div&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

<a id="htmldivelement"></a><a id="the&#45;div&#45;element&#58;htmlelement"></a><a id="the&#45;div&#45;element&#58;htmlconstructor"></a>

``` text
[Exposed=Window]
interface HTMLDivElement : HTMLElement {
  [HTMLConstructor] constructor();

  // also has obsolete members
};
```

The <a id="the&#45;div&#45;element&#58;the&#45;div&#45;element"></a><code>[div](#the-div-element)</code> element has no special meaning at all&#46; It <a id="the&#45;div&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) its children&#46; It can be used with the <a id="the&#45;div&#45;element&#58;classes"></a><code>[class](html-language-foundations--877d6146487f--b55e73c31978.md#classes)</code>&#44; <a id="the&#45;div&#45;element&#58;attr&#45;lang"></a><code>[lang](html-language-foundations--877d6146487f--b55e73c31978.md#attr-lang)</code>&#44; and <a id="the&#45;div&#45;element&#58;attr&#45;title"></a><code>[title](html-language-foundations--877d6146487f--b55e73c31978.md#attr-title)</code> attributes to mark up semantics common to a group of consecutive elements&#46; It can also be used in a <a id="the&#45;div&#45;element&#58;the&#45;dl&#45;element&#45;3"></a><code>[dl](#the-dl-element)</code> element&#44; wrapping groups of <a id="the&#45;div&#45;element&#58;the&#45;dt&#45;element&#45;2"></a><code>[dt](#the-dt-element)</code> and <a id="the&#45;div&#45;element&#58;the&#45;dd&#45;element&#45;2"></a><code>[dd](#the-dd-element)</code> elements&#46;

<strong>Note&#58;</strong>

Authors are strongly encouraged to view the <a id="the&#45;div&#45;element&#58;the&#45;div&#45;element&#45;2"></a><code>[div](#the-div-element)</code> element as an element of last resort&#44; for when no other element is suitable&#46; Use of more appropriate elements instead of the <a id="the&#45;div&#45;element&#58;the&#45;div&#45;element&#45;3"></a><code>[div](#the-div-element)</code> element leads to better accessibility for readers and easier maintainability for authors&#46;

<strong>Example&#58;</strong>

For example&#44; a blog post would be marked up using <a id="the&#45;div&#45;element&#58;the&#45;article&#45;element"></a><code>[article](#the-article-element)</code>&#44; a chapter using <a id="the&#45;div&#45;element&#58;the&#45;section&#45;element"></a><code>[section](#the-section-element)</code>&#44; a page&#39;s navigation aids using <a id="the&#45;div&#45;element&#58;the&#45;nav&#45;element"></a><code>[nav](#the-nav-element)</code>&#44; and a group of form controls using <a id="the&#45;div&#45;element&#58;the&#45;fieldset&#45;element"></a><code>[fieldset](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-fieldset-element)</code>&#46;

On the other hand&#44; <a id="the&#45;div&#45;element&#58;the&#45;div&#45;element&#45;4"></a><code>[div](#the-div-element)</code> elements can be useful for stylistic purposes or to wrap multiple paragraphs within a section that are all to be annotated in a similar way&#46; In the following example&#44; we see <a id="the&#45;div&#45;element&#58;the&#45;div&#45;element&#45;5"></a><code>[div](#the-div-element)</code> elements used as a way to set the language of two paragraphs at once&#44; instead of setting the language on the two paragraph elements separately&#58;

``` text
<article lang="en-US">
 <h1>My use of language and my cats</h1>
 <p>My cat's behavior hasn't changed much since her absence, except
 that she plays her new physique to the neighbors regularly, in an
 attempt to get pets.</p>
 <div lang="en-GB">
  <p>My other cat, coloured black and white, is a sweetie. He followed
  us to the pool today, walking down the pavement with us. Yesterday
  he apparently visited our neighbours. I wonder if he recognises that
  their flat is a mirror image of ours.</p>
  <p>Hm, I just noticed that in the last paragraph I used British
  English. But I'm supposed to write in American English. So I
  shouldn't say "pavement" or "flat" or "colour"...</p>
 </div>
 <p>I should say "sidewalk" and "apartment" and "color"!</p>
</article>
```

### <a id="text&#45;level&#45;semantics"></a>4&#46;5 Text&#45;level semantics[](#text-level-semantics)

#### <a id="the&#45;a&#45;element"></a>4&#46;5&#46;1 The <strong><code>a</code></strong> element[](#the-a-element)

<strong>✔</strong>MDN

[Element&#47;a](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong>✔</strong>MDN

[HTMLAnchorElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android37&#43;Samsung Internet&#63;Opera Android12&#46;1&#43;

[HTMLAnchorElement&#47;download](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement/download)

Support in all current engines&#46;

Firefox20&#43;Safari10&#46;1&#43;Chrome15&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;13&#43;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[HTMLAnchorElement&#47;rel](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement/rel)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

[HTMLAnchorElement&#47;relList](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement/relList)

Support in all current engines&#46;

Firefox30&#43;Safari9&#43;Chrome65&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;18Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;a&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;a&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;a&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

If the element has an <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;href"></a><code>[href](#attr-hyperlink-href)</code> attribute&#58; <a id="the&#45;a&#45;element&#58;interactive&#45;content&#45;2"></a>[Interactive content](html-language-foundations--877d6146487f--b55e73c31978.md#interactive-content-2)&#46;

<a id="the&#45;a&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;a&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;a&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;a&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;a&#45;element&#58;transparent"></a>[Transparent](html-language-foundations--877d6146487f--b55e73c31978.md#transparent)&#44; but there must be no <a id="the&#45;a&#45;element&#58;interactive&#45;content&#45;2&#45;2"></a>[interactive content](html-language-foundations--877d6146487f--b55e73c31978.md#interactive-content-2) descendant&#44; <a id="the&#45;a&#45;element&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code> element descendant&#44; or descendant with the <a id="the&#45;a&#45;element&#58;attr&#45;tabindex"></a><code>[tabindex](html-language-host-facts--877d6146487f--2dd822df9d5e.md#attr-tabindex)</code> attribute specified&#46;

<strong><a id="the&#45;a&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;a&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;a&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;href&#45;2"></a><code>[href](#attr-hyperlink-href)</code> — Address of the <a id="the&#45;a&#45;element&#58;hyperlink"></a>[hyperlink](#hyperlink)

<a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;target"></a><code>[target](#attr-hyperlink-target)</code> — <a id="the&#45;a&#45;element&#58;navigable"></a>[Navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable) for <a id="the&#45;a&#45;element&#58;hyperlink&#45;2"></a>[hyperlink](#hyperlink) <a id="the&#45;a&#45;element&#58;navigate"></a>[navigation](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigate)

<a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;download"></a><code>[download](#attr-hyperlink-download)</code> — Whether to download the resource instead of navigating to it&#44; and its filename if so

<a id="the&#45;a&#45;element&#58;ping"></a><code>[ping](#ping)</code> — <a id="the&#45;a&#45;element&#58;url"></a>[URLs](https://url.spec.whatwg.org/#concept-url) to ping

<a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;rel"></a><code>[rel](#attr-hyperlink-rel)</code> — Relationship between the location in the document containing the <a id="the&#45;a&#45;element&#58;hyperlink&#45;3"></a>[hyperlink](#hyperlink) and the destination resource

<a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;hreflang"></a><code>[hreflang](#attr-hyperlink-hreflang)</code> — Language of the linked resource

<a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;type"></a><code>[type](#attr-hyperlink-type)</code> — Hint for the type of the referenced resource

<a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;referrerpolicy"></a><code>[referrerpolicy](#attr-hyperlink-referrerpolicy)</code> — <a id="the&#45;a&#45;element&#58;referrer&#45;policy"></a>[Referrer policy](https://w3c.github.io/webappsec-referrer-policy/#referrer-policy) for <a id="the&#45;a&#45;element&#58;concept&#45;fetch"></a>[fetches](https://fetch.spec.whatwg.org/#concept-fetch) initiated by the element

<strong><a id="the&#45;a&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

If the element has an <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;href&#45;3"></a><code>[href](#attr-hyperlink-href)</code> attribute&#58; [for authors](https://w3c.github.io/html-aria/#el-a)&#59; [for implementers](https://w3c.github.io/html-aam/#el-a)&#46;

Otherwise&#58; [for authors](https://w3c.github.io/html-aria/#el-a-no-href)&#59; [for implementers](https://w3c.github.io/html-aam/#el-a-no-href)&#46;

<strong><a id="the&#45;a&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;a&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default) with attributes <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;href&#45;4"></a><code>[href](#attr-hyperlink-href)</code>&#44; <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;hreflang&#45;2"></a><code>[hreflang](#attr-hyperlink-hreflang)</code>&#44; and <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;type&#45;2"></a><code>[type](#attr-hyperlink-type)</code>&#44; and <a id="the&#45;a&#45;element&#58;navigating&#45;url&#45;attributes"></a>[navigating URL attributes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigating-url-attributes) <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;href&#45;5"></a><code>[href](#attr-hyperlink-href)</code>&#46;

<strong><a id="the&#45;a&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

<a id="htmlanchorelement"></a><a id="the&#45;a&#45;element&#58;htmlelement"></a><a id="the&#45;a&#45;element&#58;htmlconstructor"></a><a id="the&#45;a&#45;element&#58;cereactions"></a><a id="the&#45;a&#45;element&#58;xattr&#45;reflect"></a><a id="dom&#45;a&#45;download"></a><a id="the&#45;a&#45;element&#58;cereactions&#45;2"></a><a id="the&#45;a&#45;element&#58;xattr&#45;reflect&#45;2"></a><a id="dom&#45;a&#45;ping"></a><a id="the&#45;a&#45;element&#58;cereactions&#45;3"></a><a id="the&#45;a&#45;element&#58;xattr&#45;reflect&#45;3"></a><a id="dom&#45;a&#45;rel"></a><a id="the&#45;a&#45;element&#58;dom&#45;domtokenlist&#45;value"></a><a id="the&#45;a&#45;element&#58;xattr&#45;reflect&#45;4"></a><a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;rel&#45;2"></a><a id="the&#45;a&#45;element&#58;domtokenlist"></a><a id="dom&#45;a&#45;rellist"></a><a id="the&#45;a&#45;element&#58;cereactions&#45;4"></a><a id="the&#45;a&#45;element&#58;dom&#45;a&#45;text"></a><a id="the&#45;a&#45;element&#58;cereactions&#45;5"></a><a id="the&#45;a&#45;element&#58;dom&#45;a&#45;referrerpolicy"></a><a id="the&#45;a&#45;element&#58;htmlanchorelement"></a><a id="the&#45;a&#45;element&#58;hyperlinkelementutils"></a><a id="the&#45;a&#45;element&#58;htmlanchorelement&#45;2"></a><a id="the&#45;a&#45;element&#58;htmlhyperlinkelementutils"></a>

``` text
[Exposed=Window]
interface HTMLAnchorElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions, Reflect] attribute DOMString download;
  [CEReactions, Reflect] attribute USVString ping;
  [CEReactions, Reflect] attribute DOMString rel;
  [SameObject, PutForwards=value, Reflect="rel"] readonly attribute DOMTokenList relList;

  [CEReactions] attribute DOMString text;

  [CEReactions] attribute DOMString referrerPolicy;

  // also has obsolete members
};
HTMLAnchorElement includes HyperlinkElementUtils;
HTMLAnchorElement includes HTMLHyperlinkElementUtils;
```

If the <a id="the&#45;a&#45;element&#58;the&#45;a&#45;element&#45;2"></a><code>[a](#the-a-element)</code> element has an <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;href&#45;6"></a><code>[href](#attr-hyperlink-href)</code> attribute&#44; then it <a id="the&#45;a&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a <a id="the&#45;a&#45;element&#58;hyperlink&#45;4"></a>[hyperlink](#hyperlink) &#40;a hypertext anchor&#41; labeled by its contents&#46;

If the <a id="the&#45;a&#45;element&#58;the&#45;a&#45;element&#45;3"></a><code>[a](#the-a-element)</code> element has no <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;href&#45;7"></a><code>[href](#attr-hyperlink-href)</code> attribute&#44; then the element <a id="the&#45;a&#45;element&#58;represents&#45;2"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a placeholder for where a link might otherwise have been placed&#44; if it had been relevant&#44; consisting of just the element&#39;s contents&#46;

The <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;target&#45;2"></a><code>[target](#attr-hyperlink-target)</code>&#44; <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;download&#45;2"></a><code>[download](#attr-hyperlink-download)</code>&#44; <a id="the&#45;a&#45;element&#58;ping&#45;2"></a><code>[ping](#ping)</code>&#44; <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;rel&#45;3"></a><code>[rel](#attr-hyperlink-rel)</code>&#44; <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;hreflang&#45;3"></a><code>[hreflang](#attr-hyperlink-hreflang)</code>&#44; <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;type&#45;3"></a><code>[type](#attr-hyperlink-type)</code>&#44; and <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;referrerpolicy&#45;2"></a><code>[referrerpolicy](#attr-hyperlink-referrerpolicy)</code> attributes must be omitted if the <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;href&#45;8"></a><code>[href](#attr-hyperlink-href)</code> attribute is not present&#46;

If the <a id="the&#45;a&#45;element&#58;names&#58;&#45;the&#45;itemprop&#45;attribute"></a><code>[itemprop](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#names:-the-itemprop-attribute)</code> attribute is specified on an <a id="the&#45;a&#45;element&#58;the&#45;a&#45;element&#45;4"></a><code>[a](#the-a-element)</code> element&#44; then the <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;href&#45;9"></a><code>[href](#attr-hyperlink-href)</code> attribute must also be specified&#46;

<strong>Example&#58;</strong>

If a site uses a consistent navigation toolbar on every page&#44; then the link that would normally link to the page itself could be marked up using an <a id="the&#45;a&#45;element&#58;the&#45;a&#45;element&#45;5"></a><code>[a](#the-a-element)</code> element&#58;

``` text
<nav>
 <ul>
  <li> <a href="/">Home</a> </li>
  <li> <a href="/news">News</a> </li>
  <li> <a>Examples</a> </li>
  <li> <a href="/legal">Legal</a> </li>
 </ul>
</nav>
```

The <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;href&#45;10"></a><code>[href](#attr-hyperlink-href)</code>&#44; <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;target&#45;3"></a><code>[target](#attr-hyperlink-target)</code>&#44; <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;download&#45;3"></a><code>[download](#attr-hyperlink-download)</code>&#44; <a id="the&#45;a&#45;element&#58;ping&#45;3"></a><code>[ping](#ping)</code>&#44; and <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;referrerpolicy&#45;3"></a><code>[referrerpolicy](#attr-hyperlink-referrerpolicy)</code> attributes affect what happens when users <a id="the&#45;a&#45;element&#58;following&#45;hyperlinks&#45;2"></a>[follow hyperlinks](#following-hyperlinks-2) or <a id="the&#45;a&#45;element&#58;downloading&#45;hyperlinks"></a>[download hyperlinks](#downloading-hyperlinks) created using the <a id="the&#45;a&#45;element&#58;the&#45;a&#45;element&#45;6"></a><code>[a](#the-a-element)</code> element&#46; The <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;rel&#45;4"></a><code>[rel](#attr-hyperlink-rel)</code>&#44; <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;hreflang&#45;4"></a><code>[hreflang](#attr-hyperlink-hreflang)</code>&#44; and <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;type&#45;4"></a><code>[type](#attr-hyperlink-type)</code> attributes may be used to indicate to the user the likely nature of the target resource before the user follows the link&#46;

<strong><code><var>a</var>&#46;<a id="dom&#45;a&#45;text&#45;dev"></a>[text](#dom-a-text)</code></strong>

Same as <a id="the&#45;a&#45;element&#58;textcontent"></a><code>[textContent](https://dom.spec.whatwg.org/#dom-node-textcontent)</code>&#46;

<strong>✔</strong>MDN

[HTMLAnchorElement&#47;referrerPolicy](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement/referrerPolicy)

Support in all current engines&#46;

Firefox50&#43;Safari14&#43;Chrome52&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

The IDL attribute <a id="dom&#45;a&#45;referrerpolicy"></a><strong><code>referrerPolicy</code></strong> must <a id="the&#45;a&#45;element&#58;reflect"></a>[reflect](html-language-foundations--877d6146487f--b55e73c31978.md#reflect) the <a id="the&#45;a&#45;element&#58;attr&#45;hyperlink&#45;referrerpolicy&#45;4"></a><code>[referrerpolicy](#attr-hyperlink-referrerpolicy)</code> content attribute&#44; <a id="the&#45;a&#45;element&#58;limited&#45;to&#45;only&#45;known&#45;values"></a>[limited to only known values](html-language-foundations--877d6146487f--b55e73c31978.md#limited-to-only-known-values)&#46;

The <a id="dom&#45;a&#45;text"></a><strong><code>text</code></strong> attribute&#39;s getter must return this element&#39;s <a id="the&#45;a&#45;element&#58;descendant&#45;text&#45;content"></a>[descendant text content](https://dom.spec.whatwg.org/#concept-descendant-text-content)&#46;

The <a id="the&#45;a&#45;element&#58;dom&#45;a&#45;text&#45;2"></a><code>[text](#dom-a-text)</code> attribute&#39;s setter must <a id="the&#45;a&#45;element&#58;string&#45;replace&#45;all"></a>[string replace all](https://dom.spec.whatwg.org/#string-replace-all) with the given value within this element&#46;

<strong>Example&#58;</strong>

The <a id="the&#45;a&#45;element&#58;the&#45;a&#45;element&#45;7"></a><code>[a](#the-a-element)</code> element can be wrapped around entire paragraphs&#44; lists&#44; tables&#44; and so forth&#44; even entire sections&#44; so long as there is no interactive content within &#40;e&#46;g&#46;&#44; buttons or other links&#41;&#46; This example shows how this can be used to make an entire advertising block into a link&#58;

``` text
<aside class="advertising">
 <h1>Advertising</h1>
 <a href="https://ad.example.com/?adid=1929&amp;pubid=1422">
  <section>
   <h1>Mellblomatic 9000!</h1>
   <p>Turn all your widgets into mellbloms!</p>
   <p>Only $9.99 plus shipping and handling.</p>
  </section>
 </a>
 <a href="https://ad.example.com/?adid=375&amp;pubid=1422">
  <section>
   <h1>The Mellblom Browser</h1>
   <p>Web browsing at the speed of light.</p>
   <p>No other browser goes faster!</p>
  </section>
 </a>
</aside>
```

<strong>Example&#58;</strong>

The following example shows how a bit of script can be used to effectively make an entire row in a job listing table a hyperlink&#58;

``` text
<table>
 <tr>
  <th>Position
  <th>Team
  <th>Location
 <tr>
  <td><a href="/jobs/manager">Manager</a>
  <td>Remotees
  <td>Remote
 <tr>
  <td><a href="/jobs/director">Director</a>
  <td>Remotees
  <td>Remote
 <tr>
  <td><a href="/jobs/astronaut">Astronaut</a>
  <td>Architecture
  <td>Remote
</table>
<script>
document.querySelector("table").onclick = ({ target }) => {
  if (target.parentElement.localName === "tr") {
    const link = target.parentElement.querySelector("a");
    if (link) {
      link.click();
    }
  }
}
</script>
```

#### <a id="the&#45;em&#45;element"></a>4&#46;5&#46;2 The <strong><code>em</code></strong> element[](#the-em-element)

<strong>✔</strong>MDN

[Element&#47;em](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/em)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;em&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;em&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;em&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;em&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;em&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;em&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;em&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;em&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;em&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;em&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;em&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;em&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-em)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-em)&#46;

<strong><a id="the&#45;em&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;em&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;em&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;em&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;em&#45;element&#58;the&#45;em&#45;element"></a><code>[em](#the-em-element)</code> element <a id="the&#45;em&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) stress emphasis of its contents&#46;

The level of stress that a particular piece of content has is given by its number of ancestor <a id="the&#45;em&#45;element&#58;the&#45;em&#45;element&#45;2"></a><code>[em](#the-em-element)</code> elements&#46;

The placement of stress emphasis changes the meaning of the sentence&#46; The element thus forms an integral part of the content&#46; The precise way in which stress is used in this way depends on the language&#46;

<strong>Example&#58;</strong>

These examples show how changing the stress emphasis changes the meaning&#46; First&#44; a general statement of fact&#44; with no stress&#58;

``` text
<p>Cats are cute animals.</p>
```

By emphasizing the first word&#44; the statement implies that the kind of animal under discussion is in question &#40;maybe someone is asserting that dogs are cute&#41;&#58;

``` text
<p><em>Cats</em> are cute animals.</p>
```

Moving the stress to the verb&#44; one highlights that the truth of the entire sentence is in question &#40;maybe someone is saying cats are not cute&#41;&#58;

``` text
<p>Cats <em>are</em> cute animals.</p>
```

By moving it to the adjective&#44; the exact nature of the cats is reasserted &#40;maybe someone suggested cats were <em>mean</em> animals&#41;&#58;

``` text
<p>Cats are <em>cute</em> animals.</p>
```

Similarly&#44; if someone asserted that cats were vegetables&#44; someone correcting this might emphasize the last word&#58;

``` text
<p>Cats are cute <em>animals</em>.</p>
```

By emphasizing the entire sentence&#44; it becomes clear that the speaker is fighting hard to get the point across&#46; This kind of stress emphasis also typically affects the punctuation&#44; hence the exclamation mark here&#46;

``` text
<p><em>Cats are cute animals!</em></p>
```

Anger mixed with emphasizing the cuteness could lead to markup such as&#58;

``` text
<p><em>Cats are <em>cute</em> animals!</em></p>
```

<strong>Note&#58;</strong>

The <a id="the&#45;em&#45;element&#58;the&#45;em&#45;element&#45;3"></a><code>[em](#the-em-element)</code> element isn&#39;t a generic &#34;italics&#34; element&#46; Sometimes&#44; text is intended to stand out from the rest of the paragraph&#44; as if it was in a different mood or voice&#46; For this&#44; the <a id="the&#45;em&#45;element&#58;the&#45;i&#45;element"></a><code>[i](#the-i-element)</code> element is more appropriate&#46;

The <a id="the&#45;em&#45;element&#58;the&#45;em&#45;element&#45;4"></a><code>[em](#the-em-element)</code> element also isn&#39;t intended to convey importance&#59; for that purpose&#44; the <a id="the&#45;em&#45;element&#58;the&#45;strong&#45;element"></a><code>[strong](#the-strong-element)</code> element is more appropriate&#46;

#### <a id="the&#45;strong&#45;element"></a>4&#46;5&#46;3 The <strong><code>strong</code></strong> element[](#the-strong-element)

<strong>✔</strong>MDN

[Element&#47;strong](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/strong)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android4&#43;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;strong&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;strong&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;strong&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;strong&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;strong&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;strong&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;strong&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;strong&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;strong&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;strong&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;strong&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;strong&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-strong)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-strong)&#46;

<strong><a id="the&#45;strong&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;strong&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;strong&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;strong&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;strong&#45;element&#58;the&#45;strong&#45;element"></a><code>[strong](#the-strong-element)</code> element <a id="the&#45;strong&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) strong importance&#44; seriousness&#44; or urgency for its contents&#46;

<strong>Importance</strong>&#58; the <a id="the&#45;strong&#45;element&#58;the&#45;strong&#45;element&#45;2"></a><code>[strong](#the-strong-element)</code> element can be used in a heading&#44; caption&#44; or paragraph to distinguish the part that really matters from other parts that might be more detailed&#44; more jovial&#44; or merely boilerplate&#46; &#40;This is distinct from marking up subheadings&#44; for which the <a id="the&#45;strong&#45;element&#58;the&#45;hgroup&#45;element"></a><code>[hgroup](#the-hgroup-element)</code> element is appropriate&#46;&#41;

<strong>Example&#58;</strong>

For example&#44; the first word of the previous paragraph is marked up with <a id="the&#45;strong&#45;element&#58;the&#45;strong&#45;element&#45;3"></a><code>[strong](#the-strong-element)</code> to distinguish it from the more detailed text in the rest of the paragraph&#46;

<strong>Seriousness</strong>&#58; the <a id="the&#45;strong&#45;element&#58;the&#45;strong&#45;element&#45;4"></a><code>[strong](#the-strong-element)</code> element can be used to mark up a warning or caution notice&#46;

<strong>Urgency</strong>&#58; the <a id="the&#45;strong&#45;element&#58;the&#45;strong&#45;element&#45;5"></a><code>[strong](#the-strong-element)</code> element can be used to denote contents that the user needs to see sooner than other parts of the document&#46;

The relative level of importance of a piece of content is given by its number of ancestor <a id="the&#45;strong&#45;element&#58;the&#45;strong&#45;element&#45;6"></a><code>[strong](#the-strong-element)</code> elements&#59; each <a id="the&#45;strong&#45;element&#58;the&#45;strong&#45;element&#45;7"></a><code>[strong](#the-strong-element)</code> element increases the importance of its contents&#46;

Changing the importance of a piece of text with the <a id="the&#45;strong&#45;element&#58;the&#45;strong&#45;element&#45;8"></a><code>[strong](#the-strong-element)</code> element does not change the meaning of the sentence&#46;

<strong>Example&#58;</strong>

Here&#44; the word &#34;chapter&#34; and the actual chapter number are mere boilerplate&#44; and the actual name of the chapter is marked up with <a id="the&#45;strong&#45;element&#58;the&#45;strong&#45;element&#45;9"></a><code>[strong](#the-strong-element)</code>&#58;

``` text
<h1>Chapter 1: <strong>The Praxis</strong></h1>
```

In the following example&#44; the name of the diagram in the caption is marked up with <a id="the&#45;strong&#45;element&#58;the&#45;strong&#45;element&#45;10"></a><code>[strong](#the-strong-element)</code>&#44; to distinguish it from boilerplate text &#40;before&#41; and the description &#40;after&#41;&#58;

``` text
<figcaption>Figure 1. <strong>Ant colony dynamics</strong>. The ants in this colony are
affected by the heat source (upper left) and the food source (lower right).</figcaption>
```

In this example&#44; the heading is really &#34;Flowers&#44; Bees&#44; and Honey&#34;&#44; but the author has added a light&#45;hearted addition to the heading&#46; The <a id="the&#45;strong&#45;element&#58;the&#45;strong&#45;element&#45;11"></a><code>[strong](#the-strong-element)</code> element is thus used to mark up the first part to distinguish it from the latter part&#46;

``` text
<h1><strong>Flowers, Bees, and Honey</strong> and other things I don't understand</h1>
```

<strong>Example&#58;</strong>

Here is an example of a warning notice in a game&#44; with the various parts marked up according to how important they are&#58;

``` text
<p><strong>Warning.</strong> This dungeon is dangerous.
<strong>Avoid the ducks.</strong> Take any gold you find.
<strong><strong>Do not take any of the diamonds</strong>,
they are explosive and <strong>will destroy anything within
ten meters.</strong></strong> You have been warned.</p>
```

<strong>Example&#58;</strong>

In this example&#44; the <a id="the&#45;strong&#45;element&#58;the&#45;strong&#45;element&#45;12"></a><code>[strong](#the-strong-element)</code> element is used to denote the part of the text that the user is intended to read first&#46;

``` text
<p>Welcome to Remy, the reminder system.</p>
<p>Your tasks for today:</p>
<ul>
 <li><p><strong>Turn off the oven.</strong></p></li>
 <li><p>Put out the trash.</p></li>
 <li><p>Do the laundry.</p></li>
</ul>
```

#### <a id="the&#45;small&#45;element"></a>4&#46;5&#46;4 The <strong><code>small</code></strong> element[](#the-small-element)

<strong>✔</strong>MDN

[Element&#47;small](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/small)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;small&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;small&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;small&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;small&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;small&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;small&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;small&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;small&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;small&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;small&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;small&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;small&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-small)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-small)&#46;

<strong><a id="the&#45;small&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;small&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;small&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;small&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;small&#45;element&#58;the&#45;small&#45;element"></a><code>[small](#the-small-element)</code> element <a id="the&#45;small&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) side comments such as small print&#46;

<strong>Note&#58;</strong>

Small print typically features disclaimers&#44; caveats&#44; legal restrictions&#44; or copyrights&#46; Small print is also sometimes used for attribution&#44; or for satisfying licensing requirements&#46;

<strong>Note&#58;</strong>

The <a id="the&#45;small&#45;element&#58;the&#45;small&#45;element&#45;2"></a><code>[small](#the-small-element)</code> element does not &#34;de&#45;emphasize&#34; or lower the importance of text emphasized by the <a id="the&#45;small&#45;element&#58;the&#45;em&#45;element"></a><code>[em](#the-em-element)</code> element or marked as important with the <a id="the&#45;small&#45;element&#58;the&#45;strong&#45;element"></a><code>[strong](#the-strong-element)</code> element&#46; To mark text as not emphasized or important&#44; simply do not mark it up with the <a id="the&#45;small&#45;element&#58;the&#45;em&#45;element&#45;2"></a><code>[em](#the-em-element)</code> or <a id="the&#45;small&#45;element&#58;the&#45;strong&#45;element&#45;2"></a><code>[strong](#the-strong-element)</code> elements respectively&#46;

The <a id="the&#45;small&#45;element&#58;the&#45;small&#45;element&#45;3"></a><code>[small](#the-small-element)</code> element should not be used for extended spans of text&#44; such as multiple paragraphs&#44; lists&#44; or sections of text&#46; It is only intended for short runs of text&#46; The text of a page listing terms of use&#44; for instance&#44; would not be a suitable candidate for the <a id="the&#45;small&#45;element&#58;the&#45;small&#45;element&#45;4"></a><code>[small](#the-small-element)</code> element&#58; in such a case&#44; the text is not a side comment&#44; it is the main content of the page&#46;

The <a id="the&#45;small&#45;element&#58;the&#45;small&#45;element&#45;5"></a><code>[small](#the-small-element)</code> element must not be used for subheadings&#59; for that purpose&#44; use the <a id="the&#45;small&#45;element&#58;the&#45;hgroup&#45;element"></a><code>[hgroup](#the-hgroup-element)</code> element&#46;

<strong>Example&#58;</strong>

In this example&#44; the <a id="the&#45;small&#45;element&#58;the&#45;small&#45;element&#45;6"></a><code>[small](#the-small-element)</code> element is used to indicate that value&#45;added tax is not included in a price of a hotel room&#58;

``` text
<dl>
 <dt>Single room
 <dd>199 € <small>breakfast included, VAT not included</small>
 <dt>Double room
 <dd>239 € <small>breakfast included, VAT not included</small>
</dl>
```

<strong>Example&#58;</strong>

In this second example&#44; the <a id="the&#45;small&#45;element&#58;the&#45;small&#45;element&#45;7"></a><code>[small](#the-small-element)</code> element is used for a side comment in an article&#46;

``` text
<p>Example Corp today announced record profits for the
second quarter <small>(Full Disclosure: Foo News is a subsidiary of
Example Corp)</small>, leading to speculation about a third quarter
merger with Demo Group.</p>
```

This is distinct from a sidebar&#44; which might be multiple paragraphs long and is removed from the main flow of text&#46; In the following example&#44; we see a sidebar from the same article&#46; This sidebar also has small print&#44; indicating the source of the information in the sidebar&#46;

``` text
<aside>
 <h1>Example Corp</h1>
 <p>This company mostly creates small software and Web
 sites.</p>
 <p>The Example Corp company mission is "To provide entertainment
 and news on a sample basis".</p>
 <p><small>Information obtained from <a
 href="https://example.com/about.html">example.com</a> home
 page.</small></p>
</aside>
```

<strong>Example&#58;</strong>

In this last example&#44; the <a id="the&#45;small&#45;element&#58;the&#45;small&#45;element&#45;8"></a><code>[small](#the-small-element)</code> element is marked as being <em>important</em> small print&#46;

``` text
<p><strong><small>Continued use of this service will result in a kiss.</small></strong></p>
```

#### <a id="the&#45;s&#45;element"></a>4&#46;5&#46;5 The <strong><code>s</code></strong> element[](#the-s-element)

<strong>✔</strong>MDN

[Element&#47;s](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/s)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android4&#43;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;s&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;s&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;s&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;s&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;s&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;s&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;s&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;s&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;s&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;s&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;s&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;s&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-s)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-s)&#46;

<strong><a id="the&#45;s&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;s&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;s&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;s&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;s&#45;element&#58;the&#45;s&#45;element"></a><code>[s](#the-s-element)</code> element <a id="the&#45;s&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) contents that are no longer accurate or no longer relevant&#46;

<strong>Note&#58;</strong>

The <a id="the&#45;s&#45;element&#58;the&#45;s&#45;element&#45;2"></a><code>[s](#the-s-element)</code> element is not appropriate when indicating document edits&#59; to mark a span of text as having been removed from a document&#44; use the <a id="the&#45;s&#45;element&#58;the&#45;del&#45;element"></a><code>[del](#the-del-element)</code> element&#46;

<strong>Example&#58;</strong>

In this example a recommended retail price has been marked as no longer relevant as the product in question has a new sale price&#46;

``` text
<p>Buy our Iced Tea and Lemonade!</p>
<p><s>Recommended retail price: $3.99 per bottle</s></p>
<p><strong>Now selling for just $2.99 a bottle!</strong></p>
```

#### <a id="the&#45;cite&#45;element"></a>4&#46;5&#46;6 The <strong><code>cite</code></strong> element[](#the-cite-element)

<strong>✔</strong>MDN

[Element&#47;cite](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/cite)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;cite&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;cite&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;cite&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;cite&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;cite&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;cite&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;cite&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;cite&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;cite&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;cite&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;cite&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;cite&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-cite)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-cite)&#46;

<strong><a id="the&#45;cite&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;cite&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;cite&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;cite&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;cite&#45;element&#58;the&#45;cite&#45;element"></a><code>[cite](#the-cite-element)</code> element <a id="the&#45;cite&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) the title of a work &#40;e&#46;g&#46; a book&#44; a paper&#44; an essay&#44; a poem&#44; a score&#44; a song&#44; a script&#44; a film&#44; a TV show&#44; a game&#44; a sculpture&#44; a painting&#44; a theatre production&#44; a play&#44; an opera&#44; a musical&#44; an exhibition&#44; a legal case report&#44; a computer program&#44; etc&#46;&#41;&#46; This can be a work that is being quoted or <a id="the&#45;cite&#45;element&#58;referenced"></a>[referenced](html-language-foundations--877d6146487f--b55e73c31978.md#referenced) in detail &#40;i&#46;e&#46;&#44; a citation&#41;&#44; or it can just be a work that is mentioned in passing&#46;

A person&#39;s name is not the title of a work — even if people call that person a piece of work — and the element must therefore not be used to mark up people&#39;s names&#46; &#40;In some cases&#44; the <a id="the&#45;cite&#45;element&#58;the&#45;b&#45;element"></a><code>[b](#the-b-element)</code> element might be appropriate for names&#59; e&#46;g&#46; in a gossip article where the names of famous people are keywords rendered with a different style to draw attention to them&#46; In other cases&#44; if an element is <em>really</em> needed&#44; the <a id="the&#45;cite&#45;element&#58;the&#45;span&#45;element"></a><code>[span](#the-span-element)</code> element can be used&#46;&#41;

<a id="example&#45;cite&#45;usage"></a>

<strong>Example&#58;</strong>

[](#example-cite-usage)

This next example shows a typical use of the <a id="the&#45;cite&#45;element&#58;the&#45;cite&#45;element&#45;2"></a><code>[cite](#the-cite-element)</code> element&#58;

``` text
<p>My favorite book is <cite>The Reality Dysfunction</cite> by
Peter F. Hamilton. My favorite comic is <cite>Pearls Before
Swine</cite> by Stephan Pastis. My favorite track is <cite>Jive
Samba</cite> by the Cannonball Adderley Sextet.</p>
```

<a id="example&#45;cite&#45;usage&#45;correct&#45;vs&#45;incorrect"></a>

<strong>Example&#58;</strong>

[](#example-cite-usage-correct-vs-incorrect)

This is correct usage&#58;

``` text
<p>According to the Wikipedia article <cite>HTML</cite>, as it
stood in mid-February 2008, leaving attribute values unquoted is
unsafe. This is obviously an over-simplification.</p>
```

The following&#44; however&#44; is incorrect usage&#44; as the <a id="the&#45;cite&#45;element&#58;the&#45;cite&#45;element&#45;3"></a><code>[cite](#the-cite-element)</code> element here is containing far more than the title of the work&#58;

``` text
<!-- do not copy this example, it is an example of bad usage! -->
<p>According to <cite>the Wikipedia article on HTML</cite>, as it
stood in mid-February 2008, leaving attribute values unquoted is
unsafe. This is obviously an over-simplification.</p>
```

<a id="example&#45;cite&#45;usage&#45;bibliography"></a>

<strong>Example&#58;</strong>

[](#example-cite-usage-bibliography)

The <a id="the&#45;cite&#45;element&#58;the&#45;cite&#45;element&#45;4"></a><code>[cite](#the-cite-element)</code> element is a key part of any citation in a bibliography&#44; but it is only used to mark the title&#58;

``` text
<p><cite>Universal Declaration of Human Rights</cite>, United Nations,
December 1948. Adopted by General Assembly resolution 217 A (III).</p>
```

<a id="note&#45;cite&#45;is&#45;not&#45;q"></a>

<strong>Note&#58;</strong>

[](#note-cite-is-not-q)A <em>citation</em> is not a <em>quote</em> &#40;for which the <a id="the&#45;cite&#45;element&#58;the&#45;q&#45;element"></a><code>[q](#the-q-element)</code> element is appropriate&#41;&#46;

<a id="example&#45;cite&#45;usage&#45;incorrect"></a>

<strong>Example&#58;</strong>

[](#example-cite-usage-incorrect)

This is incorrect usage&#44; because <a id="the&#45;cite&#45;element&#58;the&#45;cite&#45;element&#45;5"></a><code>[cite](#the-cite-element)</code> is not for quotes&#58;

``` text
<p><cite>This is wrong!</cite>, said Ian.</p>
```

This is also incorrect usage&#44; because a person is not a work&#58;

``` text
<p><q>This is still wrong!</q>, said <cite>Ian</cite>.</p>
```

The correct usage does not use a <a id="the&#45;cite&#45;element&#58;the&#45;cite&#45;element&#45;6"></a><code>[cite](#the-cite-element)</code> element&#58;

``` text
<p><q>This is correct</q>, said Ian.</p>
```

As mentioned above&#44; the <a id="the&#45;cite&#45;element&#58;the&#45;b&#45;element&#45;2"></a><code>[b](#the-b-element)</code> element might be relevant for marking names as being keywords in certain kinds of documents&#58;

``` text
<p>And then <b>Ian</b> said <q>this might be right, in a
gossip column, maybe!</q>.</p>
```

<a id="note&#45;cite&#45;surrounding&#45;punctuation"></a>

<strong>Note&#58;</strong>

[](#note-cite-surrounding-punctuation)Where the title of a work is conventionally set off with punctuation — for example&#44; the quotation marks used around article titles in English&#44; or the corner brackets &#40;『』&#41; used around book titles in Japanese — that punctuation is not part of the title&#44; and belongs outside the <a id="the&#45;cite&#45;element&#58;the&#45;cite&#45;element&#45;7"></a><code>[cite](#the-cite-element)</code> element&#46;

<a id="example&#45;cite&#45;surrounding&#45;punctuation"></a>

<strong>Example&#58;</strong>

[](#example-cite-surrounding-punctuation)

This is correct usage&#58;

``` text
<p>My review of 『<cite lang="ja">阪急電車</cite>』 was titled “<cite>A cute series of short stories</cite>”.</p>
```

#### <a id="the&#45;q&#45;element"></a>4&#46;5&#46;7 The <strong><code>q</code></strong> element[](#the-q-element)

<strong>✔</strong>MDN

[Element&#47;q](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/q)

Support in all current engines&#46;

Firefox1&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;q&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;q&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;q&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;q&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;q&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;q&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;q&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;q&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;q&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;q&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;q&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<a id="the&#45;q&#45;element&#58;attr&#45;q&#45;cite"></a><code>[cite](#attr-q-cite)</code> — Link to the source of the quotation or more information about the edit

<strong><a id="the&#45;q&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-q)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-q)&#46;

<strong><a id="the&#45;q&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;q&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;q&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;q&#45;element&#58;htmlquoteelement"></a><code>[HTMLQuoteElement](#htmlquoteelement)</code>&#46;

The <a id="the&#45;q&#45;element&#58;the&#45;q&#45;element"></a><code>[q](#the-q-element)</code> element <a id="the&#45;q&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) some <a id="the&#45;q&#45;element&#58;phrasing&#45;content&#45;2&#45;4"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) quoted from another source&#46;

Quotation punctuation &#40;such as quotation marks&#41; that is quoting the contents of the element must not appear immediately before&#44; after&#44; or inside <a id="the&#45;q&#45;element&#58;the&#45;q&#45;element&#45;2"></a><code>[q](#the-q-element)</code> elements&#59; they will be inserted into the rendering by the user agent&#46;

Content inside a <a id="the&#45;q&#45;element&#58;the&#45;q&#45;element&#45;3"></a><code>[q](#the-q-element)</code> element must be quoted from another source&#44; whose address&#44; if it has one&#44; may be cited in the <a id="attr&#45;q&#45;cite"></a><strong><code>cite</code></strong> attribute&#46; The source may be fictional&#44; as when quoting characters in a novel or screenplay&#46;

If the <a id="the&#45;q&#45;element&#58;attr&#45;q&#45;cite&#45;2"></a><code>[cite](#attr-q-cite)</code> attribute is present&#44; it must be a <a id="the&#45;q&#45;element&#58;valid&#45;url&#45;potentially&#45;surrounded&#45;by&#45;spaces"></a>[valid URL potentially surrounded by spaces](html-language-foundations--877d6146487f--b55e73c31978.md#valid-url-potentially-surrounded-by-spaces)&#46; To obtain the corresponding citation link&#44; the value of the attribute must be <a id="the&#45;q&#45;element&#58;encoding&#45;parsing&#45;a&#45;url"></a>[parsed](html-language-foundations--877d6146487f--b55e73c31978.md#encoding-parsing-a-url) relative to the element&#39;s <a id="the&#45;q&#45;element&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46; User agents may allow users to follow such citation links&#44; but they are primarily intended for private use &#40;e&#46;g&#46;&#44; by server&#45;side scripts collecting statistics about a site&#39;s use of quotations&#41;&#44; not for readers&#46;

The <a id="the&#45;q&#45;element&#58;the&#45;q&#45;element&#45;4"></a><code>[q](#the-q-element)</code> element must not be used in place of quotation marks that do not represent quotes&#59; for example&#44; it is inappropriate to use the <a id="the&#45;q&#45;element&#58;the&#45;q&#45;element&#45;5"></a><code>[q](#the-q-element)</code> element for marking up sarcastic statements&#46;

The use of <a id="the&#45;q&#45;element&#58;the&#45;q&#45;element&#45;6"></a><code>[q](#the-q-element)</code> elements to mark up quotations is entirely optional&#59; using explicit quotation punctuation without <a id="the&#45;q&#45;element&#58;the&#45;q&#45;element&#45;7"></a><code>[q](#the-q-element)</code> elements is just as correct&#46;

<strong>Example&#58;</strong>

Here is a simple example of the use of the <a id="the&#45;q&#45;element&#58;the&#45;q&#45;element&#45;8"></a><code>[q](#the-q-element)</code> element&#58;

``` text
<p>The man said <q>Things that are impossible just take
longer</q>. I disagreed with him.</p>
```

<strong>Example&#58;</strong>

Here is an example with both an explicit citation link in the <a id="the&#45;q&#45;element&#58;the&#45;q&#45;element&#45;9"></a><code>[q](#the-q-element)</code> element&#44; and an explicit citation outside&#58;

``` text
<p>The W3C page <cite>About W3C</cite> says the W3C's
mission is <q cite="https://www.w3.org/Consortium/">To lead the
World Wide Web to its full potential by developing protocols and
guidelines that ensure long-term growth for the Web</q>. I
disagree with this mission.</p>
```

<strong>Example&#58;</strong>

In the following example&#44; the quotation itself contains a quotation&#58;

``` text
<p>In <cite>Example One</cite>, he writes <q>The man
said <q>Things that are impossible just take longer</q>. I
disagreed with him</q>. Well, I disagree even more!</p>
```

<strong>Example&#58;</strong>

In the following example&#44; quotation marks are used instead of the <a id="the&#45;q&#45;element&#58;the&#45;q&#45;element&#45;10"></a><code>[q](#the-q-element)</code> element&#58;

``` text
<p>His best argument was ❝I disagree❞, which
I thought was laughable.</p>
```

<strong>Example&#58;</strong>

In the following example&#44; there is no quote — the quotation marks are used to name a word&#46; Use of the <a id="the&#45;q&#45;element&#58;the&#45;q&#45;element&#45;11"></a><code>[q](#the-q-element)</code> element in this case would be inappropriate&#46;

``` text
<p>The word "ineffable" could have been used to describe the disaster
resulting from the campaign's mismanagement.</p>
```

#### <a id="the&#45;dfn&#45;element"></a>4&#46;5&#46;8 The <strong><code>dfn</code></strong> element[](#the-dfn-element)

<strong>✔</strong>MDN

[Element&#47;dfn](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dfn)

Support in all current engines&#46;

Firefox1&#43;Safari6&#43;Chrome15&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;dfn&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;dfn&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;dfn&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;dfn&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;dfn&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;dfn&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;dfn&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;dfn&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#44; but there must be no <a id="the&#45;dfn&#45;element&#58;the&#45;dfn&#45;element"></a><code>[dfn](#the-dfn-element)</code> element descendants&#46;

<strong><a id="the&#45;dfn&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;dfn&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;dfn&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

Also&#44; the <a id="the&#45;dfn&#45;element&#58;attr&#45;dfn&#45;title"></a><code>[title](#attr-dfn-title)</code> attribute <a id="the&#45;dfn&#45;element&#58;attr&#45;dfn&#45;title&#45;2"></a>[has special semantics](#attr-dfn-title) on this element&#58; Full term or expansion of abbreviation

<strong><a id="the&#45;dfn&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-dfn)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-dfn)&#46;

<strong><a id="the&#45;dfn&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;dfn&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;dfn&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;dfn&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;dfn&#45;element&#58;the&#45;dfn&#45;element&#45;2"></a><code>[dfn](#the-dfn-element)</code> element <a id="the&#45;dfn&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) the defining instance of a term&#46; The <a id="the&#45;dfn&#45;element&#58;paragraph"></a>[paragraph](html-language-foundations--877d6146487f--b55e73c31978.md#paragraph)&#44; <a id="the&#45;dfn&#45;element&#58;the&#45;dl&#45;element"></a>[description list group](#the-dl-element)&#44; or <a id="the&#45;dfn&#45;element&#58;sectioning&#45;content&#45;2"></a>[section](html-language-foundations--877d6146487f--b55e73c31978.md#sectioning-content-2) that is the nearest ancestor of the <a id="the&#45;dfn&#45;element&#58;the&#45;dfn&#45;element&#45;3"></a><code>[dfn](#the-dfn-element)</code> element must also contain the definition&#40;s&#41; for the <a id="the&#45;dfn&#45;element&#58;defining&#45;term"></a>[term](#defining-term) given by the <a id="the&#45;dfn&#45;element&#58;the&#45;dfn&#45;element&#45;4"></a><code>[dfn](#the-dfn-element)</code> element&#46;

<a id="defining&#45;term"></a><strong>Defining term</strong>&#58; if the <a id="the&#45;dfn&#45;element&#58;the&#45;dfn&#45;element&#45;5"></a><code>[dfn](#the-dfn-element)</code> element has a <a id="attr&#45;dfn&#45;title"></a><strong><code>title</code></strong> attribute&#44; then the exact value of that attribute is the term being defined&#46; Otherwise&#44; if it contains exactly one element child node and no child <a id="the&#45;dfn&#45;element&#58;text"></a><code>[Text](https://dom.spec.whatwg.org/#interface-text)</code> nodes&#44; and that child element is an <a id="the&#45;dfn&#45;element&#58;the&#45;abbr&#45;element"></a><code>[abbr](#the-abbr-element)</code> element with a <a id="the&#45;dfn&#45;element&#58;attr&#45;abbr&#45;title"></a><code>[title](#attr-abbr-title)</code> attribute&#44; then the exact value of <em>that</em> attribute is the term being defined&#46; Otherwise&#44; it is the <a id="the&#45;dfn&#45;element&#58;descendant&#45;text&#45;content"></a>[descendant text content](https://dom.spec.whatwg.org/#concept-descendant-text-content) of the <a id="the&#45;dfn&#45;element&#58;the&#45;dfn&#45;element&#45;6"></a><code>[dfn](#the-dfn-element)</code> element that gives the term being defined&#46;

If the <a id="the&#45;dfn&#45;element&#58;attr&#45;dfn&#45;title&#45;3"></a><code>[title](#attr-dfn-title)</code> attribute of the <a id="the&#45;dfn&#45;element&#58;the&#45;dfn&#45;element&#45;7"></a><code>[dfn](#the-dfn-element)</code> element is present&#44; then it must contain only the term being defined&#46;

<strong>Note&#58;</strong>

The <a id="the&#45;dfn&#45;element&#58;attr&#45;title"></a><code>[title](html-language-foundations--877d6146487f--b55e73c31978.md#attr-title)</code> attribute of ancestor elements does not affect <a id="the&#45;dfn&#45;element&#58;the&#45;dfn&#45;element&#45;8"></a><code>[dfn](#the-dfn-element)</code> elements&#46;

An <a id="the&#45;dfn&#45;element&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code> element that links to a <a id="the&#45;dfn&#45;element&#58;the&#45;dfn&#45;element&#45;9"></a><code>[dfn](#the-dfn-element)</code> element represents an instance of the term defined by the <a id="the&#45;dfn&#45;element&#58;the&#45;dfn&#45;element&#45;10"></a><code>[dfn](#the-dfn-element)</code> element&#46;

<strong>Example&#58;</strong>

In the following fragment&#44; the term &#34;Garage Door Opener&#34; is first defined in the first paragraph&#44; then used in the second&#46; In both cases&#44; its abbreviation is what is actually displayed&#46;

``` text
<p>The <dfn><abbr title="Garage Door Opener">GDO</abbr></dfn>
is a device that allows off-world teams to open the iris.</p>
<!-- ... later in the document: -->
<p>Teal'c activated his <abbr title="Garage Door Opener">GDO</abbr>
and so Hammond ordered the iris to be opened.</p>
```

With the addition of an <a id="the&#45;dfn&#45;element&#58;the&#45;a&#45;element&#45;2"></a><code>[a](#the-a-element)</code> element&#44; the <a id="the&#45;dfn&#45;element&#58;referenced"></a>[reference](html-language-foundations--877d6146487f--b55e73c31978.md#referenced) can be made explicit&#58;

``` text
<p>The <dfn id=gdo><abbr title="Garage Door Opener">GDO</abbr></dfn>
is a device that allows off-world teams to open the iris.</p>
<!-- ... later in the document: -->
<p>Teal'c activated his <a href=#gdo><abbr title="Garage Door Opener">GDO</abbr></a>
and so Hammond ordered the iris to be opened.</p>
```

#### <a id="the&#45;abbr&#45;element"></a>4&#46;5&#46;9 The <strong><code>abbr</code></strong> element[](#the-abbr-element)

<strong>✔</strong>MDN

[Element&#47;abbr](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/abbr)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome2&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer7&#43;

------------------------------------------------------------------------

Firefox Android4&#43;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;abbr&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;abbr&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;abbr&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;abbr&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;abbr&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;abbr&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;abbr&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;abbr&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;abbr&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;abbr&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;abbr&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

Also&#44; the <a id="the&#45;abbr&#45;element&#58;attr&#45;abbr&#45;title"></a><code>[title](#attr-abbr-title)</code> attribute <a id="the&#45;abbr&#45;element&#58;attr&#45;abbr&#45;title&#45;2"></a>[has special semantics](#attr-abbr-title) on this element&#58; Full term or expansion of abbreviation

<strong><a id="the&#45;abbr&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-abbr)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-abbr)&#46;

<strong><a id="the&#45;abbr&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;abbr&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;abbr&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;abbr&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;abbr&#45;element&#58;the&#45;abbr&#45;element"></a><code>[abbr](#the-abbr-element)</code> element <a id="the&#45;abbr&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) an abbreviation or acronym&#44; optionally with its expansion&#46; The <a id="attr&#45;abbr&#45;title"></a><strong><code>title</code></strong> attribute may be used to provide an expansion of the abbreviation&#46; The attribute&#44; if specified&#44; must contain an expansion of the abbreviation&#44; and nothing else&#46;

<strong>Example&#58;</strong>

The paragraph below contains an abbreviation marked up with the <a id="the&#45;abbr&#45;element&#58;the&#45;abbr&#45;element&#45;2"></a><code>[abbr](#the-abbr-element)</code> element&#46; This paragraph <a id="the&#45;abbr&#45;element&#58;defining&#45;term"></a>[defines the term](#defining-term) &#34;Web Hypertext Application Technology Working Group&#34;&#46;

``` text
<p>The <dfn id=whatwg><abbr
title="Web Hypertext Application Technology Working Group">WHATWG</abbr></dfn>
is a loose unofficial collaboration of web browser manufacturers and
interested parties who wish to develop new technologies designed to
allow authors to write and deploy Applications over the World Wide
Web.</p>
```

An alternative way to write this would be&#58;

``` text
<p>The <dfn id=whatwg>Web Hypertext Application Technology
Working Group</dfn> (<abbr
title="Web Hypertext Application Technology Working Group">WHATWG</abbr>)
is a loose unofficial collaboration of web browser manufacturers and
interested parties who wish to develop new technologies designed to
allow authors to write and deploy Applications over the World Wide
Web.</p>
```

<strong>Example&#58;</strong>

This paragraph has two abbreviations&#46; Notice how only one is defined&#59; the other&#44; with no expansion associated with it&#44; does not use the <a id="the&#45;abbr&#45;element&#58;the&#45;abbr&#45;element&#45;3"></a><code>[abbr](#the-abbr-element)</code> element&#46;

``` text
<p>The
<abbr title="Web Hypertext Application Technology Working Group">WHATWG</abbr>
started working on HTML5 in 2004.</p>
```

<strong>Example&#58;</strong>

This paragraph links an abbreviation to its definition&#46;

``` text
<p>The <a href="#whatwg"><abbr
title="Web Hypertext Application Technology Working Group">WHATWG</abbr></a>
community does not have much representation from Asia.</p>
```

<strong>Example&#58;</strong>

This paragraph marks up an abbreviation without giving an expansion&#44; possibly as a hook to apply styles for abbreviations &#40;e&#46;g&#46; smallcaps&#41;&#46;

``` text
<p>Philip` and Dashiva both denied that they were going to
get the issue counts from past revisions of the specification to
backfill the <abbr>WHATWG</abbr> issue graph.</p>
```

If an abbreviation is pluralized&#44; the expansion&#39;s grammatical number &#40;plural vs singular&#41; must match the grammatical number of the contents of the element&#46;

<strong>Example&#58;</strong>

Here the plural is outside the element&#44; so the expansion is in the singular&#58;

``` text
<p>Two <abbr title="Working Group">WG</abbr>s worked on
this specification: the <abbr>WHATWG</abbr> and the
<abbr>HTMLWG</abbr>.</p>
```

Here the plural is inside the element&#44; so the expansion is in the plural&#58;

``` text
<p>Two <abbr title="Working Groups">WGs</abbr> worked on
this specification: the <abbr>WHATWG</abbr> and the
<abbr>HTMLWG</abbr>.</p>
```

Abbreviations do not have to be marked up using this element&#46; It is expected to be useful in the following cases&#58;

- Abbreviations for which the author wants to give expansions&#44; where using the <a id="the&#45;abbr&#45;element&#58;the&#45;abbr&#45;element&#45;4"></a><code>[abbr](#the-abbr-element)</code> element with a <a id="the&#45;abbr&#45;element&#58;attr&#45;title"></a><code>[title](html-language-foundations--877d6146487f--b55e73c31978.md#attr-title)</code> attribute is an alternative to including the expansion inline &#40;e&#46;g&#46; in parentheses&#41;&#46;
- Abbreviations that are likely to be unfamiliar to the document&#39;s readers&#44; for which authors are encouraged to either mark up the abbreviation using an <a id="the&#45;abbr&#45;element&#58;the&#45;abbr&#45;element&#45;5"></a><code>[abbr](#the-abbr-element)</code> element with a <a id="the&#45;abbr&#45;element&#58;attr&#45;title&#45;2"></a><code>[title](html-language-foundations--877d6146487f--b55e73c31978.md#attr-title)</code> attribute or include the expansion inline in the text the first time the abbreviation is used&#46;
- Abbreviations whose presence needs to be semantically annotated&#44; e&#46;g&#46; so that they can be identified from a style sheet and given specific styles&#44; for which the <a id="the&#45;abbr&#45;element&#58;the&#45;abbr&#45;element&#45;6"></a><code>[abbr](#the-abbr-element)</code> element can be used without a <a id="the&#45;abbr&#45;element&#58;attr&#45;title&#45;3"></a><code>[title](html-language-foundations--877d6146487f--b55e73c31978.md#attr-title)</code> attribute&#46;

<strong>Note&#58;</strong>

Providing an expansion in a <a id="the&#45;abbr&#45;element&#58;attr&#45;title&#45;4"></a><code>[title](html-language-foundations--877d6146487f--b55e73c31978.md#attr-title)</code> attribute once will not necessarily cause other <a id="the&#45;abbr&#45;element&#58;the&#45;abbr&#45;element&#45;7"></a><code>[abbr](#the-abbr-element)</code> elements in the same document with the same contents but without a <a id="the&#45;abbr&#45;element&#58;attr&#45;title&#45;5"></a><code>[title](html-language-foundations--877d6146487f--b55e73c31978.md#attr-title)</code> attribute to behave as if they had the same expansion&#46; Every <a id="the&#45;abbr&#45;element&#58;the&#45;abbr&#45;element&#45;8"></a><code>[abbr](#the-abbr-element)</code> element is independent&#46;

#### <a id="the&#45;ruby&#45;element"></a>4&#46;5&#46;10 The <strong><code>ruby</code></strong> element[](#the-ruby-element)

<strong>✔</strong>MDN

[Element&#47;ruby](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ruby)

Support in all current engines&#46;

Firefox38&#43;Safari5&#43;Chrome5&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;ruby&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;ruby&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;ruby&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;ruby&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;ruby&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;ruby&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;ruby&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

See prose&#46;

<strong><a id="the&#45;ruby&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;ruby&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;ruby&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;ruby&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-ruby)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-ruby)&#46;

<strong><a id="the&#45;ruby&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;ruby&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;ruby&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;ruby&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element"></a><code>[ruby](#the-ruby-element)</code> element allows one or more spans of phrasing content to be marked with ruby annotations&#46; Ruby annotations are short runs of text presented alongside base text&#44; primarily used in East Asian typography as a guide for pronunciation or to include other annotations&#46; In Japanese&#44; this form of typography is also known as <em>furigana</em>&#46;

The content model of <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;2"></a><code>[ruby](#the-ruby-element)</code> elements consists of one or more of the following sequences&#58;

1.  One or the other of the following&#58;

    - <a id="the&#45;ruby&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#44; but with no <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;3"></a><code>[ruby](#the-ruby-element)</code> elements and with no <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;4"></a><code>[ruby](#the-ruby-element)</code> element descendants

    - A single <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;5"></a><code>[ruby](#the-ruby-element)</code> element that itself has no <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;6"></a><code>[ruby](#the-ruby-element)</code> element descendants

2.  One or the other of the following&#58;

    - One or more <a id="the&#45;ruby&#45;element&#58;the&#45;rt&#45;element"></a><code>[rt](#the-rt-element)</code> elements

    - An <a id="the&#45;ruby&#45;element&#58;the&#45;rp&#45;element"></a><code>[rp](#the-rp-element)</code> element followed by one or more <a id="the&#45;ruby&#45;element&#58;the&#45;rt&#45;element&#45;2"></a><code>[rt](#the-rt-element)</code> elements&#44; each of which is itself followed by an <a id="the&#45;ruby&#45;element&#58;the&#45;rp&#45;element&#45;2"></a><code>[rp](#the-rp-element)</code> element

The <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;7"></a><code>[ruby](#the-ruby-element)</code> and <a id="the&#45;ruby&#45;element&#58;the&#45;rt&#45;element&#45;3"></a><code>[rt](#the-rt-element)</code> elements can be used for a variety of kinds of annotations&#44; including in particular &#40;though by no means limited to&#41; those described below&#46; For more details on Japanese Ruby in particular&#44; and how to render Ruby for Japanese&#44; see Requirements for Japanese Text Layout&#46; [&#91;JLREQ&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsJLREQ)

<strong>Note&#58;</strong>

At the time of writing&#44; CSS does not yet provide a way to fully control the rendering of the HTML <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;8"></a><code>[ruby](#the-ruby-element)</code> element&#46; It is hoped that CSS will be extended to support the styles described below in due course&#46;

<strong>Mono&#45;ruby for individual base characters in Japanese</strong>

One or more hiragana or katakana characters &#40;the ruby annotation&#41; are placed with each ideographic character &#40;the base text&#41;&#46; This is used to provide readings of kanji characters&#46;

<strong>Example&#58;</strong>

``` text
<ruby>B<rt>annotation</ruby>
```

<strong>Example&#58;</strong>

In this example&#44; notice how each annotation corresponds to a single base character&#46;

``` text
<ruby>君<rt>くん</ruby><ruby>子<rt>し</ruby>は<ruby>和<rt>わ</ruby>して<ruby>同<rt>どう</ruby>ぜず。
```

君くん子しは和わして同どうぜず。

This example can also be written as follows&#44; using one <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;9"></a><code>[ruby](#the-ruby-element)</code> element with two segments of base text and two annotations &#40;one for each&#41; rather than two back&#45;to&#45;back <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;10"></a><code>[ruby](#the-ruby-element)</code> elements each with one base text segment and annotation &#40;as in the markup above&#41;&#58;

``` text
<ruby>君<rt>くん</rt>子<rt>し</ruby>は<ruby>和<rt>わ</ruby>して<ruby>同<rt>どう</ruby>ぜず。
```

<strong>Mono&#45;ruby for compound words &#40;jukugo&#41;</strong>

This is similar to the previous case&#58; each ideographic character in the compound word &#40;the base text&#41; has its reading given in hiragana or katakana characters &#40;the ruby annotation&#41;&#46; The difference is that the base text segments form a compound word rather than being separate from each other&#46;

<strong>Example&#58;</strong>

``` text
<ruby>B<rt>annotation</rt>B<rt>annotation</ruby>
```

<strong>Example&#58;</strong>

In this example&#44; notice again how each annotation corresponds to a single base character&#46; In this example&#44; each compound word &#40;jukugo&#41; corresponds to a single <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;11"></a><code>[ruby](#the-ruby-element)</code> element&#46;

The rendering here is expected to be that each annotation be placed over &#40;or next to&#44; in vertical text&#41; the corresponding base character&#44; with the annotations not overhanging any of the adjacent characters&#46;

``` text
<ruby>鬼<rt>き</rt>門<rt>もん</rt></ruby>の<ruby>方<rt>ほう</rt>角<rt>がく</rt></ruby>を<ruby>凝<rt>ぎょう</rt>視<rt>し</rt></ruby>する
```

鬼き門もんの方ほう角がくを凝ぎょう視しする

<strong>Jukugo&#45;ruby</strong>

This is semantically identical to the previous case &#40;each individual ideographic character in the base compound word has its reading given in an annotation in hiragana or katakana characters&#41;&#44; but the rendering is the more complicated Jukugo Ruby rendering&#46;

<strong>Example&#58;</strong>

This is the same example as above for mono&#45;ruby for compound words&#46; The different rendering is expected to be achieved using different styling &#40;e&#46;g&#46; in CSS&#41;&#44; and is not shown here&#46;

``` text
<ruby>鬼<rt>き</rt>門<rt>もん</rt></ruby>の<ruby>方<rt>ほう</rt>角<rt>がく</rt></ruby>を<ruby>凝<rt>ぎょう</rt>視<rt>し</rt></ruby>する
```

<strong>Note&#58;</strong>

For more details on [Jukugo Ruby rendering](https://www.w3.org/TR/jlreq/#positioning_of_jukugoruby)&#44; see Appendix F in the Requirements for Japanese Text Layout&#46; [&#91;JLREQ&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsJLREQ)

<strong>Group ruby for describing meanings</strong>

The annotation describes the meaning of the base text&#44; rather than &#40;or in addition to&#41; the pronunciation&#46; As such&#44; both the base text and the annotation can be multiple characters long&#46;

<strong>Example&#58;</strong>

``` text
<ruby>BASE<rt>annotation</ruby>
```

<strong>Example&#58;</strong>

Here a compound ideographic word has its corresponding katakana given as an annotation&#46;

``` text
<ruby>境界面<rt>インターフェース</ruby>
```

境界面インターフェース

<strong>Example&#58;</strong>

Here a compound ideographic word has its translation in English provided as an annotation&#46;

``` text
<ruby lang="ja">編集者<rt lang="en">editor</ruby>
```

編集者editor

<strong>Group ruby for Jukuji readings</strong>

A phonetic reading that corresponds to multiple base characters&#44; because a one&#45;to&#45;one mapping would be difficult&#46; &#40;In English&#44; the words &#34;Colonel&#34; and &#34;Lieutenant&#34; are examples of words where a direct mapping of pronunciation to individual letters is&#44; in some dialects&#44; rather unclear&#46;&#41;

<strong>Example&#58;</strong>

In this example&#44; the name of a species of flowers has a phonetic reading provided using group ruby&#58;

``` text
<ruby>紫陽花<rt>あじさい</ruby>
```

紫陽花あじさい

<strong>Text with both phonetic and semantic annotations &#40;double&#45;sided ruby&#41;</strong>

Sometimes&#44; ruby styles described above are combined&#46;

If this results in two annotations covering the same single base segment&#44; then the annotations can just be placed back to back&#46;

<strong>Example&#58;</strong>

``` text
<ruby>BASE<rt>annotation 1<rt>annotation 2</ruby>
```

<strong>Example&#58;</strong>

``` text
<ruby>B<rt>a<rt>a</ruby><ruby>A<rt>a<rt>a</ruby><ruby>S<rt>a<rt>a</ruby><ruby>E<rt>a<rt>a</ruby>
```

<strong>Example&#58;</strong>

In this contrived example&#44; some symbols are given names in English and French&#46;

``` text
<ruby>
 ♥ <rt> Heart <rt lang=fr> Cœur </rt>
 ☘ <rt> Shamrock <rt lang=fr> Trèfle </rt>
 ✶ <rt> Star <rt lang=fr> Étoile </rt>
</ruby>
```

In more complicated situations such as the following examples&#44; a nested <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;12"></a><code>[ruby](#the-ruby-element)</code> element is used to give the inner annotations&#44; and then that whole <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;13"></a><code>[ruby](#the-ruby-element)</code> is then given an annotation at the &#34;outer&#34; level&#46;

<strong>Example&#58;</strong>

``` text
<ruby><ruby>B<rt>a</rt>A<rt>n</rt>S<rt>t</rt>E<rt>n</rt></ruby><rt>annotation</ruby>
```

<strong>Example&#58;</strong>

Here both a phonetic reading and the meaning are given in ruby annotations&#46; The annotation on the nested <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;14"></a><code>[ruby](#the-ruby-element)</code> element gives a mono&#45;ruby phonetic annotation for each base character&#44; while the annotation in the <a id="the&#45;ruby&#45;element&#58;the&#45;rt&#45;element&#45;4"></a><code>[rt](#the-rt-element)</code> element that is a child of the outer <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;15"></a><code>[ruby](#the-ruby-element)</code> element gives the meaning using hiragana&#46;

``` text
<ruby><ruby>東<rt>とう</rt>南<rt>なん</rt></ruby><rt>たつみ</rt></ruby>の方角
```

東とう南なんたつみの方角

<strong>Example&#58;</strong>

This is the same example&#44; but the meaning is given in English instead of Japanese&#58;

``` text
<ruby><ruby>東<rt>とう</rt>南<rt>なん</rt></ruby><rt lang=en>Southeast</rt></ruby>の方角
```

東とう南なんSoutheastの方角

------------------------------------------------------------------------

Within a <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;16"></a><code>[ruby](#the-ruby-element)</code> element that does not have a <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;17"></a><code>[ruby](#the-ruby-element)</code> element ancestor&#44; content is segmented and segments are placed into three categories&#58; base text segments&#44; annotation segments&#44; and ignored segments&#46; Ignored segments do not form part of the document&#39;s semantics &#40;they consist of some <a id="the&#45;ruby&#45;element&#58;inter&#45;element&#45;whitespace"></a>[inter&#45;element whitespace](html-language-foundations--877d6146487f--b55e73c31978.md#inter-element-whitespace) and <a id="the&#45;ruby&#45;element&#58;the&#45;rp&#45;element&#45;3"></a><code>[rp](#the-rp-element)</code> elements&#44; the latter of which are used for legacy user agents that do not support ruby at all&#41;&#46; Base text segments can overlap &#40;with a limit of two segments overlapping any one position in the DOM&#44; and with any segment having an earlier start point than an overlapping segment also having an equal or later end point&#44; and any segment have a later end point than an overlapping segment also having an equal or earlier start point&#41;&#46; Annotation segments correspond to <a id="the&#45;ruby&#45;element&#58;the&#45;rt&#45;element&#45;5"></a><code>[rt](#the-rt-element)</code> elements&#46; Each annotation segment can be associated with a base text segment&#44; and each base text segment can have annotation segments associated with it&#46; &#40;In a conforming document&#44; each base text segment is associated with at least one annotation segment&#44; and each annotation segment is associated with one base text segment&#46;&#41; A <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;18"></a><code>[ruby](#the-ruby-element)</code> element <a id="the&#45;ruby&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) the union of the segments of base text it contains&#44; along with the mapping from those base text segments to annotation segments&#46; Segments are described in terms of DOM ranges&#59; annotation segment ranges always consist of exactly one element&#46; [&#91;DOM&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsDOM)

At any particular time&#44; the segmentation and categorization of content of a <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;19"></a><code>[ruby](#the-ruby-element)</code> element is the result that would be obtained from running the following algorithm&#58;

1.  Let <var>base text segments</var> be an empty list of base text segments&#44; each potentially with a list of base text subsegments&#46;

2.  Let <var>annotation segments</var> be an empty list of annotation segments&#44; each potentially being associated with a base text segment or subsegment&#46;

3.  Let <var>root</var> be the <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;20"></a><code>[ruby](#the-ruby-element)</code> element for which the algorithm is being run&#46;

4.  If <var>root</var> has a <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;21"></a><code>[ruby](#the-ruby-element)</code> element ancestor&#44; then jump to the step labeled <em>end</em>&#46;

5.  Let <var>current parent</var> be <var>root</var>&#46;

6.  Let <var>index</var> be 0&#46;

7.  Let <var>start index</var> be null&#46;

8.  Let <var>saved start index</var> be null&#46;

9.  Let <var>current base text</var> be null&#46;

10. <em>Start mode</em>&#58; If <var>index</var> is greater than or equal to the number of child nodes in <var>current parent</var>&#44; then jump to the step labeled <em>end mode</em>&#46;

11. If the <var>index</var>th node in <var>current parent</var> is an <a id="the&#45;ruby&#45;element&#58;the&#45;rt&#45;element&#45;6"></a><code>[rt](#the-rt-element)</code> or <a id="the&#45;ruby&#45;element&#58;the&#45;rp&#45;element&#45;4"></a><code>[rp](#the-rp-element)</code> element&#44; jump to the step labeled <em>annotation mode</em>&#46;

12. Set <var>start index</var> to the value of <var>index</var>&#46;

13. <em>Base mode</em>&#58; If the <var>index</var>th node in <var>current parent</var> is a <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;22"></a><code>[ruby](#the-ruby-element)</code> element&#44; and if <var>current parent</var> is the same element as <var>root</var>&#44; then <a id="the&#45;ruby&#45;element&#58;push&#45;a&#45;ruby&#45;level"></a>[push a ruby level](#push-a-ruby-level) and then jump to the step labeled <em>start mode</em>&#46;

14. If the <var>index</var>th node in <var>current parent</var> is an <a id="the&#45;ruby&#45;element&#58;the&#45;rt&#45;element&#45;7"></a><code>[rt](#the-rt-element)</code> or <a id="the&#45;ruby&#45;element&#58;the&#45;rp&#45;element&#45;5"></a><code>[rp](#the-rp-element)</code> element&#44; then <a id="the&#45;ruby&#45;element&#58;set&#45;the&#45;current&#45;base&#45;text"></a>[set the current base text](#set-the-current-base-text) and then jump to the step labeled <em>annotation mode</em>&#46;

15. Increment <var>index</var> by one&#46;

16. <em>Base mode post&#45;increment</em>&#58; If <var>index</var> is greater than or equal to the number of child nodes in <var>current parent</var>&#44; then jump to the step labeled <em>end mode</em>&#46;

17. Jump back to the step labeled <em>base mode</em>&#46;

18. <em>Annotation mode</em>&#58; If the <var>index</var>th node in <var>current parent</var> is an <a id="the&#45;ruby&#45;element&#58;the&#45;rt&#45;element&#45;8"></a><code>[rt](#the-rt-element)</code> element&#44; then <a id="the&#45;ruby&#45;element&#58;push&#45;a&#45;ruby&#45;annotation"></a>[push a ruby annotation](#push-a-ruby-annotation) and jump to the step labeled <em>annotation mode increment</em>&#46;

19. If the <var>index</var>th node in <var>current parent</var> is an <a id="the&#45;ruby&#45;element&#58;the&#45;rp&#45;element&#45;6"></a><code>[rp](#the-rp-element)</code> element&#44; jump to the step labeled <em>annotation mode increment</em>&#46;

20. If the <var>index</var>th node in <var>current parent</var> is not a <a id="the&#45;ruby&#45;element&#58;text"></a><code>[Text](https://dom.spec.whatwg.org/#interface-text)</code> node&#44; or is a <a id="the&#45;ruby&#45;element&#58;text&#45;2"></a><code>[Text](https://dom.spec.whatwg.org/#interface-text)</code> node that is not <a id="the&#45;ruby&#45;element&#58;inter&#45;element&#45;whitespace&#45;2"></a>[inter&#45;element whitespace](html-language-foundations--877d6146487f--b55e73c31978.md#inter-element-whitespace)&#44; then jump to the step labeled <em>base mode</em>&#46;

21. <em>Annotation mode increment</em>&#58; Let <var>lookahead index</var> be <var>index</var> plus one&#46;

22. <em>Annotation mode white&#45;space skipper</em>&#58; If <var>lookahead index</var> is equal to the number of child nodes in <var>current parent</var> then jump to the step labeled <em>end mode</em>&#46;

23. If the <var>lookahead index</var>th node in <var>current parent</var> is an <a id="the&#45;ruby&#45;element&#58;the&#45;rt&#45;element&#45;9"></a><code>[rt](#the-rt-element)</code> element or an <a id="the&#45;ruby&#45;element&#58;the&#45;rp&#45;element&#45;7"></a><code>[rp](#the-rp-element)</code> element&#44; then set <var>index</var> to <var>lookahead index</var> and jump to the step labeled <em>annotation mode</em>&#46;

24. If the <var>lookahead index</var>th node in <var>current parent</var> is not a <a id="the&#45;ruby&#45;element&#58;text&#45;3"></a><code>[Text](https://dom.spec.whatwg.org/#interface-text)</code> node&#44; or is a <a id="the&#45;ruby&#45;element&#58;text&#45;4"></a><code>[Text](https://dom.spec.whatwg.org/#interface-text)</code> node that is not <a id="the&#45;ruby&#45;element&#58;inter&#45;element&#45;whitespace&#45;3"></a>[inter&#45;element whitespace](html-language-foundations--877d6146487f--b55e73c31978.md#inter-element-whitespace)&#44; then jump to the step labeled <em>base mode</em> &#40;without further incrementing <var>index</var>&#44; so the <a id="the&#45;ruby&#45;element&#58;inter&#45;element&#45;whitespace&#45;4"></a>[inter&#45;element whitespace](html-language-foundations--877d6146487f--b55e73c31978.md#inter-element-whitespace) seen so far becomes part of the next base text segment&#41;&#46;

25. Increment <var>lookahead index</var> by one&#46;

26. Jump to the step labeled <em>annotation mode white&#45;space skipper</em>&#46;

27. <em>End mode</em>&#58; If <var>current parent</var> is not the same element as <var>root</var>&#44; then <a id="the&#45;ruby&#45;element&#58;pop&#45;a&#45;ruby&#45;level"></a>[pop a ruby level](#pop-a-ruby-level) and jump to the step labeled <em>base mode post&#45;increment</em>&#46;

28. <em>End</em>&#58; Return <var>base text segments</var> and <var>annotation segments</var>&#46; Any content of the <a id="the&#45;ruby&#45;element&#58;the&#45;ruby&#45;element&#45;23"></a><code>[ruby](#the-ruby-element)</code> element not described by segments in either of those lists is implicitly in an <em>ignored segment</em>&#46;

When the steps above say to <a id="set&#45;the&#45;current&#45;base&#45;text"></a><strong>set the current base text</strong>&#44; it means to run the following steps at that point in the algorithm&#58;

1.  Let <var>text range</var> be a DOM range whose <a id="the&#45;ruby&#45;element&#58;concept&#45;range&#45;start"></a>[start](https://dom.spec.whatwg.org/#concept-range-start) is the <a id="the&#45;ruby&#45;element&#58;concept&#45;range&#45;bp"></a>[boundary point](https://dom.spec.whatwg.org/#concept-range-bp) &#40;<var>current parent</var>&#44; <var>start index</var>&#41; and whose <a id="the&#45;ruby&#45;element&#58;concept&#45;range&#45;end"></a>[end](https://dom.spec.whatwg.org/#concept-range-end) is the <a id="the&#45;ruby&#45;element&#58;concept&#45;range&#45;bp&#45;2"></a>[boundary point](https://dom.spec.whatwg.org/#concept-range-bp) &#40;<var>current parent</var>&#44; <var>index</var>&#41;&#46;

2.  Let <var>new text segment</var> be a base text segment described by the range <var>text range</var>&#46;

3.  Add <var>new text segment</var> to <var>base text segments</var>&#46;

4.  Let <var>current base text</var> be <var>new text segment</var>&#46;

5.  Let <var>start index</var> be null&#46;

When the steps above say to <a id="push&#45;a&#45;ruby&#45;level"></a><strong>push a ruby level</strong>&#44; it means to run the following steps at that point in the algorithm&#58;

1.  Let <var>current parent</var> be the <var>index</var>th node in <var>current parent</var>&#46;

2.  Let <var>index</var> be 0&#46;

3.  Set <var>saved start index</var> to the value of <var>start index</var>&#46;

4.  Let <var>start index</var> be null&#46;

When the steps above say to <a id="pop&#45;a&#45;ruby&#45;level"></a><strong>pop a ruby level</strong>&#44; it means to run the following steps at that point in the algorithm&#58;

1.  Let <var>index</var> be the position of <var>current parent</var> in <var>root</var>&#46;

2.  Let <var>current parent</var> be <var>root</var>&#46;

3.  Increment <var>index</var> by one&#46;

4.  Set <var>start index</var> to the value of <var>saved start index</var>&#46;

5.  Let <var>saved start index</var> be null&#46;

When the steps above say to <a id="push&#45;a&#45;ruby&#45;annotation"></a><strong>push a ruby annotation</strong>&#44; it means to run the following steps at that point in the algorithm&#58;

1.  Let <var>rt</var> be the <a id="the&#45;ruby&#45;element&#58;the&#45;rt&#45;element&#45;10"></a><code>[rt](#the-rt-element)</code> element that is the <var>index</var>th node of <var>current parent</var>&#46;

2.  Let <var>annotation range</var> be a DOM range whose <a id="the&#45;ruby&#45;element&#58;concept&#45;range&#45;start&#45;2"></a>[start](https://dom.spec.whatwg.org/#concept-range-start) is the <a id="the&#45;ruby&#45;element&#58;concept&#45;range&#45;bp&#45;3"></a>[boundary point](https://dom.spec.whatwg.org/#concept-range-bp) &#40;<var>current parent</var>&#44; <var>index</var>&#41; and whose <a id="the&#45;ruby&#45;element&#58;concept&#45;range&#45;end&#45;2"></a>[end](https://dom.spec.whatwg.org/#concept-range-end) is the <a id="the&#45;ruby&#45;element&#58;concept&#45;range&#45;bp&#45;4"></a>[boundary point](https://dom.spec.whatwg.org/#concept-range-bp) &#40;<var>current parent</var>&#44; <var>index</var> plus one&#41; &#40;i&#46;e&#46; that contains only <var>rt</var>&#41;&#46;

3.  Let <var>new annotation segment</var> be an annotation segment described by the range <var>annotation range</var>&#46;

4.  If <var>current base text</var> is not null&#44; associate <var>new annotation segment</var> with <var>current base text</var>&#46;

5.  Add <var>new annotation segment</var> to <var>annotation segments</var>&#46;

<strong>Example&#58;</strong>

In this example&#44; each ideograph in the Japanese text 漢字 is annotated with its reading in hiragana&#46;

``` text
...
<ruby>漢<rt>かん</rt>字<rt>じ</rt></ruby>
...
```

This might be rendered as&#58;

![The two main ideographs&#44; each with its annotation in hiragana rendered in a smaller font above it&#46;](https://html.spec.whatwg.org/images/sample-ruby-ja.png)

<strong>Example&#58;</strong>

In this example&#44; each ideograph in the traditional Chinese text 漢字 is annotated with its bopomofo reading&#46;

``` text
<ruby>漢<rt>ㄏㄢˋ</rt>字<rt>ㄗˋ</rt></ruby>
```

This might be rendered as&#58;

![The two main ideographs&#44; each with its bopomofo annotation rendered in a smaller font next to it&#46;](https://html.spec.whatwg.org/images/sample-ruby-bopomofo.png)

<strong>Example&#58;</strong>

In this example&#44; each ideograph in the simplified Chinese text 汉字 is annotated with its pinyin reading&#46;

``` text
...<ruby>汉<rt>hàn</rt>字<rt>zì</rt></ruby>...
```

This might be rendered as&#58;

![The two main ideographs&#44; each with its pinyin annotation rendered in a smaller font above it&#46;](https://html.spec.whatwg.org/images/sample-ruby-pinyin.png)

<strong>Example&#58;</strong>

In this more contrived example&#44; the acronym &#34;HTML&#34; has four annotations&#58; one for the whole acronym&#44; briefly describing what it is&#44; one for the letters &#34;HT&#34; expanding them to &#34;Hypertext&#34;&#44; one for the letter &#34;M&#34; expanding it to &#34;Markup&#34;&#44; and one for the letter &#34;L&#34; expanding it to &#34;Language&#34;&#46;

``` text
<ruby>
 <ruby>HT<rt>Hypertext</rt>M<rt>Markup</rt>L<rt>Language</rt></ruby>
 <rt>An abstract language for describing documents and applications
</ruby>
```

#### <a id="the&#45;rt&#45;element"></a>4&#46;5&#46;11 The <strong><code>rt</code></strong> element[](#the-rt-element)

<strong>✔</strong>MDN

[Element&#47;rt](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/rt)

Support in all current engines&#46;

Firefox38&#43;Safari5&#43;Chrome5&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet Explorer5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;rt&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

None&#46;

<strong><a id="the&#45;rt&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

As a child of a <a id="the&#45;rt&#45;element&#58;the&#45;ruby&#45;element"></a><code>[ruby](#the-ruby-element)</code> element&#46;

<strong><a id="the&#45;rt&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;rt&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;rt&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

An <a id="the&#45;rt&#45;element&#58;the&#45;rt&#45;element"></a><code>[rt](#the-rt-element)</code> element&#39;s <a id="the&#45;rt&#45;element&#58;syntax&#45;end&#45;tag"></a>[end tag](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#syntax-end-tag) can be omitted if the <a id="the&#45;rt&#45;element&#58;the&#45;rt&#45;element&#45;2"></a><code>[rt](#the-rt-element)</code> element is immediately followed by an <a id="the&#45;rt&#45;element&#58;the&#45;rt&#45;element&#45;3"></a><code>[rt](#the-rt-element)</code> or <a id="the&#45;rt&#45;element&#58;the&#45;rp&#45;element"></a><code>[rp](#the-rp-element)</code> element&#44; or if there is no more content in the parent element&#46;

<strong><a id="the&#45;rt&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;rt&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;rt&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-rt)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-rt)&#46;

<strong><a id="the&#45;rt&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;rt&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;rt&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;rt&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;rt&#45;element&#58;the&#45;rt&#45;element&#45;4"></a><code>[rt](#the-rt-element)</code> element marks the ruby text component of a ruby annotation&#46; When it is the child of a <a id="the&#45;rt&#45;element&#58;the&#45;ruby&#45;element&#45;2"></a><code>[ruby](#the-ruby-element)</code> element&#44; it doesn&#39;t <a id="the&#45;rt&#45;element&#58;represents"></a>[represent](html-language-foundations--877d6146487f--b55e73c31978.md#represents) anything itself&#44; but the <a id="the&#45;rt&#45;element&#58;the&#45;ruby&#45;element&#45;3"></a><code>[ruby](#the-ruby-element)</code> element uses it as part of determining what <em>it</em> <a id="the&#45;rt&#45;element&#58;represents&#45;2"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents)&#46;

An <a id="the&#45;rt&#45;element&#58;the&#45;rt&#45;element&#45;5"></a><code>[rt](#the-rt-element)</code> element that is not a child of a <a id="the&#45;rt&#45;element&#58;the&#45;ruby&#45;element&#45;4"></a><code>[ruby](#the-ruby-element)</code> element <a id="the&#45;rt&#45;element&#58;represents&#45;3"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) the same thing as its children&#46;

#### <a id="the&#45;rp&#45;element"></a>4&#46;5&#46;12 The <strong><code>rp</code></strong> element[](#the-rp-element)

<strong>✔</strong>MDN

[Element&#47;rp](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/rp)

Support in all current engines&#46;

Firefox38&#43;Safari5&#43;Chrome5&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet Explorer5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;rp&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

None&#46;

<strong><a id="the&#45;rp&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

As a child of a <a id="the&#45;rp&#45;element&#58;the&#45;ruby&#45;element"></a><code>[ruby](#the-ruby-element)</code> element&#44; either immediately before or immediately after an <a id="the&#45;rp&#45;element&#58;the&#45;rt&#45;element"></a><code>[rt](#the-rt-element)</code> element&#46;

<strong><a id="the&#45;rp&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;rp&#45;element&#58;text&#45;content"></a>[Text](html-language-foundations--877d6146487f--b55e73c31978.md#text-content)&#46;

<strong><a id="the&#45;rp&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

An <a id="the&#45;rp&#45;element&#58;the&#45;rp&#45;element"></a><code>[rp](#the-rp-element)</code> element&#39;s <a id="the&#45;rp&#45;element&#58;syntax&#45;end&#45;tag"></a>[end tag](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#syntax-end-tag) can be omitted if the <a id="the&#45;rp&#45;element&#58;the&#45;rp&#45;element&#45;2"></a><code>[rp](#the-rp-element)</code> element is immediately followed by an <a id="the&#45;rp&#45;element&#58;the&#45;rt&#45;element&#45;2"></a><code>[rt](#the-rt-element)</code> or <a id="the&#45;rp&#45;element&#58;the&#45;rp&#45;element&#45;3"></a><code>[rp](#the-rp-element)</code> element&#44; or if there is no more content in the parent element&#46;

<strong><a id="the&#45;rp&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;rp&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;rp&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-rp)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-rp)&#46;

<strong><a id="the&#45;rp&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;rp&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;rp&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;rp&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;rp&#45;element&#58;the&#45;rp&#45;element&#45;4"></a><code>[rp](#the-rp-element)</code> element can be used to provide parentheses or other content around a ruby text component of a ruby annotation&#44; to be shown by user agents that don&#39;t support ruby annotations&#46;

An <a id="the&#45;rp&#45;element&#58;the&#45;rp&#45;element&#45;5"></a><code>[rp](#the-rp-element)</code> element that is a child of a <a id="the&#45;rp&#45;element&#58;the&#45;ruby&#45;element&#45;2"></a><code>[ruby](#the-ruby-element)</code> element <a id="the&#45;rp&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) nothing&#46; An <a id="the&#45;rp&#45;element&#58;the&#45;rp&#45;element&#45;6"></a><code>[rp](#the-rp-element)</code> element whose parent element is not a <a id="the&#45;rp&#45;element&#58;the&#45;ruby&#45;element&#45;3"></a><code>[ruby](#the-ruby-element)</code> element <a id="the&#45;rp&#45;element&#58;represents&#45;2"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) its children&#46;

<strong>Example&#58;</strong>

The example above&#44; in which each ideograph in the text 漢字 is annotated with its phonetic reading&#44; could be expanded to use <a id="the&#45;rp&#45;element&#58;the&#45;rp&#45;element&#45;7"></a><code>[rp](#the-rp-element)</code> so that in legacy user agents the readings are in parentheses&#58;

``` text
...
<ruby>漢<rp>（</rp><rt>かん</rt><rp>）</rp>字<rp>（</rp><rt>じ</rt><rp>）</rp></ruby>
...
```

In conforming user agents the rendering would be as above&#44; but in user agents that do not support ruby&#44; the rendering would be&#58;

``` text
... 漢（かん）字（じ）...
```

<strong>Example&#58;</strong>

When there are multiple annotations for a segment&#44; <a id="the&#45;rp&#45;element&#58;the&#45;rp&#45;element&#45;8"></a><code>[rp](#the-rp-element)</code> elements can also be placed between the annotations&#46; Here is another copy of an earlier contrived example showing some symbols with names given in English and French&#44; but this time with <a id="the&#45;rp&#45;element&#58;the&#45;rp&#45;element&#45;9"></a><code>[rp](#the-rp-element)</code> elements as well&#58;

``` text
<ruby>
♥<rp>: </rp><rt>Heart</rt><rp>, </rp><rt lang=fr>Cœur</rt><rp>.</rp>
☘<rp>: </rp><rt>Shamrock</rt><rp>, </rp><rt lang=fr>Trèfle</rt><rp>.</rp>
✶<rp>: </rp><rt>Star</rt><rp>, </rp><rt lang=fr>Étoile</rt><rp>.</rp>
</ruby>
```

This would make the example render as follows in non&#45;ruby&#45;capable user agents&#58;

``` text
♥: Heart, Cœur. ☘: Shamrock, Trèfle. ✶: Star, Étoile.
```

#### <a id="the&#45;data&#45;element"></a>4&#46;5&#46;13 The <strong><code>data</code></strong> element[](#the-data-element)

<strong>✔</strong>MDN

[Element&#47;data](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/data)

Support in all current engines&#46;

Firefox22&#43;Safari10&#43;Chrome62&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;18Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong>✔</strong>MDN

[HTMLDataElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDataElement)

Support in all current engines&#46;

Firefox22&#43;Safari10&#43;Chrome62&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;14&#43;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[HTMLDataElement&#47;value](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDataElement/value)

Support in all current engines&#46;

Firefox22&#43;Safari10&#43;Chrome62&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;14&#43;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;data&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;data&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;data&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;data&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;data&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;data&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;data&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;data&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;data&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;data&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;data&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<a id="the&#45;data&#45;element&#58;attr&#45;data&#45;value"></a><code>[value](#attr-data-value)</code> — Machine&#45;readable value

<strong><a id="the&#45;data&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-data)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-data)&#46;

<strong><a id="the&#45;data&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;data&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default) with attributes <a id="the&#45;data&#45;element&#58;attr&#45;data&#45;value&#45;2"></a><code>[value](#attr-data-value)</code>&#46;

<strong><a id="the&#45;data&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

<a id="htmldataelement"></a><a id="the&#45;data&#45;element&#58;htmlelement"></a><a id="the&#45;data&#45;element&#58;htmlconstructor"></a><a id="the&#45;data&#45;element&#58;cereactions"></a><a id="the&#45;data&#45;element&#58;xattr&#45;reflect"></a><a id="dom&#45;data&#45;value"></a>

``` text
[Exposed=Window]
interface HTMLDataElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions, Reflect] attribute DOMString value;
};
```

The <a id="the&#45;data&#45;element&#58;the&#45;data&#45;element"></a><code>[data](#the-data-element)</code> element <a id="the&#45;data&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) its contents&#44; along with a machine&#45;readable form of those contents in the <a id="the&#45;data&#45;element&#58;attr&#45;data&#45;value&#45;3"></a><code>[value](#attr-data-value)</code> attribute&#46;

The <a id="attr&#45;data&#45;value"></a><strong><code>value</code></strong> attribute must be present&#46; Its value must be a representation of the element&#39;s contents in a machine&#45;readable format&#46;

<strong>Note&#58;</strong>

When the value is date&#45; or time&#45;related&#44; the more specific <a id="the&#45;data&#45;element&#58;the&#45;time&#45;element"></a><code>[time](#the-time-element)</code> element can be used instead&#46;

The element can be used for several purposes&#46;

When combined with microformats or the [microdata attributes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#microdata) defined in this specification&#44; the element serves to provide both a machine&#45;readable value for the purposes of data processors&#44; and a human&#45;readable value for the purposes of rendering in a web browser&#46; In this case&#44; the format to be used in the <a id="the&#45;data&#45;element&#58;attr&#45;data&#45;value&#45;4"></a><code>[value](#attr-data-value)</code> attribute is determined by the microformats or microdata vocabulary in use&#46;

The element can also&#44; however&#44; be used in conjunction with scripts in the page&#44; for when a script has a literal value to store alongside a human&#45;readable value&#46; In such cases&#44; the format to be used depends only on the needs of the script&#46; &#40;The <a id="the&#45;data&#45;element&#58;attr&#45;data&#45;&#42;"></a><code>[data&#45;&#42;](html-language-foundations--877d6146487f--b55e73c31978.md#attr-data-*)</code> attributes can also be useful in such situations&#46;&#41;

<strong>Example&#58;</strong>

Here&#44; a short table has its numeric values encoded using the <a id="the&#45;data&#45;element&#58;the&#45;data&#45;element&#45;2"></a><code>[data](#the-data-element)</code> element so that the table sorting JavaScript library can provide a sorting mechanism on each column despite the numbers being presented in textual form in one column and in a decomposed form in another&#46;

``` text
<script src="sortable.js"></script>
<table class="sortable">
 <thead> <tr> <th> Game <th> Corporations <th> Map Size
 <tbody>
  <tr> <td> 1830 <td> <data value="8">Eight</data> <td> <data value="93">19+74 hexes (93 total)</data>
  <tr> <td> 1856 <td> <data value="11">Eleven</data> <td> <data value="99">12+87 hexes (99 total)</data>
  <tr> <td> 1870 <td> <data value="10">Ten</data> <td> <data value="149">4+145 hexes (149 total)</data>
</table>
```

#### <a id="the&#45;time&#45;element"></a>4&#46;5&#46;14 The <strong><code>time</code></strong> element[](#the-time-element)

<strong>✔</strong>MDN

[Element&#47;time](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/time)

Support in all current engines&#46;

Firefox22&#43;Safari7&#43;Chrome62&#43;

------------------------------------------------------------------------

Opera49&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;18Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS4&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android46&#43;

<strong>✔</strong>MDN

[HTMLTimeElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLTimeElement)

Support in all current engines&#46;

Firefox22&#43;Safari10&#43;Chrome62&#43;

------------------------------------------------------------------------

Opera49&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;14&#43;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android46&#43;

[HTMLTimeElement&#47;dateTime](https://developer.mozilla.org/en-US/docs/Web/API/HTMLTimeElement/dateTime)

Support in all current engines&#46;

Firefox22&#43;Safari10&#43;Chrome62&#43;

------------------------------------------------------------------------

Opera49&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;14&#43;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android46&#43;

<strong><a id="the&#45;time&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;time&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;time&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;time&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;time&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;time&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;time&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

If the element has a <a id="the&#45;time&#45;element&#58;attr&#45;time&#45;datetime"></a><code>[datetime](#attr-time-datetime)</code> attribute&#58; <a id="the&#45;time&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

Otherwise&#58; <a id="the&#45;time&#45;element&#58;text&#45;content"></a>[Text](html-language-foundations--877d6146487f--b55e73c31978.md#text-content)&#44; but must match requirements described in prose below&#46;

<strong><a id="the&#45;time&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;time&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;time&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<a id="the&#45;time&#45;element&#58;attr&#45;time&#45;datetime&#45;2"></a><code>[datetime](#attr-time-datetime)</code> — Machine&#45;readable value

<strong><a id="the&#45;time&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-time)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-time)&#46;

<strong><a id="the&#45;time&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;time&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default) with attributes <a id="the&#45;time&#45;element&#58;attr&#45;time&#45;datetime&#45;3"></a><code>[datetime](#attr-time-datetime)</code>&#46;

<strong><a id="the&#45;time&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

<a id="htmltimeelement"></a><a id="the&#45;time&#45;element&#58;htmlelement"></a><a id="the&#45;time&#45;element&#58;htmlconstructor"></a><a id="the&#45;time&#45;element&#58;cereactions"></a><a id="the&#45;time&#45;element&#58;xattr&#45;reflect"></a><a id="dom&#45;time&#45;datetime"></a>

``` text
[Exposed=Window]
interface HTMLTimeElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions, Reflect] attribute DOMString dateTime;
};
```

The <a id="the&#45;time&#45;element&#58;the&#45;time&#45;element"></a><code>[time](#the-time-element)</code> element <a id="the&#45;time&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) its contents&#44; along with a machine&#45;readable form of those contents in the <a id="the&#45;time&#45;element&#58;attr&#45;time&#45;datetime&#45;4"></a><code>[datetime](#attr-time-datetime)</code> attribute&#46; The kind of content is limited to various kinds of dates&#44; times&#44; time&#45;zone offsets&#44; and durations&#44; as described below&#46;

The <a id="attr&#45;time&#45;datetime"></a><strong><code>datetime</code></strong> attribute may be present&#46; If present&#44; its value must be a representation of the element&#39;s contents in a machine&#45;readable format&#46;

A <a id="the&#45;time&#45;element&#58;the&#45;time&#45;element&#45;2"></a><code>[time](#the-time-element)</code> element that does not have a <a id="the&#45;time&#45;element&#58;attr&#45;time&#45;datetime&#45;5"></a><code>[datetime](#attr-time-datetime)</code> content attribute must not have any element descendants&#46;

The <a id="datetime&#45;value"></a><strong>datetime value</strong> of a <a id="the&#45;time&#45;element&#58;the&#45;time&#45;element&#45;3"></a><code>[time](#the-time-element)</code> element is the value of the element&#39;s <a id="the&#45;time&#45;element&#58;attr&#45;time&#45;datetime&#45;6"></a><code>[datetime](#attr-time-datetime)</code> content attribute&#44; if it has one&#44; otherwise the <a id="the&#45;time&#45;element&#58;child&#45;text&#45;content"></a>[child text content](https://dom.spec.whatwg.org/#concept-child-text-content) of the <a id="the&#45;time&#45;element&#58;the&#45;time&#45;element&#45;4"></a><code>[time](#the-time-element)</code> element&#46;

The <a id="the&#45;time&#45;element&#58;datetime&#45;value"></a>[datetime value](#datetime-value) of a <a id="the&#45;time&#45;element&#58;the&#45;time&#45;element&#45;5"></a><code>[time](#the-time-element)</code> element must match one of the following syntaxes&#46;

<strong>A <a id="the&#45;time&#45;element&#58;valid&#45;month&#45;string"></a>[valid month string](html-language-foundations--877d6146487f--b55e73c31978.md#valid-month-string)</strong>

``` text
<time>2011-11</time>
```

<strong>A <a id="the&#45;time&#45;element&#58;valid&#45;date&#45;string"></a>[valid date string](html-language-foundations--877d6146487f--b55e73c31978.md#valid-date-string)</strong>

``` text
<time>2011-11-18</time>
```

<strong>A <a id="the&#45;time&#45;element&#58;valid&#45;yearless&#45;date&#45;string"></a>[valid yearless date string](html-language-foundations--877d6146487f--b55e73c31978.md#valid-yearless-date-string)</strong>

``` text
<time>11-18</time>
```

<strong>A <a id="the&#45;time&#45;element&#58;valid&#45;time&#45;string"></a>[valid time string](html-language-foundations--877d6146487f--b55e73c31978.md#valid-time-string)</strong>

``` text
<time>14:54</time>
```

``` text
<time>14:54:39</time>
```

``` text
<time>14:54:39.929</time>
```

<strong>A <a id="the&#45;time&#45;element&#58;valid&#45;local&#45;date&#45;and&#45;time&#45;string"></a>[valid local date and time string](html-language-foundations--877d6146487f--b55e73c31978.md#valid-local-date-and-time-string)</strong>

``` text
<time>2011-11-18T14:54</time>
```

``` text
<time>2011-11-18T14:54:39</time>
```

``` text
<time>2011-11-18T14:54:39.929</time>
```

``` text
<time>2011-11-18 14:54</time>
```

``` text
<time>2011-11-18 14:54:39</time>
```

``` text
<time>2011-11-18 14:54:39.929</time>
```

<strong>Note&#58;</strong>

Times with dates but without a time zone offset are useful for specifying events that are observed at the same specific time in each time zone&#44; throughout a day&#46; For example&#44; the 2020 new year is celebrated at 2020&#45;01&#45;01 00&#58;00 in each time zone&#44; not at the same precise moment across all time zones&#46; For events that occur at the same time across all time zones&#44; for example a videoconference meeting&#44; a <a id="the&#45;time&#45;element&#58;valid&#45;global&#45;date&#45;and&#45;time&#45;string"></a>[valid global date and time string](html-language-foundations--877d6146487f--b55e73c31978.md#valid-global-date-and-time-string) is likely more useful&#46;

<strong>A <a id="the&#45;time&#45;element&#58;valid&#45;time&#45;zone&#45;offset&#45;string"></a>[valid time&#45;zone offset string](html-language-foundations--877d6146487f--b55e73c31978.md#valid-time-zone-offset-string)</strong>

``` text
<time>Z</time>
```

``` text
<time>+0000</time>
```

``` text
<time>+00:00</time>
```

``` text
<time>-0800</time>
```

``` text
<time>-08:00</time>
```

<strong>Note&#58;</strong>

For times without dates &#40;or times referring to events that recur on multiple dates&#41;&#44; specifying the geographic location that controls the time is usually more useful than specifying a time zone offset&#44; because geographic locations change time zone offsets with daylight saving time&#46; In some cases&#44; geographic locations even change time zone&#44; e&#46;g&#46; when the boundaries of those time zones are redrawn&#44; as happened with Samoa at the end of 2011&#46; There exists a time zone database that describes the boundaries of time zones and what rules apply within each such zone&#44; known as the <em>time zone database</em>&#46; [&#91;TZDATABASE&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsTZDATABASE)

<strong>A <a id="the&#45;time&#45;element&#58;valid&#45;global&#45;date&#45;and&#45;time&#45;string&#45;2"></a>[valid global date and time string](html-language-foundations--877d6146487f--b55e73c31978.md#valid-global-date-and-time-string)</strong>

``` text
<time>2011-11-18T14:54Z</time>
```

``` text
<time>2011-11-18T14:54:39Z</time>
```

``` text
<time>2011-11-18T14:54:39.929Z</time>
```

``` text
<time>2011-11-18T14:54+0000</time>
```

``` text
<time>2011-11-18T14:54:39+0000</time>
```

``` text
<time>2011-11-18T14:54:39.929+0000</time>
```

``` text
<time>2011-11-18T14:54+00:00</time>
```

``` text
<time>2011-11-18T14:54:39+00:00</time>
```

``` text
<time>2011-11-18T14:54:39.929+00:00</time>
```

``` text
<time>2011-11-18T06:54-0800</time>
```

``` text
<time>2011-11-18T06:54:39-0800</time>
```

``` text
<time>2011-11-18T06:54:39.929-0800</time>
```

``` text
<time>2011-11-18T06:54-08:00</time>
```

``` text
<time>2011-11-18T06:54:39-08:00</time>
```

``` text
<time>2011-11-18T06:54:39.929-08:00</time>
```

``` text
<time>2011-11-18 14:54Z</time>
```

``` text
<time>2011-11-18 14:54:39Z</time>
```

``` text
<time>2011-11-18 14:54:39.929Z</time>
```

``` text
<time>2011-11-18 14:54+0000</time>
```

``` text
<time>2011-11-18 14:54:39+0000</time>
```

``` text
<time>2011-11-18 14:54:39.929+0000</time>
```

``` text
<time>2011-11-18 14:54+00:00</time>
```

``` text
<time>2011-11-18 14:54:39+00:00</time>
```

``` text
<time>2011-11-18 14:54:39.929+00:00</time>
```

``` text
<time>2011-11-18 06:54-0800</time>
```

``` text
<time>2011-11-18 06:54:39-0800</time>
```

``` text
<time>2011-11-18 06:54:39.929-0800</time>
```

``` text
<time>2011-11-18 06:54-08:00</time>
```

``` text
<time>2011-11-18 06:54:39-08:00</time>
```

``` text
<time>2011-11-18 06:54:39.929-08:00</time>
```

<strong>Note&#58;</strong>

Times with dates and a time zone offset are useful for specifying specific events&#44; or recurring virtual events where the time is not anchored to a specific geographic location&#46; For example&#44; the precise time of an asteroid impact&#44; or a particular meeting in a series of meetings held at 1400 UTC every day&#44; regardless of whether any particular part of the world is observing daylight saving time or not&#46; For events where the precise time varies by the local time zone offset of a specific geographic location&#44; a <a id="the&#45;time&#45;element&#58;valid&#45;local&#45;date&#45;and&#45;time&#45;string&#45;2"></a>[valid local date and time string](html-language-foundations--877d6146487f--b55e73c31978.md#valid-local-date-and-time-string) combined with that geographic location is likely more useful&#46;

<strong>A <a id="the&#45;time&#45;element&#58;valid&#45;week&#45;string"></a>[valid week string](html-language-foundations--877d6146487f--b55e73c31978.md#valid-week-string)</strong>

``` text
<time>2011-W47</time>
```

<strong>Four or more <a id="the&#45;time&#45;element&#58;ascii&#45;digits"></a>[ASCII digits](https://infra.spec.whatwg.org/#ascii-digit)&#44; at least one of which is not U&#43;0030 DIGIT ZERO &#40;0&#41;</strong>

``` text
<time>2011</time>
```

``` text
<time>0001</time>
```

<strong>A <a id="the&#45;time&#45;element&#58;valid&#45;duration&#45;string"></a>[valid duration string](html-language-foundations--877d6146487f--b55e73c31978.md#valid-duration-string)</strong>

``` text
<time>PT4H18M3S</time>
```

``` text
<time>4h 18m 3s</time>
```

The <a id="machine&#45;readable&#45;equivalent&#45;of&#45;the&#45;element&#39;s&#45;contents"></a><strong>machine&#45;readable equivalent of the element&#39;s contents</strong> must be obtained from the element&#39;s <a id="the&#45;time&#45;element&#58;datetime&#45;value&#45;2"></a>[datetime value](#datetime-value) by using the following algorithm&#58;

1.  If <a id="the&#45;time&#45;element&#58;parse&#45;a&#45;month&#45;string"></a>[parsing a month string](html-language-foundations--877d6146487f--b55e73c31978.md#parse-a-month-string) from the element&#39;s <a id="the&#45;time&#45;element&#58;datetime&#45;value&#45;3"></a>[datetime value](#datetime-value) returns a <a id="the&#45;time&#45;element&#58;concept&#45;month"></a>[month](html-language-foundations--877d6146487f--b55e73c31978.md#concept-month)&#44; that is the machine&#45;readable equivalent&#59; return&#46;

2.  If <a id="the&#45;time&#45;element&#58;parse&#45;a&#45;date&#45;string"></a>[parsing a date string](html-language-foundations--877d6146487f--b55e73c31978.md#parse-a-date-string) from the element&#39;s <a id="the&#45;time&#45;element&#58;datetime&#45;value&#45;4"></a>[datetime value](#datetime-value) returns a <a id="the&#45;time&#45;element&#58;concept&#45;date"></a>[date](html-language-foundations--877d6146487f--b55e73c31978.md#concept-date)&#44; that is the machine&#45;readable equivalent&#59; return&#46;

3.  If <a id="the&#45;time&#45;element&#58;parse&#45;a&#45;yearless&#45;date&#45;string"></a>[parsing a yearless date string](html-language-foundations--877d6146487f--b55e73c31978.md#parse-a-yearless-date-string) from the element&#39;s <a id="the&#45;time&#45;element&#58;datetime&#45;value&#45;5"></a>[datetime value](#datetime-value) returns a <a id="the&#45;time&#45;element&#58;concept&#45;yearless&#45;date"></a>[yearless date](html-language-foundations--877d6146487f--b55e73c31978.md#concept-yearless-date)&#44; that is the machine&#45;readable equivalent&#59; return&#46;

4.  If <a id="the&#45;time&#45;element&#58;parse&#45;a&#45;time&#45;string"></a>[parsing a time string](html-language-foundations--877d6146487f--b55e73c31978.md#parse-a-time-string) from the element&#39;s <a id="the&#45;time&#45;element&#58;datetime&#45;value&#45;6"></a>[datetime value](#datetime-value) returns a <a id="the&#45;time&#45;element&#58;concept&#45;time"></a>[time](html-language-foundations--877d6146487f--b55e73c31978.md#concept-time)&#44; that is the machine&#45;readable equivalent&#59; return&#46;

5.  If <a id="the&#45;time&#45;element&#58;parse&#45;a&#45;local&#45;date&#45;and&#45;time&#45;string"></a>[parsing a local date and time string](html-language-foundations--877d6146487f--b55e73c31978.md#parse-a-local-date-and-time-string) from the element&#39;s <a id="the&#45;time&#45;element&#58;datetime&#45;value&#45;7"></a>[datetime value](#datetime-value) returns a <a id="the&#45;time&#45;element&#58;concept&#45;datetime&#45;local"></a>[local date and time](html-language-foundations--877d6146487f--b55e73c31978.md#concept-datetime-local)&#44; that is the machine&#45;readable equivalent&#59; return&#46;

6.  If <a id="the&#45;time&#45;element&#58;parse&#45;a&#45;time&#45;zone&#45;offset&#45;string"></a>[parsing a time&#45;zone offset string](html-language-foundations--877d6146487f--b55e73c31978.md#parse-a-time-zone-offset-string) from the element&#39;s <a id="the&#45;time&#45;element&#58;datetime&#45;value&#45;8"></a>[datetime value](#datetime-value) returns a <a id="the&#45;time&#45;element&#58;concept&#45;timezone"></a>[time&#45;zone offset](html-language-foundations--877d6146487f--b55e73c31978.md#concept-timezone)&#44; that is the machine&#45;readable equivalent&#59; return&#46;

7.  If <a id="the&#45;time&#45;element&#58;parse&#45;a&#45;global&#45;date&#45;and&#45;time&#45;string"></a>[parsing a global date and time string](html-language-foundations--877d6146487f--b55e73c31978.md#parse-a-global-date-and-time-string) from the element&#39;s <a id="the&#45;time&#45;element&#58;datetime&#45;value&#45;9"></a>[datetime value](#datetime-value) returns a <a id="the&#45;time&#45;element&#58;concept&#45;datetime"></a>[global date and time](html-language-foundations--877d6146487f--b55e73c31978.md#concept-datetime)&#44; that is the machine&#45;readable equivalent&#59; return&#46;

8.  If <a id="the&#45;time&#45;element&#58;parse&#45;a&#45;week&#45;string"></a>[parsing a week string](html-language-foundations--877d6146487f--b55e73c31978.md#parse-a-week-string) from the element&#39;s <a id="the&#45;time&#45;element&#58;datetime&#45;value&#45;10"></a>[datetime value](#datetime-value) returns a <a id="the&#45;time&#45;element&#58;concept&#45;week"></a>[week](html-language-foundations--877d6146487f--b55e73c31978.md#concept-week)&#44; that is the machine&#45;readable equivalent&#59; return&#46;

9.  If the element&#39;s <a id="the&#45;time&#45;element&#58;datetime&#45;value&#45;11"></a>[datetime value](#datetime-value) consists of only <a id="the&#45;time&#45;element&#58;ascii&#45;digits&#45;2"></a>[ASCII digits](https://infra.spec.whatwg.org/#ascii-digit)&#44; at least one of which is not U&#43;0030 DIGIT ZERO &#40;0&#41;&#44; then the machine&#45;readable equivalent is the base&#45;ten interpretation of those digits&#44; representing a year&#59; return&#46;

10. If <a id="the&#45;time&#45;element&#58;parse&#45;a&#45;duration&#45;string"></a>[parsing a duration string](html-language-foundations--877d6146487f--b55e73c31978.md#parse-a-duration-string) from the element&#39;s <a id="the&#45;time&#45;element&#58;datetime&#45;value&#45;12"></a>[datetime value](#datetime-value) returns a <a id="the&#45;time&#45;element&#58;concept&#45;duration"></a>[duration](html-language-foundations--877d6146487f--b55e73c31978.md#concept-duration)&#44; that is the machine&#45;readable equivalent&#59; return&#46;

11. There is no machine&#45;readable equivalent&#46;

<strong>Note&#58;</strong>

The algorithms referenced above are intended to be designed such that for any arbitrary string <var>s</var>&#44; only one of the algorithms returns a value&#46; A more efficient approach might be to create a single algorithm that parses all these data types in one pass&#59; developing such an algorithm is left as an exercise to the reader&#46;

<strong>Example&#58;</strong>

The <a id="the&#45;time&#45;element&#58;the&#45;time&#45;element&#45;6"></a><code>[time](#the-time-element)</code> element can be used to encode dates&#44; for example in microformats&#46; The following shows a hypothetical way of encoding an event using a variant on hCalendar that uses the <a id="the&#45;time&#45;element&#58;the&#45;time&#45;element&#45;7"></a><code>[time](#the-time-element)</code> element&#58;

``` text
<div class="vevent">
 <a class="url" href="http://www.web2con.com/">http://www.web2con.com/</a>
 <span class="summary">Web 2.0 Conference</span>:
 <time class="dtstart" datetime="2005-10-05">October 5</time> -
 <time class="dtend" datetime="2005-10-07">7</time>,
 at the <span class="location">Argent Hotel, San Francisco, CA</span>
</div>
```

<strong>Example&#58;</strong>

Here&#44; a fictional microdata vocabulary based on the Atom vocabulary is used with the <a id="the&#45;time&#45;element&#58;the&#45;time&#45;element&#45;8"></a><code>[time](#the-time-element)</code> element to mark up a blog post&#39;s publication date&#46;

``` text
<article itemscope itemtype="https://n.example.org/rfc4287">
 <h1 itemprop="title">Big tasks</h1>
 <footer>Published <time itemprop="published" datetime="2009-08-29">two days ago</time>.</footer>
 <p itemprop="content">Today, I went out and bought a bike for my kid.</p>
</article>
```

<strong>Example&#58;</strong>

In this example&#44; another article&#39;s publication date is marked up using <a id="the&#45;time&#45;element&#58;the&#45;time&#45;element&#45;9"></a><code>[time](#the-time-element)</code>&#44; this time using the schema&#46;org microdata vocabulary&#58;

``` text
<article itemscope itemtype="http://schema.org/BlogPosting">
 <h1 itemprop="headline">Small tasks</h1>
 <footer>Published <time itemprop="datePublished" datetime="2009-08-30">yesterday</time>.</footer>
 <p itemprop="articleBody">I put a bike bell on her bike.</p>
</article>
```

<strong>Example&#58;</strong>

In the following snippet&#44; the <a id="the&#45;time&#45;element&#58;the&#45;time&#45;element&#45;10"></a><code>[time](#the-time-element)</code> element is used to encode a date in the ISO8601 format&#44; for later processing by a script&#58;

``` text
<p>Our first date was <time datetime="2006-09-23">a Saturday</time>.</p>
```

In this second snippet&#44; the value includes a time&#58;

``` text
<p>We stopped talking at <time datetime="2006-09-24T05:00-07:00">5am the next morning</time>.</p>
```

A script loaded by the page &#40;and thus privy to the page&#39;s internal convention of marking up dates and times using the <a id="the&#45;time&#45;element&#58;the&#45;time&#45;element&#45;11"></a><code>[time](#the-time-element)</code> element&#41; could scan through the page and look at all the <a id="the&#45;time&#45;element&#58;the&#45;time&#45;element&#45;12"></a><code>[time](#the-time-element)</code> elements therein to create an index of dates and times&#46;

<strong>Example&#58;</strong>

For example&#44; this element conveys the string &#34;Friday&#34; with the additional semantic that the 18th of November 2011 is the meaning that corresponds to &#34;Friday&#34;&#58;

``` text
Today is <time datetime="2011-11-18">Friday</time>.
```

<strong>Example&#58;</strong>

In this example&#44; a specific time in the Pacific Standard Time timezone is specified&#58;

``` text
Your next meeting is at <time datetime="2011-11-18T15:00-08:00">3pm</time>.
```

#### <a id="the&#45;code&#45;element"></a>4&#46;5&#46;15 The <strong><code>code</code></strong> element[](#the-code-element)

<strong>✔</strong>MDN

[Element&#47;code](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/code)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;code&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;code&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;code&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;code&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;code&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;code&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;code&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;code&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;code&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;code&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;code&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;code&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-code)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-code)&#46;

<strong><a id="the&#45;code&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;code&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;code&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;code&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;code&#45;element&#58;the&#45;code&#45;element"></a><code>[code](#the-code-element)</code> element <a id="the&#45;code&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a fragment of computer code&#46; This could be an XML element name&#44; a filename&#44; a computer program&#44; or any other string that a computer would recognize&#46;

There is no formal way to indicate the language of computer code being marked up&#46; Authors who wish to mark <a id="the&#45;code&#45;element&#58;the&#45;code&#45;element&#45;2"></a><code>[code](#the-code-element)</code> elements with the language used&#44; e&#46;g&#46; so that syntax highlighting scripts can use the right rules&#44; can use the <a id="the&#45;code&#45;element&#58;classes"></a><code>[class](html-language-foundations--877d6146487f--b55e73c31978.md#classes)</code> attribute&#44; e&#46;g&#46; by adding a class prefixed with &#34;<code>language&#45;</code>&#34; to the element&#46;

<strong>Example&#58;</strong>

The following example shows how the element can be used in a paragraph to mark up element names and computer code&#44; including punctuation&#46;

``` text
<p>The <code>code</code> element represents a fragment of computer
code.</p>

<p>When you call the <code>activate()</code> method on the
<code>robotSnowman</code> object, the eyes glow.</p>

<p>The example below uses the <code>begin</code> keyword to indicate
the start of a statement block. It is paired with an <code>end</code>
keyword, which is followed by the <code>.</code> punctuation character
(full stop) to indicate the end of the program.</p>
```

<strong>Example&#58;</strong>

The following example shows how a block of code could be marked up using the <a id="the&#45;code&#45;element&#58;the&#45;pre&#45;element"></a><code>[pre](#the-pre-element)</code> and <a id="the&#45;code&#45;element&#58;the&#45;code&#45;element&#45;3"></a><code>[code](#the-code-element)</code> elements&#46;

``` text
<pre><code class="language-pascal">var i: Integer;
begin
   i := 1;
end.</code></pre>
```

A class is used in that example to indicate the language used&#46;

<strong>Note&#58;</strong>

See the <a id="the&#45;code&#45;element&#58;the&#45;pre&#45;element&#45;2"></a><code>[pre](#the-pre-element)</code> element for more details&#46;

#### <a id="the&#45;var&#45;element"></a>4&#46;5&#46;16 The <strong><code>var</code></strong> element[](#the-var-element)

<strong>✔</strong>MDN

[Element&#47;var](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/var)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;var&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;var&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;var&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;var&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;var&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;var&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;var&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;var&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;var&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;var&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;var&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;var&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-var)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-var)&#46;

<strong><a id="the&#45;var&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;var&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;var&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;var&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;var&#45;element&#58;the&#45;var&#45;element"></a><code>[var](#the-var-element)</code> element <a id="the&#45;var&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a variable&#46; This could be an actual variable in a mathematical expression or programming context&#44; an identifier representing a constant&#44; a symbol identifying a physical quantity&#44; a function parameter&#44; or just be a term used as a placeholder in prose&#46;

<strong>Example&#58;</strong>

In the paragraph below&#44; the letter &#34;n&#34; is being used as a variable in prose&#58;

``` text
<p>If there are <var>n</var> pipes leading to the ice
cream factory then I expect at <em>least</em> <var>n</var>
flavors of ice cream to be available for purchase!</p>
```

For mathematics&#44; in particular for anything beyond the simplest of expressions&#44; MathML is more appropriate&#46; However&#44; the <a id="the&#45;var&#45;element&#58;the&#45;var&#45;element&#45;2"></a><code>[var](#the-var-element)</code> element can still be used to refer to specific variables that are then mentioned in MathML expressions&#46;

<strong>Example&#58;</strong>

In this example&#44; an equation is shown&#44; with a legend that references the variables in the equation&#46; The expression itself is marked up with MathML&#44; but the variables are mentioned in the figure&#39;s legend using <a id="the&#45;var&#45;element&#58;the&#45;var&#45;element&#45;3"></a><code>[var](#the-var-element)</code>&#46;

``` text
<figure>
 <math>
  <mi>a</mi>
  <mo>=</mo>
  <msqrt>
   <msup><mi>b</mi><mn>2</mn></msup>
   <mi>+</mi>
   <msup><mi>c</mi><mn>2</mn></msup>
  </msqrt>
 </math>
 <figcaption>
  Using Pythagoras' theorem to solve for the hypotenuse <var>a</var> of
  a triangle with sides <var>b</var> and <var>c</var>
 </figcaption>
</figure>
```

<strong>Example&#58;</strong>

Here&#44; the equation describing mass&#45;energy equivalence is used in a sentence&#44; and the <a id="the&#45;var&#45;element&#58;the&#45;var&#45;element&#45;4"></a><code>[var](#the-var-element)</code> element is used to mark the variables and constants in that equation&#58;

``` text
<p>Then she turned to the blackboard and picked up the chalk. After a few moment's
thought, she wrote <var>E</var> = <var>m</var> <var>c</var><sup>2</sup>. The teacher
looked pleased.</p>
```

#### <a id="the&#45;samp&#45;element"></a>4&#46;5&#46;17 The <strong><code>samp</code></strong> element[](#the-samp-element)

<strong>✔</strong>MDN

[Element&#47;samp](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/samp)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;samp&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;samp&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;samp&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;samp&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;samp&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;samp&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;samp&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;samp&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;samp&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;samp&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;samp&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;samp&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-samp)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-samp)&#46;

<strong><a id="the&#45;samp&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;samp&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;samp&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;samp&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;samp&#45;element&#58;the&#45;samp&#45;element"></a><code>[samp](#the-samp-element)</code> element <a id="the&#45;samp&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) sample or quoted output from another program or computing system&#46;

<strong>Note&#58;</strong>

See the <a id="the&#45;samp&#45;element&#58;the&#45;pre&#45;element"></a><code>[pre](#the-pre-element)</code> and <a id="the&#45;samp&#45;element&#58;the&#45;kbd&#45;element"></a><code>[kbd](#the-kbd-element)</code> elements for more details&#46;

<strong>Note&#58;</strong>

This element can be contrasted with the <a id="the&#45;samp&#45;element&#58;the&#45;output&#45;element"></a><code>[output](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-output-element)</code> element&#44; which can be used to provide immediate output in a web application&#46;

<strong>Example&#58;</strong>

This example shows the <a id="the&#45;samp&#45;element&#58;the&#45;samp&#45;element&#45;2"></a><code>[samp](#the-samp-element)</code> element being used inline&#58;

``` text
<p>The computer said <samp>Too much cheese in tray
two</samp> but I didn't know what that meant.</p>
```

<strong>Example&#58;</strong>

This second example shows a block of sample output from a console program&#46; Nested <a id="the&#45;samp&#45;element&#58;the&#45;samp&#45;element&#45;3"></a><code>[samp](#the-samp-element)</code> and <a id="the&#45;samp&#45;element&#58;the&#45;kbd&#45;element&#45;2"></a><code>[kbd](#the-kbd-element)</code> elements allow for the styling of specific elements of the sample output using a style sheet&#46; There&#39;s also a few parts of the <a id="the&#45;samp&#45;element&#58;the&#45;samp&#45;element&#45;4"></a><code>[samp](#the-samp-element)</code> that are annotated with even more detailed markup&#44; to enable very precise styling&#46; To achieve this&#44; <a id="the&#45;samp&#45;element&#58;the&#45;span&#45;element"></a><code>[span](#the-span-element)</code> elements are used&#46;

``` text
<pre><samp><span class="prompt">jdoe@mowmow:~$</span> <kbd>ssh demo.example.com</kbd>
Last login: Tue Apr 12 09:10:17 2005 from mowmow.example.com on pts/1
Linux demo 2.6.10-grsec+gg3+e+fhs6b+nfs+gr0501+++p3+c4a+gr2b-reslog-v6.189 #1 SMP Tue Feb 1 11:22:36 PST 2005 i686 unknown

<span class="prompt">jdoe@demo:~$</span> <span class="cursor">_</span></samp></pre>
```

<strong>Example&#58;</strong>

This third example shows a block of input and its respective output&#46; The example uses both <a id="the&#45;samp&#45;element&#58;the&#45;code&#45;element"></a><code>[code](#the-code-element)</code> and <a id="the&#45;samp&#45;element&#58;the&#45;samp&#45;element&#45;5"></a><code>[samp](#the-samp-element)</code> elements&#46;

``` text
<pre>
<code class="language-javascript">console.log(2.3 + 2.4)</code>
<samp>4.699999999999999</samp>
</pre>
```

#### <a id="the&#45;kbd&#45;element"></a>4&#46;5&#46;18 The <strong><code>kbd</code></strong> element[](#the-kbd-element)

<strong>✔</strong>MDN

[Element&#47;kbd](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/kbd)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android4&#43;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;kbd&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;kbd&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;kbd&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;kbd&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;kbd&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;kbd&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;kbd&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;kbd&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;kbd&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;kbd&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;kbd&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;kbd&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-kbd)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-kbd)&#46;

<strong><a id="the&#45;kbd&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;kbd&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;kbd&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;kbd&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;kbd&#45;element&#58;the&#45;kbd&#45;element"></a><code>[kbd](#the-kbd-element)</code> element <a id="the&#45;kbd&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) user input &#40;typically keyboard input&#44; although it may also be used to represent other input&#44; such as voice commands&#41;&#46;

When the <a id="the&#45;kbd&#45;element&#58;the&#45;kbd&#45;element&#45;2"></a><code>[kbd](#the-kbd-element)</code> element is nested inside a <a id="the&#45;kbd&#45;element&#58;the&#45;samp&#45;element"></a><code>[samp](#the-samp-element)</code> element&#44; it represents the input as it was echoed by the system&#46;

When the <a id="the&#45;kbd&#45;element&#58;the&#45;kbd&#45;element&#45;3"></a><code>[kbd](#the-kbd-element)</code> element <em>contains</em> a <a id="the&#45;kbd&#45;element&#58;the&#45;samp&#45;element&#45;2"></a><code>[samp](#the-samp-element)</code> element&#44; it represents input based on system output&#44; for example invoking a menu item&#46;

When the <a id="the&#45;kbd&#45;element&#58;the&#45;kbd&#45;element&#45;4"></a><code>[kbd](#the-kbd-element)</code> element is nested inside another <a id="the&#45;kbd&#45;element&#58;the&#45;kbd&#45;element&#45;5"></a><code>[kbd](#the-kbd-element)</code> element&#44; it represents an actual key or other single unit of input as appropriate for the input mechanism&#46;

<strong>Example&#58;</strong>

Here the <a id="the&#45;kbd&#45;element&#58;the&#45;kbd&#45;element&#45;6"></a><code>[kbd](#the-kbd-element)</code> element is used to indicate keys to press&#58;

``` text
<p>To make George eat an apple, press <kbd><kbd>Shift</kbd> + <kbd>F3</kbd></kbd></p>
```

In this second example&#44; the user is told to pick a particular menu item&#46; The outer <a id="the&#45;kbd&#45;element&#58;the&#45;kbd&#45;element&#45;7"></a><code>[kbd](#the-kbd-element)</code> element marks up a block of input&#44; with the inner <a id="the&#45;kbd&#45;element&#58;the&#45;kbd&#45;element&#45;8"></a><code>[kbd](#the-kbd-element)</code> elements representing each individual step of the input&#44; and the <a id="the&#45;kbd&#45;element&#58;the&#45;samp&#45;element&#45;3"></a><code>[samp](#the-samp-element)</code> elements inside them indicating that the steps are input based on something being displayed by the system&#44; in this case menu labels&#58;

``` text
<p>To make George eat an apple, select
    <kbd><kbd><samp>File</samp></kbd>|<kbd><samp>Eat Apple...</samp></kbd></kbd>
</p>
```

Such precision isn&#39;t necessary&#59; the following is equally fine&#58;

``` text
<p>To make George eat an apple, select <kbd>File | Eat Apple...</kbd></p>
```

#### <a id="the&#45;sub&#45;and&#45;sup&#45;elements"></a>4&#46;5&#46;19 The <a id="the&#45;sub&#45;element"></a><strong><code>sub</code></strong> and <a id="the&#45;sup&#45;element"></a><strong><code>sup</code></strong> elements[](#the-sub-and-sup-elements)

<strong>✔</strong>MDN

[Element&#47;sub](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sub)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[Element&#47;sup](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sup)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

The <a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;the&#45;sub&#45;and&#45;sup&#45;elements"></a><code>[sub](#the-sub-and-sup-elements)</code> element&#58; [for authors](https://w3c.github.io/html-aria/#el-sub)&#59; [for implementers](https://w3c.github.io/html-aam/#el-sub)&#46;

The <a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;the&#45;sub&#45;and&#45;sup&#45;elements&#45;2"></a><code>[sup](#the-sub-and-sup-elements)</code> element&#58; [for authors](https://w3c.github.io/html-aria/#el-sup)&#59; [for implementers](https://w3c.github.io/html-aam/#el-sup)&#46;

<strong><a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Use <a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;the&#45;sub&#45;and&#45;sup&#45;elements&#45;3"></a><code>[sup](#the-sub-and-sup-elements)</code> element <a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a superscript and the <a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;the&#45;sub&#45;and&#45;sup&#45;elements&#45;4"></a><code>[sub](#the-sub-and-sup-elements)</code> element <a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;represents&#45;2"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a subscript&#46;

These elements must be used only to mark up typographical conventions with specific meanings&#44; not for typographical presentation for presentation&#39;s sake&#46; For example&#44; it would be inappropriate for the <a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;the&#45;sub&#45;and&#45;sup&#45;elements&#45;5"></a><code>[sub](#the-sub-and-sup-elements)</code> and <a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;the&#45;sub&#45;and&#45;sup&#45;elements&#45;6"></a><code>[sup](#the-sub-and-sup-elements)</code> elements to be used in the name of the LaTeX document preparation system&#46; In general&#44; authors should use these elements only if the <em>absence</em> of those elements would change the meaning of the content&#46;

In certain languages&#44; superscripts are part of the typographical conventions for some abbreviations&#46;

<strong>Example&#58;</strong>

``` text
<p>Their names are
<span lang="fr"><abbr>M<sup>lle</sup></abbr> Gwendoline</span> and
<span lang="fr"><abbr>M<sup>me</sup></abbr> Denise</span>.</p>
```

The <a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;the&#45;sub&#45;and&#45;sup&#45;elements&#45;7"></a><code>[sub](#the-sub-and-sup-elements)</code> element can be used inside a <a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;the&#45;var&#45;element"></a><code>[var](#the-var-element)</code> element&#44; for variables that have subscripts&#46;

<strong>Example&#58;</strong>

Here&#44; the <a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;the&#45;sub&#45;and&#45;sup&#45;elements&#45;8"></a><code>[sub](#the-sub-and-sup-elements)</code> element is used to represent the subscript that identifies the variable in a family of variables&#58;

``` text
<p>The coordinate of the <var>i</var>th point is
(<var>x<sub><var>i</var></sub></var>, <var>y<sub><var>i</var></sub></var>).
For example, the 10th point has coordinate
(<var>x<sub>10</sub></var>, <var>y<sub>10</sub></var>).</p>
```

Mathematical expressions often use subscripts and superscripts&#46; Authors are encouraged to use MathML for marking up mathematics&#44; but authors may opt to use <a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;the&#45;sub&#45;and&#45;sup&#45;elements&#45;9"></a><code>[sub](#the-sub-and-sup-elements)</code> and <a id="the&#45;sub&#45;and&#45;sup&#45;elements&#58;the&#45;sub&#45;and&#45;sup&#45;elements&#45;10"></a><code>[sup](#the-sub-and-sup-elements)</code> if detailed mathematical markup is not desired&#46; [&#91;MATHML&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsMATHML)

<strong>Example&#58;</strong>

``` text
<var>E</var>=<var>m</var><var>c</var><sup>2</sup>
```

``` text
f(<var>x</var>, <var>n</var>) = log<sub>4</sub><var>x</var><sup><var>n</var></sup>
```

#### <a id="the&#45;i&#45;element"></a>4&#46;5&#46;20 The <strong><code>i</code></strong> element[](#the-i-element)

<strong>✔</strong>MDN

[Element&#47;i](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/i)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;i&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;i&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;i&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;i&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;i&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;i&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;i&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;i&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;i&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;i&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;i&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;i&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-i)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-i)&#46;

<strong><a id="the&#45;i&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;i&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;i&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;i&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;i&#45;element&#58;the&#45;i&#45;element"></a><code>[i](#the-i-element)</code> element <a id="the&#45;i&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a span of text in an alternate voice or mood&#44; or otherwise offset from the normal prose in a manner indicating a different quality of text&#44; such as a taxonomic designation&#44; a technical term&#44; an idiomatic phrase from another language&#44; transliteration&#44; a thought&#44; or a ship name in Western texts&#46;

Terms in languages different from the main text should be annotated with <a id="the&#45;i&#45;element&#58;attr&#45;lang"></a><code>[lang](html-language-foundations--877d6146487f--b55e73c31978.md#attr-lang)</code> attributes &#40;or&#44; in XML&#44; <a id="the&#45;i&#45;element&#58;attr&#45;xml&#45;lang"></a>[<code>lang</code> attributes in the XML namespace](https://www.w3.org/TR/xml/#sec-lang-tag)&#41;&#46;

<strong>Example&#58;</strong>

The examples below show uses of the <a id="the&#45;i&#45;element&#58;the&#45;i&#45;element&#45;2"></a><code>[i](#the-i-element)</code> element&#58;

``` text
<p>The <i class="taxonomy">Felis silvestris catus</i> is cute.</p>
<p>The term <i>prose content</i> is defined above.</p>
<p>There is a certain <i lang="fr">je ne sais quoi</i> in the air.</p>
```

In the following example&#44; a dream sequence is marked up using <a id="the&#45;i&#45;element&#58;the&#45;i&#45;element&#45;3"></a><code>[i](#the-i-element)</code> elements&#46;

``` text
<p>Raymond tried to sleep.</p>
<p><i>The ship sailed away on Thursday</i>, he
dreamt. <i>The ship had many people aboard, including a beautiful
princess called Carey. He watched her, day-in, day-out, hoping she
would notice him, but she never did.</i></p>
<p><i>Finally one night he picked up the courage to speak with
her—</i></p>
<p>Raymond woke with a start as the fire alarm rang out.</p>
```

Authors can use the <a id="the&#45;i&#45;element&#58;classes"></a><code>[class](html-language-foundations--877d6146487f--b55e73c31978.md#classes)</code> attribute on the <a id="the&#45;i&#45;element&#58;the&#45;i&#45;element&#45;4"></a><code>[i](#the-i-element)</code> element to identify why the element is being used&#44; so that if the style of a particular use &#40;e&#46;g&#46; dream sequences as opposed to taxonomic terms&#41; is to be changed at a later date&#44; the author doesn&#39;t have to go through the entire document &#40;or series of related documents&#41; annotating each use&#46;

Authors are encouraged to consider whether other elements might be more applicable than the <a id="the&#45;i&#45;element&#58;the&#45;i&#45;element&#45;5"></a><code>[i](#the-i-element)</code> element&#44; for instance the <a id="the&#45;i&#45;element&#58;the&#45;em&#45;element"></a><code>[em](#the-em-element)</code> element for marking up stress emphasis&#44; or the <a id="the&#45;i&#45;element&#58;the&#45;dfn&#45;element"></a><code>[dfn](#the-dfn-element)</code> element to mark up the defining instance of a term&#46;

<strong>Note&#58;</strong>

Style sheets can be used to format <a id="the&#45;i&#45;element&#58;the&#45;i&#45;element&#45;6"></a><code>[i](#the-i-element)</code> elements&#44; just like any other element can be restyled&#46; Thus&#44; it is not the case that content in <a id="the&#45;i&#45;element&#58;the&#45;i&#45;element&#45;7"></a><code>[i](#the-i-element)</code> elements will necessarily be italicized&#46;

#### <a id="the&#45;b&#45;element"></a>4&#46;5&#46;21 The <strong><code>b</code></strong> element[](#the-b-element)

<strong>✔</strong>MDN

[Element&#47;b](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/b)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android4&#43;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;b&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;b&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;b&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;b&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;b&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;b&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;b&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;b&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;b&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;b&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;b&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;b&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-b)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-b)&#46;

<strong><a id="the&#45;b&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;b&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;b&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;b&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;b&#45;element&#58;the&#45;b&#45;element"></a><code>[b](#the-b-element)</code> element <a id="the&#45;b&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a span of text to which attention is being drawn for utilitarian purposes without conveying any extra importance and with no implication of an alternate voice or mood&#44; such as key words in a document abstract&#44; product names in a review&#44; actionable words in interactive text&#45;driven software&#44; or an article lede&#46;

<strong>Example&#58;</strong>

The following example shows a use of the <a id="the&#45;b&#45;element&#58;the&#45;b&#45;element&#45;2"></a><code>[b](#the-b-element)</code> element to highlight key words without marking them up as important&#58;

``` text
<p>The <b>frobonitor</b> and <b>barbinator</b> components are fried.</p>
```

<strong>Example&#58;</strong>

In the following example&#44; objects in a text adventure are highlighted as being special by use of the <a id="the&#45;b&#45;element&#58;the&#45;b&#45;element&#45;3"></a><code>[b](#the-b-element)</code> element&#46;

``` text
<p>You enter a small room. Your <b>sword</b> glows
brighter. A <b>rat</b> scurries past the corner wall.</p>
```

<strong>Example&#58;</strong>

Another case where the <a id="the&#45;b&#45;element&#58;the&#45;b&#45;element&#45;4"></a><code>[b](#the-b-element)</code> element is appropriate is in marking up the lede &#40;or lead&#41; sentence or paragraph&#46; The following example shows how a [BBC article about kittens adopting a rabbit as their own](http://news.bbc.co.uk/2/hi/uk_news/scotland/north_east/7101506.stm) could be marked up&#58;

``` text
<article>
 <h2>Kittens 'adopted' by pet rabbit</h2>
 <p><b class="lede">Six abandoned kittens have found an
 unexpected new mother figure — a pet rabbit.</b></p>
 <p>Veterinary nurse Melanie Humble took the three-week-old
 kittens to her Aberdeen home.</p>
[...]
```

As with the <a id="the&#45;b&#45;element&#58;the&#45;i&#45;element"></a><code>[i](#the-i-element)</code> element&#44; authors can use the <a id="the&#45;b&#45;element&#58;classes"></a><code>[class](html-language-foundations--877d6146487f--b55e73c31978.md#classes)</code> attribute on the <a id="the&#45;b&#45;element&#58;the&#45;b&#45;element&#45;5"></a><code>[b](#the-b-element)</code> element to identify why the element is being used&#44; so that if the style of a particular use is to be changed at a later date&#44; the author doesn&#39;t have to go through annotating each use&#46;

The <a id="the&#45;b&#45;element&#58;the&#45;b&#45;element&#45;6"></a><code>[b](#the-b-element)</code> element should be used as a last resort when no other element is more appropriate&#46; In particular&#44; headings should use the <a id="the&#45;b&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements"></a><code>[h1](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code> to <a id="the&#45;b&#45;element&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;2"></a><code>[h6](#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code> elements&#44; stress emphasis should use the <a id="the&#45;b&#45;element&#58;the&#45;em&#45;element"></a><code>[em](#the-em-element)</code> element&#44; importance should be denoted with the <a id="the&#45;b&#45;element&#58;the&#45;strong&#45;element"></a><code>[strong](#the-strong-element)</code> element&#44; and text marked or highlighted should use the <a id="the&#45;b&#45;element&#58;the&#45;mark&#45;element"></a><code>[mark](#the-mark-element)</code> element&#46;

<strong>Example&#58;</strong>

The following would be <em>incorrect</em> usage&#58;

``` text
<p><b>WARNING!</b> Do not frob the barbinator!</p>
```

In the previous example&#44; the correct element to use would have been <a id="the&#45;b&#45;element&#58;the&#45;strong&#45;element&#45;2"></a><code>[strong](#the-strong-element)</code>&#44; not <a id="the&#45;b&#45;element&#58;the&#45;b&#45;element&#45;7"></a><code>[b](#the-b-element)</code>&#46;

<strong>Note&#58;</strong>

Style sheets can be used to format <a id="the&#45;b&#45;element&#58;the&#45;b&#45;element&#45;8"></a><code>[b](#the-b-element)</code> elements&#44; just like any other element can be restyled&#46; Thus&#44; it is not the case that content in <a id="the&#45;b&#45;element&#58;the&#45;b&#45;element&#45;9"></a><code>[b](#the-b-element)</code> elements will necessarily be boldened&#46;

#### <a id="the&#45;u&#45;element"></a>4&#46;5&#46;22 The <strong><code>u</code></strong> element[](#the-u-element)

<strong>✔</strong>MDN

[Element&#47;u](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/u)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android4&#43;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;u&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;u&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;u&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;u&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;u&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;u&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;u&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;u&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;u&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;u&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;u&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;u&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-u)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-u)&#46;

<strong><a id="the&#45;u&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;u&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;u&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;u&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;u&#45;element&#58;the&#45;u&#45;element"></a><code>[u](#the-u-element)</code> element <a id="the&#45;u&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a span of text with an unarticulated&#44; though explicitly rendered&#44; non&#45;textual annotation&#44; such as labeling the text as being a proper name in Chinese text &#40;a Chinese proper name mark&#41;&#44; or labeling the text as being misspelt&#46;

In most cases&#44; another element is likely to be more appropriate&#58; for marking stress emphasis&#44; the <a id="the&#45;u&#45;element&#58;the&#45;em&#45;element"></a><code>[em](#the-em-element)</code> element should be used&#59; for marking key words or phrases either the <a id="the&#45;u&#45;element&#58;the&#45;b&#45;element"></a><code>[b](#the-b-element)</code> element or the <a id="the&#45;u&#45;element&#58;the&#45;mark&#45;element"></a><code>[mark](#the-mark-element)</code> element should be used&#44; depending on the context&#59; for marking book titles&#44; the <a id="the&#45;u&#45;element&#58;the&#45;cite&#45;element"></a><code>[cite](#the-cite-element)</code> element should be used&#59; for labeling text with explicit textual annotations&#44; the <a id="the&#45;u&#45;element&#58;the&#45;ruby&#45;element"></a><code>[ruby](#the-ruby-element)</code> element should be used&#59; for technical terms&#44; taxonomic designation&#44; transliteration&#44; a thought&#44; or for labeling ship names in Western texts&#44; the <a id="the&#45;u&#45;element&#58;the&#45;i&#45;element"></a><code>[i](#the-i-element)</code> element should be used&#46;

<strong>Note&#58;</strong>

The default rendering of the <a id="the&#45;u&#45;element&#58;the&#45;u&#45;element&#45;2"></a><code>[u](#the-u-element)</code> element in visual presentations clashes with the conventional rendering of hyperlinks &#40;underlining&#41;&#46; Authors are encouraged to avoid using the <a id="the&#45;u&#45;element&#58;the&#45;u&#45;element&#45;3"></a><code>[u](#the-u-element)</code> element where it could be confused for a hyperlink&#46;

<strong>Example&#58;</strong>

In this example&#44; a <a id="the&#45;u&#45;element&#58;the&#45;u&#45;element&#45;4"></a><code>[u](#the-u-element)</code> element is used to mark a word as misspelt&#58;

``` text
<p>The <u>see</u> is full of fish.</p>
```

#### <a id="the&#45;mark&#45;element"></a>4&#46;5&#46;23 The <strong><code>mark</code></strong> element[](#the-mark-element)

<strong>✔</strong>MDN

[Element&#47;mark](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/mark)

Support in all current engines&#46;

Firefox4&#43;Safari5&#46;1&#43;Chrome7&#43;

------------------------------------------------------------------------

Opera11&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer9&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;mark&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;mark&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;mark&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;mark&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;mark&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;mark&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;mark&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;mark&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;mark&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;mark&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;mark&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;mark&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-mark)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-mark)&#46;

<strong><a id="the&#45;mark&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;mark&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;mark&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;mark&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;mark&#45;element&#58;the&#45;mark&#45;element"></a><code>[mark](#the-mark-element)</code> element <a id="the&#45;mark&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a run of text in one document marked or highlighted for <a id="the&#45;mark&#45;element&#58;referenced"></a>[reference](html-language-foundations--877d6146487f--b55e73c31978.md#referenced) purposes&#44; due to its relevance in another context&#46; When used in a quotation or other block of text referred to from the prose&#44; it indicates a highlight that was not originally present but which has been added to bring the reader&#39;s attention to a part of the text that might not have been considered important by the original author when the block was originally written&#44; but which is now under previously unexpected scrutiny&#46; When used in the main prose of a document&#44; it indicates a part of the document that has been highlighted due to its likely relevance to the user&#39;s current activity&#46;

<strong>Example&#58;</strong>

This example shows how the <a id="the&#45;mark&#45;element&#58;the&#45;mark&#45;element&#45;2"></a><code>[mark](#the-mark-element)</code> element can be used to bring attention to a particular part of a quotation&#58;

``` text
<p lang="en-US">Consider the following quote:</p>
<blockquote lang="en-GB">
 <p>Look around and you will find, no-one's really
 <mark>colour</mark> blind.</p>
</blockquote>
<p lang="en-US">As we can tell from the <em>spelling</em> of the word,
the person writing this quote is clearly not American.</p>
```

&#40;If the goal was to mark the element as misspelt&#44; however&#44; the <a id="the&#45;mark&#45;element&#58;the&#45;u&#45;element"></a><code>[u](#the-u-element)</code> element&#44; possibly with a class&#44; would be more appropriate&#46;&#41;

<strong>Example&#58;</strong>

Another example of the <a id="the&#45;mark&#45;element&#58;the&#45;mark&#45;element&#45;3"></a><code>[mark](#the-mark-element)</code> element is highlighting parts of a document that are matching some search string&#46; If someone looked at a document&#44; and the server knew that the user was searching for the word &#34;kitten&#34;&#44; then the server might return the document with one paragraph modified as follows&#58;

``` text
<p>I also have some <mark>kitten</mark>s who are visiting me
these days. They're really cute. I think they like my garden! Maybe I
should adopt a <mark>kitten</mark>.</p>
```

<strong>Example&#58;</strong>

In the following snippet&#44; a paragraph of text refers to a specific part of a code fragment&#46;

``` text
<p>The highlighted part below is where the error lies:</p>
<pre><code>var i: Integer;
begin
   i := <mark>1.1</mark>;
end.</code></pre>
```

This is separate from <em>syntax highlighting</em>&#44; for which <a id="the&#45;mark&#45;element&#58;the&#45;span&#45;element"></a><code>[span](#the-span-element)</code> is more appropriate&#46; Combining both&#44; one would get&#58;

``` text
<p>The highlighted part below is where the error lies:</p>
<pre><code><span class=keyword>var</span> <span class=ident>i</span>: <span class=type>Integer</span>;
<span class=keyword>begin</span>
   <span class=ident>i</span> := <span class=literal><mark>1.1</mark></span>;
<span class=keyword>end</span>.</code></pre>
```

<strong>Example&#58;</strong>

This is another example showing the use of <a id="the&#45;mark&#45;element&#58;the&#45;mark&#45;element&#45;4"></a><code>[mark](#the-mark-element)</code> to highlight a part of quoted text that was originally not emphasized&#46; In this example&#44; common typographic conventions have led the author to explicitly style <a id="the&#45;mark&#45;element&#58;the&#45;mark&#45;element&#45;5"></a><code>[mark](#the-mark-element)</code> elements in quotes to render in italics&#46;

``` text
<style>
 blockquote mark, q mark {
   font: inherit; font-style: italic;
   text-decoration: none;
   background: transparent; color: inherit;
 }
 .bubble em {
   font: inherit; font-size: larger;
   text-decoration: underline;
 }
</style>
<article>
 <h1>She knew</h1>
 <p>Did you notice the subtle joke in the joke on panel 4?</p>
 <blockquote>
  <p class="bubble">I didn't <em>want</em> to believe. <mark>Of course
  on some level I realized it was a known-plaintext attack.</mark> But I
  couldn't admit it until I saw for myself.</p>
 </blockquote>
 <p>(Emphasis mine.) I thought that was great. It's so pedantic, yet it
 explains everything neatly.</p>
</article>
```

Note&#44; incidentally&#44; the distinction between the <a id="the&#45;mark&#45;element&#58;the&#45;em&#45;element"></a><code>[em](#the-em-element)</code> element in this example&#44; which is part of the original text being quoted&#44; and the <a id="the&#45;mark&#45;element&#58;the&#45;mark&#45;element&#45;6"></a><code>[mark](#the-mark-element)</code> element&#44; which is highlighting a part for comment&#46;

<strong>Example&#58;</strong>

The following example shows the difference between denoting the <em>importance</em> of a span of text &#40;<a id="the&#45;mark&#45;element&#58;the&#45;strong&#45;element"></a><code>[strong](#the-strong-element)</code>&#41; as opposed to denoting the <em>relevance</em> of a span of text &#40;<a id="the&#45;mark&#45;element&#58;the&#45;mark&#45;element&#45;7"></a><code>[mark](#the-mark-element)</code>&#41;&#46; It is an extract from a textbook&#44; where the extract has had the parts relevant to the exam highlighted&#46; The safety warnings&#44; important though they may be&#44; are apparently not relevant to the exam&#46;

``` text
<h3>Wormhole Physics Introduction</h3>

<p><mark>A wormhole in normal conditions can be held open for a
maximum of just under 39 minutes.</mark> Conditions that can increase
the time include a powerful energy source coupled to one or both of
the gates connecting the wormhole, and a large gravity well (such as a
black hole).</p>

<p><mark>Momentum is preserved across the wormhole. Electromagnetic
radiation can travel in both directions through a wormhole,
but matter cannot.</mark></p>

<p>When a wormhole is created, a vortex normally forms.
<strong>Warning: The vortex caused by the wormhole opening will
annihilate anything in its path.</strong> Vortexes can be avoided when
using sufficiently advanced dialing technology.</p>

<p><mark>An obstruction in a gate will prevent it from accepting a
wormhole connection.</mark></p>
```

#### <a id="the&#45;bdi&#45;element"></a>4&#46;5&#46;24 The <strong><code>bdi</code></strong> element[](#the-bdi-element)

<strong>✔</strong>MDN

[Element&#47;bdi](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/bdi)

Support in all current engines&#46;

Firefox10&#43;Safari6&#43;Chrome16&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android37&#43;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;bdi&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;bdi&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;bdi&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;bdi&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;bdi&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;bdi&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;bdi&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;bdi&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;bdi&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;bdi&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;bdi&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

Also&#44; the <a id="the&#45;bdi&#45;element&#58;attr&#45;dir"></a><code>[dir](html-language-foundations--877d6146487f--b55e73c31978.md#attr-dir)</code> global attribute has special semantics on this element&#46;

<strong><a id="the&#45;bdi&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-bdi)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-bdi)&#46;

<strong><a id="the&#45;bdi&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;bdi&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;bdi&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;bdi&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;bdi&#45;element&#58;the&#45;bdi&#45;element"></a><code>[bdi](#the-bdi-element)</code> element <a id="the&#45;bdi&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a span of text that is to be isolated from its surroundings for the purposes of bidirectional text formatting&#46; [&#91;BIDI&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsBIDI)

<strong>Note&#58;</strong>

The <a id="the&#45;bdi&#45;element&#58;attr&#45;dir&#45;2"></a><code>[dir](html-language-foundations--877d6146487f--b55e73c31978.md#attr-dir)</code> global attribute defaults to <a id="the&#45;bdi&#45;element&#58;attr&#45;dir&#45;auto"></a><code>[auto](html-language-foundations--877d6146487f--b55e73c31978.md#attr-dir-auto)</code> on this element &#40;it never inherits from the parent element like with other elements&#41;&#46;

<strong>Note&#58;</strong>

This element [has rendering requirements involving the bidirectional algorithm](html-language-foundations--877d6146487f--b55e73c31978.md#bidireq)&#46;

<strong>Example&#58;</strong>

This element is especially useful when embedding user&#45;generated content with an unknown directionality&#46;

In this example&#44; usernames are shown along with the number of posts that the user has submitted&#46; If the <a id="the&#45;bdi&#45;element&#58;the&#45;bdi&#45;element&#45;2"></a><code>[bdi](#the-bdi-element)</code> element were not used&#44; the username of the Arabic user would end up confusing the text &#40;the bidirectional algorithm would put the colon and the number &#34;3&#34; next to the word &#34;User&#34; rather than next to the word &#34;posts&#34;&#41;&#46;

``` text
<ul>
 <li>User <bdi>jcranmer</bdi>: 12 posts.
 <li>User <bdi>hober</bdi>: 5 posts.
 <li>User <bdi>إيان</bdi>: 3 posts.
</ul>
```

<figure>
<img src="https://html.spec.whatwg.org/images/sample-bdi.png" />
<figcaption>When using the <a id="the&#45;bdi&#45;element&#58;the&#45;bdi&#45;element&#45;3"></a><code><a href="#the-bdi-element">bdi</a></code> element&#44; the username acts as expected&#46;</figcaption>
</figure>

<figure>
<img src="https://html.spec.whatwg.org/images/sample-not-bdi.png" />
<figcaption>If the <a id="the&#45;bdi&#45;element&#58;the&#45;bdi&#45;element&#45;4"></a><code><a href="#the-bdi-element">bdi</a></code> element were to be replaced by a <a id="the&#45;bdi&#45;element&#58;the&#45;b&#45;element"></a><code><a href="#the-b-element">b</a></code> element&#44; the username would confuse the bidirectional algorithm and the third bullet would end up saying &#34;User 3 &#58;&#34;&#44; followed by the Arabic name &#40;right&#45;to&#45;left&#41;&#44; followed by &#34;posts&#34; and a period&#46;</figcaption>
</figure>

#### <a id="the&#45;bdo&#45;element"></a>4&#46;5&#46;25 The <strong><code>bdo</code></strong> element[](#the-bdo-element)

<strong>✔</strong>MDN

[Element&#47;bdo](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/bdo)

Support in all current engines&#46;

Firefox10&#43;Safari4&#43;Chrome15&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;bdo&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;bdo&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;bdo&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;bdo&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;bdo&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;bdo&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;bdo&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;bdo&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;bdo&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;bdo&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;bdo&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

Also&#44; the <a id="the&#45;bdo&#45;element&#58;attr&#45;dir"></a><code>[dir](html-language-foundations--877d6146487f--b55e73c31978.md#attr-dir)</code> global attribute has special semantics on this element&#46;

<strong><a id="the&#45;bdo&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-bdo)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-bdo)&#46;

<strong><a id="the&#45;bdo&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;bdo&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;bdo&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;bdo&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;bdo&#45;element&#58;the&#45;bdo&#45;element"></a><code>[bdo](#the-bdo-element)</code> element <a id="the&#45;bdo&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) explicit text directionality formatting control for its children&#46; It allows authors to override the Unicode bidirectional algorithm by explicitly specifying a direction override&#46; [&#91;BIDI&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsBIDI)

Authors must specify the <a id="the&#45;bdo&#45;element&#58;attr&#45;dir&#45;2"></a><code>[dir](html-language-foundations--877d6146487f--b55e73c31978.md#attr-dir)</code> attribute on this element&#44; with the value <a id="the&#45;bdo&#45;element&#58;attr&#45;dir&#45;ltr"></a><code>[ltr](html-language-foundations--877d6146487f--b55e73c31978.md#attr-dir-ltr)</code> to specify a left&#45;to&#45;right override and with the value <a id="the&#45;bdo&#45;element&#58;attr&#45;dir&#45;rtl"></a><code>[rtl](html-language-foundations--877d6146487f--b55e73c31978.md#attr-dir-rtl)</code> to specify a right&#45;to&#45;left override&#46; The <a id="the&#45;bdo&#45;element&#58;attr&#45;dir&#45;auto"></a><code>[auto](html-language-foundations--877d6146487f--b55e73c31978.md#attr-dir-auto)</code> value must not be specified&#46;

<strong>Note&#58;</strong>

This element [has rendering requirements involving the bidirectional algorithm](html-language-foundations--877d6146487f--b55e73c31978.md#bidireq)&#46;

#### <a id="the&#45;span&#45;element"></a>4&#46;5&#46;26 The <strong><code>span</code></strong> element[](#the-span-element)

<strong>✔</strong>MDN

[Element&#47;span](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong>✔</strong>MDN

[HTMLSpanElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLSpanElement)

Support in all current engines&#46;

Firefox1&#43;Safari6&#43;Chrome15&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;span&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;span&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;span&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;span&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;span&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;span&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;span&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;span&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;span&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;span&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;span&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;span&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-span)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-span)&#46;

<strong><a id="the&#45;span&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;span&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;span&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

<a id="htmlspanelement"></a><a id="the&#45;span&#45;element&#58;htmlelement"></a><a id="the&#45;span&#45;element&#58;htmlconstructor"></a>

``` text
[Exposed=Window]
interface HTMLSpanElement : HTMLElement {
  [HTMLConstructor] constructor();
};
```

The <a id="the&#45;span&#45;element&#58;the&#45;span&#45;element"></a><code>[span](#the-span-element)</code> element doesn&#39;t mean anything on its own&#44; but can be useful when used together with the <a id="the&#45;span&#45;element&#58;global&#45;attributes&#45;2"></a>[global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)&#44; e&#46;g&#46; <a id="the&#45;span&#45;element&#58;classes"></a><code>[class](html-language-foundations--877d6146487f--b55e73c31978.md#classes)</code>&#44; <a id="the&#45;span&#45;element&#58;attr&#45;lang"></a><code>[lang](html-language-foundations--877d6146487f--b55e73c31978.md#attr-lang)</code>&#44; or <a id="the&#45;span&#45;element&#58;attr&#45;dir"></a><code>[dir](html-language-foundations--877d6146487f--b55e73c31978.md#attr-dir)</code>&#46; It <a id="the&#45;span&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) its children&#46;

<strong>Example&#58;</strong>

In this example&#44; a code fragment is marked up using <a id="the&#45;span&#45;element&#58;the&#45;span&#45;element&#45;2"></a><code>[span](#the-span-element)</code> elements and <a id="the&#45;span&#45;element&#58;classes&#45;2"></a><code>[class](html-language-foundations--877d6146487f--b55e73c31978.md#classes)</code> attributes so that its keywords and identifiers can be color&#45;coded from CSS&#58;

``` text
<pre><code class="lang-c"><span class="keyword">for</span> (<span class="ident">j</span> = 0; <span class="ident">j</span> &lt; 256; <span class="ident">j</span>++) {
  <span class="ident">i_t3</span> = (<span class="ident">i_t3</span> & 0x1ffff) | (<span class="ident">j</span> &lt;&lt; 17);
  <span class="ident">i_t6</span> = (((((((<span class="ident">i_t3</span> >> 3) ^ <span class="ident">i_t3</span>) >> 1) ^ <span class="ident">i_t3</span>) >> 8) ^ <span class="ident">i_t3</span>) >> 5) & 0xff;
  <span class="keyword">if</span> (<span class="ident">i_t6</span> == <span class="ident">i_t1</span>)
    <span class="keyword">break</span>;
}</code></pre>
```

#### <a id="the&#45;br&#45;element"></a>4&#46;5&#46;27 The <strong><code>br</code></strong> element[](#the-br-element)

<strong>✔</strong>MDN

[Element&#47;br](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/br)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong>✔</strong>MDN

[HTMLBRElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLBRElement)

Support in all current engines&#46;

Firefox1&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

<strong><a id="the&#45;br&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;br&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;br&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;br&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;br&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;br&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;br&#45;element&#58;concept&#45;content&#45;nothing"></a>[Nothing](html-language-foundations--877d6146487f--b55e73c31978.md#concept-content-nothing)&#46;

<strong><a id="the&#45;br&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

No <a id="the&#45;br&#45;element&#58;syntax&#45;end&#45;tag"></a>[end tag](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#syntax-end-tag)&#46;

<strong><a id="the&#45;br&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;br&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;br&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-br)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-br)&#46;

<strong><a id="the&#45;br&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;br&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;br&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

<a id="htmlbrelement"></a><a id="the&#45;br&#45;element&#58;htmlelement"></a><a id="the&#45;br&#45;element&#58;htmlconstructor"></a>

``` text
[Exposed=Window]
interface HTMLBRElement : HTMLElement {
  [HTMLConstructor] constructor();

  // also has obsolete members
};
```

The <a id="the&#45;br&#45;element&#58;the&#45;br&#45;element"></a><code>[br](#the-br-element)</code> element <a id="the&#45;br&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a line break&#46;

<strong>Note&#58;</strong>

While line breaks are usually represented in visual media by physically moving subsequent text to a new line&#44; a style sheet or user agent would be equally justified in causing line breaks to be rendered in a different manner&#44; for instance as green dots&#44; or as extra spacing&#46;

<a id="the&#45;br&#45;element&#58;the&#45;br&#45;element&#45;2"></a><code>[br](#the-br-element)</code> elements must be used only for line breaks that are actually part of the content&#44; as in poems or addresses&#46;

<strong>Example&#58;</strong>

The following example is correct usage of the <a id="the&#45;br&#45;element&#58;the&#45;br&#45;element&#45;3"></a><code>[br](#the-br-element)</code> element&#58;

``` text
<p>P. Sherman<br>
42 Wallaby Way<br>
Sydney</p>
```

<a id="the&#45;br&#45;element&#58;the&#45;br&#45;element&#45;4"></a><code>[br](#the-br-element)</code> elements must not be used for separating thematic groups in a paragraph&#46;

<strong>Example&#58;</strong>

The following examples are non&#45;conforming&#44; as they abuse the <a id="the&#45;br&#45;element&#58;the&#45;br&#45;element&#45;5"></a><code>[br](#the-br-element)</code> element&#58;

``` text
<p><a ...>34 comments.</a><br>
<a ...>Add a comment.</a></p>
```

``` text
<p><label>Name: <input name="name"></label><br>
<label>Address: <input name="address"></label></p>
```

Here are alternatives to the above&#44; which are correct&#58;

``` text
<p><a ...>34 comments.</a></p>
<p><a ...>Add a comment.</a></p>
```

``` text
<p><label>Name: <input name="name"></label></p>
<p><label>Address: <input name="address"></label></p>
```

If a <a id="the&#45;br&#45;element&#58;paragraph"></a>[paragraph](html-language-foundations--877d6146487f--b55e73c31978.md#paragraph) consists of nothing but a single <a id="the&#45;br&#45;element&#58;the&#45;br&#45;element&#45;6"></a><code>[br](#the-br-element)</code> element&#44; it represents a placeholder blank line &#40;e&#46;g&#46; as in a template&#41;&#46; Such blank lines must not be used for presentation purposes&#46;

Any content inside <a id="the&#45;br&#45;element&#58;the&#45;br&#45;element&#45;7"></a><code>[br](#the-br-element)</code> elements must not be considered part of the surrounding text&#46;

<strong>Note&#58;</strong>

This element [has rendering requirements involving the bidirectional algorithm](html-language-foundations--877d6146487f--b55e73c31978.md#bidireq)&#46;

#### <a id="the&#45;wbr&#45;element"></a>4&#46;5&#46;28 The <strong><code>wbr</code></strong> element[](#the-wbr-element)

<strong>✔</strong>MDN

[Element&#47;wbr](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/wbr)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera11&#46;6&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet Explorer5&#46;5–7

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#43;

<strong><a id="the&#45;wbr&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;wbr&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;wbr&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<strong><a id="the&#45;wbr&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;wbr&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;wbr&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;wbr&#45;element&#58;concept&#45;content&#45;nothing"></a>[Nothing](html-language-foundations--877d6146487f--b55e73c31978.md#concept-content-nothing)&#46;

<strong><a id="the&#45;wbr&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

No <a id="the&#45;wbr&#45;element&#58;syntax&#45;end&#45;tag"></a>[end tag](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#syntax-end-tag)&#46;

<strong><a id="the&#45;wbr&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;wbr&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<strong><a id="the&#45;wbr&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-wbr)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-wbr)&#46;

<strong><a id="the&#45;wbr&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;wbr&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default)&#46;

<strong><a id="the&#45;wbr&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;wbr&#45;element&#58;htmlelement"></a><code>[HTMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlelement)</code>&#46;

The <a id="the&#45;wbr&#45;element&#58;the&#45;wbr&#45;element"></a><code>[wbr](#the-wbr-element)</code> element <a id="the&#45;wbr&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a line break opportunity&#46;

<strong>Example&#58;</strong>

In the following example&#44; someone is quoted as saying something which&#44; for effect&#44; is written as one long word&#46; However&#44; to ensure that the text can be wrapped in a readable fashion&#44; the individual words in the quote are separated using a <a id="the&#45;wbr&#45;element&#58;the&#45;wbr&#45;element&#45;2"></a><code>[wbr](#the-wbr-element)</code> element&#46;

``` text
<p>So then she pointed at the tiger and screamed
"there<wbr>is<wbr>no<wbr>way<wbr>you<wbr>are<wbr>ever<wbr>going<wbr>to<wbr>catch<wbr>me"!</p>
```

Any content inside <a id="the&#45;wbr&#45;element&#58;the&#45;wbr&#45;element&#45;3"></a><code>[wbr](#the-wbr-element)</code> elements must not be considered part of the surrounding text&#46;

``` text
var wbr = document.createElement("wbr");
wbr.textContent = "This is wrong";
document.body.appendChild(wbr);
```

<strong>Note&#58;</strong>

This element [has rendering requirements involving the bidirectional algorithm](html-language-foundations--877d6146487f--b55e73c31978.md#bidireq)&#46;

#### <a id="usage&#45;summary"></a>4&#46;5&#46;29 Usage summary[](#usage-summary)

<em>This section is non&#45;normative&#46;</em>

<table><thead><tr><th>Element      </th><th>Purpose      </th><th>Example    </th></tr></thead><tbody><tr><td><code id="usage-summary:the-a-element"><a href="#the-a-element">a</a></code>      </td><td>Hyperlinks      </td><td><code class="example">Visit my &lt;a href&#61;&#34;drinks&#46;html&#34;&gt;drinks&lt;&#47;a&gt; page&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-em-element"><a href="#the-em-element">em</a></code>      </td><td>Stress emphasis      </td><td><code class="example">I must say I &lt;em&gt;adore&lt;&#47;em&gt; lemonade&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-strong-element"><a href="#the-strong-element">strong</a></code>      </td><td>Importance      </td><td><code class="example">This tea is &lt;strong&gt;very hot&lt;&#47;strong&gt;&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-small-element"><a href="#the-small-element">small</a></code>      </td><td>Side comments      </td><td><code class="example">These grapes are made into wine&#46; &lt;small&gt;Alcohol is addictive&#46;&lt;&#47;small&gt;</code>      </td></tr><tr><td><code id="usage-summary:the-s-element"><a href="#the-s-element">s</a></code>      </td><td>Inaccurate text      </td><td><code class="example">Price&#58; &lt;s&gt;£4&#46;50&lt;&#47;s&gt; £2&#46;00&#33;</code>      </td></tr><tr><td><code id="usage-summary:the-cite-element"><a href="#the-cite-element">cite</a></code>      </td><td>Titles of works      </td><td><code class="example">The case &lt;cite&gt;Hugo v&#46; Danielle&lt;&#47;cite&gt; is relevant here&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-q-element"><a href="#the-q-element">q</a></code>      </td><td>Quotations      </td><td><code class="example">The judge said &lt;q&gt;You can drink water from the fish tank&lt;&#47;q&gt; but advised against it&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-dfn-element"><a href="#the-dfn-element">dfn</a></code>      </td><td>Defining instance      </td><td><code class="example">The term &lt;dfn&gt;organic food&lt;&#47;dfn&gt; refers to food produced without synthetic chemicals&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-abbr-element"><a href="#the-abbr-element">abbr</a></code>      </td><td>Abbreviations      </td><td><code class="example">Organic food in Ireland is certified by the &lt;abbr title&#61;&#34;Irish Organic Farmers and Growers Association&#34;&gt;IOFGA&lt;&#47;abbr&gt;&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-ruby-element"><a href="#the-ruby-element">ruby</a></code>&#44; <code id="usage-summary:the-rt-element"><a href="#the-rt-element">rt</a></code>&#44; <code id="usage-summary:the-rp-element"><a href="#the-rp-element">rp</a></code>      </td><td>Ruby annotations      </td><td><code class="example">&lt;ruby&gt; OJ &lt;rp&gt;&#40;&lt;rt&gt;Orange Juice&lt;rp&gt;&#41;&lt;&#47;ruby&gt;</code>      </td></tr><tr><td><code id="usage-summary:the-data-element"><a href="#the-data-element">data</a></code>      </td><td>Machine&#45;readable equivalent      </td><td><code class="example">Available starting today&#33; &lt;data value&#61;&#34;UPC&#58;022014640201&#34;&gt;North Coast Organic Apple Cider&lt;&#47;data&gt;</code>      </td></tr><tr><td><code id="usage-summary:the-time-element"><a href="#the-time-element">time</a></code>      </td><td>Machine&#45;readable equivalent of date&#45; or time&#45;related data      </td><td><code class="example">Available starting on &lt;time datetime&#61;&#34;2011&#45;11&#45;18&#34;&gt;November 18th&lt;&#47;time&gt;&#33;</code>      </td></tr><tr><td><code id="usage-summary:the-code-element"><a href="#the-code-element">code</a></code>      </td><td>Computer code      </td><td><code class="example">The &lt;code&gt;fruitdb&lt;&#47;code&gt; program can be used for tracking fruit production&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-var-element"><a href="#the-var-element">var</a></code>      </td><td>Variables      </td><td><code class="example">If there are &lt;var&gt;n&lt;&#47;var&gt; fruit in the bowl&#44; at least &lt;var&gt;n&lt;&#47;var&gt;÷2 will be ripe&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-samp-element"><a href="#the-samp-element">samp</a></code>      </td><td>Computer output      </td><td><code class="example">The computer said &lt;samp&gt;Unknown error &#45;3&lt;&#47;samp&gt;&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-kbd-element"><a href="#the-kbd-element">kbd</a></code>      </td><td>User input      </td><td><code class="example">Hit &lt;kbd&gt;F1&lt;&#47;kbd&gt; to continue&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-sub-and-sup-elements"><a href="#the-sub-and-sup-elements">sub</a></code>      </td><td>Subscripts      </td><td><code class="example">Water is H&lt;sub&gt;2&lt;&#47;sub&gt;O&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-sub-and-sup-elements-2"><a href="#the-sub-and-sup-elements">sup</a></code>      </td><td>Superscripts      </td><td><code class="example">The Hydrogen in heavy water is usually &lt;sup&gt;2&lt;&#47;sup&gt;H&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-i-element"><a href="#the-i-element">i</a></code>      </td><td>Alternative voice      </td><td><code class="example">Lemonade consists primarily of &lt;i&gt;Citrus limon&lt;&#47;i&gt;&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-b-element"><a href="#the-b-element">b</a></code>      </td><td>Keywords      </td><td><code class="example">Take a &lt;b&gt;lemon&lt;&#47;b&gt; and squeeze it with a &lt;b&gt;juicer&lt;&#47;b&gt;&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-u-element"><a href="#the-u-element">u</a></code>      </td><td>Annotations      </td><td><code class="example">The mixture of apple juice and &lt;u class&#61;&#34;spelling&#34;&gt;eldeflower&lt;&#47;u&gt; juice is very pleasant&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-mark-element"><a href="#the-mark-element">mark</a></code>      </td><td>Highlight      </td><td><code class="example">Elderflower cordial&#44; with one &lt;mark&gt;part&lt;&#47;mark&gt; cordial to ten &lt;mark&gt;part&lt;&#47;mark&gt;s water&#44; stands a&lt;mark&gt;part&lt;&#47;mark&gt; from the rest&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-bdi-element"><a href="#the-bdi-element">bdi</a></code>      </td><td>Text directionality isolation      </td><td><code class="example">The recommended restaurant is &lt;bdi lang&#61;&#34;&#34;&gt;My Juice Café &#40;At The Beach&#41;&lt;&#47;bdi&gt;&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-bdo-element"><a href="#the-bdo-element">bdo</a></code>      </td><td>Text directionality formatting      </td><td><code class="example">The proposal is to write English&#44; but in reverse order&#46; &#34;Juice&#34; would become &#34;&lt;bdo dir&#61;rtl&gt;Juice&lt;&#47;bdo&gt;&#34;&gt;</code>      </td></tr><tr><td><code id="usage-summary:the-span-element"><a href="#the-span-element">span</a></code>      </td><td>Other      </td><td><code class="example">In French we call it &lt;span lang&#61;&#34;fr&#34;&gt;sirop de sureau&lt;&#47;span&gt;&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-br-element"><a href="#the-br-element">br</a></code>      </td><td>Line break      </td><td><code class="example">Simply Orange Juice Company&lt;br&gt;Apopka&#44; FL 32703&lt;br&gt;U&#46;S&#46;A&#46;</code>      </td></tr><tr><td><code id="usage-summary:the-wbr-element"><a href="#the-wbr-element">wbr</a></code>      </td><td>Line breaking opportunity      </td><td><code class="example">www&#46;simply&lt;wbr&gt;orange&lt;wbr&gt;juice&#46;com</code>    </td></tr></tbody></table>

### <a id="links"></a>4&#46;6 Links[](#links)

#### <a id="introduction&#45;2"></a>4&#46;6&#46;1 Introduction[](#introduction-2)

Links are a conceptual construct&#44; created by <a id="introduction&#45;2&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code>&#44; <a id="introduction&#45;2&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code>&#44; <a id="introduction&#45;2&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code>&#44; and <a id="introduction&#45;2&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements&#44; that <a id="introduction&#45;2&#58;represents"></a>[represent](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a connection between two resources&#44; one of which is the current <a id="introduction&#45;2&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#46; There are three kinds of links in HTML&#58;

<strong><a id="external&#45;resource&#45;link"></a><strong>Links to external resources</strong></strong>

These are links to resources that are to be used to augment the current document&#44; generally automatically processed by the user agent&#46; All <a id="introduction&#45;2&#58;external&#45;resource&#45;link"></a>[external resource links](#external-resource-link) have a <a id="introduction&#45;2&#58;fetch&#45;and&#45;process&#45;the&#45;linked&#45;resource"></a>[fetch and process the linked resource](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#fetch-and-process-the-linked-resource) algorithm which describes how the resource is obtained&#46;

<strong><a id="hyperlink"></a><strong>Hyperlinks</strong></strong>

These are links to other resources that are generally exposed to the user by the user agent so that the user can cause the user agent to <a id="introduction&#45;2&#58;navigate"></a>[navigate](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigate) to those resources&#44; e&#46;g&#46; to visit them in a browser or download them&#46;

<strong><a id="internal&#45;resource&#45;link"></a><strong>Internal resource links</strong></strong>

These are links to resources within the current document&#44; used to give those resources special meaning or behavior&#46;

For <a id="introduction&#45;2&#58;the&#45;link&#45;element&#45;2"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements with an <a id="introduction&#45;2&#58;attr&#45;link&#45;href"></a><code>[href](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-href)</code> attribute and a <a id="introduction&#45;2&#58;attr&#45;link&#45;rel"></a><code>[rel](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-rel)</code> attribute&#44; links must be created for the keywords of the <a id="introduction&#45;2&#58;attr&#45;link&#45;rel&#45;2"></a><code>[rel](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-rel)</code> attribute&#44; as defined for those keywords in the [link types](#linkTypes) section&#46;

Similarly&#44; for <a id="introduction&#45;2&#58;the&#45;a&#45;element&#45;2"></a><code>[a](#the-a-element)</code> and <a id="introduction&#45;2&#58;the&#45;area&#45;element&#45;2"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements with an <a id="introduction&#45;2&#58;attr&#45;hyperlink&#45;href"></a><code>[href](#attr-hyperlink-href)</code> attribute and a <a id="introduction&#45;2&#58;attr&#45;hyperlink&#45;rel"></a><code>[rel](#attr-hyperlink-rel)</code> attribute&#44; links must be created for the keywords of the <a id="introduction&#45;2&#58;attr&#45;hyperlink&#45;rel&#45;2"></a><code>[rel](#attr-hyperlink-rel)</code> attribute as defined for those keywords in the [link types](#linkTypes) section&#46; Unlike <a id="introduction&#45;2&#58;the&#45;link&#45;element&#45;3"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements&#44; however&#44; <a id="introduction&#45;2&#58;the&#45;a&#45;element&#45;3"></a><code>[a](#the-a-element)</code> and <a id="introduction&#45;2&#58;the&#45;area&#45;element&#45;3"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements with an <a id="introduction&#45;2&#58;attr&#45;hyperlink&#45;href&#45;2"></a><code>[href](#attr-hyperlink-href)</code> attribute that either do not have a <a id="introduction&#45;2&#58;attr&#45;hyperlink&#45;rel&#45;3"></a><code>[rel](#attr-hyperlink-rel)</code> attribute&#44; or whose <a id="introduction&#45;2&#58;attr&#45;hyperlink&#45;rel&#45;4"></a><code>[rel](#attr-hyperlink-rel)</code> attribute has no keywords that are defined as specifying <a id="introduction&#45;2&#58;hyperlink"></a>[hyperlinks](#hyperlink)&#44; must also create a <a id="introduction&#45;2&#58;hyperlink&#45;2"></a>[hyperlink](#hyperlink)&#46; This implied hyperlink has no special meaning &#40;it has no [link type](#linkTypes)&#41; beyond linking the element&#39;s <a id="introduction&#45;2&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document) to the resource given by the element&#39;s <a id="introduction&#45;2&#58;attr&#45;hyperlink&#45;href&#45;3"></a><code>[href](#attr-hyperlink-href)</code> attribute&#46;

Similarly&#44; for <a id="introduction&#45;2&#58;the&#45;form&#45;element&#45;2"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements with a <a id="introduction&#45;2&#58;attr&#45;form&#45;rel"></a><code>[rel](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-form-rel)</code> attribute&#44; links must be created for the keywords of the <a id="introduction&#45;2&#58;attr&#45;form&#45;rel&#45;2"></a><code>[rel](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-form-rel)</code> attribute as defined for those keywords in the [link types](#linkTypes) section&#46; <a id="introduction&#45;2&#58;the&#45;form&#45;element&#45;3"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements that do not have a <a id="introduction&#45;2&#58;attr&#45;form&#45;rel&#45;3"></a><code>[rel](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-form-rel)</code> attribute&#44; or whose <a id="introduction&#45;2&#58;attr&#45;form&#45;rel&#45;4"></a><code>[rel](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-form-rel)</code> attribute has no keywords that are defined as specifying <a id="introduction&#45;2&#58;hyperlink&#45;3"></a>[hyperlinks](#hyperlink)&#44; must also create a <a id="introduction&#45;2&#58;hyperlink&#45;4"></a>[hyperlink](#hyperlink)&#46;

A <a id="introduction&#45;2&#58;hyperlink&#45;5"></a>[hyperlink](#hyperlink) can have one or more <a id="hyperlink&#45;annotation"></a><strong>hyperlink annotations</strong> that modify the processing semantics of that hyperlink&#46;

#### <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements"></a>4&#46;6&#46;2 Links created by <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code> and <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements[](#links-created-by-a-and-area-elements)

The <a id="attr&#45;hyperlink&#45;href"></a><strong><code>href</code></strong> attribute on <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;a&#45;element&#45;2"></a><code>[a](#the-a-element)</code> and <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;area&#45;element&#45;2"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements must have a value that is a <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;valid&#45;url&#45;potentially&#45;surrounded&#45;by&#45;spaces"></a>[valid URL potentially surrounded by spaces](html-language-foundations--877d6146487f--b55e73c31978.md#valid-url-potentially-surrounded-by-spaces)&#46;

<strong>Note&#58;</strong>

The <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;attr&#45;hyperlink&#45;href"></a><code>[href](#attr-hyperlink-href)</code> attribute on <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;a&#45;element&#45;3"></a><code>[a](#the-a-element)</code> and <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;area&#45;element&#45;3"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements is not required&#59; when those elements do not have <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;attr&#45;hyperlink&#45;href&#45;2"></a><code>[href](#attr-hyperlink-href)</code> attributes they do not create hyperlinks&#46;

The <a id="attr&#45;hyperlink&#45;target"></a><strong><code>target</code></strong> attribute&#44; if present&#44; must be a <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;valid&#45;navigable&#45;target&#45;name&#45;or&#45;keyword"></a>[valid navigable target name or keyword](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#valid-navigable-target-name-or-keyword)&#46; It gives the name of the <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;navigable"></a>[navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable) that will be used&#46; User agents use this name when <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;following&#45;hyperlinks&#45;2"></a>[following hyperlinks](#following-hyperlinks-2)&#46;

The <a id="attr&#45;hyperlink&#45;download"></a><strong><code>download</code></strong> attribute&#44; if present&#44; indicates that the author intends the hyperlink to be used for <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;downloading&#45;hyperlinks"></a>[downloading a resource](#downloading-hyperlinks)&#46; The attribute may have a value&#59; the value&#44; if any&#44; specifies the default filename that the author recommends for use in labeling the resource in a local file system&#46; There are no restrictions on allowed values&#44; but authors are cautioned that most file systems have limitations with regard to what punctuation is supported in filenames&#44; and user agents are likely to adjust filenames accordingly&#46;

<strong>✔</strong>MDN

[Element&#47;a&#35;attr&#45;ping](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a#attr-ping)

Support in all current engines&#46;

Firefox🔰 1&#43;Safari6&#43;Chrome12&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;17&#43;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android≤37&#43;Samsung Internet&#63;Opera Android&#63;

The <a id="ping"></a><strong><code>ping</code></strong> attribute&#44; if present&#44; gives the URLs of the resources that are interested in being notified if the user follows the hyperlink&#46; The value must be a <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;set&#45;of&#45;space&#45;separated&#45;tokens"></a>[set of space&#45;separated tokens](html-language-foundations--877d6146487f--b55e73c31978.md#set-of-space-separated-tokens)&#44; each of which must be a <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;valid&#45;non&#45;empty&#45;url"></a>[valid non&#45;empty URL](html-language-foundations--877d6146487f--b55e73c31978.md#valid-non-empty-url) whose <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;concept&#45;url&#45;scheme"></a>[scheme](https://url.spec.whatwg.org/#concept-url-scheme) is an <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;http&#40;s&#41;&#45;scheme"></a>[HTTP&#40;S&#41; scheme](https://fetch.spec.whatwg.org/#http-scheme)&#46; The value is used by the user agent for <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;hyperlink&#45;auditing"></a>[hyperlink auditing](#hyperlink-auditing)&#46;

The <a id="attr&#45;hyperlink&#45;rel"></a><strong><code>rel</code></strong> attribute on <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;a&#45;element&#45;4"></a><code>[a](#the-a-element)</code> and <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;area&#45;element&#45;4"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements controls what kinds of links the elements create&#46; The attribute&#39;s value must be an <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;unordered&#45;set&#45;of&#45;unique&#45;space&#45;separated&#45;tokens"></a>[unordered set of unique space&#45;separated tokens](html-language-foundations--877d6146487f--b55e73c31978.md#unordered-set-of-unique-space-separated-tokens)&#46; The [allowed keywords and their meanings](#linkTypes) are defined below&#46;

<a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;attr&#45;hyperlink&#45;rel"></a><code>[rel](#attr-hyperlink-rel)</code>&#39;s <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;concept&#45;supported&#45;tokens"></a>[supported tokens](https://dom.spec.whatwg.org/#concept-supported-tokens) are the keywords defined in [HTML link types](#linkTypes) which are allowed on <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;a&#45;element&#45;5"></a><code>[a](#the-a-element)</code> and <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;area&#45;element&#45;5"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements&#44; impact the processing model&#44; and are supported by the user agent&#46; The possible <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;concept&#45;supported&#45;tokens&#45;2"></a>[supported tokens](https://dom.spec.whatwg.org/#concept-supported-tokens) are <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;link&#45;type&#45;noreferrer"></a><code>[noreferrer](#link-type-noreferrer)</code>&#44; <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;link&#45;type&#45;noopener"></a><code>[noopener](#link-type-noopener)</code>&#44; and <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;link&#45;type&#45;opener"></a><code>[opener](#link-type-opener)</code>&#46; <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;attr&#45;hyperlink&#45;rel&#45;2"></a><code>[rel](#attr-hyperlink-rel)</code>&#39;s <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;concept&#45;supported&#45;tokens&#45;3"></a>[supported tokens](https://dom.spec.whatwg.org/#concept-supported-tokens) must only include the tokens from this list that the user agent implements the processing model for&#46;

The <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;attr&#45;hyperlink&#45;rel&#45;3"></a><code>[rel](#attr-hyperlink-rel)</code> attribute has no default value&#46; If the attribute is omitted or if none of the values in the attribute are recognized by the user agent&#44; then the document has no particular relationship with the destination resource other than there being a hyperlink between the two&#46;

The <a id="attr&#45;hyperlink&#45;referrerpolicy"></a><strong><code>referrerpolicy</code></strong> attribute is a <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;referrer&#45;policy&#45;attribute"></a>[referrer policy attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#referrer-policy-attribute)&#46; Its purpose is to set the <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;referrer&#45;policy"></a>[referrer policy](https://w3c.github.io/webappsec-referrer-policy/#referrer-policy) used when <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;following&#45;hyperlinks&#45;2&#45;2"></a>[following hyperlinks](#following-hyperlinks-2)&#46; [&#91;REFERRERPOLICY&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsREFERRERPOLICY)

------------------------------------------------------------------------

When an <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;a&#45;element&#45;6"></a><code>[a](#the-a-element)</code> or <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;area&#45;element&#45;6"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> element&#39;s <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;activation&#45;behaviour"></a>[activation behavior](https://dom.spec.whatwg.org/#eventtarget-activation-behavior) is invoked&#44; the user agent may allow the user to indicate a preference regarding whether the hyperlink is to be used for <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;navigate"></a>[navigation](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigate) or whether the resource it specifies is to be downloaded&#46;

In the absence of a user preference&#44; the default should be navigation if the element has no <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;attr&#45;hyperlink&#45;download"></a><code>[download](#attr-hyperlink-download)</code> attribute&#44; and should be to download the specified resource if it does&#46;

The <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;activation&#45;behaviour&#45;2"></a>[activation behavior](https://dom.spec.whatwg.org/#eventtarget-activation-behavior) of an <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;a&#45;element&#45;7"></a><code>[a](#the-a-element)</code> or <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;area&#45;element&#45;7"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> element <var>element</var> given an event <var>event</var> is&#58;

1.  If <var>element</var> has no <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;attr&#45;hyperlink&#45;href&#45;3"></a><code>[href](#attr-hyperlink-href)</code> attribute&#44; then return&#46;

2.  Let <var>hyperlinkSuffix</var> be null&#46;

3.  If <var>element</var> is an <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;a&#45;element&#45;8"></a><code>[a](#the-a-element)</code> element&#44; and <var>event</var>&#39;s <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;concept&#45;event&#45;target"></a>[target](https://dom.spec.whatwg.org/#event-target) is an <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;img&#45;element"></a><code>[img](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-img-element)</code> with an <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;attr&#45;img&#45;ismap"></a><code>[ismap](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#attr-img-ismap)</code> attribute specified&#58;

    1.  Let <var>x</var> and <var>y</var> be 0&#46;

    2.  If <var>event</var>&#39;s <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;dom&#45;event&#45;istrusted"></a><code>[isTrusted](https://dom.spec.whatwg.org/#dom-event-istrusted)</code> attribute is initialized to true&#44; then set <var>x</var> to the distance in <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;&#39;px&#39;"></a>[CSS pixels](https://drafts.csswg.org/css-values/#px) from the left edge of the image to the location of the click&#44; and set <var>y</var> to the distance in <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;&#39;px&#39;&#45;2"></a>[CSS pixels](https://drafts.csswg.org/css-values/#px) from the top edge of the image to the location of the click&#46;

    3.  If <var>x</var> is negative&#44; set <var>x</var> to 0&#46;

    4.  If <var>y</var> is negative&#44; set <var>y</var> to 0&#46;

    5.  Set <var>hyperlinkSuffix</var> to the concatenation of U&#43;003F &#40;&#63;&#41;&#44; the value of <var>x</var> expressed as a base&#45;ten integer using <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;ascii&#45;digits"></a>[ASCII digits](https://infra.spec.whatwg.org/#ascii-digit)&#44; U&#43;002C &#40;&#44;&#41;&#44; and the value of <var>y</var> expressed as a base&#45;ten integer using <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;ascii&#45;digits&#45;2"></a>[ASCII digits](https://infra.spec.whatwg.org/#ascii-digit)&#46;

4.  Let <var>userInvolvement</var> be <var>event</var>&#39;s <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;event&#45;uni"></a>[user navigation involvement](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-uni)&#46;

5.  If the user has expressed a preference to download the hyperlink&#44; then set <var>userInvolvement</var> to &#34;<a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;uni&#45;browser&#45;ui"></a><code>[browser UI](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#uni-browser-ui)</code>&#34;&#46;

    <strong>Note&#58;</strong>

    That is&#44; if the user has expressed a specific preference for downloading&#44; this no longer counts as merely &#34;<a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;uni&#45;activation"></a><code>[activation](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#uni-activation)</code>&#34;&#46;

6.  If <var>element</var> has a <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;attr&#45;hyperlink&#45;download&#45;2"></a><code>[download](#attr-hyperlink-download)</code> attribute&#44; or if the user has expressed a preference to download the hyperlink&#44; then <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;downloading&#45;hyperlinks&#45;2"></a>[download the hyperlink](#downloading-hyperlinks) created by <var>element</var> with <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;downloading&#45;hyperlinksuffix"></a><em>[hyperlinkSuffix](#downloading-hyperlinksuffix)</em> set to <var>hyperlinkSuffix</var> and <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;downloading&#45;userinvolvement"></a><em>[userInvolvement](#downloading-userinvolvement)</em> set to <var>userInvolvement</var>&#46;

7.  Otherwise&#44; <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;following&#45;hyperlinks&#45;2&#45;3"></a>[follow the hyperlink](#following-hyperlinks-2) created by <var>element</var> with <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;following&#45;hyperlinksuffix"></a><em>[hyperlinkSuffix](#following-hyperlinksuffix)</em> set to <var>hyperlinkSuffix</var> and <a id="links&#45;created&#45;by&#45;a&#45;and&#45;area&#45;elements&#58;following&#45;userinvolvement"></a><em>[userInvolvement](#following-userinvolvement)</em> set to <var>userInvolvement</var>&#46;

#### <a id="api&#45;for&#45;hyperlink&#45;elements"></a>4&#46;6&#46;3 API for hyperlink elements[](#api-for-hyperlink-elements)

<a id="hyperlinkelementutils"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;dom&#45;hyperlink&#45;origin"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;cereactions"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;dom&#45;hyperlink&#45;protocol"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;cereactions&#45;2"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;dom&#45;hyperlink&#45;username"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;cereactions&#45;3"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;dom&#45;hyperlink&#45;password"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;cereactions&#45;4"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;dom&#45;hyperlink&#45;host"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;cereactions&#45;5"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;dom&#45;hyperlink&#45;hostname"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;cereactions&#45;6"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;dom&#45;hyperlink&#45;port"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;cereactions&#45;7"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;dom&#45;hyperlink&#45;pathname"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;cereactions&#45;8"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;dom&#45;hyperlink&#45;search"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;cereactions&#45;9"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;dom&#45;hyperlink&#45;hash"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;cereactions&#45;10"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;xattr&#45;reflect"></a><a id="dom&#45;a&#45;hreflang"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;cereactions&#45;11"></a><a id="api&#45;for&#45;hyperlink&#45;elements&#58;xattr&#45;reflect&#45;2"></a><a id="dom&#45;a&#45;type"></a>

``` text
interface mixin HyperlinkElementUtils {
  readonly attribute USVString origin;
  [CEReactions] attribute USVString protocol;
  [CEReactions] attribute USVString username;
  [CEReactions] attribute USVString password;
  [CEReactions] attribute USVString host;
  [CEReactions] attribute USVString hostname;
  [CEReactions] attribute USVString port;
  [CEReactions] attribute USVString pathname;
  [CEReactions] attribute USVString search;
  [CEReactions] attribute USVString hash;

  [CEReactions, Reflect] attribute DOMString hreflang;
  [CEReactions, Reflect] attribute DOMString type;
};
```

<strong><code><var>hyperlink</var>&#46;<a id="dom&#45;hyperlink&#45;origin&#45;dev"></a>[origin](#dom-hyperlink-origin)</code></strong>

<strong>✔</strong>MDN

[HTMLAnchorElement&#47;origin](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement/origin)

Support in all current engines&#46;

Firefox26&#43;Safari5&#46;1&#43;Chrome8&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;17&#43;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android3&#43;Samsung Internet&#63;Opera Android&#63;

[HTMLAreaElement&#47;origin](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAreaElement/origin)

Support in all current engines&#46;

Firefox26&#43;Safari10&#43;Chrome32&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;17&#43;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android37&#43;Samsung Internet&#63;Opera Android&#63;

Returns the hyperlink&#39;s URL&#39;s origin&#46;

<strong><code><var>hyperlink</var>&#46;<a id="dom&#45;hyperlink&#45;protocol&#45;dev"></a>[protocol](#dom-hyperlink-protocol)</code></strong>

<strong>✔</strong>MDN

[HTMLAnchorElement&#47;protocol](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement/protocol)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[HTMLAreaElement&#47;protocol](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAreaElement/protocol)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

Returns the hyperlink&#39;s URL&#39;s scheme&#46;

Can be set&#44; to change the URL&#39;s scheme&#46;

<strong><code><var>hyperlink</var>&#46;<a id="dom&#45;hyperlink&#45;username&#45;dev"></a>[username](#dom-hyperlink-username)</code></strong>

<strong>✔</strong>MDN

[HTMLAnchorElement&#47;username](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement/username)

Support in all current engines&#46;

Firefox26&#43;Safari10&#43;Chrome32&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[HTMLAreaElement&#47;username](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAreaElement/username)

Support in all current engines&#46;

Firefox26&#43;Safari10&#43;Chrome32&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

Returns the hyperlink&#39;s URL&#39;s username&#46;

Can be set&#44; to change the URL&#39;s username&#46;

<strong><code><var>hyperlink</var>&#46;<a id="dom&#45;hyperlink&#45;password&#45;dev"></a>[password](#dom-hyperlink-password)</code></strong>

<strong>✔</strong>MDN

[HTMLAnchorElement&#47;password](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement/password)

Support in all current engines&#46;

Firefox26&#43;Safari10&#43;Chrome32&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[HTMLAreaElement&#47;password](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAreaElement/password)

Support in all current engines&#46;

Firefox26&#43;Safari10&#43;Chrome32&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

Returns the hyperlink&#39;s URL&#39;s password&#46;

Can be set&#44; to change the URL&#39;s password&#46;

<strong><code><var>hyperlink</var>&#46;<a id="dom&#45;hyperlink&#45;host&#45;dev"></a>[host](#dom-hyperlink-host)</code></strong>

<strong>✔</strong>MDN

[HTMLAnchorElement&#47;host](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement/host)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[HTMLAreaElement&#47;host](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAreaElement/host)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

Returns the hyperlink&#39;s URL&#39;s host and port &#40;if different from the default port for the scheme&#41;&#46;

Can be set&#44; to change the URL&#39;s host and port&#46;

<strong><code><var>hyperlink</var>&#46;<a id="dom&#45;hyperlink&#45;hostname&#45;dev"></a>[hostname](#dom-hyperlink-hostname)</code></strong>

<strong>✔</strong>MDN

[HTMLAnchorElement&#47;hostname](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement/hostname)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[HTMLAreaElement&#47;hostname](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAreaElement/hostname)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

Returns the hyperlink&#39;s URL&#39;s host&#46;

Can be set&#44; to change the URL&#39;s host&#46;

<strong><code><var>hyperlink</var>&#46;<a id="dom&#45;hyperlink&#45;port&#45;dev"></a>[port](#dom-hyperlink-port)</code></strong>

<strong>✔</strong>MDN

[HTMLAnchorElement&#47;port](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement/port)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[HTMLAreaElement&#47;port](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAreaElement/port)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

Returns the hyperlink&#39;s URL&#39;s port&#46;

Can be set&#44; to change the URL&#39;s port&#46;

<strong><code><var>hyperlink</var>&#46;<a id="dom&#45;hyperlink&#45;pathname&#45;dev"></a>[pathname](#dom-hyperlink-pathname)</code></strong>

<strong>✔</strong>MDN

[HTMLAnchorElement&#47;pathname](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement/pathname)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[HTMLAreaElement&#47;pathname](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAreaElement/pathname)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

Returns the hyperlink&#39;s URL&#39;s path&#46;

Can be set&#44; to change the URL&#39;s path&#46;

<strong><code><var>hyperlink</var>&#46;<a id="dom&#45;hyperlink&#45;search&#45;dev"></a>[search](#dom-hyperlink-search)</code></strong>

<strong>✔</strong>MDN

[HTMLAnchorElement&#47;search](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement/search)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[HTMLAreaElement&#47;search](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAreaElement/search)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

Returns the hyperlink&#39;s URL&#39;s query &#40;includes leading &#34;<code>&#63;</code>&#34; if non&#45;empty&#41;&#46;

Can be set&#44; to change the URL&#39;s query &#40;ignores leading &#34;<code>&#63;</code>&#34;&#41;&#46;

<strong><code><var>hyperlink</var>&#46;<a id="dom&#45;hyperlink&#45;hash&#45;dev"></a>[hash](#dom-hyperlink-hash)</code></strong>

<strong>✔</strong>MDN

[HTMLAnchorElement&#47;hash](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement/hash)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[HTMLAreaElement&#47;hash](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAreaElement/hash)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

Returns the hyperlink&#39;s URL&#39;s fragment &#40;includes leading &#34;<code>&#35;</code>&#34; if non&#45;empty&#41;&#46;

Can be set&#44; to change the URL&#39;s fragment &#40;ignores leading &#34;<code>&#35;</code>&#34;&#41;&#46;

An element implementing the <a id="api&#45;for&#45;hyperlink&#45;elements&#58;hyperlinkelementutils"></a><code>[HyperlinkElementUtils](#hyperlinkelementutils)</code> mixin is a <a id="hyperlink&#45;element"></a><strong>hyperlink element</strong>&#46;

A <a id="api&#45;for&#45;hyperlink&#45;elements&#58;hyperlink&#45;element"></a>[hyperlink element](#hyperlink-element) has an associated <a id="concept&#45;hyperlink&#45;url"></a><strong>url</strong> &#40;null or a <a id="api&#45;for&#45;hyperlink&#45;elements&#58;url"></a>[URL](https://url.spec.whatwg.org/#concept-url)&#41;&#46; It is initially null&#46;

A <a id="api&#45;for&#45;hyperlink&#45;elements&#58;hyperlink&#45;element&#45;2"></a>[hyperlink element](#hyperlink-element) has the following <a id="api&#45;for&#45;hyperlink&#45;elements&#58;extract&#45;an&#45;origin"></a>[extract an origin](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#extract-an-origin) steps&#58;

1.  If <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url"></a>[url](#concept-hyperlink-url) is null&#44; then return null&#46;

2.  Return <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;2"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;2"></a>[url](#concept-hyperlink-url)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;origin"></a>[origin](https://url.spec.whatwg.org/#concept-url-origin)&#46;

A <a id="api&#45;for&#45;hyperlink&#45;elements&#58;hyperlink&#45;element&#45;3"></a>[hyperlink element](#hyperlink-element) must have an associated <a id="concept&#45;hyperlink&#45;url&#45;set"></a><strong>set the url</strong> algorithm&#46;

When <a id="api&#45;for&#45;hyperlink&#45;elements&#58;hyperlink&#45;element&#45;4"></a>[hyperlink elements](#hyperlink-element) are created&#44; the user agent must <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;set"></a>[set the url](#concept-hyperlink-url-set)&#46;

A <a id="api&#45;for&#45;hyperlink&#45;elements&#58;hyperlink&#45;element&#45;5"></a>[hyperlink element](#hyperlink-element) has an associated <a id="reinitialise&#45;url"></a><strong>reinitialize url</strong> algorithm&#44; which runs these steps&#58;

1.  If the element&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;3"></a>[url](#concept-hyperlink-url) is non&#45;null&#44; its <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;scheme"></a>[scheme](https://url.spec.whatwg.org/#concept-url-scheme) is &#34;<code>blob</code>&#34;&#44; and it has an <a id="api&#45;for&#45;hyperlink&#45;elements&#58;opaque&#45;path"></a>[opaque path](https://url.spec.whatwg.org/#url-opaque-path)&#44; then terminate these steps&#46;

2.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;set&#45;2"></a>[Set the url](#concept-hyperlink-url-set)&#46;

A <a id="api&#45;for&#45;hyperlink&#45;elements&#58;hyperlink&#45;element&#45;6"></a>[hyperlink element](#hyperlink-element) must have an associated <a id="update&#45;href"></a><strong>update <code>href</code></strong> algorithm&#46;

The <a id="attr&#45;hyperlink&#45;hreflang"></a><strong><code>hreflang</code></strong> attribute on <a id="api&#45;for&#45;hyperlink&#45;elements&#58;hyperlink&#45;element&#45;7"></a>[hyperlink elements](#hyperlink-element) that create <a id="api&#45;for&#45;hyperlink&#45;elements&#58;hyperlink"></a>[hyperlinks](#hyperlink)&#44; if present&#44; gives the language of the linked resource&#46; It is purely advisory&#46; The value must be a valid BCP 47 language tag&#46; [&#91;BCP47&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsBCP47) User agents must not consider this attribute authoritative — upon fetching the resource&#44; user agents must use only language information associated with the resource to determine its language&#44; not metadata included in the link to the resource&#46;

The <a id="attr&#45;hyperlink&#45;type"></a><strong><code>type</code></strong> attribute on <a id="api&#45;for&#45;hyperlink&#45;elements&#58;hyperlink&#45;element&#45;8"></a>[hyperlink elements](#hyperlink-element)&#44; if present&#44; gives the <a id="api&#45;for&#45;hyperlink&#45;elements&#58;mime&#45;type"></a>[MIME type](https://mimesniff.spec.whatwg.org/#mime-type) of the linked resource&#46; It is purely advisory&#46; The value must be a <a id="api&#45;for&#45;hyperlink&#45;elements&#58;valid&#45;mime&#45;type&#45;string"></a>[valid MIME type string](https://mimesniff.spec.whatwg.org/#valid-mime-type)&#46; User agents must not consider the <a id="api&#45;for&#45;hyperlink&#45;elements&#58;attr&#45;hyperlink&#45;type"></a><code>[type](#attr-hyperlink-type)</code> attribute authoritative — upon fetching the resource&#44; user agents must not use metadata included in the link to the resource to determine its type&#46;

------------------------------------------------------------------------

The <a id="dom&#45;hyperlink&#45;origin"></a><strong><code>origin</code></strong> getter steps are&#58;

1.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;reinitialise&#45;url"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  If <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;3"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;4"></a>[url](#concept-hyperlink-url) is null&#44; return the empty string&#46;

3.  Return the <a id="api&#45;for&#45;hyperlink&#45;elements&#58;ascii&#45;serialisation&#45;of&#45;an&#45;origin"></a>[serialization](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#ascii-serialisation-of-an-origin) of <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;4"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;5"></a>[url](#concept-hyperlink-url)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;origin&#45;2"></a>[origin](https://url.spec.whatwg.org/#concept-url-origin)&#46;

The <a id="dom&#45;hyperlink&#45;protocol"></a><strong><code>protocol</code></strong> getter steps are&#58;

1.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;reinitialise&#45;url&#45;2"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  If <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;5"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;6"></a>[url](#concept-hyperlink-url) is null&#44; return &#34;<code>&#58;</code>&#34;&#46;

3.  Return <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;6"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;7"></a>[url](#concept-hyperlink-url)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;scheme&#45;2"></a>[scheme](https://url.spec.whatwg.org/#concept-url-scheme)&#44; followed by &#34;<code>&#58;</code>&#34;&#46;

The <a id="api&#45;for&#45;hyperlink&#45;elements&#58;dom&#45;hyperlink&#45;protocol&#45;2"></a><code>[protocol](#dom-hyperlink-protocol)</code> setter steps are&#58;

1.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;reinitialise&#45;url&#45;3"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  If <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;7"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;8"></a>[url](#concept-hyperlink-url) is null&#44; then return&#46;

3.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser"></a>[Basic URL parse](https://url.spec.whatwg.org/#concept-basic-url-parser) the given value&#44; followed by &#34;<code>&#58;</code>&#34;&#44; with <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;8"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;9"></a>[url](#concept-hyperlink-url) as <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;url"></a>[<em>url</em>](https://url.spec.whatwg.org/#basic-url-parser-url) and <a id="api&#45;for&#45;hyperlink&#45;elements&#58;scheme&#45;start&#45;state"></a>[scheme start state](https://url.spec.whatwg.org/#scheme-start-state) as <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;state&#45;override"></a>[<em>state override</em>](https://url.spec.whatwg.org/#basic-url-parser-state-override)&#46;

    <strong>Note&#58;</strong>

    Because the URL parser ignores multiple consecutive colons&#44; providing a value of &#34;<code>https&#58;</code>&#34; &#40;or even &#34;<code>https&#58;&#58;&#58;&#58;</code>&#34;&#41; is the same as providing a value of &#34;<code>https</code>&#34;&#46;

4.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;update&#45;href"></a>[Update <code>href</code>](#update-href)&#46;

The <a id="dom&#45;hyperlink&#45;username"></a><strong><code>username</code></strong> getter steps are&#58;

1.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;reinitialise&#45;url&#45;4"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  If <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;9"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;10"></a>[url](#concept-hyperlink-url) is null&#44; return the empty string&#46;

3.  Return <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;10"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;11"></a>[url](#concept-hyperlink-url)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;username"></a>[username](https://url.spec.whatwg.org/#concept-url-username)&#46;

The <a id="api&#45;for&#45;hyperlink&#45;elements&#58;dom&#45;hyperlink&#45;username&#45;2"></a><code>[username](#dom-hyperlink-username)</code> setter steps are&#58;

1.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;reinitialise&#45;url&#45;5"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  Let <var>url</var> be <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;11"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;12"></a>[url](#concept-hyperlink-url)&#46;

3.  If <var>url</var> is null or <var>url</var> <a id="api&#45;for&#45;hyperlink&#45;elements&#58;cannot&#45;have&#45;a&#45;username&#47;password&#47;port"></a>[cannot have a username&#47;password&#47;port](https://url.spec.whatwg.org/#cannot-have-a-username-password-port)&#44; then return&#46;

4.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;set&#45;the&#45;username"></a>[Set the username](https://url.spec.whatwg.org/#set-the-username)&#44; given <var>url</var> and the given value&#46;

5.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;update&#45;href&#45;2"></a>[Update <code>href</code>](#update-href)&#46;

The <a id="dom&#45;hyperlink&#45;password"></a><strong><code>password</code></strong> getter steps are&#58;

1.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;reinitialise&#45;url&#45;6"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  Let <var>url</var> be <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;12"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;13"></a>[url](#concept-hyperlink-url)&#46;

3.  If <var>url</var> is null&#44; then return the empty string&#46;

4.  Return <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;password"></a>[password](https://url.spec.whatwg.org/#concept-url-password)&#46;

The <a id="api&#45;for&#45;hyperlink&#45;elements&#58;dom&#45;hyperlink&#45;password&#45;2"></a><code>[password](#dom-hyperlink-password)</code> setter steps are&#58;

1.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;reinitialise&#45;url&#45;7"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  Let <var>url</var> be <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;13"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;14"></a>[url](#concept-hyperlink-url)&#46;

3.  If <var>url</var> is null or <var>url</var> <a id="api&#45;for&#45;hyperlink&#45;elements&#58;cannot&#45;have&#45;a&#45;username&#47;password&#47;port&#45;2"></a>[cannot have a username&#47;password&#47;port](https://url.spec.whatwg.org/#cannot-have-a-username-password-port)&#44; then return&#46;

4.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;set&#45;the&#45;password"></a>[Set the password](https://url.spec.whatwg.org/#set-the-password)&#44; given <var>url</var> and the given value&#46;

5.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;update&#45;href&#45;3"></a>[Update <code>href</code>](#update-href)&#46;

The <a id="dom&#45;hyperlink&#45;host"></a><strong><code>host</code></strong> getter steps are&#58;

1.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;reinitialise&#45;url&#45;8"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  Let <var>url</var> be <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;14"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;15"></a>[url](#concept-hyperlink-url)&#46;

3.  If <var>url</var> or <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;host"></a>[host](https://url.spec.whatwg.org/#concept-url-host) is null&#44; return the empty string&#46;

4.  If <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;port"></a>[port](https://url.spec.whatwg.org/#concept-url-port) is null&#44; return <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;host&#45;2"></a>[host](https://url.spec.whatwg.org/#concept-url-host)&#44; <a id="api&#45;for&#45;hyperlink&#45;elements&#58;host&#45;serializer"></a>[serialized](https://url.spec.whatwg.org/#concept-host-serializer)&#46;

5.  Return <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;host&#45;3"></a>[host](https://url.spec.whatwg.org/#concept-url-host)&#44; <a id="api&#45;for&#45;hyperlink&#45;elements&#58;host&#45;serializer&#45;2"></a>[serialized](https://url.spec.whatwg.org/#concept-host-serializer)&#44; followed by &#34;<code>&#58;</code>&#34; and <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;port&#45;2"></a>[port](https://url.spec.whatwg.org/#concept-url-port)&#44; <a id="api&#45;for&#45;hyperlink&#45;elements&#58;serialize&#45;an&#45;integer"></a>[serialized](https://url.spec.whatwg.org/#serialize-an-integer)&#46;

The <a id="api&#45;for&#45;hyperlink&#45;elements&#58;dom&#45;hyperlink&#45;host&#45;2"></a><code>[host](#dom-hyperlink-host)</code> setter steps are&#58;

1.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;reinitialise&#45;url&#45;9"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  Let <var>url</var> be <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;15"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;16"></a>[url](#concept-hyperlink-url)&#46;

3.  If <var>url</var> is null or <var>url</var> has an <a id="api&#45;for&#45;hyperlink&#45;elements&#58;opaque&#45;path&#45;2"></a>[opaque path](https://url.spec.whatwg.org/#url-opaque-path)&#44; then return&#46;

4.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;2"></a>[Basic URL parse](https://url.spec.whatwg.org/#concept-basic-url-parser) the given value&#44; with <var>url</var> as <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;url&#45;2"></a>[<em>url</em>](https://url.spec.whatwg.org/#basic-url-parser-url) and <a id="api&#45;for&#45;hyperlink&#45;elements&#58;host&#45;state"></a>[host state](https://url.spec.whatwg.org/#host-state) as <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;state&#45;override&#45;2"></a>[<em>state override</em>](https://url.spec.whatwg.org/#basic-url-parser-state-override)&#46;

5.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;update&#45;href&#45;4"></a>[Update <code>href</code>](#update-href)&#46;

The <a id="dom&#45;hyperlink&#45;hostname"></a><strong><code>hostname</code></strong> getter steps are&#58;

1.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;reinitialise&#45;url&#45;10"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  Let <var>url</var> be <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;16"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;17"></a>[url](#concept-hyperlink-url)&#46;

3.  If <var>url</var> or <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;host&#45;4"></a>[host](https://url.spec.whatwg.org/#concept-url-host) is null&#44; return the empty string&#46;

4.  Return <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;host&#45;5"></a>[host](https://url.spec.whatwg.org/#concept-url-host)&#44; <a id="api&#45;for&#45;hyperlink&#45;elements&#58;host&#45;serializer&#45;3"></a>[serialized](https://url.spec.whatwg.org/#concept-host-serializer)&#46;

The <a id="api&#45;for&#45;hyperlink&#45;elements&#58;dom&#45;hyperlink&#45;hostname&#45;2"></a><code>[hostname](#dom-hyperlink-hostname)</code> setter steps are&#58;

1.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;reinitialise&#45;url&#45;11"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  Let <var>url</var> be <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;17"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;18"></a>[url](#concept-hyperlink-url)&#46;

3.  If <var>url</var> is null or <var>url</var> has an <a id="api&#45;for&#45;hyperlink&#45;elements&#58;opaque&#45;path&#45;3"></a>[opaque path](https://url.spec.whatwg.org/#url-opaque-path)&#44; then return&#46;

4.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;3"></a>[Basic URL parse](https://url.spec.whatwg.org/#concept-basic-url-parser) the given value&#44; with <var>url</var> as <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;url&#45;3"></a>[<em>url</em>](https://url.spec.whatwg.org/#basic-url-parser-url) and <a id="api&#45;for&#45;hyperlink&#45;elements&#58;hostname&#45;state"></a>[hostname state](https://url.spec.whatwg.org/#hostname-state) as <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;state&#45;override&#45;3"></a>[<em>state override</em>](https://url.spec.whatwg.org/#basic-url-parser-state-override)&#46;

5.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;update&#45;href&#45;5"></a>[Update <code>href</code>](#update-href)&#46;

The <a id="dom&#45;hyperlink&#45;port"></a><strong><code>port</code></strong> getter steps are&#58;

1.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;reinitialise&#45;url&#45;12"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  Let <var>url</var> be <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;18"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;19"></a>[url](#concept-hyperlink-url)&#46;

3.  If <var>url</var> or <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;port&#45;3"></a>[port](https://url.spec.whatwg.org/#concept-url-port) is null&#44; return the empty string&#46;

4.  Return <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;port&#45;4"></a>[port](https://url.spec.whatwg.org/#concept-url-port)&#44; <a id="api&#45;for&#45;hyperlink&#45;elements&#58;serialize&#45;an&#45;integer&#45;2"></a>[serialized](https://url.spec.whatwg.org/#serialize-an-integer)&#46;

The <a id="api&#45;for&#45;hyperlink&#45;elements&#58;dom&#45;hyperlink&#45;port&#45;2"></a><code>[port](#dom-hyperlink-port)</code> setter steps are&#58;

1.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;reinitialise&#45;url&#45;13"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  Let <var>url</var> be <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;19"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;20"></a>[url](#concept-hyperlink-url)&#46;

3.  If <var>url</var> is null or <var>url</var> <a id="api&#45;for&#45;hyperlink&#45;elements&#58;cannot&#45;have&#45;a&#45;username&#47;password&#47;port&#45;3"></a>[cannot have a username&#47;password&#47;port](https://url.spec.whatwg.org/#cannot-have-a-username-password-port)&#44; then return&#46;

4.  If the given value is the empty string&#44; then set <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;port&#45;5"></a>[port](https://url.spec.whatwg.org/#concept-url-port) to null&#46;

5.  Otherwise&#44; <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;4"></a>[basic URL parse](https://url.spec.whatwg.org/#concept-basic-url-parser) the given value&#44; with <var>url</var> as <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;url&#45;4"></a>[<em>url</em>](https://url.spec.whatwg.org/#basic-url-parser-url) and <a id="api&#45;for&#45;hyperlink&#45;elements&#58;port&#45;state"></a>[port state](https://url.spec.whatwg.org/#port-state) as <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;state&#45;override&#45;4"></a>[<em>state override</em>](https://url.spec.whatwg.org/#basic-url-parser-state-override)&#46;

6.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;update&#45;href&#45;6"></a>[Update <code>href</code>](#update-href)&#46;

The <a id="dom&#45;hyperlink&#45;pathname"></a><strong><code>pathname</code></strong> getter steps are&#58;

1.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;reinitialise&#45;url&#45;14"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  Let <var>url</var> be <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;20"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;21"></a>[url](#concept-hyperlink-url)&#46;

3.  If <var>url</var> is null&#44; then return the empty string&#46;

4.  Return the result of <a id="api&#45;for&#45;hyperlink&#45;elements&#58;url&#45;path&#45;serializer"></a>[URL path serializing](https://url.spec.whatwg.org/#url-path-serializer) <var>url</var>&#46;

The <a id="api&#45;for&#45;hyperlink&#45;elements&#58;dom&#45;hyperlink&#45;pathname&#45;2"></a><code>[pathname](#dom-hyperlink-pathname)</code> setter steps are&#58;

1.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;reinitialise&#45;url&#45;15"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  Let <var>url</var> be <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;21"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;22"></a>[url](#concept-hyperlink-url)&#46;

3.  If <var>url</var> is null or <var>url</var> has an <a id="api&#45;for&#45;hyperlink&#45;elements&#58;opaque&#45;path&#45;4"></a>[opaque path](https://url.spec.whatwg.org/#url-opaque-path)&#44; then return&#46;

4.  Set <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;path"></a>[path](https://url.spec.whatwg.org/#concept-url-path) to the empty list&#46;

5.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;5"></a>[Basic URL parse](https://url.spec.whatwg.org/#concept-basic-url-parser) the given value&#44; with <var>url</var> as <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;url&#45;5"></a>[<em>url</em>](https://url.spec.whatwg.org/#basic-url-parser-url) and <a id="api&#45;for&#45;hyperlink&#45;elements&#58;path&#45;start&#45;state"></a>[path start state](https://url.spec.whatwg.org/#path-start-state) as <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;state&#45;override&#45;5"></a>[<em>state override</em>](https://url.spec.whatwg.org/#basic-url-parser-state-override)&#46;

6.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;update&#45;href&#45;7"></a>[Update <code>href</code>](#update-href)&#46;

The <a id="dom&#45;hyperlink&#45;search"></a><strong><code>search</code></strong> getter steps are&#58;

1.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;reinitialise&#45;url&#45;16"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  Let <var>url</var> be <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;22"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;23"></a>[url](#concept-hyperlink-url)&#46;

3.  If <var>url</var> is null&#44; or <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;query"></a>[query](https://url.spec.whatwg.org/#concept-url-query) is either null or the empty string&#44; return the empty string&#46;

4.  Return &#34;<code>&#63;</code>&#34;&#44; followed by <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;query&#45;2"></a>[query](https://url.spec.whatwg.org/#concept-url-query)&#46;

The <a id="api&#45;for&#45;hyperlink&#45;elements&#58;dom&#45;hyperlink&#45;search&#45;2"></a><code>[search](#dom-hyperlink-search)</code> setter steps are&#58;

1.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;reinitialise&#45;url&#45;17"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  Let <var>url</var> be <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;23"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;24"></a>[url](#concept-hyperlink-url)&#46;

3.  If <var>url</var> is null&#44; terminate these steps&#46;

4.  If the given value is the empty string&#44; set <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;query&#45;3"></a>[query](https://url.spec.whatwg.org/#concept-url-query) to null&#46;

5.  Otherwise&#58;

    1.  Let <var>input</var> be the given value with a single leading &#34;<code>&#63;</code>&#34; removed&#44; if any&#46;

    2.  Set <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;query&#45;4"></a>[query](https://url.spec.whatwg.org/#concept-url-query) to the empty string&#46;

    3.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;6"></a>[Basic URL parse](https://url.spec.whatwg.org/#concept-basic-url-parser) <var>input</var>&#44; with <var>url</var> as <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;url&#45;6"></a>[<em>url</em>](https://url.spec.whatwg.org/#basic-url-parser-url) and <a id="api&#45;for&#45;hyperlink&#45;elements&#58;query&#45;state"></a>[query state](https://url.spec.whatwg.org/#query-state) as <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;state&#45;override&#45;6"></a>[<em>state override</em>](https://url.spec.whatwg.org/#basic-url-parser-state-override)&#46;

6.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;update&#45;href&#45;8"></a>[Update <code>href</code>](#update-href)&#46;

The <a id="dom&#45;hyperlink&#45;hash"></a><strong><code>hash</code></strong> getter steps are&#58;

1.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;reinitialise&#45;url&#45;18"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  Let <var>url</var> be <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;24"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;25"></a>[url](#concept-hyperlink-url)&#46;

3.  If <var>url</var> is null&#44; or <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;fragment"></a>[fragment](https://url.spec.whatwg.org/#concept-url-fragment) is either null or the empty string&#44; return the empty string&#46;

4.  Return &#34;<code>&#35;</code>&#34;&#44; followed by <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;fragment&#45;2"></a>[fragment](https://url.spec.whatwg.org/#concept-url-fragment)&#46;

The <a id="api&#45;for&#45;hyperlink&#45;elements&#58;dom&#45;hyperlink&#45;hash&#45;2"></a><code>[hash](#dom-hyperlink-hash)</code> setter steps are&#58;

1.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;reinitialise&#45;url&#45;19"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  Let <var>url</var> be <a id="api&#45;for&#45;hyperlink&#45;elements&#58;this&#45;25"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;26"></a>[url](#concept-hyperlink-url)&#46;

3.  If <var>url</var> is null&#44; then return&#46;

4.  If the given value is the empty string&#44; set <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;fragment&#45;3"></a>[fragment](https://url.spec.whatwg.org/#concept-url-fragment) to null&#46;

5.  Otherwise&#58;

    1.  Let <var>input</var> be the given value with a single leading &#34;<code>&#35;</code>&#34; removed&#44; if any&#46;

    2.  Set <var>url</var>&#39;s <a id="api&#45;for&#45;hyperlink&#45;elements&#58;concept&#45;url&#45;fragment&#45;4"></a>[fragment](https://url.spec.whatwg.org/#concept-url-fragment) to the empty string&#46;

    3.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;7"></a>[Basic URL parse](https://url.spec.whatwg.org/#concept-basic-url-parser) <var>input</var>&#44; with <var>url</var> as <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;url&#45;7"></a>[<em>url</em>](https://url.spec.whatwg.org/#basic-url-parser-url) and <a id="api&#45;for&#45;hyperlink&#45;elements&#58;fragment&#45;state"></a>[fragment state](https://url.spec.whatwg.org/#fragment-state) as <a id="api&#45;for&#45;hyperlink&#45;elements&#58;basic&#45;url&#45;parser&#45;state&#45;override&#45;7"></a>[<em>state override</em>](https://url.spec.whatwg.org/#basic-url-parser-state-override)&#46;

6.  <a id="api&#45;for&#45;hyperlink&#45;elements&#58;update&#45;href&#45;9"></a>[Update <code>href</code>](#update-href)&#46;

#### <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements"></a>4&#46;6&#46;4 API for <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code> and <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements[](#api-for-a-and-area-elements)

<a id="htmlhyperlinkelementutils"></a><a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;cereactions"></a><a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;xattr&#45;reflectsetter"></a><a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;dom&#45;hyperlink&#45;href"></a><a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;cereactions&#45;2"></a><a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;xattr&#45;reflect"></a><a id="dom&#45;a&#45;target"></a>

``` text
interface mixin HTMLHyperlinkElementUtils {
  [CEReactions, ReflectSetter] stringifier attribute USVString href;
  [CEReactions, Reflect] attribute DOMString target;
};
```

<strong><code><var>hyperlink</var>&#46;toString&#40;&#41;</code></strong>

<strong><code><var>hyperlink</var>&#46;<a id="dom&#45;hyperlink&#45;href&#45;dev"></a>[href](#dom-hyperlink-href)</code></strong>

<strong>✔</strong>MDN

[HTMLAnchorElement&#47;href](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement/href)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

[HTMLAnchorElement&#47;toString](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement/toString)

Support in all current engines&#46;

Firefox22&#43;Safari3&#43;Chrome52&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[HTMLAreaElement&#47;href](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAreaElement/href)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

[HTMLAreaElement&#47;toString](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAreaElement/toString)

Support in all current engines&#46;

Firefox22&#43;Safari10&#46;1&#43;Chrome32&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

Returns the hyperlink&#39;s URL&#46;

Can be set&#44; to change the URL&#46;

The <a id="dom&#45;hyperlink&#45;href"></a><strong><code>href</code></strong> getter steps are&#58;

1.  <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;reinitialise&#45;url"></a>[Reinitialize url](#reinitialise-url)&#46;

2.  Let <var>url</var> be <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;this"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;concept&#45;hyperlink&#45;url"></a>[url](#concept-hyperlink-url)&#46;

3.  If <var>url</var> is null and <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;this&#45;2"></a>[this](https://webidl.spec.whatwg.org/#this) has no <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;attr&#45;hyperlink&#45;href"></a><code>[href](#attr-hyperlink-href)</code> content attribute&#44; return the empty string&#46;

4.  Otherwise&#44; if <var>url</var> is null&#44; return <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;this&#45;3"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;attr&#45;hyperlink&#45;href&#45;2"></a><code>[href](#attr-hyperlink-href)</code> content attribute&#39;s value&#46;

5.  Return <var>url</var>&#44; <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;concept&#45;url&#45;serializer"></a>[serialized](https://url.spec.whatwg.org/#concept-url-serializer)&#46;

The <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;html&#45;element&#45;insertion&#45;steps"></a>[HTML element insertion steps](html-language-foundations--877d6146487f--b55e73c31978.md#html-element-insertion-steps) for <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;a&#45;element&#45;2"></a><code>[a](#the-a-element)</code> and <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;area&#45;element&#45;2"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements&#44; given <var>insertedNode</var>&#44; are&#58;

1.  If <var>insertedNode</var> is not <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;connected"></a>[connected](https://dom.spec.whatwg.org/#connected)&#44; then return&#46;

2.  <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;consider&#45;speculative&#45;loads"></a>[Consider speculative loads](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#consider-speculative-loads) given <var>insertedNode</var>&#39;s <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46;

The <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;html&#45;element&#45;removing&#45;steps"></a>[HTML element removing steps](html-language-foundations--877d6146487f--b55e73c31978.md#html-element-removing-steps) for <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;a&#45;element&#45;3"></a><code>[a](#the-a-element)</code> and <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;area&#45;element&#45;3"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements&#44; given <var>removedNode</var>&#44; <var>isSubtreeRoot</var>&#44; and <var>oldAncestor</var> are&#58;

1.  If <var>oldAncestor</var> is not <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;connected&#45;2"></a>[connected](https://dom.spec.whatwg.org/#connected)&#44; then return&#46;

2.  <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;consider&#45;speculative&#45;loads&#45;2"></a>[Consider speculative loads](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#consider-speculative-loads) given <var>oldAncestor</var>&#39;s <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;node&#45;document&#45;2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46;

The <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;html&#45;element&#45;moving&#45;steps"></a>[HTML element moving steps](html-language-foundations--877d6146487f--b55e73c31978.md#html-element-moving-steps) for <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;a&#45;element&#45;4"></a><code>[a](#the-a-element)</code> and <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;area&#45;element&#45;4"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements&#44; given <var>movedNode</var>&#44; are&#58;

1.  <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;consider&#45;speculative&#45;loads&#45;3"></a>[Consider speculative loads](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#consider-speculative-loads) given <var>movedNode</var>&#39;s <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;node&#45;document&#45;3"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46;

The following <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;concept&#45;element&#45;attributes&#45;change&#45;ext"></a>[attribute change steps](https://dom.spec.whatwg.org/#concept-element-attributes-change-ext)&#44; given <var>element</var>&#44; <var>localName</var>&#44; <var>oldValue</var>&#44; <var>value</var>&#44; and <var>namespace</var>&#44; are used for all <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;a&#45;element&#45;5"></a><code>[a](#the-a-element)</code> and <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;the&#45;area&#45;element&#45;5"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements&#58;

1.  If <var>namespace</var> is not null&#44; then return&#46;

2.  If <var>oldValue</var> equals <var>value</var>&#44; then return&#46;

3.  If <var>localName</var> is <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;attr&#45;hyperlink&#45;href&#45;3"></a><code>[href](#attr-hyperlink-href)</code>&#44; then <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;set"></a>[set the url](#concept-hyperlink-url-set) given <var>element</var>&#46;

    <strong>Note&#58;</strong>

    This is only observable for <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;blob&#45;protocol"></a><code>[blob&#58;](https://w3c.github.io/FileAPI/#blob-url)</code> URLs as <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;url&#45;parser"></a>[parsing](https://url.spec.whatwg.org/#concept-url-parser) them involves a <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;blob&#45;url&#45;store"></a>[Blob URL Store](https://w3c.github.io/FileAPI/#BlobURLStore) lookup&#46;

4.  If <var>localName</var> is <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;attr&#45;hyperlink&#45;href&#45;4"></a><code>[href](#attr-hyperlink-href)</code>&#44; <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;attr&#45;hyperlink&#45;referrerpolicy"></a><code>[referrerpolicy](#attr-hyperlink-referrerpolicy)</code>&#44; or <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;attr&#45;hyperlink&#45;rel"></a><code>[rel](#attr-hyperlink-rel)</code>&#44; then <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;consider&#45;speculative&#45;loads&#45;4"></a>[consider speculative loads](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#consider-speculative-loads) given <var>element</var>&#39;s <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;node&#45;document&#45;4"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46;

To <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;update&#45;href"></a>[update <code>href</code>](#update-href) for an <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;htmlanchorelement"></a><code>[HTMLAnchorElement](#htmlanchorelement)</code> or <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;htmlareaelement"></a><code>[HTMLAreaElement](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#htmlareaelement)</code> element&#44; set the element&#39;s <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;attr&#45;hyperlink&#45;href&#45;5"></a><code>[href](#attr-hyperlink-href)</code> content attribute&#39;s value to the element&#39;s <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;2"></a>[url](#concept-hyperlink-url)&#44; <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;concept&#45;url&#45;serializer&#45;2"></a>[serialized](https://url.spec.whatwg.org/#concept-url-serializer)&#46;

The <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;set&#45;2"></a>[set the url](#concept-hyperlink-url-set) algorithm for <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;htmlanchorelement&#45;2"></a><code>[HTMLAnchorElement](#htmlanchorelement)</code> and <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;htmlareaelement&#45;2"></a><code>[HTMLAreaElement](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#htmlareaelement)</code> elements is as follows&#58;

1.  Set this element&#39;s <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;3"></a>[url](#concept-hyperlink-url) to null&#46;

2.  If this element&#39;s <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;attr&#45;hyperlink&#45;href&#45;6"></a><code>[href](#attr-hyperlink-href)</code> content attribute is absent&#44; then return&#46;

3.  Let <var>url</var> be the result of <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;encoding&#45;parsing&#45;a&#45;url"></a>[encoding&#45;parsing a URL](html-language-foundations--877d6146487f--b55e73c31978.md#encoding-parsing-a-url) given this element&#39;s <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;attr&#45;hyperlink&#45;href&#45;7"></a><code>[href](#attr-hyperlink-href)</code> content attribute&#39;s value&#44; relative to this element&#39;s <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;node&#45;document&#45;5"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46;

4.  If <var>url</var> is not failure&#44; then set this element&#39;s <a id="api&#45;for&#45;a&#45;and&#45;area&#45;elements&#58;concept&#45;hyperlink&#45;url&#45;4"></a>[url](#concept-hyperlink-url) to <var>url</var>&#46;

#### <a id="following&#45;hyperlinks"></a>4&#46;6&#46;5 Following hyperlinks[](#following-hyperlinks)

An element <var>element</var> <a id="cannot&#45;navigate"></a><strong>cannot navigate</strong> if any of the following are true&#58;

- <var>element</var>&#39;s <a id="following&#45;hyperlinks&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document) is not <a id="following&#45;hyperlinks&#58;fully&#45;active"></a>[fully active](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fully-active)&#59; or

- <var>element</var> is not an <a id="following&#45;hyperlinks&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code> element and is not <a id="following&#45;hyperlinks&#58;connected"></a>[connected](https://dom.spec.whatwg.org/#connected)&#46;

<strong>Note&#58;</strong>

This is also used by <a id="following&#45;hyperlinks&#58;concept&#45;form&#45;submit"></a>[form submission](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-form-submit) for the <a id="following&#45;hyperlinks&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> element&#46; The exception for <a id="following&#45;hyperlinks&#58;the&#45;a&#45;element&#45;2"></a><code>[a](#the-a-element)</code> elements is for compatibility with web content&#46;

To <a id="get&#45;an&#45;element&#39;s&#45;noopener"></a><strong>get an element&#39;s noopener</strong>&#44; given an <a id="following&#45;hyperlinks&#58;the&#45;a&#45;element&#45;3"></a><code>[a](#the-a-element)</code>&#44; <a id="following&#45;hyperlinks&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code>&#44; or <a id="following&#45;hyperlinks&#58;the&#45;form&#45;element&#45;2"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> element <var>element</var>&#44; a <a id="following&#45;hyperlinks&#58;url&#45;record"></a>[URL record](https://url.spec.whatwg.org/#concept-url) <var>url</var>&#44; and a string <var>target</var>&#44; perform the following steps&#46; They return a boolean&#46;

1.  If <var>element</var>&#39;s [link types](#linkTypes) include the <a id="following&#45;hyperlinks&#58;link&#45;type&#45;noopener"></a><code>[noopener](#link-type-noopener)</code> or <a id="following&#45;hyperlinks&#58;link&#45;type&#45;noreferrer"></a><code>[noreferrer](#link-type-noreferrer)</code> keyword&#44; then return true&#46;

2.  <a id="opener&#45;processing&#45;model"></a>

    If <var>element</var>&#39;s [link types](#linkTypes) do not include the <a id="following&#45;hyperlinks&#58;link&#45;type&#45;opener"></a><code>[opener](#link-type-opener)</code> keyword and <var>target</var> is an <a id="following&#45;hyperlinks&#58;ascii&#45;case&#45;insensitive"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for &#34;<code>&#95;blank</code>&#34;&#44; then return true&#46;

3.  If <var>url</var>&#39;s <a id="following&#45;hyperlinks&#58;concept&#45;url&#45;blob&#45;entry"></a>[blob URL entry](https://url.spec.whatwg.org/#concept-url-blob-entry) is not null&#58;

    1.  Let <var>blobOrigin</var> be <var>url</var>&#39;s <a id="following&#45;hyperlinks&#58;concept&#45;url&#45;blob&#45;entry&#45;2"></a>[blob URL entry](https://url.spec.whatwg.org/#concept-url-blob-entry)&#39;s <a id="following&#45;hyperlinks&#58;blob&#45;url&#45;entry&#45;environment"></a>[environment](https://w3c.github.io/FileAPI/#blob-url-entry-environment)&#39;s <a id="following&#45;hyperlinks&#58;concept&#45;settings&#45;object&#45;origin"></a>[origin](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-settings-object-origin)&#46;

    2.  Let <var>topLevelOrigin</var> be <var>element</var>&#39;s <a id="following&#45;hyperlinks&#58;relevant&#45;settings&#45;object"></a>[relevant settings object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant-settings-object)&#39;s <a id="following&#45;hyperlinks&#58;concept&#45;environment&#45;top&#45;level&#45;origin"></a>[top&#45;level origin](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-top-level-origin)&#46;

    3.  If <var>blobOrigin</var> is not <a id="following&#45;hyperlinks&#58;same&#45;site"></a>[same site](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#same-site) with <var>topLevelOrigin</var>&#44; then return true&#46;

4.  Return false&#46;

To <a id="following&#45;hyperlinks&#45;2"></a><strong>follow the hyperlink</strong> created by an element <var>subject</var>&#44; given an optional <a id="following&#45;hyperlinksuffix"></a><strong><var>hyperlinkSuffix</var></strong> &#40;default null&#41; and an optional <a id="following&#45;userinvolvement"></a><strong><var>userInvolvement</var></strong> &#40;default &#34;<a id="following&#45;hyperlinks&#58;uni&#45;none"></a><code>[none](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#uni-none)</code>&#34;&#41;&#58;

1.  If <var>subject</var> <a id="following&#45;hyperlinks&#58;cannot&#45;navigate"></a>[cannot navigate](#cannot-navigate)&#44; then return&#46;

2.  Let <var>targetAttributeValue</var> be the empty string&#46;

3.  If <var>subject</var> is an <a id="following&#45;hyperlinks&#58;the&#45;a&#45;element&#45;4"></a><code>[a](#the-a-element)</code> or <a id="following&#45;hyperlinks&#58;the&#45;area&#45;element&#45;2"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> element&#44; then set <var>targetAttributeValue</var> to the result of <a id="following&#45;hyperlinks&#58;get&#45;an&#45;element&#39;s&#45;target"></a>[getting an element&#39;s target](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#get-an-element's-target) given <var>subject</var>&#46;

4.  Let <var>urlRecord</var> be the result of <a id="following&#45;hyperlinks&#58;encoding&#45;parsing&#45;a&#45;url"></a>[encoding&#45;parsing a URL](html-language-foundations--877d6146487f--b55e73c31978.md#encoding-parsing-a-url) given <var>subject</var>&#39;s <a id="following&#45;hyperlinks&#58;attr&#45;hyperlink&#45;href"></a><code>[href](#attr-hyperlink-href)</code> attribute value&#44; relative to <var>subject</var>&#39;s <a id="following&#45;hyperlinks&#58;node&#45;document&#45;2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46;

5.  If <var>urlRecord</var> is failure&#44; then return&#46;

6.  Let <var>noopener</var> be the result of <a id="following&#45;hyperlinks&#58;get&#45;an&#45;element&#39;s&#45;noopener"></a>[getting an element&#39;s noopener](#get-an-element's-noopener) with <var>subject</var>&#44; <var>urlRecord</var>&#44; and <var>targetAttributeValue</var>&#46;

7.  Let <var>targetNavigable</var> be the first return value of applying <a id="following&#45;hyperlinks&#58;the&#45;rules&#45;for&#45;choosing&#45;a&#45;navigable"></a>[the rules for choosing a navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-rules-for-choosing-a-navigable) given <var>targetAttributeValue</var>&#44; <var>subject</var>&#39;s <a id="following&#45;hyperlinks&#58;node&#45;navigable"></a>[node navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable)&#44; and <var>noopener</var>&#46;

8.  If <var>targetNavigable</var> is null&#44; then return&#46;

9.  Let <var>urlString</var> be the result of applying the <a id="following&#45;hyperlinks&#58;concept&#45;url&#45;serializer"></a>[URL serializer](https://url.spec.whatwg.org/#concept-url-serializer) to <var>urlRecord</var>&#46;

10. If <var>hyperlinkSuffix</var> is non&#45;null&#44; then append it to <var>urlString</var>&#46;

11. <a id="following&#45;hyperlinks&#58;navigate"></a>[Navigate](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigate) <var>targetNavigable</var> to <var>urlString</var> using <var>subject</var>&#39;s <a id="following&#45;hyperlinks&#58;node&#45;document&#45;3"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#44; with <a id="following&#45;hyperlinks&#58;navigation&#45;referrer&#45;policy"></a><em>[referrerPolicy](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-referrer-policy)</em> set to <var>subject</var>&#39;s <a id="following&#45;hyperlinks&#58;hyperlink&#45;referrer&#45;policy"></a>[hyperlink referrer policy](#hyperlink-referrer-policy)&#44; <a id="following&#45;hyperlinks&#58;navigation&#45;user&#45;involvement"></a><em>[userInvolvement](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-user-involvement)</em> set to <var>userInvolvement</var>&#44; and <a id="following&#45;hyperlinks&#58;navigation&#45;source&#45;element"></a><em>[sourceElement](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-source-element)</em> set to <var>subject</var>&#46;

    <strong>Note&#58;</strong>

    Unlike many other types of navigations&#44; following hyperlinks does not have special &#34;<a id="following&#45;hyperlinks&#58;navigationhistorybehavior&#45;replace"></a><code>[replace](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigationhistorybehavior-replace)</code>&#34; behavior for when documents are not <a id="following&#45;hyperlinks&#58;completely&#45;loaded"></a>[completely loaded](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#completely-loaded)&#46; This is true for both user&#45;initiated instances of following hyperlinks&#44; as well as script&#45;triggered ones via&#44; e&#46;g&#46;&#44; <code>aElement&#46;click&#40;&#41;</code>&#46;

The <a id="hyperlink&#45;referrer&#45;policy"></a><strong>hyperlink referrer policy</strong> for an element <var>subject</var> is the value returned by the following steps&#58;

1.  <a id="noreferrer&#45;a&#45;area&#45;processing&#45;model"></a>

    If <var>subject</var>&#39;s [link types](#linkTypes) includes the <a id="following&#45;hyperlinks&#58;link&#45;type&#45;noreferrer&#45;2"></a><code>[noreferrer](#link-type-noreferrer)</code> keyword&#44; then return &#34;<code>no&#45;referrer</code>&#34;&#46;

2.  Return the current state of <var>subject</var>&#39;s <a id="following&#45;hyperlinks&#58;attr&#45;hyperlink&#45;referrerpolicy"></a><code>[referrerpolicy](#attr-hyperlink-referrerpolicy)</code> content attribute&#46;

#### <a id="downloading&#45;resources"></a>4&#46;6&#46;6 Downloading resources[](#downloading-resources)

<strong>✔</strong>MDN

[HTMLAnchorElement&#47;download](https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement/download)

Support in all current engines&#46;

Firefox20&#43;Safari10&#46;1&#43;Chrome15&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;13&#43;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

In some cases&#44; resources are intended for later use rather than immediate viewing&#46; To indicate that a resource is intended to be downloaded for use later&#44; rather than immediately used&#44; the <a id="downloading&#45;resources&#58;attr&#45;hyperlink&#45;download"></a><code>[download](#attr-hyperlink-download)</code> attribute can be specified on the <a id="downloading&#45;resources&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code> or <a id="downloading&#45;resources&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> element that creates the <a id="downloading&#45;resources&#58;hyperlink"></a>[hyperlink](#hyperlink) to that resource&#46;

The attribute can furthermore be given a value&#44; to specify the filename that user agents are to use when storing the resource in a file system&#46; This value can be overridden by the &#96;<a id="downloading&#45;resources&#58;http&#45;content&#45;disposition"></a><code>[Content&#45;Disposition](https://httpwg.org/specs/rfc6266.html)</code>&#96; HTTP header&#39;s filename parameters&#46; [&#91;RFC6266&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsRFC6266)

In cross&#45;origin situations&#44; the <a id="downloading&#45;resources&#58;attr&#45;hyperlink&#45;download&#45;2"></a><code>[download](#attr-hyperlink-download)</code> attribute has to be combined with the &#96;<a id="downloading&#45;resources&#58;http&#45;content&#45;disposition&#45;2"></a><code>[Content&#45;Disposition](https://httpwg.org/specs/rfc6266.html)</code>&#96; HTTP header&#44; specifically with the <code>attachment</code> disposition type&#44; to avoid the user being warned of possibly nefarious activity&#46; &#40;This is to protect users from being made to download sensitive personal or confidential information without their full understanding&#46;&#41;

------------------------------------------------------------------------

To <a id="downloading&#45;hyperlinks"></a><strong>download the hyperlink</strong> created by an element <var>subject</var>&#44; given an optional <a id="downloading&#45;hyperlinksuffix"></a><strong><var>hyperlinkSuffix</var></strong> &#40;default null&#41; and an optional <a id="downloading&#45;userinvolvement"></a><strong><var>userInvolvement</var></strong> &#40;default &#34;<a id="downloading&#45;resources&#58;uni&#45;none"></a><code>[none](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#uni-none)</code>&#34;&#41;&#58;

1.  If <var>subject</var> <a id="downloading&#45;resources&#58;cannot&#45;navigate"></a>[cannot navigate](#cannot-navigate)&#44; then return&#46;

2.  <a id="allowed&#45;to&#45;download"></a>

    If <var>subject</var>&#39;s <a id="downloading&#45;resources&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="downloading&#45;resources&#58;active&#45;sandboxing&#45;flag&#45;set"></a>[active sandboxing flag set](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#active-sandboxing-flag-set) has the <a id="downloading&#45;resources&#58;sandboxed&#45;downloads&#45;browsing&#45;context&#45;flag"></a>[sandboxed downloads browsing context flag](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-downloads-browsing-context-flag) set&#44; then return&#46;

3.  Let <var>urlString</var> be the result of <a id="downloading&#45;resources&#58;encoding&#45;parsing&#45;and&#45;serializing&#45;a&#45;url"></a>[encoding&#45;parsing&#45;and&#45;serializing a URL](html-language-foundations--877d6146487f--b55e73c31978.md#encoding-parsing-and-serializing-a-url) given <var>subject</var>&#39;s <a id="downloading&#45;resources&#58;attr&#45;hyperlink&#45;href"></a><code>[href](#attr-hyperlink-href)</code> attribute value&#44; relative to <var>subject</var>&#39;s <a id="downloading&#45;resources&#58;node&#45;document&#45;2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46;

4.  If <var>urlString</var> is failure&#44; then return&#46;

5.  If <var>hyperlinkSuffix</var> is non&#45;null&#44; then append it to <var>urlString</var>&#46;

6.  If <var>userInvolvement</var> is not &#34;<a id="downloading&#45;resources&#58;uni&#45;browser&#45;ui"></a><code>[browser UI](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#uni-browser-ui)</code>&#34;&#58;

    1.  <a id="downloading&#45;resources&#58;assert"></a>[Assert](https://infra.spec.whatwg.org/#assert)&#58; <var>subject</var> has a <a id="downloading&#45;resources&#58;attr&#45;hyperlink&#45;download&#45;3"></a><code>[download](#attr-hyperlink-download)</code> attribute&#46;

    2.  Let <var>navigation</var> be <var>subject</var>&#39;s <a id="downloading&#45;resources&#58;concept&#45;relevant&#45;global"></a>[relevant global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global)&#39;s <a id="downloading&#45;resources&#58;window&#45;navigation&#45;api"></a>[navigation API](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window-navigation-api)&#46;

    3.  Let <var>filename</var> be the value of <var>subject</var>&#39;s <a id="downloading&#45;resources&#58;attr&#45;hyperlink&#45;download&#45;4"></a><code>[download](#attr-hyperlink-download)</code> attribute&#46;

    4.  Let <var>continue</var> be the result of <a id="downloading&#45;resources&#58;fire&#45;a&#45;download&#45;request&#45;navigate&#45;event"></a>[firing a download request <code>navigate</code> event](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fire-a-download-request-navigate-event) at <var>navigation</var> with <a id="downloading&#45;resources&#58;fire&#45;navigate&#45;download&#45;destinationurl"></a><em>[destinationURL](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fire-navigate-download-destinationurl)</em> set to <var>urlString</var>&#44; <a id="downloading&#45;resources&#58;fire&#45;navigate&#45;download&#45;userinvolvement"></a><em>[userInvolvement](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fire-navigate-download-userinvolvement)</em> set to <var>userInvolvement</var>&#44; <a id="downloading&#45;resources&#58;fire&#45;navigate&#45;download&#45;sourceelement"></a><em>[sourceElement](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fire-navigate-download-sourceelement)</em> set to <var>subject</var>&#44; and <a id="downloading&#45;resources&#58;fire&#45;navigate&#45;download&#45;filename"></a><em>[filename](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fire-navigate-download-filename)</em> set to <var>filename</var>&#46;

    5.  If <var>continue</var> is false&#44; then return&#46;

    6.  <a id="downloading&#45;resources&#58;inform&#45;the&#45;navigation&#45;api&#45;about&#45;aborting&#45;navigation"></a>[Inform the navigation API about aborting navigation](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#inform-the-navigation-api-about-aborting-navigation) given <var>subject</var>&#39;s <a id="downloading&#45;resources&#58;node&#45;navigable"></a>[node navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable)&#46;

7.  Optionally&#44; the user agent may return&#44; if it believes doing so would safeguard the user from a potentially hostile download&#46;

8.  Let <var>request</var> be a new <a id="downloading&#45;resources&#58;concept&#45;request"></a>[request](https://fetch.spec.whatwg.org/#concept-request) whose <a id="downloading&#45;resources&#58;concept&#45;request&#45;url"></a>[URL](https://fetch.spec.whatwg.org/#concept-request-url) is <var>urlString</var>&#44; <a id="downloading&#45;resources&#58;concept&#45;request&#45;client"></a>[client](https://fetch.spec.whatwg.org/#concept-request-client) is <a id="downloading&#45;resources&#58;entry&#45;settings&#45;object"></a>[entry settings object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#entry-settings-object)&#44; <a id="downloading&#45;resources&#58;concept&#45;request&#45;initiator"></a>[initiator](https://fetch.spec.whatwg.org/#concept-request-initiator) is &#34;<code>download</code>&#34;&#44; <a id="downloading&#45;resources&#58;concept&#45;request&#45;destination"></a>[destination](https://fetch.spec.whatwg.org/#concept-request-destination) is the empty string&#44; and whose <a id="downloading&#45;resources&#58;use&#45;url&#45;credentials&#45;flag"></a>[use&#45;URL&#45;credentials flag](https://fetch.spec.whatwg.org/#concept-request-use-url-credentials-flag) is set&#46;

9.  <a id="downloading&#45;resources&#58;concept&#45;fetch"></a>[Fetch](https://fetch.spec.whatwg.org/#concept-fetch) <var>request</var>&#44; with <a id="downloading&#45;resources&#58;processresponse"></a><em>[processResponse](https://fetch.spec.whatwg.org/#process-response)</em> set to the following steps given <a id="downloading&#45;resources&#58;concept&#45;response"></a>[response](https://fetch.spec.whatwg.org/#concept-response) <var>response</var>&#58;

    1.  <a id="downloading&#45;resources&#58;handle&#45;as&#45;a&#45;download"></a>[Handle as a download](#handle-as-a-download) <var>response</var> with <var>subject</var>&#39;s <a id="downloading&#45;resources&#58;node&#45;navigable&#45;2"></a>[node navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable) and null&#46;

To <a id="handle&#45;as&#45;a&#45;download"></a><strong>handle as a download</strong> a <a id="downloading&#45;resources&#58;concept&#45;response&#45;2"></a>[response](https://fetch.spec.whatwg.org/#concept-response) <var>response</var> with a <a id="downloading&#45;resources&#58;navigable"></a>[navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable) <var>navigable</var> and a <a id="downloading&#45;resources&#58;navigation&#45;id"></a>[navigation ID](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-id) or null <var>navigationId</var>&#58;

1.  Let <var>suggestedFilename</var> be the result of <a id="downloading&#45;resources&#58;getting&#45;the&#45;suggested&#45;filename"></a>[getting the suggested filename](#getting-the-suggested-filename) for <var>response</var>&#46;

2.  Let <var>download behavior</var> be the result of <a id="downloading&#45;resources&#58;webdriver&#45;bidi&#45;download&#45;will&#45;begin"></a>[WebDriver BiDi download will begin](https://w3c.github.io/webdriver-bidi/#webdriver-bidi-download-will-begin) with <var>navigable</var> and a new <a id="downloading&#45;resources&#58;webdriver&#45;bidi&#45;navigation&#45;status"></a>[WebDriver BiDi navigation status](https://w3c.github.io/webdriver-bidi/#webdriver-bidi-navigation-status) whose <a id="downloading&#45;resources&#58;navigation&#45;status&#45;id"></a>[id](https://w3c.github.io/webdriver-bidi/#navigation-status-id) is <var>navigationId</var>&#44; <a id="downloading&#45;resources&#58;navigation&#45;status&#45;status"></a>[status](https://w3c.github.io/webdriver-bidi/#navigation-status-status) is &#34;<a id="downloading&#45;resources&#58;navigation&#45;status&#45;pending"></a><code>[pending](https://w3c.github.io/webdriver-bidi/#navigation-status-pending)</code>&#34;&#44; <a id="downloading&#45;resources&#58;navigation&#45;status&#45;url"></a>[url](https://w3c.github.io/webdriver-bidi/#navigation-status-url) is <var>response</var>&#39;s <a id="downloading&#45;resources&#58;concept&#45;response&#45;url"></a>[URL](https://fetch.spec.whatwg.org/#concept-response-url)&#44; <a id="downloading&#45;resources&#58;navigation&#45;status&#45;suggested&#45;filename"></a>[suggestedFilename](https://w3c.github.io/webdriver-bidi/#navigation-status-suggested-filename) is <var>suggestedFilename</var>&#44; and <a id="downloading&#45;resources&#58;concept&#45;response&#45;3"></a>[downloadResponse](https://fetch.spec.whatwg.org/#concept-response) is <var>response</var>&#46;

3.  If <var>download behavior</var> is not null and <var>download behavior</var>&#39;s <a id="downloading&#45;resources&#58;download&#45;behavior&#45;struct&#45;allowed"></a>[allowed](https://w3c.github.io/webdriver-bidi/#download-behavior-struct-allowed) is false&#58;

    1.  Invoke <a id="downloading&#45;resources&#58;webdriver&#45;bidi&#45;download&#45;end"></a>[WebDriver BiDi download end](https://w3c.github.io/webdriver-bidi/#webdriver-bidi-download-end) with <var>navigable</var> and a new <a id="downloading&#45;resources&#58;webdriver&#45;bidi&#45;navigation&#45;status&#45;2"></a>[WebDriver BiDi navigation status](https://w3c.github.io/webdriver-bidi/#webdriver-bidi-navigation-status) whose <a id="downloading&#45;resources&#58;navigation&#45;status&#45;id&#45;2"></a>[id](https://w3c.github.io/webdriver-bidi/#navigation-status-id) is <var>navigationId</var>&#44; <a id="downloading&#45;resources&#58;navigation&#45;status&#45;status&#45;2"></a>[status](https://w3c.github.io/webdriver-bidi/#navigation-status-status) is &#34;<a id="downloading&#45;resources&#58;navigation&#45;status&#45;canceled"></a><code>[canceled](https://w3c.github.io/webdriver-bidi/#navigation-status-canceled)</code>&#34;&#44; <a id="downloading&#45;resources&#58;navigation&#45;status&#45;url&#45;2"></a>[url](https://w3c.github.io/webdriver-bidi/#navigation-status-url) is <var>response</var>&#39;s <a id="downloading&#45;resources&#58;concept&#45;response&#45;url&#45;2"></a>[URL](https://fetch.spec.whatwg.org/#concept-response-url)&#44; and <a id="downloading&#45;resources&#58;concept&#45;response&#45;4"></a>[downloadResponse](https://fetch.spec.whatwg.org/#concept-response) is <var>response</var>&#46;

    2.  Return&#46;

4.  If <var>download behavior</var> is not null&#44; let <var>destinationFolder</var> be <var>download behavior</var>&#39;s <a id="downloading&#45;resources&#58;download&#45;behavior&#45;struct&#45;destination&#45;folder"></a>[destinationFolder](https://w3c.github.io/webdriver-bidi/#download-behavior-struct-destination-folder)&#46;

5.  Run these steps <a id="downloading&#45;resources&#58;in&#45;parallel"></a>[in parallel](html-language-foundations--877d6146487f--b55e73c31978.md#in-parallel)&#58;

    1.  Run <a id="downloading&#45;resources&#58;implementation&#45;defined"></a>[implementation&#45;defined](https://infra.spec.whatwg.org/#implementation-defined) steps to save <var>response</var> for later use&#46; If <var>destinationFolder</var> is not null&#44; the user agent should save the file to that path&#46; If the user agent needs a filename&#44; the user agent should use the <var>suggestedFilename</var>&#46;

    2.  If any of the following are true&#58;

        - the download is canceled by the user&#59;

        - the download is canceled by the user agent&#59;

        - an error occurs &#40;for example&#44; a network error&#44; not enough storage&#44; an unavailable destination folder&#41;&#59;

        then&#58;

        1.  Invoke <a id="downloading&#45;resources&#58;webdriver&#45;bidi&#45;download&#45;end&#45;2"></a>[WebDriver BiDi download end](https://w3c.github.io/webdriver-bidi/#webdriver-bidi-download-end) with <var>navigable</var> and a new <a id="downloading&#45;resources&#58;webdriver&#45;bidi&#45;navigation&#45;status&#45;3"></a>[WebDriver BiDi navigation status](https://w3c.github.io/webdriver-bidi/#webdriver-bidi-navigation-status) whose <a id="downloading&#45;resources&#58;navigation&#45;status&#45;id&#45;3"></a>[id](https://w3c.github.io/webdriver-bidi/#navigation-status-id) is <var>navigationId</var>&#44; <a id="downloading&#45;resources&#58;navigation&#45;status&#45;status&#45;3"></a>[status](https://w3c.github.io/webdriver-bidi/#navigation-status-status) is &#34;<a id="downloading&#45;resources&#58;navigation&#45;status&#45;canceled&#45;2"></a><code>[canceled](https://w3c.github.io/webdriver-bidi/#navigation-status-canceled)</code>&#34;&#44; <a id="downloading&#45;resources&#58;navigation&#45;status&#45;url&#45;3"></a>[url](https://w3c.github.io/webdriver-bidi/#navigation-status-url) is <var>response</var>&#39;s <a id="downloading&#45;resources&#58;concept&#45;response&#45;url&#45;3"></a>[URL](https://fetch.spec.whatwg.org/#concept-response-url)&#46;

        2.  Return&#46;

    3.  When the download completes successfully&#44; invoke <a id="downloading&#45;resources&#58;webdriver&#45;bidi&#45;download&#45;end&#45;3"></a>[WebDriver BiDi download end](https://w3c.github.io/webdriver-bidi/#webdriver-bidi-download-end) with <var>navigable</var> and a new <a id="downloading&#45;resources&#58;webdriver&#45;bidi&#45;navigation&#45;status&#45;4"></a>[WebDriver BiDi navigation status](https://w3c.github.io/webdriver-bidi/#webdriver-bidi-navigation-status) whose <a id="downloading&#45;resources&#58;navigation&#45;status&#45;id&#45;4"></a>[id](https://w3c.github.io/webdriver-bidi/#navigation-status-id) is <var>navigationId</var>&#44; <a id="downloading&#45;resources&#58;navigation&#45;status&#45;status&#45;4"></a>[status](https://w3c.github.io/webdriver-bidi/#navigation-status-status) is &#34;<a id="downloading&#45;resources&#58;navigation&#45;status&#45;complete"></a><code>[complete](https://w3c.github.io/webdriver-bidi/#navigation-status-complete)</code>&#34;&#44; <a id="downloading&#45;resources&#58;navigation&#45;status&#45;downloaded&#45;filepath"></a>[downloadedFilepath](https://w3c.github.io/webdriver-bidi/#navigation-status-downloaded-filepath) is an absolute path of the downloaded file if available&#44; otherwise null&#44; <a id="downloading&#45;resources&#58;navigation&#45;status&#45;url&#45;4"></a>[url](https://w3c.github.io/webdriver-bidi/#navigation-status-url) is <var>response</var>&#39;s <a id="downloading&#45;resources&#58;concept&#45;response&#45;url&#45;4"></a>[URL](https://fetch.spec.whatwg.org/#concept-response-url)&#44; and <a id="downloading&#45;resources&#58;concept&#45;response&#45;5"></a>[downloadResponse](https://fetch.spec.whatwg.org/#concept-response) is <var>response</var>&#46;

To <a id="getting&#45;the&#45;suggested&#45;filename"></a><strong>get the suggested filename</strong> for a <a id="downloading&#45;resources&#58;concept&#45;response&#45;6"></a>[response](https://fetch.spec.whatwg.org/#concept-response) <var>response</var>&#58;

<strong>Warning&#58;</strong>

This algorithm is intended to mitigate security dangers involved in downloading files from untrusted sites&#44; and user agents are strongly urged to follow it&#46;

1.  Let <var>filename</var> be the undefined value&#46;

2.  If <var>response</var> has a &#96;<a id="downloading&#45;resources&#58;http&#45;content&#45;disposition&#45;3"></a><code>[Content&#45;Disposition](https://httpwg.org/specs/rfc6266.html)</code>&#96; header&#44; that header specifies the <code>attachment</code> disposition type&#44; and the header includes filename information&#44; then let <var>filename</var> have the value specified by the header&#44; and jump to the step labeled <em>sanitize</em> below&#46; [&#91;RFC6266&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsRFC6266)

3.  Let <var>interface origin</var> be the <a id="downloading&#45;resources&#58;concept&#45;document&#45;origin"></a>[origin](https://dom.spec.whatwg.org/#concept-document-origin) of the <a id="downloading&#45;resources&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> in which the <a id="downloading&#45;resources&#58;downloading&#45;hyperlinks"></a>[download](#downloading-hyperlinks) or <a id="downloading&#45;resources&#58;navigate"></a>[navigate](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigate) action resulting in the download was initiated&#44; if any&#46;

4.  Let <var>response origin</var> be the <a id="downloading&#45;resources&#58;concept&#45;origin"></a>[origin](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin) of the URL of <var>response</var>&#44; unless that URL&#39;s <a id="downloading&#45;resources&#58;concept&#45;url&#45;scheme"></a>[scheme](https://url.spec.whatwg.org/#concept-url-scheme) component is <code>data</code>&#44; in which case let <var>response origin</var> be the same as the <var>interface origin</var>&#44; if any&#46;

5.  If there is no <var>interface origin</var>&#44; then let <var>trusted operation</var> be true&#46; Otherwise&#44; let <var>trusted operation</var> be true if <var>response origin</var> is the <a id="downloading&#45;resources&#58;same&#45;origin"></a>[same origin](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#same-origin) as <var>interface origin</var>&#44; and false otherwise&#46;

6.  If <var>trusted operation</var> is true and <var>response</var> has a &#96;<a id="downloading&#45;resources&#58;http&#45;content&#45;disposition&#45;4"></a><code>[Content&#45;Disposition](https://httpwg.org/specs/rfc6266.html)</code>&#96; header and that header includes filename information&#44; then let <var>filename</var> have the value specified by the header&#44; and jump to the step labeled <em>sanitize</em> below&#46; [&#91;RFC6266&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsRFC6266)

7.  If the download was not initiated from a <a id="downloading&#45;resources&#58;hyperlink&#45;2"></a>[hyperlink](#hyperlink) created by an <a id="downloading&#45;resources&#58;the&#45;a&#45;element&#45;2"></a><code>[a](#the-a-element)</code> or <a id="downloading&#45;resources&#58;the&#45;area&#45;element&#45;2"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> element&#44; or if the element of the <a id="downloading&#45;resources&#58;hyperlink&#45;3"></a>[hyperlink](#hyperlink) from which it was initiated did not have a <a id="downloading&#45;resources&#58;attr&#45;hyperlink&#45;download&#45;5"></a><code>[download](#attr-hyperlink-download)</code> attribute when the download was initiated&#44; or if there was such an attribute but its value when the download was initiated was the empty string&#44; then jump to the step labeled <em>no proposed filename</em>&#46;

8.  Let <var>proposed filename</var> have the value of the <a id="downloading&#45;resources&#58;attr&#45;hyperlink&#45;download&#45;6"></a><code>[download](#attr-hyperlink-download)</code> attribute of the element of the <a id="downloading&#45;resources&#58;hyperlink&#45;4"></a>[hyperlink](#hyperlink) that initiated the download at the time the download was initiated&#46;

9.  If <var>trusted operation</var> is true&#44; let <var>filename</var> have the value of <var>proposed filename</var>&#44; and jump to the step labeled <em>sanitize</em> below&#46;

10. If <var>response</var> has a &#96;<a id="downloading&#45;resources&#58;http&#45;content&#45;disposition&#45;5"></a><code>[Content&#45;Disposition](https://httpwg.org/specs/rfc6266.html)</code>&#96; header and that header specifies the <code>attachment</code> disposition type&#44; let <var>filename</var> have the value of <var>proposed filename</var>&#44; and jump to the step labeled <em>sanitize</em> below&#46; [&#91;RFC6266&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsRFC6266)

11. <em>No proposed filename</em>&#58; If <var>trusted operation</var> is true&#44; or if the user indicated a preference for having the response in question downloaded&#44; let <var>filename</var> have a value derived from the <a id="downloading&#45;resources&#58;url"></a>[URL](https://url.spec.whatwg.org/#concept-url) of <var>response</var> in an <a id="downloading&#45;resources&#58;implementation&#45;defined&#45;2"></a>[implementation&#45;defined](https://infra.spec.whatwg.org/#implementation-defined) manner&#44; and jump to the step labeled <em>sanitize</em> below&#46;

12. Let <var>filename</var> be set to the user&#39;s preferred filename or to a filename selected by the user agent&#44; and jump to the step labeled <em>sanitize</em> below&#46;

    <strong>Warning&#58;</strong>

    If the algorithm reaches this step&#44; then a download was begun from a different origin than <var>response</var>&#44; and the origin did not mark the file as suitable for downloading&#44; and the download was not initiated by the user&#46; This could be because a <a id="downloading&#45;resources&#58;attr&#45;hyperlink&#45;download&#45;7"></a><code>[download](#attr-hyperlink-download)</code> attribute was used to trigger the download&#44; or because <var>response</var> is not of a type that the user agent supports&#46;

    This could be dangerous&#44; because&#44; for instance&#44; a hostile server could be trying to get a user to unknowingly download private information and then re&#45;upload it to the hostile server&#44; by tricking the user into thinking the data is from the hostile server&#46;

    Thus&#44; it is in the user&#39;s interests that the user be somehow notified that <var>response</var> comes from quite a different source&#44; and to prevent confusion&#44; any suggested filename from the potentially hostile <var>interface origin</var> should be ignored&#46;

13. <em>Sanitize</em>&#58; Optionally&#44; allow the user to influence <var>filename</var>&#46; For example&#44; a user agent could prompt the user for a filename&#44; potentially providing the value of <var>filename</var> as determined above as a default value&#46;

14. Adjust <var>filename</var> to be suitable for the local file system&#46;

    <strong>Example&#58;</strong>

    For example&#44; this could involve removing characters that are not legal in filenames&#44; or trimming leading and trailing whitespace&#46;

15. If the platform conventions do not in any way use <a id="downloading&#45;resources&#58;concept&#45;extension"></a>[extensions](#concept-extension) to determine the types of file on the file system&#44; then return <var>filename</var> as the filename&#46;

16. Let <var>claimed type</var> be the type given by <var>response</var>&#39;s <a id="downloading&#45;resources&#58;content&#45;type"></a>[Content&#45;Type metadata](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-type)&#44; if any is known&#46; Let <var>named type</var> be the type given by <var>filename</var>&#39;s <a id="downloading&#45;resources&#58;concept&#45;extension&#45;2"></a>[extension](#concept-extension)&#44; if any is known&#46; For the purposes of this step&#44; a <em>type</em> is a mapping of a <a id="downloading&#45;resources&#58;mime&#45;type"></a>[MIME type](https://mimesniff.spec.whatwg.org/#mime-type) to an <a id="downloading&#45;resources&#58;concept&#45;extension&#45;3"></a>[extension](#concept-extension)&#46;

17. If <var>named type</var> is consistent with the user&#39;s preferences &#40;e&#46;g&#46;&#44; because the value of <var>filename</var> was determined by prompting the user&#41;&#44; then return <var>filename</var> as the filename&#46;

18. If <var>claimed type</var> and <var>named type</var> are the same type &#40;i&#46;e&#46;&#44; the type given by <var>response</var>&#39;s <a id="downloading&#45;resources&#58;content&#45;type&#45;2"></a>[Content&#45;Type metadata](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-type) is consistent with the type given by <var>filename</var>&#39;s <a id="downloading&#45;resources&#58;concept&#45;extension&#45;4"></a>[extension](#concept-extension)&#41;&#44; then return <var>filename</var> as the filename&#46;

19. If the <var>claimed type</var> is known&#44; then alter <var>filename</var> to add an <a id="downloading&#45;resources&#58;concept&#45;extension&#45;5"></a>[extension](#concept-extension) corresponding to <var>claimed type</var>&#46;

    Otherwise&#44; if <var>named type</var> is known to be potentially dangerous &#40;e&#46;g&#46; it will be treated by the platform conventions as a native executable&#44; shell script&#44; HTML application&#44; or executable&#45;macro&#45;capable document&#41;&#44; then optionally alter <var>filename</var> to add a known&#45;safe <a id="downloading&#45;resources&#58;concept&#45;extension&#45;6"></a>[extension](#concept-extension) &#40;e&#46;g&#46; &#34;<code>&#46;txt</code>&#34;&#41;&#46;

    <strong>Note&#58;</strong>

    This last step would make it impossible to download executables&#44; which might not be desirable&#46; As always&#44; implementers are forced to balance security and usability in this matter&#46;

20. Return <var>filename</var> as the filename&#46;

For the purposes of this algorithm&#44; a file <a id="concept&#45;extension"></a><strong>extension</strong> consists of any part of the filename that platform conventions dictate will be used for identifying the type of the file&#46; For example&#44; many operating systems use the part of the filename following the last dot &#40;&#34;<code>&#46;</code>&#34;&#41; in the filename to determine the type of the file&#44; and from that the manner in which the file is to be opened or executed&#46;

User agents should ignore any directory or path information provided by the response itself&#44; its <a id="downloading&#45;resources&#58;url&#45;2"></a>[URL](https://url.spec.whatwg.org/#concept-url)&#44; and any <a id="downloading&#45;resources&#58;attr&#45;hyperlink&#45;download&#45;8"></a><code>[download](#attr-hyperlink-download)</code> attribute&#44; in deciding where to store the resulting file in the user&#39;s file system&#46;

#### <a id="hyperlink&#45;auditing"></a>4&#46;6&#46;7 <strong>Hyperlink auditing</strong>[](#hyperlink-auditing)

If a <a id="hyperlink&#45;auditing&#58;hyperlink"></a>[hyperlink](#hyperlink) created by an <a id="hyperlink&#45;auditing&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code> or <a id="hyperlink&#45;auditing&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> element has a <a id="hyperlink&#45;auditing&#58;ping"></a><code>[ping](#ping)</code> attribute&#44; and the user follows the hyperlink&#44; and the value of the element&#39;s <a id="hyperlink&#45;auditing&#58;attr&#45;hyperlink&#45;href"></a><code>[href](#attr-hyperlink-href)</code> attribute can be <a id="hyperlink&#45;auditing&#58;encoding&#45;parsing&#45;a&#45;url"></a>[parsed](html-language-foundations--877d6146487f--b55e73c31978.md#encoding-parsing-a-url)&#44; relative to the element&#39;s <a id="hyperlink&#45;auditing&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#44; without failure&#44; then the user agent must take the <a id="hyperlink&#45;auditing&#58;ping&#45;2"></a><code>[ping](#ping)</code> attribute&#39;s value&#44; <a id="hyperlink&#45;auditing&#58;split&#45;a&#45;string&#45;on&#45;spaces"></a>[split that string on ASCII whitespace](https://infra.spec.whatwg.org/#split-on-ascii-whitespace)&#44; <a id="hyperlink&#45;auditing&#58;encoding&#45;parsing&#45;a&#45;url&#45;2"></a>[parse](html-language-foundations--877d6146487f--b55e73c31978.md#encoding-parsing-a-url) each resulting token&#44; relative to the element&#39;s <a id="hyperlink&#45;auditing&#58;node&#45;document&#45;2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#44; and then run these steps for each resulting <a id="hyperlink&#45;auditing&#58;url"></a>[URL](https://url.spec.whatwg.org/#concept-url) <var>ping URL</var>&#44; ignoring when parsing returns failure&#58;

1.  If <var>ping URL</var>&#39;s <a id="hyperlink&#45;auditing&#58;concept&#45;url&#45;scheme"></a>[scheme](https://url.spec.whatwg.org/#concept-url-scheme) is not an <a id="hyperlink&#45;auditing&#58;http&#40;s&#41;&#45;scheme"></a>[HTTP&#40;S&#41; scheme](https://fetch.spec.whatwg.org/#http-scheme)&#44; then return&#46;

2.  Optionally&#44; return&#46; &#40;For example&#44; the user agent might wish to ignore any or all ping URLs in accordance with the user&#39;s expressed preferences&#46;&#41;

3.  Let <var>settingsObject</var> be the element&#39;s <a id="hyperlink&#45;auditing&#58;node&#45;document&#45;3"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="hyperlink&#45;auditing&#58;relevant&#45;settings&#45;object"></a>[relevant settings object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant-settings-object)&#46;

4.  Let <var>request</var> be a new <a id="hyperlink&#45;auditing&#58;concept&#45;request"></a>[request](https://fetch.spec.whatwg.org/#concept-request) whose <a id="hyperlink&#45;auditing&#58;concept&#45;request&#45;url"></a>[URL](https://fetch.spec.whatwg.org/#concept-request-url) is <var>ping URL</var>&#44; <a id="hyperlink&#45;auditing&#58;concept&#45;request&#45;method"></a>[method](https://fetch.spec.whatwg.org/#concept-request-method) is &#96;<code>POST</code>&#96;&#44; <a id="hyperlink&#45;auditing&#58;concept&#45;request&#45;header&#45;list"></a>[header list](https://fetch.spec.whatwg.org/#concept-request-header-list) is « &#40;&#96;<a id="hyperlink&#45;auditing&#58;content&#45;type"></a><code>[Content&#45;Type](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-type)</code>&#96;&#44; &#96;<a id="hyperlink&#45;auditing&#58;text&#47;ping"></a><code>[text&#47;ping](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#text/ping)</code>&#96;&#41; »&#44; <a id="hyperlink&#45;auditing&#58;concept&#45;request&#45;body"></a>[body](https://fetch.spec.whatwg.org/#concept-request-body) is &#96;<code>PING</code>&#96;&#44; <a id="hyperlink&#45;auditing&#58;concept&#45;request&#45;client"></a>[client](https://fetch.spec.whatwg.org/#concept-request-client) is <var>settingsObject</var>&#44; <a id="hyperlink&#45;auditing&#58;concept&#45;request&#45;destination"></a>[destination](https://fetch.spec.whatwg.org/#concept-request-destination) is the empty string&#44; <a id="hyperlink&#45;auditing&#58;concept&#45;request&#45;credentials&#45;mode"></a>[credentials mode](https://fetch.spec.whatwg.org/#concept-request-credentials-mode) is &#34;<code>include</code>&#34;&#44; <a id="hyperlink&#45;auditing&#58;concept&#45;request&#45;referrer"></a>[referrer](https://fetch.spec.whatwg.org/#concept-request-referrer) is &#34;<code>no&#45;referrer</code>&#34;&#44; and whose <a id="hyperlink&#45;auditing&#58;use&#45;url&#45;credentials&#45;flag"></a>[use&#45;URL&#45;credentials flag](https://fetch.spec.whatwg.org/#concept-request-use-url-credentials-flag) is set&#44; and whose <a id="hyperlink&#45;auditing&#58;concept&#45;request&#45;initiator&#45;type"></a>[initiator type](https://fetch.spec.whatwg.org/#request-initiator-type) is &#34;<code>ping</code>&#34;&#46;

5.  Let <var>target URL</var> be the result of <a id="hyperlink&#45;auditing&#58;encoding&#45;parsing&#45;and&#45;serializing&#45;a&#45;url"></a>[encoding&#45;parsing&#45;and&#45;serializing a URL](html-language-foundations--877d6146487f--b55e73c31978.md#encoding-parsing-and-serializing-a-url) given the element&#39;s <a id="hyperlink&#45;auditing&#58;attr&#45;hyperlink&#45;href&#45;2"></a><code>[href](#attr-hyperlink-href)</code> attribute&#39;s value&#44; relative to the element&#39;s <a id="hyperlink&#45;auditing&#58;node&#45;document&#45;4"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#44; and then&#58;

    <strong>If the <a id="hyperlink&#45;auditing&#58;the&#45;document&#39;s&#45;address"></a>[URL](https://dom.spec.whatwg.org/#concept-document-url) of the <a id="hyperlink&#45;auditing&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> object containing the hyperlink being audited and <var>ping URL</var> have the <a id="hyperlink&#45;auditing&#58;same&#45;origin"></a>[same origin](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#same-origin)</strong>

    <strong>If the origins are different&#44; but the <a id="hyperlink&#45;auditing&#58;concept&#45;url&#45;scheme&#45;2"></a>[scheme](https://url.spec.whatwg.org/#concept-url-scheme) of the <a id="hyperlink&#45;auditing&#58;the&#45;document&#39;s&#45;address&#45;2"></a>[URL](https://dom.spec.whatwg.org/#concept-document-url) of the <a id="hyperlink&#45;auditing&#58;document&#45;2"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> containing the hyperlink being audited is not &#34;<code>https</code>&#34;</strong>

    <var>request</var> must include a &#96;<a id="hyperlink&#45;auditing&#58;ping&#45;from"></a><code>[Ping&#45;From](#ping-from)</code>&#96; header with&#44; as its value&#44; the <a id="hyperlink&#45;auditing&#58;the&#45;document&#39;s&#45;address&#45;3"></a>[URL](https://dom.spec.whatwg.org/#concept-document-url) of the document containing the hyperlink&#44; and a &#96;<a id="hyperlink&#45;auditing&#58;ping&#45;to"></a><code>[Ping&#45;To](#ping-to)</code>&#96; HTTP header with&#44; as its value&#44; the <var>target URL</var>&#46;
    <strong>Otherwise</strong>

    <var>request</var> must include a &#96;<a id="hyperlink&#45;auditing&#58;ping&#45;to&#45;2"></a><code>[Ping&#45;To](#ping-to)</code>&#96; HTTP header with&#44; as its value&#44; <var>target URL</var>&#46;
    <strong>Note&#58;</strong>

    <var>request</var> does not include a &#96;<a id="hyperlink&#45;auditing&#58;ping&#45;from&#45;2"></a><code>[Ping&#45;From](#ping-from)</code>&#96; header&#46;

6.  <a id="hyperlink&#45;auditing&#58;concept&#45;fetch"></a>[Fetch](https://fetch.spec.whatwg.org/#concept-fetch) <var>request</var>&#46;

This may be done <a id="hyperlink&#45;auditing&#58;in&#45;parallel"></a>[in parallel](html-language-foundations--877d6146487f--b55e73c31978.md#in-parallel) with the primary fetch&#44; and is independent of the result of that fetch&#46;

User agents should allow the user to adjust this behavior&#44; for example in conjunction with a setting that disables the sending of HTTP &#96;<a id="hyperlink&#45;auditing&#58;http&#45;referer"></a><code>[Referer](https://httpwg.org/specs/rfc7231.html#header.referer)</code>&#96; &#40;sic&#41; headers&#46; Based on the user&#39;s preferences&#44; UAs may either <a id="hyperlink&#45;auditing&#58;ignore"></a>[ignore](html-language-foundations--877d6146487f--b55e73c31978.md#ignore) the <a id="hyperlink&#45;auditing&#58;ping&#45;3"></a><code>[ping](#ping)</code> attribute altogether&#44; or selectively ignore URLs in the list &#40;e&#46;g&#46; ignoring any third&#45;party URLs&#41;&#59; this is explicitly accounted for in the steps above&#46;

User agents must ignore any entity bodies returned in the responses&#46; User agents may close the connection prematurely once they start receiving a response body&#46;

<a id="fingerprint&#45;ping"></a> <a id="hyperlink&#45;auditing&#58;tracking&#45;vector"></a>[![&#40;This is a tracking vector&#46;&#41;](https://resources.whatwg.org/tracking-vector.svg)](https://infra.spec.whatwg.org/#tracking-vector) An <a id="hyperlink&#45;auditing&#58;the&#45;a&#45;element&#45;2"></a><code>[a](#the-a-element)</code> or <a id="hyperlink&#45;auditing&#58;the&#45;area&#45;element&#45;2"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> element that creates a <a id="hyperlink&#45;auditing&#58;hyperlink&#45;2"></a>[hyperlink](#hyperlink) and has the <a id="hyperlink&#45;auditing&#58;ping&#45;4"></a><code>[ping](#ping)</code> attribute is present&#44; user agents may indicate to the user that following the hyperlink will also cause secondary requests to be sent in the background&#44; possibly including listing the actual target URLs&#46;

<strong>Example&#58;</strong>

For example&#44; a visual user agent could include the hostnames of the target ping URLs along with the hyperlink&#39;s actual URL in a status bar or tooltip&#46;

<strong>Note&#58;</strong>

The <a id="hyperlink&#45;auditing&#58;ping&#45;5"></a><code>[ping](#ping)</code> attribute is redundant with pre&#45;existing technologies like HTTP redirects and JavaScript in allowing web pages to track which off&#45;site links are most popular or allowing advertisers to track click&#45;through rates&#46;

However&#44; the <a id="hyperlink&#45;auditing&#58;ping&#45;6"></a><code>[ping](#ping)</code> attribute provides these advantages to the user over those alternatives&#58;

- It allows the user to see the final target URL unobscured&#46;
- It allows the UA to inform the user about the out&#45;of&#45;band notifications&#46;
- It allows the UA to optimize the use of available network bandwidth so that the target page loads faster&#46;

##### <a id="the&#45;ping&#45;headers"></a>4&#46;6&#46;7&#46;1 The &#96;<a id="the&#45;ping&#45;headers&#58;ping&#45;from"></a><code>[Ping&#45;From](#ping-from)</code>&#96; and &#96;<a id="the&#45;ping&#45;headers&#58;ping&#45;to"></a><code>[Ping&#45;To](#ping-to)</code>&#96; headers[](#the-ping-headers)

The &#96;<a id="ping&#45;from"></a><strong><code>Ping&#45;From</code></strong>&#96; and &#96;<a id="ping&#45;to"></a><strong><code>Ping&#45;To</code></strong>&#96; HTTP request headers are included in <a id="the&#45;ping&#45;headers&#58;hyperlink&#45;auditing"></a>[hyperlink auditing](#hyperlink-auditing) requests&#46; Their value is a <a id="the&#45;ping&#45;headers&#58;url"></a>[URL](https://url.spec.whatwg.org/#concept-url)&#44; <a id="the&#45;ping&#45;headers&#58;concept&#45;url&#45;serializer"></a>[serialized](https://url.spec.whatwg.org/#concept-url-serializer)&#46;

#### <a id="linkTypes"></a>4&#46;6&#46;8 Link types[](#linkTypes)

<strong>✔</strong>MDN

[Link&#95;types](https://developer.mozilla.org/en-US/docs/Web/HTML/Link_types)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera9&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android10&#46;1&#43;

[Link&#95;types](https://developer.mozilla.org/en-US/docs/Web/HTML/Link_types)

The following table summarizes the link types that are defined by this specification&#44; by their corresponding keywords&#46; This table is non&#45;normative&#59; the actual definitions for the link types are given in the next few sections&#46;

In this section&#44; the term <em>referenced document</em> refers to the resource identified by the element representing the link&#44; and the term <em>current document</em> refers to the resource within which the element representing the link finds itself&#46;

To determine which link types apply to a <a id="linkTypes&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code>&#44; <a id="linkTypes&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code>&#44; <a id="linkTypes&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code>&#44; or <a id="linkTypes&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> element&#44; the element&#39;s <code>rel</code> attribute must be <a id="linkTypes&#58;split&#45;a&#45;string&#45;on&#45;spaces"></a>[split on ASCII whitespace](https://infra.spec.whatwg.org/#split-on-ascii-whitespace)&#46; The resulting tokens are the keywords for the link types that apply to that element&#46;

Except where otherwise specified&#44; a keyword must not be specified more than once per <a id="linkTypes&#58;attr&#45;hyperlink&#45;rel"></a><code>[rel](#attr-hyperlink-rel)</code> attribute&#46;

Some of the sections that follow the table below list synonyms for certain keywords&#46; The indicated synonyms are to be handled as specified by user agents&#44; but must not be used in documents &#40;for example&#44; the keyword &#34;<code>copyright</code>&#34;&#41;&#46;

Keywords are always <a id="linkTypes&#58;ascii&#45;case&#45;insensitive"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive)&#44; and must be compared as such&#46;

<strong>Example&#58;</strong>

Thus&#44; <code>rel&#61;&#34;next&#34;</code> is the same as <code>rel&#61;&#34;NEXT&#34;</code>&#46;

Keywords that are <a id="body&#45;ok"></a><strong>body&#45;ok</strong> affect whether <a id="linkTypes&#58;the&#45;link&#45;element&#45;2"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements are <a id="linkTypes&#58;allowed&#45;in&#45;the&#45;body"></a>[allowed in the body](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#allowed-in-the-body)&#46; The <a id="linkTypes&#58;body&#45;ok"></a>[body&#45;ok](#body-ok) keywords are <a id="linkTypes&#58;link&#45;type&#45;dns&#45;prefetch"></a><code>[dns&#45;prefetch](#link-type-dns-prefetch)</code>&#44; <a id="linkTypes&#58;link&#45;type&#45;modulepreload"></a><code>[modulepreload](#link-type-modulepreload)</code>&#44; <a id="linkTypes&#58;link&#45;type&#45;pingback"></a><code>[pingback](#link-type-pingback)</code>&#44; <a id="linkTypes&#58;link&#45;type&#45;preconnect"></a><code>[preconnect](#link-type-preconnect)</code>&#44; <a id="linkTypes&#58;link&#45;type&#45;prefetch"></a><code>[prefetch](#link-type-prefetch)</code>&#44; <a id="linkTypes&#58;link&#45;type&#45;preload"></a><code>[preload](#link-type-preload)</code>&#44; and <a id="linkTypes&#58;link&#45;type&#45;stylesheet"></a><code>[stylesheet](#link-type-stylesheet)</code>&#46;

New link types that are to be implemented by web browsers are to be added to this standard&#46; The remainder can be <a id="linkTypes&#58;concept&#45;rel&#45;extensions"></a>[registered as extensions](#concept-rel-extensions)&#46;

<table id="table-link-relations" class="yesno"><thead><tr><th rowspan="2">Link type</th><th colspan="3">Effect on&#46;&#46;&#46;</th><th rowspan="2"><a href="#body-ok" id="linkTypes:body-ok-2">body&#45;ok</a></th><th rowspan="2">Has &#96;<code id="linkTypes:http-link"><a data-x-internal="http-link" href="https://httpwg.org/specs/rfc8288.html#header">Link</a></code>&#96; processing</th><th rowspan="2">Brief description</th></tr><tr><th><code id="linkTypes:the-link-element-3"><a href="html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element">link</a></code></th><th><code id="linkTypes:the-a-element-2"><a href="#the-a-element">a</a></code> and <code id="linkTypes:the-area-element-2"><a href="html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element">area</a></code></th><th><code id="linkTypes:the-form-element-2"><a href="html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element">form</a></code></th></tr></thead><tbody><tr><td><code id="linkTypes:rel-alternate"><a href="#rel-alternate">alternate</a></code></td><td colspan="2"><a href="#hyperlink" id="linkTypes:hyperlink">Hyperlink</a></td><td><em>not allowed</em></td><td class="no"> · </td><td class="no"> · </td><td>Gives alternate representations of the current document&#46;</td></tr><tr><td><code id="linkTypes:link-type-canonical"><a href="#link-type-canonical">canonical</a></code></td><td><a href="#hyperlink" id="linkTypes:hyperlink-2">Hyperlink</a></td><td colspan="2"><em>not allowed</em></td><td class="no"> · </td><td class="no"> · </td><td>Gives the preferred URL for the current document&#46;</td></tr><tr><td><code id="linkTypes:link-type-author"><a href="#link-type-author">author</a></code></td><td colspan="2"><a href="#hyperlink" id="linkTypes:hyperlink-3">Hyperlink</a></td><td><em>not allowed</em></td><td class="no"> · </td><td class="no"> · </td><td>Gives a link to the author of the current document or article&#46;</td></tr><tr><td><code id="linkTypes:link-type-bookmark"><a href="#link-type-bookmark">bookmark</a></code></td><td><em>not allowed</em></td><td><a href="#hyperlink" id="linkTypes:hyperlink-4">Hyperlink</a></td><td><em>not allowed</em></td><td class="no"> · </td><td class="no"> · </td><td>Gives the permalink for the nearest ancestor section&#46;</td></tr><tr><td><code id="linkTypes:link-type-dns-prefetch-2"><a href="#link-type-dns-prefetch">dns&#45;prefetch</a></code></td><td><a href="#external-resource-link" id="linkTypes:external-resource-link">External Resource</a></td><td colspan="2"><em>not allowed</em></td><td class="yes"> Yes </td><td class="no"> · </td><td>Specifies that the user agent should preemptively perform DNS resolution for the target resource&#39;s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin" id="linkTypes:concept-origin">origin</a>&#46;</td></tr><tr><td><code id="linkTypes:link-type-expect"><a href="#link-type-expect">expect</a></code></td><td><a href="#internal-resource-link" id="linkTypes:internal-resource-link">Internal Resource</a></td><td colspan="2"><em>not allowed</em></td><td class="no"> · </td><td class="no"> · </td><td>Expect an element with the target ID to appear in the current document&#46;</td></tr><tr><td><code id="linkTypes:link-type-external"><a href="#link-type-external">external</a></code></td><td><em>not allowed</em></td><td colspan="2"><a href="#hyperlink-annotation" id="linkTypes:hyperlink-annotation">Annotation</a></td><td class="no"> · </td><td class="no"> · </td><td>Indicates that the referenced document is not part of the same site as the current document&#46;</td></tr><tr><td><code id="linkTypes:link-type-help"><a href="#link-type-help">help</a></code></td><td colspan="3"><a href="#hyperlink" id="linkTypes:hyperlink-5">Hyperlink</a></td><td class="no"> · </td><td class="no"> · </td><td>Provides a link to context&#45;sensitive help&#46;</td></tr><tr><td><code id="linkTypes:rel-icon"><a href="#rel-icon">icon</a></code></td><td><a href="#external-resource-link" id="linkTypes:external-resource-link-2">External Resource</a></td><td colspan="2"><em>not allowed</em></td><td class="no"> · </td><td class="no"> · </td><td>Imports an icon to represent the current document&#46;</td></tr><tr><td><code id="linkTypes:link-type-manifest"><a href="#link-type-manifest">manifest</a></code></td><td><a href="#external-resource-link" id="linkTypes:external-resource-link-3">External Resource</a></td><td colspan="2"><em>not allowed</em></td><td class="no"> · </td><td class="no"> · </td><td>Imports or links to an <a id="linkTypes:application-manifest" href="https://w3c.github.io/manifest/#dfn-manifest" data-x-internal="application-manifest">application manifest</a>&#46; <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsMANIFEST">&#91;MANIFEST&#93;</a></td></tr><tr><td><code id="linkTypes:link-type-modulepreload-2"><a href="#link-type-modulepreload">modulepreload</a></code></td><td><a href="#external-resource-link" id="linkTypes:external-resource-link-4">External Resource</a></td><td colspan="2"><em>not allowed</em></td><td class="yes"> Yes </td><td class="no"> · </td><td>Specifies that the user agent must preemptively <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fetch-a-single-module-script" id="linkTypes:fetch-a-single-module-script">fetch the module      script</a> and store it in the document&#39;s <a href="html-language-foundations--877d6146487f--b55e73c31978.md#concept-document-module-map" id="linkTypes:concept-document-module-map">module map</a> for later      evaluation&#46; Optionally&#44; the module&#39;s dependencies can be fetched as well&#46;</td></tr><tr><td><code id="linkTypes:link-type-license"><a href="#link-type-license">license</a></code></td><td colspan="3"><a href="#hyperlink" id="linkTypes:hyperlink-6">Hyperlink</a></td><td class="no"> · </td><td class="no"> · </td><td>Indicates that the main content of the current document is covered by the copyright license described by the referenced document&#46;</td></tr><tr><td><code id="linkTypes:link-type-next"><a href="#link-type-next">next</a></code></td><td colspan="3"><a href="#hyperlink" id="linkTypes:hyperlink-7">Hyperlink</a></td><td class="no"> · </td><td class="no"> · </td><td>Indicates that the current document is a part of a series&#44; and that the next document in the series is the referenced document&#46;</td></tr><tr><td><code id="linkTypes:link-type-nofollow"><a href="#link-type-nofollow">nofollow</a></code></td><td><em>not allowed</em></td><td colspan="2"><a href="#hyperlink-annotation" id="linkTypes:hyperlink-annotation-2">Annotation</a></td><td class="no"> · </td><td class="no"> · </td><td>Indicates that the current document&#39;s original author or publisher does not endorse the referenced document&#46;</td></tr><tr><td><code id="linkTypes:link-type-noopener"><a href="#link-type-noopener">noopener</a></code></td><td><em>not allowed</em></td><td colspan="2"><a href="#hyperlink-annotation" id="linkTypes:hyperlink-annotation-3">Annotation</a></td><td class="no"> · </td><td class="no"> · </td><td>Creates a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable" id="linkTypes:top-level-traversable">top&#45;level traversable</a> with a non&#45;<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#auxiliary-browsing-context" id="linkTypes:auxiliary-browsing-context">auxiliary browsing      context</a> if the hyperlink would otherwise create one that was auxiliary &#40;i&#46;e&#46;&#44; has an      appropriate <code id="linkTypes:attr-hyperlink-target"><a href="#attr-hyperlink-target">target</a></code> attribute value&#41;&#46;</td></tr><tr><td><code id="linkTypes:link-type-noreferrer"><a href="#link-type-noreferrer">noreferrer</a></code></td><td><em>not allowed</em></td><td colspan="2"><a href="#hyperlink-annotation" id="linkTypes:hyperlink-annotation-4">Annotation</a></td><td class="no"> · </td><td class="no"> · </td><td>No &#96;<code id="linkTypes:http-referer"><a data-x-internal="http-referer" href="https://httpwg.org/specs/rfc7231.html#header.referer">Referer</a></code>&#96; &#40;sic&#41; header will be included&#46;      Additionally&#44; has the same effect as <code id="linkTypes:link-type-noopener-2"><a href="#link-type-noopener">noopener</a></code>&#46;</td></tr><tr><td><code id="linkTypes:link-type-opener"><a href="#link-type-opener">opener</a></code></td><td><em>not allowed</em></td><td colspan="2"><a href="#hyperlink-annotation" id="linkTypes:hyperlink-annotation-5">Annotation</a></td><td class="no"> · </td><td class="no"> · </td><td>Creates an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#auxiliary-browsing-context" id="linkTypes:auxiliary-browsing-context-2">auxiliary browsing context</a> if the hyperlink would otherwise create      a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable" id="linkTypes:top-level-traversable-2">top&#45;level traversable</a> with a non&#45;<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#auxiliary-browsing-context" id="linkTypes:auxiliary-browsing-context-3">auxiliary browsing context</a> &#40;i&#46;e&#46;&#44;      has &#34;<code>&#95;blank</code>&#34; as <code id="linkTypes:attr-hyperlink-target-2"><a href="#attr-hyperlink-target">target</a></code>      attribute value&#41;&#46;</td></tr><tr><td><code id="linkTypes:link-type-pingback-2"><a href="#link-type-pingback">pingback</a></code></td><td><a href="#external-resource-link" id="linkTypes:external-resource-link-5">External Resource</a></td><td colspan="2"><em>not allowed</em></td><td class="yes"> Yes </td><td class="no"> · </td><td>Gives the address of the pingback server that handles pingbacks to the current document&#46;</td></tr><tr><td><code id="linkTypes:link-type-preconnect-2"><a href="#link-type-preconnect">preconnect</a></code></td><td><a href="#external-resource-link" id="linkTypes:external-resource-link-6">External Resource</a></td><td colspan="2"><em>not allowed</em></td><td class="yes"> Yes </td><td class="yes"> Yes </td><td>Specifies that the user agent should preemptively connect to the target resource&#39;s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin" id="linkTypes:concept-origin-2">origin</a>&#46;</td></tr><tr><td><code id="linkTypes:link-type-prefetch-2"><a href="#link-type-prefetch">prefetch</a></code></td><td><a href="#external-resource-link" id="linkTypes:external-resource-link-7">External Resource</a></td><td colspan="2"><em>not allowed</em></td><td class="yes"> Yes </td><td class="no"> · </td><td>Specifies that the user agent should preemptively <a href="https://fetch.spec.whatwg.org/#concept-fetch" id="linkTypes:concept-fetch" data-x-internal="concept-fetch">fetch</a> and cache the target resource as it is likely to be required for a followup <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigate" id="linkTypes:navigate">navigation</a>&#46;</td></tr><tr><td><code id="linkTypes:link-type-preload-2"><a href="#link-type-preload">preload</a></code></td><td><a href="#external-resource-link" id="linkTypes:external-resource-link-8">External Resource</a></td><td colspan="2"><em>not allowed</em></td><td class="yes"> Yes </td><td class="yes"> Yes </td><td>Specifies that the user agent must preemptively <a href="https://fetch.spec.whatwg.org/#concept-fetch" id="linkTypes:concept-fetch-2" data-x-internal="concept-fetch">fetch</a> and cache the target resource for current <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigate" id="linkTypes:navigate-2">navigation</a> according to the <a href="#preload-destination" id="linkTypes:preload-destination">preload destination</a> given by the <code id="linkTypes:attr-link-as"><a href="html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-as">as</a></code> attribute &#40;and the <a href="https://fetch.spec.whatwg.org/#request-priority" id="linkTypes:concept-request-priority" data-x-internal="concept-request-priority">priority</a> associated with the <a href="https://fetch.spec.whatwg.org/#concept-potential-destination-translate" id="linkTypes:concept-potential-destination-translate" data-x-internal="concept-potential-destination-translate">corresponding</a> <a href="https://fetch.spec.whatwg.org/#concept-request-destination" id="linkTypes:concept-request-destination" data-x-internal="concept-request-destination">destination</a>&#41;&#46;</td></tr><tr><td><code id="linkTypes:link-type-prev"><a href="#link-type-prev">prev</a></code></td><td colspan="3"><a href="#hyperlink" id="linkTypes:hyperlink-8">Hyperlink</a></td><td class="no"> · </td><td class="no"> · </td><td>Indicates that the current document is a part of a series&#44; and that the previous document in the series is the referenced document&#46;</td></tr><tr><td><code id="linkTypes:link-type-privacy-policy"><a href="#link-type-privacy-policy">privacy&#45;policy</a></code></td><td colspan="2"><a href="#hyperlink" id="linkTypes:hyperlink-9">Hyperlink</a></td><td><em>not allowed</em></td><td class="no"> · </td><td class="no"> · </td><td>Gives a link to information about the data collection and usage practices that apply to the current document&#46;</td></tr><tr><td><code id="linkTypes:link-type-search"><a href="#link-type-search">search</a></code></td><td colspan="3"><a href="#hyperlink" id="linkTypes:hyperlink-10">Hyperlink</a></td><td class="no"> · </td><td class="no"> · </td><td>Gives a link to a resource that can be used to search through the current document and its related pages&#46;</td></tr><tr><td><code id="linkTypes:link-type-stylesheet-2"><a href="#link-type-stylesheet">stylesheet</a></code></td><td><a href="#external-resource-link" id="linkTypes:external-resource-link-9">External Resource</a></td><td colspan="2"><em>not allowed</em></td><td class="yes"> Yes </td><td class="no"> · </td><td>Imports a style sheet&#46;</td></tr><tr><td><code id="linkTypes:link-type-tag"><a href="#link-type-tag">tag</a></code></td><td><em>not allowed</em></td><td><a href="#hyperlink" id="linkTypes:hyperlink-11">Hyperlink</a></td><td><em>not allowed</em></td><td class="no"> · </td><td class="no"> · </td><td>Gives a tag &#40;identified by the given address&#41; that applies to the current document&#46;</td></tr><tr><td><code id="linkTypes:link-type-terms-of-service"><a href="#link-type-terms-of-service">terms&#45;of&#45;service</a></code></td><td colspan="2"><a href="#hyperlink" id="linkTypes:hyperlink-12">Hyperlink</a></td><td><em>not allowed</em></td><td class="no"> · </td><td class="no"> · </td><td>Gives a link to information about the agreements between the current document&#39;s provider and users who wish to use the current document&#46;</td></tr></tbody></table>

##### <a id="rel&#45;alternate"></a>4&#46;6&#46;8&#46;1 <a id="link&#45;type&#45;alternate"></a>Link type &#34;<strong><code>alternate</code></strong>&#34;[](#rel-alternate)

<strong>⚠</strong>MDN

[Alternative&#95;style&#95;sheets](https://developer.mozilla.org/en-US/docs/Web/CSS/Alternative_style_sheets)

Support in one engine only&#46;

Firefox3&#43;Safari&#63;Chrome1–48

------------------------------------------------------------------------

OperaYesEdgeNo

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet Explorer8&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

The <a id="rel&#45;alternate&#58;rel&#45;alternate"></a><code>[alternate](#rel-alternate)</code> keyword may be used with <a id="rel&#45;alternate&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code>&#44; <a id="rel&#45;alternate&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code>&#44; and <a id="rel&#45;alternate&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements&#46;

The meaning of this keyword depends on the values of the other attributes&#46;

<strong>If the element is a <a id="rel&#45;alternate&#58;the&#45;link&#45;element&#45;2"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element and the <a id="rel&#45;alternate&#58;attr&#45;link&#45;rel"></a><code>[rel](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-rel)</code> attribute also contains the keyword <a id="rel&#45;alternate&#58;link&#45;type&#45;stylesheet"></a><code>[stylesheet](#link-type-stylesheet)</code></strong>

The <a id="rel&#45;alternate&#58;rel&#45;alternate&#45;2"></a><code>[alternate](#rel-alternate)</code> keyword modifies the meaning of the <a id="rel&#45;alternate&#58;link&#45;type&#45;stylesheet&#45;2"></a><code>[stylesheet](#link-type-stylesheet)</code> keyword in the way described for that keyword&#46; The <a id="rel&#45;alternate&#58;rel&#45;alternate&#45;3"></a><code>[alternate](#rel-alternate)</code> keyword does not create a link of its own&#46;

<strong>Example&#58;</strong>

Here&#44; a set of <a id="rel&#45;alternate&#58;the&#45;link&#45;element&#45;3"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements provide some style sheets&#58;

``` text
<!-- a persistent style sheet -->
<link rel="stylesheet" href="default.css">

<!-- the preferred alternate style sheet -->
<link rel="stylesheet" href="green.css" title="Green styles">

<!-- some alternate style sheets -->
<link rel="alternate stylesheet" href="contrast.css" title="High contrast">
<link rel="alternate stylesheet" href="big.css" title="Big fonts">
<link rel="alternate stylesheet" href="wide.css" title="Wide screen">
```

<strong>If the <a id="rel&#45;alternate&#58;rel&#45;alternate&#45;4"></a><code>[alternate](#rel-alternate)</code> keyword is used with the <a id="rel&#45;alternate&#58;attr&#45;hyperlink&#45;type"></a><code>[type](#attr-hyperlink-type)</code> attribute set to the value <code>application&#47;rss&#43;xml</code> or the value <code>application&#47;atom&#43;xml</code></strong>

The keyword creates a <a id="rel&#45;alternate&#58;hyperlink"></a>[hyperlink](#hyperlink) referencing a syndication feed &#40;though not necessarily syndicating exactly the same content as the current page&#41;&#46;

For the purposes of feed autodiscovery&#44; user agents should consider all <a id="rel&#45;alternate&#58;the&#45;link&#45;element&#45;4"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements in the document with the <a id="rel&#45;alternate&#58;rel&#45;alternate&#45;5"></a><code>[alternate](#rel-alternate)</code> keyword used and with their <a id="rel&#45;alternate&#58;attr&#45;hyperlink&#45;type&#45;2"></a><code>[type](#attr-hyperlink-type)</code> attribute set to the value <code>application&#47;rss&#43;xml</code> or the value <code>application&#47;atom&#43;xml</code>&#46; If the user agent has the concept of a default syndication feed&#44; the first such element &#40;in <a id="rel&#45;alternate&#58;tree&#45;order"></a>[tree order](https://dom.spec.whatwg.org/#concept-tree-order)&#41; should be used as the default&#46;

<strong>Example&#58;</strong>

The following <a id="rel&#45;alternate&#58;the&#45;link&#45;element&#45;5"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements give syndication feeds for a blog&#58;

``` text
<link rel="alternate" type="application/atom+xml" href="posts.xml" title="Cool Stuff Blog">
<link rel="alternate" type="application/atom+xml" href="posts.xml?category=robots" title="Cool Stuff Blog: robots category">
<link rel="alternate" type="application/atom+xml" href="comments.xml" title="Cool Stuff Blog: Comments">
```

Such <a id="rel&#45;alternate&#58;the&#45;link&#45;element&#45;6"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements would be used by user agents engaged in feed autodiscovery&#44; with the first being the default &#40;where applicable&#41;&#46;

The following example offers various different syndication feeds to the user&#44; using <a id="rel&#45;alternate&#58;the&#45;a&#45;element&#45;2"></a><code>[a](#the-a-element)</code> elements&#58;

``` text
<p>You can access the planets database using Atom feeds:</p>
<ul>
 <li><a href="recently-visited-planets.xml" rel="alternate" type="application/atom+xml">Recently Visited Planets</a></li>
 <li><a href="known-bad-planets.xml" rel="alternate" type="application/atom+xml">Known Bad Planets</a></li>
 <li><a href="unexplored-planets.xml" rel="alternate" type="application/atom+xml">Unexplored Planets</a></li>
</ul>
```

These links would not be used in feed autodiscovery&#46;

<strong>Otherwise</strong>

The keyword creates a <a id="rel&#45;alternate&#58;hyperlink&#45;2"></a>[hyperlink](#hyperlink) referencing an alternate representation of the current document&#46;

The nature of the referenced document is given by the <a id="rel&#45;alternate&#58;attr&#45;hyperlink&#45;hreflang"></a><code>[hreflang](#attr-hyperlink-hreflang)</code>&#44; and <a id="rel&#45;alternate&#58;attr&#45;hyperlink&#45;type&#45;3"></a><code>[type](#attr-hyperlink-type)</code> attributes&#46;

If the <a id="rel&#45;alternate&#58;rel&#45;alternate&#45;6"></a><code>[alternate](#rel-alternate)</code> keyword is used with the <a id="rel&#45;alternate&#58;attr&#45;hyperlink&#45;hreflang&#45;2"></a><code>[hreflang](#attr-hyperlink-hreflang)</code> attribute&#44; and that attribute&#39;s value differs from the <a id="rel&#45;alternate&#58;document&#45;element"></a>[document element](https://dom.spec.whatwg.org/#document-element)&#39;s <a id="rel&#45;alternate&#58;language"></a>[language](html-language-foundations--877d6146487f--b55e73c31978.md#language)&#44; it indicates that the referenced document is a translation&#46;

If the <a id="rel&#45;alternate&#58;rel&#45;alternate&#45;7"></a><code>[alternate](#rel-alternate)</code> keyword is used with the <a id="rel&#45;alternate&#58;attr&#45;hyperlink&#45;type&#45;4"></a><code>[type](#attr-hyperlink-type)</code> attribute&#44; it indicates that the referenced document is a reformulation of the current document in the specified format&#46;

The <a id="rel&#45;alternate&#58;attr&#45;hyperlink&#45;hreflang&#45;3"></a><code>[hreflang](#attr-hyperlink-hreflang)</code> and <a id="rel&#45;alternate&#58;attr&#45;hyperlink&#45;type&#45;5"></a><code>[type](#attr-hyperlink-type)</code> attributes can be combined when specified with the <a id="rel&#45;alternate&#58;rel&#45;alternate&#45;8"></a><code>[alternate](#rel-alternate)</code> keyword&#46;

<strong>Example&#58;</strong>

The following example shows how you can specify versions of the page that use alternative formats&#44; are aimed at other languages&#44; and that are intended for other media&#58;

``` text
<link rel=alternate href="/en/html" hreflang=en type=text/html title="English HTML">
<link rel=alternate href="/fr/html" hreflang=fr type=text/html title="French HTML">
<link rel=alternate href="/en/html/print" hreflang=en type=text/html media=print title="English HTML (for printing)">
<link rel=alternate href="/fr/html/print" hreflang=fr type=text/html media=print title="French HTML (for printing)">
<link rel=alternate href="/en/pdf" hreflang=en type=application/pdf title="English PDF">
<link rel=alternate href="/fr/pdf" hreflang=fr type=application/pdf title="French PDF">
```

This relationship is transitive — that is&#44; if a document links to two other documents with the link type &#34;<a id="rel&#45;alternate&#58;rel&#45;alternate&#45;9"></a><code>[alternate](#rel-alternate)</code>&#34;&#44; then&#44; in addition to implying that those documents are alternative representations of the first document&#44; it is also implying that those two documents are alternative representations of each other&#46;

##### <a id="link&#45;type&#45;author"></a>4&#46;6&#46;8&#46;2 Link type &#34;<strong><code>author</code></strong>&#34;[](#link-type-author)

The <a id="link&#45;type&#45;author&#58;link&#45;type&#45;author"></a><code>[author](#link-type-author)</code> keyword may be used with <a id="link&#45;type&#45;author&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code>&#44; <a id="link&#45;type&#45;author&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code>&#44; and <a id="link&#45;type&#45;author&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements&#46; This keyword creates a <a id="link&#45;type&#45;author&#58;hyperlink"></a>[hyperlink](#hyperlink)&#46;

For <a id="link&#45;type&#45;author&#58;the&#45;a&#45;element&#45;2"></a><code>[a](#the-a-element)</code> and <a id="link&#45;type&#45;author&#58;the&#45;area&#45;element&#45;2"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements&#44; the <a id="link&#45;type&#45;author&#58;link&#45;type&#45;author&#45;2"></a><code>[author](#link-type-author)</code> keyword indicates that the referenced document provides further information about the author of the nearest <a id="link&#45;type&#45;author&#58;the&#45;article&#45;element"></a><code>[article](#the-article-element)</code> element ancestor of the element defining the hyperlink&#44; if there is one&#44; or of the page as a whole&#44; otherwise&#46;

For <a id="link&#45;type&#45;author&#58;the&#45;link&#45;element&#45;2"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements&#44; the <a id="link&#45;type&#45;author&#58;link&#45;type&#45;author&#45;3"></a><code>[author](#link-type-author)</code> keyword indicates that the referenced document provides further information about the author for the page as a whole&#46;

<strong>Note&#58;</strong>

The &#34;referenced document&#34; can be&#44; and often is&#44; a <a id="link&#45;type&#45;author&#58;mailto&#45;protocol"></a><code>[mailto&#58;](https://www.rfc-editor.org/rfc/rfc6068#section-2)</code> URL giving the email address of the author&#46; [&#91;MAILTO&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsMAILTO)

<strong>Synonyms</strong>&#58; For historical reasons&#44; user agents must also treat <a id="link&#45;type&#45;author&#58;the&#45;link&#45;element&#45;3"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code>&#44; <a id="link&#45;type&#45;author&#58;the&#45;a&#45;element&#45;3"></a><code>[a](#the-a-element)</code>&#44; and <a id="link&#45;type&#45;author&#58;the&#45;area&#45;element&#45;3"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements that have a <code>rev</code> attribute with the value &#34;<code>made</code>&#34; as having the <a id="link&#45;type&#45;author&#58;link&#45;type&#45;author&#45;4"></a><code>[author](#link-type-author)</code> keyword specified as a link relationship&#46;

##### <a id="link&#45;type&#45;bookmark"></a>4&#46;6&#46;8&#46;3 Link type &#34;<strong><code>bookmark</code></strong>&#34;[](#link-type-bookmark)

The <a id="link&#45;type&#45;bookmark&#58;link&#45;type&#45;bookmark"></a><code>[bookmark](#link-type-bookmark)</code> keyword may be used with <a id="link&#45;type&#45;bookmark&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code> and <a id="link&#45;type&#45;bookmark&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements&#46; This keyword creates a <a id="link&#45;type&#45;bookmark&#58;hyperlink"></a>[hyperlink](#hyperlink)&#46;

The <a id="link&#45;type&#45;bookmark&#58;link&#45;type&#45;bookmark&#45;2"></a><code>[bookmark](#link-type-bookmark)</code> keyword gives a permalink for the nearest ancestor <a id="link&#45;type&#45;bookmark&#58;the&#45;article&#45;element"></a><code>[article](#the-article-element)</code> element of the linking element in question&#44; or of the section the linking element is most closely associated with&#44; if there are no ancestor <a id="link&#45;type&#45;bookmark&#58;the&#45;article&#45;element&#45;2"></a><code>[article](#the-article-element)</code> elements&#46;

<strong>Example&#58;</strong>

The following snippet has three permalinks&#46; A user agent could determine which permalink applies to which part of the spec by looking at where the permalinks are given&#46;

``` text
 ...
 <body>
  <h1>Example of permalinks</h1>
  <div id="a">
   <h2>First example</h2>
   <p><a href="a.html" rel="bookmark">This permalink applies to
   only the content from the first H2 to the second H2</a>. The DIV isn't
   exactly that section, but it roughly corresponds to it.</p>
  </div>
  <h2>Second example</h2>
  <article id="b">
   <p><a href="b.html" rel="bookmark">This permalink applies to
   the outer ARTICLE element</a> (which could be, e.g., a blog post).</p>
   <article id="c">
    <p><a href="c.html" rel="bookmark">This permalink applies to
    the inner ARTICLE element</a> (which could be, e.g., a blog comment).</p>
   </article>
  </article>
 </body>
 ...
```

##### <a id="link&#45;type&#45;canonical"></a>4&#46;6&#46;8&#46;4 Link type &#34;<strong><code>canonical</code></strong>&#34;[](#link-type-canonical)

The <a id="link&#45;type&#45;canonical&#58;link&#45;type&#45;canonical"></a><code>[canonical](#link-type-canonical)</code> keyword may be used with <a id="link&#45;type&#45;canonical&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element&#46; This keyword creates a <a id="link&#45;type&#45;canonical&#58;hyperlink"></a>[hyperlink](#hyperlink)&#46;

The <a id="link&#45;type&#45;canonical&#58;link&#45;type&#45;canonical&#45;2"></a><code>[canonical](#link-type-canonical)</code> keyword indicates that URL given by the <a id="link&#45;type&#45;canonical&#58;attr&#45;link&#45;href"></a><code>[href](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-href)</code> attribute is the preferred URL for the current document&#46; That helps search engines reduce duplicate content&#44; as described in more detail in The Canonical Link Relation&#46; [&#91;RFC6596&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsRFC6596)

##### <a id="link&#45;type&#45;dns&#45;prefetch"></a>4&#46;6&#46;8&#46;5 Link type &#34;<strong><code>dns&#45;prefetch</code></strong>&#34;[](#link-type-dns-prefetch)

MDN

[Link&#95;types&#47;dns&#45;prefetch](https://developer.mozilla.org/en-US/docs/Web/HTML/Link_types/dns-prefetch)

Firefox3&#43;Safari&#63;Chrome46&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;NoInternet Explorer&#63;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome AndroidYesWebView Android46&#43;Samsung Internet&#63;Opera Android&#63;

The <a id="link&#45;type&#45;dns&#45;prefetch&#58;link&#45;type&#45;dns&#45;prefetch"></a><code>[dns&#45;prefetch](#link-type-dns-prefetch)</code> keyword may be used with <a id="link&#45;type&#45;dns&#45;prefetch&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements&#46; This keyword creates an <a id="link&#45;type&#45;dns&#45;prefetch&#58;external&#45;resource&#45;link"></a>[external resource link](#external-resource-link)&#46; This keyword is <a id="link&#45;type&#45;dns&#45;prefetch&#58;body&#45;ok"></a>[body&#45;ok](#body-ok)&#46;

The <a id="link&#45;type&#45;dns&#45;prefetch&#58;link&#45;type&#45;dns&#45;prefetch&#45;2"></a><code>[dns&#45;prefetch](#link-type-dns-prefetch)</code> keyword indicates that preemptively performing DNS resolution for the <a id="link&#45;type&#45;dns&#45;prefetch&#58;concept&#45;origin"></a>[origin](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin) of the specified resource is likely to be beneficial&#44; as it is highly likely that the user will require resources located at that <a id="link&#45;type&#45;dns&#45;prefetch&#58;concept&#45;origin&#45;2"></a>[origin](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin)&#44; and the user experience would be improved by preempting the latency costs associated with DNS resolution&#46;

There is no default type for resources given by the <a id="link&#45;type&#45;dns&#45;prefetch&#58;link&#45;type&#45;dns&#45;prefetch&#45;3"></a><code>[dns&#45;prefetch](#link-type-dns-prefetch)</code> keyword&#46;

The appropriate times to <a id="link&#45;type&#45;dns&#45;prefetch&#58;fetch&#45;and&#45;process&#45;the&#45;linked&#45;resource"></a>[fetch and process](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#fetch-and-process-the-linked-resource) this type of link are&#58;

- When the <a id="link&#45;type&#45;dns&#45;prefetch&#58;external&#45;resource&#45;link&#45;2"></a>[external resource link](#external-resource-link) is created on a <a id="link&#45;type&#45;dns&#45;prefetch&#58;the&#45;link&#45;element&#45;2"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element that is already <a id="link&#45;type&#45;dns&#45;prefetch&#58;browsing&#45;context&#45;connected"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected)&#46;

- When the <a id="link&#45;type&#45;dns&#45;prefetch&#58;external&#45;resource&#45;link&#45;3"></a>[external resource link](#external-resource-link)&#39;s <a id="link&#45;type&#45;dns&#45;prefetch&#58;the&#45;link&#45;element&#45;3"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <a id="link&#45;type&#45;dns&#45;prefetch&#58;becomes&#45;browsing&#45;context&#45;connected"></a>[becomes browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#becomes-browsing-context-connected)&#46;

- When the <a id="link&#45;type&#45;dns&#45;prefetch&#58;attr&#45;link&#45;href"></a><code>[href](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-href)</code> attribute of the <a id="link&#45;type&#45;dns&#45;prefetch&#58;the&#45;link&#45;element&#45;4"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element of an <a id="link&#45;type&#45;dns&#45;prefetch&#58;external&#45;resource&#45;link&#45;4"></a>[external resource link](#external-resource-link) that is already <a id="link&#45;type&#45;dns&#45;prefetch&#58;browsing&#45;context&#45;connected&#45;2"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected) is changed&#46;

The <a id="link&#45;type&#45;dns&#45;prefetch&#58;fetch&#45;and&#45;process&#45;the&#45;linked&#45;resource&#45;2"></a>[fetch and process the linked resource](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#fetch-and-process-the-linked-resource) steps for this type of linked resource&#44; given a <a id="link&#45;type&#45;dns&#45;prefetch&#58;the&#45;link&#45;element&#45;5"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <var>el</var>&#44; are&#58;

1.  Let <var>url</var> be the result of <a id="link&#45;type&#45;dns&#45;prefetch&#58;encoding&#45;parsing&#45;a&#45;url"></a>[encoding&#45;parsing a URL](html-language-foundations--877d6146487f--b55e73c31978.md#encoding-parsing-a-url) given <var>el</var>&#39;s <a id="link&#45;type&#45;dns&#45;prefetch&#58;attr&#45;link&#45;href&#45;2"></a><code>[href](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-href)</code> attribute&#39;s value&#44; relative to <var>el</var>&#39;s <a id="link&#45;type&#45;dns&#45;prefetch&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46;

2.  If <var>url</var> is failure&#44; then return&#46;

3.  Let <var>partitionKey</var> be the result of <a id="link&#45;type&#45;dns&#45;prefetch&#58;determine&#45;the&#45;network&#45;partition&#45;key"></a>[determining the network partition key](https://fetch.spec.whatwg.org/#determine-the-network-partition-key) given <var>el</var>&#39;s <a id="link&#45;type&#45;dns&#45;prefetch&#58;node&#45;document&#45;2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="link&#45;type&#45;dns&#45;prefetch&#58;relevant&#45;settings&#45;object"></a>[relevant settings object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant-settings-object)&#46;

4.  The user agent should <a id="link&#45;type&#45;dns&#45;prefetch&#58;resolve&#45;an&#45;origin"></a>[resolve an origin](https://fetch.spec.whatwg.org/#resolve-an-origin) given <var>partitionKey</var> and <var>url</var>&#39;s <a id="link&#45;type&#45;dns&#45;prefetch&#58;concept&#45;url&#45;origin"></a>[origin](https://url.spec.whatwg.org/#concept-url-origin)&#46;

    <strong>Note&#58;</strong>

    As the results of this algorithm can be cached&#44; future fetches could be faster&#46;

##### <a id="link&#45;type&#45;expect"></a>4&#46;6&#46;8&#46;6 Link type &#34;<strong><code>expect</code></strong>&#34;[](#link-type-expect)

The <a id="link&#45;type&#45;expect&#58;link&#45;type&#45;expect"></a><code>[expect](#link-type-expect)</code> keyword may be used with <a id="link&#45;type&#45;expect&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements&#46; This keyword creates an <a id="link&#45;type&#45;expect&#58;internal&#45;resource&#45;link"></a>[internal resource link](#internal-resource-link)&#46;

An <a id="link&#45;type&#45;expect&#58;internal&#45;resource&#45;link&#45;2"></a>[internal resource link](#internal-resource-link) created by the <a id="link&#45;type&#45;expect&#58;link&#45;type&#45;expect&#45;2"></a><code>[expect](#link-type-expect)</code> keyword can be used to <a id="link&#45;type&#45;expect&#58;render&#45;blocked"></a>[block rendering](html-language-foundations--877d6146487f--b55e73c31978.md#render-blocked) until the element that it <a id="link&#45;type&#45;expect&#58;the&#45;indicated&#45;part&#45;of&#45;the&#45;document"></a>[indicates](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-indicated-part-of-the-document) is connected to the document and fully parsed&#46;

There is no default type for resources given by the <a id="link&#45;type&#45;expect&#58;link&#45;type&#45;expect&#45;3"></a><code>[expect](#link-type-expect)</code> keyword&#46;

Whenever any of the following conditions occur for a <a id="link&#45;type&#45;expect&#58;the&#45;link&#45;element&#45;2"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <var>el</var>&#58;

- the <a id="link&#45;type&#45;expect&#58;link&#45;type&#45;expect&#45;4"></a><code>[expect](#link-type-expect)</code> <a id="link&#45;type&#45;expect&#58;internal&#45;resource&#45;link&#45;3"></a>[internal resource link](#internal-resource-link) is created on <var>el</var> that is already <a id="link&#45;type&#45;expect&#58;browsing&#45;context&#45;connected"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected)&#59;

- an <a id="link&#45;type&#45;expect&#58;link&#45;type&#45;expect&#45;5"></a><code>[expect](#link-type-expect)</code> <a id="link&#45;type&#45;expect&#58;internal&#45;resource&#45;link&#45;4"></a>[internal resource link](#internal-resource-link) has been created on <var>el</var> and <var>el</var> becomes <a id="link&#45;type&#45;expect&#58;browsing&#45;context&#45;connected&#45;2"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected)&#59;

- an <a id="link&#45;type&#45;expect&#58;link&#45;type&#45;expect&#45;6"></a><code>[expect](#link-type-expect)</code> <a id="link&#45;type&#45;expect&#58;internal&#45;resource&#45;link&#45;5"></a>[internal resource link](#internal-resource-link) has been created on <var>el</var>&#44; <var>el</var> is already <a id="link&#45;type&#45;expect&#58;browsing&#45;context&#45;connected&#45;3"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected)&#44; and <var>el</var>&#39;s <a id="link&#45;type&#45;expect&#58;attr&#45;link&#45;href"></a><code>[href](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-href)</code> attribute is set&#44; changed&#44; or removed&#59; or

- an <a id="link&#45;type&#45;expect&#58;link&#45;type&#45;expect&#45;7"></a><code>[expect](#link-type-expect)</code> <a id="link&#45;type&#45;expect&#58;internal&#45;resource&#45;link&#45;6"></a>[internal resource link](#internal-resource-link) has been created on <var>el</var>&#44; <var>el</var> is already <a id="link&#45;type&#45;expect&#58;browsing&#45;context&#45;connected&#45;4"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected)&#44; and <var>el</var>&#39;s <a id="link&#45;type&#45;expect&#58;attr&#45;link&#45;media"></a><code>[media](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-media)</code> attribute is set&#44; changed&#44; or removed&#44;

then <a id="link&#45;type&#45;expect&#58;process&#45;internal&#45;resource&#45;link"></a>[process](#process-internal-resource-link) <var>el</var>&#46;

To <a id="process&#45;internal&#45;resource&#45;link"></a><strong>process internal resource link</strong> given a <a id="link&#45;type&#45;expect&#58;the&#45;link&#45;element&#45;3"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <var>el</var>&#44; run these steps&#58;

1.  Let <var>doc</var> be <var>el</var>&#39;s <a id="link&#45;type&#45;expect&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46;

2.  Let <var>url</var> be the result of <a id="link&#45;type&#45;expect&#58;encoding&#45;parsing&#45;a&#45;url"></a>[encoding&#45;parsing a URL](html-language-foundations--877d6146487f--b55e73c31978.md#encoding-parsing-a-url) given <var>el</var>&#39;s <a id="link&#45;type&#45;expect&#58;attr&#45;link&#45;href&#45;2"></a><code>[href](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-href)</code> attribute&#39;s value&#44; relative to <var>doc</var>&#46;

3.  If this fails&#44; or if <var>url</var> does not <a id="link&#45;type&#45;expect&#58;concept&#45;url&#45;equals"></a>[equal](https://url.spec.whatwg.org/#concept-url-equals) <var>doc</var>&#39;s <a id="link&#45;type&#45;expect&#58;the&#45;document&#39;s&#45;address"></a>[URL](https://dom.spec.whatwg.org/#concept-document-url) with <a id="link&#45;type&#45;expect&#58;url&#45;equals&#45;exclude&#45;fragments"></a><em>[exclude fragments](https://url.spec.whatwg.org/#url-equals-exclude-fragments)</em> set to true&#44; then <a id="link&#45;type&#45;expect&#58;unblock&#45;rendering"></a>[unblock rendering](html-language-foundations--877d6146487f--b55e73c31978.md#unblock-rendering) on <var>el</var> and return&#46;

4.  Let <var>indicatedElement</var> be the result of <a id="link&#45;type&#45;expect&#58;select&#45;the&#45;indicated&#45;part"></a>[selecting the indicated part](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#select-the-indicated-part) given <var>doc</var> and <var>url</var>&#46;

5.  If all of the following are true&#58;

    - <var>doc</var>&#39;s <a id="link&#45;type&#45;expect&#58;current&#45;document&#45;readiness"></a>[current document readiness](html-language-foundations--877d6146487f--b55e73c31978.md#current-document-readiness) is &#34;<code>loading</code>&#34;&#59;

    - <var>el</var> creates an <a id="link&#45;type&#45;expect&#58;internal&#45;resource&#45;link&#45;7"></a>[internal resource link](#internal-resource-link)&#59;

    - <var>el</var> is <a id="link&#45;type&#45;expect&#58;browsing&#45;context&#45;connected&#45;5"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected)&#59;

    - <var>el</var>&#39;s <a id="link&#45;type&#45;expect&#58;attr&#45;link&#45;rel"></a><code>[rel](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-rel)</code> attribute contains <a id="link&#45;type&#45;expect&#58;link&#45;type&#45;expect&#45;8"></a><code>[expect](#link-type-expect)</code>&#59;

    - <var>el</var> is <a id="link&#45;type&#45;expect&#58;potentially&#45;render&#45;blocking"></a>[potentially render&#45;blocking](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#potentially-render-blocking)&#59;

    - <var>el</var>&#39;s <a id="link&#45;type&#45;expect&#58;attr&#45;link&#45;media&#45;2"></a><code>[media](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-media)</code> attribute <a id="link&#45;type&#45;expect&#58;matches&#45;the&#45;environment"></a>[matches the environment](html-language-foundations--877d6146487f--b55e73c31978.md#matches-the-environment)&#59; and

    - <var>indicatedElement</var> is not an element&#44; or is on a <a id="link&#45;type&#45;expect&#58;stack&#45;of&#45;open&#45;elements"></a>[stack of open elements](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#stack-of-open-elements) of an <a id="link&#45;type&#45;expect&#58;html&#45;parser"></a>[HTML parser](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#html-parser) whose associated <a id="link&#45;type&#45;expect&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> is <var>doc</var>&#44;

    then <a id="link&#45;type&#45;expect&#58;block&#45;rendering"></a>[block rendering](html-language-foundations--877d6146487f--b55e73c31978.md#block-rendering) on <var>el</var>&#46;

6.  Otherwise&#44; <a id="link&#45;type&#45;expect&#58;unblock&#45;rendering&#45;2"></a>[unblock rendering](html-language-foundations--877d6146487f--b55e73c31978.md#unblock-rendering) on <var>el</var>&#46;

To <a id="process&#45;internal&#45;resource&#45;links"></a><strong>process internal resource links</strong> given a <a id="link&#45;type&#45;expect&#58;document&#45;2"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> <var>doc</var>&#58;

1.  <a id="link&#45;type&#45;expect&#58;list&#45;iterate"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <a id="link&#45;type&#45;expect&#58;link&#45;type&#45;expect&#45;9"></a><code>[expect](#link-type-expect)</code> <a id="link&#45;type&#45;expect&#58;the&#45;link&#45;element&#45;4"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <var>link</var> in <var>doc</var>&#39;s <a id="link&#45;type&#45;expect&#58;render&#45;blocking&#45;element&#45;set"></a>[render&#45;blocking element set](html-language-foundations--877d6146487f--b55e73c31978.md#render-blocking-element-set)&#44; <a id="link&#45;type&#45;expect&#58;process&#45;internal&#45;resource&#45;link&#45;2"></a>[process](#process-internal-resource-link) <var>link</var>&#46;

The following <a id="link&#45;type&#45;expect&#58;concept&#45;element&#45;attributes&#45;change&#45;ext"></a>[attribute change steps](https://dom.spec.whatwg.org/#concept-element-attributes-change-ext)&#44; given <var>element</var>&#44; <var>localName</var>&#44; <var>oldValue</var>&#44; <var>value</var>&#44; and <var>namespace</var>&#44; are used to ensure <a id="link&#45;type&#45;expect&#58;link&#45;type&#45;expect&#45;10"></a><code>[expect](#link-type-expect)</code> <a id="link&#45;type&#45;expect&#58;the&#45;link&#45;element&#45;5"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements respond to dynamic <a id="link&#45;type&#45;expect&#58;the&#45;id&#45;attribute"></a><code>[id](html-language-foundations--877d6146487f--b55e73c31978.md#the-id-attribute)</code> and <a id="link&#45;type&#45;expect&#58;attr&#45;a&#45;name"></a><code>[name](html-language-obsolete--877d6146487f--66870e9cdabe.md#attr-a-name)</code> changes&#58;

1.  If <var>namespace</var> is not null&#44; then return&#46;

2.  If <var>element</var> is in a <a id="link&#45;type&#45;expect&#58;stack&#45;of&#45;open&#45;elements&#45;2"></a>[stack of open elements](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#stack-of-open-elements) of an <a id="link&#45;type&#45;expect&#58;html&#45;parser&#45;2"></a>[HTML parser](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#html-parser)&#44; then return&#46;

3.  If any of the following is true&#58;

    - <var>localName</var> is <a id="link&#45;type&#45;expect&#58;the&#45;id&#45;attribute&#45;2"></a><code>[id](html-language-foundations--877d6146487f--b55e73c31978.md#the-id-attribute)</code>&#59; or

    - <var>localName</var> is <a id="link&#45;type&#45;expect&#58;attr&#45;a&#45;name&#45;2"></a><code>[name](html-language-obsolete--877d6146487f--66870e9cdabe.md#attr-a-name)</code> and <var>element</var> is an <a id="link&#45;type&#45;expect&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code> element&#44;

    then <a id="link&#45;type&#45;expect&#58;process&#45;internal&#45;resource&#45;links"></a>[process internal resource links](#process-internal-resource-links) given <var>element</var>&#39;s <a id="link&#45;type&#45;expect&#58;node&#45;document&#45;2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46;

##### <a id="link&#45;type&#45;external"></a>4&#46;6&#46;8&#46;7 Link type &#34;<strong><code>external</code></strong>&#34;[](#link-type-external)

The <a id="link&#45;type&#45;external&#58;link&#45;type&#45;external"></a><code>[external](#link-type-external)</code> keyword may be used with <a id="link&#45;type&#45;external&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code>&#44; <a id="link&#45;type&#45;external&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code>&#44; and <a id="link&#45;type&#45;external&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements&#46; This keyword does not create a <a id="link&#45;type&#45;external&#58;hyperlink"></a>[hyperlink](#hyperlink)&#44; but <a id="link&#45;type&#45;external&#58;hyperlink&#45;annotation"></a>[annotates](#hyperlink-annotation) any other hyperlinks created by the element &#40;the implied hyperlink&#44; if no other keywords create one&#41;&#46;

The <a id="link&#45;type&#45;external&#58;link&#45;type&#45;external&#45;2"></a><code>[external](#link-type-external)</code> keyword indicates that the link is leading to a document that is not part of the site that the current document forms a part of&#46;

##### <a id="link&#45;type&#45;help"></a>4&#46;6&#46;8&#46;8 Link type &#34;<strong><code>help</code></strong>&#34;[](#link-type-help)

The <a id="link&#45;type&#45;help&#58;link&#45;type&#45;help"></a><code>[help](#link-type-help)</code> keyword may be used with <a id="link&#45;type&#45;help&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code>&#44; <a id="link&#45;type&#45;help&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code>&#44; <a id="link&#45;type&#45;help&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code>&#44; and <a id="link&#45;type&#45;help&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements&#46; This keyword creates a <a id="link&#45;type&#45;help&#58;hyperlink"></a>[hyperlink](#hyperlink)&#46;

For <a id="link&#45;type&#45;help&#58;the&#45;a&#45;element&#45;2"></a><code>[a](#the-a-element)</code>&#44; <a id="link&#45;type&#45;help&#58;the&#45;area&#45;element&#45;2"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code>&#44; and <a id="link&#45;type&#45;help&#58;the&#45;form&#45;element&#45;2"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements&#44; the <a id="link&#45;type&#45;help&#58;link&#45;type&#45;help&#45;2"></a><code>[help](#link-type-help)</code> keyword indicates that the referenced document provides further help information for the parent of the element defining the hyperlink&#44; and its children&#46;

<strong>Example&#58;</strong>

In the following example&#44; the form control has associated context&#45;sensitive help&#46; The user agent could use this information&#44; for example&#44; displaying the referenced document if the user presses the &#34;Help&#34; or &#34;F1&#34; key&#46;

``` text
 <p><label> Topic: <input name=topic> <a href="help/topic.html" rel="help">(Help)</a></label></p>
```

For <a id="link&#45;type&#45;help&#58;the&#45;link&#45;element&#45;2"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements&#44; the <a id="link&#45;type&#45;help&#58;link&#45;type&#45;help&#45;3"></a><code>[help](#link-type-help)</code> keyword indicates that the referenced document provides help for the page as a whole&#46;

For <a id="link&#45;type&#45;help&#58;the&#45;a&#45;element&#45;3"></a><code>[a](#the-a-element)</code> and <a id="link&#45;type&#45;help&#58;the&#45;area&#45;element&#45;3"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements&#44; on some browsers&#44; the <a id="link&#45;type&#45;help&#58;link&#45;type&#45;help&#45;4"></a><code>[help](#link-type-help)</code> keyword causes the link to use a different cursor&#46;

##### <a id="rel&#45;icon"></a>4&#46;6&#46;8&#46;9 Link type &#34;<strong><code>icon</code></strong>&#34;[](#rel-icon)

<strong>✔</strong>MDN

[Link&#95;types&#35;icon](https://developer.mozilla.org/en-US/docs/Web/HTML/Link_types#icon)

Support in all current engines&#46;

Firefox2&#43;Safari3&#46;1&#43;Chrome4&#43;

------------------------------------------------------------------------

Opera9&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer11

------------------------------------------------------------------------

Firefox Android4&#43;Safari iOSNoChrome Android18&#43;WebView Android38&#43;Samsung Internet4&#46;0&#43;Opera AndroidNo

------------------------------------------------------------------------

[caniuse&#46;com table](https://caniuse.com/#feat=link-icon-png)

The <a id="rel&#45;icon&#58;rel&#45;icon"></a><code>[icon](#rel-icon)</code> keyword may be used with <a id="rel&#45;icon&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements&#46; This keyword creates an <a id="rel&#45;icon&#58;external&#45;resource&#45;link"></a>[external resource link](#external-resource-link)&#46;

The specified resource is an icon representing the page or site&#44; and should be used by the user agent when representing the page in the user interface&#46;

Icons could be auditory icons&#44; visual icons&#44; or other kinds of icons&#46; If multiple icons are provided&#44; the user agent must select the most appropriate icon according to the <a id="rel&#45;icon&#58;attr&#45;link&#45;type"></a><code>[type](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-type)</code>&#44; <a id="rel&#45;icon&#58;attr&#45;link&#45;media"></a><code>[media](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-media)</code>&#44; and <a id="rel&#45;icon&#58;attr&#45;link&#45;sizes"></a><code>[sizes](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-sizes)</code> attributes&#46; If there are multiple equally appropriate icons&#44; user agents must use the last one declared in <a id="rel&#45;icon&#58;tree&#45;order"></a>[tree order](https://dom.spec.whatwg.org/#concept-tree-order) at the time that the user agent collected the list of icons&#46; If the user agent tries to use an icon but that icon is determined&#44; upon closer examination&#44; to in fact be inappropriate &#40;e&#46;g&#46; because it uses an unsupported format&#41;&#44; then the user agent must try the next&#45;most&#45;appropriate icon as determined by the attributes&#46;

<strong>Note&#58;</strong>

User agents are not required to update icons when the list of icons changes&#44; but are encouraged to do so&#46;

There is no default type for resources given by the <a id="rel&#45;icon&#58;rel&#45;icon&#45;2"></a><code>[icon](#rel-icon)</code> keyword&#46; However&#44; for the purposes of [determining the type of the resource](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#concept-link-type-sniffing)&#44; user agents must expect the resource to be an image&#46;

The <a id="rel&#45;icon&#58;attr&#45;link&#45;sizes&#45;2"></a><code>[sizes](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-sizes)</code> keywords represent icon sizes in raw pixels &#40;as opposed to <a id="rel&#45;icon&#58;&#39;px&#39;"></a>[CSS pixels](https://drafts.csswg.org/css-values/#px)&#41;&#46;

<strong>Note&#58;</strong>

An icon that is 50 <a id="rel&#45;icon&#58;&#39;px&#39;&#45;2"></a>[CSS pixels](https://drafts.csswg.org/css-values/#px) wide intended for displays with a device pixel density of two device pixels per <a id="rel&#45;icon&#58;&#39;px&#39;&#45;3"></a>[CSS pixel](https://drafts.csswg.org/css-values/#px) &#40;2x&#44; 192dpi&#41; would have a width of 100 raw pixels&#46; This feature does not support indicating that a different resource is to be used for small high&#45;resolution icons vs large low&#45;resolution icons &#40;e&#46;g&#46; 50×50 2x vs 100×100 1x&#41;&#46;

To parse and process the attribute&#39;s value&#44; the user agent must first <a id="rel&#45;icon&#58;split&#45;a&#45;string&#45;on&#45;spaces"></a>[split the attribute&#39;s value on ASCII whitespace](https://infra.spec.whatwg.org/#split-on-ascii-whitespace)&#44; and must then parse each resulting keyword to determine what it represents&#46;

The <a id="attr&#45;link&#45;sizes&#45;any"></a><strong><code>any</code></strong> keyword represents that the resource contains a scalable icon&#44; e&#46;g&#46; as provided by an SVG image&#46;

Other keywords must be further parsed as follows to determine what they represent&#58;

1.  If the keyword doesn&#39;t contain exactly one U&#43;0078 LATIN SMALL LETTER X or U&#43;0058 LATIN CAPITAL LETTER X character&#44; then this keyword doesn&#39;t represent anything&#46; Return for that keyword&#46;

2.  Let <var>width string</var> be the string before the &#34;<code>x</code>&#34; or &#34;<code>X</code>&#34;&#46;

3.  Let <var>height string</var> be the string after the &#34;<code>x</code>&#34; or &#34;<code>X</code>&#34;&#46;

4.  If either <var>width string</var> or <var>height string</var> start with a U&#43;0030 DIGIT ZERO &#40;0&#41; character or contain any characters other than <a id="rel&#45;icon&#58;ascii&#45;digits"></a>[ASCII digits](https://infra.spec.whatwg.org/#ascii-digit)&#44; then this keyword doesn&#39;t represent anything&#46; Return for that keyword&#46;

5.  Apply the <a id="rel&#45;icon&#58;rules&#45;for&#45;parsing&#45;non&#45;negative&#45;integers"></a>[rules for parsing non&#45;negative integers](html-language-foundations--877d6146487f--b55e73c31978.md#rules-for-parsing-non-negative-integers) to <var>width string</var> to obtain <var>width</var>&#46;

6.  Apply the <a id="rel&#45;icon&#58;rules&#45;for&#45;parsing&#45;non&#45;negative&#45;integers&#45;2"></a>[rules for parsing non&#45;negative integers](html-language-foundations--877d6146487f--b55e73c31978.md#rules-for-parsing-non-negative-integers) to <var>height string</var> to obtain <var>height</var>&#46;

7.  The keyword represents that the resource contains a bitmap icon with a width of <var>width</var> device pixels and a height of <var>height</var> device pixels&#46;

The keywords specified on the <a id="rel&#45;icon&#58;attr&#45;link&#45;sizes&#45;3"></a><code>[sizes](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-sizes)</code> attribute must not represent icon sizes that are not actually available in the linked resource&#46;

The <a id="rel&#45;icon&#58;linked&#45;resource&#45;fetch&#45;setup&#45;steps"></a>[linked resource fetch setup steps](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#linked-resource-fetch-setup-steps) for this type of linked resource&#44; given a <a id="rel&#45;icon&#58;the&#45;link&#45;element&#45;2"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <var>el</var> and <a id="rel&#45;icon&#58;concept&#45;request"></a>[request](https://fetch.spec.whatwg.org/#concept-request) <var>request</var>&#44; are&#58;

1.  Set <var>request</var>&#39;s <a id="rel&#45;icon&#58;concept&#45;request&#45;destination"></a>[destination](https://fetch.spec.whatwg.org/#concept-request-destination) to &#34;<code>image</code>&#34;&#46;

2.  Return true&#46;

The <a id="rel&#45;icon&#58;process&#45;a&#45;link&#45;header"></a>[process a link header](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#process-a-link-header) steps for this type of linked resource are to do nothing&#46;

In the absence of a <a id="rel&#45;icon&#58;the&#45;link&#45;element&#45;3"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> with the <a id="rel&#45;icon&#58;rel&#45;icon&#45;3"></a><code>[icon](#rel-icon)</code> keyword&#44; for <a id="rel&#45;icon&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> objects whose <a id="rel&#45;icon&#58;the&#45;document&#39;s&#45;address"></a>[URL](https://dom.spec.whatwg.org/#concept-document-url)&#39;s <a id="rel&#45;icon&#58;concept&#45;url&#45;scheme"></a>[scheme](https://url.spec.whatwg.org/#concept-url-scheme) is an <a id="rel&#45;icon&#58;http&#40;s&#41;&#45;scheme"></a>[HTTP&#40;S&#41; scheme](https://fetch.spec.whatwg.org/#http-scheme)&#44; user agents may instead run these steps&#58;

1.  Let <var>request</var> be a new <a id="rel&#45;icon&#58;concept&#45;request&#45;2"></a>[request](https://fetch.spec.whatwg.org/#concept-request) whose <a id="rel&#45;icon&#58;concept&#45;request&#45;url"></a>[URL](https://fetch.spec.whatwg.org/#concept-request-url) is the <a id="rel&#45;icon&#58;url&#45;record"></a>[URL record](https://url.spec.whatwg.org/#concept-url) obtained by resolving the <a id="rel&#45;icon&#58;url"></a>[URL](https://url.spec.whatwg.org/#concept-url) &#34;<code>&#47;favicon&#46;ico</code>&#34; against the <a id="rel&#45;icon&#58;document&#45;2"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> object&#39;s <a id="rel&#45;icon&#58;the&#45;document&#39;s&#45;address&#45;2"></a>[URL](https://dom.spec.whatwg.org/#concept-document-url)&#44; <a id="rel&#45;icon&#58;concept&#45;request&#45;client"></a>[client](https://fetch.spec.whatwg.org/#concept-request-client) is the <a id="rel&#45;icon&#58;document&#45;3"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> object&#39;s <a id="rel&#45;icon&#58;relevant&#45;settings&#45;object"></a>[relevant settings object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant-settings-object)&#44; <a id="rel&#45;icon&#58;concept&#45;request&#45;destination&#45;2"></a>[destination](https://fetch.spec.whatwg.org/#concept-request-destination) is &#34;<code>image</code>&#34;&#44; <a id="rel&#45;icon&#58;concept&#45;request&#45;credentials&#45;mode"></a>[credentials mode](https://fetch.spec.whatwg.org/#concept-request-credentials-mode) is &#34;<code>include</code>&#34;&#44; and whose <a id="rel&#45;icon&#58;use&#45;url&#45;credentials&#45;flag"></a>[use&#45;URL&#45;credentials flag](https://fetch.spec.whatwg.org/#concept-request-use-url-credentials-flag) is set&#46;

2.  <a id="rel&#45;icon&#58;concept&#45;fetch"></a>[Fetch](https://fetch.spec.whatwg.org/#concept-fetch) <var>request</var>&#44; with <a id="rel&#45;icon&#58;processresponse"></a><em>[processResponse](https://fetch.spec.whatwg.org/#process-response)</em> set to the following steps given <a id="rel&#45;icon&#58;concept&#45;response"></a>[response](https://fetch.spec.whatwg.org/#concept-response) <var>response</var>&#58;

    1.  Use <var>response</var>&#39;s <a id="rel&#45;icon&#58;unsafe&#45;response"></a>[unsafe response](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unsafe-response) as an icon as if it had been declared using the <a id="rel&#45;icon&#58;rel&#45;icon&#45;4"></a><code>[icon](#rel-icon)</code> keyword&#46;

<strong>Example&#58;</strong>

The following snippet shows the top part of an application with several icons&#46;

``` text
<!DOCTYPE HTML>
<html lang="en">
 <head>
  <title>lsForums — Inbox</title>
  <link rel=icon href=favicon.png sizes="16x16" type="image/png">
  <link rel=icon href=windows.ico sizes="32x32 48x48" type="image/vnd.microsoft.icon">
  <link rel=icon href=mac.icns sizes="128x128 512x512 8192x8192 32768x32768">
  <link rel=icon href=iphone.png sizes="57x57" type="image/png">
  <link rel=icon href=gnome.svg sizes="any" type="image/svg+xml">
  <link rel=stylesheet href=lsforums.css>
  <script src=lsforums.js></script>
  <meta name=application-name content="lsForums">
 </head>
 <body>
  ...
```

For historical reasons&#44; the <a id="rel&#45;icon&#58;rel&#45;icon&#45;5"></a><code>[icon](#rel-icon)</code> keyword may be preceded by the keyword &#34;<code>shortcut</code>&#34;&#46; If the &#34;<code>shortcut</code>&#34; keyword is present&#44; the <a id="rel&#45;icon&#58;attr&#45;hyperlink&#45;rel"></a><code>[rel](#attr-hyperlink-rel)</code> attribute&#39;s entire value must be an <a id="rel&#45;icon&#58;ascii&#45;case&#45;insensitive"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for the string &#34;<code>shortcut icon</code>&#34; &#40;with a single U&#43;0020 SPACE character between the tokens and no other <a id="rel&#45;icon&#58;space&#45;characters"></a>[ASCII whitespace](https://infra.spec.whatwg.org/#ascii-whitespace)&#41;&#46;

##### <a id="link&#45;type&#45;license"></a>4&#46;6&#46;8&#46;10 Link type &#34;<strong><code>license</code></strong>&#34;[](#link-type-license)

The <a id="link&#45;type&#45;license&#58;link&#45;type&#45;license"></a><code>[license](#link-type-license)</code> keyword may be used with <a id="link&#45;type&#45;license&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code>&#44; <a id="link&#45;type&#45;license&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code>&#44; <a id="link&#45;type&#45;license&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code>&#44; and <a id="link&#45;type&#45;license&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements&#46; This keyword creates a <a id="link&#45;type&#45;license&#58;hyperlink"></a>[hyperlink](#hyperlink)&#46;

The <a id="link&#45;type&#45;license&#58;link&#45;type&#45;license&#45;2"></a><code>[license](#link-type-license)</code> keyword indicates that the referenced document provides the copyright license terms under which the main content of the current document is provided&#46;

This specification does not specify how to distinguish between the main content of a document and content that is not deemed to be part of that main content&#46; The distinction should be made clear to the user&#46;

<strong>Example&#58;</strong>

Consider a photo sharing site&#46; A page on that site might describe and show a photograph&#44; and the page might be marked up as follows&#58;

``` text
<!DOCTYPE HTML>
<html lang="en">
 <head>
  <title>Exampl Pictures: Kissat</title>
  <link rel="stylesheet" href="/style/default">
 </head>
 <body>
  <h1>Kissat</h1>
  <nav>
   <a href="../">Return to photo index</a>
  </nav>
  <figure>
   <img src="/pix/39627052_fd8dcd98b5.jpg">
   <figcaption>Kissat</figcaption>
  </figure>
  <p>One of them has six toes!</p>
  <p><small><a rel="license" href="http://www.opensource.org/licenses/mit-license.php">MIT Licensed</a></small></p>
  <footer>
   <a href="/">Home</a> | <a href="../">Photo index</a>
   <p><small>© copyright 2009 Exampl Pictures. All Rights Reserved.</small></p>
  </footer>
 </body>
</html>
```

In this case the <a id="link&#45;type&#45;license&#58;link&#45;type&#45;license&#45;3"></a><code>[license](#link-type-license)</code> applies to just the photo &#40;the main content of the document&#41;&#44; not the whole document&#46; In particular not the design of the page itself&#44; which is covered by the copyright given at the bottom of the document&#46; This could be made clearer in the styling &#40;e&#46;g&#46; making the license link prominently positioned near the photograph&#44; while having the page copyright in light small text at the foot of the page&#41;&#46;

<strong>Synonyms</strong>&#58; For historical reasons&#44; user agents must also treat the keyword &#34;<code>copyright</code>&#34; like the <a id="link&#45;type&#45;license&#58;link&#45;type&#45;license&#45;4"></a><code>[license](#link-type-license)</code> keyword&#46;

##### <a id="link&#45;type&#45;manifest"></a>4&#46;6&#46;8&#46;11 Link type &#34;<strong><code>manifest</code></strong>&#34;[](#link-type-manifest)

<strong>⚠</strong>MDN

[Link&#95;types&#47;manifest](https://developer.mozilla.org/en-US/docs/Web/HTML/Link_types/manifest)

Support in one engine only&#46;

Firefox&#63;Safari&#63;ChromeNo

------------------------------------------------------------------------

Opera&#63;EdgeNo

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet Explorer&#63;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android39&#43;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

The <a id="link&#45;type&#45;manifest&#58;link&#45;type&#45;manifest"></a><code>[manifest](#link-type-manifest)</code> keyword may be used with <a id="link&#45;type&#45;manifest&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements&#46; This keyword creates an <a id="link&#45;type&#45;manifest&#58;external&#45;resource&#45;link"></a>[external resource link](#external-resource-link)&#46;

The <a id="link&#45;type&#45;manifest&#58;link&#45;type&#45;manifest&#45;2"></a><code>[manifest](#link-type-manifest)</code> keyword indicates the manifest file that provides metadata associated with the current document&#46;

There is no default type for resources given by the <a id="link&#45;type&#45;manifest&#58;link&#45;type&#45;manifest&#45;3"></a><code>[manifest](#link-type-manifest)</code> keyword&#46;

When a web application is not <a id="link&#45;type&#45;manifest&#58;installed&#45;web&#45;application"></a>[installed](https://w3c.github.io/manifest/#dfn-installed-web-application)&#44; the appropriate time to <a id="link&#45;type&#45;manifest&#58;fetch&#45;and&#45;process&#45;the&#45;linked&#45;resource"></a>[fetch and process the linked resource](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#fetch-and-process-the-linked-resource) for this link type is when the user agent deems it necessary&#46; For example&#44; when the user chooses to <a id="link&#45;type&#45;manifest&#58;installed&#45;web&#45;application&#45;2"></a>[install the web application](https://w3c.github.io/manifest/#dfn-installed-web-application)&#46;

For an <a id="link&#45;type&#45;manifest&#58;installed&#45;web&#45;application&#45;3"></a>[installed web application](https://w3c.github.io/manifest/#dfn-installed-web-application)&#44; the appropriate times to <a id="link&#45;type&#45;manifest&#58;fetch&#45;and&#45;process&#45;the&#45;linked&#45;resource&#45;2"></a>[fetch and process the linked resource](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#fetch-and-process-the-linked-resource) for this link type are&#58;

- When the <a id="link&#45;type&#45;manifest&#58;external&#45;resource&#45;link&#45;2"></a>[external resource link](#external-resource-link) is created on a <a id="link&#45;type&#45;manifest&#58;the&#45;link&#45;element&#45;2"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element that is already <a id="link&#45;type&#45;manifest&#58;browsing&#45;context&#45;connected"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected)&#46;

- When the <a id="link&#45;type&#45;manifest&#58;external&#45;resource&#45;link&#45;3"></a>[external resource link](#external-resource-link)&#39;s <a id="link&#45;type&#45;manifest&#58;the&#45;link&#45;element&#45;3"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <a id="link&#45;type&#45;manifest&#58;becomes&#45;browsing&#45;context&#45;connected"></a>[becomes browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#becomes-browsing-context-connected)&#46;

- When the <a id="link&#45;type&#45;manifest&#58;attr&#45;link&#45;href"></a><code>[href](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-href)</code> attribute of the <a id="link&#45;type&#45;manifest&#58;the&#45;link&#45;element&#45;4"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element of an <a id="link&#45;type&#45;manifest&#58;external&#45;resource&#45;link&#45;4"></a>[external resource link](#external-resource-link) that is already <a id="link&#45;type&#45;manifest&#58;browsing&#45;context&#45;connected&#45;2"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected) is changed&#46;

In any case&#44; only the first <a id="link&#45;type&#45;manifest&#58;the&#45;link&#45;element&#45;5"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element in <a id="link&#45;type&#45;manifest&#58;tree&#45;order"></a>[tree order](https://dom.spec.whatwg.org/#concept-tree-order) whose <a id="link&#45;type&#45;manifest&#58;attr&#45;link&#45;rel"></a><code>[rel](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-rel)</code> attribute contains the token <a id="link&#45;type&#45;manifest&#58;link&#45;type&#45;manifest&#45;4"></a><code>[manifest](#link-type-manifest)</code> may be used&#46;

A user agent must not <a id="link&#45;type&#45;manifest&#58;delay&#45;the&#45;load&#45;event"></a>[delay the load event](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#delay-the-load-event) for this link type&#46;

The <a id="link&#45;type&#45;manifest&#58;linked&#45;resource&#45;fetch&#45;setup&#45;steps"></a>[linked resource fetch setup steps](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#linked-resource-fetch-setup-steps) for this type of linked resource&#44; given a <a id="link&#45;type&#45;manifest&#58;the&#45;link&#45;element&#45;6"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <var>el</var> and <a id="link&#45;type&#45;manifest&#58;concept&#45;request"></a>[request](https://fetch.spec.whatwg.org/#concept-request) <var>request</var>&#44; are&#58;

1.  Let <var>navigable</var> be <var>el</var>&#39;s <a id="link&#45;type&#45;manifest&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="link&#45;type&#45;manifest&#58;node&#45;navigable"></a>[node navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable)&#46;

2.  If <var>navigable</var> is null&#44; then return false&#46;

3.  If <var>navigable</var> is not a <a id="link&#45;type&#45;manifest&#58;top&#45;level&#45;traversable"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable)&#44; then return false&#46;

4.  Set <var>request</var>&#39;s <a id="link&#45;type&#45;manifest&#58;concept&#45;request&#45;initiator"></a>[initiator](https://fetch.spec.whatwg.org/#concept-request-initiator) to &#34;<code>manifest</code>&#34;&#46;

5.  Set <var>request</var>&#39;s <a id="link&#45;type&#45;manifest&#58;concept&#45;request&#45;destination"></a>[destination](https://fetch.spec.whatwg.org/#concept-request-destination) to &#34;<code>manifest</code>&#34;&#46;

6.  Set <var>request</var>&#39;s <a id="link&#45;type&#45;manifest&#58;concept&#45;request&#45;mode"></a>[mode](https://fetch.spec.whatwg.org/#concept-request-mode) to &#34;<code>cors</code>&#34;&#46;

7.  Set <var>request</var>&#39;s <a id="link&#45;type&#45;manifest&#58;concept&#45;request&#45;credentials&#45;mode"></a>[credentials mode](https://fetch.spec.whatwg.org/#concept-request-credentials-mode) to the <a id="link&#45;type&#45;manifest&#58;cors&#45;settings&#45;attribute&#45;credentials&#45;mode"></a>[CORS settings attribute credentials mode](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#cors-settings-attribute-credentials-mode) for <var>el</var>&#39;s <a id="link&#45;type&#45;manifest&#58;attr&#45;link&#45;crossorigin"></a><code>[crossorigin](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-crossorigin)</code> content attribute&#46;

8.  Return true&#46;

To <a id="link&#45;type&#45;manifest&#58;process&#45;the&#45;linked&#45;resource"></a>[process this type of linked resource](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#process-the-linked-resource) given a <a id="link&#45;type&#45;manifest&#58;the&#45;link&#45;element&#45;7"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <var>el</var>&#44; boolean <var>success</var>&#44; <a id="link&#45;type&#45;manifest&#58;concept&#45;response"></a>[response](https://fetch.spec.whatwg.org/#concept-response) <var>response</var>&#44; and <a id="link&#45;type&#45;manifest&#58;byte&#45;sequence"></a>[byte sequence](https://infra.spec.whatwg.org/#byte-sequence) <var>bodyBytes</var>&#58;

1.  If <var>response</var>&#39;s <a id="link&#45;type&#45;manifest&#58;content&#45;type"></a>[Content&#45;Type metadata](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-type) is not a <a id="link&#45;type&#45;manifest&#58;json&#45;mime&#45;type"></a>[JSON MIME type](https://mimesniff.spec.whatwg.org/#json-mime-type)&#44; then set <var>success</var> to false&#46;

2.  If <var>success</var> is true&#58;

    1.  Let <var>document URL</var> be <var>el</var>&#39;s <a id="link&#45;type&#45;manifest&#58;node&#45;document&#45;2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="link&#45;type&#45;manifest&#58;the&#45;document&#39;s&#45;address"></a>[URL](https://dom.spec.whatwg.org/#concept-document-url)&#46;

    2.  Let <var>manifest URL</var> be <var>response</var>&#39;s <a id="link&#45;type&#45;manifest&#58;concept&#45;response&#45;url"></a>[URL](https://fetch.spec.whatwg.org/#concept-response-url)&#46;

    3.  Let <var>client</var> be <var>el</var>&#39;s <a id="link&#45;type&#45;manifest&#58;node&#45;document&#45;3"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="link&#45;type&#45;manifest&#58;relevant&#45;settings&#45;object"></a>[relevant settings object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant-settings-object)&#46;

    4.  <a id="link&#45;type&#45;manifest&#58;process&#45;the&#45;manifest"></a>[Process the manifest](https://w3c.github.io/manifest/#dfn-processing-a-manifest) given <var>document URL</var>&#44; <var>manifest URL</var>&#44; <var>bodyBytes</var>&#44; and <var>client</var>&#46; [&#91;MANIFEST&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsMANIFEST)

The <a id="link&#45;type&#45;manifest&#58;process&#45;a&#45;link&#45;header"></a>[process a link header](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#process-a-link-header) steps for this type of linked resource are to do nothing&#46;

##### <a id="link&#45;type&#45;modulepreload"></a>4&#46;6&#46;8&#46;12 Link type &#34;<strong><code>modulepreload</code></strong>&#34;[](#link-type-modulepreload)

MDN

[Link&#95;types&#47;modulepreload](https://developer.mozilla.org/en-US/docs/Web/HTML/Link_types/modulepreload)

Firefox115&#43;Safari&#63;Chrome66&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;NoInternet Explorer&#63;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

The <a id="link&#45;type&#45;modulepreload&#58;link&#45;type&#45;modulepreload"></a><code>[modulepreload](#link-type-modulepreload)</code> keyword may be used with <a id="link&#45;type&#45;modulepreload&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements&#46; This keyword creates an <a id="link&#45;type&#45;modulepreload&#58;external&#45;resource&#45;link"></a>[external resource link](#external-resource-link)&#46; This keyword is <a id="link&#45;type&#45;modulepreload&#58;body&#45;ok"></a>[body&#45;ok](#body-ok)&#46;

The <a id="link&#45;type&#45;modulepreload&#58;link&#45;type&#45;modulepreload&#45;2"></a><code>[modulepreload](#link-type-modulepreload)</code> keyword is a specialized alternative to the <a id="link&#45;type&#45;modulepreload&#58;link&#45;type&#45;preload"></a><code>[preload](#link-type-preload)</code> keyword&#44; with a processing model geared toward preloading <a id="link&#45;type&#45;modulepreload&#58;module&#45;script"></a>[module scripts](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#module-script)&#46; In particular&#44; it uses the specific fetch behavior for module scripts &#40;including&#44; e&#46;g&#46;&#44; a different interpretation of the <a id="link&#45;type&#45;modulepreload&#58;attr&#45;link&#45;crossorigin"></a><code>[crossorigin](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-crossorigin)</code> attribute&#41;&#44; and places the result into the appropriate <a id="link&#45;type&#45;modulepreload&#58;concept&#45;document&#45;module&#45;map"></a>[module map](html-language-foundations--877d6146487f--b55e73c31978.md#concept-document-module-map) for later evaluation&#46; In contrast&#44; a similar <a id="link&#45;type&#45;modulepreload&#58;external&#45;resource&#45;link&#45;2"></a>[external resource link](#external-resource-link) using the <a id="link&#45;type&#45;modulepreload&#58;link&#45;type&#45;preload&#45;2"></a><code>[preload](#link-type-preload)</code> keyword would place the result in the preload cache&#44; without affecting the document&#39;s <a id="link&#45;type&#45;modulepreload&#58;concept&#45;document&#45;module&#45;map&#45;2"></a>[module map](html-language-foundations--877d6146487f--b55e73c31978.md#concept-document-module-map)&#46;

Additionally&#44; implementations can take advantage of the fact that <a id="link&#45;type&#45;modulepreload&#58;module&#45;script&#45;2"></a>[module scripts](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#module-script) declare their dependencies in order to fetch the specified module&#39;s dependency as well&#46; This is intended as an optimization opportunity&#44; since the user agent knows that&#44; in all likelihood&#44; those dependencies will also be needed later&#46; It will not generally be observable without using technology such as service workers&#44; or monitoring on the server side&#46; Notably&#44; the appropriate <a id="link&#45;type&#45;modulepreload&#58;event&#45;load"></a><code>[load](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-load)</code> or <a id="link&#45;type&#45;modulepreload&#58;event&#45;error"></a><code>[error](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-error)</code> events will occur after the specified module is fetched&#44; and will not wait for any dependencies&#46;

A user agent must not <a id="link&#45;type&#45;modulepreload&#58;delay&#45;the&#45;load&#45;event"></a>[delay the load event](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#delay-the-load-event) for this link type&#46;

A <a id="module&#45;preload&#45;destination"></a><strong>module preload destination</strong> is &#34;<code>json</code>&#34;&#44; &#34;<code>style</code>&#34;&#44; &#34;<code>text</code>&#34;&#44; or a <a id="link&#45;type&#45;modulepreload&#58;concept&#45;script&#45;like&#45;destination"></a>[script&#45;like destination](https://fetch.spec.whatwg.org/#request-destination-script-like)&#46;

The appropriate times to <a id="link&#45;type&#45;modulepreload&#58;fetch&#45;and&#45;process&#45;the&#45;linked&#45;resource"></a>[fetch and process the linked resource](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#fetch-and-process-the-linked-resource) for such a link are&#58;

- When the <a id="link&#45;type&#45;modulepreload&#58;external&#45;resource&#45;link&#45;3"></a>[external resource link](#external-resource-link) is created on a <a id="link&#45;type&#45;modulepreload&#58;the&#45;link&#45;element&#45;2"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element that is already <a id="link&#45;type&#45;modulepreload&#58;browsing&#45;context&#45;connected"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected)&#46;

- When the <a id="link&#45;type&#45;modulepreload&#58;external&#45;resource&#45;link&#45;4"></a>[external resource link](#external-resource-link)&#39;s <a id="link&#45;type&#45;modulepreload&#58;the&#45;link&#45;element&#45;3"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <a id="link&#45;type&#45;modulepreload&#58;becomes&#45;browsing&#45;context&#45;connected"></a>[becomes browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#becomes-browsing-context-connected)&#46;

- When the <a id="link&#45;type&#45;modulepreload&#58;attr&#45;link&#45;href"></a><code>[href](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-href)</code> attribute of the <a id="link&#45;type&#45;modulepreload&#58;the&#45;link&#45;element&#45;4"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element of an <a id="link&#45;type&#45;modulepreload&#58;external&#45;resource&#45;link&#45;5"></a>[external resource link](#external-resource-link) that is already <a id="link&#45;type&#45;modulepreload&#58;browsing&#45;context&#45;connected&#45;2"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected) is changed&#46;

<strong>Note&#58;</strong>

Unlike some other link relations&#44; changing the relevant attributes &#40;such as <a id="link&#45;type&#45;modulepreload&#58;attr&#45;link&#45;as"></a><code>[as](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-as)</code>&#44; <a id="link&#45;type&#45;modulepreload&#58;attr&#45;link&#45;crossorigin&#45;2"></a><code>[crossorigin](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-crossorigin)</code>&#44; and <a id="link&#45;type&#45;modulepreload&#58;attr&#45;link&#45;referrerpolicy"></a><code>[referrerpolicy](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-referrerpolicy)</code>&#41; of such a <a id="link&#45;type&#45;modulepreload&#58;the&#45;link&#45;element&#45;5"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> does not trigger a new fetch&#46; This is because the document&#39;s <a id="link&#45;type&#45;modulepreload&#58;concept&#45;document&#45;module&#45;map&#45;3"></a>[module map](html-language-foundations--877d6146487f--b55e73c31978.md#concept-document-module-map) has already been populated by a previous fetch&#44; and so re&#45;fetching would be pointless&#46;

The <a id="link&#45;type&#45;modulepreload&#58;fetch&#45;and&#45;process&#45;the&#45;linked&#45;resource&#45;2"></a>[fetch and process the linked resource](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#fetch-and-process-the-linked-resource) algorithm for <a id="link&#45;type&#45;modulepreload&#58;link&#45;type&#45;modulepreload&#45;3"></a><code>[modulepreload](#link-type-modulepreload)</code> links&#44; given a <a id="link&#45;type&#45;modulepreload&#58;the&#45;link&#45;element&#45;6"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <var>el</var>&#44; is as follows&#58;

1.  If <var>el</var>&#39;s <a id="link&#45;type&#45;modulepreload&#58;attr&#45;link&#45;href&#45;2"></a><code>[href](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-href)</code> attribute&#39;s value is the empty string&#44; then return&#46;

2.  Let <var>destination</var> be the current state of <var>el</var>&#39;s <a id="link&#45;type&#45;modulepreload&#58;attr&#45;link&#45;as&#45;2"></a><code>[as](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-as)</code> attribute &#40;a <a id="link&#45;type&#45;modulepreload&#58;concept&#45;request&#45;destination"></a>[destination](https://fetch.spec.whatwg.org/#concept-request-destination)&#41;&#44; or &#34;<code>script</code>&#34; if it is in no state&#46;

3.  If <var>destination</var> is not a <a id="link&#45;type&#45;modulepreload&#58;module&#45;preload&#45;destination"></a>[module preload destination](#module-preload-destination)&#44; then <a id="link&#45;type&#45;modulepreload&#58;queue&#45;an&#45;element&#45;task"></a>[queue an element task](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#queue-an-element-task) on the <a id="link&#45;type&#45;modulepreload&#58;networking&#45;task&#45;source"></a>[networking task source](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#networking-task-source) given <var>el</var> to <a id="link&#45;type&#45;modulepreload&#58;concept&#45;event&#45;fire"></a>[fire an event](https://dom.spec.whatwg.org/#concept-event-fire) named <a id="link&#45;type&#45;modulepreload&#58;event&#45;error&#45;2"></a><code>[error](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-error)</code> at <var>el</var>&#44; and return&#46;

4.  Let <var>url</var> be the result of <a id="link&#45;type&#45;modulepreload&#58;encoding&#45;parsing&#45;a&#45;url"></a>[encoding&#45;parsing a URL](html-language-foundations--877d6146487f--b55e73c31978.md#encoding-parsing-a-url) given <var>el</var>&#39;s <a id="link&#45;type&#45;modulepreload&#58;attr&#45;link&#45;href&#45;3"></a><code>[href](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-href)</code> attribute&#39;s value&#44; relative to <var>el</var>&#39;s <a id="link&#45;type&#45;modulepreload&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46;

5.  If <var>url</var> is failure&#44; then return&#46;

6.  Let <var>settings object</var> be <var>el</var>&#39;s <a id="link&#45;type&#45;modulepreload&#58;node&#45;document&#45;2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="link&#45;type&#45;modulepreload&#58;relevant&#45;settings&#45;object"></a>[relevant settings object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant-settings-object)&#46;

7.  Let <var>credentials mode</var> be the <a id="link&#45;type&#45;modulepreload&#58;cors&#45;settings&#45;attribute&#45;credentials&#45;mode"></a>[CORS settings attribute credentials mode](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#cors-settings-attribute-credentials-mode) for <var>el</var>&#39;s <a id="link&#45;type&#45;modulepreload&#58;attr&#45;link&#45;crossorigin&#45;3"></a><code>[crossorigin](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-crossorigin)</code> attribute&#46;

8.  Let <var>cryptographic nonce</var> be <var>el</var>&#46;<a id="link&#45;type&#45;modulepreload&#58;cryptographicnonce"></a>[&#91;&#91;CryptographicNonce&#93;&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#cryptographicnonce)&#46;

9.  Let <var>integrity metadata</var> be the value of <var>el</var>&#39;s <a id="link&#45;type&#45;modulepreload&#58;attr&#45;link&#45;integrity"></a><code>[integrity](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-integrity)</code> attribute&#44; if it is specified&#44; or the empty string otherwise&#46;

10. If <var>el</var> does not have an <a id="link&#45;type&#45;modulepreload&#58;attr&#45;link&#45;integrity&#45;2"></a><code>[integrity](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-integrity)</code> attribute&#44; then set <var>integrity metadata</var> to the result of <a id="link&#45;type&#45;modulepreload&#58;resolving&#45;a&#45;module&#45;integrity&#45;metadata"></a>[resolving a module integrity metadata](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#resolving-a-module-integrity-metadata) with <var>url</var> and <var>settings object</var>&#46;

11. Let <var>referrer policy</var> be the current state of <var>el</var>&#39;s <a id="link&#45;type&#45;modulepreload&#58;attr&#45;link&#45;referrerpolicy&#45;2"></a><code>[referrerpolicy](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-referrerpolicy)</code> attribute&#46;

12. Let <var>fetch priority</var> be the current state of <var>el</var>&#39;s <a id="link&#45;type&#45;modulepreload&#58;attr&#45;link&#45;fetchpriority"></a><code>[fetchpriority](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-fetchpriority)</code> attribute&#46;

13. Let <var>options</var> be a <a id="link&#45;type&#45;modulepreload&#58;script&#45;fetch&#45;options"></a>[script fetch options](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#script-fetch-options) whose <a id="link&#45;type&#45;modulepreload&#58;concept&#45;script&#45;fetch&#45;options&#45;nonce"></a>[cryptographic nonce](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-script-fetch-options-nonce) is <var>cryptographic nonce</var>&#44; <a id="link&#45;type&#45;modulepreload&#58;concept&#45;script&#45;fetch&#45;options&#45;integrity"></a>[integrity metadata](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-script-fetch-options-integrity) is <var>integrity metadata</var>&#44; <a id="link&#45;type&#45;modulepreload&#58;concept&#45;script&#45;fetch&#45;options&#45;parser"></a>[parser metadata](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-script-fetch-options-parser) is &#34;<code>not&#45;parser&#45;inserted</code>&#34;&#44; <a id="link&#45;type&#45;modulepreload&#58;concept&#45;script&#45;fetch&#45;options&#45;credentials"></a>[credentials mode](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-script-fetch-options-credentials) is <var>credentials mode</var>&#44; <a id="link&#45;type&#45;modulepreload&#58;concept&#45;script&#45;fetch&#45;options&#45;referrer&#45;policy"></a>[referrer policy](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-script-fetch-options-referrer-policy) is <var>referrer policy</var>&#44; and <a id="link&#45;type&#45;modulepreload&#58;concept&#45;script&#45;fetch&#45;options&#45;fetch&#45;priority"></a>[fetch priority](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-script-fetch-options-fetch-priority) is <var>fetch priority</var>&#46;

14. <a id="link&#45;type&#45;modulepreload&#58;fetch&#45;a&#45;modulepreload&#45;module&#45;script&#45;graph"></a>[Fetch a modulepreload module script graph](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fetch-a-modulepreload-module-script-graph) given <var>url</var>&#44; <var>destination</var>&#44; <var>settings object</var>&#44; <var>options</var>&#44; and with the following steps given <var>result</var>&#58;

    1.  If <var>result</var> is null&#44; then <a id="link&#45;type&#45;modulepreload&#58;concept&#45;event&#45;fire&#45;2"></a>[fire an event](https://dom.spec.whatwg.org/#concept-event-fire) named <a id="link&#45;type&#45;modulepreload&#58;event&#45;error&#45;3"></a><code>[error](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-error)</code> at <var>el</var>&#44; and return&#46;

    2.  <a id="link&#45;type&#45;modulepreload&#58;concept&#45;event&#45;fire&#45;3"></a>[Fire an event](https://dom.spec.whatwg.org/#concept-event-fire) named <a id="link&#45;type&#45;modulepreload&#58;event&#45;load&#45;2"></a><code>[load](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-load)</code> at <var>el</var>&#46;

The <a id="link&#45;type&#45;modulepreload&#58;process&#45;a&#45;link&#45;header"></a>[process a link header](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#process-a-link-header) steps for this type of linked resource are to do nothing&#46;

<a id="example&#45;modulepreload&#45;manifest"></a>

<strong>Example&#58;</strong>

[](#example-modulepreload-manifest)

The following snippet shows the top part of an application with several modules preloaded&#58;

``` text
<!DOCTYPE html>
<html lang="en">
<title>IRCFog</title>

<link rel="modulepreload" href="app.mjs">
<link rel="modulepreload" href="helpers.mjs">
<link rel="modulepreload" href="irc.mjs">
<link rel="modulepreload" href="fog-machine.mjs">

<script type="module" src="app.mjs">
...
```

Assume that the module graph for the application is as follows&#58;

![The module graph is rooted at app&#46;mjs&#44; which depends on irc&#46;mjs and fog&#45;machine&#46;mjs&#46; In turn&#44; irc&#46;mjs depends on helpers&#46;mjs&#46;](https://html.spec.whatwg.org/images/ircfog-modules.svg)

Here we see the application developer has used <a id="link&#45;type&#45;modulepreload&#58;link&#45;type&#45;modulepreload&#45;4"></a><code>[modulepreload](#link-type-modulepreload)</code> to declare all of the modules in their module graph&#44; ensuring that the user agent initiates fetches for them all&#46; Without such preloading&#44; the user agent might need to go through multiple network roundtrips before discovering <code>helpers&#46;mjs</code>&#44; if technologies such as HTTP&#47;2 Server Push are not in play&#46; In this way&#44; <a id="link&#45;type&#45;modulepreload&#58;link&#45;type&#45;modulepreload&#45;5"></a><code>[modulepreload](#link-type-modulepreload)</code> <a id="link&#45;type&#45;modulepreload&#58;the&#45;link&#45;element&#45;7"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements can be used as a sort of &#34;manifest&#34; of the application&#39;s modules&#46;

<a id="example&#45;modulepreload&#45;dynamic&#45;import"></a>

<strong>Example&#58;</strong>

[](#example-modulepreload-dynamic-import)

The following code shows how <a id="link&#45;type&#45;modulepreload&#58;link&#45;type&#45;modulepreload&#45;6"></a><code>[modulepreload](#link-type-modulepreload)</code> links can be used in conjunction with <a id="link&#45;type&#45;modulepreload&#58;import&#40;&#41;"></a><code>[import&#40;&#41;](https://tc39.es/ecma262/#sec-import-calls)</code> to ensure network fetching is done ahead of time&#44; so that when <a id="link&#45;type&#45;modulepreload&#58;import&#40;&#41;&#45;2"></a><code>[import&#40;&#41;](https://tc39.es/ecma262/#sec-import-calls)</code> is called&#44; the module is already ready &#40;but not evaluated&#41; in the <a id="link&#45;type&#45;modulepreload&#58;module&#45;map"></a>[module map](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#module-map)&#58;

``` text
<link rel="modulepreload" href="awesome-viewer.mjs">

<button onclick="import('./awesome-viewer.mjs').then(m => m.view())">
  View awesome thing
</button>
```

##### <a id="link&#45;type&#45;nofollow"></a>4&#46;6&#46;8&#46;13 Link type &#34;<strong><code>nofollow</code></strong>&#34;[](#link-type-nofollow)

The <a id="link&#45;type&#45;nofollow&#58;link&#45;type&#45;nofollow"></a><code>[nofollow](#link-type-nofollow)</code> keyword may be used with <a id="link&#45;type&#45;nofollow&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code>&#44; <a id="link&#45;type&#45;nofollow&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code>&#44; and <a id="link&#45;type&#45;nofollow&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements&#46; This keyword does not create a <a id="link&#45;type&#45;nofollow&#58;hyperlink"></a>[hyperlink](#hyperlink)&#44; but <a id="link&#45;type&#45;nofollow&#58;hyperlink&#45;annotation"></a>[annotates](#hyperlink-annotation) any other hyperlinks created by the element &#40;the implied hyperlink&#44; if no other keywords create one&#41;&#46;

The <a id="link&#45;type&#45;nofollow&#58;link&#45;type&#45;nofollow&#45;2"></a><code>[nofollow](#link-type-nofollow)</code> keyword indicates that the link is not endorsed by the original author or publisher of the page&#44; or that the link to the referenced document was included primarily because of a commercial relationship between people affiliated with the two pages&#46;

##### <a id="link&#45;type&#45;noopener"></a>4&#46;6&#46;8&#46;14 Link type &#34;<strong><code>noopener</code></strong>&#34;[](#link-type-noopener)

<strong>✔</strong>MDN

[Link&#95;types&#47;noopener](https://developer.mozilla.org/en-US/docs/Web/HTML/Link_types/noopener)

Support in all current engines&#46;

Firefox52&#43;Safari10&#46;1&#43;Chrome49&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[Link&#95;types&#47;noopener](https://developer.mozilla.org/en-US/docs/Web/HTML/Link_types/noopener)

Support in all current engines&#46;

Firefox52&#43;Safari10&#46;1&#43;Chrome49&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

The <a id="link&#45;type&#45;noopener&#58;link&#45;type&#45;noopener"></a><code>[noopener](#link-type-noopener)</code> keyword may be used with <a id="link&#45;type&#45;noopener&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code>&#44; <a id="link&#45;type&#45;noopener&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code>&#44; and <a id="link&#45;type&#45;noopener&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements&#46; This keyword does not create a <a id="link&#45;type&#45;noopener&#58;hyperlink"></a>[hyperlink](#hyperlink)&#44; but <a id="link&#45;type&#45;noopener&#58;hyperlink&#45;annotation"></a>[annotates](#hyperlink-annotation) any other hyperlinks created by the element &#40;the implied hyperlink&#44; if no other keywords create one&#41;&#46;

The keyword indicates that any newly created <a id="link&#45;type&#45;noopener&#58;top&#45;level&#45;traversable"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable) which results from following the <a id="link&#45;type&#45;noopener&#58;hyperlink&#45;2"></a>[hyperlink](#hyperlink) will not contain an <a id="link&#45;type&#45;noopener&#58;auxiliary&#45;browsing&#45;context"></a>[auxiliary browsing context](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#auxiliary-browsing-context)&#46; E&#46;g&#46;&#44; the resulting <a id="link&#45;type&#45;noopener&#58;window"></a><code>[Window](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window)</code>&#39;s <a id="link&#45;type&#45;noopener&#58;dom&#45;opener"></a><code>[opener](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-opener)</code> getter will return null&#46;

<strong>Note&#58;</strong>

See also the [processing model](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#noopener)&#46;

<strong>Example&#58;</strong>

This typically creates a <a id="link&#45;type&#45;noopener&#58;top&#45;level&#45;traversable&#45;2"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable) with an <a id="link&#45;type&#45;noopener&#58;auxiliary&#45;browsing&#45;context&#45;2"></a>[auxiliary browsing context](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#auxiliary-browsing-context) &#40;assuming there is no existing <a id="link&#45;type&#45;noopener&#58;navigable"></a>[navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable) whose <a id="link&#45;type&#45;noopener&#58;nav&#45;target"></a>[target name](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-target) is &#34;<code>example</code>&#34;&#41;&#58;

``` text
<a href=help.html target=example>Help!</a>
```

This creates a <a id="link&#45;type&#45;noopener&#58;top&#45;level&#45;traversable&#45;3"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable) with a non&#45;<a id="link&#45;type&#45;noopener&#58;auxiliary&#45;browsing&#45;context&#45;3"></a>[auxiliary browsing context](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#auxiliary-browsing-context) &#40;assuming the same thing&#41;&#58;

``` text
<a href=help.html target=example rel=noopener>Help!</a>
```

These are equivalent and only navigate the <a id="link&#45;type&#45;noopener&#58;nav&#45;parent"></a>[parent navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-parent)&#58;

``` text
<a href=index.html target=_parent>Home</a>
```

``` text
<a href=index.html target=_parent rel=noopener>Home</a>
```

##### <a id="link&#45;type&#45;noreferrer"></a>4&#46;6&#46;8&#46;15 Link type &#34;<strong><code>noreferrer</code></strong>&#34;[](#link-type-noreferrer)

<strong>✔</strong>MDN

[Link&#95;types&#47;noreferrer](https://developer.mozilla.org/en-US/docs/Web/HTML/Link_types/noreferrer)

Support in all current engines&#46;

Firefox33&#43;Safari5&#43;Chrome16&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;13&#43;Internet Explorer🔰 11

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet1&#46;5&#43;Opera Android&#63;

[Link&#95;types&#47;noreferrer](https://developer.mozilla.org/en-US/docs/Web/HTML/Link_types/noreferrer)

Support in all current engines&#46;

Firefox33&#43;Safari5&#43;Chrome16&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;13&#43;Internet Explorer🔰 11

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet1&#46;5&#43;Opera Android&#63;

The <a id="link&#45;type&#45;noreferrer&#58;link&#45;type&#45;noreferrer"></a><code>[noreferrer](#link-type-noreferrer)</code> keyword may be used with <a id="link&#45;type&#45;noreferrer&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code>&#44; <a id="link&#45;type&#45;noreferrer&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code>&#44; and <a id="link&#45;type&#45;noreferrer&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements&#46; This keyword does not create a <a id="link&#45;type&#45;noreferrer&#58;hyperlink"></a>[hyperlink](#hyperlink)&#44; but <a id="link&#45;type&#45;noreferrer&#58;hyperlink&#45;annotation"></a>[annotates](#hyperlink-annotation) any other hyperlinks created by the element &#40;the implied hyperlink&#44; if no other keywords create one&#41;&#46;

It indicates that no referrer information is to be leaked when following the link and also implies the <a id="link&#45;type&#45;noreferrer&#58;link&#45;type&#45;noopener"></a><code>[noopener](#link-type-noopener)</code> keyword behavior under the same conditions&#46;

<strong>Note&#58;</strong>

See also the [processing model](#noreferrer-a-area-processing-model) where referrer is directly manipulated&#46;

<strong>Example&#58;</strong>

<code>&#60;a href&#61;&#34;&#46;&#46;&#46;&#34; rel&#61;&#34;noreferrer&#34; target&#61;&#34;&#95;blank&#34;&#62;</code> has the same behavior as <code>&#60;a href&#61;&#34;&#46;&#46;&#46;&#34; rel&#61;&#34;noreferrer noopener&#34; target&#61;&#34;&#95;blank&#34;&#62;</code>&#46;

##### <a id="link&#45;type&#45;opener"></a>4&#46;6&#46;8&#46;16 Link type &#34;<strong><code>opener</code></strong>&#34;[](#link-type-opener)

The <a id="link&#45;type&#45;opener&#58;link&#45;type&#45;opener"></a><code>[opener](#link-type-opener)</code> keyword may be used with <a id="link&#45;type&#45;opener&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code>&#44; <a id="link&#45;type&#45;opener&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code>&#44; and <a id="link&#45;type&#45;opener&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements&#46; This keyword does not create a <a id="link&#45;type&#45;opener&#58;hyperlink"></a>[hyperlink](#hyperlink)&#44; but <a id="link&#45;type&#45;opener&#58;hyperlink&#45;annotation"></a>[annotates](#hyperlink-annotation) any other hyperlinks created by the element &#40;the implied hyperlink&#44; if no other keywords create one&#41;&#46;

The keyword indicates that any newly created <a id="link&#45;type&#45;opener&#58;top&#45;level&#45;traversable"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable) which results from following the <a id="link&#45;type&#45;opener&#58;hyperlink&#45;2"></a>[hyperlink](#hyperlink) will contain an <a id="link&#45;type&#45;opener&#58;auxiliary&#45;browsing&#45;context"></a>[auxiliary browsing context](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#auxiliary-browsing-context)&#46;

<strong>Note&#58;</strong>

See also the [processing model](#opener-processing-model)&#46;

<strong>Example&#58;</strong>

In the following example the <a id="link&#45;type&#45;opener&#58;link&#45;type&#45;opener&#45;2"></a><code>[opener](#link-type-opener)</code> is used to allow the help page popup to navigate its opener&#44; e&#46;g&#46;&#44; in case what the user is looking for can be found elsewhere&#46; An alternative might be to use a named target&#44; rather than <code>&#95;blank</code>&#44; but this has the potential to clash with existing names&#46;

``` text
<a href="..." rel=opener target=_blank>Help!</a>
```

##### <a id="link&#45;type&#45;pingback"></a>4&#46;6&#46;8&#46;17 Link type &#34;<strong><code>pingback</code></strong>&#34;[](#link-type-pingback)

The <a id="link&#45;type&#45;pingback&#58;link&#45;type&#45;pingback"></a><code>[pingback](#link-type-pingback)</code> keyword may be used with <a id="link&#45;type&#45;pingback&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements&#46; This keyword creates an <a id="link&#45;type&#45;pingback&#58;external&#45;resource&#45;link"></a>[external resource link](#external-resource-link)&#46; This keyword is <a id="link&#45;type&#45;pingback&#58;body&#45;ok"></a>[body&#45;ok](#body-ok)&#46;

For the semantics of the <a id="link&#45;type&#45;pingback&#58;link&#45;type&#45;pingback&#45;2"></a><code>[pingback](#link-type-pingback)</code> keyword&#44; see Pingback 1&#46;0&#46; [&#91;PINGBACK&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsPINGBACK)

##### <a id="link&#45;type&#45;preconnect"></a>4&#46;6&#46;8&#46;18 Link type &#34;<strong><code>preconnect</code></strong>&#34;[](#link-type-preconnect)

<strong>✔</strong>MDN

[Link&#95;types&#47;preconnect](https://developer.mozilla.org/en-US/docs/Web/HTML/Link_types/preconnect)

Support in all current engines&#46;

Firefox39&#43;Safari11&#46;1&#43;Chrome46&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet4&#46;0&#43;Opera Android&#63;

The <a id="link&#45;type&#45;preconnect&#58;link&#45;type&#45;preconnect"></a><code>[preconnect](#link-type-preconnect)</code> keyword may be used with <a id="link&#45;type&#45;preconnect&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements&#46; This keyword creates an <a id="link&#45;type&#45;preconnect&#58;external&#45;resource&#45;link"></a>[external resource link](#external-resource-link)&#46; This keyword is <a id="link&#45;type&#45;preconnect&#58;body&#45;ok"></a>[body&#45;ok](#body-ok)&#46;

The <a id="link&#45;type&#45;preconnect&#58;link&#45;type&#45;preconnect&#45;2"></a><code>[preconnect](#link-type-preconnect)</code> keyword indicates that preemptively initiating a connection to the <a id="link&#45;type&#45;preconnect&#58;concept&#45;origin"></a>[origin](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin) of the specified resource is likely to be beneficial&#44; as it is highly likely that the user will require resources located at that <a id="link&#45;type&#45;preconnect&#58;concept&#45;origin&#45;2"></a>[origin](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin)&#44; and the user experience would be improved by preempting the latency costs associated with establishing the connection&#46;

There is no default type for resources given by the <a id="link&#45;type&#45;preconnect&#58;link&#45;type&#45;preconnect&#45;3"></a><code>[preconnect](#link-type-preconnect)</code> keyword&#46;

A user agent must not <a id="link&#45;type&#45;preconnect&#58;delay&#45;the&#45;load&#45;event"></a>[delay the load event](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#delay-the-load-event) for this link type&#46;

The appropriate times to <a id="link&#45;type&#45;preconnect&#58;fetch&#45;and&#45;process&#45;the&#45;linked&#45;resource"></a>[fetch and process](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#fetch-and-process-the-linked-resource) this type of link are&#58;

- When the <a id="link&#45;type&#45;preconnect&#58;external&#45;resource&#45;link&#45;2"></a>[external resource link](#external-resource-link) is created on a <a id="link&#45;type&#45;preconnect&#58;the&#45;link&#45;element&#45;2"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element that is already <a id="link&#45;type&#45;preconnect&#58;browsing&#45;context&#45;connected"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected)&#46;

- When the <a id="link&#45;type&#45;preconnect&#58;external&#45;resource&#45;link&#45;3"></a>[external resource link](#external-resource-link)&#39;s <a id="link&#45;type&#45;preconnect&#58;the&#45;link&#45;element&#45;3"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <a id="link&#45;type&#45;preconnect&#58;becomes&#45;browsing&#45;context&#45;connected"></a>[becomes browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#becomes-browsing-context-connected)&#46;

- When the <a id="link&#45;type&#45;preconnect&#58;attr&#45;link&#45;href"></a><code>[href](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-href)</code> attribute of the <a id="link&#45;type&#45;preconnect&#58;the&#45;link&#45;element&#45;4"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element of an <a id="link&#45;type&#45;preconnect&#58;external&#45;resource&#45;link&#45;4"></a>[external resource link](#external-resource-link) that is already <a id="link&#45;type&#45;preconnect&#58;browsing&#45;context&#45;connected&#45;2"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected) is changed&#46;

- When the <a id="link&#45;type&#45;preconnect&#58;attr&#45;link&#45;crossorigin"></a><code>[crossorigin](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-crossorigin)</code> attribute of the <a id="link&#45;type&#45;preconnect&#58;the&#45;link&#45;element&#45;5"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element of an <a id="link&#45;type&#45;preconnect&#58;external&#45;resource&#45;link&#45;5"></a>[external resource link](#external-resource-link) that is already <a id="link&#45;type&#45;preconnect&#58;browsing&#45;context&#45;connected&#45;3"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected) is set&#44; changed&#44; or removed&#46;

The <a id="link&#45;type&#45;preconnect&#58;fetch&#45;and&#45;process&#45;the&#45;linked&#45;resource&#45;2"></a>[fetch and process the linked resource](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#fetch-and-process-the-linked-resource) steps for this type of linked resource&#44; given a <a id="link&#45;type&#45;preconnect&#58;the&#45;link&#45;element&#45;6"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <var>el</var>&#44; are to <a id="link&#45;type&#45;preconnect&#58;create&#45;link&#45;options&#45;from&#45;element"></a>[create link options](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#create-link-options-from-element) from <var>el</var> and to <a id="link&#45;type&#45;preconnect&#58;preconnect"></a>[preconnect](#preconnect) given the result&#46;

The <a id="link&#45;type&#45;preconnect&#58;process&#45;a&#45;link&#45;header"></a>[process a link header](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#process-a-link-header) step for this type of linked resource given a <a id="link&#45;type&#45;preconnect&#58;link&#45;processing&#45;options"></a>[link processing options](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-processing-options) <var>options</var> are to <a id="link&#45;type&#45;preconnect&#58;preconnect&#45;2"></a>[preconnect](#preconnect) given <var>options</var>&#46;

To <a id="preconnect"></a><strong>preconnect</strong> given a <a id="link&#45;type&#45;preconnect&#58;link&#45;processing&#45;options&#45;2"></a>[link processing options](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-processing-options) <var>options</var>&#58;

1.  If <var>options</var>&#39;s <a id="link&#45;type&#45;preconnect&#58;link&#45;options&#45;href"></a>[href](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-href) is an empty string&#44; return&#46;

2.  Let <var>url</var> be the result of <a id="link&#45;type&#45;preconnect&#58;encoding&#45;parsing&#45;a&#45;url"></a>[encoding&#45;parsing a URL](html-language-foundations--877d6146487f--b55e73c31978.md#encoding-parsing-a-url) given <var>options</var>&#39;s <a id="link&#45;type&#45;preconnect&#58;link&#45;options&#45;href&#45;2"></a>[href](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-href)&#44; relative to <var>options</var>&#39;s <a id="link&#45;type&#45;preconnect&#58;link&#45;options&#45;base&#45;url"></a>[base URL](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-base-url)&#46;

    Passing the base URL instead of a document or environment is tracked by [issue &#35;9715](https://github.com/whatwg/html/issues/9715)&#46;

3.  If <var>url</var> is failure&#44; then return&#46;

4.  If <var>url</var>&#39;s <a id="link&#45;type&#45;preconnect&#58;concept&#45;url&#45;scheme"></a>[scheme](https://url.spec.whatwg.org/#concept-url-scheme) is not an <a id="link&#45;type&#45;preconnect&#58;http&#40;s&#41;&#45;scheme"></a>[HTTP&#40;S&#41; scheme](https://fetch.spec.whatwg.org/#http-scheme)&#44; then return&#46;

5.  Let <var>partitionKey</var> be the result of <a id="link&#45;type&#45;preconnect&#58;determine&#45;the&#45;network&#45;partition&#45;key"></a>[determining the network partition key](https://fetch.spec.whatwg.org/#determine-the-network-partition-key) given <var>options</var>&#39;s <a id="link&#45;type&#45;preconnect&#58;link&#45;options&#45;environment"></a>[environment](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-environment)&#46;

6.  Let <var>useCredentials</var> be true&#46;

7.  If <var>options</var>&#39;s <a id="link&#45;type&#45;preconnect&#58;link&#45;options&#45;crossorigin"></a>[crossorigin](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-crossorigin) is <a id="link&#45;type&#45;preconnect&#58;attr&#45;crossorigin&#45;anonymous"></a>[Anonymous](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-crossorigin-anonymous) and <var>options</var>&#39;s <a id="link&#45;type&#45;preconnect&#58;link&#45;options&#45;origin"></a>[origin](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-origin) does not have the <a id="link&#45;type&#45;preconnect&#58;same&#45;origin"></a>[same origin](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#same-origin) as <var>url</var>&#39;s <a id="link&#45;type&#45;preconnect&#58;concept&#45;url&#45;origin"></a>[origin](https://url.spec.whatwg.org/#concept-url-origin)&#44; then set <var>useCredentials</var> to false&#46;

8.  The user agent should <a id="link&#45;type&#45;preconnect&#58;obtain&#45;a&#45;connection"></a>[obtain a connection](https://fetch.spec.whatwg.org/#concept-connection-obtain) given <var>partitionKey</var>&#44; <var>url</var>&#39;s <a id="link&#45;type&#45;preconnect&#58;concept&#45;url&#45;origin&#45;2"></a>[origin](https://url.spec.whatwg.org/#concept-url-origin)&#44; and <var>useCredentials</var>&#46;

    <strong>Note&#58;</strong>

    This connection is obtained but not used directly&#46; It will remain in the <a id="link&#45;type&#45;preconnect&#58;connection&#45;pool"></a>[connection pool](https://fetch.spec.whatwg.org/#concept-connection-pool) for subsequent use&#46;

    The user agent should attempt to initiate a preconnect and perform the full connection handshake &#40;DNS&#43;TCP for HTTP&#44; and DNS&#43;TCP&#43;TLS for HTTPS origins&#41; whenever possible&#44; but is allowed to elect to perform a partial handshake &#40;DNS only for HTTP&#44; and DNS or DNS&#43;TCP for HTTPS origins&#41;&#44; or skip it entirely&#44; due to resource constraints or other reasons&#46;

    The optimal number of connections per origin is dependent on the negotiated protocol&#44; users current connectivity profile&#44; available device resources&#44; global connection limits&#44; and other context specific variables&#46; As a result&#44; the decision for how many connections should be opened is deferred to the user agent&#46;

##### <a id="link&#45;type&#45;prefetch"></a>4&#46;6&#46;8&#46;19 Link type &#34;<strong><code>prefetch</code></strong>&#34;[](#link-type-prefetch)

MDN

[Link&#95;types&#47;prefetch](https://developer.mozilla.org/en-US/docs/Web/HTML/Link_types/prefetch)

Firefox2&#43;SafariNoChrome8&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer11

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet1&#46;5&#43;Opera Android&#63;

The <a id="link&#45;type&#45;prefetch&#58;link&#45;type&#45;prefetch"></a><code>[prefetch](#link-type-prefetch)</code> keyword may be used with <a id="link&#45;type&#45;prefetch&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements&#46; This keyword creates an <a id="link&#45;type&#45;prefetch&#58;external&#45;resource&#45;link"></a>[external resource link](#external-resource-link)&#46; This keyword is <a id="link&#45;type&#45;prefetch&#58;body&#45;ok"></a>[body&#45;ok](#body-ok)&#46;

The <a id="link&#45;type&#45;prefetch&#58;link&#45;type&#45;prefetch&#45;2"></a><code>[prefetch](#link-type-prefetch)</code> keyword indicates that preemptively <a id="link&#45;type&#45;prefetch&#58;concept&#45;fetch"></a>[fetching](https://fetch.spec.whatwg.org/#concept-fetch) and caching the specified resource or same&#45;site document is likely to be beneficial&#44; as it is highly likely that the user will require this resource for future navigations&#46;

There is no default type for resources given by the <a id="link&#45;type&#45;prefetch&#58;link&#45;type&#45;prefetch&#45;3"></a><code>[prefetch](#link-type-prefetch)</code> keyword&#46;

The appropriate times to <a id="link&#45;type&#45;prefetch&#58;fetch&#45;and&#45;process&#45;the&#45;linked&#45;resource"></a>[fetch and process](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#fetch-and-process-the-linked-resource) this type of link are&#58;

- When the <a id="link&#45;type&#45;prefetch&#58;external&#45;resource&#45;link&#45;2"></a>[external resource link](#external-resource-link) is created on a <a id="link&#45;type&#45;prefetch&#58;the&#45;link&#45;element&#45;2"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element that is already <a id="link&#45;type&#45;prefetch&#58;browsing&#45;context&#45;connected"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected)&#46;

- When the <a id="link&#45;type&#45;prefetch&#58;external&#45;resource&#45;link&#45;3"></a>[external resource link](#external-resource-link)&#39;s <a id="link&#45;type&#45;prefetch&#58;the&#45;link&#45;element&#45;3"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <a id="link&#45;type&#45;prefetch&#58;becomes&#45;browsing&#45;context&#45;connected"></a>[becomes browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#becomes-browsing-context-connected)&#46;

- When the <a id="link&#45;type&#45;prefetch&#58;attr&#45;link&#45;href"></a><code>[href](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-href)</code> attribute of the <a id="link&#45;type&#45;prefetch&#58;the&#45;link&#45;element&#45;4"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element of an <a id="link&#45;type&#45;prefetch&#58;external&#45;resource&#45;link&#45;4"></a>[external resource link](#external-resource-link) that is already <a id="link&#45;type&#45;prefetch&#58;browsing&#45;context&#45;connected&#45;2"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected) is changed&#46;

- When the <a id="link&#45;type&#45;prefetch&#58;attr&#45;link&#45;crossorigin"></a><code>[crossorigin](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-crossorigin)</code> attribute of the <a id="link&#45;type&#45;prefetch&#58;the&#45;link&#45;element&#45;5"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element of an <a id="link&#45;type&#45;prefetch&#58;external&#45;resource&#45;link&#45;5"></a>[external resource link](#external-resource-link) that is already <a id="link&#45;type&#45;prefetch&#58;browsing&#45;context&#45;connected&#45;3"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected) is set&#44; changed&#44; or removed&#46;

The <a id="link&#45;type&#45;prefetch&#58;fetch&#45;and&#45;process&#45;the&#45;linked&#45;resource&#45;2"></a>[fetch and process the linked resource](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#fetch-and-process-the-linked-resource) algorithm for <a id="link&#45;type&#45;prefetch&#58;link&#45;type&#45;prefetch&#45;4"></a><code>[prefetch](#link-type-prefetch)</code> links&#44; given a <a id="link&#45;type&#45;prefetch&#58;the&#45;link&#45;element&#45;6"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <var>el</var>&#44; is as follows&#58;

1.  If <var>el</var>&#39;s <a id="link&#45;type&#45;prefetch&#58;attr&#45;link&#45;href&#45;2"></a><code>[href](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-href)</code> attribute&#39;s value is the empty string&#44; then return&#46;

2.  Let <var>options</var> be the result of <a id="link&#45;type&#45;prefetch&#58;create&#45;link&#45;options&#45;from&#45;element"></a>[creating link options](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#create-link-options-from-element) from <var>el</var>&#46;

3.  Let <var>request</var> be the result of <a id="link&#45;type&#45;prefetch&#58;create&#45;a&#45;link&#45;request"></a>[creating a link request](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#create-a-link-request) given <var>options</var>&#46;

4.  If <var>request</var> is null&#44; then return&#46;

5.  Set <var>request</var>&#39;s <a id="link&#45;type&#45;prefetch&#58;concept&#45;request&#45;initiator"></a>[initiator](https://fetch.spec.whatwg.org/#concept-request-initiator) to &#34;<code>prefetch</code>&#34;&#46;

6.  Let <var>processPrefetchResponse</var> be the following steps given a <a id="link&#45;type&#45;prefetch&#58;concept&#45;response"></a>[response](https://fetch.spec.whatwg.org/#concept-response) <var>response</var> and null&#44; failure&#44; or a <a id="link&#45;type&#45;prefetch&#58;byte&#45;sequence"></a>[byte sequence](https://infra.spec.whatwg.org/#byte-sequence) <var>bytesOrNull</var>&#58;

    1.  If <var>response</var> is a <a id="link&#45;type&#45;prefetch&#58;network&#45;error"></a>[network error](https://fetch.spec.whatwg.org/#concept-network-error)&#44; <a id="link&#45;type&#45;prefetch&#58;concept&#45;event&#45;fire"></a>[fire an event](https://dom.spec.whatwg.org/#concept-event-fire) named <a id="link&#45;type&#45;prefetch&#58;event&#45;error"></a><code>[error](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-error)</code> at <var>el</var>&#46;

    2.  Otherwise&#44; <a id="link&#45;type&#45;prefetch&#58;concept&#45;event&#45;fire&#45;2"></a>[fire an event](https://dom.spec.whatwg.org/#concept-event-fire) named <a id="link&#45;type&#45;prefetch&#58;event&#45;load"></a><code>[load](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-load)</code> at <var>el</var>&#46;

7.  The user agent should <a id="link&#45;type&#45;prefetch&#58;concept&#45;fetch&#45;2"></a>[fetch](https://fetch.spec.whatwg.org/#concept-fetch) <var>request</var>&#44; with <a id="link&#45;type&#45;prefetch&#58;processresponseconsumebody"></a><em>[processResponseConsumeBody](https://fetch.spec.whatwg.org/#process-response-end-of-body)</em> set to <var>processPrefetchResponse</var>&#46; User agents may delay the fetching of <var>request</var> to prioritize other requests that are necessary for the current document&#46;

The <a id="link&#45;type&#45;prefetch&#58;process&#45;a&#45;link&#45;header"></a>[process a link header](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#process-a-link-header) steps for this type of linked resource are to do nothing&#46;

##### <a id="link&#45;type&#45;preload"></a>4&#46;6&#46;8&#46;20 Link type &#34;<strong><code>preload</code></strong>&#34;[](#link-type-preload)

<strong>⚠</strong>MDN

[Link&#95;types&#47;preload](https://developer.mozilla.org/en-US/docs/Web/HTML/Link_types/preload)

Support in one engine only&#46;

Firefox85&#43;Safari&#63;Chrome🔰 50&#43;

------------------------------------------------------------------------

Opera37&#43;Edge🔰 79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;NoInternet Explorer&#63;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android50&#43;Samsung Internet5&#46;0&#43;Opera Android&#63;

The <a id="link&#45;type&#45;preload&#58;link&#45;type&#45;preload"></a><code>[preload](#link-type-preload)</code> keyword may be used with <a id="link&#45;type&#45;preload&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements&#46; This keyword creates an <a id="link&#45;type&#45;preload&#58;external&#45;resource&#45;link"></a>[external resource link](#external-resource-link)&#46; This keyword is <a id="link&#45;type&#45;preload&#58;body&#45;ok"></a>[body&#45;ok](#body-ok)&#46;

The <a id="link&#45;type&#45;preload&#58;link&#45;type&#45;preload&#45;2"></a><code>[preload](#link-type-preload)</code> keyword indicates that the user agent will preemptively <a id="link&#45;type&#45;preload&#58;concept&#45;fetch"></a>[fetch](https://fetch.spec.whatwg.org/#concept-fetch) and cache the specified resource according to the <a id="link&#45;type&#45;preload&#58;preload&#45;destination"></a>[preload destination](#preload-destination) given by the <a id="link&#45;type&#45;preload&#58;attr&#45;link&#45;as"></a><code>[as](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-as)</code> attribute&#44; and the <a id="link&#45;type&#45;preload&#58;concept&#45;request&#45;priority"></a>[priority](https://fetch.spec.whatwg.org/#request-priority) given by the <a id="link&#45;type&#45;preload&#58;attr&#45;link&#45;fetchpriority"></a><code>[fetchpriority](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-fetchpriority)</code> attribute&#44; as it is highly likely that the user will require this resource for the current navigation&#46;

<strong>Note&#58;</strong>

User&#45;agents might perform additional operations when a resource is loaded&#44; such as preemptively <a id="link&#45;type&#45;preload&#58;dom&#45;img&#45;decode"></a>[decoding images](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#dom-img-decode) or <a id="link&#45;type&#45;preload&#58;create&#45;a&#45;css&#45;style&#45;sheet"></a>[creating style sheets](https://drafts.csswg.org/cssom/#create-a-css-style-sheet)&#46; However&#44; these additional operations cannot have observable effects&#46;

There is no default type for resources given by the <a id="link&#45;type&#45;preload&#58;link&#45;type&#45;preload&#45;3"></a><code>[preload](#link-type-preload)</code> keyword&#46;

A user agent must not <a id="link&#45;type&#45;preload&#58;delay&#45;the&#45;load&#45;event"></a>[delay the load event](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#delay-the-load-event) for this link type&#46;

The appropriate times to <a id="link&#45;type&#45;preload&#58;fetch&#45;and&#45;process&#45;the&#45;linked&#45;resource"></a>[fetch and process the linked resource](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#fetch-and-process-the-linked-resource) for such a link are&#58;

- When the <a id="link&#45;type&#45;preload&#58;external&#45;resource&#45;link&#45;2"></a>[external resource link](#external-resource-link) is created on a <a id="link&#45;type&#45;preload&#58;the&#45;link&#45;element&#45;2"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element that is already <a id="link&#45;type&#45;preload&#58;browsing&#45;context&#45;connected"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected)&#46;

- When the <a id="link&#45;type&#45;preload&#58;external&#45;resource&#45;link&#45;3"></a>[external resource link](#external-resource-link)&#39;s <a id="link&#45;type&#45;preload&#58;the&#45;link&#45;element&#45;3"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <a id="link&#45;type&#45;preload&#58;becomes&#45;browsing&#45;context&#45;connected"></a>[becomes browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#becomes-browsing-context-connected)&#46;

- When the <a id="link&#45;type&#45;preload&#58;attr&#45;link&#45;href"></a><code>[href](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-href)</code> attribute of the <a id="link&#45;type&#45;preload&#58;the&#45;link&#45;element&#45;4"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element of an <a id="link&#45;type&#45;preload&#58;external&#45;resource&#45;link&#45;4"></a>[external resource link](#external-resource-link) that is already <a id="link&#45;type&#45;preload&#58;browsing&#45;context&#45;connected&#45;2"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected) is changed&#46;

- When the <a id="link&#45;type&#45;preload&#58;attr&#45;link&#45;as&#45;2"></a><code>[as](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-as)</code> attribute of the <a id="link&#45;type&#45;preload&#58;the&#45;link&#45;element&#45;5"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element of an <a id="link&#45;type&#45;preload&#58;external&#45;resource&#45;link&#45;5"></a>[external resource link](#external-resource-link) that is already <a id="link&#45;type&#45;preload&#58;browsing&#45;context&#45;connected&#45;3"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected) is changed&#46;

- When the <a id="link&#45;type&#45;preload&#58;attr&#45;link&#45;type"></a><code>[type](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-type)</code> attribute of the <a id="link&#45;type&#45;preload&#58;the&#45;link&#45;element&#45;6"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element of an <a id="link&#45;type&#45;preload&#58;external&#45;resource&#45;link&#45;6"></a>[external resource link](#external-resource-link) that is already <a id="link&#45;type&#45;preload&#58;browsing&#45;context&#45;connected&#45;4"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected)&#44; but was previously not obtained due to the <a id="link&#45;type&#45;preload&#58;attr&#45;link&#45;type&#45;2"></a><code>[type](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-type)</code> attribute specifying an unsupported type for the request <a id="link&#45;type&#45;preload&#58;concept&#45;request&#45;destination"></a>[destination](https://fetch.spec.whatwg.org/#concept-request-destination)&#44; is set&#44; removed&#44; or changed&#46;

- When the <a id="link&#45;type&#45;preload&#58;attr&#45;link&#45;media"></a><code>[media](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-media)</code> attribute of the <a id="link&#45;type&#45;preload&#58;the&#45;link&#45;element&#45;7"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element of an <a id="link&#45;type&#45;preload&#58;external&#45;resource&#45;link&#45;7"></a>[external resource link](#external-resource-link) that is already <a id="link&#45;type&#45;preload&#58;browsing&#45;context&#45;connected&#45;5"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected)&#44; but was previously not obtained due to the <a id="link&#45;type&#45;preload&#58;attr&#45;link&#45;media&#45;2"></a><code>[media](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-media)</code> attribute not <a id="link&#45;type&#45;preload&#58;matches&#45;the&#45;environment"></a>[matching the environment](html-language-foundations--877d6146487f--b55e73c31978.md#matches-the-environment)&#44; is changed or removed&#46;

A <a id="link&#45;type&#45;preload&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> has a <a id="map&#45;of&#45;preloaded&#45;resources"></a><strong>map of preloaded resources</strong>&#44; which is an <a id="link&#45;type&#45;preload&#58;ordered&#45;map"></a>[ordered map](https://infra.spec.whatwg.org/#ordered-map)&#44; initially empty&#46;

A <a id="preload&#45;key"></a><strong>preload key</strong> is a <a id="link&#45;type&#45;preload&#58;struct"></a>[struct](https://infra.spec.whatwg.org/#struct)&#46; It has the following <a id="link&#45;type&#45;preload&#58;struct&#45;item"></a>[items](https://infra.spec.whatwg.org/#struct-item)&#58;

<strong><a id="preload&#45;url"></a><strong>URL</strong></strong>

A <a id="link&#45;type&#45;preload&#58;url"></a>[URL](https://url.spec.whatwg.org/#concept-url)

<strong><a id="preload&#45;key&#45;destination"></a><strong>destination</strong></strong>

A <a id="link&#45;type&#45;preload&#58;preload&#45;destination&#45;2"></a>[preload destination](#preload-destination)

<strong><a id="preload&#45;mode"></a><strong>mode</strong></strong>

A <a id="link&#45;type&#45;preload&#58;concept&#45;request&#45;mode"></a>[request mode](https://fetch.spec.whatwg.org/#concept-request-mode)&#44; either &#34;<code>same&#45;origin</code>&#34;&#44; &#34;<code>cors</code>&#34;&#44; or &#34;<code>no&#45;cors</code>&#34;

<strong><a id="preload&#45;credentials&#45;mode"></a><strong>credentials mode</strong></strong>

A <a id="link&#45;type&#45;preload&#58;concept&#45;request&#45;credentials&#45;mode"></a>[credentials mode](https://fetch.spec.whatwg.org/#concept-request-credentials-mode)

A <a id="link&#45;type&#45;preload&#58;preload&#45;key"></a>[preload key](#preload-key)&#39;s <a id="link&#45;type&#45;preload&#58;equality"></a>[equality](https://infra.spec.whatwg.org/#equality) is <a id="link&#45;type&#45;preload&#58;itemwise&#45;equality"></a>[itemwise equality](https://infra.spec.whatwg.org/#struct-itemwise-equality)&#46;

A <a id="preload&#45;entry"></a><strong>preload entry</strong> is a <a id="link&#45;type&#45;preload&#58;struct&#45;2"></a>[struct](https://infra.spec.whatwg.org/#struct)&#46; It has the following <a id="link&#45;type&#45;preload&#58;struct&#45;item&#45;2"></a>[items](https://infra.spec.whatwg.org/#struct-item)&#58;

<strong><a id="preload&#45;integrity&#45;metadata"></a><strong>integrity metadata</strong></strong>

A string

<strong><a id="preload&#45;response"></a><strong>response</strong></strong>

Null or a <a id="link&#45;type&#45;preload&#58;concept&#45;response"></a>[response](https://fetch.spec.whatwg.org/#concept-response)

<strong><a id="preload&#45;on&#45;response&#45;available"></a><strong>on response available</strong></strong>

Null&#44; or an algorithm accepting a <a id="link&#45;type&#45;preload&#58;concept&#45;response&#45;2"></a>[response](https://fetch.spec.whatwg.org/#concept-response) or null

To <a id="consume&#45;a&#45;preloaded&#45;resource"></a><strong>consume a preloaded resource</strong> for <a id="link&#45;type&#45;preload&#58;window"></a><code>[Window](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window)</code> <var>window</var>&#44; given a <a id="link&#45;type&#45;preload&#58;url&#45;2"></a>[URL](https://url.spec.whatwg.org/#concept-url) <var>url</var>&#44; a string <var>destination</var>&#44; a string <var>mode</var>&#44; a string <var>credentialsMode</var>&#44; a string <var>integrityMetadata</var>&#44; and <var>onResponseAvailable</var>&#44; which is an algorithm accepting a <a id="link&#45;type&#45;preload&#58;concept&#45;response&#45;3"></a>[response](https://fetch.spec.whatwg.org/#concept-response)&#58;

1.  Let <var>key</var> be a <a id="link&#45;type&#45;preload&#58;preload&#45;key&#45;2"></a>[preload key](#preload-key) whose <a id="link&#45;type&#45;preload&#58;preload&#45;url"></a>[URL](#preload-url) is <var>url</var>&#44; <a id="link&#45;type&#45;preload&#58;preload&#45;destination&#45;3"></a>[destination](#preload-destination) is <var>destination</var>&#44; <a id="link&#45;type&#45;preload&#58;preload&#45;mode"></a>[mode](#preload-mode) is <var>mode</var>&#44; and <a id="link&#45;type&#45;preload&#58;preload&#45;credentials&#45;mode"></a>[credentials mode](#preload-credentials-mode) is <var>credentialsMode</var>&#46;

2.  Let <var>preloads</var> be <var>window</var>&#39;s <a id="link&#45;type&#45;preload&#58;concept&#45;document&#45;window"></a>[associated <code>Document</code>](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-window)&#39;s <a id="link&#45;type&#45;preload&#58;map&#45;of&#45;preloaded&#45;resources"></a>[map of preloaded resources](#map-of-preloaded-resources)&#46;

3.  If <var>key</var> does not <a id="link&#45;type&#45;preload&#58;map&#45;exists"></a>[exist](https://infra.spec.whatwg.org/#map-exists) in <var>preloads</var>&#44; then return false&#46;

4.  Let <var>entry</var> be <var>preloads</var>&#91;<var>key</var>&#93;&#46;

5.  Let <var>consumerIntegrityMetadata</var> be the result of <a id="link&#45;type&#45;preload&#58;parse&#45;integrity&#45;metadata"></a>[parsing](https://w3c.github.io/webappsec-subresource-integrity/#parse-metadata) <var>integrityMetadata</var>&#46;

6.  Let <var>preloadIntegrityMetadata</var> be the result of <a id="link&#45;type&#45;preload&#58;parse&#45;integrity&#45;metadata&#45;2"></a>[parsing](https://w3c.github.io/webappsec-subresource-integrity/#parse-metadata) <var>entry</var>&#39;s <a id="link&#45;type&#45;preload&#58;preload&#45;integrity&#45;metadata"></a>[integrity metadata](#preload-integrity-metadata)&#46;

7.  If none of the following conditions apply&#58;

    - <var>consumerIntegrityMetadata</var> is <code>no metadata</code>&#59;

    - <var>consumerIntegrityMetadata</var> is equal to <var>preloadIntegrityMetadata</var>&#59; or

      This comparison would ignore unknown integrity options&#46; See [issue &#35;116&#46;](https://github.com/w3c/webappsec-subresource-integrity/issues/116)

    then return false&#46;

    <strong>Note&#58;</strong>

    A mismatch in integrity metadata between the preload and the consumer&#44; even if both match the data&#44; would lead to an additional fetch from the network&#46;

    <strong>Note&#58;</strong>

    It is important that <a id="link&#45;type&#45;preload&#58;network&#45;error"></a>[network errors](https://fetch.spec.whatwg.org/#concept-network-error) are added to the preload cache so that if a preload request results in an error&#44; the erroneous response isn&#39;t re&#45;requested from the network later&#46; This also has security implications&#59; consider the case where a developer specifies subresource integrity metadata on a preload request&#44; but not the following resource request&#46; If the preload request fails subresource integrity verification and is discarded&#44; the resource request will fetch and consume a potentially&#45;malicious response from the network without verifying its integrity&#46; [&#91;SRI&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsSRI)

8.  <a id="link&#45;type&#45;preload&#58;map&#45;remove"></a>[Remove](https://infra.spec.whatwg.org/#map-remove) <var>preloads</var>&#91;<var>key</var>&#93;&#46;

9.  If <var>entry</var>&#39;s <a id="link&#45;type&#45;preload&#58;preload&#45;response"></a>[response](#preload-response) is null&#44; then set <var>entry</var>&#39;s <a id="link&#45;type&#45;preload&#58;preload&#45;on&#45;response&#45;available"></a>[on response available](#preload-on-response-available) to <var>onResponseAvailable</var>&#46;

10. Otherwise&#44; call <var>onResponseAvailable</var> with <var>entry</var>&#39;s <a id="link&#45;type&#45;preload&#58;preload&#45;response&#45;2"></a>[response](#preload-response)&#46;

11. Return true&#46;

For the purposes of this section&#44; a string <var>type</var> <a id="match&#45;preload&#45;type"></a><strong>matches</strong> a <a id="link&#45;type&#45;preload&#58;preload&#45;destination&#45;4"></a>[preload destination](#preload-destination) <var>destination</var> if the following algorithm returns true&#58;

1.  If <var>type</var> is an empty string&#44; then return true&#46;

2.  If <var>destination</var> is &#34;<code>fetch</code>&#34;&#44; then return true&#46;

3.  Let <var>mimeTypeRecord</var> be the result of <a id="link&#45;type&#45;preload&#58;parse&#45;a&#45;mime&#45;type"></a>[parsing](https://mimesniff.spec.whatwg.org/#parse-a-mime-type) <var>type</var>&#46;

4.  If <var>mimeTypeRecord</var> is failure&#44; then return false&#46;

5.  If <var>mimeTypeRecord</var> is not <a id="link&#45;type&#45;preload&#58;is&#45;mime&#45;type&#45;supported&#45;by&#45;the&#45;user&#45;agent"></a>[supported by the user agent](https://mimesniff.spec.whatwg.org/#supported-by-the-user-agent)&#44; then return false&#46;

6.  If any of the following are true&#58;

    - <var>destination</var> is &#34;<code>script</code>&#34; and <var>mimeTypeRecord</var> is a <a id="link&#45;type&#45;preload&#58;javascript&#45;mime&#45;type"></a>[JavaScript MIME type](https://mimesniff.spec.whatwg.org/#javascript-mime-type)&#59;

    - <var>destination</var> is &#34;<code>image</code>&#34; and <var>mimeTypeRecord</var> is an <a id="link&#45;type&#45;preload&#58;image&#45;mime&#45;type"></a>[image MIME type](https://mimesniff.spec.whatwg.org/#image-mime-type)&#59;

    - <var>destination</var> is &#34;<code>font</code>&#34; and <var>mimeTypeRecord</var> is a <a id="link&#45;type&#45;preload&#58;font&#45;mime&#45;type"></a>[font MIME type](https://mimesniff.spec.whatwg.org/#font-mime-type)&#59;

    - <var>destination</var> is &#34;<code>style</code>&#34; and <var>mimeTypeRecord</var>&#39;s <a id="link&#45;type&#45;preload&#58;mime&#45;type&#45;essence"></a>[essence](https://mimesniff.spec.whatwg.org/#mime-type-essence) is <a id="link&#45;type&#45;preload&#58;text&#47;css"></a><code>[text&#47;css](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#text/css)</code>&#59; or

    - <var>destination</var> is &#34;<code>track</code>&#34; and <var>mimeTypeRecord</var>&#39;s <a id="link&#45;type&#45;preload&#58;mime&#45;type&#45;essence&#45;2"></a>[essence](https://mimesniff.spec.whatwg.org/#mime-type-essence) is <a id="link&#45;type&#45;preload&#58;text&#47;vtt"></a><code>[text&#47;vtt](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#text/vtt)</code>&#44;

    then return true&#46;

7.  Return false&#46;

To <a id="create&#45;a&#45;preload&#45;key"></a><strong>create a preload key</strong> for a <a id="link&#45;type&#45;preload&#58;concept&#45;request"></a>[request](https://fetch.spec.whatwg.org/#concept-request) <var>request</var>&#44; return a new <a id="link&#45;type&#45;preload&#58;preload&#45;key&#45;3"></a>[preload key](#preload-key) whose <a id="link&#45;type&#45;preload&#58;preload&#45;url&#45;2"></a>[URL](#preload-url) is <var>request</var>&#39;s <a id="link&#45;type&#45;preload&#58;concept&#45;request&#45;url"></a>[URL](https://fetch.spec.whatwg.org/#concept-request-url)&#44; <a id="link&#45;type&#45;preload&#58;preload&#45;destination&#45;5"></a>[destination](#preload-destination) is <var>request</var>&#39;s <a id="link&#45;type&#45;preload&#58;concept&#45;request&#45;destination&#45;2"></a>[destination](https://fetch.spec.whatwg.org/#concept-request-destination)&#44; <a id="link&#45;type&#45;preload&#58;preload&#45;mode&#45;2"></a>[mode](#preload-mode) is <var>request</var>&#39;s <a id="link&#45;type&#45;preload&#58;concept&#45;request&#45;mode&#45;2"></a>[mode](https://fetch.spec.whatwg.org/#concept-request-mode)&#44; and <a id="link&#45;type&#45;preload&#58;preload&#45;credentials&#45;mode&#45;2"></a>[credentials mode](#preload-credentials-mode) is <var>request</var>&#39;s <a id="link&#45;type&#45;preload&#58;concept&#45;request&#45;credentials&#45;mode&#45;2"></a>[credentials mode](https://fetch.spec.whatwg.org/#concept-request-credentials-mode)&#46;

A <a id="preload&#45;destination"></a><strong>preload destination</strong> is &#34;<code>fetch</code>&#34;&#44; &#34;<code>font</code>&#34;&#44; &#34;<code>image</code>&#34;&#44; &#34;<code>script</code>&#34;&#44; &#34;<code>style</code>&#34;&#44; or &#34;<code>track</code>&#34;&#46;

To <a id="translate&#45;a&#45;preload&#45;destination"></a><strong>translate a preload destination</strong> given a string <var>destination</var>&#58;

1.  If <var>destination</var> is not a <a id="link&#45;type&#45;preload&#58;preload&#45;destination&#45;6"></a>[preload destination](#preload-destination)&#44; then return null&#46;

2.  Return the result of <a id="link&#45;type&#45;preload&#58;concept&#45;potential&#45;destination&#45;translate"></a>[translating](https://fetch.spec.whatwg.org/#concept-potential-destination-translate) <var>destination</var>&#46;

To <a id="preload"></a><strong>preload</strong> given a <a id="link&#45;type&#45;preload&#58;link&#45;processing&#45;options"></a>[link processing options](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-processing-options) <var>options</var> and an optional <var>processResponse</var>&#44; which is an algorithm accepting a <a id="link&#45;type&#45;preload&#58;concept&#45;response&#45;4"></a>[response](https://fetch.spec.whatwg.org/#concept-response)&#58;

1.  If <var>options</var>&#39;s <a id="link&#45;type&#45;preload&#58;link&#45;options&#45;type"></a>[type](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-type) doesn&#39;t <a id="link&#45;type&#45;preload&#58;match&#45;preload&#45;type"></a>[match](#match-preload-type) <var>options</var>&#39;s <a id="link&#45;type&#45;preload&#58;link&#45;options&#45;destination"></a>[destination](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-destination)&#44; then return&#46;

2.  If <var>options</var>&#39;s <a id="link&#45;type&#45;preload&#58;link&#45;options&#45;destination&#45;2"></a>[destination](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-destination) is &#34;<code>image</code>&#34; and <var>options</var>&#39;s <a id="link&#45;type&#45;preload&#58;link&#45;options&#45;source&#45;set"></a>[source set](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-source-set) is not null&#44; then set <var>options</var>&#39;s <a id="link&#45;type&#45;preload&#58;link&#45;options&#45;href"></a>[href](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-href) to the result of <a id="link&#45;type&#45;preload&#58;select&#45;an&#45;image&#45;source&#45;from&#45;a&#45;source&#45;set"></a>[selecting an image source](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#select-an-image-source-from-a-source-set) from <var>options</var>&#39;s <a id="link&#45;type&#45;preload&#58;link&#45;options&#45;source&#45;set&#45;2"></a>[source set](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-source-set)&#46;

3.  Let <var>request</var> be the result of <a id="link&#45;type&#45;preload&#58;create&#45;a&#45;link&#45;request"></a>[creating a link request](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#create-a-link-request) given <var>options</var>&#46;

4.  If <var>request</var> is null&#44; then return&#46;

5.  Let <var>unsafeEndTime</var> be 0&#46;

6.  Let <var>entry</var> be a new <a id="link&#45;type&#45;preload&#58;preload&#45;entry"></a>[preload entry](#preload-entry) whose <a id="link&#45;type&#45;preload&#58;preload&#45;integrity&#45;metadata&#45;2"></a>[integrity metadata](#preload-integrity-metadata) is <var>options</var>&#39;s <a id="link&#45;type&#45;preload&#58;link&#45;options&#45;integrity"></a>[integrity](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-integrity)&#46;

7.  Let <var>key</var> be the result of <a id="link&#45;type&#45;preload&#58;create&#45;a&#45;preload&#45;key"></a>[creating a preload key](#create-a-preload-key) given <var>request</var>&#46;

8.  If <var>options</var>&#39;s <a id="link&#45;type&#45;preload&#58;link&#45;options&#45;document"></a>[document](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-document) is null&#44; then set <var>request</var>&#39;s <a id="link&#45;type&#45;preload&#58;concept&#45;request&#45;initiator&#45;type"></a>[initiator type](https://fetch.spec.whatwg.org/#request-initiator-type) to &#34;<code>early hint</code>&#34;&#46;

9.  Let <var>controller</var> be null&#46;

10. Let <var>reportTiming</var> given a <a id="link&#45;type&#45;preload&#58;document&#45;2"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> <var>document</var> be to <a id="link&#45;type&#45;preload&#58;report&#45;timing"></a>[report timing](https://fetch.spec.whatwg.org/#finalize-and-report-timing) for <var>controller</var> given <var>document</var>&#39;s <a id="link&#45;type&#45;preload&#58;concept&#45;relevant&#45;global"></a>[relevant global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global)&#46;

11. Set <var>controller</var> to the result of <a id="link&#45;type&#45;preload&#58;concept&#45;fetch&#45;2"></a>[fetching](https://fetch.spec.whatwg.org/#concept-fetch) <var>request</var>&#44; with <a id="link&#45;type&#45;preload&#58;processresponseconsumebody"></a><em>[processResponseConsumeBody](https://fetch.spec.whatwg.org/#process-response-end-of-body)</em> set to the following steps given a <a id="link&#45;type&#45;preload&#58;concept&#45;response&#45;5"></a>[response](https://fetch.spec.whatwg.org/#concept-response) <var>response</var> and null&#44; failure&#44; or a <a id="link&#45;type&#45;preload&#58;byte&#45;sequence"></a>[byte sequence](https://infra.spec.whatwg.org/#byte-sequence) <var>bodyBytes</var>&#58;

    1.  If <var>bodyBytes</var> is a <a id="link&#45;type&#45;preload&#58;byte&#45;sequence&#45;2"></a>[byte sequence](https://infra.spec.whatwg.org/#byte-sequence)&#44; then set <var>response</var>&#39;s <a id="link&#45;type&#45;preload&#58;concept&#45;response&#45;body"></a>[body](https://fetch.spec.whatwg.org/#concept-response-body) to <var>bodyBytes</var> <a id="link&#45;type&#45;preload&#58;as&#45;a&#45;body"></a>[as a body](https://fetch.spec.whatwg.org/#byte-sequence-as-a-body)&#46;

        <strong>Note&#58;</strong>

        By using <a id="link&#45;type&#45;preload&#58;processresponseconsumebody&#45;2"></a><em>[processResponseConsumeBody](https://fetch.spec.whatwg.org/#process-response-end-of-body)</em>&#44; we have <a id="link&#45;type&#45;preload&#58;body&#45;safely&#45;extract"></a>[extracted](https://fetch.spec.whatwg.org/#bodyinit-safely-extract) the entire <a id="link&#45;type&#45;preload&#58;concept&#45;response&#45;body&#45;2"></a>[body](https://fetch.spec.whatwg.org/#concept-response-body)&#46; This is necessary to ensure the preloader loads the entire body from the network&#44; regardless of whether the preload will be consumed &#40;which is uncertain at this point&#41;&#46; This step then resets the request&#39;s body to a new body containing the same bytes&#44; so that other specifications can read from it at the time of actual consumption&#44; despite us having already done so once&#46;

    2.  Otherwise&#44; set <var>response</var> to a <a id="link&#45;type&#45;preload&#58;network&#45;error&#45;2"></a>[network error](https://fetch.spec.whatwg.org/#concept-network-error)&#46;

    3.  Set <var>unsafeEndTime</var> to the <a id="link&#45;type&#45;preload&#58;unsafe&#45;shared&#45;current&#45;time"></a>[unsafe shared current time](https://w3c.github.io/hr-time/#dfn-unsafe-shared-current-time)&#46;

    4.  If <var>options</var>&#39;s <a id="link&#45;type&#45;preload&#58;link&#45;options&#45;document&#45;2"></a>[document](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-document) is not null&#44; then call <var>reportTiming</var> given <var>options</var>&#39;s <a id="link&#45;type&#45;preload&#58;link&#45;options&#45;document&#45;3"></a>[document](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-document)&#46;

    5.  If <var>entry</var>&#39;s <a id="link&#45;type&#45;preload&#58;preload&#45;on&#45;response&#45;available&#45;2"></a>[on response available](#preload-on-response-available) is null&#44; then set <var>entry</var>&#39;s <a id="link&#45;type&#45;preload&#58;preload&#45;response&#45;3"></a>[response](#preload-response) to <var>response</var>&#59; otherwise call <var>entry</var>&#39;s <a id="link&#45;type&#45;preload&#58;preload&#45;on&#45;response&#45;available&#45;3"></a>[on response available](#preload-on-response-available) given <var>response</var>&#46;

    6.  If <var>processResponse</var> is given&#44; then call <var>processResponse</var> with <var>response</var>&#46;

12. Let <var>commit</var> be the following steps given a <a id="link&#45;type&#45;preload&#58;document&#45;3"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> <var>document</var>&#58;

    1.  If <var>entry</var>&#39;s <a id="link&#45;type&#45;preload&#58;preload&#45;response&#45;4"></a>[response](#preload-response) is not null&#44; then call <var>reportTiming</var> given <var>document</var>&#46;

    2.  Set <var>document</var>&#39;s <a id="link&#45;type&#45;preload&#58;map&#45;of&#45;preloaded&#45;resources&#45;2"></a>[map of preloaded resources](#map-of-preloaded-resources)&#91;<var>key</var>&#93; to <var>entry</var>&#46;

13. If <var>options</var>&#39;s <a id="link&#45;type&#45;preload&#58;link&#45;options&#45;document&#45;4"></a>[document](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-document) is null&#44; then set <var>options</var>&#39;s <a id="link&#45;type&#45;preload&#58;link&#45;options&#45;on&#45;document&#45;ready"></a>[on document ready](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-on-document-ready) to <var>commit</var>&#46; Otherwise&#44; call <var>commit</var> with <var>options</var>&#39;s <a id="link&#45;type&#45;preload&#58;link&#45;options&#45;document&#45;5"></a>[document](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-document)&#46;

The <a id="link&#45;type&#45;preload&#58;fetch&#45;and&#45;process&#45;the&#45;linked&#45;resource&#45;2"></a>[fetch and process the linked resource](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#fetch-and-process-the-linked-resource) steps for this type of linked resource&#44; given a <a id="link&#45;type&#45;preload&#58;the&#45;link&#45;element&#45;8"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <var>el</var>&#44; are&#58;

1.  <a id="link&#45;type&#45;preload&#58;update&#45;the&#45;source&#45;set"></a>[Update the source set](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#update-the-source-set) for <var>el</var>&#46;

2.  Let <var>options</var> be the result of <a id="link&#45;type&#45;preload&#58;create&#45;link&#45;options&#45;from&#45;element"></a>[creating link options](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#create-link-options-from-element) from <var>el</var>&#46;

3.  Let <var>destination</var> be the result of <a id="link&#45;type&#45;preload&#58;translate&#45;a&#45;preload&#45;destination"></a>[translating](#translate-a-preload-destination) the keyword representing the state of <var>el</var>&#39;s <a id="link&#45;type&#45;preload&#58;attr&#45;link&#45;as&#45;3"></a><code>[as](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-as)</code> attribute&#46;

4.  If <var>destination</var> is null&#44; then return&#46;

5.  Set <var>options</var>&#39;s <a id="link&#45;type&#45;preload&#58;link&#45;options&#45;destination&#45;3"></a>[destination](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-options-destination) to <var>destination</var>&#46;

6.  <a id="link&#45;type&#45;preload&#58;preload"></a>[Preload](#preload) <var>options</var>&#44; with the following steps given a <a id="link&#45;type&#45;preload&#58;concept&#45;response&#45;6"></a>[response](https://fetch.spec.whatwg.org/#concept-response) <var>response</var>&#58;

    1.  If <var>response</var> is a <a id="link&#45;type&#45;preload&#58;network&#45;error&#45;3"></a>[network error](https://fetch.spec.whatwg.org/#concept-network-error)&#44; <a id="link&#45;type&#45;preload&#58;concept&#45;event&#45;fire"></a>[fire an event](https://dom.spec.whatwg.org/#concept-event-fire) named <a id="link&#45;type&#45;preload&#58;event&#45;error"></a><code>[error](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-error)</code> at <var>el</var>&#46; Otherwise&#44; <a id="link&#45;type&#45;preload&#58;concept&#45;event&#45;fire&#45;2"></a>[fire an event](https://dom.spec.whatwg.org/#concept-event-fire) named <a id="link&#45;type&#45;preload&#58;event&#45;load"></a><code>[load](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-load)</code> at <var>el</var>&#46;

        The actual browsers&#39; behavior is different from the spec here&#44; and the feasibility of changing the behavior has not yet been investigated&#46; See [issue &#35;1142](https://github.com/whatwg/html/issues/1142)&#46;

The <a id="link&#45;type&#45;preload&#58;process&#45;a&#45;link&#45;header"></a>[process a link header](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#process-a-link-header) step for this type of link given a <a id="link&#45;type&#45;preload&#58;link&#45;processing&#45;options&#45;2"></a>[link processing options](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#link-processing-options) <var>options</var> is to <a id="link&#45;type&#45;preload&#58;preload&#45;2"></a>[preload](#preload) <var>options</var>&#46;

##### <a id="link&#45;type&#45;privacy&#45;policy"></a>4&#46;6&#46;8&#46;21 Link type &#34;<strong><code>privacy&#45;policy</code></strong>&#34;[](#link-type-privacy-policy)

The <a id="link&#45;type&#45;privacy&#45;policy&#58;link&#45;type&#45;privacy&#45;policy"></a><code>[privacy&#45;policy](#link-type-privacy-policy)</code> keyword may be used with <a id="link&#45;type&#45;privacy&#45;policy&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code>&#44; <a id="link&#45;type&#45;privacy&#45;policy&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code>&#44; and <a id="link&#45;type&#45;privacy&#45;policy&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements&#46; This keyword creates a <a id="link&#45;type&#45;privacy&#45;policy&#58;hyperlink"></a>[hyperlink](#hyperlink)&#46;

The <a id="link&#45;type&#45;privacy&#45;policy&#58;link&#45;type&#45;privacy&#45;policy&#45;2"></a><code>[privacy&#45;policy](#link-type-privacy-policy)</code> keyword indicates that the referenced document contains information about the data collection and usage practices that apply to the current document&#44; as described in more detail in Additional Link Relation Types&#46; The referenced document may be a standalone privacy policy&#44; or a specific section of some more general document&#46; [&#91;RFC6903&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsRFC6903)

##### <a id="link&#45;type&#45;search"></a>4&#46;6&#46;8&#46;22 Link type &#34;<strong><code>search</code></strong>&#34;[](#link-type-search)

The <a id="link&#45;type&#45;search&#58;link&#45;type&#45;search"></a><code>[search](#link-type-search)</code> keyword may be used with <a id="link&#45;type&#45;search&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code>&#44; <a id="link&#45;type&#45;search&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code>&#44; <a id="link&#45;type&#45;search&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code>&#44; and <a id="link&#45;type&#45;search&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements&#46; This keyword creates a <a id="link&#45;type&#45;search&#58;hyperlink"></a>[hyperlink](#hyperlink)&#46;

The <a id="link&#45;type&#45;search&#58;link&#45;type&#45;search&#45;2"></a><code>[search](#link-type-search)</code> keyword indicates that the referenced document provides an interface specifically for searching the document and its related resources&#46;

<strong>Note&#58;</strong>

OpenSearch description documents can be used with <a id="link&#45;type&#45;search&#58;the&#45;link&#45;element&#45;2"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements and the <a id="link&#45;type&#45;search&#58;link&#45;type&#45;search&#45;3"></a><code>[search](#link-type-search)</code> link type to enable user agents to autodiscover search interfaces&#46; [&#91;OPENSEARCH&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsOPENSEARCH)

##### <a id="link&#45;type&#45;stylesheet"></a>4&#46;6&#46;8&#46;23 Link type &#34;<strong><code>stylesheet</code></strong>&#34;[](#link-type-stylesheet)

The <a id="link&#45;type&#45;stylesheet&#58;link&#45;type&#45;stylesheet"></a><code>[stylesheet](#link-type-stylesheet)</code> keyword may be used with <a id="link&#45;type&#45;stylesheet&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements&#46; This keyword creates an <a id="link&#45;type&#45;stylesheet&#58;external&#45;resource&#45;link"></a>[external resource link](#external-resource-link) that contributes to the styling processing model&#46; This keyword is <a id="link&#45;type&#45;stylesheet&#58;body&#45;ok"></a>[body&#45;ok](#body-ok)&#46;

The specified resource is a <a id="link&#45;type&#45;stylesheet&#58;css&#45;style&#45;sheet"></a>[CSS style sheet](https://drafts.csswg.org/cssom/#css-style-sheet) that describes how to present the document&#46;

<strong>⚠</strong>MDN

[Alternative&#95;style&#95;sheets](https://developer.mozilla.org/en-US/docs/Web/CSS/Alternative_style_sheets)

Support in one engine only&#46;

Firefox3&#43;Safari&#63;Chrome1–48

------------------------------------------------------------------------

OperaYesEdgeNo

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet Explorer8&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

If the <a id="link&#45;type&#45;stylesheet&#58;rel&#45;alternate"></a><code>[alternate](#rel-alternate)</code> keyword is also specified on the <a id="link&#45;type&#45;stylesheet&#58;the&#45;link&#45;element&#45;2"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element&#44; then <a id="the&#45;link&#45;is&#45;an&#45;alternative&#45;stylesheet"></a><strong>the link is an alternative style sheet</strong>&#59; in this case&#44; the <a id="link&#45;type&#45;stylesheet&#58;attr&#45;title"></a><code>[title](html-language-foundations--877d6146487f--b55e73c31978.md#attr-title)</code> attribute must be specified on the <a id="link&#45;type&#45;stylesheet&#58;the&#45;link&#45;element&#45;3"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element&#44; with a non&#45;empty value&#46;

The default type for resources given by the <a id="link&#45;type&#45;stylesheet&#58;link&#45;type&#45;stylesheet&#45;2"></a><code>[stylesheet](#link-type-stylesheet)</code> keyword is <a id="link&#45;type&#45;stylesheet&#58;text&#47;css"></a><code>[text&#47;css](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#text/css)</code>&#46;

A <a id="link&#45;type&#45;stylesheet&#58;the&#45;link&#45;element&#45;4"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element of this type is <a id="link&#45;type&#45;stylesheet&#58;implicitly&#45;potentially&#45;render&#45;blocking"></a>[implicitly potentially render&#45;blocking](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#implicitly-potentially-render-blocking) if the element was created by its <a id="link&#45;type&#45;stylesheet&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s parser&#46;

When the <a id="link&#45;type&#45;stylesheet&#58;attr&#45;link&#45;disabled"></a><code>[disabled](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-disabled)</code> attribute of a <a id="link&#45;type&#45;stylesheet&#58;the&#45;link&#45;element&#45;5"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element with a <a id="link&#45;type&#45;stylesheet&#58;link&#45;type&#45;stylesheet&#45;3"></a><code>[stylesheet](#link-type-stylesheet)</code> keyword is set&#44; <a id="link&#45;type&#45;stylesheet&#58;disable&#45;a&#45;css&#45;style&#45;sheet"></a>[disable](https://drafts.csswg.org/cssom/#disable-a-css-style-sheet) the <a id="link&#45;type&#45;stylesheet&#58;associated&#45;css&#45;style&#45;sheet"></a>[associated CSS style sheet](https://drafts.csswg.org/cssom/#associated-css-style-sheet)&#46;

The appropriate times to <a id="link&#45;type&#45;stylesheet&#58;fetch&#45;and&#45;process&#45;the&#45;linked&#45;resource"></a>[fetch and process](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#fetch-and-process-the-linked-resource) this type of link are&#58;

- When the <a id="link&#45;type&#45;stylesheet&#58;external&#45;resource&#45;link&#45;2"></a>[external resource link](#external-resource-link) is created on a <a id="link&#45;type&#45;stylesheet&#58;the&#45;link&#45;element&#45;6"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element that is already <a id="link&#45;type&#45;stylesheet&#58;browsing&#45;context&#45;connected"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected)&#46;

- When the <a id="link&#45;type&#45;stylesheet&#58;external&#45;resource&#45;link&#45;3"></a>[external resource link](#external-resource-link)&#39;s <a id="link&#45;type&#45;stylesheet&#58;the&#45;link&#45;element&#45;7"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <a id="link&#45;type&#45;stylesheet&#58;becomes&#45;browsing&#45;context&#45;connected"></a>[becomes browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#becomes-browsing-context-connected)&#46;

- When the <a id="link&#45;type&#45;stylesheet&#58;attr&#45;link&#45;href"></a><code>[href](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-href)</code> attribute of the <a id="link&#45;type&#45;stylesheet&#58;the&#45;link&#45;element&#45;8"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element of an <a id="link&#45;type&#45;stylesheet&#58;external&#45;resource&#45;link&#45;4"></a>[external resource link](#external-resource-link) that is already <a id="link&#45;type&#45;stylesheet&#58;browsing&#45;context&#45;connected&#45;2"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected) is changed&#46;

- When the <a id="link&#45;type&#45;stylesheet&#58;attr&#45;link&#45;disabled&#45;2"></a><code>[disabled](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-disabled)</code> attribute of the <a id="link&#45;type&#45;stylesheet&#58;the&#45;link&#45;element&#45;9"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element of an <a id="link&#45;type&#45;stylesheet&#58;external&#45;resource&#45;link&#45;5"></a>[external resource link](#external-resource-link) that is already <a id="link&#45;type&#45;stylesheet&#58;browsing&#45;context&#45;connected&#45;3"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected) is set&#44; changed&#44; or removed&#46;

- When the <a id="link&#45;type&#45;stylesheet&#58;attr&#45;link&#45;crossorigin"></a><code>[crossorigin](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-crossorigin)</code> attribute of the <a id="link&#45;type&#45;stylesheet&#58;the&#45;link&#45;element&#45;10"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element of an <a id="link&#45;type&#45;stylesheet&#58;external&#45;resource&#45;link&#45;6"></a>[external resource link](#external-resource-link) that is already <a id="link&#45;type&#45;stylesheet&#58;browsing&#45;context&#45;connected&#45;4"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected) is set&#44; changed&#44; or removed&#46;

- When the <a id="link&#45;type&#45;stylesheet&#58;attr&#45;link&#45;type"></a><code>[type](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-type)</code> attribute of the <a id="link&#45;type&#45;stylesheet&#58;the&#45;link&#45;element&#45;11"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element of an <a id="link&#45;type&#45;stylesheet&#58;external&#45;resource&#45;link&#45;7"></a>[external resource link](#external-resource-link) that is already <a id="link&#45;type&#45;stylesheet&#58;browsing&#45;context&#45;connected&#45;5"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected) is set or changed to a value that does not or no longer matches the <a id="link&#45;type&#45;stylesheet&#58;content&#45;type"></a>[Content&#45;Type metadata](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-type) of the previous obtained external resource&#44; if any&#46;

- When the <a id="link&#45;type&#45;stylesheet&#58;attr&#45;link&#45;type&#45;2"></a><code>[type](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-type)</code> attribute of the <a id="link&#45;type&#45;stylesheet&#58;the&#45;link&#45;element&#45;12"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element of an <a id="link&#45;type&#45;stylesheet&#58;external&#45;resource&#45;link&#45;8"></a>[external resource link](#external-resource-link) that is already <a id="link&#45;type&#45;stylesheet&#58;browsing&#45;context&#45;connected&#45;6"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected)&#44; but was previously not obtained due to the <a id="link&#45;type&#45;stylesheet&#58;attr&#45;link&#45;type&#45;3"></a><code>[type](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-type)</code> attribute specifying an unsupported type&#44; is removed or changed&#46;

- When the <a id="link&#45;type&#45;stylesheet&#58;external&#45;resource&#45;link&#45;9"></a>[external resource link](#external-resource-link) that is already <a id="link&#45;type&#45;stylesheet&#58;browsing&#45;context&#45;connected&#45;7"></a>[browsing&#45;context connected](html-language-foundations--877d6146487f--b55e73c31978.md#browsing-context-connected) changes from being <a id="link&#45;type&#45;stylesheet&#58;the&#45;link&#45;is&#45;an&#45;alternative&#45;stylesheet"></a>[an alternative style sheet](#the-link-is-an-alternative-stylesheet) to not being one&#44; or vice versa&#46;

<strong>Quirk</strong>&#58; If the document has been set to <a id="link&#45;type&#45;stylesheet&#58;quirks&#45;mode"></a>[quirks mode](https://dom.spec.whatwg.org/#concept-document-quirks)&#44; has the <a id="link&#45;type&#45;stylesheet&#58;same&#45;origin"></a>[same origin](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#same-origin) as the <a id="link&#45;type&#45;stylesheet&#58;url"></a>[URL](https://url.spec.whatwg.org/#concept-url) of the external resource&#44; and the <a id="link&#45;type&#45;stylesheet&#58;content&#45;type&#45;2"></a>[Content&#45;Type metadata](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-type) of the external resource is not a supported style sheet type&#44; the user agent must instead assume it to be <a id="link&#45;type&#45;stylesheet&#58;text&#47;css&#45;2"></a><code>[text&#47;css](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#text/css)</code>&#46;

The <a id="link&#45;type&#45;stylesheet&#58;linked&#45;resource&#45;fetch&#45;setup&#45;steps"></a>[linked resource fetch setup steps](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#linked-resource-fetch-setup-steps) for this type of linked resource&#44; given a <a id="link&#45;type&#45;stylesheet&#58;the&#45;link&#45;element&#45;13"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <var>el</var> and <a id="link&#45;type&#45;stylesheet&#58;concept&#45;request"></a>[request](https://fetch.spec.whatwg.org/#concept-request) <var>request</var>&#44; are&#58;

1.  If <var>el</var>&#39;s <a id="link&#45;type&#45;stylesheet&#58;attr&#45;link&#45;disabled&#45;3"></a><code>[disabled](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-disabled)</code> attribute is set&#44; then return false&#46;

2.  If <var>el</var> <a id="link&#45;type&#45;stylesheet&#58;contributes&#45;a&#45;script&#45;blocking&#45;style&#45;sheet"></a>[contributes a script&#45;blocking style sheet](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#contributes-a-script-blocking-style-sheet)&#44; <a id="link&#45;type&#45;stylesheet&#58;set&#45;append"></a>[append](https://infra.spec.whatwg.org/#set-append) <var>el</var> to its <a id="link&#45;type&#45;stylesheet&#58;node&#45;document&#45;2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="link&#45;type&#45;stylesheet&#58;script&#45;blocking&#45;style&#45;sheet&#45;set"></a>[script&#45;blocking style sheet set](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#script-blocking-style-sheet-set)&#46;

3.  If <var>el</var>&#39;s <a id="link&#45;type&#45;stylesheet&#58;attr&#45;link&#45;media"></a><code>[media](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-media)</code> attribute&#39;s value <a id="link&#45;type&#45;stylesheet&#58;matches&#45;the&#45;environment"></a>[matches the environment](html-language-foundations--877d6146487f--b55e73c31978.md#matches-the-environment) and <var>el</var> is <a id="link&#45;type&#45;stylesheet&#58;potentially&#45;render&#45;blocking"></a>[potentially render&#45;blocking](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#potentially-render-blocking)&#44; then <a id="link&#45;type&#45;stylesheet&#58;block&#45;rendering"></a>[block rendering](html-language-foundations--877d6146487f--b55e73c31978.md#block-rendering) on <var>el</var>&#46;

4.  If <var>el</var> is currently <a id="link&#45;type&#45;stylesheet&#58;render&#45;blocking"></a>[render&#45;blocking](html-language-foundations--877d6146487f--b55e73c31978.md#render-blocking)&#44; then set <var>request</var>&#39;s <a id="link&#45;type&#45;stylesheet&#58;concept&#45;request&#45;render&#45;blocking"></a>[render&#45;blocking](https://fetch.spec.whatwg.org/#request-render-blocking) to true&#46;

5.  Return true&#46;

See [issue &#35;968](https://github.com/whatwg/html/issues/968) for plans to use the CSSOM [fetch a CSS style sheet](https://drafts.csswg.org/cssom/#fetching-css-style-sheets) algorithm instead of the <a id="link&#45;type&#45;stylesheet&#58;default&#45;fetch&#45;and&#45;process&#45;the&#45;linked&#45;resource"></a>[default fetch and process the linked resource](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#default-fetch-and-process-the-linked-resource) algorithm&#46; In the meantime&#44; any <a id="link&#45;type&#45;stylesheet&#58;critical&#45;subresources"></a>[critical subresource](html-language-foundations--877d6146487f--b55e73c31978.md#critical-subresources) <a id="link&#45;type&#45;stylesheet&#58;concept&#45;request&#45;2"></a>[request](https://fetch.spec.whatwg.org/#concept-request) should have its <a id="link&#45;type&#45;stylesheet&#58;concept&#45;request&#45;render&#45;blocking&#45;2"></a>[render&#45;blocking](https://fetch.spec.whatwg.org/#request-render-blocking) set to whether or not the <a id="link&#45;type&#45;stylesheet&#58;the&#45;link&#45;element&#45;14"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element is currently <a id="link&#45;type&#45;stylesheet&#58;render&#45;blocking&#45;2"></a>[render&#45;blocking](html-language-foundations--877d6146487f--b55e73c31978.md#render-blocking)&#46;

To <a id="link&#45;type&#45;stylesheet&#58;process&#45;the&#45;linked&#45;resource"></a>[process this type of linked resource](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#process-the-linked-resource) given a <a id="link&#45;type&#45;stylesheet&#58;the&#45;link&#45;element&#45;15"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element <var>el</var>&#44; boolean <var>success</var>&#44; <a id="link&#45;type&#45;stylesheet&#58;concept&#45;response"></a>[response](https://fetch.spec.whatwg.org/#concept-response) <var>response</var>&#44; and <a id="link&#45;type&#45;stylesheet&#58;byte&#45;sequence"></a>[byte sequence](https://infra.spec.whatwg.org/#byte-sequence) <var>bodyBytes</var>&#58;

1.  If the resource&#39;s <a id="link&#45;type&#45;stylesheet&#58;content&#45;type&#45;3"></a>[Content&#45;Type metadata](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-type) is not <a id="link&#45;type&#45;stylesheet&#58;text&#47;css&#45;3"></a><code>[text&#47;css](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#text/css)</code>&#44; then set <var>success</var> to false&#46;

2.  If <var>el</var> no longer creates an <a id="link&#45;type&#45;stylesheet&#58;external&#45;resource&#45;link&#45;10"></a>[external resource link](#external-resource-link) that contributes to the styling processing model&#44; or if&#44; since the resource in question was <a id="link&#45;type&#45;stylesheet&#58;fetch&#45;and&#45;process&#45;the&#45;linked&#45;resource&#45;2"></a>[fetched](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#fetch-and-process-the-linked-resource)&#44; it has become appropriate to <a id="link&#45;type&#45;stylesheet&#58;fetch&#45;and&#45;process&#45;the&#45;linked&#45;resource&#45;3"></a>[fetch](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#fetch-and-process-the-linked-resource) it again&#58;

    1.  <a id="link&#45;type&#45;stylesheet&#58;list&#45;remove"></a>[Remove](https://infra.spec.whatwg.org/#list-remove) <var>el</var> from <var>el</var>&#39;s <a id="link&#45;type&#45;stylesheet&#58;node&#45;document&#45;3"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="link&#45;type&#45;stylesheet&#58;script&#45;blocking&#45;style&#45;sheet&#45;set&#45;2"></a>[script&#45;blocking style sheet set](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#script-blocking-style-sheet-set)&#46;

    2.  Return&#46;

3.  If <var>el</var> has an <a id="link&#45;type&#45;stylesheet&#58;associated&#45;css&#45;style&#45;sheet&#45;2"></a>[associated CSS style sheet](https://drafts.csswg.org/cssom/#associated-css-style-sheet)&#44; <a id="link&#45;type&#45;stylesheet&#58;remove&#45;a&#45;css&#45;style&#45;sheet"></a>[remove the CSS style sheet](https://drafts.csswg.org/cssom/#remove-a-css-style-sheet)&#46;

4.  If <var>success</var> is true&#58;

    1.  <a id="link&#45;type&#45;stylesheet&#58;create&#45;a&#45;css&#45;style&#45;sheet"></a>[Create a CSS style sheet](https://drafts.csswg.org/cssom/#create-a-css-style-sheet) with the following properties&#58;

        <strong><a id="link&#45;type&#45;stylesheet&#58;concept&#45;css&#45;style&#45;sheet&#45;type"></a>[type](https://drafts.csswg.org/cssom/#concept-css-style-sheet-type)</strong>

        <a id="link&#45;type&#45;stylesheet&#58;text&#47;css&#45;4"></a><code>[text&#47;css](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#text/css)</code>

        <strong><a id="link&#45;type&#45;stylesheet&#58;concept&#45;css&#45;style&#45;sheet&#45;location"></a>[location](https://drafts.csswg.org/cssom/#concept-css-style-sheet-location)</strong>

        <var>response</var>&#39;s <a id="link&#45;type&#45;stylesheet&#58;concept&#45;response&#45;url&#45;list"></a>[URL list](https://fetch.spec.whatwg.org/#concept-response-url-list)&#91;0&#93;

        We provide a URL here on the assumption that [w3c&#47;csswg&#45;drafts issue &#35;9316](https://github.com/w3c/csswg-drafts/issues/9316) will be fixed&#46;

        <strong><a id="link&#45;type&#45;stylesheet&#58;concept&#45;css&#45;style&#45;sheet&#45;owner&#45;node"></a>[owner node](https://drafts.csswg.org/cssom/#concept-css-style-sheet-owner-node)</strong>

        <var>el</var>

        <strong><a id="link&#45;type&#45;stylesheet&#58;concept&#45;css&#45;style&#45;sheet&#45;media"></a>[media](https://drafts.csswg.org/cssom/#concept-css-style-sheet-media)</strong>

        The <a id="link&#45;type&#45;stylesheet&#58;attr&#45;link&#45;media&#45;2"></a><code>[media](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-media)</code> attribute of <var>el</var>&#46;

        <strong>Note&#58;</strong>

        This is a reference to the &#40;possibly absent at this time&#41; attribute&#44; rather than a copy of the attribute&#39;s current value&#46; CSSOM defines what happens when the attribute is dynamically set&#44; changed&#44; or removed&#46;
        <strong><a id="link&#45;type&#45;stylesheet&#58;concept&#45;css&#45;style&#45;sheet&#45;title"></a>[title](https://drafts.csswg.org/cssom/#concept-css-style-sheet-title)</strong>

        The <a id="link&#45;type&#45;stylesheet&#58;attr&#45;link&#45;title"></a><code>[title](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-link-title)</code> attribute of <var>el</var>&#44; if <var>el</var> is <a id="link&#45;type&#45;stylesheet&#58;in&#45;a&#45;document&#45;tree"></a>[in a document tree](https://dom.spec.whatwg.org/#in-a-document-tree)&#44; or the empty string otherwise&#46;

        <strong>Note&#58;</strong>

        This is similarly a reference to the attribute&#44; rather than a copy of the attribute&#39;s current value&#46;
        <strong><a id="link&#45;type&#45;stylesheet&#58;concept&#45;css&#45;style&#45;sheet&#45;alternate&#45;flag"></a>[alternate flag](https://drafts.csswg.org/cssom/#concept-css-style-sheet-alternate-flag)</strong>

        Set if <a id="link&#45;type&#45;stylesheet&#58;the&#45;link&#45;is&#45;an&#45;alternative&#45;stylesheet&#45;2"></a>[the link is an alternative style sheet](#the-link-is-an-alternative-stylesheet) and <var>el</var>&#39;s <a id="link&#45;type&#45;stylesheet&#58;explicitly&#45;enabled"></a>[explicitly enabled](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#explicitly-enabled) is false&#59; unset otherwise&#46;

        <strong><a id="link&#45;type&#45;stylesheet&#58;concept&#45;css&#45;style&#45;sheet&#45;origin&#45;clean&#45;flag"></a>[origin&#45;clean flag](https://drafts.csswg.org/cssom/#concept-css-style-sheet-origin-clean-flag)</strong>

        Set if the resource is <a id="link&#45;type&#45;stylesheet&#58;cors&#45;same&#45;origin"></a>[CORS&#45;same&#45;origin](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#cors-same-origin)&#59; unset otherwise&#46;

        <strong><a id="link&#45;type&#45;stylesheet&#58;concept&#45;css&#45;style&#45;sheet&#45;parent&#45;css&#45;style&#45;sheet"></a>[parent CSS style sheet](https://drafts.csswg.org/cssom/#concept-css-style-sheet-parent-css-style-sheet)</strong>

        <strong><a id="link&#45;type&#45;stylesheet&#58;concept&#45;css&#45;style&#45;sheet&#45;owner&#45;css&#45;rule"></a>[owner CSS rule](https://drafts.csswg.org/cssom/#concept-css-style-sheet-owner-css-rule)</strong>

        null

        <strong><a id="link&#45;type&#45;stylesheet&#58;concept&#45;css&#45;style&#45;sheet&#45;disabled&#45;flag"></a>[disabled flag](https://drafts.csswg.org/cssom/#concept-css-style-sheet-disabled-flag)</strong>

        Left at its default value&#46;

        <strong><a id="link&#45;type&#45;stylesheet&#58;concept&#45;css&#45;style&#45;sheet&#45;css&#45;rules"></a>[CSS rules](https://drafts.csswg.org/cssom/#concept-css-style-sheet-css-rules)</strong>

        Left uninitialized&#46;

        This doesn&#39;t seem right&#46; Presumably we should be using <var>bodyBytes</var>&#63; Tracked as [issue &#35;2997](https://github.com/whatwg/html/issues/2997)&#46;

        The CSS <a id="link&#45;type&#45;stylesheet&#58;environment&#45;encoding"></a>[environment encoding](https://drafts.csswg.org/css-syntax/#environment-encoding) is the result of running the following steps&#58; [&#91;CSSSYNTAX&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsCSSSYNTAX)

        1.  If <var>el</var> has a <a id="link&#45;type&#45;stylesheet&#58;attr&#45;link&#45;charset"></a><code>[charset](html-language-obsolete--877d6146487f--66870e9cdabe.md#attr-link-charset)</code> attribute&#44; <a id="link&#45;type&#45;stylesheet&#58;getting&#45;an&#45;encoding"></a>[get an encoding](https://encoding.spec.whatwg.org/#concept-encoding-get) from that attribute&#39;s value&#46; If that succeeds&#44; return the resulting encoding&#46; [&#91;ENCODING&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsENCODING)

        2.  Otherwise&#44; return the <a id="link&#45;type&#45;stylesheet&#58;document&#39;s&#45;character&#45;encoding"></a>[document&#39;s character encoding](https://dom.spec.whatwg.org/#concept-document-encoding)&#46; [&#91;DOM&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsDOM)

    2.  <a id="link&#45;type&#45;stylesheet&#58;concept&#45;event&#45;fire"></a>[Fire an event](https://dom.spec.whatwg.org/#concept-event-fire) named <a id="link&#45;type&#45;stylesheet&#58;event&#45;load"></a><code>[load](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-load)</code> at <var>el</var>&#46;

5.  Otherwise&#44; <a id="link&#45;type&#45;stylesheet&#58;concept&#45;event&#45;fire&#45;2"></a>[fire an event](https://dom.spec.whatwg.org/#concept-event-fire) named <a id="link&#45;type&#45;stylesheet&#58;event&#45;error"></a><code>[error](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-error)</code> at <var>el</var>&#46;

6.  If <var>el</var> <a id="link&#45;type&#45;stylesheet&#58;contributes&#45;a&#45;script&#45;blocking&#45;style&#45;sheet&#45;2"></a>[contributes a script&#45;blocking style sheet](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#contributes-a-script-blocking-style-sheet)&#58;

    1.  <a id="link&#45;type&#45;stylesheet&#58;assert"></a>[Assert](https://infra.spec.whatwg.org/#assert)&#58; <var>el</var>&#39;s <a id="link&#45;type&#45;stylesheet&#58;node&#45;document&#45;4"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="link&#45;type&#45;stylesheet&#58;script&#45;blocking&#45;style&#45;sheet&#45;set&#45;3"></a>[script&#45;blocking style sheet set](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#script-blocking-style-sheet-set) <a id="link&#45;type&#45;stylesheet&#58;list&#45;contains"></a>[contains](https://infra.spec.whatwg.org/#list-contain) <var>el</var>&#46;

    2.  <a id="link&#45;type&#45;stylesheet&#58;list&#45;remove&#45;2"></a>[Remove](https://infra.spec.whatwg.org/#list-remove) <var>el</var> from its <a id="link&#45;type&#45;stylesheet&#58;node&#45;document&#45;5"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="link&#45;type&#45;stylesheet&#58;script&#45;blocking&#45;style&#45;sheet&#45;set&#45;4"></a>[script&#45;blocking style sheet set](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#script-blocking-style-sheet-set)&#46;

7.  <a id="link&#45;type&#45;stylesheet&#58;unblock&#45;rendering"></a>[Unblock rendering](html-language-foundations--877d6146487f--b55e73c31978.md#unblock-rendering) on <var>el</var>&#46;

The <a id="link&#45;type&#45;stylesheet&#58;process&#45;a&#45;link&#45;header"></a>[process a link header](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#process-a-link-header) steps for this type of linked resource are to do nothing&#46;

##### <a id="link&#45;type&#45;tag"></a>4&#46;6&#46;8&#46;24 Link type &#34;<strong><code>tag</code></strong>&#34;[](#link-type-tag)

The <a id="link&#45;type&#45;tag&#58;link&#45;type&#45;tag"></a><code>[tag](#link-type-tag)</code> keyword may be used with <a id="link&#45;type&#45;tag&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code> and <a id="link&#45;type&#45;tag&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements&#46; This keyword creates a <a id="link&#45;type&#45;tag&#58;hyperlink"></a>[hyperlink](#hyperlink)&#46;

The <a id="link&#45;type&#45;tag&#58;link&#45;type&#45;tag&#45;2"></a><code>[tag](#link-type-tag)</code> keyword indicates that the <em>tag</em> that the referenced document represents applies to the current document&#46;

<strong>Note&#58;</strong>

Since it indicates that the tag <em>applies to the current document</em>&#44; it would be inappropriate to use this keyword in the markup of a [tag cloud](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#tag-cloud)&#44; which lists the popular tags across a set of pages&#46;

<strong>Example&#58;</strong>

This document is about some gems&#44; and so it is <em>tagged</em> with &#34;<code>https&#58;&#47;&#47;en&#46;wikipedia&#46;org&#47;wiki&#47;Gemstone</code>&#34; to unambiguously categorize it as applying to the &#34;jewel&#34; kind of gems&#44; and not to&#44; say&#44; the towns in the US&#44; the Ruby package format&#44; or the Swiss locomotive class&#58;

``` text
<!DOCTYPE HTML>
<html lang="en">
 <head>
  <title>My Precious</title>
 </head>
 <body>
  <header><h1>My precious</h1> <p>Summer 2012</p></header>
  <p>Recently I managed to dispose of a red gem that had been
  bothering me. I now have a much nicer blue sapphire.</p>
  <p>The red gem had been found in a bauxite stone while I was digging
  out the office level, but nobody was willing to haul it away. The
  same red gem stayed there for literally years.</p>
  <footer>
   Tags: <a rel=tag href="https://en.wikipedia.org/wiki/Gemstone">Gemstone</a>
  </footer>
 </body>
</html>
```

<strong>Example&#58;</strong>

In <em>this</em> document&#44; there are two articles&#46; The &#34;<a id="link&#45;type&#45;tag&#58;link&#45;type&#45;tag&#45;3"></a><code>[tag](#link-type-tag)</code>&#34; link&#44; however&#44; applies to the whole page &#40;and would do so wherever it was placed&#44; including if it was within the <a id="link&#45;type&#45;tag&#58;the&#45;article&#45;element"></a><code>[article](#the-article-element)</code> elements&#41;&#46;

``` text
<!DOCTYPE HTML>
<html lang="en">
 <head>
  <title>Gem 4/4</title>
 </head>
 <body>
  <article>
   <h1>801: Steinbock</h1>
   <p>The number 801 Gem 4/4 electro-diesel has an ibex and was rebuilt in 2002.</p>
  </article>
  <article>
   <h1>802: Murmeltier</h1>
   <figure>
    <img src="https://upload.wikimedia.org/wikipedia/commons/b/b0/Trains_de_la_Bernina_en_hiver_2.jpg"
         alt="The 802 was red with pantographs and tall vents on the side.">
    <figcaption>The 802 in the 1980s, above Lago Bianco.</figcaption>
   </figure>
   <p>The number 802 Gem 4/4 electro-diesel has a marmot and was rebuilt in 2003.</p>
  </article>
  <p class="topic"><a rel=tag href="https://en.wikipedia.org/wiki/Rhaetian_Railway_Gem_4/4">Gem 4/4</a></p>
 </body>
</html>
```

##### <a id="link&#45;type&#45;terms&#45;of&#45;service"></a>4&#46;6&#46;8&#46;25 Link Type &#34;<strong><code>terms&#45;of&#45;service</code></strong>&#34;[](#link-type-terms-of-service)

The <a id="link&#45;type&#45;terms&#45;of&#45;service&#58;link&#45;type&#45;terms&#45;of&#45;service"></a><code>[terms&#45;of&#45;service](#link-type-terms-of-service)</code> keyword may be used with <a id="link&#45;type&#45;terms&#45;of&#45;service&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code>&#44; <a id="link&#45;type&#45;terms&#45;of&#45;service&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code>&#44; and <a id="link&#45;type&#45;terms&#45;of&#45;service&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements&#46; This keyword creates a <a id="link&#45;type&#45;terms&#45;of&#45;service&#58;hyperlink"></a>[hyperlink](#hyperlink)&#46;

The <a id="link&#45;type&#45;terms&#45;of&#45;service&#58;link&#45;type&#45;terms&#45;of&#45;service&#45;2"></a><code>[terms&#45;of&#45;service](#link-type-terms-of-service)</code> keyword indicates that the referenced document contains information about the agreements between the current document&#39;s provider and users who wish to use the current document&#44; as described in more detail in Additional Link Relation Types&#46; [&#91;RFC6903&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsRFC6903)

##### <a id="sequential&#45;link&#45;types"></a>4&#46;6&#46;8&#46;26 Sequential link types[](#sequential-link-types)

Some documents form part of a sequence of documents&#46;

A sequence of documents is one where each document can have a <em>previous sibling</em> and a <em>next sibling</em>&#46; A document with no previous sibling is the start of its sequence&#44; a document with no next sibling is the end of its sequence&#46;

A document may be part of multiple sequences&#46;

###### <a id="link&#45;type&#45;next"></a>4&#46;6&#46;8&#46;26&#46;1 Link type &#34;<strong><code>next</code></strong>&#34;[](#link-type-next)

The <a id="link&#45;type&#45;next&#58;link&#45;type&#45;next"></a><code>[next](#link-type-next)</code> keyword may be used with <a id="link&#45;type&#45;next&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code>&#44; <a id="link&#45;type&#45;next&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code>&#44; <a id="link&#45;type&#45;next&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code>&#44; and <a id="link&#45;type&#45;next&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements&#46; This keyword creates a <a id="link&#45;type&#45;next&#58;hyperlink"></a>[hyperlink](#hyperlink)&#46;

The <a id="link&#45;type&#45;next&#58;link&#45;type&#45;next&#45;2"></a><code>[next](#link-type-next)</code> keyword indicates that the document is part of a sequence&#44; and that the link is leading to the document that is the next logical document in the sequence&#46;

When the <a id="link&#45;type&#45;next&#58;link&#45;type&#45;next&#45;3"></a><code>[next](#link-type-next)</code> keyword is used with a <a id="link&#45;type&#45;next&#58;the&#45;link&#45;element&#45;2"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element&#44; user agents should process such links as if they were using one of the <a id="link&#45;type&#45;next&#58;link&#45;type&#45;dns&#45;prefetch"></a><code>[dns&#45;prefetch](#link-type-dns-prefetch)</code>&#44; <a id="link&#45;type&#45;next&#58;link&#45;type&#45;preconnect"></a><code>[preconnect](#link-type-preconnect)</code>&#44; or <a id="link&#45;type&#45;next&#58;link&#45;type&#45;prefetch"></a><code>[prefetch](#link-type-prefetch)</code> keywords&#46; Which keyword the user agent wishes to use is implementation&#45;dependent&#59; for example&#44; a user agent may wish to use the less&#45;costly <a id="link&#45;type&#45;next&#58;link&#45;type&#45;preconnect&#45;2"></a><code>[preconnect](#link-type-preconnect)</code> processing model when trying to conserve data&#44; battery power&#44; or processing power&#44; or may wish to pick a keyword depending on heuristic analysis of past user behavior in similar scenarios&#46;

###### <a id="link&#45;type&#45;prev"></a>4&#46;6&#46;8&#46;26&#46;2 Link type &#34;<strong><code>prev</code></strong>&#34;[](#link-type-prev)

The <a id="link&#45;type&#45;prev&#58;link&#45;type&#45;prev"></a><code>[prev](#link-type-prev)</code> keyword may be used with <a id="link&#45;type&#45;prev&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code>&#44; <a id="link&#45;type&#45;prev&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code>&#44; <a id="link&#45;type&#45;prev&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code>&#44; and <a id="link&#45;type&#45;prev&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements&#46; This keyword creates a <a id="link&#45;type&#45;prev&#58;hyperlink"></a>[hyperlink](#hyperlink)&#46;

The <a id="link&#45;type&#45;prev&#58;link&#45;type&#45;prev&#45;2"></a><code>[prev](#link-type-prev)</code> keyword indicates that the document is part of a sequence&#44; and that the link is leading to the document that is the previous logical document in the sequence&#46;

<strong>Synonyms</strong>&#58; For historical reasons&#44; user agents must also treat the keyword &#34;<code>previous</code>&#34; like the <a id="link&#45;type&#45;prev&#58;link&#45;type&#45;prev&#45;3"></a><code>[prev](#link-type-prev)</code> keyword&#46;

##### <a id="other&#45;link&#45;types"></a>4&#46;6&#46;8&#46;27 Other link types[](#other-link-types)

<a id="concept&#45;rel&#45;extensions"></a><strong>Extensions to the predefined set of link types</strong> may be registered on the [microformats page for existing rel values](https://microformats.org/wiki/existing-rel-values#HTML5_link_type_extensions)&#46; [&#91;MFREL&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsMFREL)

Anyone is free to edit the microformats page for existing rel values at any time to add a type&#46; Extension types must be specified with the following information&#58;

<strong>Keyword</strong>

The actual value being defined&#46; The value should not be confusingly similar to any other defined value &#40;e&#46;g&#46; differing only in case&#41;&#46;

If the value contains a U&#43;003A COLON character &#40;&#58;&#41;&#44; it must also be an <a id="other&#45;link&#45;types&#58;absolute&#45;url"></a>[absolute URL](https://url.spec.whatwg.org/#syntax-url-absolute)&#46;

<strong>Effect on&#46;&#46;&#46; <a id="other&#45;link&#45;types&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code></strong>

One of the following&#58;

<strong>Not allowed</strong>

The keyword must not be specified on <a id="other&#45;link&#45;types&#58;the&#45;link&#45;element&#45;2"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements&#46;

<strong>Hyperlink</strong>

The keyword may be specified on a <a id="other&#45;link&#45;types&#58;the&#45;link&#45;element&#45;3"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element&#59; it creates a <a id="other&#45;link&#45;types&#58;hyperlink"></a>[hyperlink](#hyperlink)&#46;

<strong>External Resource</strong>

The keyword may be specified on a <a id="other&#45;link&#45;types&#58;the&#45;link&#45;element&#45;4"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> element&#59; it creates an <a id="other&#45;link&#45;types&#58;external&#45;resource&#45;link"></a>[external resource link](#external-resource-link)&#46;

<strong>Effect on&#46;&#46;&#46; <a id="other&#45;link&#45;types&#58;the&#45;a&#45;element"></a><code>[a](#the-a-element)</code> and <a id="other&#45;link&#45;types&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code></strong>

One of the following&#58;

<strong>Not allowed</strong>

The keyword must not be specified on <a id="other&#45;link&#45;types&#58;the&#45;a&#45;element&#45;2"></a><code>[a](#the-a-element)</code> and <a id="other&#45;link&#45;types&#58;the&#45;area&#45;element&#45;2"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements&#46;

<strong>Hyperlink</strong>

The keyword may be specified on <a id="other&#45;link&#45;types&#58;the&#45;a&#45;element&#45;3"></a><code>[a](#the-a-element)</code> and <a id="other&#45;link&#45;types&#58;the&#45;area&#45;element&#45;3"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements&#59; it creates a <a id="other&#45;link&#45;types&#58;hyperlink&#45;2"></a>[hyperlink](#hyperlink)&#46;

<strong>External Resource</strong>

The keyword may be specified on <a id="other&#45;link&#45;types&#58;the&#45;a&#45;element&#45;4"></a><code>[a](#the-a-element)</code> and <a id="other&#45;link&#45;types&#58;the&#45;area&#45;element&#45;4"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements&#59; it creates an <a id="other&#45;link&#45;types&#58;external&#45;resource&#45;link&#45;2"></a>[external resource link](#external-resource-link)&#46;

<strong>Hyperlink Annotation</strong>

The keyword may be specified on <a id="other&#45;link&#45;types&#58;the&#45;a&#45;element&#45;5"></a><code>[a](#the-a-element)</code> and <a id="other&#45;link&#45;types&#58;the&#45;area&#45;element&#45;5"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements&#59; it <a id="other&#45;link&#45;types&#58;hyperlink&#45;annotation"></a>[annotates](#hyperlink-annotation) other <a id="other&#45;link&#45;types&#58;hyperlink&#45;3"></a>[hyperlinks](#hyperlink) created by the element&#46;

<strong>Effect on&#46;&#46;&#46; <a id="other&#45;link&#45;types&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code></strong>

One of the following&#58;

<strong>Not allowed</strong>

The keyword must not be specified on <a id="other&#45;link&#45;types&#58;the&#45;form&#45;element&#45;2"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements&#46;

<strong>Hyperlink</strong>

The keyword may be specified on <a id="other&#45;link&#45;types&#58;the&#45;form&#45;element&#45;3"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements&#59; it creates a <a id="other&#45;link&#45;types&#58;hyperlink&#45;4"></a>[hyperlink](#hyperlink)&#46;

<strong>External Resource</strong>

The keyword may be specified on <a id="other&#45;link&#45;types&#58;the&#45;form&#45;element&#45;4"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements&#59; it creates an <a id="other&#45;link&#45;types&#58;external&#45;resource&#45;link&#45;3"></a>[external resource link](#external-resource-link)&#46;

<strong>Hyperlink Annotation</strong>

The keyword may be specified on <a id="other&#45;link&#45;types&#58;the&#45;form&#45;element&#45;5"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements&#59; it <a id="other&#45;link&#45;types&#58;hyperlink&#45;annotation&#45;2"></a>[annotates](#hyperlink-annotation) other <a id="other&#45;link&#45;types&#58;hyperlink&#45;5"></a>[hyperlinks](#hyperlink) created by the element&#46;

<strong>Brief description</strong>

A short non&#45;normative description of what the keyword&#39;s meaning is&#46;

<strong>Specification</strong>

A link to a more detailed description of the keyword&#39;s semantics and requirements&#46; It could be another page on the wiki&#44; or a link to an external page&#46;

<strong>Synonyms</strong>

A list of other keyword values that have exactly the same processing requirements&#46; Authors should not use the values defined to be synonyms&#44; they are only intended to allow user agents to support legacy content&#46; Anyone may remove synonyms that are not used in practice&#59; only names that need to be processed as synonyms for compatibility with legacy content are to be registered in this way&#46;

<strong>Status</strong>

One of the following&#58;

<strong>Proposed</strong>

The keyword has not received wide peer review and approval&#46; Someone has proposed it and is&#44; or soon will be&#44; using it&#46;

<strong>Ratified</strong>

The keyword has received wide peer review and approval&#46; It has a specification that unambiguously defines how to handle pages that use the keyword&#44; including when they use it in incorrect ways&#46;

<strong>Discontinued</strong>

The keyword has received wide peer review and it has been found wanting&#46; Existing pages are using this keyword&#44; but new pages should avoid it&#46; The &#34;brief description&#34; and &#34;specification&#34; entries will give details of what authors should use instead&#44; if anything&#46;

If a keyword is found to be redundant with existing values&#44; it should be removed and listed as a synonym for the existing value&#46;

If a keyword is registered in the &#34;proposed&#34; state for a period of a month or more without being used or specified&#44; then it may be removed from the registry&#46;

If a keyword is added with the &#34;proposed&#34; status and found to be redundant with existing values&#44; it should be removed and listed as a synonym for the existing value&#46; If a keyword is added with the &#34;proposed&#34; status and found to be harmful&#44; then it should be changed to &#34;discontinued&#34; status&#46;

Anyone can change the status at any time&#44; but should only do so in accordance with the definitions above&#46;

Conformance checkers must use the information given on the microformats page for existing rel values to establish if a value is allowed or not&#58; values defined in this specification or marked as &#34;proposed&#34; or &#34;ratified&#34; must be accepted when used on the elements for which they apply as described in the &#34;Effect on&#46;&#46;&#46;&#34; field&#44; whereas values marked as &#34;discontinued&#34; or not listed in either this specification or on the aforementioned page must be rejected as invalid&#46; Conformance checkers may cache this information &#40;e&#46;g&#46; for performance reasons or to avoid the use of unreliable network connectivity&#41;&#46;

When an author uses a new type not defined by either this specification or the wiki page&#44; conformance checkers should offer to add the value to the wiki&#44; with the details described above&#44; with the &#34;proposed&#34; status&#46;

Types defined as extensions in the [microformats page for existing rel values](https://microformats.org/wiki/existing-rel-values#HTML5_link_type_extensions) with the status &#34;proposed&#34; or &#34;ratified&#34; may be used with the <code>rel</code> attribute on <a id="other&#45;link&#45;types&#58;the&#45;link&#45;element&#45;5"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code>&#44; <a id="other&#45;link&#45;types&#58;the&#45;a&#45;element&#45;6"></a><code>[a](#the-a-element)</code>&#44; and <a id="other&#45;link&#45;types&#58;the&#45;area&#45;element&#45;6"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements in accordance to the &#34;Effect on&#46;&#46;&#46;&#34; field&#46; [&#91;MFREL&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsMFREL)

### <a id="edits"></a>4&#46;7 Edits[](#edits)

The <a id="edits&#58;the&#45;ins&#45;element"></a><code>[ins](#the-ins-element)</code> and <a id="edits&#58;the&#45;del&#45;element"></a><code>[del](#the-del-element)</code> elements represent edits to the document&#46;

#### <a id="the&#45;ins&#45;element"></a>4&#46;7&#46;1 The <strong><code>ins</code></strong> element[](#the-ins-element)

<strong>✔</strong>MDN

[Element&#47;ins](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ins)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;ins&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;ins&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;ins&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;ins&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;ins&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;ins&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;ins&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;ins&#45;element&#58;transparent"></a>[Transparent](html-language-foundations--877d6146487f--b55e73c31978.md#transparent)&#46;

<strong><a id="the&#45;ins&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;ins&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;ins&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<a id="the&#45;ins&#45;element&#58;attr&#45;mod&#45;cite"></a><code>[cite](#attr-mod-cite)</code> — Link to the source of the quotation or more information about the edit

<a id="the&#45;ins&#45;element&#58;attr&#45;mod&#45;datetime"></a><code>[datetime](#attr-mod-datetime)</code> — Date and &#40;optionally&#41; time of the change

<strong><a id="the&#45;ins&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-ins)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-ins)&#46;

<strong><a id="the&#45;ins&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;ins&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default) with attributes <a id="the&#45;ins&#45;element&#58;attr&#45;mod&#45;cite&#45;2"></a><code>[cite](#attr-mod-cite)</code>&#44; <a id="the&#45;ins&#45;element&#58;attr&#45;mod&#45;datetime&#45;2"></a><code>[datetime](#attr-mod-datetime)</code>&#46;

<strong><a id="the&#45;ins&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;ins&#45;element&#58;htmlmodelement"></a><code>[HTMLModElement](#htmlmodelement)</code>&#46;

The <a id="the&#45;ins&#45;element&#58;the&#45;ins&#45;element"></a><code>[ins](#the-ins-element)</code> element <a id="the&#45;ins&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) an addition to the document&#46;

<strong>Example&#58;</strong>

The following represents the addition of a single paragraph&#58;

``` text
<aside>
 <ins>
  <p> I like fruit. </p>
 </ins>
</aside>
```

As does the following&#44; because everything in the <a id="the&#45;ins&#45;element&#58;the&#45;aside&#45;element"></a><code>[aside](#the-aside-element)</code> element here counts as <a id="the&#45;ins&#45;element&#58;phrasing&#45;content&#45;2&#45;3"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) and therefore there is just one <a id="the&#45;ins&#45;element&#58;paragraph"></a>[paragraph](html-language-foundations--877d6146487f--b55e73c31978.md#paragraph)&#58;

``` text
<aside>
 <ins>
  Apples are <em>tasty</em>.
 </ins>
 <ins>
  So are pears.
 </ins>
</aside>
```

<a id="the&#45;ins&#45;element&#58;the&#45;ins&#45;element&#45;2"></a><code>[ins](#the-ins-element)</code> elements should not cross <a id="the&#45;ins&#45;element&#58;paragraph&#45;2"></a>[implied paragraph](html-language-foundations--877d6146487f--b55e73c31978.md#paragraph) boundaries&#46;

<strong>Example&#58;</strong>

The following example represents the addition of two paragraphs&#44; the second of which was inserted in two parts&#46; The first <a id="the&#45;ins&#45;element&#58;the&#45;ins&#45;element&#45;3"></a><code>[ins](#the-ins-element)</code> element in this example thus crosses a paragraph boundary&#44; which is considered poor form&#46;

``` text
<aside>
 <!-- don't do this -->
 <ins datetime="2005-03-16 00:00Z">
  <p> I like fruit. </p>
  Apples are <em>tasty</em>.
 </ins>
 <ins datetime="2007-12-19 00:00Z">
  So are pears.
 </ins>
</aside>
```

Here is a better way of marking this up&#46; It uses more elements&#44; but none of the elements cross implied paragraph boundaries&#46;

``` text
<aside>
 <ins datetime="2005-03-16 00:00Z">
  <p> I like fruit. </p>
 </ins>
 <ins datetime="2005-03-16 00:00Z">
  Apples are <em>tasty</em>.
 </ins>
 <ins datetime="2007-12-19 00:00Z">
  So are pears.
 </ins>
</aside>
```

#### <a id="the&#45;del&#45;element"></a>4&#46;7&#46;2 The <strong><code>del</code></strong> element[](#the-del-element)

<strong>✔</strong>MDN

[Element&#47;del](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/del)

Support in all current engines&#46;

Firefox1&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="the&#45;del&#45;element&#58;concept&#45;element&#45;categories"></a>[Categories](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-categories)&#58;</strong>

<a id="the&#45;del&#45;element&#58;flow&#45;content&#45;2"></a>[Flow content](html-language-foundations--877d6146487f--b55e73c31978.md#flow-content-2)&#46;

<a id="the&#45;del&#45;element&#58;phrasing&#45;content&#45;2"></a>[Phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2)&#46;

<a id="the&#45;del&#45;element&#58;palpable&#45;content&#45;2"></a>[Palpable content](html-language-foundations--877d6146487f--b55e73c31978.md#palpable-content-2)&#46;

<strong><a id="the&#45;del&#45;element&#58;concept&#45;element&#45;contexts"></a>[Contexts in which this element can be used](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-contexts)&#58;</strong>

Where <a id="the&#45;del&#45;element&#58;phrasing&#45;content&#45;2&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) is expected&#46;

<strong><a id="the&#45;del&#45;element&#58;concept&#45;element&#45;content&#45;model"></a>[Content model](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-content-model)&#58;</strong>

<a id="the&#45;del&#45;element&#58;transparent"></a>[Transparent](html-language-foundations--877d6146487f--b55e73c31978.md#transparent)&#46;

<strong><a id="the&#45;del&#45;element&#58;concept&#45;element&#45;tag&#45;omission"></a>[Tag omission in text&#47;html](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-tag-omission)&#58;</strong>

Neither tag is omissible&#46;

<strong><a id="the&#45;del&#45;element&#58;concept&#45;element&#45;attributes"></a>[Content attributes](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-attributes)&#58;</strong>

<a id="the&#45;del&#45;element&#58;global&#45;attributes"></a>[Global attributes](html-language-foundations--877d6146487f--b55e73c31978.md#global-attributes)

<a id="the&#45;del&#45;element&#58;attr&#45;mod&#45;cite"></a><code>[cite](#attr-mod-cite)</code> — Link to the source of the quotation or more information about the edit

<a id="the&#45;del&#45;element&#58;attr&#45;mod&#45;datetime"></a><code>[datetime](#attr-mod-datetime)</code> — Date and &#40;optionally&#41; time of the change

<strong><a id="the&#45;del&#45;element&#58;concept&#45;element&#45;accessibility&#45;considerations"></a>[Accessibility considerations](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-accessibility-considerations)&#58;</strong>

[For authors](https://w3c.github.io/html-aria/#el-del)&#46;

[For implementers](https://w3c.github.io/html-aam/#el-del)&#46;

<strong><a id="the&#45;del&#45;element&#58;concept&#45;element&#45;sanitization"></a>[Sanitization](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-sanitization)&#58;</strong>

<a id="the&#45;del&#45;element&#58;sanitizer&#45;category&#45;default"></a>[Default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-default) with attributes <a id="the&#45;del&#45;element&#58;attr&#45;mod&#45;cite&#45;2"></a><code>[cite](#attr-mod-cite)</code>&#44; <a id="the&#45;del&#45;element&#58;attr&#45;mod&#45;datetime&#45;2"></a><code>[datetime](#attr-mod-datetime)</code>&#46;

<strong><a id="the&#45;del&#45;element&#58;concept&#45;element&#45;dom"></a>[DOM interface](html-language-foundations--877d6146487f--b55e73c31978.md#concept-element-dom)&#58;</strong>

Uses <a id="the&#45;del&#45;element&#58;htmlmodelement"></a><code>[HTMLModElement](#htmlmodelement)</code>&#46;

The <a id="the&#45;del&#45;element&#58;the&#45;del&#45;element"></a><code>[del](#the-del-element)</code> element <a id="the&#45;del&#45;element&#58;represents"></a>[represents](html-language-foundations--877d6146487f--b55e73c31978.md#represents) a removal from the document&#46;

<a id="the&#45;del&#45;element&#58;the&#45;del&#45;element&#45;2"></a><code>[del](#the-del-element)</code> elements should not cross <a id="the&#45;del&#45;element&#58;paragraph"></a>[implied paragraph](html-language-foundations--877d6146487f--b55e73c31978.md#paragraph) boundaries&#46;

<strong>Example&#58;</strong>

The following shows a &#34;to do&#34; list where items that have been done are crossed&#45;off with the date and time of their completion&#46;

``` text
<h1>To Do</h1>
<ul>
 <li>Empty the dishwasher</li>
 <li><del datetime="2009-10-11T01:25-07:00">Watch Walter Lewin's lectures</del></li>
 <li><del datetime="2009-10-10T23:38-07:00">Download more tracks</del></li>
 <li>Buy a printer</li>
</ul>
```

#### <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements"></a>4&#46;7&#46;3 Attributes common to <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;the&#45;ins&#45;element"></a><code>[ins](#the-ins-element)</code> and <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;the&#45;del&#45;element"></a><code>[del](#the-del-element)</code> elements[](#attributes-common-to-ins-and-del-elements)

The <a id="attr&#45;mod&#45;cite"></a><strong><code>cite</code></strong> attribute may be used to specify the <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;the&#45;document&#39;s&#45;address"></a>[URL](https://dom.spec.whatwg.org/#concept-document-url) of a document that explains the change&#46; When that document is long&#44; for instance the minutes of a meeting&#44; authors are encouraged to include a <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;concept&#45;url&#45;fragment"></a>[fragment](https://url.spec.whatwg.org/#concept-url-fragment) pointing to the specific part of that document that discusses the change&#46;

If the <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;attr&#45;mod&#45;cite"></a><code>[cite](#attr-mod-cite)</code> attribute is present&#44; it must be a <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;valid&#45;url&#45;potentially&#45;surrounded&#45;by&#45;spaces"></a>[valid URL potentially surrounded by spaces](html-language-foundations--877d6146487f--b55e73c31978.md#valid-url-potentially-surrounded-by-spaces) that explains the change&#46; To obtain the corresponding citation link&#44; the value of the attribute must be <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;encoding&#45;parsing&#45;a&#45;url"></a>[parsed](html-language-foundations--877d6146487f--b55e73c31978.md#encoding-parsing-a-url) relative to the element&#39;s <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46; User agents may allow users to follow such citation links&#44; but they are primarily intended for private use &#40;e&#46;g&#46;&#44; by server&#45;side scripts collecting statistics about a site&#39;s edits&#41;&#44; not for readers&#46;

The <a id="attr&#45;mod&#45;datetime"></a><strong><code>datetime</code></strong> attribute may be used to specify the time and date of the change&#46;

If present&#44; the <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;attr&#45;mod&#45;datetime"></a><code>[datetime](#attr-mod-datetime)</code> attribute&#39;s value must be a <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;valid&#45;date&#45;string&#45;with&#45;optional&#45;time"></a>[valid date string with optional time](html-language-foundations--877d6146487f--b55e73c31978.md#valid-date-string-with-optional-time)&#46;

User agents must parse the <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;attr&#45;mod&#45;datetime&#45;2"></a><code>[datetime](#attr-mod-datetime)</code> attribute according to the <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;parse&#45;a&#45;date&#45;or&#45;time&#45;string"></a>[parse a date or time string](html-language-foundations--877d6146487f--b55e73c31978.md#parse-a-date-or-time-string) algorithm&#46; If that doesn&#39;t return a <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;concept&#45;date"></a>[date](html-language-foundations--877d6146487f--b55e73c31978.md#concept-date) or a <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;concept&#45;datetime"></a>[global date and time](html-language-foundations--877d6146487f--b55e73c31978.md#concept-datetime)&#44; then the modification has no associated timestamp &#40;the value is non&#45;conforming&#59; it is not a <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;valid&#45;date&#45;string&#45;with&#45;optional&#45;time&#45;2"></a>[valid date string with optional time](html-language-foundations--877d6146487f--b55e73c31978.md#valid-date-string-with-optional-time)&#41;&#46; Otherwise&#44; the modification is marked as having been made at the given <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;concept&#45;date&#45;2"></a>[date](html-language-foundations--877d6146487f--b55e73c31978.md#concept-date) or <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;concept&#45;datetime&#45;2"></a>[global date and time](html-language-foundations--877d6146487f--b55e73c31978.md#concept-datetime)&#46; If the given value is a <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;concept&#45;datetime&#45;3"></a>[global date and time](html-language-foundations--877d6146487f--b55e73c31978.md#concept-datetime)&#44; then user agents should use the associated time&#45;zone offset information to determine which time zone to present the given datetime in&#46;

This value may be shown to the user&#44; but it is primarily intended for private use&#46;

The <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;the&#45;ins&#45;element&#45;2"></a><code>[ins](#the-ins-element)</code> and <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;the&#45;del&#45;element&#45;2"></a><code>[del](#the-del-element)</code> elements must implement the <a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;htmlmodelement"></a><code>[HTMLModElement](#htmlmodelement)</code> interface&#58;

<strong>✔</strong>MDN

[HTMLModElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLModElement)

Support in all current engines&#46;

Firefox1&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer6&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

<a id="htmlmodelement"></a><a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;htmlelement"></a><a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;htmlconstructor"></a><a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;cereactions"></a><a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;xattr&#45;reflecturl"></a><a id="dom&#45;mod&#45;cite"></a><a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;cereactions&#45;2"></a><a id="attributes&#45;common&#45;to&#45;ins&#45;and&#45;del&#45;elements&#58;xattr&#45;reflect"></a><a id="dom&#45;mod&#45;datetime"></a>

``` text
[Exposed=Window]
interface HTMLModElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions, ReflectURL] attribute USVString cite;
  [CEReactions, Reflect] attribute DOMString dateTime;
};
```

#### <a id="edits&#45;and&#45;paragraphs"></a>4&#46;7&#46;4 Edits and paragraphs[](#edits-and-paragraphs)

<em>This section is non&#45;normative&#46;</em>

Since the <a id="edits&#45;and&#45;paragraphs&#58;the&#45;ins&#45;element"></a><code>[ins](#the-ins-element)</code> and <a id="edits&#45;and&#45;paragraphs&#58;the&#45;del&#45;element"></a><code>[del](#the-del-element)</code> elements do not affect <a id="edits&#45;and&#45;paragraphs&#58;paragraph"></a>[paragraphing](html-language-foundations--877d6146487f--b55e73c31978.md#paragraph)&#44; it is possible&#44; in some cases where paragraphs are <a id="edits&#45;and&#45;paragraphs&#58;paragraph&#45;2"></a>[implied](html-language-foundations--877d6146487f--b55e73c31978.md#paragraph) &#40;without explicit <a id="edits&#45;and&#45;paragraphs&#58;the&#45;p&#45;element"></a><code>[p](#the-p-element)</code> elements&#41;&#44; for an <a id="edits&#45;and&#45;paragraphs&#58;the&#45;ins&#45;element&#45;2"></a><code>[ins](#the-ins-element)</code> or <a id="edits&#45;and&#45;paragraphs&#58;the&#45;del&#45;element&#45;2"></a><code>[del](#the-del-element)</code> element to span both an entire paragraph or other non&#45;<a id="edits&#45;and&#45;paragraphs&#58;phrasing&#45;content&#45;2"></a>[phrasing content](html-language-foundations--877d6146487f--b55e73c31978.md#phrasing-content-2) elements and part of another paragraph&#46; For example&#58;

``` text
<section>
 <ins>
  <p>
   This is a paragraph that was inserted.
  </p>
  This is another paragraph whose first sentence was inserted
  at the same time as the paragraph above.
 </ins>
 This is a second sentence, which was there all along.
</section>
```

By only wrapping some paragraphs in <a id="edits&#45;and&#45;paragraphs&#58;the&#45;p&#45;element&#45;2"></a><code>[p](#the-p-element)</code> elements&#44; one can even get the end of one paragraph&#44; a whole second paragraph&#44; and the start of a third paragraph to be covered by the same <a id="edits&#45;and&#45;paragraphs&#58;the&#45;ins&#45;element&#45;3"></a><code>[ins](#the-ins-element)</code> or <a id="edits&#45;and&#45;paragraphs&#58;the&#45;del&#45;element&#45;3"></a><code>[del](#the-del-element)</code> element &#40;though this is very confusing&#44; and not considered good practice&#41;&#58;

``` text
<section>
 This is the first paragraph. <ins>This sentence was
 inserted.
 <p>This second paragraph was inserted.</p>
 This sentence was inserted too.</ins> This is the
 third paragraph in this example.
 <!-- (don't do this) -->
</section>
```

However&#44; due to the way <a id="edits&#45;and&#45;paragraphs&#58;paragraph&#45;3"></a>[implied paragraphs](html-language-foundations--877d6146487f--b55e73c31978.md#paragraph) are defined&#44; it is not possible to mark up the end of one paragraph and the start of the very next one using the same <a id="edits&#45;and&#45;paragraphs&#58;the&#45;ins&#45;element&#45;4"></a><code>[ins](#the-ins-element)</code> or <a id="edits&#45;and&#45;paragraphs&#58;the&#45;del&#45;element&#45;4"></a><code>[del](#the-del-element)</code> element&#46; You instead have to use one &#40;or two&#41; <a id="edits&#45;and&#45;paragraphs&#58;the&#45;p&#45;element&#45;3"></a><code>[p](#the-p-element)</code> element&#40;s&#41; and two <a id="edits&#45;and&#45;paragraphs&#58;the&#45;ins&#45;element&#45;5"></a><code>[ins](#the-ins-element)</code> or <a id="edits&#45;and&#45;paragraphs&#58;the&#45;del&#45;element&#45;5"></a><code>[del](#the-del-element)</code> elements&#44; as for example&#58;

``` text
<section>
 <p>This is the first paragraph. <del>This sentence was
 deleted.</del></p>
 <p><del>This sentence was deleted too.</del> That
 sentence needed a separate &lt;del&gt; element.</p>
</section>
```

Partly because of the confusion described above&#44; authors are strongly encouraged to always mark up all paragraphs with the <a id="edits&#45;and&#45;paragraphs&#58;the&#45;p&#45;element&#45;4"></a><code>[p](#the-p-element)</code> element&#44; instead of having <a id="edits&#45;and&#45;paragraphs&#58;the&#45;ins&#45;element&#45;6"></a><code>[ins](#the-ins-element)</code> or <a id="edits&#45;and&#45;paragraphs&#58;the&#45;del&#45;element&#45;6"></a><code>[del](#the-del-element)</code> elements that cross <a id="edits&#45;and&#45;paragraphs&#58;paragraph&#45;4"></a>[implied paragraphs](html-language-foundations--877d6146487f--b55e73c31978.md#paragraph) boundaries&#46;

#### <a id="edits&#45;and&#45;lists"></a>4&#46;7&#46;5 Edits and lists[](#edits-and-lists)

<em>This section is non&#45;normative&#46;</em>

The content models of the <a id="edits&#45;and&#45;lists&#58;the&#45;ol&#45;element"></a><code>[ol](#the-ol-element)</code> and <a id="edits&#45;and&#45;lists&#58;the&#45;ul&#45;element"></a><code>[ul](#the-ul-element)</code> elements do not allow <a id="edits&#45;and&#45;lists&#58;the&#45;ins&#45;element"></a><code>[ins](#the-ins-element)</code> and <a id="edits&#45;and&#45;lists&#58;the&#45;del&#45;element"></a><code>[del](#the-del-element)</code> elements as children&#46; Lists always represent all their items&#44; including items that would otherwise have been marked as deleted&#46;

To indicate that an item is inserted or deleted&#44; an <a id="edits&#45;and&#45;lists&#58;the&#45;ins&#45;element&#45;2"></a><code>[ins](#the-ins-element)</code> or <a id="edits&#45;and&#45;lists&#58;the&#45;del&#45;element&#45;2"></a><code>[del](#the-del-element)</code> element can be wrapped around the contents of the <a id="edits&#45;and&#45;lists&#58;the&#45;li&#45;element"></a><code>[li](#the-li-element)</code> element&#46; To indicate that an item has been replaced by another&#44; a single <a id="edits&#45;and&#45;lists&#58;the&#45;li&#45;element&#45;2"></a><code>[li](#the-li-element)</code> element can have one or more <a id="edits&#45;and&#45;lists&#58;the&#45;del&#45;element&#45;3"></a><code>[del](#the-del-element)</code> elements followed by one or more <a id="edits&#45;and&#45;lists&#58;the&#45;ins&#45;element&#45;3"></a><code>[ins](#the-ins-element)</code> elements&#46;

<strong>Example&#58;</strong>

In the following example&#44; a list that started empty had items added and removed from it over time&#46; The bits in the example that have been emphasized show the parts that are the &#34;current&#34; state of the list&#46; The list item numbers don&#39;t take into account the edits&#44; though&#46;

``` text
<h1>Stop-ship bugs</h1>
<ol>
 <li><ins datetime="2008-02-12T15:20Z">Bug 225:
 Rain detector doesn't work in snow</ins></li>
 <li><del datetime="2008-03-01T20:22Z"><ins datetime="2008-02-14T12:02Z">Bug 228:
 Water buffer overflows in April</ins></del></li>
 <li><ins datetime="2008-02-16T13:50Z">Bug 230:
 Water heater doesn't use renewable fuels</ins></li>
 <li><del datetime="2008-02-20T21:15Z"><ins datetime="2008-02-16T14:25Z">Bug 232:
 Carbon dioxide emissions detected after startup</ins></del></li>
</ol>
```

<strong>Example&#58;</strong>

In the following example&#44; a list that started with just fruit was replaced by a list with just colors&#46;

``` text
<h1>List of <del>fruits</del><ins>colors</ins></h1>
<ul>
 <li><del>Lime</del><ins>Green</ins></li>
 <li><del>Apple</del></li>
 <li>Orange</li>
 <li><del>Pear</del></li>
 <li><ins>Teal</ins></li>
 <li><del>Lemon</del><ins>Yellow</ins></li>
 <li>Olive</li>
 <li><ins>Purple</ins></li>
</ul>
```

#### <a id="edits&#45;and&#45;tables"></a>4&#46;7&#46;6 Edits and tables[](#edits-and-tables)

<em>This section is non&#45;normative&#46;</em>

The elements that form part of the table model have complicated content model requirements that do not allow for the <a id="edits&#45;and&#45;tables&#58;the&#45;ins&#45;element"></a><code>[ins](#the-ins-element)</code> and <a id="edits&#45;and&#45;tables&#58;the&#45;del&#45;element"></a><code>[del](#the-del-element)</code> elements&#44; so indicating edits to a table can be difficult&#46;

To indicate that an entire row or an entire column has been added or removed&#44; the entire contents of each cell in that row or column can be wrapped in <a id="edits&#45;and&#45;tables&#58;the&#45;ins&#45;element&#45;2"></a><code>[ins](#the-ins-element)</code> or <a id="edits&#45;and&#45;tables&#58;the&#45;del&#45;element&#45;2"></a><code>[del](#the-del-element)</code> elements &#40;respectively&#41;&#46;

<strong>Example&#58;</strong>

Here&#44; a table&#39;s row has been added&#58;

``` text
<table>
 <thead>
  <tr> <th> Game name           <th> Game publisher   <th> Verdict
 <tbody>
  <tr> <td> Diablo 2            <td> Blizzard         <td> 8/10
  <tr> <td> Portal              <td> Valve            <td> 10/10
  <tr> <td> <ins>Portal 2</ins> <td> <ins>Valve</ins> <td> <ins>10/10</ins>
</table>
```

Here&#44; a column has been removed &#40;the time at which it was removed is given also&#44; as is a link to the page explaining why&#41;&#58;

``` text
<table>
 <thead>
  <tr> <th> Game name           <th> Game publisher   <th> <del cite="/edits/r192" datetime="2011-05-02 14:23Z">Verdict</del>
 <tbody>
  <tr> <td> Diablo 2            <td> Blizzard         <td> <del cite="/edits/r192" datetime="2011-05-02 14:23Z">8/10</del>
  <tr> <td> Portal              <td> Valve            <td> <del cite="/edits/r192" datetime="2011-05-02 14:23Z">10/10</del>
  <tr> <td> Portal 2            <td> Valve            <td> <del cite="/edits/r192" datetime="2011-05-02 14:23Z">10/10</del>
</table>
```

Generally speaking&#44; there is no good way to indicate more complicated edits &#40;e&#46;g&#46; that a cell was removed&#44; moving all subsequent cells up or to the left&#41;&#46;
