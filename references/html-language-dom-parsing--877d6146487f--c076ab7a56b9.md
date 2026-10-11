# HTML Standard: dom parsing

Attribution and reformatting notice added for Surgeist on 2026-10-11. Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Original English standard and normative/informative labels remain authoritative. This partial format conversion supports the selected HTML language baseline; it is not a replacement specification or an implementation-conformance claim. No endorsement is implied.

Source: [HTML Standard commit snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/). Source revision `877d6146487f15b15bc8dd0e3c625063a37b456a` (9 October 2026); retrieved 11 October 2026 UTC. This immutable snapshot is the selected language baseline, independently of the html5ever implementation ingestion revision.

The exact [WHATWG LICENSE](../licenses/whatwg/LICENSE), also present in the HTML repository at this revision, has SHA-256 `85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`. Specification prose is Creative Commons Attribution 4.0 International; the BSD 3-Clause scope applies when portions are incorporated into code, not as an alternative license for this Markdown.

Complete original HTML: 14,930,614 bytes, SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`. Complete-section extraction input: 119,523 bytes, SHA-256 `c076ab7a56b988b8fb9cadca5697f0c2a73cbaded98a66fee3e153b2fad6b383`.

Retained complete heading sections, including all descendant clauses and bounded by the next equal-or-higher-level heading: `dom-parsing-and-serialization`, `enabling-and-disabling-scripting`. Original snapshot header is retained. The [edition map](SOURCE-EDITIONS.md#pinned-html-language-baseline) records the ten companion captures and bounded ownership. Every §4 element definition is retained across the collection. Omitted browser runtime chapters remain linked to this immutable snapshot; retaining JavaScript interfaces, networking, media, script execution or custom-element algorithms as source evidence does not assign their implementation to the native HTML crate. HTML syntax, author conformance and required parsing recovery remain distinct; malformed input is not rejected solely for an author-conformance error.

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

### <a id="dom&#45;parsing&#45;and&#45;serialization"></a>8&#46;5 DOM parsing and serialization APIs[](#dom-parsing-and-serialization)

<strong>✔</strong>MDN

[DOMParser](https://developer.mozilla.org/en-US/docs/Web/API/DOMParser)

Support in all current engines&#46;

Firefox1&#43;Safari1&#46;3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera8&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer9&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android10&#46;1&#43;

<a id="Element&#45;partial"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;cereactions"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;dom&#45;element&#45;sethtml"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;sethtmloptions"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;cereactions&#45;2"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;dom&#45;element&#45;sethtmlunsafe"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;tt&#45;trustedhtml"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;sethtmlunsafeoptions"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;tt&#45;trustedhtmlparseroptions"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;dom&#45;element&#45;gethtml"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;gethtmloptions"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;cereactions&#45;3"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;tt&#45;trustedhtml&#45;2"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;legacynulltoemptystring"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;dom&#45;element&#45;innerhtml"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;cereactions&#45;4"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;tt&#45;trustedhtml&#45;3"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;legacynulltoemptystring&#45;2"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;dom&#45;element&#45;outerhtml"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;cereactions&#45;5"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;dom&#45;element&#45;insertadjacenthtml"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;tt&#45;trustedhtml&#45;4"></a><a id="ShadowRoot&#45;partial"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;cereactions&#45;6"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;dom&#45;shadowroot&#45;sethtml"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;sethtmloptions&#45;2"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;cereactions&#45;7"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;dom&#45;shadowroot&#45;sethtmlunsafe"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;tt&#45;trustedhtml&#45;5"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;sethtmlunsafeoptions&#45;2"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;tt&#45;trustedhtmlparseroptions&#45;2"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;dom&#45;shadowroot&#45;gethtml"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;gethtmloptions&#45;2"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;cereactions&#45;8"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;tt&#45;trustedhtml&#45;6"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;legacynulltoemptystring&#45;3"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;dom&#45;shadowroot&#45;innerhtml"></a><a id="sanitizerpresets"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;dom&#45;sanitizerpresets&#45;default"></a><a id="sethtmloptions"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;sanitizer"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;sanitizerconfig"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;sanitizerpresets"></a><a id="dom&#45;sethtmloptions&#45;sanitizer"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;dom&#45;sanitizerpresets&#45;default&#45;2"></a><a id="sethtmlunsafeoptions"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;sanitizer&#45;2"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;sanitizerconfig&#45;2"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;sanitizerpresets&#45;2"></a><a id="dom&#45;sethtmlunsafeoptions&#45;sanitizer"></a><a id="dom&#45;sethtmlunsafeoptions&#45;runscripts"></a><a id="parsehtmlunsafeoptions"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;sanitizer&#45;3"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;sanitizerconfig&#45;3"></a><a id="dom&#45;parsing&#45;and&#45;serialization&#58;sanitizerpresets&#45;3"></a><a id="dom&#45;parsehtmlunsafeoptions&#45;sanitizer"></a><a id="gethtmloptions"></a><a id="dom&#45;gethtmloptions&#45;serializableshadowroots"></a><a id="dom&#45;gethtmloptions&#45;shadowroots"></a>

``` text
partial interface Element {
  [CEReactions] undefined setHTML(DOMString html, optional SetHTMLOptions options = {});
  [CEReactions] undefined setHTMLUnsafe((TrustedHTML or DOMString) html, optional (SetHTMLUnsafeOptions or TrustedHTMLParserOptions) options = {});
  DOMString getHTML(optional GetHTMLOptions options = {});

  [CEReactions] attribute (TrustedHTML or [LegacyNullToEmptyString] DOMString) innerHTML;
  [CEReactions] attribute (TrustedHTML or [LegacyNullToEmptyString] DOMString) outerHTML;
  [CEReactions] undefined insertAdjacentHTML(DOMString position, (TrustedHTML or DOMString) string);
};

partial interface ShadowRoot {
  [CEReactions] undefined setHTML(DOMString html, optional SetHTMLOptions options = {});
  [CEReactions] undefined setHTMLUnsafe((TrustedHTML or DOMString) html, optional (SetHTMLUnsafeOptions or TrustedHTMLParserOptions) options = {});
  DOMString getHTML(optional GetHTMLOptions options = {});

  [CEReactions] attribute (TrustedHTML or [LegacyNullToEmptyString] DOMString) innerHTML;
};

enum SanitizerPresets { "default" };
dictionary SetHTMLOptions {
  (Sanitizer or SanitizerConfig or SanitizerPresets) sanitizer = "default";
};
dictionary SetHTMLUnsafeOptions {
  (Sanitizer or SanitizerConfig or SanitizerPresets) sanitizer;
  boolean runScripts = false;
};
dictionary ParseHTMLUnsafeOptions {
  (Sanitizer or SanitizerConfig or SanitizerPresets) sanitizer;
};

dictionary GetHTMLOptions {
  boolean serializableShadowRoots = false;
  sequence<ShadowRoot> shadowRoots = [];
};
```

#### <a id="the&#45;domparser&#45;interface"></a>8&#46;5&#46;1 The <a id="the&#45;domparser&#45;interface&#58;domparser"></a><code>[DOMParser](#domparser)</code> interface[](#the-domparser-interface)

The <a id="the&#45;domparser&#45;interface&#58;domparser&#45;2"></a><code>[DOMParser](#domparser)</code> interface allows authors to create new <a id="the&#45;domparser&#45;interface&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> objects by parsing strings&#44; as either HTML or XML&#46;

<strong><code><var>parser</var> &#61; new <a id="dom&#45;domparser&#45;constructor&#45;dev"></a>[DOMParser](#dom-domparser-constructor)&#40;&#41;</code></strong>

<strong>✔</strong>MDN

[DOMParser&#47;DOMParser](https://developer.mozilla.org/en-US/docs/Web/API/DOMParser/DOMParser)

Support in all current engines&#46;

Firefox1&#43;Safari1&#46;3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera8&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer9&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android10&#46;1&#43;

Constructs a new <a id="the&#45;domparser&#45;interface&#58;domparser&#45;3"></a><code>[DOMParser](#domparser)</code> object&#46;

<strong><code><var>document</var> &#61; <var>parser</var>&#46;<a id="dom&#45;domparser&#45;parsefromstring&#45;dev"></a>[parseFromString](#dom-domparser-parsefromstring)&#40;<var>string</var>&#44; <var>type</var>&#41;</code></strong>

<strong>✔</strong>MDN

[DOMParser&#47;parseFromString](https://developer.mozilla.org/en-US/docs/Web/API/DOMParser/parseFromString)

Support in all current engines&#46;

Firefox1&#43;Safari1&#46;3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera8&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer9&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android10&#46;1&#43;

Parses <var>string</var> using either the HTML or XML parser&#44; according to <var>type</var>&#44; and returns the resulting <a id="the&#45;domparser&#45;interface&#58;document&#45;2"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#46; <var>type</var> can be &#34;<a id="the&#45;domparser&#45;interface&#58;text&#47;html"></a><code>[text&#47;html](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#text/html)</code>&#34; &#40;which will invoke the HTML parser&#41;&#44; or any of &#34;<a id="the&#45;domparser&#45;interface&#58;text&#47;xml"></a><code>[text&#47;xml](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#text/xml)</code>&#34;&#44; &#34;<a id="the&#45;domparser&#45;interface&#58;application&#47;xml"></a><code>[application&#47;xml](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#application/xml)</code>&#34;&#44; &#34;<a id="the&#45;domparser&#45;interface&#58;application&#47;xhtml&#43;xml"></a><code>[application&#47;xhtml&#43;xml](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#application/xhtml+xml)</code>&#34;&#44; or &#34;<a id="the&#45;domparser&#45;interface&#58;image&#47;svg&#43;xml"></a><code>[image&#47;svg&#43;xml](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#image/svg+xml)</code>&#34; &#40;which will invoke the XML parser&#41;&#46;

For the XML parser&#44; if <var>string</var> cannot be parsed&#44; then the returned <a id="the&#45;domparser&#45;interface&#58;document&#45;3"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> will contain elements describing the resulting error&#46;

Note that <a id="the&#45;domparser&#45;interface&#58;the&#45;script&#45;element"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> elements are not evaluated during parsing&#44; and the resulting document&#39;s <a id="the&#45;domparser&#45;interface&#58;document&#39;s&#45;character&#45;encoding"></a>[encoding](https://dom.spec.whatwg.org/#concept-document-encoding) will always be <a id="the&#45;domparser&#45;interface&#58;utf&#45;8"></a>[UTF&#45;8](https://encoding.spec.whatwg.org/#utf-8)&#46; The document&#39;s <a id="the&#45;domparser&#45;interface&#58;the&#45;document&#39;s&#45;address"></a>[URL](https://dom.spec.whatwg.org/#concept-document-url) will be inherited from <var>parser</var>&#39;s <a id="the&#45;domparser&#45;interface&#58;concept&#45;relevant&#45;global"></a>[relevant global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global)&#46;

Values other than the above for <var>type</var> will cause a <a id="the&#45;domparser&#45;interface&#58;typeerror"></a><code>[TypeError](https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-typeerror)</code> exception to be thrown&#46;

<strong>Note&#58;</strong>

The design of <a id="the&#45;domparser&#45;interface&#58;domparser&#45;4"></a><code>[DOMParser](#domparser)</code>&#44; as a class that needs to be constructed and then have its <a id="the&#45;domparser&#45;interface&#58;dom&#45;domparser&#45;parsefromstring"></a><code>[parseFromString&#40;&#41;](#dom-domparser-parsefromstring)</code> method called&#44; is an unfortunate historical artifact&#46; If we were designing this functionality today it would be a standalone function&#46; For parsing HTML&#44; the modern alternative is <a id="the&#45;domparser&#45;interface&#58;dom&#45;parsehtmlunsafe"></a><code>[Document&#46;parseHTMLUnsafe&#40;&#41;](#dom-parsehtmlunsafe)</code>&#46;

<strong>Warning&#58;</strong>

This method performs no sanitization to remove potentially&#45;dangerous elements and attributes like <a id="the&#45;domparser&#45;interface&#58;the&#45;script&#45;element&#45;2"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> or <a id="the&#45;domparser&#45;interface&#58;event&#45;handler&#45;content&#45;attributes"></a>[event handler content attributes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-handler-content-attributes)&#46;

<a id="domparser"></a><a id="the&#45;domparser&#45;interface&#58;dom&#45;domparser&#45;constructor"></a><a id="the&#45;domparser&#45;interface&#58;document&#45;4"></a><a id="the&#45;domparser&#45;interface&#58;dom&#45;domparser&#45;parsefromstring&#45;2"></a><a id="the&#45;domparser&#45;interface&#58;tt&#45;trustedhtml"></a><a id="the&#45;domparser&#45;interface&#58;domparsersupportedtype"></a><a id="domparsersupportedtype"></a><a id="the&#45;domparser&#45;interface&#58;dom&#45;domparsersupportedtype&#45;texthtml"></a><a id="the&#45;domparser&#45;interface&#58;dom&#45;domparsersupportedtype&#45;otherwise"></a><a id="the&#45;domparser&#45;interface&#58;dom&#45;domparsersupportedtype&#45;otherwise&#45;2"></a><a id="the&#45;domparser&#45;interface&#58;dom&#45;domparsersupportedtype&#45;otherwise&#45;3"></a><a id="the&#45;domparser&#45;interface&#58;dom&#45;domparsersupportedtype&#45;otherwise&#45;4"></a>

``` text
[Exposed=Window]
interface DOMParser {
  constructor();

  [NewObject] Document parseFromString((TrustedHTML or DOMString) string, DOMParserSupportedType type);
};

enum DOMParserSupportedType {
  "text/html",
  "text/xml",
  "application/xml",
  "application/xhtml+xml",
  "image/svg+xml"
};
```

The <a id="dom&#45;domparser&#45;constructor"></a><strong><code>new DOMParser&#40;&#41;</code></strong> constructor steps are to do nothing&#46;

The <a id="dom&#45;domparser&#45;parsefromstring"></a><strong><code>parseFromString&#40;<var>string</var>&#44; <var>type</var>&#41;</code></strong> method steps are&#58;

1.  Let <var>compliantString</var> be the result of invoking the <a id="the&#45;domparser&#45;interface&#58;tt&#45;getcompliantstring"></a>[get trusted type compliant string](https://w3c.github.io/trusted-types/dist/spec/#get-trusted-type-compliant-string) algorithm with <a id="the&#45;domparser&#45;interface&#58;tt&#45;trustedhtml&#45;2"></a><code>[TrustedHTML](https://w3c.github.io/trusted-types/dist/spec/#trustedhtml)</code>&#44; <a id="the&#45;domparser&#45;interface&#58;this"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;domparser&#45;interface&#58;concept&#45;relevant&#45;global&#45;2"></a>[relevant global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global)&#44; <var>string</var>&#44; &#34;<code>DOMParser parseFromString</code>&#34;&#44; and &#34;<code>script</code>&#34;&#46;

2.  Let <var>relevantDocument</var> be <a id="the&#45;domparser&#45;interface&#58;this&#45;2"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;domparser&#45;interface&#58;concept&#45;relevant&#45;global&#45;3"></a>[relevant global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global)&#39;s <a id="the&#45;domparser&#45;interface&#58;concept&#45;document&#45;window"></a>[associated <code>Document</code>](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-window)&#46;

3.  Let <var>document</var> be a new <a id="the&#45;domparser&#45;interface&#58;document&#45;5"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#44; whose <a id="the&#45;domparser&#45;interface&#58;concept&#45;document&#45;content&#45;type"></a>[content type](https://dom.spec.whatwg.org/#concept-document-content-type) is <var>type</var>&#44; <a id="the&#45;domparser&#45;interface&#58;concept&#45;document&#45;origin"></a>[origin](https://dom.spec.whatwg.org/#concept-document-origin) is <var>relevantDocument</var>&#39;s <a id="the&#45;domparser&#45;interface&#58;concept&#45;document&#45;origin&#45;2"></a>[origin](https://dom.spec.whatwg.org/#concept-document-origin)&#44; and <a id="the&#45;domparser&#45;interface&#58;the&#45;document&#39;s&#45;address&#45;2"></a>[URL](https://dom.spec.whatwg.org/#concept-document-url) is <var>relevantDocument</var>&#39;s <a id="the&#45;domparser&#45;interface&#58;the&#45;document&#39;s&#45;address&#45;3"></a>[URL](https://dom.spec.whatwg.org/#concept-document-url)&#46;

    <strong>Note&#58;</strong>

    The document&#39;s <a id="the&#45;domparser&#45;interface&#58;document&#39;s&#45;character&#45;encoding&#45;2"></a>[encoding](https://dom.spec.whatwg.org/#concept-document-encoding) will be left as its default&#44; of <a id="the&#45;domparser&#45;interface&#58;utf&#45;8&#45;2"></a>[UTF&#45;8](https://encoding.spec.whatwg.org/#utf-8)&#46; In particular&#44; any XML declarations or <a id="the&#45;domparser&#45;interface&#58;the&#45;meta&#45;element"></a><code>[meta](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-meta-element)</code> elements found while parsing <var>compliantString</var> will have no effect&#46;

4.  Switch on <var>type</var>&#58;

    <strong>&#34;<a id="dom&#45;domparsersupportedtype&#45;texthtml"></a><strong><code>text&#47;html</code></strong>&#34;</strong>

    1.  <a id="the&#45;domparser&#45;interface&#58;parse&#45;html&#45;from&#45;a&#45;string"></a>[Parse HTML from a string](#parse-html-from-a-string) given <var>document</var> and <var>compliantString</var>&#46;

    <strong>Note&#58;</strong>

    Since <var>document</var> does not have a <a id="the&#45;domparser&#45;interface&#58;concept&#45;document&#45;bc"></a>[browsing context](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc)&#44; <a id="the&#45;domparser&#45;interface&#58;concept&#45;n&#45;script"></a>[scripting is disabled](#concept-n-script)&#46;
    <strong><a id="dom&#45;domparsersupportedtype&#45;otherwise"></a><strong>Otherwise</strong></strong>

    1.  Create an <a id="the&#45;domparser&#45;interface&#58;xml&#45;parser"></a>[XML parser](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#xml-parser) <var>parser</var>&#44; associated with <var>document</var>&#44; and with <a id="the&#45;domparser&#45;interface&#58;xml&#45;scripting&#45;support&#45;disabled"></a>[XML scripting support disabled](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#xml-scripting-support-disabled)&#46;

    2.  Parse <var>compliantString</var> using <var>parser</var>&#46;

    3.  If the previous step resulted in an XML well&#45;formedness or XML namespace well&#45;formedness error&#58;

        1.  <a id="the&#45;domparser&#45;interface&#58;assert"></a>[Assert](https://infra.spec.whatwg.org/#assert)&#58; <var>document</var> has no child nodes&#46;

        2.  Let <var>root</var> be the result of <a id="the&#45;domparser&#45;interface&#58;create&#45;an&#45;element"></a>[creating an element](https://dom.spec.whatwg.org/#concept-create-element) given <var>document</var>&#44; &#34;<code>parsererror</code>&#34;&#44; and &#34;<code>http&#58;&#47;&#47;www&#46;mozilla&#46;org&#47;newlayout&#47;xml&#47;parsererror&#46;xml</code>&#34;&#46;

        3.  Optionally&#44; add attributes or children to <var>root</var> to describe the nature of the parsing error&#46;

        4.  <a id="the&#45;domparser&#45;interface&#58;concept&#45;node&#45;append"></a>[Append](https://dom.spec.whatwg.org/#concept-node-append) <var>root</var> to <var>document</var>&#46;

5.  Return <var>document</var>&#46;

To <a id="parse&#45;html&#45;from&#45;a&#45;string"></a><strong>parse HTML from a string</strong>&#44; given a <a id="the&#45;domparser&#45;interface&#58;document&#45;6"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> <var>document</var> and a <a id="the&#45;domparser&#45;interface&#58;string"></a>[string](https://infra.spec.whatwg.org/#string) <var>html</var>&#58;

1.  Set <var>document</var>&#39;s <a id="the&#45;domparser&#45;interface&#58;concept&#45;document&#45;type"></a>[type](https://dom.spec.whatwg.org/#concept-document-type) to &#34;<code>html</code>&#34;&#46;

2.  Let <var>parser</var> be a new <a id="the&#45;domparser&#45;interface&#58;html&#45;parser"></a>[HTML parser](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#html-parser) whose <a id="the&#45;domparser&#45;interface&#58;allow&#45;declarative&#45;shadow&#45;roots"></a>[allow declarative shadow roots](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#allow-declarative-shadow-roots) is <var>document</var>&#39;s <a id="the&#45;domparser&#45;interface&#58;concept&#45;document&#45;allow&#45;declarative&#45;shadow&#45;roots"></a>[allow declarative shadow roots](https://dom.spec.whatwg.org/#concept-document-allow-declarative-shadow-roots)&#44; associated with <var>document</var>&#46;

3.  Place <var>html</var> into the <a id="the&#45;domparser&#45;interface&#58;input&#45;stream"></a>[input stream](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#input-stream) for <var>parser</var>&#46; The encoding <a id="the&#45;domparser&#45;interface&#58;concept&#45;encoding&#45;confidence"></a>[confidence](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#concept-encoding-confidence) is <em>irrelevant</em>&#46;

4.  Start <var>parser</var> and let it run until it has consumed all the characters just inserted into the input stream&#46;

    <strong>Note&#58;</strong>

    This might mutate the document&#39;s <a id="the&#45;domparser&#45;interface&#58;concept&#45;document&#45;mode"></a>[mode](https://dom.spec.whatwg.org/#concept-document-mode)&#46;

#### <a id="html&#45;parsing&#45;methods"></a>8&#46;5&#46;2 HTML parsing methods[](#html-parsing-methods)

<strong><code><var>element</var>&#46;<a id="dom&#45;element&#45;sethtml&#45;dev"></a>[setHTML](#dom-element-sethtml)&#40;<var>html</var>&#44; <var>options</var>&#41;</code></strong>

Parses <var>html</var> using the HTML parser with options <var>options</var>&#44; and replaces the children of <var>element</var> with the result&#46; <var>element</var> provides context for the HTML parser&#46; The parsed fragment is <a id="html&#45;parsing&#45;methods&#58;sanitize"></a>[sanitized](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitize) based on <var>options</var>&#39;s <a id="html&#45;parsing&#45;methods&#58;dom&#45;sethtmloptions&#45;sanitizer"></a><code>[sanitizer](#dom-sethtmloptions-sanitizer)</code> member&#44; and <a id="html&#45;parsing&#45;methods&#58;dom&#45;sanitizer&#45;removeunsafe"></a>[unsafe content is removed](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-sanitizer-removeunsafe)&#46;

<strong><code><var>shadowRoot</var>&#46;<a id="dom&#45;shadowroot&#45;sethtml&#45;dev"></a>[setHTML](#dom-shadowroot-sethtml)&#40;<var>html</var>&#44; <var>options</var>&#41;</code></strong>

Parses <var>html</var> using the HTML parser with options <var>options</var>&#44; and replaces the children of <var>shadowRoot</var> with the result&#46; <var>shadowRoot</var>&#39;s <a id="html&#45;parsing&#45;methods&#58;concept&#45;documentfragment&#45;host"></a>[host](https://dom.spec.whatwg.org/#concept-documentfragment-host) provides context for the HTML parser&#46; The parsed fragment is <a id="html&#45;parsing&#45;methods&#58;sanitize&#45;2"></a>[sanitized](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitize) based on <var>options</var>&#39;s <a id="html&#45;parsing&#45;methods&#58;dom&#45;sethtmloptions&#45;sanitizer&#45;2"></a><code>[sanitizer](#dom-sethtmloptions-sanitizer)</code> member&#44; and <a id="html&#45;parsing&#45;methods&#58;dom&#45;sanitizer&#45;removeunsafe&#45;2"></a>[unsafe content is removed](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-sanitizer-removeunsafe)&#46;

<strong><code><var>element</var>&#46;<a id="dom&#45;element&#45;sethtmlunsafe&#45;dev"></a>[setHTMLUnsafe](#dom-element-sethtmlunsafe)&#40;<var>html</var>&#44; <var>options</var>&#41;</code></strong>

Parses <var>html</var> using the HTML parser with options <var>options</var>&#44; and replaces the children of <var>element</var> with the result&#46; <var>element</var> provides context for the HTML parser&#46; If <var>options</var> is a <a id="html&#45;parsing&#45;methods&#58;sethtmlunsafeoptions"></a><code>[SetHTMLUnsafeOptions](#sethtmlunsafeoptions)</code> dictionary containing a <a id="html&#45;parsing&#45;methods&#58;dom&#45;sethtmlunsafeoptions&#45;sanitizer"></a><code>[sanitizer](#dom-sethtmlunsafeoptions-sanitizer)</code> member or a <a id="html&#45;parsing&#45;methods&#58;tt&#45;trustedhtmlparseroptions"></a><code>[TrustedHTMLParserOptions](https://w3c.github.io/trusted-types/dist/spec/#trusted-html-parser-options)</code> object with a non&#45;null sanitizer configuration&#44; it is used to <a id="html&#45;parsing&#45;methods&#58;sanitize&#45;3"></a>[sanitize](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitize) the parsed fragment before it is inserted into <var>element</var>&#46; If <var>options</var>&#39;s <a id="html&#45;parsing&#45;methods&#58;dom&#45;sethtmlunsafeoptions&#45;runscripts"></a><code>[runScripts](#dom-sethtmlunsafeoptions-runscripts)</code> is true&#44; scripts contained in <var>html</var> will be executed immediately after the node tree is updated &#40;unless altered by a Trusted Types default policy&#41;&#46; If Trusted Types are enforced and <var>options</var> is not a <a id="html&#45;parsing&#45;methods&#58;tt&#45;trustedhtmlparseroptions&#45;2"></a><code>[TrustedHTMLParserOptions](https://w3c.github.io/trusted-types/dist/spec/#trusted-html-parser-options)</code> object&#44; throws a <a id="html&#45;parsing&#45;methods&#58;typeerror"></a><code>[TypeError](https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-typeerror)</code> if a default policy&#39;s <code>createParserOptions</code> returns null or undefined&#44; or if no default policy defines <code>createParserOptions</code> and either <var>options</var> enables <a id="html&#45;parsing&#45;methods&#58;dom&#45;sethtmlunsafeoptions&#45;runscripts&#45;2"></a><code>[runScripts](#dom-sethtmlunsafeoptions-runscripts)</code> or specifies a non&#45;empty <a id="html&#45;parsing&#45;methods&#58;dom&#45;sethtmlunsafeoptions&#45;sanitizer&#45;2"></a><code>[sanitizer](#dom-sethtmlunsafeoptions-sanitizer)</code>&#44; or <var>html</var> is not a <a id="html&#45;parsing&#45;methods&#58;tt&#45;trustedhtml"></a><code>[TrustedHTML](https://w3c.github.io/trusted-types/dist/spec/#trustedhtml)</code> object &#40;unless a default policy&#39;s <code>createHTML</code> returns one&#41;&#46; Exceptions thrown by a default policy are rethrown&#46;

<strong><code><var>shadowRoot</var>&#46;<a id="dom&#45;shadowroot&#45;sethtmlunsafe&#45;dev"></a>[setHTMLUnsafe](#dom-shadowroot-sethtmlunsafe)&#40;<var>html</var>&#44; <var>options</var>&#41;</code></strong>

Parses <var>html</var> using the HTML parser with options <var>options</var>&#44; and replaces the children of <var>shadowRoot</var> with the result&#46; <var>shadowRoot</var>&#39;s <a id="html&#45;parsing&#45;methods&#58;concept&#45;documentfragment&#45;host&#45;2"></a>[host](https://dom.spec.whatwg.org/#concept-documentfragment-host) provides context for the HTML parser&#46; If <var>options</var> is a <a id="html&#45;parsing&#45;methods&#58;sethtmlunsafeoptions&#45;2"></a><code>[SetHTMLUnsafeOptions](#sethtmlunsafeoptions)</code> dictionary containing a <a id="html&#45;parsing&#45;methods&#58;dom&#45;sethtmlunsafeoptions&#45;sanitizer&#45;3"></a><code>[sanitizer](#dom-sethtmlunsafeoptions-sanitizer)</code> member or a <a id="html&#45;parsing&#45;methods&#58;tt&#45;trustedhtmlparseroptions&#45;3"></a><code>[TrustedHTMLParserOptions](https://w3c.github.io/trusted-types/dist/spec/#trusted-html-parser-options)</code> object with a non&#45;null sanitizer configuration&#44; it is used to <a id="html&#45;parsing&#45;methods&#58;sanitize&#45;4"></a>[sanitize](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitize) the parsed fragment before it is inserted into <var>shadowRoot</var>&#46; If <var>options</var>&#39;s <a id="html&#45;parsing&#45;methods&#58;dom&#45;sethtmlunsafeoptions&#45;runscripts&#45;3"></a><code>[runScripts](#dom-sethtmlunsafeoptions-runscripts)</code> is true&#44; scripts contained in <var>html</var> will be executed immediately after the node tree is updated &#40;unless altered by a Trusted Types default policy&#41;&#46; If Trusted Types are enforced and <var>options</var> is not a <a id="html&#45;parsing&#45;methods&#58;tt&#45;trustedhtmlparseroptions&#45;4"></a><code>[TrustedHTMLParserOptions](https://w3c.github.io/trusted-types/dist/spec/#trusted-html-parser-options)</code> object&#44; throws a <a id="html&#45;parsing&#45;methods&#58;typeerror&#45;2"></a><code>[TypeError](https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-typeerror)</code> if a default policy&#39;s <code>createParserOptions</code> returns null or undefined&#44; or if no default policy defines <code>createParserOptions</code> and either <var>options</var> enables <a id="html&#45;parsing&#45;methods&#58;dom&#45;sethtmlunsafeoptions&#45;runscripts&#45;4"></a><code>[runScripts](#dom-sethtmlunsafeoptions-runscripts)</code> or specifies a non&#45;empty <a id="html&#45;parsing&#45;methods&#58;dom&#45;sethtmlunsafeoptions&#45;sanitizer&#45;4"></a><code>[sanitizer](#dom-sethtmlunsafeoptions-sanitizer)</code>&#44; or <var>html</var> is not a <a id="html&#45;parsing&#45;methods&#58;tt&#45;trustedhtml&#45;2"></a><code>[TrustedHTML](https://w3c.github.io/trusted-types/dist/spec/#trustedhtml)</code> object &#40;unless a default policy&#39;s <code>createHTML</code> returns one&#41;&#46; Exceptions thrown by a default policy are rethrown&#46;

<strong><code><var>doc</var> &#61; Document&#46;<a id="dom&#45;parsehtml&#45;dev"></a>[parseHTML](#dom-parsehtml)&#40;<var>html</var>&#44; <var>options</var>&#41;</code></strong>

Parses <var>html</var> using the HTML parser with options <var>options</var>&#44; and returns a new <a id="html&#45;parsing&#45;methods&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> containing the result&#46; The resulting document is <a id="html&#45;parsing&#45;methods&#58;sanitize&#45;5"></a>[sanitized](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitize) based on <var>options</var>&#39;s <a id="html&#45;parsing&#45;methods&#58;dom&#45;sethtmloptions&#45;sanitizer&#45;3"></a><code>[sanitizer](#dom-sethtmloptions-sanitizer)</code> member&#44; and <a id="html&#45;parsing&#45;methods&#58;dom&#45;sanitizer&#45;removeunsafe&#45;3"></a>[unsafe content is removed](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-sanitizer-removeunsafe)&#46;

<strong><code><var>doc</var> &#61; Document&#46;<a id="html&#45;parsing&#45;methods&#58;dom&#45;parsehtmlunsafe"></a>[parseHTMLUnsafe](#dom-parsehtmlunsafe)&#40;<var>html</var>&#44; <var>options</var>&#41;</code></strong>

Parses <var>html</var> using the HTML parser with options <var>options</var>&#44; and returns the resulting <a id="html&#45;parsing&#45;methods&#58;document&#45;2"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#46;

Note that <a id="html&#45;parsing&#45;methods&#58;the&#45;script&#45;element"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> elements are not evaluated during parsing&#44; and the resulting document&#39;s <a id="html&#45;parsing&#45;methods&#58;document&#39;s&#45;character&#45;encoding"></a>[encoding](https://dom.spec.whatwg.org/#concept-document-encoding) will always be <a id="html&#45;parsing&#45;methods&#58;utf&#45;8"></a>[UTF&#45;8](https://encoding.spec.whatwg.org/#utf-8)&#46; The document&#39;s <a id="html&#45;parsing&#45;methods&#58;the&#45;document&#39;s&#45;address"></a>[URL](https://dom.spec.whatwg.org/#concept-document-url) will be <a id="html&#45;parsing&#45;methods&#58;about&#58;blank"></a><code>[about&#58;blank](html-language-foundations--877d6146487f--b55e73c31978.md#about:blank)</code>&#46; If <var>options</var> is a <a id="html&#45;parsing&#45;methods&#58;parsehtmlunsafeoptions"></a><code>[ParseHTMLUnsafeOptions](#parsehtmlunsafeoptions)</code> dictionary containing a <a id="html&#45;parsing&#45;methods&#58;dom&#45;parsehtmlunsafeoptions&#45;sanitizer"></a><code>[sanitizer](#dom-parsehtmlunsafeoptions-sanitizer)</code> member or a <a id="html&#45;parsing&#45;methods&#58;tt&#45;trustedhtmlparseroptions&#45;5"></a><code>[TrustedHTMLParserOptions](https://w3c.github.io/trusted-types/dist/spec/#trusted-html-parser-options)</code> object with a non&#45;null sanitizer configuration&#44; it is used to <a id="html&#45;parsing&#45;methods&#58;sanitize&#45;6"></a>[sanitize](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitize) the resulting DOM&#46; If Trusted Types are enforced and <var>options</var> is not a <a id="html&#45;parsing&#45;methods&#58;tt&#45;trustedhtmlparseroptions&#45;6"></a><code>[TrustedHTMLParserOptions](https://w3c.github.io/trusted-types/dist/spec/#trusted-html-parser-options)</code> object&#44; throws a <a id="html&#45;parsing&#45;methods&#58;typeerror&#45;3"></a><code>[TypeError](https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-typeerror)</code> if a default policy&#39;s <code>createParserOptions</code> returns null or undefined&#44; or if no default policy defines <code>createParserOptions</code> and either <var>options</var> specifies a non&#45;empty <a id="html&#45;parsing&#45;methods&#58;dom&#45;parsehtmlunsafeoptions&#45;sanitizer&#45;2"></a><code>[sanitizer](#dom-parsehtmlunsafeoptions-sanitizer)</code> or <var>html</var> is not a <a id="html&#45;parsing&#45;methods&#58;tt&#45;trustedhtml&#45;3"></a><code>[TrustedHTML](https://w3c.github.io/trusted-types/dist/spec/#trustedhtml)</code> object &#40;unless a default policy&#39;s <code>createHTML</code> returns one&#41;&#46; Exceptions thrown by a default policy are rethrown&#46;

<strong>Warning&#58;</strong>

By default&#44; the methods with an <code>Unsafe</code> suffix perform no sanitization to remove potentially&#45;dangerous elements and attributes like <a id="html&#45;parsing&#45;methods&#58;the&#45;script&#45;element&#45;2"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> or <a id="html&#45;parsing&#45;methods&#58;event&#45;handler&#45;content&#45;attributes"></a>[event handler content attributes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-handler-content-attributes)&#46;

<a id="html&#45;parsing&#45;methods&#58;element"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code>&#39;s <a id="dom&#45;element&#45;sethtml"></a><strong><code>setHTML&#40;<var>html</var>&#44; <var>options</var>&#41;</code></strong> method steps are&#58;

1.  Let <var>target</var> be <a id="html&#45;parsing&#45;methods&#58;this"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="html&#45;parsing&#45;methods&#58;template&#45;contents"></a>[template contents](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#template-contents) if <a id="html&#45;parsing&#45;methods&#58;this&#45;2"></a>[this](https://webidl.spec.whatwg.org/#this) is a <a id="html&#45;parsing&#45;methods&#58;the&#45;template&#45;element"></a><code>[template](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-template-element)</code> element&#59; otherwise <a id="html&#45;parsing&#45;methods&#58;this&#45;3"></a>[this](https://webidl.spec.whatwg.org/#this)&#46;

2.  <a id="html&#45;parsing&#45;methods&#58;set&#45;and&#45;filter&#45;html"></a>[Set and filter HTML](#set-and-filter-html) given <var>target</var>&#44; <var>html</var>&#44; <var>options</var>&#44; and <a id="html&#45;parsing&#45;methods&#58;fragment&#45;parser&#45;mode&#45;safe"></a>[Safe](#fragment-parser-mode-safe)&#46;

<a id="html&#45;parsing&#45;methods&#58;shadowroot"></a><code>[ShadowRoot](https://dom.spec.whatwg.org/#interface-shadowroot)</code>&#39;s <a id="dom&#45;shadowroot&#45;sethtml"></a><strong><code>setHTML&#40;<var>html</var>&#44; <var>options</var>&#41;</code></strong> method steps are&#58;

1.  <a id="html&#45;parsing&#45;methods&#58;set&#45;and&#45;filter&#45;html&#45;2"></a>[Set and filter HTML](#set-and-filter-html) given <a id="html&#45;parsing&#45;methods&#58;this&#45;4"></a>[this](https://webidl.spec.whatwg.org/#this)&#44; <var>html</var>&#44; <var>options</var>&#44; and <a id="html&#45;parsing&#45;methods&#58;fragment&#45;parser&#45;mode&#45;safe&#45;2"></a>[Safe](#fragment-parser-mode-safe)&#46;

<a id="html&#45;parsing&#45;methods&#58;element&#45;2"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code>&#39;s <a id="dom&#45;element&#45;sethtmlunsafe"></a><strong><code>setHTMLUnsafe&#40;<var>html</var>&#44; <var>options</var>&#41;</code></strong> method steps are&#58;

1.  Let &#40;<var>compliantHTML</var>&#44; <var>compliantOptions</var>&#41; be the result of invoking the <a id="html&#45;parsing&#45;methods&#58;tt&#45;getcompliantinput"></a>[get trusted type compliant input](https://w3c.github.io/trusted-types/dist/spec/#get-trusted-type-compliant-input) algorithm with <a id="html&#45;parsing&#45;methods&#58;this&#45;5"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="html&#45;parsing&#45;methods&#58;concept&#45;relevant&#45;global"></a>[relevant global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global)&#44; <var>html</var>&#44; <var>options</var>&#44; and &#34;<code>Element setHTMLUnsafe</code>&#34;&#46;

2.  Let <var>target</var> be <a id="html&#45;parsing&#45;methods&#58;this&#45;6"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="html&#45;parsing&#45;methods&#58;template&#45;contents&#45;2"></a>[template contents](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#template-contents) if <a id="html&#45;parsing&#45;methods&#58;this&#45;7"></a>[this](https://webidl.spec.whatwg.org/#this) is a <a id="html&#45;parsing&#45;methods&#58;the&#45;template&#45;element&#45;2"></a><code>[template](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-template-element)</code> element&#59; otherwise <a id="html&#45;parsing&#45;methods&#58;this&#45;8"></a>[this](https://webidl.spec.whatwg.org/#this)&#46;

3.  <a id="html&#45;parsing&#45;methods&#58;set&#45;and&#45;filter&#45;html&#45;3"></a>[Set and filter HTML](#set-and-filter-html) given <var>target</var>&#44; <var>compliantHTML</var>&#44; <var>compliantOptions</var>&#44; and <a id="html&#45;parsing&#45;methods&#58;fragment&#45;parser&#45;mode&#45;unsafe"></a>[Unsafe](#fragment-parser-mode-unsafe)&#46;

<a id="html&#45;parsing&#45;methods&#58;shadowroot&#45;2"></a><code>[ShadowRoot](https://dom.spec.whatwg.org/#interface-shadowroot)</code>&#39;s <a id="dom&#45;shadowroot&#45;sethtmlunsafe"></a><strong><code>setHTMLUnsafe&#40;<var>html</var>&#44; <var>options</var>&#41;</code></strong> method steps are&#58;

1.  Let &#40;<var>compliantHTML</var>&#44; <var>compliantOptions</var>&#41; be the result of invoking the <a id="html&#45;parsing&#45;methods&#58;tt&#45;getcompliantinput&#45;2"></a>[get trusted type compliant input](https://w3c.github.io/trusted-types/dist/spec/#get-trusted-type-compliant-input) algorithm with <a id="html&#45;parsing&#45;methods&#58;this&#45;9"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="html&#45;parsing&#45;methods&#58;concept&#45;relevant&#45;global&#45;2"></a>[relevant global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global)&#44; <var>html</var>&#44; <var>options</var>&#44; and &#34;<code>ShadowRoot setHTMLUnsafe</code>&#34;&#46;

2.  <a id="html&#45;parsing&#45;methods&#58;set&#45;and&#45;filter&#45;html&#45;4"></a>[Set and filter HTML](#set-and-filter-html) given <a id="html&#45;parsing&#45;methods&#58;this&#45;10"></a>[this](https://webidl.spec.whatwg.org/#this)&#44; <var>compliantHTML</var>&#44; <var>compliantOptions</var>&#44; and <a id="html&#45;parsing&#45;methods&#58;fragment&#45;parser&#45;mode&#45;unsafe&#45;2"></a>[Unsafe](#fragment-parser-mode-unsafe)&#46;

------------------------------------------------------------------------

The static <a id="dom&#45;parsehtml"></a><strong><code>parseHTML&#40;<var>html</var>&#44; <var>options</var>&#41;</code></strong> method steps are&#58;

1.  Let <var>document</var> be a new <a id="html&#45;parsing&#45;methods&#58;document&#45;3"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#44; whose <a id="html&#45;parsing&#45;methods&#58;concept&#45;document&#45;content&#45;type"></a>[content type](https://dom.spec.whatwg.org/#concept-document-content-type) is &#34;<code>text&#47;html</code>&#34; and <a id="html&#45;parsing&#45;methods&#58;concept&#45;document&#45;origin"></a>[origin](https://dom.spec.whatwg.org/#concept-document-origin) is <a id="html&#45;parsing&#45;methods&#58;this&#45;11"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="html&#45;parsing&#45;methods&#58;concept&#45;relevant&#45;global&#45;3"></a>[relevant global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global)&#39;s <a id="html&#45;parsing&#45;methods&#58;concept&#45;document&#45;window"></a>[associated <code>Document</code>](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-window)&#39;s <a id="html&#45;parsing&#45;methods&#58;concept&#45;document&#45;origin&#45;2"></a>[origin](https://dom.spec.whatwg.org/#concept-document-origin)&#46;

    <strong>Note&#58;</strong>

    Since <var>document</var> does not have a <a id="html&#45;parsing&#45;methods&#58;concept&#45;document&#45;bc"></a>[browsing context](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc)&#44; <a id="html&#45;parsing&#45;methods&#58;concept&#45;n&#45;script"></a>[scripting is disabled](#concept-n-script)&#46;

2.  Set <var>document</var>&#39;s <a id="html&#45;parsing&#45;methods&#58;concept&#45;document&#45;allow&#45;declarative&#45;shadow&#45;roots"></a>[allow declarative shadow roots](https://dom.spec.whatwg.org/#concept-document-allow-declarative-shadow-roots) to true&#46;

3.  <a id="html&#45;parsing&#45;methods&#58;parse&#45;html&#45;from&#45;a&#45;string"></a>[Parse HTML from a string](#parse-html-from-a-string) given <var>document</var> and <var>html</var>&#46;

4.  Let <var>sanitizer</var> be the result of <a id="html&#45;parsing&#45;methods&#58;get&#45;a&#45;sanitizer&#45;instance&#45;from&#45;options"></a>[getting a sanitizer instance from options](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#get-a-sanitizer-instance-from-options) given <var>options</var> and true&#46;

5.  <a id="html&#45;parsing&#45;methods&#58;assert"></a>[Assert](https://infra.spec.whatwg.org/#assert)&#58; <var>sanitizer</var> is non&#45;null&#46;

    <strong>Note&#58;</strong>

    The <a id="html&#45;parsing&#45;methods&#58;dom&#45;sethtmloptions&#45;sanitizer&#45;4"></a><code>[sanitizer](#dom-sethtmloptions-sanitizer)</code> member of <a id="html&#45;parsing&#45;methods&#58;sethtmloptions"></a><code>[SetHTMLOptions](#sethtmloptions)</code> defaults to &#34;<a id="html&#45;parsing&#45;methods&#58;dom&#45;sanitizerpresets&#45;default"></a><code>[default](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-sanitizerpresets-default)</code>&#34;&#46;

6.  <a id="html&#45;parsing&#45;methods&#58;sanitize&#45;7"></a>[Sanitize](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitize) <var>document</var> with <var>sanitizer</var> and true&#46;

7.  Return <var>document</var>&#46;

The static <a id="dom&#45;parsehtmlunsafe"></a><strong><code>parseHTMLUnsafe&#40;<var>html</var>&#44; <var>options</var>&#41;</code></strong> method steps are&#58;

1.  Let &#40;<var>compliantHTML</var>&#44; <var>compliantOptions</var>&#41; be the result of invoking the <a id="html&#45;parsing&#45;methods&#58;tt&#45;getcompliantinput&#45;3"></a>[get trusted type compliant input](https://w3c.github.io/trusted-types/dist/spec/#get-trusted-type-compliant-input) algorithm with the <a id="html&#45;parsing&#45;methods&#58;current&#45;global&#45;object"></a>[current global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#current-global-object)&#44; <var>html</var>&#44; <var>options</var>&#44; and &#34;<code>Document parseHTMLUnsafe</code>&#34;&#46;

2.  Let <var>document</var> be a new <a id="html&#45;parsing&#45;methods&#58;document&#45;4"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#44; whose <a id="html&#45;parsing&#45;methods&#58;concept&#45;document&#45;content&#45;type&#45;2"></a>[content type](https://dom.spec.whatwg.org/#concept-document-content-type) is &#34;<code>text&#47;html</code>&#34; and <a id="html&#45;parsing&#45;methods&#58;concept&#45;document&#45;origin&#45;3"></a>[origin](https://dom.spec.whatwg.org/#concept-document-origin) is <a id="html&#45;parsing&#45;methods&#58;this&#45;12"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="html&#45;parsing&#45;methods&#58;concept&#45;relevant&#45;global&#45;4"></a>[relevant global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global)&#39;s <a id="html&#45;parsing&#45;methods&#58;concept&#45;document&#45;window&#45;2"></a>[associated <code>Document</code>](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-window)&#39;s <a id="html&#45;parsing&#45;methods&#58;concept&#45;document&#45;origin&#45;4"></a>[origin](https://dom.spec.whatwg.org/#concept-document-origin)&#46;

    <strong>Note&#58;</strong>

    Since <var>document</var> does not have a <a id="html&#45;parsing&#45;methods&#58;concept&#45;document&#45;bc&#45;2"></a>[browsing context](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc)&#44; <a id="html&#45;parsing&#45;methods&#58;concept&#45;n&#45;script&#45;2"></a>[scripting is disabled](#concept-n-script)&#46;

3.  Set <var>document</var>&#39;s <a id="html&#45;parsing&#45;methods&#58;concept&#45;document&#45;allow&#45;declarative&#45;shadow&#45;roots&#45;2"></a>[allow declarative shadow roots](https://dom.spec.whatwg.org/#concept-document-allow-declarative-shadow-roots) to true&#46;

4.  <a id="html&#45;parsing&#45;methods&#58;parse&#45;html&#45;from&#45;a&#45;string&#45;2"></a>[Parse HTML from a string](#parse-html-from-a-string) given <var>document</var> and <var>compliantHTML</var>&#46;

5.  Let <var>sanitizer</var> be the result of <a id="html&#45;parsing&#45;methods&#58;get&#45;a&#45;sanitizer&#45;instance&#45;from&#45;options&#45;2"></a>[getting a sanitizer instance from options](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#get-a-sanitizer-instance-from-options) given <var>compliantOptions</var> and false&#46;

6.  If <var>sanitizer</var> is not null&#44; then <a id="html&#45;parsing&#45;methods&#58;sanitize&#45;8"></a>[sanitize](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitize) <var>document</var> with <var>sanitizer</var> and false&#46;

7.  Return <var>document</var>&#46;

#### <a id="html&#45;serialization&#45;methods"></a>8&#46;5&#46;3 HTML serialization methods[](#html-serialization-methods)

<strong><code><var>html</var> &#61; <var>element</var>&#46;<a id="dom&#45;element&#45;gethtml&#45;dev"></a>[getHTML](#dom-element-gethtml)&#40;&#123; <a id="html&#45;serialization&#45;methods&#58;dom&#45;gethtmloptions&#45;serializableshadowroots"></a>[serializableShadowRoots](#dom-gethtmloptions-serializableshadowroots)&#44; <a id="html&#45;serialization&#45;methods&#58;dom&#45;gethtmloptions&#45;shadowroots"></a>[shadowRoots](#dom-gethtmloptions-shadowroots) &#125;&#41;</code></strong>

Returns the result of serializing <var>element</var> to HTML&#46; <a id="html&#45;serialization&#45;methods&#58;shadow&#45;root"></a>[Shadow roots](https://dom.spec.whatwg.org/#concept-shadow-root) within <var>element</var> are serialized according to the provided options&#58;

- If <a id="html&#45;serialization&#45;methods&#58;dom&#45;gethtmloptions&#45;serializableshadowroots&#45;2"></a><code>[serializableShadowRoots](#dom-gethtmloptions-serializableshadowroots)</code> is true&#44; then all shadow roots marked as <a id="html&#45;serialization&#45;methods&#58;shadow&#45;serializable"></a>[serializable](https://dom.spec.whatwg.org/#shadowroot-serializable) are serialized&#46;

- If the <a id="html&#45;serialization&#45;methods&#58;dom&#45;gethtmloptions&#45;shadowroots&#45;2"></a><code>[shadowRoots](#dom-gethtmloptions-shadowroots)</code> array is provided&#44; then all shadow roots specified in the array are serialized&#44; regardless of whether or not they are marked as serializable&#46;

If neither option is provided&#44; then no shadow roots are serialized&#46;

<strong><code><var>html</var> &#61; <var>shadowRoot</var>&#46;<a id="dom&#45;shadowroot&#45;gethtml&#45;dev"></a>[getHTML](#dom-shadowroot-gethtml)&#40;&#123; <a id="html&#45;serialization&#45;methods&#58;dom&#45;gethtmloptions&#45;serializableshadowroots&#45;3"></a>[serializableShadowRoots](#dom-gethtmloptions-serializableshadowroots)&#44; <a id="html&#45;serialization&#45;methods&#58;dom&#45;gethtmloptions&#45;shadowroots&#45;3"></a>[shadowRoots](#dom-gethtmloptions-shadowroots) &#125;&#41;</code></strong>

Returns the result of serializing <var>shadowRoot</var> to HTML&#44; using its <a id="html&#45;serialization&#45;methods&#58;concept&#45;documentfragment&#45;host"></a>[shadow host](https://dom.spec.whatwg.org/#concept-documentfragment-host) as the context element&#46; <a id="html&#45;serialization&#45;methods&#58;shadow&#45;root&#45;2"></a>[Shadow roots](https://dom.spec.whatwg.org/#concept-shadow-root) within <var>shadowRoot</var> are serialized according to the provided options&#44; as above&#46;

<a id="html&#45;serialization&#45;methods&#58;element"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code>&#39;s <a id="dom&#45;element&#45;gethtml"></a><strong><code>getHTML&#40;<var>options</var>&#41;</code></strong> method steps are to return the result of <a id="html&#45;serialization&#45;methods&#58;html&#45;fragment&#45;serialisation&#45;algorithm"></a>[HTML fragment serialization algorithm](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#html-fragment-serialisation-algorithm) with <a id="html&#45;serialization&#45;methods&#58;this"></a>[this](https://webidl.spec.whatwg.org/#this)&#44; <var>options</var>&#91;&#34;<a id="html&#45;serialization&#45;methods&#58;dom&#45;gethtmloptions&#45;serializableshadowroots&#45;4"></a><code>[serializableShadowRoots](#dom-gethtmloptions-serializableshadowroots)</code>&#34;&#93;&#44; and <var>options</var>&#91;&#34;<a id="html&#45;serialization&#45;methods&#58;dom&#45;gethtmloptions&#45;shadowroots&#45;4"></a><code>[shadowRoots](#dom-gethtmloptions-shadowroots)</code>&#34;&#93;&#46;

<a id="html&#45;serialization&#45;methods&#58;shadowroot"></a><code>[ShadowRoot](https://dom.spec.whatwg.org/#interface-shadowroot)</code>&#39;s <a id="dom&#45;shadowroot&#45;gethtml"></a><strong><code>getHTML&#40;<var>options</var>&#41;</code></strong> method steps are to return the result of <a id="html&#45;serialization&#45;methods&#58;html&#45;fragment&#45;serialisation&#45;algorithm&#45;2"></a>[HTML fragment serialization algorithm](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#html-fragment-serialisation-algorithm) with <a id="html&#45;serialization&#45;methods&#58;this&#45;2"></a>[this](https://webidl.spec.whatwg.org/#this)&#44; <var>options</var>&#91;&#34;<a id="html&#45;serialization&#45;methods&#58;dom&#45;gethtmloptions&#45;serializableshadowroots&#45;5"></a><code>[serializableShadowRoots](#dom-gethtmloptions-serializableshadowroots)</code>&#34;&#93;&#44; and <var>options</var>&#91;&#34;<a id="html&#45;serialization&#45;methods&#58;dom&#45;gethtmloptions&#45;shadowroots&#45;5"></a><code>[shadowRoots](#dom-gethtmloptions-shadowroots)</code>&#34;&#93;&#46;

#### <a id="the&#45;innerhtml&#45;property"></a>8&#46;5&#46;4 The <a id="the&#45;innerhtml&#45;property&#58;dom&#45;element&#45;innerhtml"></a><code>[innerHTML](#dom-element-innerhtml)</code> property[](#the-innerhtml-property)

The <a id="the&#45;innerhtml&#45;property&#58;dom&#45;element&#45;innerhtml&#45;2"></a><code>[innerHTML](#dom-element-innerhtml)</code> property has a number of outstanding issues in the DOM Parsing and Serialization [issue tracker](https://github.com/w3c/DOM-Parsing/issues)&#44; documenting various problems with its specification&#46;

<strong><code><var>element</var>&#46;<a id="dom&#45;element&#45;innerhtml&#45;dev"></a>[innerHTML](#dom-element-innerhtml)</code></strong>

Returns a fragment of HTML or XML that represents the element&#39;s contents&#46;

In the case of an XML document&#44; throws an <a id="the&#45;innerhtml&#45;property&#58;invalidstateerror"></a>[&#34;<code>InvalidStateError</code>&#34;](https://webidl.spec.whatwg.org/#invalidstateerror) <a id="the&#45;innerhtml&#45;property&#58;domexception"></a><code>[DOMException](https://webidl.spec.whatwg.org/#dfn-DOMException)</code> if the element cannot be serialized to XML&#46;

<strong><code><var>element</var>&#46;<a id="the&#45;innerhtml&#45;property&#58;dom&#45;element&#45;innerhtml&#45;3"></a>[innerHTML](#dom-element-innerhtml) &#61; <var>value</var></code></strong>

Replaces the contents of the element with nodes parsed from the given string&#46;

In the case of an XML document&#44; throws a <a id="the&#45;innerhtml&#45;property&#58;syntaxerror"></a>[&#34;<code>SyntaxError</code>&#34;](https://webidl.spec.whatwg.org/#syntaxerror) <a id="the&#45;innerhtml&#45;property&#58;domexception&#45;2"></a><code>[DOMException](https://webidl.spec.whatwg.org/#dfn-DOMException)</code> if the given string is not well&#45;formed&#46;

<strong>Note&#58;</strong>

Setting <a id="the&#45;innerhtml&#45;property&#58;dom&#45;element&#45;innerhtml&#45;4"></a><code>[innerHTML](#dom-element-innerhtml)</code> on a <a id="the&#45;innerhtml&#45;property&#58;the&#45;template&#45;element"></a><code>[template](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-template-element)</code> element will replace all the nodes in its <a id="the&#45;innerhtml&#45;property&#58;template&#45;contents"></a>[template contents](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#template-contents) rather than its <a id="the&#45;innerhtml&#45;property&#58;concept&#45;tree&#45;child"></a>[children](https://dom.spec.whatwg.org/#concept-tree-child)&#46;

<strong><code><var>shadowRoot</var>&#46;<a id="dom&#45;shadowroot&#45;innerhtml&#45;dev"></a>[innerHTML](#dom-shadowroot-innerhtml)</code></strong>

Returns a fragment of HTML that represents the shadow roots&#39;s contents&#46;

<strong><code><var>shadowRoot</var>&#46;<a id="the&#45;innerhtml&#45;property&#58;dom&#45;shadowroot&#45;innerhtml"></a>[innerHTML](#dom-shadowroot-innerhtml) &#61; <var>value</var></code></strong>

Replaces the contents of the shadow root with nodes parsed from the given string&#46;

<strong>Warning&#58;</strong>

By default&#44; these properties&#39; setters perform no sanitization to remove potentially&#45;dangerous elements and attributes like <a id="the&#45;innerhtml&#45;property&#58;the&#45;script&#45;element"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> or <a id="the&#45;innerhtml&#45;property&#58;event&#45;handler&#45;content&#45;attributes"></a>[event handler content attributes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-handler-content-attributes)&#46;

The <a id="fragment&#45;serializing&#45;algorithm&#45;steps"></a><strong>fragment serializing algorithm steps</strong>&#44; given an <a id="the&#45;innerhtml&#45;property&#58;element"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code>&#44; <a id="the&#45;innerhtml&#45;property&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#44; or <a id="the&#45;innerhtml&#45;property&#58;documentfragment"></a><code>[DocumentFragment](https://dom.spec.whatwg.org/#interface-documentfragment)</code> <var>node</var> and a boolean <var>require well&#45;formed</var>&#44; are&#58;

1.  Let <var>context document</var> be <var>node</var>&#39;s <a id="the&#45;innerhtml&#45;property&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46;

2.  If <var>context document</var> is an <a id="the&#45;innerhtml&#45;property&#58;html&#45;documents"></a>[HTML document](https://dom.spec.whatwg.org/#html-document)&#44; then return the result of <a id="the&#45;innerhtml&#45;property&#58;html&#45;fragment&#45;serialisation&#45;algorithm"></a>[HTML fragment serialization algorithm](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#html-fragment-serialisation-algorithm) with <var>node</var>&#44; false&#44; and « »&#46;

3.  Return the <a id="the&#45;innerhtml&#45;property&#58;xml&#45;serialization"></a>[XML serialization](https://w3c.github.io/DOM-Parsing/#dfn-xml-serialization) of <var>node</var> given <var>require well&#45;formed</var>&#46;

A <a id="fragment&#45;parser&#45;mode"></a><strong>fragment parser mode</strong> is one of the following&#58;

<strong><a id="fragment&#45;parser&#45;mode&#45;safe"></a><strong>Safe</strong></strong>

<a id="the&#45;innerhtml&#45;property&#58;sanitizer&#45;category&#45;unsafe"></a>[Unsafe](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-unsafe) elements and attributes are removed&#44; and <a id="the&#45;innerhtml&#45;property&#58;allow&#45;declarative&#45;shadow&#45;roots"></a>[allow declarative shadow roots](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#allow-declarative-shadow-roots) is true&#46; The <a id="the&#45;innerhtml&#45;property&#58;html&#45;parser"></a>[HTML parser](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#html-parser) is used regardless of the type of document&#46;

<strong><a id="fragment&#45;parser&#45;mode&#45;unsafe"></a><strong>Unsafe</strong></strong>

<a id="the&#45;innerhtml&#45;property&#58;sanitizer&#45;category&#45;unsafe&#45;2"></a>[Unsafe](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-unsafe) elements and attributes are not removed unless a sanitizer is supplied&#44; and <a id="the&#45;innerhtml&#45;property&#58;allow&#45;declarative&#45;shadow&#45;roots&#45;2"></a>[allow declarative shadow roots](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#allow-declarative-shadow-roots) is true&#46; The <a id="the&#45;innerhtml&#45;property&#58;html&#45;parser&#45;2"></a>[HTML parser](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#html-parser) is used regardless of the type of document&#46;

<strong><a id="fragment&#45;parser&#45;mode&#45;legacy"></a><strong>Legacy</strong></strong>

<a id="the&#45;innerhtml&#45;property&#58;sanitizer&#45;category&#45;unsafe&#45;3"></a>[Unsafe](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitizer-category-unsafe) elements and attributes are not removed unless a sanitizer is supplied&#44; and <a id="the&#45;innerhtml&#45;property&#58;allow&#45;declarative&#45;shadow&#45;roots&#45;3"></a>[allow declarative shadow roots](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#allow-declarative-shadow-roots) is false&#46; In <a id="the&#45;innerhtml&#45;property&#58;xml&#45;documents"></a>[XML documents](https://dom.spec.whatwg.org/#xml-document)&#44; the <a id="the&#45;innerhtml&#45;property&#58;xml&#45;parser"></a>[XML parser](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#xml-parser) is used&#46;

To <a id="set&#45;and&#45;filter&#45;html"></a><strong>set and filter HTML</strong>&#44; given an <a id="the&#45;innerhtml&#45;property&#58;element&#45;2"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code> or <a id="the&#45;innerhtml&#45;property&#58;documentfragment&#45;2"></a><code>[DocumentFragment](https://dom.spec.whatwg.org/#interface-documentfragment)</code> <var>target</var>&#44; a string <var>html</var>&#44; a <a id="the&#45;innerhtml&#45;property&#58;sethtmloptions"></a><code>[SetHTMLOptions](#sethtmloptions)</code> or <a id="the&#45;innerhtml&#45;property&#58;sethtmlunsafeoptions"></a><code>[SetHTMLUnsafeOptions](#sethtmlunsafeoptions)</code> dictionary&#44; or a <a id="the&#45;innerhtml&#45;property&#58;tt&#45;trustedhtmlparseroptions"></a><code>[TrustedHTMLParserOptions](https://w3c.github.io/trusted-types/dist/spec/#trusted-html-parser-options)</code> object <var>options</var>&#44; and a <a id="the&#45;innerhtml&#45;property&#58;fragment&#45;parser&#45;mode"></a>[fragment parser mode](#fragment-parser-mode) <var>mode</var>&#58;

1.  If <var>mode</var> is <a id="the&#45;innerhtml&#45;property&#58;fragment&#45;parser&#45;mode&#45;safe"></a>[Safe](#fragment-parser-mode-safe)&#58;

    1.  Let <var>context</var> be <var>target</var> if <var>target</var> is an <a id="the&#45;innerhtml&#45;property&#58;element&#45;3"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code>&#59; otherwise <var>target</var>&#39;s <a id="the&#45;innerhtml&#45;property&#58;concept&#45;documentfragment&#45;host"></a>[host](https://dom.spec.whatwg.org/#concept-documentfragment-host)&#46;

    2.  <a id="the&#45;innerhtml&#45;property&#58;assert"></a>[Assert](https://infra.spec.whatwg.org/#assert)&#58; <var>context</var> is non&#45;null&#46;

    3.  If all of the following are true&#58;

        - <var>context</var>&#39;s <a id="the&#45;innerhtml&#45;property&#58;concept&#45;element&#45;local&#45;name"></a>[local name](https://dom.spec.whatwg.org/#concept-element-local-name) is &#34;<a id="the&#45;innerhtml&#45;property&#58;the&#45;script&#45;element&#45;2"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code>&#34;&#59; and

        - <var>context</var>&#39;s <a id="the&#45;innerhtml&#45;property&#58;concept&#45;element&#45;namespace"></a>[namespace](https://dom.spec.whatwg.org/#concept-element-namespace) is the <a id="the&#45;innerhtml&#45;property&#58;html&#45;namespace&#45;2"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace) or the <a id="the&#45;innerhtml&#45;property&#58;svg&#45;namespace"></a>[SVG namespace](https://infra.spec.whatwg.org/#svg-namespace)&#44;

        then return&#46;

2.  Let <var>fragment</var> be the result of invoking the <a id="the&#45;innerhtml&#45;property&#58;fragment&#45;parsing&#45;algorithm&#45;steps"></a>[fragment parsing algorithm steps](#fragment-parsing-algorithm-steps) given <var>target</var>&#44; <var>html</var>&#44; <var>options</var>&#44; and <var>mode</var>&#46;

3.  <a id="the&#45;innerhtml&#45;property&#58;concept&#45;node&#45;replace&#45;all"></a>[Replace all](https://dom.spec.whatwg.org/#concept-node-replace-all) with <var>fragment</var> within <var>target</var>&#46;

The <a id="fragment&#45;parsing&#45;algorithm&#45;steps"></a><strong>fragment parsing algorithm steps</strong>&#44; given an <a id="the&#45;innerhtml&#45;property&#58;element&#45;4"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code> or <a id="the&#45;innerhtml&#45;property&#58;documentfragment&#45;3"></a><code>[DocumentFragment](https://dom.spec.whatwg.org/#interface-documentfragment)</code> <var>target</var>&#44; a string <var>markup</var>&#44; a <a id="the&#45;innerhtml&#45;property&#58;sethtmloptions&#45;2"></a><code>[SetHTMLOptions](#sethtmloptions)</code> or <a id="the&#45;innerhtml&#45;property&#58;sethtmlunsafeoptions&#45;2"></a><code>[SetHTMLUnsafeOptions](#sethtmlunsafeoptions)</code> dictionary&#44; or a <a id="the&#45;innerhtml&#45;property&#58;tt&#45;trustedhtmlparseroptions&#45;2"></a><code>[TrustedHTMLParserOptions](https://w3c.github.io/trusted-types/dist/spec/#trusted-html-parser-options)</code> object <var>options</var>&#44; and a <a id="the&#45;innerhtml&#45;property&#58;fragment&#45;parser&#45;mode&#45;2"></a>[fragment parser mode](#fragment-parser-mode) <var>mode</var>&#44; are&#58;

1.  If <var>mode</var> is <a id="the&#45;innerhtml&#45;property&#58;fragment&#45;parser&#45;mode&#45;legacy"></a>[Legacy](#fragment-parser-mode-legacy) and <var>target</var>&#39;s <a id="the&#45;innerhtml&#45;property&#58;node&#45;document&#45;2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document) is an <a id="the&#45;innerhtml&#45;property&#58;xml&#45;documents&#45;2"></a>[XML document](https://dom.spec.whatwg.org/#xml-document)&#44; then return the result of invoking the <a id="the&#45;innerhtml&#45;property&#58;xml&#45;fragment&#45;parsing&#45;algorithm"></a>[XML fragment parsing algorithm](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#xml-fragment-parsing-algorithm) given <var>target</var> and <var>markup</var>&#46;

    <strong>Note&#58;</strong>

    Sanitization is not supported for <a id="the&#45;innerhtml&#45;property&#58;xml&#45;documents&#45;3"></a>[XML documents](https://dom.spec.whatwg.org/#xml-document)&#46;

2.  Let <var>safe</var> be true if <var>mode</var> is <a id="the&#45;innerhtml&#45;property&#58;fragment&#45;parser&#45;mode&#45;safe&#45;2"></a>[Safe](#fragment-parser-mode-safe)&#59; otherwise false&#46;

3.  Let <var>sanitizer</var> be the result of <a id="the&#45;innerhtml&#45;property&#58;get&#45;a&#45;sanitizer&#45;instance&#45;from&#45;options"></a>[getting a sanitizer instance from options](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#get-a-sanitizer-instance-from-options) given <var>options</var> and <var>safe</var>&#46;

4.  Let <var>scriptingMode</var> be <a id="the&#45;innerhtml&#45;property&#58;scripting&#45;mode&#45;inert"></a>[Inert](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#scripting-mode-inert)&#46;

5.  Let <var>runScripts</var> be <var>options</var>&#39;s <a id="the&#45;innerhtml&#45;property&#58;tt&#45;trustedhtmlparseroptions&#45;runscripts"></a>[runScripts](https://w3c.github.io/trusted-types/dist/spec/#trustedhtmlparseroptions-runscripts) if <var>options</var> is a <a id="the&#45;innerhtml&#45;property&#58;tt&#45;trustedhtmlparseroptions&#45;3"></a><code>[TrustedHTMLParserOptions](https://w3c.github.io/trusted-types/dist/spec/#trusted-html-parser-options)</code>&#59; otherwise <var>options</var>&#91;&#34;<a id="the&#45;innerhtml&#45;property&#58;dom&#45;sethtmlunsafeoptions&#45;runscripts"></a><code>[runScripts](#dom-sethtmlunsafeoptions-runscripts)</code>&#34;&#93; <a id="the&#45;innerhtml&#45;property&#58;map&#45;with&#45;default"></a>[with default](https://infra.spec.whatwg.org/#map-with-default) false&#46;

6.  If <var>runScripts</var> is true&#58;

    1.  <a id="the&#45;innerhtml&#45;property&#58;assert&#45;2"></a>[Assert](https://infra.spec.whatwg.org/#assert)&#58; <var>mode</var> is not <a id="the&#45;innerhtml&#45;property&#58;fragment&#45;parser&#45;mode&#45;safe&#45;3"></a>[Safe](#fragment-parser-mode-safe)&#46;

    2.  Set <var>scriptingMode</var> to <a id="the&#45;innerhtml&#45;property&#58;scripting&#45;mode&#45;fragment"></a>[Fragment](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#scripting-mode-fragment)&#46;

7.  Let <var>allowDeclarativeShadowRoots</var> be false if <var>mode</var> is <a id="the&#45;innerhtml&#45;property&#58;fragment&#45;parser&#45;mode&#45;legacy&#45;2"></a>[Legacy](#fragment-parser-mode-legacy)&#59; otherwise true&#46;

8.  Let <var>fragment</var> be the result of the <a id="the&#45;innerhtml&#45;property&#58;html&#45;fragment&#45;parsing&#45;algorithm"></a>[HTML fragment parsing algorithm](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#html-fragment-parsing-algorithm) given <var>target</var>&#44; <var>markup</var>&#44; <var>allowDeclarativeShadowRoots</var>&#44; and <var>scriptingMode</var>&#46;

    <strong>Note&#58;</strong>

    Scripts in <var>fragment</var> will only execute once they are inserted into a document&#44; and only if <var>scriptingMode</var> is <a id="the&#45;innerhtml&#45;property&#58;scripting&#45;mode&#45;fragment&#45;2"></a>[Fragment](html-language-syntax-parser--877d6146487f--37a9ba51eed8.md#scripting-mode-fragment)&#46;

9.  If <var>sanitizer</var> is not null&#44; then <a id="the&#45;innerhtml&#45;property&#58;sanitize"></a>[sanitize](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sanitize) <var>fragment</var> with <var>sanitizer</var> and <var>safe</var>&#46;

10. Return <var>fragment</var>&#46;

<a id="the&#45;innerhtml&#45;property&#58;element&#45;5"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code>&#39;s <a id="dom&#45;element&#45;innerhtml"></a><strong><code>innerHTML</code></strong> getter steps are to return the result of running <a id="the&#45;innerhtml&#45;property&#58;fragment&#45;serializing&#45;algorithm&#45;steps"></a>[fragment serializing algorithm steps](#fragment-serializing-algorithm-steps) with <a id="the&#45;innerhtml&#45;property&#58;this"></a>[this](https://webidl.spec.whatwg.org/#this) and true&#46;

<a id="the&#45;innerhtml&#45;property&#58;shadowroot"></a><code>[ShadowRoot](https://dom.spec.whatwg.org/#interface-shadowroot)</code>&#39;s <a id="dom&#45;shadowroot&#45;innerhtml"></a><strong><code>innerHTML</code></strong> getter steps are to return the result of running <a id="the&#45;innerhtml&#45;property&#58;fragment&#45;serializing&#45;algorithm&#45;steps&#45;2"></a>[fragment serializing algorithm steps](#fragment-serializing-algorithm-steps) with <a id="the&#45;innerhtml&#45;property&#58;this&#45;2"></a>[this](https://webidl.spec.whatwg.org/#this) and true&#46;

<a id="the&#45;innerhtml&#45;property&#58;element&#45;6"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code>&#39;s <a id="the&#45;innerhtml&#45;property&#58;dom&#45;element&#45;innerhtml&#45;5"></a><code>[innerHTML](#dom-element-innerhtml)</code> setter steps are&#58;

1.  Let &#40;<var>compliantString</var>&#44; <var>compliantOptions</var>&#41; be the result of invoking the <a id="the&#45;innerhtml&#45;property&#58;tt&#45;getcompliantinput"></a>[get trusted type compliant input](https://w3c.github.io/trusted-types/dist/spec/#get-trusted-type-compliant-input) algorithm with <a id="the&#45;innerhtml&#45;property&#58;this&#45;3"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;innerhtml&#45;property&#58;concept&#45;relevant&#45;global"></a>[relevant global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global)&#44; the given value&#44; a new <a id="the&#45;innerhtml&#45;property&#58;sethtmlunsafeoptions&#45;3"></a><code>[SetHTMLUnsafeOptions](#sethtmlunsafeoptions)</code> dictionary&#44; &#34;<code>Element innerHTML</code>&#34;&#44; <a id="the&#45;innerhtml&#45;property&#58;this&#45;4"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;innerhtml&#45;property&#58;node&#45;document&#45;3"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="the&#45;innerhtml&#45;property&#58;concept&#45;document&#45;type"></a>[type](https://dom.spec.whatwg.org/#concept-document-type)&#44; and false&#46;

2.  Let <var>target</var> be <a id="the&#45;innerhtml&#45;property&#58;this&#45;5"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;innerhtml&#45;property&#58;template&#45;contents&#45;2"></a>[template contents](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#template-contents) if <a id="the&#45;innerhtml&#45;property&#58;this&#45;6"></a>[this](https://webidl.spec.whatwg.org/#this) is a <a id="the&#45;innerhtml&#45;property&#58;the&#45;template&#45;element&#45;2"></a><code>[template](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-template-element)</code> element&#59; otherwise <a id="the&#45;innerhtml&#45;property&#58;this&#45;7"></a>[this](https://webidl.spec.whatwg.org/#this)&#46;

3.  <a id="the&#45;innerhtml&#45;property&#58;set&#45;and&#45;filter&#45;html"></a>[Set and filter HTML](#set-and-filter-html) given <var>target</var>&#44; <var>compliantString</var>&#44; <var>compliantOptions</var>&#44; and <a id="the&#45;innerhtml&#45;property&#58;fragment&#45;parser&#45;mode&#45;legacy&#45;3"></a>[Legacy](#fragment-parser-mode-legacy)&#46;

<a id="the&#45;innerhtml&#45;property&#58;shadowroot&#45;2"></a><code>[ShadowRoot](https://dom.spec.whatwg.org/#interface-shadowroot)</code>&#39;s <a id="the&#45;innerhtml&#45;property&#58;dom&#45;shadowroot&#45;innerhtml&#45;2"></a><code>[innerHTML](#dom-shadowroot-innerhtml)</code> setter steps are&#58;

1.  Let &#40;<var>compliantString</var>&#44; <var>compliantOptions</var>&#41; be the result of invoking the <a id="the&#45;innerhtml&#45;property&#58;tt&#45;getcompliantinput&#45;2"></a>[get trusted type compliant input](https://w3c.github.io/trusted-types/dist/spec/#get-trusted-type-compliant-input) algorithm with <a id="the&#45;innerhtml&#45;property&#58;this&#45;8"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;innerhtml&#45;property&#58;concept&#45;relevant&#45;global&#45;2"></a>[relevant global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global)&#44; the given value&#44; a new <a id="the&#45;innerhtml&#45;property&#58;sethtmlunsafeoptions&#45;4"></a><code>[SetHTMLUnsafeOptions](#sethtmlunsafeoptions)</code> dictionary&#44; &#34;<code>ShadowRoot innerHTML</code>&#34;&#44; <a id="the&#45;innerhtml&#45;property&#58;this&#45;9"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;innerhtml&#45;property&#58;node&#45;document&#45;4"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="the&#45;innerhtml&#45;property&#58;concept&#45;document&#45;type&#45;2"></a>[type](https://dom.spec.whatwg.org/#concept-document-type)&#44; and false&#46;

2.  <a id="the&#45;innerhtml&#45;property&#58;set&#45;and&#45;filter&#45;html&#45;2"></a>[Set and filter HTML](#set-and-filter-html) given <a id="the&#45;innerhtml&#45;property&#58;this&#45;10"></a>[this](https://webidl.spec.whatwg.org/#this)&#44; <var>compliantString</var>&#44; <var>compliantOptions</var>&#44; and <a id="the&#45;innerhtml&#45;property&#58;fragment&#45;parser&#45;mode&#45;legacy&#45;4"></a>[Legacy](#fragment-parser-mode-legacy)&#46;

#### <a id="the&#45;outerhtml&#45;property"></a>8&#46;5&#46;5 The <a id="the&#45;outerhtml&#45;property&#58;dom&#45;element&#45;outerhtml"></a><code>[outerHTML](#dom-element-outerhtml)</code> property[](#the-outerhtml-property)

The <a id="the&#45;outerhtml&#45;property&#58;dom&#45;element&#45;outerhtml&#45;2"></a><code>[outerHTML](#dom-element-outerhtml)</code> property has a number of outstanding issues in the DOM Parsing and Serialization [issue tracker](https://github.com/w3c/DOM-Parsing/issues)&#44; documenting various problems with its specification&#46;

<strong><code><var>element</var>&#46;<a id="dom&#45;element&#45;outerhtml&#45;dev"></a>[outerHTML](#dom-element-outerhtml)</code></strong>

Returns a fragment of HTML or XML that represents the element and its contents&#46;

In the case of an XML document&#44; throws an <a id="the&#45;outerhtml&#45;property&#58;invalidstateerror"></a>[&#34;<code>InvalidStateError</code>&#34;](https://webidl.spec.whatwg.org/#invalidstateerror) <a id="the&#45;outerhtml&#45;property&#58;domexception"></a><code>[DOMException](https://webidl.spec.whatwg.org/#dfn-DOMException)</code> if the element cannot be serialized to XML&#46;

<strong><code><var>element</var>&#46;<a id="the&#45;outerhtml&#45;property&#58;dom&#45;element&#45;outerhtml&#45;3"></a>[outerHTML](#dom-element-outerhtml) &#61; <var>value</var></code></strong>

Replaces the element with nodes parsed from the given string&#46;

In the case of an XML document&#44; throws a <a id="the&#45;outerhtml&#45;property&#58;syntaxerror"></a>[&#34;<code>SyntaxError</code>&#34;](https://webidl.spec.whatwg.org/#syntaxerror) <a id="the&#45;outerhtml&#45;property&#58;domexception&#45;2"></a><code>[DOMException](https://webidl.spec.whatwg.org/#dfn-DOMException)</code> if the given string is not well&#45;formed&#46;

Throws a <a id="the&#45;outerhtml&#45;property&#58;nomodificationallowederror"></a>[&#34;<code>NoModificationAllowedError</code>&#34;](https://webidl.spec.whatwg.org/#nomodificationallowederror) <a id="the&#45;outerhtml&#45;property&#58;domexception&#45;3"></a><code>[DOMException](https://webidl.spec.whatwg.org/#dfn-DOMException)</code> if the parent of the element is a <a id="the&#45;outerhtml&#45;property&#58;document"></a>[<code>Document</code>](html-language-foundations--877d6146487f--b55e73c31978.md#document)&#46;

<strong>Warning&#58;</strong>

By default&#44; this property&#39;s setter performs no sanitization to remove potentially&#45;dangerous elements and attributes like <a id="the&#45;outerhtml&#45;property&#58;the&#45;script&#45;element"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> or <a id="the&#45;outerhtml&#45;property&#58;event&#45;handler&#45;content&#45;attributes"></a>[event handler content attributes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-handler-content-attributes)&#46;

<a id="the&#45;outerhtml&#45;property&#58;element"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code>&#39;s <a id="dom&#45;element&#45;outerhtml"></a><strong><code>outerHTML</code></strong> getter steps are&#58;

1.  Let <var>element</var> be a fictional node whose only child is <a id="the&#45;outerhtml&#45;property&#58;this"></a>[this](https://webidl.spec.whatwg.org/#this)&#46;

2.  Return the result of running <a id="the&#45;outerhtml&#45;property&#58;fragment&#45;serializing&#45;algorithm&#45;steps"></a>[fragment serializing algorithm steps](#fragment-serializing-algorithm-steps) with <var>element</var> and true&#46;

<a id="the&#45;outerhtml&#45;property&#58;element&#45;2"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code>&#39;s <a id="the&#45;outerhtml&#45;property&#58;dom&#45;element&#45;outerhtml&#45;4"></a><code>[outerHTML](#dom-element-outerhtml)</code> setter steps are&#58;

1.  Let &#40;<var>compliantString</var>&#44; <var>compliantOptions</var>&#41; be the result of invoking the <a id="the&#45;outerhtml&#45;property&#58;tt&#45;getcompliantinput"></a>[get trusted type compliant input](https://w3c.github.io/trusted-types/dist/spec/#get-trusted-type-compliant-input) algorithm with <a id="the&#45;outerhtml&#45;property&#58;this&#45;2"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;outerhtml&#45;property&#58;concept&#45;relevant&#45;global"></a>[relevant global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global)&#44; the given value&#44; a new <a id="the&#45;outerhtml&#45;property&#58;sethtmlunsafeoptions"></a><code>[SetHTMLUnsafeOptions](#sethtmlunsafeoptions)</code> dictionary&#44; &#34;<code>Element outerHTML</code>&#34;&#44; <a id="the&#45;outerhtml&#45;property&#58;this&#45;3"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;outerhtml&#45;property&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="the&#45;outerhtml&#45;property&#58;concept&#45;document&#45;type"></a>[type](https://dom.spec.whatwg.org/#concept-document-type)&#44; and false&#46;

2.  Let <var>parent</var> be <a id="the&#45;outerhtml&#45;property&#58;this&#45;4"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;outerhtml&#45;property&#58;parent"></a>[parent](https://dom.spec.whatwg.org/#concept-tree-parent)&#46;

3.  If <var>parent</var> is null&#44; return&#46; There would be no way to obtain a reference to the nodes created even if the remaining steps were run&#46;

4.  If <var>parent</var> is a <a id="the&#45;outerhtml&#45;property&#58;document&#45;2"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#44; throw a <a id="the&#45;outerhtml&#45;property&#58;nomodificationallowederror&#45;2"></a>[&#34;<code>NoModificationAllowedError</code>&#34;](https://webidl.spec.whatwg.org/#nomodificationallowederror) <a id="the&#45;outerhtml&#45;property&#58;domexception&#45;4"></a><code>[DOMException](https://webidl.spec.whatwg.org/#dfn-DOMException)</code>&#46;

5.  If <var>parent</var> is a <a id="the&#45;outerhtml&#45;property&#58;documentfragment"></a><code>[DocumentFragment](https://dom.spec.whatwg.org/#interface-documentfragment)</code>&#44; set <var>parent</var> to the result of <a id="the&#45;outerhtml&#45;property&#58;create&#45;an&#45;element"></a>[creating an element](https://dom.spec.whatwg.org/#concept-create-element) given <a id="the&#45;outerhtml&#45;property&#58;this&#45;5"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;outerhtml&#45;property&#58;node&#45;document&#45;2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#44; &#34;<code>body</code>&#34;&#44; and the <a id="the&#45;outerhtml&#45;property&#58;html&#45;namespace&#45;2"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace)&#46;

6.  Let <var>fragment</var> be the result of invoking the <a id="the&#45;outerhtml&#45;property&#58;fragment&#45;parsing&#45;algorithm&#45;steps"></a>[fragment parsing algorithm steps](#fragment-parsing-algorithm-steps) given <var>parent</var>&#44; <var>compliantString</var>&#44; <var>compliantOptions</var>&#44; and <a id="the&#45;outerhtml&#45;property&#58;fragment&#45;parser&#45;mode&#45;legacy"></a>[Legacy](#fragment-parser-mode-legacy)&#46;

7.  <a id="the&#45;outerhtml&#45;property&#58;concept&#45;node&#45;replace"></a>[Replace](https://dom.spec.whatwg.org/#concept-node-replace) <a id="the&#45;outerhtml&#45;property&#58;this&#45;6"></a>[this](https://webidl.spec.whatwg.org/#this) with <var>fragment</var> within <a id="the&#45;outerhtml&#45;property&#58;this&#45;7"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;outerhtml&#45;property&#58;parent&#45;2"></a>[parent](https://dom.spec.whatwg.org/#concept-tree-parent)&#46;

#### <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method"></a>8&#46;5&#46;6 The <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;dom&#45;element&#45;insertadjacenthtml"></a><code>[insertAdjacentHTML&#40;&#41;](#dom-element-insertadjacenthtml)</code> method[](#the-insertadjacenthtml()-method)

The <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;dom&#45;element&#45;insertadjacenthtml&#45;2"></a><code>[insertAdjacentHTML&#40;&#41;](#dom-element-insertadjacenthtml)</code> method has a number of outstanding issues in the DOM Parsing and Serialization [issue tracker](https://github.com/w3c/DOM-Parsing/issues)&#44; documenting various problems with its specification&#46;

<strong><code><var>element</var>&#46;<a id="dom&#45;element&#45;insertadjacenthtml&#45;dev"></a>[insertAdjacentHTML](#dom-element-insertadjacenthtml)&#40;<var>position</var>&#44; <var>string</var>&#41;</code></strong>

Parses <var>string</var> as HTML or XML and inserts the resulting nodes into the tree in the position given by the <var>position</var> argument&#44; as follows&#58;

<strong>&#34;<code>beforebegin</code>&#34;</strong>

Before the element itself &#40;i&#46;e&#46;&#44; after <var>element</var>&#39;s previous sibling&#41;

<strong>&#34;<code>afterbegin</code>&#34;</strong>

Just inside the element&#44; before its first child&#46;

<strong>&#34;<code>beforeend</code>&#34;</strong>

Just inside the element&#44; after its last child&#46;

<strong>&#34;<code>afterend</code>&#34;</strong>

After the element itself &#40;i&#46;e&#46;&#44; before <var>element</var>&#39;s next sibling&#41;

Throws a <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;syntaxerror"></a>[&#34;<code>SyntaxError</code>&#34;](https://webidl.spec.whatwg.org/#syntaxerror) <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;domexception"></a><code>[DOMException](https://webidl.spec.whatwg.org/#dfn-DOMException)</code> if the arguments have invalid values &#40;e&#46;g&#46;&#44; in the case of an <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;xml&#45;documents"></a>[XML document](https://dom.spec.whatwg.org/#xml-document)&#44; if the given string is not well&#45;formed&#41;&#46;

Throws a <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;nomodificationallowederror"></a>[&#34;<code>NoModificationAllowedError</code>&#34;](https://webidl.spec.whatwg.org/#nomodificationallowederror) <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;domexception&#45;2"></a><code>[DOMException](https://webidl.spec.whatwg.org/#dfn-DOMException)</code> if the given position isn&#39;t possible &#40;e&#46;g&#46; inserting elements after the root element of a <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#41;&#46;

<strong>Warning&#58;</strong>

By default&#44; this method performs no sanitization to remove potentially&#45;dangerous elements and attributes like <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;the&#45;script&#45;element"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> or <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;event&#45;handler&#45;content&#45;attributes"></a>[event handler content attributes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-handler-content-attributes)&#46;

<a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;element"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code>&#39;s <a id="dom&#45;element&#45;insertadjacenthtml"></a><strong><code>insertAdjacentHTML&#40;<var>position</var>&#44; <var>string</var>&#41;</code></strong> method steps are&#58;

1.  Let &#40;<var>compliantString</var>&#44; <var>compliantOptions</var>&#41; be the result of invoking the <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;tt&#45;getcompliantinput"></a>[get trusted type compliant input](https://w3c.github.io/trusted-types/dist/spec/#get-trusted-type-compliant-input) algorithm with <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;this"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;concept&#45;relevant&#45;global"></a>[relevant global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global)&#44; <var>string</var>&#44; a new <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;sethtmlunsafeoptions"></a><code>[SetHTMLUnsafeOptions](#sethtmlunsafeoptions)</code> dictionary&#44; &#34;<code>Element insertAdjacentHTML</code>&#34;&#44; <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;this&#45;2"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;concept&#45;document&#45;type"></a>[type](https://dom.spec.whatwg.org/#concept-document-type)&#44; and false&#46;

2.  Let <var>context</var> be null&#46;

3.  Use the first matching item from this list&#58;

    <strong>If <var>position</var> is an <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;ascii&#45;case&#45;insensitive"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for the string &#34;<code>beforebegin</code>&#34;</strong>

    <strong>If <var>position</var> is an <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;ascii&#45;case&#45;insensitive&#45;2"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for the string &#34;<code>afterend</code>&#34;</strong>

    1.  Set <var>context</var> to <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;this&#45;3"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;dom&#45;parent"></a>[parent](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-parent)&#46;

    2.  If <var>context</var> is null or a <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;document&#45;2"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#44; throw a <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;nomodificationallowederror&#45;2"></a>[&#34;<code>NoModificationAllowedError</code>&#34;](https://webidl.spec.whatwg.org/#nomodificationallowederror) <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;domexception&#45;3"></a><code>[DOMException](https://webidl.spec.whatwg.org/#dfn-DOMException)</code>&#46;

    <strong>If <var>position</var> is an <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;ascii&#45;case&#45;insensitive&#45;3"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for the string &#34;<code>afterbegin</code>&#34;</strong>

    <strong>If <var>position</var> is an <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;ascii&#45;case&#45;insensitive&#45;4"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for the string &#34;<code>beforeend</code>&#34;</strong>

    Set <var>context</var> to <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;this&#45;4"></a>[this](https://webidl.spec.whatwg.org/#this)&#46;
    <strong>Otherwise</strong>

    Throw a <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;syntaxerror&#45;2"></a>[&#34;<code>SyntaxError</code>&#34;](https://webidl.spec.whatwg.org/#syntaxerror) <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;domexception&#45;4"></a><code>[DOMException](https://webidl.spec.whatwg.org/#dfn-DOMException)</code>&#46;

4.  If <var>context</var> is not an <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;element&#45;2"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code> or all of the following are true&#58;

    - <var>context</var>&#39;s <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;node&#45;document&#45;2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document) is an HTML document&#59;

    - <var>context</var>&#39;s <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;concept&#45;element&#45;local&#45;name"></a>[local name](https://dom.spec.whatwg.org/#concept-element-local-name) is &#34;<a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;the&#45;html&#45;element"></a><code>[html](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-html-element)</code>&#34;&#59; and

    - <var>context</var>&#39;s <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;concept&#45;element&#45;namespace"></a>[namespace](https://dom.spec.whatwg.org/#concept-element-namespace) is the <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;html&#45;namespace&#45;2"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace)&#44;

    then set <var>context</var> to the result of <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;create&#45;an&#45;element"></a>[creating an element](https://dom.spec.whatwg.org/#concept-create-element) given <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;this&#45;5"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;node&#45;document&#45;3"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#44; &#34;<code>body</code>&#34;&#44; and the <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;html&#45;namespace&#45;2&#45;2"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace)&#46;

5.  Let <var>fragment</var> be the result of invoking the <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;fragment&#45;parsing&#45;algorithm&#45;steps"></a>[fragment parsing algorithm steps](#fragment-parsing-algorithm-steps) given <var>context</var>&#44; <var>compliantString</var>&#44; <var>compliantOptions</var>&#44; and <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;fragment&#45;parser&#45;mode&#45;legacy"></a>[Legacy](#fragment-parser-mode-legacy)&#46;

6.  Use the first matching item from this list&#58;
    <strong>If <var>position</var> is an <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;ascii&#45;case&#45;insensitive&#45;5"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for the string &#34;<code>beforebegin</code>&#34;</strong>

    <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;concept&#45;node&#45;insert"></a>[Insert](https://dom.spec.whatwg.org/#concept-node-insert) <var>fragment</var> into <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;this&#45;6"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;dom&#45;parent&#45;2"></a>[parent](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-parent) before <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;this&#45;7"></a>[this](https://webidl.spec.whatwg.org/#this)&#46;

    <strong>If <var>position</var> is an <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;ascii&#45;case&#45;insensitive&#45;6"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for the string &#34;<code>afterbegin</code>&#34;</strong>

    <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;concept&#45;node&#45;insert&#45;2"></a>[Insert](https://dom.spec.whatwg.org/#concept-node-insert) <var>fragment</var> into <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;this&#45;8"></a>[this](https://webidl.spec.whatwg.org/#this) before its <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;first&#45;child"></a>[first child](https://dom.spec.whatwg.org/#concept-tree-first-child)&#46;

    <strong>If <var>position</var> is an <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;ascii&#45;case&#45;insensitive&#45;7"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for the string &#34;<code>beforeend</code>&#34;</strong>

    <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;concept&#45;node&#45;append"></a>[Append](https://dom.spec.whatwg.org/#concept-node-append) <var>fragment</var> to <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;this&#45;9"></a>[this](https://webidl.spec.whatwg.org/#this)&#46;

    <strong>If <var>position</var> is an <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;ascii&#45;case&#45;insensitive&#45;8"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for the string &#34;<code>afterend</code>&#34;</strong>

    <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;concept&#45;node&#45;insert&#45;3"></a>[Insert](https://dom.spec.whatwg.org/#concept-node-insert) <var>fragment</var> into <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;this&#45;10"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;dom&#45;parent&#45;3"></a>[parent](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-parent) before <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;this&#45;11"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;next&#45;sibling"></a>[next sibling](https://dom.spec.whatwg.org/#concept-tree-next-sibling)&#46;

<strong>Note&#58;</strong>

As with other direct <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;node"></a><code>[Node](https://dom.spec.whatwg.org/#interface-node)</code>&#45;manipulation APIs &#40;and unlike <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;dom&#45;element&#45;innerhtml"></a><code>[innerHTML](#dom-element-innerhtml)</code>&#41;&#44; <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;dom&#45;element&#45;insertadjacenthtml&#45;3"></a><code>[insertAdjacentHTML&#40;&#41;](#dom-element-insertadjacenthtml)</code> does not include any special handling for <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;the&#45;template&#45;element"></a><code>[template](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-template-element)</code> elements&#46; In most cases you will want to use <code>templateEl&#46;<a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;dom&#45;template&#45;content"></a>[content](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#dom-template-content)&#46;<a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;dom&#45;element&#45;insertadjacenthtml&#45;4"></a>[insertAdjacentHTML&#40;&#41;](#dom-element-insertadjacenthtml)</code> instead of directly manipulating the child nodes of a <a id="the&#45;insertadjacenthtml&#40;&#41;&#45;method&#58;the&#45;template&#45;element&#45;2"></a><code>[template](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-template-element)</code> element&#46;

#### <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method"></a>8&#46;5&#46;7 The <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;dom&#45;range&#45;createcontextualfragment"></a><code>[createContextualFragment&#40;&#41;](#dom-range-createcontextualfragment)</code> method[](#the-createcontextualfragment()-method)

The <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;dom&#45;range&#45;createcontextualfragment&#45;2"></a><code>[createContextualFragment&#40;&#41;](#dom-range-createcontextualfragment)</code> method has a number of outstanding issues in the DOM Parsing and Serialization [issue tracker](https://github.com/w3c/DOM-Parsing/issues)&#44; documenting various problems with its specification&#46;

<strong><code><var>docFragment</var> &#61; <var>range</var>&#46;<a id="dom&#45;range&#45;createcontextualfragment&#45;dev"></a>[createContextualFragment](#dom-range-createcontextualfragment)&#40;<var>string</var>&#41;</code></strong>

Returns a <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;documentfragment"></a><code>[DocumentFragment](https://dom.spec.whatwg.org/#interface-documentfragment)</code> created from the markup string <var>string</var> using <var>range</var>&#39;s <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;concept&#45;range&#45;start&#45;node"></a>[start node](https://dom.spec.whatwg.org/#concept-range-start-node) as the context in which <var>fragment</var> is parsed&#46;

<strong>Warning&#58;</strong>

By default&#44; this method performs no sanitization to remove potentially&#45;dangerous elements and attributes like <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;the&#45;script&#45;element"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> or <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;event&#45;handler&#45;content&#45;attributes"></a>[event handler content attributes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-handler-content-attributes)&#46;

<a id="Range&#45;partial"></a><a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;cereactions"></a><a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;documentfragment&#45;2"></a><a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;dom&#45;range&#45;createcontextualfragment&#45;3"></a><a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;tt&#45;trustedhtml"></a>

``` text
partial interface Range {
  [CEReactions, NewObject] DocumentFragment createContextualFragment((TrustedHTML or DOMString) string);
};
```

<a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;range"></a><code>[Range](https://dom.spec.whatwg.org/#interface-range)</code>&#39;s <a id="dom&#45;range&#45;createcontextualfragment"></a><strong><code>createContextualFragment&#40;<var>string</var>&#41;</code></strong> method steps are&#58;

1.  Let <var>node</var> be <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;this"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;concept&#45;range&#45;start&#45;node&#45;2"></a>[start node](https://dom.spec.whatwg.org/#concept-range-start-node)&#46;

2.  Let &#40;<var>compliantString</var>&#44; <var>compliantOptions</var>&#41; be the result of invoking the <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;tt&#45;getcompliantinput"></a>[get trusted type compliant input](https://w3c.github.io/trusted-types/dist/spec/#get-trusted-type-compliant-input) algorithm with <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;this&#45;2"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;concept&#45;relevant&#45;global"></a>[relevant global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global)&#44; <var>string</var>&#44; a new <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;sethtmlunsafeoptions"></a><code>[SetHTMLUnsafeOptions](#sethtmlunsafeoptions)</code> dictionary whose <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;dom&#45;sethtmlunsafeoptions&#45;runscripts"></a><code>[runScripts](#dom-sethtmlunsafeoptions-runscripts)</code> member is set to true and whose other members are set to their default values&#44; &#34;<code>Range createContextualFragment</code>&#34;&#44; <var>node</var>&#39;s <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;concept&#45;document&#45;type"></a>[type](https://dom.spec.whatwg.org/#concept-document-type)&#44; and false&#46;

    <strong>Note&#58;</strong>

    A Trusted Types default policy can override the options&#44; for example by setting <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;dom&#45;sethtmlunsafeoptions&#45;runscripts&#45;2"></a><code>[runScripts](#dom-sethtmlunsafeoptions-runscripts)</code> to false to prevent scripts from executing when inserting the returned fragment&#46;

3.  Let <var>element</var> be null&#46;

4.  If <var>node</var> <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;implements"></a>[implements](https://webidl.spec.whatwg.org/#implements) <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;element"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code>&#44; set <var>element</var> to <var>node</var>&#46;

5.  Otherwise&#44; if <var>node</var> <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;implements&#45;2"></a>[implements](https://webidl.spec.whatwg.org/#implements) <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;text"></a><code>[Text](https://dom.spec.whatwg.org/#interface-text)</code> or <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;comment&#45;2"></a><code>[Comment](https://dom.spec.whatwg.org/#interface-comment)</code>&#44; set <var>element</var> to <var>node</var>&#39;s <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;parent&#45;element"></a>[parent element](https://dom.spec.whatwg.org/#parent-element)&#46;

6.  If <var>element</var> is null or all of the following are true&#58;

    - <var>element</var>&#39;s <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;node&#45;document&#45;2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document) is an HTML document&#59;

    - <var>element</var>&#39;s <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;concept&#45;element&#45;local&#45;name"></a>[local name](https://dom.spec.whatwg.org/#concept-element-local-name) is &#34;<a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;the&#45;html&#45;element"></a><code>[html](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-html-element)</code>&#34;&#59; and

    - <var>element</var>&#39;s <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;concept&#45;element&#45;namespace"></a>[namespace](https://dom.spec.whatwg.org/#concept-element-namespace) is the <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;html&#45;namespace&#45;2"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace)&#44;

    then set <var>element</var> to the result of <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;create&#45;an&#45;element"></a>[creating an element](https://dom.spec.whatwg.org/#concept-create-element) given <var>node</var>&#39;s <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;node&#45;document&#45;3"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#44; &#34;<code>body</code>&#34;&#44; and the <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;html&#45;namespace&#45;2&#45;2"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace)&#46;

7.  Return the result of invoking the <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;fragment&#45;parsing&#45;algorithm&#45;steps"></a>[fragment parsing algorithm steps](#fragment-parsing-algorithm-steps) given <var>element</var>&#44; <var>compliantString</var>&#44; <var>compliantOptions</var>&#44; and <a id="the&#45;createcontextualfragment&#40;&#41;&#45;method&#58;fragment&#45;parser&#45;mode&#45;legacy"></a>[Legacy](#fragment-parser-mode-legacy)&#46;

#### <a id="the&#45;xmlserializer&#45;interface"></a>8&#46;5&#46;8 The <a id="the&#45;xmlserializer&#45;interface&#58;xmlserializer"></a><code>[XMLSerializer](#xmlserializer)</code> interface[](#the-xmlserializer-interface)

The <a id="the&#45;xmlserializer&#45;interface&#58;xmlserializer&#45;2"></a><code>[XMLSerializer](#xmlserializer)</code> interface has a number of outstanding issues in the DOM Parsing and Serialization [issue tracker](https://github.com/w3c/DOM-Parsing/issues)&#44; documenting various problems with its specification&#46; The remainder of DOM Parsing and Serialization will be gradually upstreamed to this specification&#46;

<strong><code><var>xmlSerializer</var> &#61; new <a id="dom&#45;xmlserializer&#45;constructor&#45;dev"></a>[XMLSerializer](#dom-xmlserializer-constructor)&#40;&#41;</code></strong>

Constructs a new <a id="the&#45;xmlserializer&#45;interface&#58;xmlserializer&#45;3"></a><code>[XMLSerializer](#xmlserializer)</code> object&#46;

<strong><code><var>string</var> &#61; <var>xmlSerializer</var>&#46;<a id="dom&#45;xmlserializer&#45;serializetostring&#45;dev"></a>[serializeToString](#dom-xmlserializer-serializetostring)&#40;<var>root</var>&#41;</code></strong>

Returns the result of serializing <var>root</var> to XML&#46;

Throws an <a id="the&#45;xmlserializer&#45;interface&#58;invalidstateerror"></a>[&#34;<code>InvalidStateError</code>&#34;](https://webidl.spec.whatwg.org/#invalidstateerror) <a id="the&#45;xmlserializer&#45;interface&#58;domexception"></a><code>[DOMException](https://webidl.spec.whatwg.org/#dfn-DOMException)</code> if <var>root</var> cannot be serialized to XML&#46;

<strong>Note&#58;</strong>

The design of <a id="the&#45;xmlserializer&#45;interface&#58;xmlserializer&#45;4"></a><code>[XMLSerializer](#xmlserializer)</code>&#44; as a class that needs to be constructed and then have its <a id="the&#45;xmlserializer&#45;interface&#58;dom&#45;xmlserializer&#45;serializetostring"></a><code>[serializeToString&#40;&#41;](#dom-xmlserializer-serializetostring)</code> method called&#44; is an unfortunate historical artifact&#46; If we were designing this functionality today it would be a standalone function&#46;

<a id="xmlserializer"></a><a id="the&#45;xmlserializer&#45;interface&#58;dom&#45;xmlserializer&#45;constructor"></a><a id="the&#45;xmlserializer&#45;interface&#58;dom&#45;xmlserializer&#45;serializetostring&#45;2"></a><a id="the&#45;xmlserializer&#45;interface&#58;node"></a>

``` text
[Exposed=Window]
interface XMLSerializer {
  constructor();

  DOMString serializeToString(Node root);
};
```

The <a id="dom&#45;xmlserializer&#45;constructor"></a><strong><code>new XMLSerializer&#40;&#41;</code></strong> constructor steps are to do nothing&#46;

The <a id="dom&#45;xmlserializer&#45;serializetostring"></a><strong><code>serializeToString&#40;<var>root</var>&#41;</code></strong> method steps are&#58;

1.  Return the <a id="the&#45;xmlserializer&#45;interface&#58;xml&#45;serialization"></a>[XML serialization](https://w3c.github.io/DOM-Parsing/#dfn-xml-serialization) of <var>root</var> given false&#46;

##### <a id="enabling&#45;and&#45;disabling&#45;scripting"></a>8&#46;1&#46;3&#46;4 Enabling and disabling scripting[](#enabling-and-disabling-scripting)

<a id="concept&#45;bc&#45;script"></a><a id="concept&#45;environment&#45;script"></a><strong>Scripting is enabled</strong> for an <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;environment&#45;settings&#45;object"></a>[environment settings object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object) <var>settings</var> when all of the following conditions are true&#58;

- The user agent supports scripting&#46;
- <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;tracking&#45;vector"></a>[![&#40;This is a tracking vector&#46;&#41;](https://resources.whatwg.org/tracking-vector.svg)](https://infra.spec.whatwg.org/#tracking-vector) The user has not disabled scripting for <var>settings</var> at this time&#46; &#40;User agents may provide users with the option to disable scripting globally&#44; or in a finer&#45;grained manner&#44; e&#46;g&#46;&#44; on a per&#45;origin basis&#44; down to the level of individual <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;environment&#45;settings&#45;object&#45;2"></a>[environment settings objects](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object)&#46;&#41;
- <a id="sandboxScriptBlocked"></a>Either <var>settings</var>&#39;s <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;concept&#45;settings&#45;object&#45;global"></a>[global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-settings-object-global) is not a <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;window"></a><code>[Window](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window)</code> object&#44; or <var>settings</var>&#39;s <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;concept&#45;settings&#45;object&#45;global&#45;2"></a>[global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-settings-object-global)&#39;s <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;concept&#45;document&#45;window"></a>[associated <code>Document</code>](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-window)&#39;s <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;active&#45;sandboxing&#45;flag&#45;set"></a>[active sandboxing flag set](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#active-sandboxing-flag-set) does not have its <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;sandboxed&#45;scripts&#45;browsing&#45;context&#45;flag"></a>[sandboxed scripts browsing context flag](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-scripts-browsing-context-flag) set&#46;
- <a id="WebDriverBiDiScriptingIsEnabled"></a>The result of <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;webdriver&#45;bidi&#45;scripting&#45;is&#45;enabled"></a>[WebDriver BiDi scripting is enabled](https://w3c.github.io/webdriver-bidi/#webdriver-bidi-scripting-is-enabled) with <var>settings</var> is true&#46;

<a id="concept&#45;bc&#45;noscript"></a><a id="concept&#45;environment&#45;noscript"></a><strong>Scripting is disabled</strong> for an <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;environment&#45;settings&#45;object&#45;3"></a>[environment settings object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object) when scripting is not <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;concept&#45;environment&#45;script"></a>[enabled](#concept-environment-script) for it&#44; i&#46;e&#46;&#44; when any of the above conditions are false&#46;

------------------------------------------------------------------------

<a id="concept&#45;n&#45;noscript"></a><strong>Scripting is disabled</strong> for a <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;platform&#45;object"></a>[platform object](https://webidl.spec.whatwg.org/#dfn-platform-object) <var>object</var> if any of the following are true&#58;

- <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;concept&#45;environment&#45;noscript"></a>[Scripting is disabled](#concept-environment-noscript) for <var>object</var>&#39;s <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;relevant&#45;settings&#45;object"></a>[relevant settings object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant-settings-object)&#46;

- The <var>object</var> implements <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;node"></a><code>[Node](https://dom.spec.whatwg.org/#interface-node)</code>&#44; and <var>object</var>&#39;s <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;concept&#45;document&#45;bc"></a>[browsing context](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc) is null&#46;

- The <var>object</var> implements <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;window&#45;2"></a><code>[Window](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window)</code> and <var>object</var>&#39;s <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;concept&#45;document&#45;window&#45;2"></a>[associated <code>Document</code>](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-window)&#39;s <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;concept&#45;document&#45;bc&#45;2"></a>[browsing context](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc) is null&#46;

<a id="concept&#45;n&#45;script"></a><strong>Scripting is enabled</strong> for a <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;platform&#45;object&#45;2"></a>[platform object](https://webidl.spec.whatwg.org/#dfn-platform-object) <var>object</var>&#44; when <var>object</var>&#39;s scripting is not <a id="enabling&#45;and&#45;disabling&#45;scripting&#58;concept&#45;n&#45;noscript"></a>[disabled](#concept-n-noscript)&#46;
