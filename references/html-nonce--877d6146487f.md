# HTML Standard: nonce attributes

Source: [immutable HTML snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nonce-attributes), revision `877d6146487f15b15bc8dd0e3c625063a37b456a` (9 October 2026); complete original HTML retrieved 11 October 2026, 14,930,614 bytes, SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`. This bounded offline extraction was prepared on 13 October 2026 from those same retained bytes; no new edition was acquired.

The complete nonce attribute, slot getter/setter, attribute-change, conditioned insertion/hiding and cloning clauses. Browsing-context connection and mixin eligibility remain linked to the same edition. The extraction starts at `nonce-attributes` and ends before the next equal-or-higher heading `lazy-loading-attributes`. Normalized extraction input: 7,794 bytes, SHA-256 `1ce61900b19cdbad79076014e3a03b4ec1401ceae153bdedd08f43adb594fb5a`.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose is [Creative Commons Attribution 4.0 International](https://creativecommons.org/licenses/by/4.0/), with the [exact WHATWG license](../licenses/whatwg/LICENSE); BSD 3-Clause applies to portions incorporated into code, not as an alternative prose license. Bounded extraction and Markdown reformatting by Surgeist use the existing preparation/filter, approved Pandoc 3.1.11.1 and parsed-GFM comparison. Source text, IDs, links, literal examples, variables, headings and tables are preserved; MDN compatibility annotations and source scripts/styles are omitted without execution. No semantic change, endorsement, browser capability or implementation-conformance claim is implied. The [edition map](SOURCE-EDITIONS.md#remaining-intrinsic-html-source-clauses) records the owning scope.

---

<!-- captured-body-start -->

#### <a id="nonce&#45;attributes"></a>2&#46;5&#46;6 Nonce attributes[](#nonce-attributes)

A <a id="attr&#45;nonce"></a><strong><code>nonce</code></strong> content attribute represents a cryptographic nonce &#40;&#34;number used once&#34;&#41; which can be used by Content Security Policy to determine whether or not a given fetch will be allowed to proceed&#46; The value is text&#46; [&#91;CSP&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsCSP)

Elements that have a <a id="nonce&#45;attributes&#58;attr&#45;nonce"></a><code>[nonce](#attr-nonce)</code> content attribute ensure that the cryptographic nonce is only exposed to script &#40;and not to side&#45;channels like CSS attribute selectors&#41; by taking the value from the content attribute&#44; moving it into an internal slot named <a id="cryptographicnonce"></a><strong>&#91;&#91;CryptographicNonce&#93;&#93;</strong>&#44; exposing it to script via the <a id="nonce&#45;attributes&#58;htmlorsvgelement"></a><code>[HTMLOrSVGOrMathMLElement](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#htmlorsvgelement)</code> interface mixin&#44; and setting the content attribute to the empty string&#46; Unless otherwise specified&#44; the slot&#39;s value is the empty string&#46;

<strong><code><var>element</var>&#46;<a id="dom&#45;htmlorsvgormathmlelement&#45;nonce&#45;dev"></a>[nonce](#dom-noncedelement-nonce)</code></strong>

Returns the value set for <var>element</var>&#39;s cryptographic nonce&#46; If the setter was not used&#44; this will be the value originally found in the <a id="nonce&#45;attributes&#58;attr&#45;nonce&#45;2"></a><code>[nonce](#attr-nonce)</code> content attribute&#46;

<strong><code><var>element</var>&#46;<a id="nonce&#45;attributes&#58;dom&#45;noncedelement&#45;nonce"></a>[nonce](#dom-noncedelement-nonce) &#61; <var>value</var></code></strong>

Updates <var>element</var>&#39;s cryptographic nonce value&#46;

The <a id="dom&#45;noncedelement&#45;nonce"></a><strong><code>nonce</code></strong> IDL attribute must&#44; on getting&#44; return the value of this element&#39;s <a id="nonce&#45;attributes&#58;cryptographicnonce"></a>[&#91;&#91;CryptographicNonce&#93;&#93;](#cryptographicnonce)&#59; and on setting&#44; set this element&#39;s <a id="nonce&#45;attributes&#58;cryptographicnonce&#45;2"></a>[&#91;&#91;CryptographicNonce&#93;&#93;](#cryptographicnonce) to the given value&#46;

<a id="nonce&#45;does&#45;not&#45;update&#45;dom"></a>

<strong>Note&#58;</strong>

[](#nonce-does-not-update-dom)Note how the setter for the <a id="nonce&#45;attributes&#58;dom&#45;noncedelement&#45;nonce&#45;2"></a><code>[nonce](#dom-noncedelement-nonce)</code> IDL attribute does not update the corresponding content attribute&#46; This&#44; as well as the below setting of the <a id="nonce&#45;attributes&#58;attr&#45;nonce&#45;3"></a><code>[nonce](#attr-nonce)</code> content attribute to the empty string when an element <a id="nonce&#45;attributes&#58;becomes&#45;browsing&#45;context&#45;connected"></a>[becomes browsing&#45;context connected](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#becomes-browsing-context-connected)&#44; is meant to prevent exfiltration of the nonce value through mechanisms that can easily read content attributes&#44; such as selectors&#46; Learn more in [issue &#35;2369](https://github.com/whatwg/html/issues/2369)&#44; where this behavior was introduced&#46;

The following <a id="nonce&#45;attributes&#58;concept&#45;element&#45;attributes&#45;change&#45;ext"></a>[attribute change steps](https://dom.spec.whatwg.org/#concept-element-attributes-change-ext) are used for the <a id="nonce&#45;attributes&#58;attr&#45;nonce&#45;4"></a><code>[nonce](#attr-nonce)</code> content attribute&#58;

1.  If <var>element</var> does not <a id="nonce&#45;attributes&#58;include"></a>[include](https://webidl.spec.whatwg.org/#include) <a id="nonce&#45;attributes&#58;htmlorsvgelement&#45;2"></a><code>[HTMLOrSVGOrMathMLElement](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#htmlorsvgelement)</code>&#44; then return&#46;

2.  If <var>localName</var> is not <a id="nonce&#45;attributes&#58;attr&#45;nonce&#45;5"></a><code>[nonce](#attr-nonce)</code> or <var>namespace</var> is not null&#44; then return&#46;

3.  If <var>value</var> is null&#44; then set <var>element</var>&#39;s <a id="nonce&#45;attributes&#58;cryptographicnonce&#45;3"></a>[&#91;&#91;CryptographicNonce&#93;&#93;](#cryptographicnonce) to the empty string&#46;

4.  Otherwise&#44; set <var>element</var>&#39;s <a id="nonce&#45;attributes&#58;cryptographicnonce&#45;4"></a>[&#91;&#91;CryptographicNonce&#93;&#93;](#cryptographicnonce) to <var>value</var>&#46;

Whenever an element <a id="nonce&#45;attributes&#58;include&#45;2"></a>[including](https://webidl.spec.whatwg.org/#include) <a id="nonce&#45;attributes&#58;htmlorsvgelement&#45;3"></a><code>[HTMLOrSVGOrMathMLElement](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#htmlorsvgelement)</code> <a id="nonce&#45;attributes&#58;becomes&#45;browsing&#45;context&#45;connected&#45;2"></a>[becomes browsing&#45;context connected](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#becomes-browsing-context-connected)&#44; the user agent must execute the following steps on the <var>element</var>&#58;

1.  Let <var>CSP list</var> be <var>element</var>&#39;s <a id="nonce&#45;attributes&#58;shadow&#45;including&#45;root"></a>[shadow&#45;including root](https://dom.spec.whatwg.org/#concept-shadow-including-root)&#39;s <a id="nonce&#45;attributes&#58;concept&#45;document&#45;policy&#45;container"></a>[policy container](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-policy-container)&#39;s <a id="nonce&#45;attributes&#58;policy&#45;container&#45;csp&#45;list"></a>[CSP list](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container-csp-list)&#46;

2.  If <var>CSP list</var> <a id="nonce&#45;attributes&#58;contains&#45;a&#45;header&#45;delivered&#45;content&#45;security&#45;policy"></a>[contains a header&#45;delivered Content Security Policy](https://w3c.github.io/webappsec-csp/#contains-a-header-delivered-content-security-policy)&#44; and <var>element</var> has a <a id="nonce&#45;attributes&#58;attr&#45;nonce&#45;6"></a><code>[nonce](#attr-nonce)</code> content attribute whose value is not the empty string&#58;

    1.  Let <var>nonce</var> be <var>element</var>&#39;s <a id="nonce&#45;attributes&#58;cryptographicnonce&#45;5"></a>[&#91;&#91;CryptographicNonce&#93;&#93;](#cryptographicnonce)&#46;

    2.  <a id="nonce&#45;attributes&#58;concept&#45;element&#45;attributes&#45;set&#45;value"></a>[Set an attribute value](https://dom.spec.whatwg.org/#concept-element-attributes-set-value) for <var>element</var> using &#34;<a id="nonce&#45;attributes&#58;attr&#45;nonce&#45;7"></a><code>[nonce](#attr-nonce)</code>&#34; and the empty string&#46;

    3.  Set <var>element</var>&#39;s <a id="nonce&#45;attributes&#58;cryptographicnonce&#45;6"></a>[&#91;&#91;CryptographicNonce&#93;&#93;](#cryptographicnonce) to <var>nonce</var>&#46;

    <strong>Note&#58;</strong>

    If <var>element</var>&#39;s <a id="nonce&#45;attributes&#58;cryptographicnonce&#45;7"></a>[&#91;&#91;CryptographicNonce&#93;&#93;](#cryptographicnonce) were not restored it would be the empty string at this point&#46;

The <a id="nonce&#45;attributes&#58;concept&#45;node&#45;clone&#45;ext"></a>[cloning steps](https://dom.spec.whatwg.org/#concept-node-clone-ext) for elements that <a id="nonce&#45;attributes&#58;include&#45;3"></a>[include](https://webidl.spec.whatwg.org/#include) <a id="nonce&#45;attributes&#58;htmlorsvgelement&#45;4"></a><code>[HTMLOrSVGOrMathMLElement](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#htmlorsvgelement)</code> given <var>node</var>&#44; <var>copy</var>&#44; and <var>subtree</var> are to set <var>copy</var>&#39;s <a id="nonce&#45;attributes&#58;cryptographicnonce&#45;8"></a>[&#91;&#91;CryptographicNonce&#93;&#93;](#cryptographicnonce) to <var>node</var>&#39;s <a id="nonce&#45;attributes&#58;cryptographicnonce&#45;9"></a>[&#91;&#91;CryptographicNonce&#93;&#93;](#cryptographicnonce)&#46;
