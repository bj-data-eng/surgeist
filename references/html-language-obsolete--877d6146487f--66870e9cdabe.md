# HTML Standard: obsolete

Attribution and reformatting notice added for Surgeist on 2026-10-11. Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Original English standard and normative/informative labels remain authoritative. This partial format conversion supports the selected HTML language baseline; it is not a replacement specification or an implementation-conformance claim. No endorsement is implied.

Source: [HTML Standard commit snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/). Source revision `877d6146487f15b15bc8dd0e3c625063a37b456a` (9 October 2026); retrieved 11 October 2026 UTC. This immutable snapshot is the selected language baseline, independently of the html5ever implementation ingestion revision.

The exact [WHATWG LICENSE](../licenses/whatwg/LICENSE), also present in the HTML repository at this revision, has SHA-256 `85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`. Specification prose is Creative Commons Attribution 4.0 International; the BSD 3-Clause scope applies when portions are incorporated into code, not as an alternative license for this Markdown.

Complete original HTML: 14,930,614 bytes, SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`. Complete-section extraction input: 166,499 bytes, SHA-256 `66870e9cdabe63d1709b2c0af75a97018377c008fea5f3619b0ea6a79af367f9`.

Retained complete heading sections, including all descendant clauses and bounded by the next equal-or-higher-level heading: `obsolete`. Original snapshot header is retained. The [edition map](SOURCE-EDITIONS.md#pinned-html-language-baseline) records the ten companion captures and bounded ownership. Every §4 element definition is retained across the collection. Omitted browser runtime chapters remain linked to this immutable snapshot; retaining JavaScript interfaces, networking, media, script execution or custom-element algorithms as source evidence does not assign their implementation to the native HTML crate. HTML syntax, author conformance and required parsing recovery remain distinct; malformed input is not rejected solely for an author-conformance error.

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

## <a id="obsolete"></a>16 Obsolete features[](#obsolete)

### <a id="obsolete&#45;but&#45;conforming&#45;features"></a>16&#46;1 Obsolete but conforming features[](#obsolete-but-conforming-features)

Features listed in this section will trigger warnings in conformance checkers&#46;

Authors should not specify a <a id="obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;img&#45;border"></a><code>[border](#attr-img-border)</code> attribute on an <a id="obsolete&#45;but&#45;conforming&#45;features&#58;the&#45;img&#45;element"></a><code>[img](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-img-element)</code> element&#46; If the attribute is present&#44; its value must be the string &#34;<code>0</code>&#34;&#46; CSS should be used instead&#46;

Authors should not specify a <a id="obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;script&#45;charset"></a><code>[charset](#attr-script-charset)</code> attribute on a <a id="obsolete&#45;but&#45;conforming&#45;features&#58;the&#45;script&#45;element"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> element&#46; If the attribute is present&#44; its value must be an <a id="obsolete&#45;but&#45;conforming&#45;features&#58;ascii&#45;case&#45;insensitive"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for &#34;<code>utf&#45;8</code>&#34;&#46; &#40;This has no effect in a document that conforms to the requirements elsewhere in this standard of being encoded as <a id="obsolete&#45;but&#45;conforming&#45;features&#58;utf&#45;8"></a>[UTF&#45;8](https://encoding.spec.whatwg.org/#utf-8)&#46;&#41;

Authors should not specify a <a id="obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;script&#45;language"></a><code>[language](#attr-script-language)</code> attribute on a <a id="obsolete&#45;but&#45;conforming&#45;features&#58;the&#45;script&#45;element&#45;2"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> element&#46; If the attribute is present&#44; its value must be an <a id="obsolete&#45;but&#45;conforming&#45;features&#58;ascii&#45;case&#45;insensitive&#45;2"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for the string &#34;<code>JavaScript</code>&#34; and either the <a id="obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;script&#45;type"></a><code>[type](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#attr-script-type)</code> attribute must be omitted or its value must be an <a id="obsolete&#45;but&#45;conforming&#45;features&#58;ascii&#45;case&#45;insensitive&#45;3"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for the string &#34;<code>text&#47;javascript</code>&#34;&#46; The attribute should be entirely omitted instead &#40;with the value &#34;<code>JavaScript</code>&#34;&#44; it has no effect&#41;&#44; or replaced with use of the <a id="obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;script&#45;type&#45;2"></a><code>[type](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#attr-script-type)</code> attribute&#46;

Authors should not specify a value for the <a id="obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;script&#45;type&#45;3"></a><code>[type](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#attr-script-type)</code> attribute on <a id="obsolete&#45;but&#45;conforming&#45;features&#58;the&#45;script&#45;element&#45;3"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> elements that is the empty string or a <a id="obsolete&#45;but&#45;conforming&#45;features&#58;javascript&#45;mime&#45;type&#45;essence&#45;match"></a>[JavaScript MIME type essence match](https://mimesniff.spec.whatwg.org/#javascript-mime-type-essence-match)&#46; Instead&#44; they should omit the attribute&#44; which has the same effect&#46;

Authors should not specify a <a id="obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;style&#45;type"></a><code>[type](#attr-style-type)</code> attribute on a <a id="obsolete&#45;but&#45;conforming&#45;features&#58;the&#45;style&#45;element"></a><code>[style](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-style-element)</code> element&#46; If the attribute is present&#44; its value must be an <a id="obsolete&#45;but&#45;conforming&#45;features&#58;ascii&#45;case&#45;insensitive&#45;4"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for &#34;<a id="obsolete&#45;but&#45;conforming&#45;features&#58;text&#47;css"></a><code>[text&#47;css](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#text/css)</code>&#34;&#46;

Authors should not specify the <a id="obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;a&#45;name"></a><code>[name](#attr-a-name)</code> attribute on <a id="obsolete&#45;but&#45;conforming&#45;features&#58;the&#45;a&#45;element"></a><code>[a](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element)</code> elements&#46; If the attribute is present&#44; its value must not be the empty string and must neither be equal to the value of any of the <a id="obsolete&#45;but&#45;conforming&#45;features&#58;concept&#45;id"></a>[IDs](https://dom.spec.whatwg.org/#concept-id) in the element&#39;s <a id="obsolete&#45;but&#45;conforming&#45;features&#58;tree"></a>[tree](https://dom.spec.whatwg.org/#concept-tree) other than the element&#39;s own <a id="obsolete&#45;but&#45;conforming&#45;features&#58;concept&#45;id&#45;2"></a>[ID](https://dom.spec.whatwg.org/#concept-id)&#44; if any&#44; nor be equal to the value of any of the other <a id="obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;a&#45;name&#45;2"></a><code>[name](#attr-a-name)</code> attributes on <a id="obsolete&#45;but&#45;conforming&#45;features&#58;the&#45;a&#45;element&#45;2"></a><code>[a](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element)</code> elements in the element&#39;s <a id="obsolete&#45;but&#45;conforming&#45;features&#58;tree&#45;2"></a>[tree](https://dom.spec.whatwg.org/#concept-tree)&#46; If this attribute is present and the element has an <a id="obsolete&#45;but&#45;conforming&#45;features&#58;concept&#45;id&#45;3"></a>[ID](https://dom.spec.whatwg.org/#concept-id)&#44; then the attribute&#39;s value must be equal to the element&#39;s <a id="obsolete&#45;but&#45;conforming&#45;features&#58;concept&#45;id&#45;4"></a>[ID](https://dom.spec.whatwg.org/#concept-id)&#46; In earlier versions of the language&#44; this attribute was intended as a way to specify possible targets for <a id="obsolete&#45;but&#45;conforming&#45;features&#58;concept&#45;url&#45;fragment"></a>[fragments](https://url.spec.whatwg.org/#concept-url-fragment) in <a id="obsolete&#45;but&#45;conforming&#45;features&#58;url"></a>[URLs](https://url.spec.whatwg.org/#concept-url)&#46; The <a id="obsolete&#45;but&#45;conforming&#45;features&#58;the&#45;id&#45;attribute"></a><code>[id](html-language-foundations--877d6146487f--b55e73c31978.md#the-id-attribute)</code> attribute should be used instead&#46;

Authors should not&#44; but may despite requirements to the contrary elsewhere in this specification&#44; specify the <a id="obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;input&#45;maxlength"></a><code>[maxlength](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-maxlength)</code> and <a id="obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;input&#45;size"></a><code>[size](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-size)</code> attributes on <a id="obsolete&#45;but&#45;conforming&#45;features&#58;the&#45;input&#45;element"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements whose <a id="obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;input&#45;type"></a><code>[type](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-type)</code> attributes are in the <a id="obsolete&#45;but&#45;conforming&#45;features&#58;number&#45;state&#45;&#40;type&#61;number&#41;"></a>[Number](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#number-state-(type=number)) state&#46; One valid reason for using these attributes regardless is to help legacy user agents that do not support <a id="obsolete&#45;but&#45;conforming&#45;features&#58;the&#45;input&#45;element&#45;2"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements with <code>type&#61;&#34;number&#34;</code> to still render the text control with a useful width&#46;

#### <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features"></a>16&#46;1&#46;1 Warnings for obsolete but conforming features[](#warnings-for-obsolete-but-conforming-features)

To ease the transition from HTML4 Transitional documents to the language defined in <em>this</em> specification&#44; and to discourage certain features that are only allowed in very few circumstances&#44; conformance checkers must warn the user when the following features are used in a document&#46; These are generally old obsolete features that have no effect&#44; and are allowed only to distinguish between likely mistakes &#40;regular conformance errors&#41; and mere vestigial markup or unusual and discouraged practices &#40;these warnings&#41;&#46;

The following features must be categorized as described above&#58;

- The presence of a <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;img&#45;border"></a><code>[border](#attr-img-border)</code> attribute on an <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;the&#45;img&#45;element"></a><code>[img](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-img-element)</code> element if its value is the string &#34;<code>0</code>&#34;&#46;

- The presence of a <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;script&#45;charset"></a><code>[charset](#attr-script-charset)</code> attribute on a <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;the&#45;script&#45;element"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> element if its value is an <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;ascii&#45;case&#45;insensitive"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for &#34;<code>utf&#45;8</code>&#34;&#46;

- The presence of a <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;script&#45;language"></a><code>[language](#attr-script-language)</code> attribute on a <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;the&#45;script&#45;element&#45;2"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> element if its value is an <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;ascii&#45;case&#45;insensitive&#45;2"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for the string &#34;<code>JavaScript</code>&#34; and if there is no <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;script&#45;type"></a><code>[type](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#attr-script-type)</code> attribute or there is and its value is an <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;ascii&#45;case&#45;insensitive&#45;3"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for the string &#34;<code>text&#47;javascript</code>&#34;&#46;

- The presence of a <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;style&#45;type"></a><code>[type](#attr-style-type)</code> attribute on a <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;the&#45;script&#45;element&#45;3"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> element if its value is a <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;javascript&#45;mime&#45;type&#45;essence&#45;match"></a>[JavaScript MIME type essence match](https://mimesniff.spec.whatwg.org/#javascript-mime-type-essence-match)&#46;

- The presence of a <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;style&#45;type&#45;2"></a><code>[type](#attr-style-type)</code> attribute on a <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;the&#45;style&#45;element"></a><code>[style](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-style-element)</code> element if its value is an <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;ascii&#45;case&#45;insensitive&#45;4"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for &#34;<a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;text&#47;css"></a><code>[text&#47;css](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#text/css)</code>&#34;&#46;

- The presence of a <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;a&#45;name"></a><code>[name](#attr-a-name)</code> attribute on an <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;the&#45;a&#45;element"></a><code>[a](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element)</code> element&#44; if its value is not the empty string&#46;

- The presence of a <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;input&#45;maxlength"></a><code>[maxlength](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-maxlength)</code> attribute on an <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;the&#45;input&#45;element"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> element whose <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;input&#45;type"></a><code>[type](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-type)</code> attribute is in the <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;number&#45;state&#45;&#40;type&#61;number&#41;"></a>[Number](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#number-state-(type=number)) state&#46;

- The presence of a <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;input&#45;size"></a><code>[size](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-size)</code> attribute on an <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;the&#45;input&#45;element&#45;2"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> element whose <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;attr&#45;input&#45;type&#45;2"></a><code>[type](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-type)</code> attribute is in the <a id="warnings&#45;for&#45;obsolete&#45;but&#45;conforming&#45;features&#58;number&#45;state&#45;&#40;type&#61;number&#41;&#45;2"></a>[Number](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#number-state-(type=number)) state&#46;

Conformance checkers must distinguish between pages that have no conformance errors and have none of these obsolete features&#44; and pages that have no conformance errors but do have some of these obsolete features&#46;

<strong>Example&#58;</strong>

For example&#44; a validator could report some pages as &#34;Valid HTML&#34; and others as &#34;Valid HTML with warnings&#34;&#46;

### <a id="non&#45;conforming&#45;features"></a>16&#46;2 Non&#45;conforming features[](#non-conforming-features)

Elements in the following list are entirely obsolete&#44; and must not be used by authors&#58;

<strong><a id="applet"></a><strong><code>applet</code></strong></strong>

Use <a id="non&#45;conforming&#45;features&#58;the&#45;embed&#45;element"></a><code>[embed](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-embed-element)</code> or <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code> instead&#46;

<strong><a id="acronym"></a><strong><code>acronym</code></strong></strong>

Use <a id="non&#45;conforming&#45;features&#58;the&#45;abbr&#45;element"></a><code>[abbr](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-abbr-element)</code> instead&#46;

<strong><a id="bgsound"></a><strong><code>bgsound</code></strong></strong>

Use <a id="non&#45;conforming&#45;features&#58;the&#45;audio&#45;element"></a><code>[audio](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-audio-element)</code> instead&#46;

<strong><a id="dir"></a><strong><code>dir</code></strong></strong>

Use <a id="non&#45;conforming&#45;features&#58;the&#45;ul&#45;element"></a><code>[ul](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-ul-element)</code> instead&#46;

<strong><a id="non&#45;conforming&#45;features&#58;frame"></a><code>[frame](#frame)</code></strong>

<strong><a id="non&#45;conforming&#45;features&#58;frameset"></a><code>[frameset](#frameset)</code></strong>

<strong><a id="noframes"></a><strong><code>noframes</code></strong></strong>

Either use <a id="non&#45;conforming&#45;features&#58;the&#45;iframe&#45;element"></a><code>[iframe](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element)</code> and CSS instead&#44; or use server&#45;side includes to generate complete pages with the various invariant parts merged in&#46;

<strong><a id="isindex"></a><strong><code>isindex</code></strong></strong>

Use an explicit <a id="non&#45;conforming&#45;features&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> and <a id="non&#45;conforming&#45;features&#58;text&#45;&#40;type&#61;text&#41;&#45;state&#45;and&#45;search&#45;state&#45;&#40;type&#61;search&#41;"></a>[text control](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#text-(type=text)-state-and-search-state-(type=search)) combination instead&#46;

<strong><a id="keygen"></a><strong><code>keygen</code></strong></strong>

For enterprise device management use cases&#44; use native on&#45;device management capabilities&#46;

For certificate enrollment use cases&#44; use the Web Cryptography API to generate a keypair for the certificate&#44; and then export the certificate and key to allow the user to install them manually&#46; [&#91;WEBCRYPTO&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsWEBCRYPTO)

<strong><a id="listing"></a><strong><code>listing</code></strong></strong>

Use <a id="non&#45;conforming&#45;features&#58;the&#45;pre&#45;element"></a><code>[pre](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-pre-element)</code> and <a id="non&#45;conforming&#45;features&#58;the&#45;code&#45;element"></a><code>[code](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-code-element)</code> instead&#46;

<strong><a id="menuitem"></a><strong><code>menuitem</code></strong></strong>

To implement a custom context menu&#44; use script to handle the <a id="non&#45;conforming&#45;features&#58;event&#45;contextmenu"></a><code>[contextmenu](https://w3c.github.io/uievents/#event-type-contextmenu)</code> event&#46;

<strong><a id="nextid"></a><strong><code>nextid</code></strong></strong>

Use GUIDs instead&#46;

<strong><a id="noembed"></a><strong><code>noembed</code></strong></strong>

Use <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;2"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code> instead of <a id="non&#45;conforming&#45;features&#58;the&#45;embed&#45;element&#45;2"></a><code>[embed](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-embed-element)</code> when fallback is necessary&#46;

<a id="the&#45;param&#45;element"></a><strong><a id="param"></a><strong><code>param</code></strong></strong>

Use the <a id="non&#45;conforming&#45;features&#58;attr&#45;object&#45;data"></a><code>[data](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#attr-object-data)</code> attribute of the <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;3"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code> element to set the URL of the external resource&#46;

<strong><a id="plaintext"></a><strong><code>plaintext</code></strong></strong>

Use the &#34;<a id="non&#45;conforming&#45;features&#58;text&#47;plain"></a><code>[text&#47;plain](https://www.rfc-editor.org/rfc/rfc2046#section-4.1.3)</code>&#34; <a id="non&#45;conforming&#45;features&#58;mime&#45;type"></a>[MIME type](https://mimesniff.spec.whatwg.org/#mime-type) instead&#46;

<strong><a id="rb"></a><strong><code>rb</code></strong></strong>

<strong><a id="rtc"></a><strong><code>rtc</code></strong></strong>

Providing the ruby base directly inside the <a id="non&#45;conforming&#45;features&#58;the&#45;ruby&#45;element"></a><code>[ruby](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-ruby-element)</code> element or using nested <a id="non&#45;conforming&#45;features&#58;the&#45;ruby&#45;element&#45;2"></a><code>[ruby](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-ruby-element)</code> elements is sufficient&#46;

<strong><a id="strike"></a><strong><code>strike</code></strong></strong>

Use <a id="non&#45;conforming&#45;features&#58;the&#45;del&#45;element"></a><code>[del](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-del-element)</code> instead if the element is marking an edit&#44; otherwise use <a id="non&#45;conforming&#45;features&#58;the&#45;s&#45;element"></a><code>[s](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-s-element)</code> instead&#46;

<strong><a id="xmp"></a><strong><code>xmp</code></strong></strong>

Use <a id="non&#45;conforming&#45;features&#58;the&#45;pre&#45;element&#45;2"></a><code>[pre](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-pre-element)</code> and <a id="non&#45;conforming&#45;features&#58;the&#45;code&#45;element&#45;2"></a><code>[code](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-code-element)</code> instead&#44; and escape &#34;<code>&#60;</code>&#34; and &#34;<code>&#38;</code>&#34; characters as &#34;<code>&#38;lt&#59;</code>&#34; and &#34;<code>&#38;amp&#59;</code>&#34; respectively&#46;

<strong><a id="basefont"></a><strong><code>basefont</code></strong></strong>

<strong><a id="big"></a><strong><code>big</code></strong></strong>

<strong><a id="blink"></a><strong><code>blink</code></strong></strong>

<strong><a id="center"></a><strong><code>center</code></strong></strong>

<strong><a id="font"></a><strong><code>font</code></strong></strong>

<strong><a id="non&#45;conforming&#45;features&#58;the&#45;marquee&#45;element"></a><code>[marquee](#the-marquee-element)</code></strong>

<strong><a id="multicol"></a><strong><code>multicol</code></strong></strong>

<strong><a id="nobr"></a><strong><code>nobr</code></strong></strong>

<strong><a id="spacer"></a><strong><code>spacer</code></strong></strong>

<strong><a id="tt"></a><strong><code>tt</code></strong></strong>

Use appropriate elements or CSS instead&#46;

Where the <a id="non&#45;conforming&#45;features&#58;tt"></a><code>[tt](#tt)</code> element would have been used for marking up keyboard input&#44; consider the <a id="non&#45;conforming&#45;features&#58;the&#45;kbd&#45;element"></a><code>[kbd](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-kbd-element)</code> element&#59; for variables&#44; consider the <a id="non&#45;conforming&#45;features&#58;the&#45;var&#45;element"></a><code>[var](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-var-element)</code> element&#59; for computer code&#44; consider the <a id="non&#45;conforming&#45;features&#58;the&#45;code&#45;element&#45;3"></a><code>[code](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-code-element)</code> element&#59; and for computer output&#44; consider the <a id="non&#45;conforming&#45;features&#58;the&#45;samp&#45;element"></a><code>[samp](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-samp-element)</code> element&#46;

Similarly&#44; if the <a id="non&#45;conforming&#45;features&#58;big"></a><code>[big](#big)</code> element is being used to denote a heading&#44; consider using the <a id="non&#45;conforming&#45;features&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements"></a><code>[h1](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code> element&#59; if it is being used for marking up important passages&#44; consider the <a id="non&#45;conforming&#45;features&#58;the&#45;strong&#45;element"></a><code>[strong](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-strong-element)</code> element&#59; and if it is being used for highlighting text for reference purposes&#44; consider the <a id="non&#45;conforming&#45;features&#58;the&#45;mark&#45;element"></a><code>[mark](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-mark-element)</code> element&#46;

See also the [text&#45;level semantics usage summary](html-language-elements-text-links--877d6146487f--8833a85fd242.md#usage-summary) for more suggestions with examples&#46;

------------------------------------------------------------------------

The following attributes are obsolete &#40;though the elements are still part of the language&#41;&#44; and must not be used by authors&#58;

<strong><a id="attr&#45;a&#45;charset"></a><strong><code>charset</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;a&#45;element"></a><code>[a](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element)</code> elements</strong>

<strong><a id="attr&#45;link&#45;charset"></a><strong><code>charset</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;link&#45;element"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements</strong>

Use an HTTP &#96;<a id="non&#45;conforming&#45;features&#58;content&#45;type"></a><code>[Content&#45;Type](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-type)</code>&#96; header on the linked resource instead&#46;

<strong><a id="attr&#45;script&#45;charset"></a><strong><code>charset</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;script&#45;element"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> elements &#40;except as noted in the previous section&#41;</strong>

Omit the attribute&#46; Both documents and scripts are required to use <a id="non&#45;conforming&#45;features&#58;utf&#45;8"></a>[UTF&#45;8](https://encoding.spec.whatwg.org/#utf-8)&#44; so it is redundant to specify it on the <a id="non&#45;conforming&#45;features&#58;the&#45;script&#45;element&#45;2"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> element since it inherits from the document&#46;

<strong><a id="attr&#45;a&#45;coords"></a><strong><code>coords</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;a&#45;element&#45;2"></a><code>[a](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element)</code> elements</strong>

<strong><a id="attr&#45;a&#45;shape"></a><strong><code>shape</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;a&#45;element&#45;3"></a><code>[a](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element)</code> elements</strong>

Use <a id="non&#45;conforming&#45;features&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> instead of <a id="non&#45;conforming&#45;features&#58;the&#45;a&#45;element&#45;4"></a><code>[a](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element)</code> for image maps&#46;

<strong><a id="attr&#45;a&#45;methods"></a><strong><code>methods</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;a&#45;element&#45;5"></a><code>[a](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element)</code> elements</strong>

<strong><a id="attr&#45;link&#45;methods"></a><strong><code>methods</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;link&#45;element&#45;2"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements</strong>

Use the HTTP OPTIONS feature instead&#46;

<strong><a id="attr&#45;a&#45;name"></a><strong><code>name</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;a&#45;element&#45;6"></a><code>[a](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element)</code> elements &#40;except as noted in the previous section&#41;</strong>

<strong><a id="attr&#45;embed&#45;name"></a><strong><code>name</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;embed&#45;element&#45;3"></a><code>[embed](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-embed-element)</code> elements</strong>

<strong><a id="attr&#45;img&#45;name"></a><strong><code>name</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;img&#45;element"></a><code>[img](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-img-element)</code> elements</strong>

<strong><a id="attr&#45;option&#45;name"></a><strong><code>name</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;option&#45;element"></a><code>[option](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-option-element)</code> elements</strong>

Use the <a id="non&#45;conforming&#45;features&#58;the&#45;id&#45;attribute"></a><code>[id](html-language-foundations--877d6146487f--b55e73c31978.md#the-id-attribute)</code> attribute instead&#46;

<strong><a id="attr&#45;a&#45;rev"></a><strong><code>rev</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;a&#45;element&#45;7"></a><code>[a](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element)</code> elements</strong>

<strong><a id="attr&#45;link&#45;rev"></a><strong><code>rev</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;link&#45;element&#45;3"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements</strong>

Use the <a id="non&#45;conforming&#45;features&#58;attr&#45;hyperlink&#45;rel"></a><code>[rel](html-language-elements-text-links--877d6146487f--8833a85fd242.md#attr-hyperlink-rel)</code> attribute instead&#44; with an opposite term&#46; &#40;For example&#44; instead of <code>rev&#61;&#34;made&#34;</code>&#44; use <code>rel&#61;&#34;author&#34;</code>&#46;&#41;

<strong><a id="attr&#45;a&#45;urn"></a><strong><code>urn</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;a&#45;element&#45;8"></a><code>[a](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element)</code> elements</strong>

<strong><a id="attr&#45;link&#45;urn"></a><strong><code>urn</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;link&#45;element&#45;4"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements</strong>

Specify the preferred persistent identifier using the <a id="non&#45;conforming&#45;features&#58;attr&#45;hyperlink&#45;href"></a><code>[href](html-language-elements-text-links--877d6146487f--8833a85fd242.md#attr-hyperlink-href)</code> attribute instead&#46;

<strong><a id="attr&#45;form&#45;accept"></a><strong><code>accept</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;form&#45;element&#45;2"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements</strong>

Use the <a id="non&#45;conforming&#45;features&#58;attr&#45;input&#45;accept"></a><code>[accept](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-accept)</code> attribute directly on the <a id="non&#45;conforming&#45;features&#58;the&#45;input&#45;element"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements instead&#46;

<strong><a id="attr&#45;area&#45;hreflang"></a><strong><code>hreflang</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;area&#45;element&#45;2"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements</strong>

<strong><a id="attr&#45;area&#45;type"></a><strong><code>type</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;area&#45;element&#45;3"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements</strong>

These attributes do not do anything useful&#44; and for historical reasons there are no corresponding IDL attributes on <a id="non&#45;conforming&#45;features&#58;the&#45;area&#45;element&#45;4"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements&#46; Omit them altogether&#46;

<strong><a id="attr&#45;area&#45;nohref"></a><strong><code>nohref</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;area&#45;element&#45;5"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements</strong>

Omitting the <a id="non&#45;conforming&#45;features&#58;attr&#45;hyperlink&#45;href&#45;2"></a><code>[href](html-language-elements-text-links--877d6146487f--8833a85fd242.md#attr-hyperlink-href)</code> attribute is sufficient&#59; the <a id="non&#45;conforming&#45;features&#58;attr&#45;area&#45;nohref"></a><code>[nohref](#attr-area-nohref)</code> attribute is unnecessary&#46; Omit it altogether&#46;

<strong><a id="attr&#45;head&#45;profile"></a><strong><code>profile</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;head&#45;element"></a><code>[head](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-head-element)</code> elements</strong>

Unnecessary&#46; Omit it altogether&#46;

<strong><a id="attr&#45;html&#45;manifest"></a><strong><code>manifest</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;html&#45;element"></a><code>[html](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-html-element)</code> elements</strong>

Use service workers instead&#46; [&#91;SW&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsSW)

<strong><a id="attr&#45;html&#45;version"></a><strong><code>version</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;html&#45;element&#45;2"></a><code>[html](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-html-element)</code> elements</strong>

Unnecessary&#46; Omit it altogether&#46;

<strong><a id="attr&#45;input&#45;ismap"></a><strong><code>ismap</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;input&#45;element&#45;2"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements</strong>

Unnecessary&#46; Omit it altogether&#46; All <a id="non&#45;conforming&#45;features&#58;the&#45;input&#45;element&#45;3"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements with a <a id="non&#45;conforming&#45;features&#58;attr&#45;input&#45;type"></a><code>[type](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-type)</code> attribute in the <a id="non&#45;conforming&#45;features&#58;image&#45;button&#45;state&#45;&#40;type&#61;image&#41;"></a>[Image Button](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#image-button-state-(type=image)) state are processed as server&#45;side image maps&#46;

<strong><a id="attr&#45;input&#45;usemap"></a><strong><code>usemap</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;input&#45;element&#45;4"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements</strong>

<strong><a id="attr&#45;object&#45;usemap"></a><strong><code>usemap</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;4"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code> elements</strong>

Use the <a id="non&#45;conforming&#45;features&#58;the&#45;img&#45;element&#45;2"></a><code>[img](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-img-element)</code> element for image maps&#46;

<strong><a id="attr&#45;iframe&#45;longdesc"></a><strong><code>longdesc</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;iframe&#45;element&#45;2"></a><code>[iframe](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element)</code> elements</strong>

<strong><a id="attr&#45;img&#45;longdesc"></a><strong><code>longdesc</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;img&#45;element&#45;3"></a><code>[img](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-img-element)</code> elements</strong>

Use a regular <a id="non&#45;conforming&#45;features&#58;the&#45;a&#45;element&#45;9"></a><code>[a](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element)</code> element to link to the description&#44; or &#40;in the case of images&#41; use an <a id="non&#45;conforming&#45;features&#58;image&#45;map"></a>[image map](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#image-map) to provide a link from the image to the image&#39;s description&#46;

<strong><a id="attr&#45;img&#45;lowsrc"></a><strong><code>lowsrc</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;img&#45;element&#45;4"></a><code>[img](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-img-element)</code> elements</strong>

Use a progressive JPEG image &#40;given in the <a id="non&#45;conforming&#45;features&#58;attr&#45;img&#45;src"></a><code>[src](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#attr-img-src)</code> attribute&#41;&#44; instead of using two separate images&#46;

<strong><a id="attr&#45;link&#45;target"></a><strong><code>target</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;link&#45;element&#45;5"></a><code>[link](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-link-element)</code> elements</strong>

Unnecessary&#46; Omit it altogether&#46;

<strong><a id="attr&#45;menu&#45;type"></a><strong><code>type</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;menu&#45;element"></a><code>[menu](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-menu-element)</code> elements</strong>

To implement a custom context menu&#44; use script to handle the <a id="non&#45;conforming&#45;features&#58;event&#45;contextmenu&#45;2"></a><code>[contextmenu](https://w3c.github.io/uievents/#event-type-contextmenu)</code> event&#46; For toolbar menus&#44; omit the attribute&#46;

<strong><a id="attr&#45;menu&#45;label"></a><strong><code>label</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;menu&#45;element&#45;2"></a><code>[menu](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-menu-element)</code> elements</strong>

<strong><a id="attr&#45;contextmenu"></a><strong><code>contextmenu</code></strong> on all elements</strong>

<strong><a id="handler&#45;onshow"></a><strong><code>onshow</code></strong> on all elements</strong>

To implement a custom context menu&#44; use script to handle the <a id="non&#45;conforming&#45;features&#58;event&#45;contextmenu&#45;3"></a><code>[contextmenu](https://w3c.github.io/uievents/#event-type-contextmenu)</code> event&#46;

<strong><a id="attr&#45;meta&#45;scheme"></a><strong><code>scheme</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;meta&#45;element"></a><code>[meta](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-meta-element)</code> elements</strong>

Use only one scheme per field&#44; or make the scheme declaration part of the value&#46;

<strong><a id="attr&#45;object&#45;archive"></a><strong><code>archive</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;5"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code> elements</strong>

<strong><a id="attr&#45;object&#45;classid"></a><strong><code>classid</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;6"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code> elements</strong>

<strong><a id="attr&#45;object&#45;code"></a><strong><code>code</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;7"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code> elements</strong>

<strong><a id="attr&#45;object&#45;codebase"></a><strong><code>codebase</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;8"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code> elements</strong>

<strong><a id="attr&#45;object&#45;codetype"></a><strong><code>codetype</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;9"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code> elements</strong>

Use the <a id="non&#45;conforming&#45;features&#58;attr&#45;object&#45;data&#45;2"></a><code>[data](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#attr-object-data)</code> and <a id="non&#45;conforming&#45;features&#58;attr&#45;object&#45;type"></a><code>[type](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#attr-object-type)</code> attributes to invoke <a id="non&#45;conforming&#45;features&#58;plugin"></a>[plugins](html-language-foundations--877d6146487f--b55e73c31978.md#plugin)&#46;

<strong><a id="attr&#45;object&#45;declare"></a><strong><code>declare</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;10"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code> elements</strong>

Repeat the <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;11"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code> element completely each time the resource is to be reused&#46;

<strong><a id="attr&#45;object&#45;standby"></a><strong><code>standby</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;12"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code> elements</strong>

Optimize the linked resource so that it loads quickly or&#44; at least&#44; incrementally&#46;

<strong><a id="attr&#45;object&#45;typemustmatch"></a><strong><code>typemustmatch</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;13"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code> elements</strong>

Avoid using <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;14"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code> elements with untrusted resources&#46;

<strong><a id="attr&#45;script&#45;language"></a><strong><code>language</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;script&#45;element&#45;3"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> elements &#40;except as noted in the previous section&#41;</strong>

Omit the attribute for JavaScript&#59; for <a id="non&#45;conforming&#45;features&#58;data&#45;block"></a>[data blocks](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#data-block)&#44; use the <a id="non&#45;conforming&#45;features&#58;attr&#45;script&#45;type"></a><code>[type](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#attr-script-type)</code> attribute instead&#46;

<strong><a id="attr&#45;script&#45;event"></a><strong><code>event</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;script&#45;element&#45;4"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> elements</strong>

<strong><a id="attr&#45;script&#45;for"></a><strong><code>for</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;script&#45;element&#45;5"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> elements</strong>

Use DOM events mechanisms to register event listeners&#46; [&#91;DOM&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsDOM)

<strong><a id="attr&#45;style&#45;type"></a><strong><code>type</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;style&#45;element"></a><code>[style](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-style-element)</code> elements &#40;except as noted in the previous section&#41;</strong>

Omit the attribute for CSS&#59; for <a id="non&#45;conforming&#45;features&#58;data&#45;block&#45;2"></a>[data blocks](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#data-block)&#44; use <a id="non&#45;conforming&#45;features&#58;the&#45;script&#45;element&#45;6"></a><code>[script](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-script-element)</code> as the container instead of <a id="non&#45;conforming&#45;features&#58;the&#45;style&#45;element&#45;2"></a><code>[style](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-style-element)</code>&#46;

<strong><a id="attr&#45;table&#45;datapagesize"></a><strong><code>datapagesize</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;table&#45;element"></a><code>[table](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-table-element)</code> elements</strong>

Unnecessary&#46; Omit it altogether&#46;

<strong><a id="attr&#45;table&#45;summary"></a><strong><code>summary</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;table&#45;element&#45;2"></a><code>[table](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-table-element)</code> elements</strong>

Use one of the [techniques for describing tables](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#table-descriptions-techniques) given in the <a id="non&#45;conforming&#45;features&#58;the&#45;table&#45;element&#45;3"></a><code>[table](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-table-element)</code> section instead&#46;

<strong><a id="attr&#45;td&#45;abbr"></a><strong><code>abbr</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;td&#45;element"></a><code>[td](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-td-element)</code> elements</strong>

Use text that begins in an unambiguous and terse manner&#44; and include any more elaborate text after that&#46; The <a id="non&#45;conforming&#45;features&#58;attr&#45;title"></a><code>[title](html-language-foundations--877d6146487f--b55e73c31978.md#attr-title)</code> attribute can also be useful in including more detailed text&#44; so that the cell&#39;s contents can be made terse&#46; If it&#39;s a heading&#44; use <a id="non&#45;conforming&#45;features&#58;the&#45;th&#45;element"></a><code>[th](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-th-element)</code> &#40;which has an <a id="non&#45;conforming&#45;features&#58;attr&#45;th&#45;abbr"></a><code>[abbr](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-th-abbr)</code> attribute&#41;&#46;

<strong><a id="attr&#45;tdth&#45;axis"></a><strong><code>axis</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;td&#45;element&#45;2"></a><code>[td](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-td-element)</code> and <a id="non&#45;conforming&#45;features&#58;the&#45;th&#45;element&#45;2"></a><code>[th](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-th-element)</code> elements</strong>

Use the <a id="non&#45;conforming&#45;features&#58;attr&#45;th&#45;scope"></a><code>[scope](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-th-scope)</code> attribute on the relevant <a id="non&#45;conforming&#45;features&#58;the&#45;th&#45;element&#45;3"></a><code>[th](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-th-element)</code>&#46;

<strong><a id="attr&#45;td&#45;scope"></a><strong><code>scope</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;td&#45;element&#45;3"></a><code>[td](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-td-element)</code> elements</strong>

Use <a id="non&#45;conforming&#45;features&#58;the&#45;th&#45;element&#45;4"></a><code>[th](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-th-element)</code> elements for heading cells&#46;

<strong><a id="attr&#45;datasrc"></a><strong><code>datasrc</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;a&#45;element&#45;10"></a><code>[a](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;button&#45;element"></a><code>[button](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-button-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;div&#45;element"></a><code>[div](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-div-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;frame&#45;2"></a><code>[frame](#frame)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;iframe&#45;element&#45;3"></a><code>[iframe](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;img&#45;element&#45;5"></a><code>[img](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-img-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;input&#45;element&#45;5"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;label&#45;element"></a><code>[label](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-label-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;legend&#45;element"></a><code>[legend](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-legend-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;marquee&#45;element&#45;2"></a><code>[marquee](#the-marquee-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;15"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;option&#45;element&#45;2"></a><code>[option](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-option-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;select&#45;element"></a><code>[select](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-select-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;span&#45;element"></a><code>[span](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-span-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;table&#45;element&#45;4"></a><code>[table](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-table-element)</code>&#44; and <a id="non&#45;conforming&#45;features&#58;the&#45;textarea&#45;element"></a><code>[textarea](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-textarea-element)</code> elements</strong>

<strong><a id="attr&#45;datafld"></a><strong><code>datafld</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;a&#45;element&#45;11"></a><code>[a](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;button&#45;element&#45;2"></a><code>[button](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-button-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;div&#45;element&#45;2"></a><code>[div](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-div-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;fieldset&#45;element"></a><code>[fieldset](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-fieldset-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;frame&#45;3"></a><code>[frame](#frame)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;iframe&#45;element&#45;4"></a><code>[iframe](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;img&#45;element&#45;6"></a><code>[img](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-img-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;input&#45;element&#45;6"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;label&#45;element&#45;2"></a><code>[label](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-label-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;legend&#45;element&#45;2"></a><code>[legend](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-legend-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;marquee&#45;element&#45;3"></a><code>[marquee](#the-marquee-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;16"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;select&#45;element&#45;2"></a><code>[select](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-select-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;span&#45;element&#45;2"></a><code>[span](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-span-element)</code>&#44; and <a id="non&#45;conforming&#45;features&#58;the&#45;textarea&#45;element&#45;2"></a><code>[textarea](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-textarea-element)</code> elements</strong>

<strong><a id="attr&#45;dataformatas"></a><strong><code>dataformatas</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;button&#45;element&#45;3"></a><code>[button](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-button-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;div&#45;element&#45;3"></a><code>[div](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-div-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;input&#45;element&#45;7"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;label&#45;element&#45;3"></a><code>[label](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-label-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;legend&#45;element&#45;3"></a><code>[legend](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-legend-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;marquee&#45;element&#45;4"></a><code>[marquee](#the-marquee-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;17"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;option&#45;element&#45;3"></a><code>[option](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-option-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;select&#45;element&#45;3"></a><code>[select](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-select-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;span&#45;element&#45;3"></a><code>[span](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-span-element)</code>&#44; and <a id="non&#45;conforming&#45;features&#58;the&#45;table&#45;element&#45;5"></a><code>[table](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-table-element)</code> elements</strong>

Use script and a mechanism such as <a id="non&#45;conforming&#45;features&#58;xmlhttprequest"></a><code>[XMLHttpRequest](https://xhr.spec.whatwg.org/#xmlhttprequest)</code> to populate the page dynamically&#46; [&#91;XHR&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsXHR)

<strong><a id="attr&#45;dropzone"></a><strong><code>dropzone</code></strong> on all elements</strong>

Use script to handle the <a id="non&#45;conforming&#45;features&#58;event&#45;dnd&#45;dragenter"></a><code>[dragenter](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-dnd-dragenter)</code> and <a id="non&#45;conforming&#45;features&#58;event&#45;dnd&#45;dragover"></a><code>[dragover](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-dnd-dragover)</code> events instead&#46;

<strong><a id="attr&#45;body&#45;alink"></a><strong><code>alink</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;body&#45;element"></a><code>[body](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-body-element)</code> elements</strong>

<strong><a id="attr&#45;body&#45;bgcolor"></a><strong><code>bgcolor</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;body&#45;element&#45;2"></a><code>[body](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-body-element)</code> elements</strong>

<strong><a id="attr&#45;body&#45;bottommargin"></a><strong><code>bottommargin</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;body&#45;element&#45;3"></a><code>[body](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-body-element)</code> elements</strong>

<strong><a id="attr&#45;body&#45;leftmargin"></a><strong><code>leftmargin</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;body&#45;element&#45;4"></a><code>[body](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-body-element)</code> elements</strong>

<strong><a id="attr&#45;body&#45;link"></a><strong><code>link</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;body&#45;element&#45;5"></a><code>[body](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-body-element)</code> elements</strong>

<strong><a id="attr&#45;body&#45;marginheight"></a><strong><code>marginheight</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;body&#45;element&#45;6"></a><code>[body](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-body-element)</code> elements</strong>

<strong><a id="attr&#45;body&#45;marginwidth"></a><strong><code>marginwidth</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;body&#45;element&#45;7"></a><code>[body](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-body-element)</code> elements</strong>

<strong><a id="attr&#45;body&#45;rightmargin"></a><strong><code>rightmargin</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;body&#45;element&#45;8"></a><code>[body](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-body-element)</code> elements</strong>

<strong><a id="attr&#45;body&#45;text"></a><strong><code>text</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;body&#45;element&#45;9"></a><code>[body](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-body-element)</code> elements</strong>

<strong><a id="attr&#45;body&#45;topmargin"></a><strong><code>topmargin</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;body&#45;element&#45;10"></a><code>[body](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-body-element)</code> elements</strong>

<strong><a id="attr&#45;body&#45;vlink"></a><strong><code>vlink</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;body&#45;element&#45;11"></a><code>[body](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-body-element)</code> elements</strong>

<strong><a id="attr&#45;br&#45;clear"></a><strong><code>clear</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;br&#45;element"></a><code>[br](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-br-element)</code> elements</strong>

<strong><a id="attr&#45;caption&#45;align"></a><strong><code>align</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;caption&#45;element"></a><code>[caption](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-caption-element)</code> elements</strong>

<strong><a id="attr&#45;col&#45;align"></a><strong><code>align</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;col&#45;element"></a><code>[col](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-col-element)</code> elements</strong>

<strong><a id="attr&#45;col&#45;char"></a><strong><code>char</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;col&#45;element&#45;2"></a><code>[col](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-col-element)</code> elements</strong>

<strong><a id="attr&#45;col&#45;charoff"></a><strong><code>charoff</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;col&#45;element&#45;3"></a><code>[col](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-col-element)</code> elements</strong>

<strong><a id="attr&#45;col&#45;valign"></a><strong><code>valign</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;col&#45;element&#45;4"></a><code>[col](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-col-element)</code> elements</strong>

<strong><a id="attr&#45;col&#45;width"></a><strong><code>width</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;col&#45;element&#45;5"></a><code>[col](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-col-element)</code> elements</strong>

<strong><a id="attr&#45;div&#45;align"></a><strong><code>align</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;div&#45;element&#45;4"></a><code>[div](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-div-element)</code> elements</strong>

<strong><a id="attr&#45;dl&#45;compact"></a><strong><code>compact</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;dl&#45;element"></a><code>[dl](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-dl-element)</code> elements</strong>

<strong><a id="attr&#45;embed&#45;align"></a><strong><code>align</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;embed&#45;element&#45;4"></a><code>[embed](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-embed-element)</code> elements</strong>

<strong><a id="attr&#45;embed&#45;hspace"></a><strong><code>hspace</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;embed&#45;element&#45;5"></a><code>[embed](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-embed-element)</code> elements</strong>

<strong><a id="attr&#45;embed&#45;vspace"></a><strong><code>vspace</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;embed&#45;element&#45;6"></a><code>[embed](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-embed-element)</code> elements</strong>

<strong><a id="attr&#45;hr&#45;align"></a><strong><code>align</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;hr&#45;element"></a><code>[hr](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-hr-element)</code> elements</strong>

<strong><a id="attr&#45;hr&#45;color"></a><strong><code>color</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;hr&#45;element&#45;2"></a><code>[hr](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-hr-element)</code> elements</strong>

<strong><a id="attr&#45;hr&#45;noshade"></a><strong><code>noshade</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;hr&#45;element&#45;3"></a><code>[hr](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-hr-element)</code> elements</strong>

<strong><a id="attr&#45;hr&#45;size"></a><strong><code>size</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;hr&#45;element&#45;4"></a><code>[hr](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-hr-element)</code> elements</strong>

<strong><a id="attr&#45;hr&#45;width"></a><strong><code>width</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;hr&#45;element&#45;5"></a><code>[hr](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-hr-element)</code> elements</strong>

<strong><a id="attr&#45;hx&#45;align"></a><strong><code>align</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;2"></a><code>[h1](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>—<a id="non&#45;conforming&#45;features&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;3"></a><code>[h6](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code> elements</strong>

<strong><a id="attr&#45;iframe&#45;align"></a><strong><code>align</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;iframe&#45;element&#45;5"></a><code>[iframe](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element)</code> elements</strong>

<strong><a id="attr&#45;iframe&#45;allowtransparency"></a><strong><code>allowtransparency</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;iframe&#45;element&#45;6"></a><code>[iframe](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element)</code> elements</strong>

<strong><a id="attr&#45;iframe&#45;frameborder"></a><strong><code>frameborder</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;iframe&#45;element&#45;7"></a><code>[iframe](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element)</code> elements</strong>

<strong><a id="attr&#45;iframe&#45;framespacing"></a><strong><code>framespacing</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;iframe&#45;element&#45;8"></a><code>[iframe](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element)</code> elements</strong>

<strong><a id="attr&#45;iframe&#45;hspace"></a><strong><code>hspace</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;iframe&#45;element&#45;9"></a><code>[iframe](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element)</code> elements</strong>

<strong><a id="attr&#45;iframe&#45;marginheight"></a><strong><code>marginheight</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;iframe&#45;element&#45;10"></a><code>[iframe](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element)</code> elements</strong>

<strong><a id="attr&#45;iframe&#45;marginwidth"></a><strong><code>marginwidth</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;iframe&#45;element&#45;11"></a><code>[iframe](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element)</code> elements</strong>

<strong><a id="attr&#45;iframe&#45;scrolling"></a><strong><code>scrolling</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;iframe&#45;element&#45;12"></a><code>[iframe](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element)</code> elements</strong>

<strong><a id="attr&#45;iframe&#45;vspace"></a><strong><code>vspace</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;iframe&#45;element&#45;13"></a><code>[iframe](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element)</code> elements</strong>

<strong><a id="attr&#45;input&#45;align"></a><strong><code>align</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;input&#45;element&#45;8"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements</strong>

<strong><a id="attr&#45;input&#45;border"></a><strong><code>border</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;input&#45;element&#45;9"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements</strong>

<strong><a id="attr&#45;input&#45;hspace"></a><strong><code>hspace</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;input&#45;element&#45;10"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements</strong>

<strong><a id="attr&#45;input&#45;vspace"></a><strong><code>vspace</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;input&#45;element&#45;11"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements</strong>

<strong><a id="attr&#45;img&#45;align"></a><strong><code>align</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;img&#45;element&#45;7"></a><code>[img](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-img-element)</code> elements</strong>

<strong><a id="attr&#45;img&#45;border"></a><strong><code>border</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;img&#45;element&#45;8"></a><code>[img](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-img-element)</code> elements &#40;except as noted in the previous section&#41;</strong>

<strong><a id="attr&#45;img&#45;hspace"></a><strong><code>hspace</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;img&#45;element&#45;9"></a><code>[img](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-img-element)</code> elements</strong>

<strong><a id="attr&#45;img&#45;vspace"></a><strong><code>vspace</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;img&#45;element&#45;10"></a><code>[img](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-img-element)</code> elements</strong>

<strong><a id="attr&#45;legend&#45;align"></a><strong><code>align</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;legend&#45;element&#45;4"></a><code>[legend](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-legend-element)</code> elements</strong>

<strong><a id="attr&#45;li&#45;type"></a><strong><code>type</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;li&#45;element"></a><code>[li](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-li-element)</code> elements</strong>

<strong><a id="attr&#45;menu&#45;compact"></a><strong><code>compact</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;menu&#45;element&#45;3"></a><code>[menu](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-menu-element)</code> elements</strong>

<strong><a id="attr&#45;object&#45;align"></a><strong><code>align</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;18"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code> elements</strong>

<strong><a id="attr&#45;object&#45;border"></a><strong><code>border</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;19"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code> elements</strong>

<strong><a id="attr&#45;object&#45;hspace"></a><strong><code>hspace</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;20"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code> elements</strong>

<strong><a id="attr&#45;object&#45;vspace"></a><strong><code>vspace</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;object&#45;element&#45;21"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code> elements</strong>

<strong><a id="attr&#45;ol&#45;compact"></a><strong><code>compact</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;ol&#45;element"></a><code>[ol](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-ol-element)</code> elements</strong>

<strong><a id="attr&#45;p&#45;align"></a><strong><code>align</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;p&#45;element"></a><code>[p](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-p-element)</code> elements</strong>

<strong><a id="attr&#45;pre&#45;width"></a><strong><code>width</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;pre&#45;element&#45;3"></a><code>[pre](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-pre-element)</code> elements</strong>

<strong><a id="attr&#45;table&#45;align"></a><strong><code>align</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;table&#45;element&#45;6"></a><code>[table](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-table-element)</code> elements</strong>

<strong><a id="attr&#45;table&#45;bgcolor"></a><strong><code>bgcolor</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;table&#45;element&#45;7"></a><code>[table](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-table-element)</code> elements</strong>

<strong><a id="attr&#45;table&#45;border"></a><strong><code>border</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;table&#45;element&#45;8"></a><code>[table](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-table-element)</code> elements</strong>

<strong><a id="attr&#45;table&#45;bordercolor"></a><strong><code>bordercolor</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;table&#45;element&#45;9"></a><code>[table](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-table-element)</code> elements</strong>

<strong><a id="attr&#45;table&#45;cellpadding"></a><strong><code>cellpadding</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;table&#45;element&#45;10"></a><code>[table](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-table-element)</code> elements</strong>

<strong><a id="attr&#45;table&#45;cellspacing"></a><strong><code>cellspacing</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;table&#45;element&#45;11"></a><code>[table](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-table-element)</code> elements</strong>

<strong><a id="attr&#45;table&#45;frame"></a><strong><code>frame</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;table&#45;element&#45;12"></a><code>[table](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-table-element)</code> elements</strong>

<strong><a id="attr&#45;table&#45;height"></a><strong><code>height</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;table&#45;element&#45;13"></a><code>[table](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-table-element)</code> elements</strong>

<strong><a id="attr&#45;table&#45;rules"></a><strong><code>rules</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;table&#45;element&#45;14"></a><code>[table](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-table-element)</code> elements</strong>

<strong><a id="attr&#45;table&#45;width"></a><strong><code>width</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;table&#45;element&#45;15"></a><code>[table](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-table-element)</code> elements</strong>

<strong><a id="attr&#45;tbody&#45;align"></a><strong><code>align</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;tbody&#45;element"></a><code>[tbody](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-tbody-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;thead&#45;element"></a><code>[thead](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-thead-element)</code>&#44; and <a id="non&#45;conforming&#45;features&#58;the&#45;tfoot&#45;element"></a><code>[tfoot](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-tfoot-element)</code> elements</strong>

<strong><a id="attr&#45;tbody&#45;char"></a><strong><code>char</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;tbody&#45;element&#45;2"></a><code>[tbody](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-tbody-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;thead&#45;element&#45;2"></a><code>[thead](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-thead-element)</code>&#44; and <a id="non&#45;conforming&#45;features&#58;the&#45;tfoot&#45;element&#45;2"></a><code>[tfoot](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-tfoot-element)</code> elements</strong>

<strong><a id="attr&#45;tbody&#45;charoff"></a><strong><code>charoff</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;tbody&#45;element&#45;3"></a><code>[tbody](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-tbody-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;thead&#45;element&#45;3"></a><code>[thead](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-thead-element)</code>&#44; and <a id="non&#45;conforming&#45;features&#58;the&#45;tfoot&#45;element&#45;3"></a><code>[tfoot](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-tfoot-element)</code> elements</strong>

<strong><a id="attr&#45;tbody&#45;height"></a><strong><code>height</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;thead&#45;element&#45;4"></a><code>[thead](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-thead-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;tbody&#45;element&#45;4"></a><code>[tbody](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-tbody-element)</code>&#44; and <a id="non&#45;conforming&#45;features&#58;the&#45;tfoot&#45;element&#45;4"></a><code>[tfoot](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-tfoot-element)</code> elements</strong>

<strong><a id="attr&#45;tbody&#45;valign"></a><strong><code>valign</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;tbody&#45;element&#45;5"></a><code>[tbody](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-tbody-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;thead&#45;element&#45;5"></a><code>[thead](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-thead-element)</code>&#44; and <a id="non&#45;conforming&#45;features&#58;the&#45;tfoot&#45;element&#45;5"></a><code>[tfoot](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-tfoot-element)</code> elements</strong>

<strong><a id="attr&#45;tdth&#45;align"></a><strong><code>align</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;td&#45;element&#45;4"></a><code>[td](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-td-element)</code> and <a id="non&#45;conforming&#45;features&#58;the&#45;th&#45;element&#45;5"></a><code>[th](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-th-element)</code> elements</strong>

<strong><a id="attr&#45;tdth&#45;bgcolor"></a><strong><code>bgcolor</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;td&#45;element&#45;5"></a><code>[td](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-td-element)</code> and <a id="non&#45;conforming&#45;features&#58;the&#45;th&#45;element&#45;6"></a><code>[th](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-th-element)</code> elements</strong>

<strong><a id="attr&#45;tdth&#45;char"></a><strong><code>char</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;td&#45;element&#45;6"></a><code>[td](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-td-element)</code> and <a id="non&#45;conforming&#45;features&#58;the&#45;th&#45;element&#45;7"></a><code>[th](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-th-element)</code> elements</strong>

<strong><a id="attr&#45;tdth&#45;charoff"></a><strong><code>charoff</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;td&#45;element&#45;7"></a><code>[td](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-td-element)</code> and <a id="non&#45;conforming&#45;features&#58;the&#45;th&#45;element&#45;8"></a><code>[th](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-th-element)</code> elements</strong>

<strong><a id="attr&#45;tdth&#45;height"></a><strong><code>height</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;td&#45;element&#45;8"></a><code>[td](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-td-element)</code> and <a id="non&#45;conforming&#45;features&#58;the&#45;th&#45;element&#45;9"></a><code>[th](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-th-element)</code> elements</strong>

<strong><a id="attr&#45;tdth&#45;nowrap"></a><strong><code>nowrap</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;td&#45;element&#45;9"></a><code>[td](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-td-element)</code> and <a id="non&#45;conforming&#45;features&#58;the&#45;th&#45;element&#45;10"></a><code>[th](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-th-element)</code> elements</strong>

<strong><a id="attr&#45;tdth&#45;valign"></a><strong><code>valign</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;td&#45;element&#45;10"></a><code>[td](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-td-element)</code> and <a id="non&#45;conforming&#45;features&#58;the&#45;th&#45;element&#45;11"></a><code>[th](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-th-element)</code> elements</strong>

<strong><a id="attr&#45;tdth&#45;width"></a><strong><code>width</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;td&#45;element&#45;11"></a><code>[td](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-td-element)</code> and <a id="non&#45;conforming&#45;features&#58;the&#45;th&#45;element&#45;12"></a><code>[th](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-th-element)</code> elements</strong>

<strong><a id="attr&#45;tr&#45;align"></a><strong><code>align</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;tr&#45;element"></a><code>[tr](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-tr-element)</code> elements</strong>

<strong><a id="attr&#45;tr&#45;bgcolor"></a><strong><code>bgcolor</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;tr&#45;element&#45;2"></a><code>[tr](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-tr-element)</code> elements</strong>

<strong><a id="attr&#45;tr&#45;char"></a><strong><code>char</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;tr&#45;element&#45;3"></a><code>[tr](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-tr-element)</code> elements</strong>

<strong><a id="attr&#45;tr&#45;charoff"></a><strong><code>charoff</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;tr&#45;element&#45;4"></a><code>[tr](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-tr-element)</code> elements</strong>

<strong><a id="attr&#45;tr&#45;height"></a><strong><code>height</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;tr&#45;element&#45;5"></a><code>[tr](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-tr-element)</code> elements</strong>

<strong><a id="attr&#45;tr&#45;valign"></a><strong><code>valign</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;tr&#45;element&#45;6"></a><code>[tr](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-tr-element)</code> elements</strong>

<strong><a id="attr&#45;ul&#45;compact"></a><strong><code>compact</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;ul&#45;element&#45;2"></a><code>[ul](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-ul-element)</code> elements</strong>

<strong><a id="attr&#45;ul&#45;type"></a><strong><code>type</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;ul&#45;element&#45;3"></a><code>[ul](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-ul-element)</code> elements</strong>

<strong><a id="attr&#45;background"></a><strong><code>background</code></strong> on <a id="non&#45;conforming&#45;features&#58;the&#45;body&#45;element&#45;12"></a><code>[body](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-body-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;table&#45;element&#45;16"></a><code>[table](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-table-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;thead&#45;element&#45;6"></a><code>[thead](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-thead-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;tbody&#45;element&#45;6"></a><code>[tbody](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-tbody-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;tfoot&#45;element&#45;6"></a><code>[tfoot](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-tfoot-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;tr&#45;element&#45;7"></a><code>[tr](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-tr-element)</code>&#44; <a id="non&#45;conforming&#45;features&#58;the&#45;td&#45;element&#45;12"></a><code>[td](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-td-element)</code>&#44; and <a id="non&#45;conforming&#45;features&#58;the&#45;th&#45;element&#45;13"></a><code>[th](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-th-element)</code> elements</strong>

Use CSS instead&#46;

### <a id="requirements&#45;for&#45;implementations"></a>16&#46;3 Requirements for implementations[](#requirements-for-implementations)

#### <a id="the&#45;marquee&#45;element"></a>16&#46;3&#46;1 The <strong><code>marquee</code></strong> element[](#the-marquee-element)

The <a id="the&#45;marquee&#45;element&#58;the&#45;marquee&#45;element"></a><code>[marquee](#the-marquee-element)</code> element is a presentational element that animates content&#46; CSS transitions and animations are a more appropriate mechanism&#46; [&#91;CSSANIMATIONS&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsCSSANIMATIONS) [&#91;CSSTRANSITIONS&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsCSSTRANSITIONS)

The <a id="the&#45;marquee&#45;element&#58;the&#45;marquee&#45;element&#45;2"></a><code>[marquee](#the-marquee-element)</code> element must implement the <a id="the&#45;marquee&#45;element&#58;htmlmarqueeelement"></a><code>[HTMLMarqueeElement](#htmlmarqueeelement)</code> interface&#46;

<a id="htmlmarqueeelement"></a><a id="the&#45;marquee&#45;element&#58;htmlelement"></a><a id="the&#45;marquee&#45;element&#58;htmlconstructor"></a><a id="the&#45;marquee&#45;element&#58;cereactions"></a><a id="the&#45;marquee&#45;element&#58;xattr&#45;reflect"></a><a id="dom&#45;marquee&#45;behavior"></a><a id="the&#45;marquee&#45;element&#58;cereactions&#45;2"></a><a id="the&#45;marquee&#45;element&#58;xattr&#45;reflect&#45;2"></a><a id="dom&#45;marquee&#45;bgcolor"></a><a id="the&#45;marquee&#45;element&#58;cereactions&#45;3"></a><a id="the&#45;marquee&#45;element&#58;xattr&#45;reflect&#45;3"></a><a id="dom&#45;marquee&#45;direction"></a><a id="the&#45;marquee&#45;element&#58;cereactions&#45;4"></a><a id="the&#45;marquee&#45;element&#58;xattr&#45;reflect&#45;4"></a><a id="dom&#45;marquee&#45;height"></a><a id="the&#45;marquee&#45;element&#58;cereactions&#45;5"></a><a id="the&#45;marquee&#45;element&#58;xattr&#45;reflect&#45;5"></a><a id="dom&#45;marquee&#45;hspace"></a><a id="the&#45;marquee&#45;element&#58;cereactions&#45;6"></a><a id="the&#45;marquee&#45;element&#58;dom&#45;marquee&#45;loop"></a><a id="the&#45;marquee&#45;element&#58;cereactions&#45;7"></a><a id="the&#45;marquee&#45;element&#58;xattr&#45;reflect&#45;6"></a><a id="the&#45;marquee&#45;element&#58;xattr&#45;reflectdefault"></a><a id="dom&#45;marquee&#45;scrollamount"></a><a id="the&#45;marquee&#45;element&#58;cereactions&#45;8"></a><a id="the&#45;marquee&#45;element&#58;xattr&#45;reflect&#45;7"></a><a id="the&#45;marquee&#45;element&#58;xattr&#45;reflectdefault&#45;2"></a><a id="dom&#45;marquee&#45;scrolldelay"></a><a id="the&#45;marquee&#45;element&#58;cereactions&#45;9"></a><a id="the&#45;marquee&#45;element&#58;xattr&#45;reflect&#45;8"></a><a id="dom&#45;marquee&#45;truespeed"></a><a id="the&#45;marquee&#45;element&#58;cereactions&#45;10"></a><a id="the&#45;marquee&#45;element&#58;xattr&#45;reflect&#45;9"></a><a id="dom&#45;marquee&#45;vspace"></a><a id="the&#45;marquee&#45;element&#58;cereactions&#45;11"></a><a id="the&#45;marquee&#45;element&#58;xattr&#45;reflect&#45;10"></a><a id="dom&#45;marquee&#45;width"></a><a id="the&#45;marquee&#45;element&#58;dom&#45;marquee&#45;start"></a><a id="the&#45;marquee&#45;element&#58;dom&#45;marquee&#45;stop"></a>

``` text
[Exposed=Window]
interface HTMLMarqueeElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions, Reflect] attribute DOMString behavior;
  [CEReactions, Reflect] attribute DOMString bgColor;
  [CEReactions, Reflect] attribute DOMString direction;
  [CEReactions, Reflect] attribute DOMString height;
  [CEReactions, Reflect] attribute unsigned long hspace;
  [CEReactions] attribute long loop;
  [CEReactions, Reflect, ReflectDefault=6] attribute unsigned long scrollAmount;
  [CEReactions, Reflect, ReflectDefault=85] attribute unsigned long scrollDelay;
  [CEReactions, Reflect] attribute boolean trueSpeed;
  [CEReactions, Reflect] attribute unsigned long vspace;
  [CEReactions, Reflect] attribute DOMString width;

  undefined start();
  undefined stop();
};
```

A <a id="the&#45;marquee&#45;element&#58;the&#45;marquee&#45;element&#45;3"></a><code>[marquee](#the-marquee-element)</code> element can be <a id="concept&#45;marquee&#45;on"></a><strong>turned on</strong> or <a id="concept&#45;marquee&#45;off"></a><strong>turned off</strong>&#46; When it is created&#44; it is <a id="the&#45;marquee&#45;element&#58;concept&#45;marquee&#45;on"></a>[turned on](#concept-marquee-on)&#46;

When the <a id="dom&#45;marquee&#45;start"></a><strong><code>start&#40;&#41;</code></strong> method is called&#44; the <a id="the&#45;marquee&#45;element&#58;the&#45;marquee&#45;element&#45;4"></a><code>[marquee](#the-marquee-element)</code> element must be <a id="the&#45;marquee&#45;element&#58;concept&#45;marquee&#45;on&#45;2"></a>[turned on](#concept-marquee-on)&#46;

When the <a id="dom&#45;marquee&#45;stop"></a><strong><code>stop&#40;&#41;</code></strong> method is called&#44; the <a id="the&#45;marquee&#45;element&#58;the&#45;marquee&#45;element&#45;5"></a><code>[marquee](#the-marquee-element)</code> element must be <a id="the&#45;marquee&#45;element&#58;concept&#45;marquee&#45;off"></a>[turned off](#concept-marquee-off)&#46;

------------------------------------------------------------------------

The <a id="attr&#45;marquee&#45;behavior"></a><strong><code>behavior</code></strong> content attribute on <a id="the&#45;marquee&#45;element&#58;the&#45;marquee&#45;element&#45;6"></a><code>[marquee](#the-marquee-element)</code> elements is an <a id="the&#45;marquee&#45;element&#58;enumerated&#45;attribute"></a>[enumerated attribute](html-language-foundations--877d6146487f--b55e73c31978.md#enumerated-attribute) with the following keywords and states &#40;all non&#45;conforming&#41;&#58;

<table><thead><tr><th>Keyword      </th><th>State    </th></tr></thead><tbody><tr><td><code>scroll</code>      </td><td><dfn id="attr-marquee-behavior-scroll">scroll</dfn>     </td></tr><tr><td><code>slide</code>      </td><td><dfn id="attr-marquee-behavior-slide">slide</dfn>     </td></tr><tr><td><code>alternate</code>      </td><td><dfn id="attr-marquee-behavior-alternate">alternate</dfn>   </td></tr></tbody></table>

The attribute&#39;s <a id="the&#45;marquee&#45;element&#58;missing&#45;value&#45;default"></a><em>[missing value default](html-language-foundations--877d6146487f--b55e73c31978.md#missing-value-default)</em> and <a id="the&#45;marquee&#45;element&#58;invalid&#45;value&#45;default"></a><em>[invalid value default](html-language-foundations--877d6146487f--b55e73c31978.md#invalid-value-default)</em> are both the <a id="the&#45;marquee&#45;element&#58;attr&#45;marquee&#45;behavior&#45;scroll"></a>[scroll](#attr-marquee-behavior-scroll) state&#46;

------------------------------------------------------------------------

The <a id="attr&#45;marquee&#45;direction"></a><strong><code>direction</code></strong> content attribute on <a id="the&#45;marquee&#45;element&#58;the&#45;marquee&#45;element&#45;7"></a><code>[marquee](#the-marquee-element)</code> elements is an <a id="the&#45;marquee&#45;element&#58;enumerated&#45;attribute&#45;2"></a>[enumerated attribute](html-language-foundations--877d6146487f--b55e73c31978.md#enumerated-attribute) with the following keywords and states &#40;all non&#45;conforming&#41;&#58;

<table><thead><tr><th>Keyword      </th><th>State    </th></tr></thead><tbody><tr><td><code>left</code>      </td><td><dfn id="attr-marquee-direction-left">left</dfn>     </td></tr><tr><td><code>right</code>      </td><td><dfn id="attr-marquee-direction-right">right</dfn>     </td></tr><tr><td><code>up</code>      </td><td><dfn id="attr-marquee-direction-up">up</dfn>     </td></tr><tr><td><code>down</code>      </td><td><dfn id="attr-marquee-direction-down">down</dfn>   </td></tr></tbody></table>

The attribute&#39;s <a id="the&#45;marquee&#45;element&#58;missing&#45;value&#45;default&#45;2"></a><em>[missing value default](html-language-foundations--877d6146487f--b55e73c31978.md#missing-value-default)</em> and <a id="the&#45;marquee&#45;element&#58;invalid&#45;value&#45;default&#45;2"></a><em>[invalid value default](html-language-foundations--877d6146487f--b55e73c31978.md#invalid-value-default)</em> are both the <a id="the&#45;marquee&#45;element&#58;attr&#45;marquee&#45;direction&#45;left"></a>[left](#attr-marquee-direction-left) state&#46;

------------------------------------------------------------------------

The <a id="attr&#45;marquee&#45;truespeed"></a><strong><code>truespeed</code></strong> content attribute on <a id="the&#45;marquee&#45;element&#58;the&#45;marquee&#45;element&#45;8"></a><code>[marquee](#the-marquee-element)</code> elements is a <a id="the&#45;marquee&#45;element&#58;boolean&#45;attribute"></a>[boolean attribute](html-language-foundations--877d6146487f--b55e73c31978.md#boolean-attribute)&#46;

------------------------------------------------------------------------

A <a id="the&#45;marquee&#45;element&#58;the&#45;marquee&#45;element&#45;9"></a><code>[marquee](#the-marquee-element)</code> element has a <a id="marquee&#45;scroll&#45;interval"></a><strong>marquee scroll interval</strong>&#44; which is obtained as follows&#58;

1.  If the element has a <code>scrolldelay</code> attribute&#44; and parsing its value using the <a id="the&#45;marquee&#45;element&#58;rules&#45;for&#45;parsing&#45;non&#45;negative&#45;integers"></a>[rules for parsing non&#45;negative integers](html-language-foundations--877d6146487f--b55e73c31978.md#rules-for-parsing-non-negative-integers) does not return an error&#44; then let <var>delay</var> be the parsed value&#46; Otherwise&#44; let <var>delay</var> be 85&#46;

2.  If the element does not have a <a id="the&#45;marquee&#45;element&#58;attr&#45;marquee&#45;truespeed"></a><code>[truespeed](#attr-marquee-truespeed)</code> attribute&#44; and the <var>delay</var> value is less than 60&#44; then let <var>delay</var> be 60 instead&#46;

3.  The <a id="the&#45;marquee&#45;element&#58;marquee&#45;scroll&#45;interval"></a>[marquee scroll interval](#marquee-scroll-interval) is <var>delay</var>&#44; interpreted in milliseconds&#46;

------------------------------------------------------------------------

A <a id="the&#45;marquee&#45;element&#58;the&#45;marquee&#45;element&#45;10"></a><code>[marquee](#the-marquee-element)</code> element has a <a id="marquee&#45;scroll&#45;distance"></a><strong>marquee scroll distance</strong>&#44; which&#44; if the element has a <code>scrollamount</code> attribute&#44; and parsing its value using the <a id="the&#45;marquee&#45;element&#58;rules&#45;for&#45;parsing&#45;non&#45;negative&#45;integers&#45;2"></a>[rules for parsing non&#45;negative integers](html-language-foundations--877d6146487f--b55e73c31978.md#rules-for-parsing-non-negative-integers) does not return an error&#44; is the parsed value interpreted in <a id="the&#45;marquee&#45;element&#58;&#39;px&#39;"></a>[CSS pixels](https://drafts.csswg.org/css-values/#px)&#44; and otherwise is 6 <a id="the&#45;marquee&#45;element&#58;&#39;px&#39;&#45;2"></a>[CSS pixels](https://drafts.csswg.org/css-values/#px)&#46;

------------------------------------------------------------------------

A <a id="the&#45;marquee&#45;element&#58;the&#45;marquee&#45;element&#45;11"></a><code>[marquee](#the-marquee-element)</code> element has a <a id="marquee&#45;loop&#45;count"></a><strong>marquee loop count</strong>&#44; which&#44; if the element has a <a id="attr&#45;marquee&#45;loop"></a><strong><code>loop</code></strong> attribute&#44; and parsing its value using the <a id="the&#45;marquee&#45;element&#58;rules&#45;for&#45;parsing&#45;integers"></a>[rules for parsing integers](html-language-foundations--877d6146487f--b55e73c31978.md#rules-for-parsing-integers) does not return an error or a number less than 1&#44; is the parsed value&#44; and otherwise is −1&#46;

The <a id="dom&#45;marquee&#45;loop"></a><strong><code>loop</code></strong> IDL attribute&#44; on getting&#44; must return the element&#39;s <a id="the&#45;marquee&#45;element&#58;marquee&#45;loop&#45;count"></a>[marquee loop count](#marquee-loop-count)&#59; and on setting&#44; if the new value is different than the element&#39;s <a id="the&#45;marquee&#45;element&#58;marquee&#45;loop&#45;count&#45;2"></a>[marquee loop count](#marquee-loop-count) and either greater than zero or equal to −1&#44; must set the element&#39;s <a id="the&#45;marquee&#45;element&#58;attr&#45;marquee&#45;loop"></a><code>[loop](#attr-marquee-loop)</code> content attribute &#40;adding it if necessary&#41; to the <a id="the&#45;marquee&#45;element&#58;valid&#45;integer"></a>[valid integer](html-language-foundations--877d6146487f--b55e73c31978.md#valid-integer) that represents the new value&#46; &#40;Other values are ignored&#46;&#41;

A <a id="the&#45;marquee&#45;element&#58;the&#45;marquee&#45;element&#45;12"></a><code>[marquee](#the-marquee-element)</code> element also has a <a id="marquee&#45;current&#45;loop&#45;index"></a><strong>marquee current loop index</strong>&#44; which is zero when the element is created&#46;

The rendering layer will occasionally <a id="increment&#45;the&#45;marquee&#45;current&#45;loop&#45;index"></a><strong>increment the marquee current loop index</strong>&#44; which must cause the following steps to be run&#58;

1.  If the <a id="the&#45;marquee&#45;element&#58;marquee&#45;loop&#45;count&#45;3"></a>[marquee loop count](#marquee-loop-count) is −1&#44; then return&#46;

2.  Increment the <a id="the&#45;marquee&#45;element&#58;marquee&#45;current&#45;loop&#45;index"></a>[marquee current loop index](#marquee-current-loop-index) by one&#46;

3.  If the <a id="the&#45;marquee&#45;element&#58;marquee&#45;current&#45;loop&#45;index&#45;2"></a>[marquee current loop index](#marquee-current-loop-index) is now greater than or equal to the element&#39;s <a id="the&#45;marquee&#45;element&#58;marquee&#45;loop&#45;count&#45;4"></a>[marquee loop count](#marquee-loop-count)&#44; <a id="the&#45;marquee&#45;element&#58;concept&#45;marquee&#45;off&#45;2"></a>[turn off](#concept-marquee-off) the <a id="the&#45;marquee&#45;element&#58;the&#45;marquee&#45;element&#45;13"></a><code>[marquee](#the-marquee-element)</code> element&#46;

------------------------------------------------------------------------

#### <a id="frames"></a>16&#46;3&#46;2 Frames[](#frames)

The <a id="frameset"></a><strong><code>frameset</code></strong> element acts as <a id="frames&#58;the&#45;body&#45;element&#45;2"></a>[the body element](html-language-foundations--877d6146487f--b55e73c31978.md#the-body-element-2) in documents that use frames&#46;

The <a id="frames&#58;frameset"></a><code>[frameset](#frameset)</code> element must implement the <a id="frames&#58;htmlframesetelement"></a><code>[HTMLFrameSetElement](#htmlframesetelement)</code> interface&#46;

<a id="htmlframesetelement"></a><a id="frames&#58;htmlelement"></a><a id="frames&#58;htmlconstructor"></a><a id="frames&#58;cereactions"></a><a id="frames&#58;xattr&#45;reflect"></a><a id="dom&#45;frameset&#45;cols"></a><a id="frames&#58;cereactions&#45;2"></a><a id="frames&#58;xattr&#45;reflect&#45;2"></a><a id="dom&#45;frameset&#45;rows"></a><a id="frames&#58;htmlframesetelement&#45;2"></a><a id="frames&#58;windoweventhandlers"></a>

``` text
[Exposed=Window]
interface HTMLFrameSetElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions, Reflect] attribute DOMString cols;
  [CEReactions, Reflect] attribute DOMString rows;
};
HTMLFrameSetElement includes WindowEventHandlers;
```

The <a id="frames&#58;frameset&#45;2"></a><code>[frameset](#frameset)</code> element exposes as <a id="frames&#58;event&#45;handler&#45;content&#45;attributes"></a>[event handler content attributes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-handler-content-attributes) a number of the <a id="frames&#58;event&#45;handlers"></a>[event handlers](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-handlers) of the <a id="frames&#58;window"></a><code>[Window](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window)</code> object&#46; It also mirrors their <a id="frames&#58;event&#45;handler&#45;idl&#45;attributes"></a>[event handler IDL attributes](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-handler-idl-attributes)&#46;

The <a id="frames&#58;event&#45;handlers&#45;2"></a>[event handlers](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-handlers) of the <a id="frames&#58;window&#45;2"></a><code>[Window](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window)</code> object named by the <a id="frames&#58;window&#45;reflecting&#45;body&#45;element&#45;event&#45;handler&#45;set"></a>[<code>Window</code>&#45;reflecting body element event handler set](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window-reflecting-body-element-event-handler-set)&#44; exposed on the <a id="frames&#58;frameset&#45;3"></a><code>[frameset](#frameset)</code> element&#44; replace the generic <a id="frames&#58;event&#45;handlers&#45;3"></a>[event handlers](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-handlers) with the same names normally supported by <a id="frames&#58;html&#45;elements"></a>[HTML elements](html-language-foundations--877d6146487f--b55e73c31978.md#html-elements)&#46;

------------------------------------------------------------------------

The <a id="frame"></a><strong><code>frame</code></strong> element has a <a id="frames&#58;content&#45;navigable"></a>[content navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable) similar to the <a id="frames&#58;the&#45;iframe&#45;element"></a><code>[iframe](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element)</code> element&#44; but rendered within a <a id="frames&#58;frameset&#45;4"></a><code>[frameset](#frameset)</code> element&#46;

The <a id="frames&#58;frame"></a><code>[frame](#frame)</code> <a id="frames&#58;html&#45;element&#45;post&#45;connection&#45;steps"></a>[HTML element post&#45;connection steps](html-language-foundations--877d6146487f--b55e73c31978.md#html-element-post-connection-steps)&#44; given <var>insertedNode</var>&#44; are&#58;

1.  If <var>insertedNode</var>&#39;s <a id="frames&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="frames&#58;concept&#45;document&#45;bc"></a>[browsing context](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc) is null&#44; then return&#46;

2.  <a id="frames&#58;create&#45;a&#45;new&#45;child&#45;navigable"></a>[Create a new child navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#create-a-new-child-navigable) for <var>insertedNode</var>&#46;

3.  <a id="frames&#58;process&#45;the&#45;frame&#45;attributes"></a>[Process the <code>frame</code> attributes](#process-the-frame-attributes) for <var>insertedNode</var>&#44; with <a id="frames&#58;process&#45;frame&#45;initial&#45;insertion"></a><em>[initialInsertion](#process-frame-initial-insertion)</em> set to true&#46;

The <a id="frames&#58;frame&#45;2"></a><code>[frame](#frame)</code> <a id="frames&#58;html&#45;element&#45;removing&#45;steps"></a>[HTML element removing steps](html-language-foundations--877d6146487f--b55e73c31978.md#html-element-removing-steps)&#44; given <var>removedNode</var>&#44; are to <a id="frames&#58;destroy&#45;a&#45;child&#45;navigable"></a>[destroy a child navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#destroy-a-child-navigable) given <var>removedNode</var>&#46;

Whenever a <a id="frames&#58;frame&#45;3"></a><code>[frame](#frame)</code> element with a non&#45;null <a id="frames&#58;content&#45;navigable&#45;2"></a>[content navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable) has its <code>src</code> attribute set&#44; changed&#44; or removed&#44; the user agent must <a id="frames&#58;process&#45;the&#45;frame&#45;attributes&#45;2"></a>[process the <code>frame</code> attributes](#process-the-frame-attributes)&#46;

To <a id="process&#45;the&#45;frame&#45;attributes"></a><strong>process the <code>frame</code> attributes</strong> for an element <var>element</var>&#44; with an optional boolean <a id="process&#45;frame&#45;initial&#45;insertion"></a><strong><var>initialInsertion</var></strong> &#40;default false&#41;&#58;

1.  Let <var>url</var> be the result of running the <a id="frames&#58;shared&#45;attribute&#45;processing&#45;steps&#45;for&#45;iframe&#45;and&#45;frame&#45;elements"></a>[shared attribute processing steps for <code>iframe</code> and <code>frame</code> elements](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#shared-attribute-processing-steps-for-iframe-and-frame-elements) given <var>element</var> and <var>initialInsertion</var>&#46;

2.  If <var>url</var> is null&#44; then return&#46;

3.  If <var>url</var> <a id="frames&#58;matches&#45;about&#58;blank"></a>[matches <code>about&#58;blank</code>](html-language-foundations--877d6146487f--b55e73c31978.md#matches-about:blank) and <var>initialInsertion</var> is true&#58;

    1.  <a id="frames&#58;concept&#45;event&#45;fire"></a>[Fire an event](https://dom.spec.whatwg.org/#concept-event-fire) named <a id="frames&#58;event&#45;load"></a><code>[load](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-load)</code> at <var>element</var>&#46;

    2.  Return&#46;

4.  <a id="frames&#58;navigate&#45;an&#45;iframe&#45;or&#45;frame"></a>[Navigate an <code>iframe</code> or <code>frame</code>](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#navigate-an-iframe-or-frame) given <var>element</var>&#44; <var>url</var>&#44; the empty string&#44; null&#44; and <var>initialInsertion</var>&#46;

The <a id="frames&#58;frame&#45;4"></a><code>[frame](#frame)</code> element <a id="frames&#58;potentially&#45;delays&#45;the&#45;load&#45;event"></a>[potentially delays the load event](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#potentially-delays-the-load-event)&#46;

The <a id="frames&#58;frame&#45;5"></a><code>[frame](#frame)</code> element must implement the <a id="frames&#58;htmlframeelement"></a><code>[HTMLFrameElement](#htmlframeelement)</code> interface&#46;

<a id="htmlframeelement"></a><a id="frames&#58;htmlelement&#45;2"></a><a id="frames&#58;htmlconstructor&#45;2"></a><a id="frames&#58;cereactions&#45;3"></a><a id="frames&#58;xattr&#45;reflect&#45;3"></a><a id="dom&#45;frame&#45;name"></a><a id="frames&#58;cereactions&#45;4"></a><a id="frames&#58;xattr&#45;reflect&#45;4"></a><a id="dom&#45;frame&#45;scrolling"></a><a id="frames&#58;cereactions&#45;5"></a><a id="frames&#58;xattr&#45;reflecturl"></a><a id="dom&#45;frame&#45;src"></a><a id="frames&#58;cereactions&#45;6"></a><a id="frames&#58;xattr&#45;reflect&#45;5"></a><a id="dom&#45;frame&#45;frameborder"></a><a id="frames&#58;cereactions&#45;7"></a><a id="frames&#58;xattr&#45;reflecturl&#45;2"></a><a id="dom&#45;frame&#45;longdesc"></a><a id="frames&#58;cereactions&#45;8"></a><a id="frames&#58;xattr&#45;reflect&#45;6"></a><a id="dom&#45;frame&#45;noresize"></a><a id="frames&#58;document"></a><a id="frames&#58;dom&#45;frame&#45;contentdocument"></a><a id="frames&#58;windowproxy"></a><a id="frames&#58;dom&#45;frame&#45;contentwindow"></a><a id="frames&#58;cereactions&#45;9"></a><a id="frames&#58;xattr&#45;reflect&#45;7"></a><a id="frames&#58;legacynulltoemptystring"></a><a id="dom&#45;frame&#45;marginheight"></a><a id="frames&#58;cereactions&#45;10"></a><a id="frames&#58;xattr&#45;reflect&#45;8"></a><a id="frames&#58;legacynulltoemptystring&#45;2"></a><a id="dom&#45;frame&#45;marginwidth"></a>

``` text
[Exposed=Window]
interface HTMLFrameElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions, Reflect] attribute DOMString name;
  [CEReactions, Reflect] attribute DOMString scrolling;
  [CEReactions, ReflectURL] attribute USVString src;
  [CEReactions, Reflect] attribute DOMString frameBorder;
  [CEReactions, ReflectURL] attribute USVString longDesc;
  [CEReactions, Reflect] attribute boolean noResize;
  readonly attribute Document? contentDocument;
  readonly attribute WindowProxy? contentWindow;

  [CEReactions, Reflect] attribute [LegacyNullToEmptyString] DOMString marginHeight;
  [CEReactions, Reflect] attribute [LegacyNullToEmptyString] DOMString marginWidth;
};
```

The <a id="dom&#45;frame&#45;contentdocument"></a><strong><code>contentDocument</code></strong> getter steps are to return <a id="frames&#58;this"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="frames&#58;concept&#45;bcc&#45;content&#45;document"></a>[content document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-bcc-content-document)&#46;

The <a id="dom&#45;frame&#45;contentwindow"></a><strong><code>contentWindow</code></strong> getter steps are to return <a id="frames&#58;this&#45;2"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="frames&#58;content&#45;window"></a>[content window](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-window)&#46;

#### <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis"></a>16&#46;3&#46;3 Other elements&#44; attributes and APIs[](#other-elements,-attributes-and-apis)

User agents must treat <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;acronym"></a><code>[acronym](#acronym)</code> elements in a manner equivalent to <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;the&#45;abbr&#45;element"></a><code>[abbr](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-abbr-element)</code> elements in terms of semantics and for purposes of rendering&#46;

------------------------------------------------------------------------

<a id="HTMLAnchorElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect"></a><a id="dom&#45;a&#45;coords"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;2"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;2"></a><a id="dom&#45;a&#45;charset"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;3"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;3"></a><a id="dom&#45;a&#45;name"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;4"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;4"></a><a id="dom&#45;a&#45;rev"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;5"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;5"></a><a id="dom&#45;a&#45;shape"></a>

``` text
partial interface HTMLAnchorElement {
  [CEReactions, Reflect] attribute DOMString coords;
  [CEReactions, Reflect] attribute DOMString charset;
  [CEReactions, Reflect] attribute DOMString name;
  [CEReactions, Reflect] attribute DOMString rev;
  [CEReactions, Reflect] attribute DOMString shape;
};
```

------------------------------------------------------------------------

<a id="HTMLAreaElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;6"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;6"></a><a id="dom&#45;area&#45;nohref"></a>

``` text
partial interface HTMLAreaElement {
  [CEReactions, Reflect] attribute boolean noHref;
};
```

------------------------------------------------------------------------

<a id="HTMLBodyElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;7"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;7"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring"></a><a id="dom&#45;body&#45;text"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;8"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;8"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring&#45;2"></a><a id="dom&#45;body&#45;link"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;9"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;9"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring&#45;3"></a><a id="dom&#45;body&#45;vlink"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;10"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;10"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring&#45;4"></a><a id="dom&#45;body&#45;alink"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;11"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;11"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring&#45;5"></a><a id="dom&#45;body&#45;bgcolor"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;12"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;12"></a><a id="dom&#45;body&#45;background"></a>

``` text
partial interface HTMLBodyElement {
  [CEReactions, Reflect] attribute [LegacyNullToEmptyString] DOMString text;
  [CEReactions, Reflect] attribute [LegacyNullToEmptyString] DOMString link;
  [CEReactions, Reflect] attribute [LegacyNullToEmptyString] DOMString vLink;
  [CEReactions, Reflect] attribute [LegacyNullToEmptyString] DOMString aLink;
  [CEReactions, Reflect] attribute [LegacyNullToEmptyString] DOMString bgColor;
  [CEReactions, Reflect] attribute DOMString background;
};
```

<strong>Note&#58;</strong>

The <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;dom&#45;body&#45;background"></a><code>[background](#dom-body-background)</code> IDL attribute does not use <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflecturl"></a>[ReflectURL](html-language-foundations--877d6146487f--b55e73c31978.md#xattr-reflecturl) or <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;idl&#45;usvstring"></a><code>[USVString](https://webidl.spec.whatwg.org/#idl-USVString)</code> for historical reasons&#46;

------------------------------------------------------------------------

<a id="HTMLBRElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;13"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;13"></a><a id="dom&#45;br&#45;clear"></a>

``` text
partial interface HTMLBRElement {
  [CEReactions, Reflect] attribute DOMString clear;
};
```

------------------------------------------------------------------------

<a id="HTMLTableCaptionElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;14"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;14"></a><a id="dom&#45;caption&#45;align"></a>

``` text
partial interface HTMLTableCaptionElement {
  [CEReactions, Reflect] attribute DOMString align;
};
```

------------------------------------------------------------------------

<a id="HTMLTableColElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;15"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;15"></a><a id="dom&#45;col&#45;align"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;16"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;16"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;col&#45;char"></a><a id="dom&#45;col&#45;ch"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;17"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;17"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;col&#45;charoff"></a><a id="dom&#45;col&#45;choff"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;18"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;18"></a><a id="dom&#45;col&#45;valign"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;19"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;19"></a><a id="dom&#45;col&#45;width"></a>

``` text
partial interface HTMLTableColElement {
  [CEReactions, Reflect] attribute DOMString align;
  [CEReactions, Reflect="char"] attribute DOMString ch;
  [CEReactions, Reflect="charoff"] attribute DOMString chOff;
  [CEReactions, Reflect] attribute DOMString vAlign;
  [CEReactions, Reflect] attribute DOMString width;
};
```

------------------------------------------------------------------------

User agents must treat <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;dir"></a><code>[dir](#dir)</code> elements in a manner equivalent to <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;the&#45;ul&#45;element"></a><code>[ul](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-ul-element)</code> elements in terms of semantics and for purposes of rendering&#46;

The <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;dir&#45;2"></a><code>[dir](#dir)</code> element must implement the <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;htmldirectoryelement"></a><code>[HTMLDirectoryElement](#htmldirectoryelement)</code> interface&#46;

<a id="htmldirectoryelement"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;htmlelement"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;htmlconstructor"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;20"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;20"></a><a id="dom&#45;dir&#45;compact"></a>

``` text
[Exposed=Window]
interface HTMLDirectoryElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions, Reflect] attribute boolean compact;
};
```

------------------------------------------------------------------------

<a id="HTMLDivElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;21"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;21"></a><a id="dom&#45;div&#45;align"></a>

``` text
partial interface HTMLDivElement {
  [CEReactions, Reflect] attribute DOMString align;
};
```

------------------------------------------------------------------------

<a id="HTMLDListElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;22"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;22"></a><a id="dom&#45;dl&#45;compact"></a>

``` text
partial interface HTMLDListElement {
  [CEReactions, Reflect] attribute boolean compact;
};
```

------------------------------------------------------------------------

<a id="HTMLEmbedElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;23"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;23"></a><a id="dom&#45;embed&#45;align"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;24"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;24"></a><a id="dom&#45;embed&#45;name"></a>

``` text
partial interface HTMLEmbedElement {
  [CEReactions, Reflect] attribute DOMString align;
  [CEReactions, Reflect] attribute DOMString name;
};
```

------------------------------------------------------------------------

The <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;font"></a><code>[font](#font)</code> element must implement the <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;htmlfontelement"></a><code>[HTMLFontElement](#htmlfontelement)</code> interface&#46;

<a id="htmlfontelement"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;htmlelement&#45;2"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;htmlconstructor&#45;2"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;25"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;25"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring&#45;6"></a><a id="dom&#45;font&#45;color"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;26"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;26"></a><a id="dom&#45;font&#45;face"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;27"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;27"></a><a id="dom&#45;font&#45;size"></a>

``` text
[Exposed=Window]
interface HTMLFontElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions, Reflect] attribute [LegacyNullToEmptyString] DOMString color;
  [CEReactions, Reflect] attribute DOMString face;
  [CEReactions, Reflect] attribute DOMString size; 
};
```

------------------------------------------------------------------------

<a id="HTMLHeadingElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;28"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;28"></a><a id="dom&#45;hx&#45;align"></a>

``` text
partial interface HTMLHeadingElement {
  [CEReactions, Reflect] attribute DOMString align;
};
```

------------------------------------------------------------------------

<strong>Note&#58;</strong>

The <a id="dom&#45;head&#45;profile"></a><strong><code>profile</code></strong> IDL attribute on <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;the&#45;head&#45;element"></a><code>[head](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-head-element)</code> elements &#40;with the <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;htmlheadelement"></a><code>[HTMLHeadElement](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#htmlheadelement)</code> interface&#41; is intentionally omitted&#46; Unless so required by <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;other&#45;applicable&#45;specifications"></a>[another applicable specification](html-language-foundations--877d6146487f--b55e73c31978.md#other-applicable-specifications)&#44; implementations would therefore not support this attribute&#46; &#40;It is mentioned here as it was defined in a previous version of DOM&#46;&#41;

------------------------------------------------------------------------

<a id="HTMLHRElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;29"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;29"></a><a id="dom&#45;hr&#45;align"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;30"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;30"></a><a id="dom&#45;hr&#45;color"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;31"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;31"></a><a id="dom&#45;hr&#45;noshade"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;32"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;32"></a><a id="dom&#45;hr&#45;size"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;33"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;33"></a><a id="dom&#45;hr&#45;width"></a>

``` text
partial interface HTMLHRElement {
  [CEReactions, Reflect] attribute DOMString align;
  [CEReactions, Reflect] attribute DOMString color;
  [CEReactions, Reflect] attribute boolean noShade;
  [CEReactions, Reflect] attribute DOMString size;
  [CEReactions, Reflect] attribute DOMString width;
};
```

------------------------------------------------------------------------

<a id="HTMLHtmlElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;34"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;34"></a><a id="dom&#45;html&#45;version"></a>

``` text
partial interface HTMLHtmlElement {
  [CEReactions, Reflect] attribute DOMString version;
};
```

------------------------------------------------------------------------

<a id="HTMLIFrameElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;35"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;35"></a><a id="dom&#45;iframe&#45;align"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;36"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;36"></a><a id="dom&#45;iframe&#45;scrolling"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;37"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;37"></a><a id="dom&#45;iframe&#45;frameborder"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;38"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflecturl&#45;2"></a><a id="dom&#45;iframe&#45;longdesc"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;39"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;38"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring&#45;7"></a><a id="dom&#45;iframe&#45;marginheight"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;40"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;39"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring&#45;8"></a><a id="dom&#45;iframe&#45;marginwidth"></a>

``` text
partial interface HTMLIFrameElement {
  [CEReactions, Reflect] attribute DOMString align;
  [CEReactions, Reflect] attribute DOMString scrolling;
  [CEReactions, Reflect] attribute DOMString frameBorder;
  [CEReactions, ReflectURL] attribute USVString longDesc;

  [CEReactions, Reflect] attribute [LegacyNullToEmptyString] DOMString marginHeight;
  [CEReactions, Reflect] attribute [LegacyNullToEmptyString] DOMString marginWidth;
};
```

------------------------------------------------------------------------

<a id="HTMLImageElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;41"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;40"></a><a id="dom&#45;img&#45;name"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;42"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflecturl&#45;3"></a><a id="dom&#45;img&#45;lowsrc"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;43"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;41"></a><a id="dom&#45;img&#45;align"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;44"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;42"></a><a id="dom&#45;img&#45;hspace"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;45"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;43"></a><a id="dom&#45;img&#45;vspace"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;46"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflecturl&#45;4"></a><a id="dom&#45;img&#45;longdesc"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;47"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;44"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring&#45;9"></a><a id="dom&#45;img&#45;border"></a>

``` text
partial interface HTMLImageElement {
  [CEReactions, Reflect] attribute DOMString name;
  [CEReactions, ReflectURL] attribute USVString lowsrc;
  [CEReactions, Reflect] attribute DOMString align;
  [CEReactions, Reflect] attribute unsigned long hspace;
  [CEReactions, Reflect] attribute unsigned long vspace;
  [CEReactions, ReflectURL] attribute USVString longDesc;

  [CEReactions, Reflect] attribute [LegacyNullToEmptyString] DOMString border;
};
```

------------------------------------------------------------------------

<a id="HTMLInputElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;48"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;45"></a><a id="dom&#45;input&#45;align"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;49"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;46"></a><a id="dom&#45;input&#45;usemap"></a>

``` text
partial interface HTMLInputElement {
  [CEReactions, Reflect] attribute DOMString align;
  [CEReactions, Reflect] attribute DOMString useMap;
};
```

------------------------------------------------------------------------

<a id="HTMLLegendElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;50"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;47"></a><a id="dom&#45;legend&#45;align"></a>

``` text
partial interface HTMLLegendElement {
  [CEReactions, Reflect] attribute DOMString align;
};
```

------------------------------------------------------------------------

<a id="HTMLLIElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;51"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;48"></a><a id="dom&#45;li&#45;type"></a>

``` text
partial interface HTMLLIElement {
  [CEReactions, Reflect] attribute DOMString type;
};
```

------------------------------------------------------------------------

<a id="HTMLLinkElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;52"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;49"></a><a id="dom&#45;link&#45;charset"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;53"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;50"></a><a id="dom&#45;link&#45;rev"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;54"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;51"></a><a id="dom&#45;link&#45;target"></a>

``` text
partial interface HTMLLinkElement {
  [CEReactions, Reflect] attribute DOMString charset;
  [CEReactions, Reflect] attribute DOMString rev;
  [CEReactions, Reflect] attribute DOMString target;
};
```

------------------------------------------------------------------------

User agents must treat <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;listing"></a><code>[listing](#listing)</code> elements in a manner equivalent to <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;the&#45;pre&#45;element"></a><code>[pre](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-pre-element)</code> elements in terms of semantics and for purposes of rendering&#46;

------------------------------------------------------------------------

<a id="HTMLMenuElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;55"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;52"></a><a id="dom&#45;menu&#45;compact"></a>

``` text
partial interface HTMLMenuElement {
  [CEReactions, Reflect] attribute boolean compact;
};
```

------------------------------------------------------------------------

<a id="HTMLMetaElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;56"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;53"></a><a id="dom&#45;meta&#45;scheme"></a>

``` text
partial interface HTMLMetaElement {
  [CEReactions, Reflect] attribute DOMString scheme;
};
```

User agents may treat the <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;meta&#45;scheme"></a><code>[scheme](#attr-meta-scheme)</code> content attribute on the <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;the&#45;meta&#45;element"></a><code>[meta](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-meta-element)</code> element as an extension of the element&#39;s <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;meta&#45;name"></a><code>[name](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-meta-name)</code> content attribute when processing a <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;the&#45;meta&#45;element&#45;2"></a><code>[meta](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-meta-element)</code> element with a <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;meta&#45;name&#45;2"></a><code>[name](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-meta-name)</code> attribute whose value is one that the user agent recognizes as supporting the <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;meta&#45;scheme&#45;2"></a><code>[scheme](#attr-meta-scheme)</code> attribute&#46;

User agents are encouraged to ignore the <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;meta&#45;scheme&#45;3"></a><code>[scheme](#attr-meta-scheme)</code> attribute and instead process the value given to the metadata name as if it had been specified for each expected value of the <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;meta&#45;scheme&#45;4"></a><code>[scheme](#attr-meta-scheme)</code> attribute&#46;

<strong>Example&#58;</strong>

For example&#44; if the user agent acts on <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;the&#45;meta&#45;element&#45;3"></a><code>[meta](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-meta-element)</code> elements with <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;meta&#45;name&#45;3"></a><code>[name](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-meta-name)</code> attributes having the value &#34;eGMS&#46;subject&#46;keyword&#34;&#44; and knows that the <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;meta&#45;scheme&#45;5"></a><code>[scheme](#attr-meta-scheme)</code> attribute is used with this metadata name&#44; then it could take the <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;meta&#45;scheme&#45;6"></a><code>[scheme](#attr-meta-scheme)</code> attribute into account&#44; acting as if it was an extension of the <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;meta&#45;name&#45;4"></a><code>[name](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#attr-meta-name)</code> attribute&#46; Thus the following two <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;the&#45;meta&#45;element&#45;4"></a><code>[meta](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#the-meta-element)</code> elements could be treated as two elements giving values for two different metadata names&#44; one consisting of a combination of &#34;eGMS&#46;subject&#46;keyword&#34; and &#34;LGCL&#34;&#44; and the other consisting of a combination of &#34;eGMS&#46;subject&#46;keyword&#34; and &#34;ORLY&#34;&#58;

``` text
<!-- this markup is invalid -->
<meta name="eGMS.subject.keyword" scheme="LGCL" content="Abandoned vehicles">
<meta name="eGMS.subject.keyword" scheme="ORLY" content="Mah car: kthxbye">
```

The suggested processing of this markup&#44; however&#44; would be equivalent to the following&#58;

``` text
<meta name="eGMS.subject.keyword" content="Abandoned vehicles">
<meta name="eGMS.subject.keyword" content="Mah car: kthxbye">
```

------------------------------------------------------------------------

<strong>✔</strong>MDN

[HTMLObjectElement&#47;useMap](https://developer.mozilla.org/en-US/docs/Web/API/HTMLObjectElement/useMap)

Support in all current engines&#46;

Firefox1&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer6&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

<a id="HTMLObjectElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;57"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;54"></a><a id="dom&#45;object&#45;align"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;58"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;55"></a><a id="dom&#45;object&#45;archive"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;59"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;56"></a><a id="dom&#45;object&#45;code"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;60"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;57"></a><a id="dom&#45;object&#45;declare"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;61"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;58"></a><a id="dom&#45;object&#45;hspace"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;62"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;59"></a><a id="dom&#45;object&#45;standby"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;63"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;60"></a><a id="dom&#45;object&#45;vspace"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;64"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflecturl&#45;5"></a><a id="dom&#45;object&#45;codebase"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;65"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;61"></a><a id="dom&#45;object&#45;codetype"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;66"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;62"></a><a id="dom&#45;object&#45;usemap"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;67"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;63"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring&#45;10"></a><a id="dom&#45;object&#45;border"></a>

``` text
partial interface HTMLObjectElement {
  [CEReactions, Reflect] attribute DOMString align;
  [CEReactions, Reflect] attribute DOMString archive;
  [CEReactions, Reflect] attribute DOMString code;
  [CEReactions, Reflect] attribute boolean declare;
  [CEReactions, Reflect] attribute unsigned long hspace;
  [CEReactions, Reflect] attribute DOMString standby;
  [CEReactions, Reflect] attribute unsigned long vspace;
  [CEReactions, ReflectURL] attribute DOMString codeBase;
  [CEReactions, Reflect] attribute DOMString codeType;
  [CEReactions, Reflect] attribute DOMString useMap;

  [CEReactions, Reflect] attribute [LegacyNullToEmptyString] DOMString border;
};
```

------------------------------------------------------------------------

<a id="HTMLOListElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;68"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;64"></a><a id="dom&#45;ol&#45;compact"></a>

``` text
partial interface HTMLOListElement {
  [CEReactions, Reflect] attribute boolean compact;
};
```

------------------------------------------------------------------------

<a id="HTMLParagraphElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;69"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;65"></a><a id="dom&#45;p&#45;align"></a>

``` text
partial interface HTMLParagraphElement {
  [CEReactions, Reflect] attribute DOMString align;
};
```

------------------------------------------------------------------------

The <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;param"></a><code>[param](#param)</code> element must implement the <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;htmlparamelement"></a><code>[HTMLParamElement](#htmlparamelement)</code> interface&#46;

<a id="htmlparamelement"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;htmlelement&#45;3"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;htmlconstructor&#45;3"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;70"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;66"></a><a id="dom&#45;param&#45;name"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;71"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;67"></a><a id="dom&#45;param&#45;value"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;72"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;68"></a><a id="dom&#45;param&#45;type"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;73"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;69"></a><a id="dom&#45;param&#45;valuetype"></a>

``` text
[Exposed=Window]
interface HTMLParamElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions, Reflect] attribute DOMString name;
  [CEReactions, Reflect] attribute DOMString value;
  [CEReactions, Reflect] attribute DOMString type;
  [CEReactions, Reflect] attribute DOMString valueType;
};
```

------------------------------------------------------------------------

User agents must treat <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;plaintext"></a><code>[plaintext](#plaintext)</code> elements in a manner equivalent to <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;the&#45;pre&#45;element&#45;2"></a><code>[pre](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-pre-element)</code> elements in terms of semantics and for purposes of rendering&#46; &#40;The parser has special behavior for this element&#44; though&#46;&#41;

------------------------------------------------------------------------

<a id="HTMLPreElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;74"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;70"></a><a id="dom&#45;pre&#45;width"></a>

``` text
partial interface HTMLPreElement {
  [CEReactions, Reflect] attribute long width;
};
```

------------------------------------------------------------------------

<a id="HTMLStyleElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;75"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;71"></a><a id="dom&#45;style&#45;type"></a>

``` text
partial interface HTMLStyleElement {
  [CEReactions, Reflect] attribute DOMString type;
};
```

------------------------------------------------------------------------

<a id="HTMLScriptElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;76"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;72"></a><a id="dom&#45;script&#45;charset"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;77"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;73"></a><a id="dom&#45;script&#45;event"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;78"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;74"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;script&#45;for"></a><a id="dom&#45;script&#45;htmlfor"></a>

``` text
partial interface HTMLScriptElement {
  [CEReactions, Reflect] attribute DOMString charset;
  [CEReactions, Reflect] attribute DOMString event;
  [CEReactions, Reflect="for"] attribute DOMString htmlFor;
};
```

------------------------------------------------------------------------

<a id="HTMLTableElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;79"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;75"></a><a id="dom&#45;table&#45;align"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;80"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;76"></a><a id="dom&#45;table&#45;border"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;81"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;77"></a><a id="dom&#45;table&#45;frame"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;82"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;78"></a><a id="dom&#45;table&#45;rules"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;83"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;79"></a><a id="dom&#45;table&#45;summary"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;84"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;80"></a><a id="dom&#45;table&#45;width"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;85"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;81"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring&#45;11"></a><a id="dom&#45;table&#45;bgcolor"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;86"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;82"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring&#45;12"></a><a id="dom&#45;table&#45;cellpadding"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;87"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;83"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring&#45;13"></a><a id="dom&#45;table&#45;cellspacing"></a>

``` text
partial interface HTMLTableElement {
  [CEReactions, Reflect] attribute DOMString align;
  [CEReactions, Reflect] attribute DOMString border;
  [CEReactions, Reflect] attribute DOMString frame;
  [CEReactions, Reflect] attribute DOMString rules;
  [CEReactions, Reflect] attribute DOMString summary;
  [CEReactions, Reflect] attribute DOMString width;

  [CEReactions, Reflect] attribute [LegacyNullToEmptyString] DOMString bgColor;
  [CEReactions, Reflect] attribute [LegacyNullToEmptyString] DOMString cellPadding;
  [CEReactions, Reflect] attribute [LegacyNullToEmptyString] DOMString cellSpacing;
};
```

------------------------------------------------------------------------

<a id="HTMLTableSectionElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;88"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;84"></a><a id="dom&#45;tbody&#45;align"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;89"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;85"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;tbody&#45;char"></a><a id="dom&#45;tbody&#45;ch"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;90"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;86"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;tbody&#45;charoff"></a><a id="dom&#45;tbody&#45;choff"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;91"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;87"></a><a id="dom&#45;tbody&#45;valign"></a>

``` text
partial interface HTMLTableSectionElement {
  [CEReactions, Reflect] attribute DOMString align;
  [CEReactions, Reflect="char"] attribute DOMString ch;
  [CEReactions, Reflect="charoff"] attribute DOMString chOff;
  [CEReactions, Reflect] attribute DOMString vAlign;
};
```

------------------------------------------------------------------------

<a id="HTMLTableCellElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;92"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;88"></a><a id="dom&#45;tdth&#45;align"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;93"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;89"></a><a id="dom&#45;tdth&#45;axis"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;94"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;90"></a><a id="dom&#45;tdth&#45;height"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;95"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;91"></a><a id="dom&#45;tdth&#45;width"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;96"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;92"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;tdth&#45;char"></a><a id="dom&#45;tdth&#45;ch"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;97"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;93"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;tdth&#45;charoff"></a><a id="dom&#45;tdth&#45;choff"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;98"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;94"></a><a id="dom&#45;tdth&#45;nowrap"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;99"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;95"></a><a id="dom&#45;tdth&#45;valign"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;100"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;96"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring&#45;14"></a><a id="dom&#45;tdth&#45;bgcolor"></a>

``` text
partial interface HTMLTableCellElement {
  [CEReactions, Reflect] attribute DOMString align;
  [CEReactions, Reflect] attribute DOMString axis;
  [CEReactions, Reflect] attribute DOMString height;
  [CEReactions, Reflect] attribute DOMString width;

  [CEReactions, Reflect="char"] attribute DOMString ch;
  [CEReactions, Reflect="charoff"] attribute DOMString chOff;
  [CEReactions, Reflect] attribute boolean noWrap;
  [CEReactions, Reflect] attribute DOMString vAlign;

  [CEReactions, Reflect] attribute [LegacyNullToEmptyString] DOMString bgColor;
};
```

------------------------------------------------------------------------

<a id="HTMLTableRowElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;101"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;97"></a><a id="dom&#45;tr&#45;align"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;102"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;98"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;tr&#45;char"></a><a id="dom&#45;tr&#45;ch"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;103"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;99"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;tr&#45;charoff"></a><a id="dom&#45;tr&#45;choff"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;104"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;100"></a><a id="dom&#45;tr&#45;valign"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;105"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;101"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring&#45;15"></a><a id="dom&#45;tr&#45;bgcolor"></a>

``` text
partial interface HTMLTableRowElement {
  [CEReactions, Reflect] attribute DOMString align;
  [CEReactions, Reflect="char"] attribute DOMString ch;
  [CEReactions, Reflect="charoff"] attribute DOMString chOff;
  [CEReactions, Reflect] attribute DOMString vAlign;

  [CEReactions, Reflect] attribute [LegacyNullToEmptyString] DOMString bgColor;
};
```

------------------------------------------------------------------------

<a id="HTMLUListElement&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;106"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;102"></a><a id="dom&#45;ul&#45;compact"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;107"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xattr&#45;reflect&#45;103"></a><a id="dom&#45;ul&#45;type"></a>

``` text
partial interface HTMLUListElement {
  [CEReactions, Reflect] attribute boolean compact;
  [CEReactions, Reflect] attribute DOMString type;
};
```

------------------------------------------------------------------------

User agents must treat <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;xmp"></a><code>[xmp](#xmp)</code> elements in a manner equivalent to <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;the&#45;pre&#45;element&#45;3"></a><code>[pre](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-pre-element)</code> elements in terms of semantics and for purposes of rendering&#46; &#40;The parser has special behavior for this element though&#46;&#41;

------------------------------------------------------------------------

<a id="Document&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;108"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring&#45;16"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;dom&#45;document&#45;fgcolor"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;109"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring&#45;17"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;dom&#45;document&#45;linkcolor"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;110"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring&#45;18"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;dom&#45;document&#45;vlinkcolor"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;111"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring&#45;19"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;dom&#45;document&#45;alinkcolor"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;cereactions&#45;112"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;legacynulltoemptystring&#45;20"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;dom&#45;document&#45;bgcolor"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;htmlcollection"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;dom&#45;document&#45;anchors"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;htmlcollection&#45;2"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;dom&#45;document&#45;applets"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;dom&#45;document&#45;clear"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;dom&#45;document&#45;captureevents"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;dom&#45;document&#45;releaseevents"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;htmlallcollection"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;dom&#45;document&#45;all"></a>

``` text
partial interface Document {
  [CEReactions] attribute [LegacyNullToEmptyString] DOMString fgColor;
  [CEReactions] attribute [LegacyNullToEmptyString] DOMString linkColor;
  [CEReactions] attribute [LegacyNullToEmptyString] DOMString vlinkColor;
  [CEReactions] attribute [LegacyNullToEmptyString] DOMString alinkColor;
  [CEReactions] attribute [LegacyNullToEmptyString] DOMString bgColor;

  [SameObject] readonly attribute HTMLCollection anchors;
  [SameObject] readonly attribute HTMLCollection applets;

  undefined clear();
  undefined captureEvents();
  undefined releaseEvents();

  [SameObject] readonly attribute HTMLAllCollection all;
};
```

The attributes of the <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> object listed in the first column of the following table must <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;reflect"></a>[reflect](html-language-foundations--877d6146487f--b55e73c31978.md#reflect) the content attribute on <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;the&#45;body&#45;element&#45;2"></a>[the body element](html-language-foundations--877d6146487f--b55e73c31978.md#the-body-element-2) with the name given in the corresponding cell in the second column on the same row&#44; if <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;the&#45;body&#45;element&#45;2&#45;2"></a>[the body element](html-language-foundations--877d6146487f--b55e73c31978.md#the-body-element-2) is a <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;the&#45;body&#45;element"></a><code>[body](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-body-element)</code> element &#40;as opposed to a <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;frameset"></a><code>[frameset](#frameset)</code> element&#41;&#46; When there is no <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;the&#45;body&#45;element&#45;2&#45;3"></a>[body element](html-language-foundations--877d6146487f--b55e73c31978.md#the-body-element-2) or if it is a <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;frameset&#45;2"></a><code>[frameset](#frameset)</code> element&#44; the attributes must instead return the empty string on getting and do nothing on setting&#46;

<table><thead><tr><th> IDL attribute      </th><th> Content attribute    </th></tr></thead><tbody><tr><td><dfn data-dfn-for="Document" id="dom-document-fgcolor" data-dfn-type="attribute"><code>fgColor</code></dfn>      </td><td><code id="other-elements,-attributes-and-apis:attr-body-text"><a href="#attr-body-text">text</a></code>     </td></tr><tr><td><dfn data-dfn-for="Document" id="dom-document-linkcolor" data-dfn-type="attribute"><code>linkColor</code></dfn>      </td><td><code id="other-elements,-attributes-and-apis:attr-body-link"><a href="#attr-body-link">link</a></code>     </td></tr><tr><td><dfn data-dfn-for="Document" id="dom-document-vlinkcolor" data-dfn-type="attribute"><code>vlinkColor</code></dfn>      </td><td><code id="other-elements,-attributes-and-apis:attr-body-vlink"><a href="#attr-body-vlink">vlink</a></code>     </td></tr><tr><td><dfn data-dfn-for="Document" id="dom-document-alinkcolor" data-dfn-type="attribute"><code>alinkColor</code></dfn>      </td><td><code id="other-elements,-attributes-and-apis:attr-body-alink"><a href="#attr-body-alink">alink</a></code>     </td></tr><tr><td><dfn data-dfn-for="Document" id="dom-document-bgcolor" data-dfn-type="attribute"><code>bgColor</code></dfn>      </td><td><code id="other-elements,-attributes-and-apis:attr-body-bgcolor"><a href="#attr-body-bgcolor">bgcolor</a></code>   </td></tr></tbody></table>

------------------------------------------------------------------------

The <a id="dom&#45;document&#45;anchors"></a><strong><code>anchors</code></strong> attribute must return an <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;htmlcollection&#45;3"></a><code>[HTMLCollection](https://dom.spec.whatwg.org/#interface-htmlcollection)</code> rooted at the <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;document&#45;2"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> node&#44; whose filter matches only <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;the&#45;a&#45;element"></a><code>[a](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element)</code> elements with <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;attr&#45;a&#45;name"></a><code>[name](#attr-a-name)</code> attributes&#46;

The <a id="dom&#45;document&#45;applets"></a><strong><code>applets</code></strong> attribute must return an <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;htmlcollection&#45;4"></a><code>[HTMLCollection](https://dom.spec.whatwg.org/#interface-htmlcollection)</code> rooted at the <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;document&#45;3"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> node&#44; whose filter matches nothing&#46; &#40;It exists for historical reasons&#46;&#41;

The <a id="dom&#45;document&#45;clear"></a><strong><code>clear&#40;&#41;</code></strong>&#44; <a id="dom&#45;document&#45;captureevents"></a><strong><code>captureEvents&#40;&#41;</code></strong>&#44; and <a id="dom&#45;document&#45;releaseevents"></a><strong><code>releaseEvents&#40;&#41;</code></strong> methods must do nothing&#46;

------------------------------------------------------------------------

The <a id="dom&#45;document&#45;all"></a><strong><code>all</code></strong> attribute must return an <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;htmlallcollection&#45;2"></a><code>[HTMLAllCollection](html-language-foundations--877d6146487f--b55e73c31978.md#htmlallcollection)</code> rooted at the <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;document&#45;4"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> node&#44; whose filter matches all elements&#46;

------------------------------------------------------------------------

<a id="Window&#45;partial"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;dom&#45;window&#45;captureevents"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;dom&#45;window&#45;releaseevents"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;external"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;dom&#45;external"></a>

``` text
partial interface Window {
  undefined captureEvents();
  undefined releaseEvents();

  [Replaceable, SameObject] readonly attribute External external;
};
```

The <a id="dom&#45;window&#45;captureevents"></a><strong><code>captureEvents&#40;&#41;</code></strong> and <a id="dom&#45;window&#45;releaseevents"></a><strong><code>releaseEvents&#40;&#41;</code></strong> methods must do nothing&#46;

The <a id="dom&#45;external"></a><strong><code>external</code></strong> attribute of the <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;window"></a><code>[Window](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window)</code> interface must return an instance of the <a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;external&#45;2"></a><code>[External](#external)</code> interface&#58;

<a id="external"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;dom&#45;external&#45;addsearchprovider"></a><a id="other&#45;elements&#44;&#45;attributes&#45;and&#45;apis&#58;dom&#45;external&#45;issearchproviderinstalled"></a>

``` text
[Exposed=Window]
interface External {
  undefined AddSearchProvider();
  undefined IsSearchProviderInstalled();
};
```

The <a id="dom&#45;external&#45;addsearchprovider"></a><strong><code>AddSearchProvider&#40;&#41;</code></strong> and <a id="dom&#45;external&#45;issearchproviderinstalled"></a><strong><code>IsSearchProviderInstalled&#40;&#41;</code></strong> methods must do nothing&#46;
