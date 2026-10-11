# HTML Standard: host facts

Attribution and reformatting notice added for Surgeist on 2026-10-11. Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Original English standard and normative/informative labels remain authoritative. This partial format conversion supports the selected HTML language baseline; it is not a replacement specification or an implementation-conformance claim. No endorsement is implied.

Source: [HTML Standard commit snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/). Source revision `877d6146487f15b15bc8dd0e3c625063a37b456a` (9 October 2026); retrieved 11 October 2026 UTC. This immutable snapshot is the selected language baseline, independently of the html5ever implementation ingestion revision.

The exact [WHATWG LICENSE](../licenses/whatwg/LICENSE), also present in the HTML repository at this revision, has SHA-256 `85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`. Specification prose is Creative Commons Attribution 4.0 International; the BSD 3-Clause scope applies when portions are incorporated into code, not as an alternative license for this Markdown.

Complete original HTML: 14,930,614 bytes, SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`. Complete-section extraction input: 308,515 bytes, SHA-256 `2dd822df9d5e4827029d5f9715860d60ba1b0f7ff596c50e254bfd3283596b83`.

Retained complete heading sections, including all descendant clauses and bounded by the next equal-or-higher-level heading: `the-hidden-attribute`, `inert-subtrees`, `user-activation-data-model`, `focus`, `disabled-elements`, `selectors`. Original snapshot header is retained. The [edition map](SOURCE-EDITIONS.md#pinned-html-language-baseline) records the ten companion captures and bounded ownership. Every §4 element definition is retained across the collection. Omitted browser runtime chapters remain linked to this immutable snapshot; retaining JavaScript interfaces, networking, media, script execution or custom-element algorithms as source evidence does not assign their implementation to the native HTML crate. HTML syntax, author conformance and required parsing recovery remain distinct; malformed input is not rejected solely for an author-conformance error.

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

### <a id="the&#45;hidden&#45;attribute"></a>6&#46;1 The <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden"></a><code>[hidden](#attr-hidden)</code> attribute[](#the-hidden-attribute)

<strong>⚠</strong>MDN

[Global&#95;attributes&#47;hidden](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/hidden)

Support in one engine only&#46;

FirefoxNoSafariNoChrome102&#43;

------------------------------------------------------------------------

OperaNoEdge102&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[Global&#95;attributes&#47;hidden](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/hidden)

Support in all current engines&#46;

Firefox4&#43;Safari5&#46;1&#43;Chrome10&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer11

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android4&#43;Samsung Internet&#63;Opera Android&#63;

All <a id="the&#45;hidden&#45;attribute&#58;html&#45;elements"></a>[HTML elements](html-language-foundations--877d6146487f--b55e73c31978.md#html-elements) may have the <a id="attr&#45;hidden"></a><strong><code>hidden</code></strong> content attribute set&#46; The <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;2"></a><code>[hidden](#attr-hidden)</code> attribute is an <a id="the&#45;hidden&#45;attribute&#58;enumerated&#45;attribute"></a>[enumerated attribute](html-language-foundations--877d6146487f--b55e73c31978.md#enumerated-attribute) with the following keywords and states&#58;

<table><thead><tr><th>Keyword      </th><th>State      </th><th>Brief description    </th></tr></thead><tbody><tr><td><dfn data-dfn-for="html-global/hidden" id="attr-hidden-hidden" data-dfn-type="attr-value"><code>hidden</code></dfn>      </td><td><dfn id="attr-hidden-hidden-state">Hidden</dfn>      </td><td>Will not be rendered&#46;     </td></tr><tr><td><dfn data-dfn-for="html-global/hidden" id="attr-hidden-until-found" data-dfn-type="attr-value"><code>until&#45;found</code></dfn>      </td><td><dfn id="attr-hidden-until-found-state">Hidden Until Found</dfn>      </td><td>Will not be rendered&#44; but content inside will be accessible to <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#find-in-page-2" id="the-hidden-attribute:find-in-page-2">find&#45;in&#45;page</a>      and <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigate-fragid" id="the-hidden-attribute:navigate-fragid">fragment navigation</a>&#46;   </td></tr></tbody></table>

The attribute&#39;s <a id="the&#45;hidden&#45;attribute&#58;missing&#45;value&#45;default"></a><em>[missing value default](html-language-foundations--877d6146487f--b55e73c31978.md#missing-value-default)</em> is the <a id="attr&#45;hidden&#45;not&#45;hidden&#45;state"></a><strong>Not Hidden</strong> state&#44; and its <a id="the&#45;hidden&#45;attribute&#58;invalid&#45;value&#45;default"></a><em>[invalid value default](html-language-foundations--877d6146487f--b55e73c31978.md#invalid-value-default)</em> and <a id="the&#45;hidden&#45;attribute&#58;empty&#45;value&#45;default"></a><em>[empty value default](html-language-foundations--877d6146487f--b55e73c31978.md#empty-value-default)</em> are both the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;hidden&#45;state"></a>[Hidden](#attr-hidden-hidden-state) state&#46;

When an element has the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;3"></a><code>[hidden](#attr-hidden)</code> attribute in the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;hidden&#45;state&#45;2"></a>[Hidden](#attr-hidden-hidden-state) state&#44; it indicates that the element is not yet&#44; or is no longer&#44; directly relevant to the page&#39;s current state&#44; or that it is being used to declare content to be reused by other parts of the page as opposed to being directly accessed by the user&#46; User agents should not render elements that are in the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;hidden&#45;state&#45;3"></a>[Hidden](#attr-hidden-hidden-state) state&#46;

<strong>Note&#58;</strong>

The requirement for user agents not to render elements that are in the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;hidden&#45;state&#45;4"></a>[Hidden](#attr-hidden-hidden-state) state can be implemented indirectly through the style layer&#46; For example&#44; a web browser could implement these requirements [using the rules suggested in the Rendering section](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#hiddenCSS)&#46;

When an element has the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;4"></a><code>[hidden](#attr-hidden)</code> attribute in the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;until&#45;found&#45;state"></a>[Hidden Until Found](#attr-hidden-until-found-state) state&#44; it indicates that the element is hidden like the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;hidden&#45;state&#45;5"></a>[Hidden](#attr-hidden-hidden-state) state but the content inside the element will be accessible to <a id="the&#45;hidden&#45;attribute&#58;find&#45;in&#45;page&#45;2&#45;2"></a>[find&#45;in&#45;page](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#find-in-page-2) and <a id="the&#45;hidden&#45;attribute&#58;navigate&#45;fragid&#45;2"></a>[fragment navigation](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigate-fragid)&#46; When these features attempt to scroll to a target which is in the element&#39;s subtree&#44; the user agent will remove the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;5"></a><code>[hidden](#attr-hidden)</code> attribute in order to reveal the content before scrolling to it by running the <a id="the&#45;hidden&#45;attribute&#58;ancestor&#45;revealing&#45;algorithm"></a>[ancestor revealing algorithm](#ancestor-revealing-algorithm) on the target node&#46;

<strong>Note&#58;</strong>

Web browsers will use &#39;content&#45;visibility&#58; hidden&#39; instead of &#39;display&#58; none&#39; when the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;6"></a><code>[hidden](#attr-hidden)</code> attribute is in the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;until&#45;found&#45;state&#45;2"></a>[Hidden Until Found](#attr-hidden-until-found-state) state&#44; as specified in the [Rendering section](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#hiddenCSS)&#46;

<strong>Note&#58;</strong>

Because this attribute is typically implemented using CSS&#44; it&#39;s also possible to override it using CSS&#46; For instance&#44; a rule that applies &#39;display&#58; block&#39; to all elements will cancel the effects of the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;hidden&#45;state&#45;6"></a>[Hidden](#attr-hidden-hidden-state) state&#46; Authors therefore have to take care when writing their style sheets to make sure that the attribute is still styled as expected&#46; In addition&#44; legacy user agents which don&#39;t support the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;until&#45;found&#45;state&#45;3"></a>[Hidden Until Found](#attr-hidden-until-found-state) state will have &#39;display&#58; none&#39; instead of &#39;content&#45;visibility&#58; hidden&#39;&#44; so authors are encouraged to make sure that their style sheets don&#39;t change the &#39;display&#39; or &#39;content&#45;visibility&#39; properties of <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;until&#45;found&#45;state&#45;4"></a>[Hidden Until Found](#attr-hidden-until-found-state) elements&#46;

Since elements with the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;7"></a><code>[hidden](#attr-hidden)</code> attribute in the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;until&#45;found&#45;state&#45;5"></a>[Hidden Until Found](#attr-hidden-until-found-state) state use &#39;content&#45;visibility&#58; hidden&#39; instead of &#39;display&#58; none&#39;&#44; there are two caveats of the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;until&#45;found&#45;state&#45;6"></a>[Hidden Until Found](#attr-hidden-until-found-state) state that make it different from the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;hidden&#45;state&#45;7"></a>[Hidden](#attr-hidden-hidden-state) state&#58;

1.  The element needs to be affected by <a id="the&#45;hidden&#45;attribute&#58;layout&#45;containment"></a>[layout containment](https://drafts.csswg.org/css-contain/#containment-layout) in order to be revealed by find&#45;in&#45;page&#46; This means that if the element in the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;until&#45;found&#45;state&#45;7"></a>[Hidden Until Found](#attr-hidden-until-found-state) state has a &#39;display&#39; value of &#39;none&#39;&#44; &#39;contents&#39;&#44; or &#39;inline&#39;&#44; then the element will not be revealed by find&#45;in&#45;page&#46;

2.  The element will still have a <a id="the&#45;hidden&#45;attribute&#58;&#39;visibility&#39;"></a>[generated box](https://drafts.csswg.org/css2/#propdef-visibility) when in the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;until&#45;found&#45;state&#45;8"></a>[Hidden Until Found](#attr-hidden-until-found-state) state&#44; which means that borders&#44; margin&#44; and padding will still be rendered around the element&#46;

<strong>Example&#58;</strong>

In the following skeletal example&#44; the attribute is used to hide the web game&#39;s main screen until the user logs in&#58;

``` text
  <h1>The Example Game</h1>
  <section id="login">
   <h2>Login</h2>
   <form>
    ...
    <!-- calls login() once the user's credentials have been checked -->
   </form>
   <script>
    function login() {
      // switch screens
      document.getElementById('login').hidden = true;
      document.getElementById('game').hidden = false;
    }
   </script>
  </section>
  <section id="game" hidden>
   ...
  </section>
```

The <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;8"></a><code>[hidden](#attr-hidden)</code> attribute must not be used to hide content that could legitimately be shown in another presentation&#46; For example&#44; it is incorrect to use <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;9"></a><code>[hidden](#attr-hidden)</code> to hide panels in a tabbed dialog&#44; because the tabbed interface is merely a kind of overflow presentation — one could equally well just show all the form controls in one big page with a scrollbar&#46; It is similarly incorrect to use this attribute to hide content just from one presentation — if something is marked <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;10"></a><code>[hidden](#attr-hidden)</code>&#44; it is hidden from all presentations&#44; including&#44; for instance&#44; screen readers&#46;

Elements that are not themselves <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;11"></a><code>[hidden](#attr-hidden)</code> must not <a id="the&#45;hidden&#45;attribute&#58;hyperlink"></a>[hyperlink](html-language-elements-text-links--877d6146487f--8833a85fd242.md#hyperlink) to elements that are <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;12"></a><code>[hidden](#attr-hidden)</code>&#46; The <code>for</code> attributes of <a id="the&#45;hidden&#45;attribute&#58;the&#45;label&#45;element"></a><code>[label](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-label-element)</code> and <a id="the&#45;hidden&#45;attribute&#58;the&#45;output&#45;element"></a><code>[output](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-output-element)</code> elements that are not themselves <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;13"></a><code>[hidden](#attr-hidden)</code> must similarly not refer to elements that are <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;14"></a><code>[hidden](#attr-hidden)</code>&#46; In both cases&#44; such references would cause user confusion&#46;

Elements and scripts may&#44; however&#44; refer to elements that are <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;15"></a><code>[hidden](#attr-hidden)</code> in other contexts&#46;

<strong>Example&#58;</strong>

For example&#44; it would be incorrect to use the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hyperlink&#45;href"></a><code>[href](html-language-elements-text-links--877d6146487f--8833a85fd242.md#attr-hyperlink-href)</code> attribute to link to a section marked with the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;16"></a><code>[hidden](#attr-hidden)</code> attribute&#46; If the content is not applicable or relevant&#44; then there is no reason to link to it&#46;

It would be fine&#44; however&#44; to use the ARIA <a id="the&#45;hidden&#45;attribute&#58;attr&#45;aria&#45;describedby"></a><code>[aria&#45;describedby](https://w3c.github.io/aria/#aria-describedby)</code> attribute to refer to descriptions that are themselves <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;17"></a><code>[hidden](#attr-hidden)</code>&#46; While hiding the descriptions implies that they are not useful alone&#44; they could be written in such a way that they are useful in the specific context of being referenced from the elements that they describe&#46;

Similarly&#44; a <a id="the&#45;hidden&#45;attribute&#58;the&#45;canvas&#45;element"></a><code>[canvas](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-canvas-element)</code> element with the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;18"></a><code>[hidden](#attr-hidden)</code> attribute could be used by a scripted graphics engine as an off&#45;screen buffer&#44; and a form control could refer to a hidden <a id="the&#45;hidden&#45;attribute&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> element using its <a id="the&#45;hidden&#45;attribute&#58;attr&#45;fae&#45;form"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-fae-form)</code> attribute&#46;

Elements in a section hidden by the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;19"></a><code>[hidden](#attr-hidden)</code> attribute are still active&#44; e&#46;g&#46; scripts and form controls in such sections still execute and submit respectively&#46; Only their presentation to the user changes&#46;

<strong>✔</strong>MDN

[HTMLElement&#47;hidden](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/hidden)

Support in all current engines&#46;

Firefox4&#43;Safari5&#46;1&#43;Chrome6&#43;

------------------------------------------------------------------------

Opera11&#46;6&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer11

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android3&#43;Samsung Internet&#63;Opera Android12&#43;

The <a id="dom&#45;hidden"></a><strong><code>hidden</code></strong> getter steps are&#58;

1.  If the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;20"></a><code>[hidden](#attr-hidden)</code> attribute is in the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;until&#45;found&#45;state&#45;9"></a>[Hidden Until Found](#attr-hidden-until-found-state) state&#44; then return &#34;<a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;until&#45;found"></a><code>[until&#45;found](#attr-hidden-until-found)</code>&#34;&#46;

2.  If the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;21"></a><code>[hidden](#attr-hidden)</code> attribute is set&#44; then return true&#46;

3.  Return false&#46;

The <a id="the&#45;hidden&#45;attribute&#58;dom&#45;hidden"></a><code>[hidden](#dom-hidden)</code> setter steps are&#58;

1.  If the given value is a string that is an <a id="the&#45;hidden&#45;attribute&#58;ascii&#45;case&#45;insensitive"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for &#34;<a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;until&#45;found&#45;2"></a><code>[until&#45;found](#attr-hidden-until-found)</code>&#34;&#44; then set the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;22"></a><code>[hidden](#attr-hidden)</code> attribute to &#34;<a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;until&#45;found&#45;3"></a><code>[until&#45;found](#attr-hidden-until-found)</code>&#34;&#46;

2.  Otherwise&#44; if the given value is false&#44; then remove the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;23"></a><code>[hidden](#attr-hidden)</code> attribute&#46;

3.  Otherwise&#44; if the given value is the empty string&#44; then remove the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;24"></a><code>[hidden](#attr-hidden)</code> attribute&#46;

4.  Otherwise&#44; if the given value is null&#44; then remove the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;25"></a><code>[hidden](#attr-hidden)</code> attribute&#46;

5.  Otherwise&#44; if the given value is 0&#44; then remove the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;26"></a><code>[hidden](#attr-hidden)</code> attribute&#46;

6.  Otherwise&#44; if the given value is NaN&#44; then remove the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;27"></a><code>[hidden](#attr-hidden)</code> attribute&#46;

7.  Otherwise&#44; set the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;28"></a><code>[hidden](#attr-hidden)</code> attribute to the empty string&#46;

An <a id="ancestor&#45;reveal&#45;pair"></a><strong>ancestor reveal pair</strong> is a <a id="the&#45;hidden&#45;attribute&#58;tuple"></a>[tuple](https://infra.spec.whatwg.org/#tuple) consisting of a <a id="ancestor&#45;reveal&#45;pair&#45;node"></a><strong>node</strong> and a <a id="ancestor&#45;reveal&#45;pair&#45;string"></a><strong>string</strong>&#46;

The <a id="ancestor&#45;revealing&#45;algorithm"></a><strong>ancestor revealing algorithm</strong> given a node <var>target</var> is&#58;

1.  Let <var>ancestorsToReveal</var> be « »&#46;

2.  Let <var>ancestor</var> be <var>target</var>&#46;

3.  While <var>ancestor</var> has a parent node within the <a id="the&#45;hidden&#45;attribute&#58;flat&#45;tree"></a>[flat tree](https://drafts.csswg.org/css-scoping/#flat-tree)&#58;

    1.  If <var>ancestor</var> has a <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;29"></a><code>[hidden](#attr-hidden)</code> attribute in the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;until&#45;found&#45;state&#45;10"></a>[Hidden Until Found](#attr-hidden-until-found-state) state&#44; then <a id="the&#45;hidden&#45;attribute&#58;list&#45;append"></a>[append](https://infra.spec.whatwg.org/#list-append) &#40;<var>ancestor</var>&#44; &#34;<code>until&#45;found</code>&#34;&#41; to <var>ancestorsToReveal</var>&#46;

    2.  If <var>ancestor</var> is slotted into the second slot of a <a id="the&#45;hidden&#45;attribute&#58;the&#45;details&#45;element"></a><code>[details](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-details-element)</code> element which does not have an <a id="the&#45;hidden&#45;attribute&#58;attr&#45;details&#45;open"></a><code>[open](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#attr-details-open)</code> attribute&#44; then <a id="the&#45;hidden&#45;attribute&#58;list&#45;append&#45;2"></a>[append](https://infra.spec.whatwg.org/#list-append) &#40;<var>ancestor</var>&#39;s parent node&#44; &#34;<code>details</code>&#34;&#41; to <var>ancestorsToReveal</var>&#46;

    3.  Set <var>ancestor</var> to the parent node of <var>ancestor</var> within the <a id="the&#45;hidden&#45;attribute&#58;flat&#45;tree&#45;2"></a>[flat tree](https://drafts.csswg.org/css-scoping/#flat-tree)&#46;

4.  For each &#40;<var>ancestorToReveal</var>&#44; <var>revealType</var>&#41; of <var>ancestorsToReveal</var>&#58;

    1.  If <var>ancestorToReveal</var> is not <a id="the&#45;hidden&#45;attribute&#58;connected"></a>[connected](https://dom.spec.whatwg.org/#connected)&#44; then return&#46;

    2.  If <var>revealType</var> is &#34;<code>until&#45;found</code>&#34;&#58;

        1.  If <var>ancestorToReveal</var>&#39;s <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;30"></a><code>[hidden](#attr-hidden)</code> attribute is not in the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;until&#45;found&#45;state&#45;11"></a>[Hidden Until Found](#attr-hidden-until-found-state) state&#44; then return&#46;

        2.  <a id="the&#45;hidden&#45;attribute&#58;concept&#45;event&#45;fire"></a>[Fire an event](https://dom.spec.whatwg.org/#concept-event-fire) named <a id="the&#45;hidden&#45;attribute&#58;event&#45;beforematch"></a><code>[beforematch](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-beforematch)</code> at <var>ancestorToReveal</var> with the <a id="the&#45;hidden&#45;attribute&#58;dom&#45;event&#45;bubbles"></a><code>[bubbles](https://dom.spec.whatwg.org/#dom-event-bubbles)</code> attribute initialized to true&#46;

        3.  If <var>ancestorToReveal</var> is not <a id="the&#45;hidden&#45;attribute&#58;connected&#45;2"></a>[connected](https://dom.spec.whatwg.org/#connected)&#44; then return&#46;

        4.  If <var>ancestorToReveal</var>&#39;s <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;31"></a><code>[hidden](#attr-hidden)</code> attribute is not in the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;until&#45;found&#45;state&#45;12"></a>[Hidden Until Found](#attr-hidden-until-found-state) state&#44; then return&#46;

        5.  Remove the <a id="the&#45;hidden&#45;attribute&#58;attr&#45;hidden&#45;32"></a><code>[hidden](#attr-hidden)</code> attribute from <var>ancestorToReveal</var>&#46;

    3.  Otherwise&#58;

        1.  <a id="the&#45;hidden&#45;attribute&#58;assert"></a>[Assert](https://infra.spec.whatwg.org/#assert)&#58; <var>revealType</var> is &#34;<code>details</code>&#34;&#46;

        2.  If <var>ancestorToReveal</var> has an <a id="the&#45;hidden&#45;attribute&#58;attr&#45;details&#45;open&#45;2"></a><code>[open](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#attr-details-open)</code> attribute&#44; then return&#46;

        3.  Set <var>ancestorToReveal</var>&#39;s <a id="the&#45;hidden&#45;attribute&#58;attr&#45;details&#45;open&#45;3"></a><code>[open](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#attr-details-open)</code> attribute to the empty string&#46;

### <a id="inert&#45;subtrees"></a>6&#46;3 Inert subtrees[](#inert-subtrees)

<strong>Note&#58;</strong>

See also <a id="inert&#45;subtrees&#58;the&#45;inert&#45;attribute"></a><code>[inert](#the-inert-attribute)</code> for an explanation of the attribute of the same name&#46;

A node &#40;in particular elements and text nodes&#41; can be <a id="inert"></a><strong>inert</strong>&#46; When a node is <a id="inert&#45;subtrees&#58;inert"></a>[inert](#inert)&#58;

- Hit&#45;testing must act as if the <a id="inert&#45;subtrees&#58;&#39;pointer&#45;events&#39;"></a>[&#39;pointer&#45;events&#39;](https://drafts.csswg.org/css-ui-4/#pointer-events-control) CSS property were set to &#39;none&#39;&#46;

- Text selection functionality must act as if the <a id="inert&#45;subtrees&#58;&#39;user&#45;select&#39;"></a>[&#39;user&#45;select&#39;](https://drafts.csswg.org/css-ui-4/#content-selection) CSS property were set to &#39;none&#39;&#46;

- If it is <a id="inert&#45;subtrees&#58;editable"></a>[editable](https://w3c.github.io/editing/docs/execCommand/#editable)&#44; the node behaves as if it were non&#45;editable&#46;

- The user agent should ignore the node for the purposes of <a id="inert&#45;subtrees&#58;find&#45;in&#45;page&#45;2"></a>[find&#45;in&#45;page](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#find-in-page-2)&#46;

<strong>Note&#58;</strong>

Inert nodes generally cannot be focused&#44; and user agents do not expose the inert nodes to accessibility APIs or assistive technologies&#46; Inert nodes that are <a id="inert&#45;subtrees&#58;concept&#45;command"></a>[commands](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#concept-command) will become inoperable to users&#44; in the manner described above&#46;

User agents may allow the user to override the restrictions on <a id="inert&#45;subtrees&#58;find&#45;in&#45;page&#45;2&#45;2"></a>[find&#45;in&#45;page](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#find-in-page-2) and text selection&#44; however&#46;

By default&#44; a node is not <a id="inert&#45;subtrees&#58;inert&#45;2"></a>[inert](#inert)&#46;

#### <a id="modal&#45;dialogs&#45;and&#45;inert&#45;subtrees"></a>6&#46;3&#46;1 Modal dialogs and inert subtrees[](#modal-dialogs-and-inert-subtrees)

A <a id="modal&#45;dialogs&#45;and&#45;inert&#45;subtrees&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> <var>document</var> is <a id="blocked&#45;by&#45;a&#45;modal&#45;dialog"></a><strong>blocked by a modal dialog</strong> <var>subject</var> if <var>subject</var> is the topmost <a id="modal&#45;dialogs&#45;and&#45;inert&#45;subtrees&#58;the&#45;dialog&#45;element"></a><code>[dialog](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-dialog-element)</code> element in <var>document</var>&#39;s <a id="modal&#45;dialogs&#45;and&#45;inert&#45;subtrees&#58;top&#45;layer"></a>[top layer](https://drafts.csswg.org/css-position-4/#document-top-layer)&#46; While <var>document</var> is so blocked&#44; every node that is <a id="modal&#45;dialogs&#45;and&#45;inert&#45;subtrees&#58;connected"></a>[connected](https://dom.spec.whatwg.org/#connected) to <var>document</var>&#44; with the exception of the <var>subject</var> element and its <a id="modal&#45;dialogs&#45;and&#45;inert&#45;subtrees&#58;flat&#45;tree"></a>[flat tree](https://drafts.csswg.org/css-scoping/#flat-tree) descendants&#44; must become <a id="modal&#45;dialogs&#45;and&#45;inert&#45;subtrees&#58;inert"></a>[inert](#inert)&#46;

<var>subject</var> can additionally become <a id="modal&#45;dialogs&#45;and&#45;inert&#45;subtrees&#58;inert&#45;2"></a>[inert](#inert) via the <a id="modal&#45;dialogs&#45;and&#45;inert&#45;subtrees&#58;the&#45;inert&#45;attribute"></a><code>[inert](#the-inert-attribute)</code> attribute&#44; but only if specified on <var>subject</var> itself &#40;i&#46;e&#46;&#44; <var>subject</var> escapes inertness of ancestors&#41;&#59; <var>subject</var>&#39;s <a id="modal&#45;dialogs&#45;and&#45;inert&#45;subtrees&#58;flat&#45;tree&#45;2"></a>[flat tree](https://drafts.csswg.org/css-scoping/#flat-tree) descendants can become <a id="modal&#45;dialogs&#45;and&#45;inert&#45;subtrees&#58;inert&#45;3"></a>[inert](#inert) in a similar fashion&#46;

<strong>Note&#58;</strong>

The <a id="modal&#45;dialogs&#45;and&#45;inert&#45;subtrees&#58;the&#45;dialog&#45;element&#45;2"></a><code>[dialog](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-dialog-element)</code> element&#39;s <a id="modal&#45;dialogs&#45;and&#45;inert&#45;subtrees&#58;dom&#45;dialog&#45;showmodal"></a><code>[showModal&#40;&#41;](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#dom-dialog-showmodal)</code> method causes this mechanism to trigger&#44; by <a id="modal&#45;dialogs&#45;and&#45;inert&#45;subtrees&#58;add&#45;an&#45;element&#45;to&#45;the&#45;top&#45;layer"></a>[adding](https://drafts.csswg.org/css-position-4/#add-an-element-to-the-top-layer) the <a id="modal&#45;dialogs&#45;and&#45;inert&#45;subtrees&#58;the&#45;dialog&#45;element&#45;3"></a><code>[dialog](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-dialog-element)</code> element to its <a id="modal&#45;dialogs&#45;and&#45;inert&#45;subtrees&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="modal&#45;dialogs&#45;and&#45;inert&#45;subtrees&#58;top&#45;layer&#45;2"></a>[top layer](https://drafts.csswg.org/css-position-4/#document-top-layer)&#46;

#### <a id="the&#45;inert&#45;attribute"></a>6&#46;3&#46;2 The <strong><code>inert</code></strong> attribute[](#the-inert-attribute)

<strong>✔</strong>MDN

[Global&#95;attributes&#47;inert](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/inert)

Support in all current engines&#46;

Firefox112&#43;Safari15&#46;5&#43;Chrome102&#43;

------------------------------------------------------------------------

Opera&#63;Edge102&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

The <a id="the&#45;inert&#45;attribute&#58;the&#45;inert&#45;attribute"></a><code>[inert](#the-inert-attribute)</code> attribute is a <a id="the&#45;inert&#45;attribute&#58;boolean&#45;attribute"></a>[boolean attribute](html-language-foundations--877d6146487f--b55e73c31978.md#boolean-attribute) that indicates&#44; by its presence&#44; that the element and all its <a id="the&#45;inert&#45;attribute&#58;flat&#45;tree"></a>[flat tree](https://drafts.csswg.org/css-scoping/#flat-tree) descendants which don&#39;t otherwise escape inertness &#40;such as modal dialogs&#41; are to be made <a id="the&#45;inert&#45;attribute&#58;inert"></a>[inert](#inert) by the user agent&#46;

An inert subtree should not contain any content or controls which are critical to understanding or using aspects of the page which are not in the inert state&#46; Content in an inert subtree will not be perceivable by all users&#44; or interactive&#46; Authors should not specify elements as inert unless the content they represent are also visually obscured in some way&#46; In most cases&#44; authors should not specify the <a id="the&#45;inert&#45;attribute&#58;inert&#45;2"></a><code>[inert](#inert)</code> attribute on individual form controls&#46; In these instances&#44; the <a id="the&#45;inert&#45;attribute&#58;attr&#45;fe&#45;disabled"></a><code>[disabled](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-fe-disabled)</code> attribute is probably more appropriate&#46;

<strong>Example&#58;</strong>

The following example shows how to mark partially loaded content&#44; visually obscured by a &#34;loading&#34; message&#44; as inert&#46;

``` text
<section aria-labelledby=s1>
  <h3 id=s1>Population by City</h3>
  <div class=container>
    <div class=loading><p>Loading...</p></div>
    <div inert>
      <form>
        <fieldset>
          <legend>Date range</legend>
          <div>
            <label for=start>Start</label>
            <input type=date id=start>
          </div>
          <div>
            <label for=end>End</label>
            <input type=date id=end>
          </div>
          <div>
            <button>Apply</button>
          </div>
        </fieldset>
      </form>
      <table>
        <caption>From 20-- to 20--</caption>
        <thead>
          <tr>
            <th>City</th>
            <th>State</th>
            <th>20-- Population</th>
            <th>20-- Population</th>
            <th>Percentage change</th>
          </tr>
        </thead>
        <tbody>
          <!-- ... -->
        </tbody>
      </table>
    </div>
  </div>
</section>
```

![Screenshot of Population by City content with an overlaid loading message which visually obscures the form controls and data table which have not fully rendered&#44; and thus are in the inert state&#46;](https://html.spec.whatwg.org/images/inert-example-loading-section.png)

The &#34;loading&#34; overlay obscures the inert content&#44; making it visually apparent that the inert content is not presently accessible&#46; Notice that the heading and &#34;loading&#34; text are not descendants of the element with the <a id="the&#45;inert&#45;attribute&#58;inert&#45;3"></a><code>[inert](#inert)</code> attribute&#46; This will ensure this text is accessible to all users&#44; while the inert content cannot be interacted with by anyone&#46;

<strong>Note&#58;</strong>

By default&#44; there is no persistent visual indication of an element or its subtree being inert&#46; Appropriate visual styles for such content is often context&#45;dependent&#46; For instance&#44; an inert off&#45;screen navigation panel would not require a default style&#44; as its off&#45;screen position visually obscures the content&#46; Similarly&#44; a modal <a id="the&#45;inert&#45;attribute&#58;the&#45;dialog&#45;element"></a><code>[dialog](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-dialog-element)</code> element&#39;s backdrop will serve as the means to visually obscure the inert content of the web page&#44; rather than styling the inert content specifically&#46;

However&#44; for many other situations authors are strongly encouraged to clearly mark what parts of their document are active and which are inert&#44; to avoid user confusion&#46; In particular&#44; it is worth remembering that not all users can see all parts of a page at once&#59; for example&#44; users of screen readers&#44; users on small devices or with magnifiers&#44; and even users using particularly small windows might not be able to see the active part of a page and might get frustrated if inert sections are not obviously inert&#46;

#### <a id="user&#45;activation&#45;data&#45;model"></a>6&#46;4&#46;1 Data model[](#user-activation-data-model)

For the purpose of tracking user activation&#44; each <a id="user&#45;activation&#45;data&#45;model&#58;window"></a><code>[Window](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window)</code> <var>W</var> has two relevant values&#58;

- A <a id="last&#45;activation&#45;timestamp"></a><strong>last activation timestamp</strong>&#44; which is either a <a id="user&#45;activation&#45;data&#45;model&#58;domhighrestimestamp"></a><code>[DOMHighResTimeStamp](https://w3c.github.io/hr-time/#dom-domhighrestimestamp)</code>&#44; positive infinity &#40;indicating that <var>W</var> has never been activated&#41;&#44; or negative infinity &#40;indicating that the activation has been <a id="user&#45;activation&#45;data&#45;model&#58;consume&#45;user&#45;activation"></a>[consumed](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#consume-user-activation)&#41;&#46; Initially positive infinity&#46;

- A <a id="last&#45;history&#45;action&#45;activation&#45;timestamp"></a><strong>last history&#45;action activation timestamp</strong>&#44; which is either a <a id="user&#45;activation&#45;data&#45;model&#58;domhighrestimestamp&#45;2"></a><code>[DOMHighResTimeStamp](https://w3c.github.io/hr-time/#dom-domhighrestimestamp)</code> or positive infinity&#44; initially positive infinity&#46;

A user agent also defines a <a id="transient&#45;activation&#45;duration"></a><strong>transient activation duration</strong>&#44; which is a constant number indicating how long a user activation is available for certain [user activation&#45;gated APIs](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#user-activation-gated-apis) &#40;e&#46;g&#46;&#44; for opening popups&#41;&#46;

<strong>Note&#58;</strong>

The <a id="user&#45;activation&#45;data&#45;model&#58;transient&#45;activation&#45;duration"></a>[transient activation duration](#transient-activation-duration) is expected be at most a few seconds&#44; so that the user can possibly perceive the link between an interaction with the page and the page calling the activation&#45;gated API&#46;

We then have the following boolean user activation states for <var>W</var>&#58;

<strong><a id="sticky&#45;activation"></a><strong>Sticky activation</strong></strong>

When the <a id="user&#45;activation&#45;data&#45;model&#58;current&#45;high&#45;resolution&#45;time"></a>[current high resolution time](https://w3c.github.io/hr-time/#dfn-current-high-resolution-time) given <var>W</var> is greater than or equal to the <a id="user&#45;activation&#45;data&#45;model&#58;last&#45;activation&#45;timestamp"></a>[last activation timestamp](#last-activation-timestamp) in <var>W</var>&#44; <var>W</var> is said to have <a id="user&#45;activation&#45;data&#45;model&#58;sticky&#45;activation"></a>[sticky activation](#sticky-activation)&#46;

This is <var>W</var>&#39;s historical activation state&#44; indicating whether the user has ever interacted in <var>W</var>&#46; It starts false&#44; then changes to true &#40;and never changes back to false&#41; when <var>W</var> gets the very first <a id="user&#45;activation&#45;data&#45;model&#58;activation&#45;notification"></a>[activation notification](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#activation-notification)&#46;

<strong><a id="transient&#45;activation"></a><strong>Transient activation</strong></strong>

When the <a id="user&#45;activation&#45;data&#45;model&#58;current&#45;high&#45;resolution&#45;time&#45;2"></a>[current high resolution time](https://w3c.github.io/hr-time/#dfn-current-high-resolution-time) given <var>W</var> is greater than or equal to the <a id="user&#45;activation&#45;data&#45;model&#58;last&#45;activation&#45;timestamp&#45;2"></a>[last activation timestamp](#last-activation-timestamp) in <var>W</var>&#44; and less than the <a id="user&#45;activation&#45;data&#45;model&#58;last&#45;activation&#45;timestamp&#45;3"></a>[last activation timestamp](#last-activation-timestamp) in <var>W</var> plus the <a id="user&#45;activation&#45;data&#45;model&#58;transient&#45;activation&#45;duration&#45;2"></a>[transient activation duration](#transient-activation-duration)&#44; then <var>W</var> is said to have <a id="user&#45;activation&#45;data&#45;model&#58;transient&#45;activation"></a>[transient activation](#transient-activation)&#46;

This is <var>W</var>&#39;s current activation state&#44; indicating whether the user has interacted in <var>W</var> recently&#46; This starts with a false value&#44; and remains true for a limited time after every <a id="user&#45;activation&#45;data&#45;model&#58;activation&#45;notification&#45;2"></a>[activation notification](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#activation-notification) <var>W</var> gets&#46;

The <a id="user&#45;activation&#45;data&#45;model&#58;transient&#45;activation&#45;2"></a>[transient activation](#transient-activation) state is considered <a id="activation&#45;expiry"></a><strong>expired</strong> if it becomes false because the <a id="user&#45;activation&#45;data&#45;model&#58;transient&#45;activation&#45;duration&#45;3"></a>[transient activation duration](#transient-activation-duration) time has elapsed since the last user activation&#46; Note that it can become false even before the expiry time through an <a id="user&#45;activation&#45;data&#45;model&#58;consume&#45;user&#45;activation&#45;2"></a>[activation consumption](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#consume-user-activation)&#46;

<strong><a id="history&#45;action&#45;activation"></a><strong>History&#45;action activation</strong></strong>

When the <a id="user&#45;activation&#45;data&#45;model&#58;last&#45;history&#45;action&#45;activation&#45;timestamp"></a>[last history&#45;action activation timestamp](#last-history-action-activation-timestamp) of <var>W</var> is not equal to the <a id="user&#45;activation&#45;data&#45;model&#58;last&#45;activation&#45;timestamp&#45;4"></a>[last activation timestamp](#last-activation-timestamp) of <var>W</var>&#44; then <var>W</var> is said to have <a id="user&#45;activation&#45;data&#45;model&#58;history&#45;action&#45;activation"></a>[history&#45;action activation](#history-action-activation)&#46;

This is a special variant of user activation&#44; used to allow access to certain session history APIs which&#44; if used too frequently&#44; would make it harder for the user to traverse back using [browser UI](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-traversal-ui)&#46; It starts with a false value&#44; and becomes true whenever the user interacts with <var>W</var>&#44; but is reset to false through <a id="user&#45;activation&#45;data&#45;model&#58;consume&#45;history&#45;action&#45;user&#45;activation"></a>[history&#45;action activation consumption](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#consume-history-action-user-activation)&#46; This ensures such APIs cannot be used multiple times in a row without an intervening user activation&#46; But unlike <a id="user&#45;activation&#45;data&#45;model&#58;transient&#45;activation&#45;3"></a>[transient activation](#transient-activation)&#44; there is no time limit within which such APIs must be used&#46;

<strong>Note&#58;</strong>

The <a id="user&#45;activation&#45;data&#45;model&#58;last&#45;activation&#45;timestamp&#45;5"></a>[last activation timestamp](#last-activation-timestamp) and <a id="user&#45;activation&#45;data&#45;model&#58;last&#45;history&#45;action&#45;activation&#45;timestamp&#45;2"></a>[last history&#45;action activation timestamp](#last-history-action-activation-timestamp) are retained even after the <a id="user&#45;activation&#45;data&#45;model&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> changes its <a id="user&#45;activation&#45;data&#45;model&#58;fully&#45;active"></a>[fully active](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fully-active) status &#40;e&#46;g&#46;&#44; after navigating away from a <a id="user&#45;activation&#45;data&#45;model&#58;document&#45;2"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#44; or navigating to a cached <a id="user&#45;activation&#45;data&#45;model&#58;document&#45;3"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#41;&#46; This means <a id="user&#45;activation&#45;data&#45;model&#58;sticky&#45;activation&#45;2"></a>[sticky activation](#sticky-activation) state spans multiple navigations as long as the same <a id="user&#45;activation&#45;data&#45;model&#58;document&#45;4"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> gets reused&#46; For the transient activation state&#44; the original <a id="user&#45;activation&#45;data&#45;model&#58;activation&#45;expiry"></a>[expiry](#activation-expiry) time remains unchanged &#40;i&#46;e&#46;&#44; the state still expires within the <a id="user&#45;activation&#45;data&#45;model&#58;transient&#45;activation&#45;duration&#45;4"></a>[transient activation duration](#transient-activation-duration) limit from the original <a id="user&#45;activation&#45;data&#45;model&#58;activation&#45;triggering&#45;input&#45;event"></a>[activation triggering input event](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#activation-triggering-input-event)&#41;&#46; It is important to consider this when deciding whether to base certain things off <a id="user&#45;activation&#45;data&#45;model&#58;sticky&#45;activation&#45;3"></a>[sticky activation](#sticky-activation) or <a id="user&#45;activation&#45;data&#45;model&#58;transient&#45;activation&#45;4"></a>[transient activation](#transient-activation)&#46;

### <a id="focus"></a>6&#46;6 Focus[](#focus)

#### <a id="introduction&#45;8"></a>6&#46;6&#46;1 Introduction[](#introduction-8)

<em>This section is non&#45;normative&#46;</em>

An HTML user interface typically consists of multiple interactive widgets&#44; such as form controls&#44; scrollable regions&#44; links&#44; dialog boxes&#44; browser tabs&#44; and so forth&#46; These widgets form a hierarchy&#44; with some &#40;e&#46;g&#46; browser tabs&#44; dialog boxes&#41; containing others &#40;e&#46;g&#46; links&#44; form controls&#41;&#46;

When interacting with an interface using a keyboard&#44; key input is channeled from the system&#44; through the hierarchy of interactive widgets&#44; to an active widget&#44; which is said to be <a id="introduction&#45;8&#58;focused"></a>[focused](#focused)&#46;

<strong>Example&#58;</strong>

Consider an HTML application running in a browser tab running in a graphical environment&#46; Suppose this application had a page with some text controls and links&#44; and was currently showing a modal dialog&#44; which itself had a text control and a button&#46;

The hierarchy of focusable widgets&#44; in this scenario&#44; would include the browser window&#44; which would have&#44; amongst its children&#44; the browser tab containing the HTML application&#46; The tab itself would have as its children the various links and text controls&#44; as well as the dialog&#46; The dialog itself would have as its children the text control and the button&#46;

![](https://html.spec.whatwg.org/images/focus-tree.png)

If the widget with <a id="introduction&#45;8&#58;focused&#45;2"></a>[focus](#focused) in this example was the text control in the dialog box&#44; then key input would be channeled from the graphical system to ① the web browser&#44; then to ② the tab&#44; then to ③ the dialog&#44; and finally to ④ the text control&#46;

Keyboard <em>events</em> are always targeted at this <a id="introduction&#45;8&#58;focused&#45;3"></a>[focused](#focused) element&#46;

#### <a id="data&#45;model"></a>6&#46;6&#46;2 Data model[](#data-model)

<a id="tlbc&#45;system&#45;focus"></a>A <a id="data&#45;model&#58;top&#45;level&#45;traversable"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable) has <a id="system&#45;focus"></a><strong>system focus</strong> when it can receive keyboard input channeled from the operating system&#44; possibly targeted at one of its <a id="data&#45;model&#58;nav&#45;document"></a>[active document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document)&#39;s <a id="data&#45;model&#58;descendant&#45;navigables"></a>[descendant navigables](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#descendant-navigables)&#46;

A <a id="data&#45;model&#58;top&#45;level&#45;traversable&#45;2"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable) has <a id="user&#45;attention"></a><strong>user attention</strong> when its <a id="data&#45;model&#58;system&#45;visibility&#45;state"></a>[system visibility state](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#system-visibility-state) is &#34;<code>visible</code>&#34;&#44; and it either has <a id="data&#45;model&#58;system&#45;focus"></a>[system focus](#system-focus) or user agent widgets directly related to it can receive keyboard input channeled from the operating system&#46;

<strong>Note&#58;</strong>

User attention is lost when a browser window loses focus&#44; whereas system focus might also be lost to other system widgets in the browser window such as a location bar&#46;

A <a id="data&#45;model&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> <var>d</var> is a <a id="fully&#45;active&#45;descendant&#45;of&#45;a&#45;top&#45;level&#45;traversable&#45;with&#45;user&#45;attention"></a><strong>fully active descendant of a top&#45;level traversable with user attention</strong> when <var>d</var> is <a id="data&#45;model&#58;fully&#45;active"></a>[fully active](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fully-active) and <var>d</var>&#39;s <a id="data&#45;model&#58;node&#45;navigable"></a>[node navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable)&#39;s <a id="data&#45;model&#58;nav&#45;top"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-top) has <a id="data&#45;model&#58;user&#45;attention"></a>[user attention](#user-attention)&#46;

The term <a id="focusable&#45;area"></a><strong>focusable area</strong> is used to refer to regions of the interface that can further become the target of such keyboard input&#46; Focusable areas can be elements&#44; parts of elements&#44; or other regions managed by the user agent&#46;

Each <a id="data&#45;model&#58;focusable&#45;area"></a>[focusable area](#focusable-area) has a <a id="dom&#45;anchor"></a><strong>DOM anchor</strong>&#44; which is a <a id="data&#45;model&#58;node"></a><code>[Node](https://dom.spec.whatwg.org/#interface-node)</code> object that represents the position of the <a id="data&#45;model&#58;focusable&#45;area&#45;2"></a>[focusable area](#focusable-area) in the DOM&#46; &#40;When the <a id="data&#45;model&#58;focusable&#45;area&#45;3"></a>[focusable area](#focusable-area) is itself a <a id="data&#45;model&#58;node&#45;2"></a><code>[Node](https://dom.spec.whatwg.org/#interface-node)</code>&#44; it is its own <a id="data&#45;model&#58;dom&#45;anchor"></a>[DOM anchor](#dom-anchor)&#46;&#41; The <a id="data&#45;model&#58;dom&#45;anchor&#45;2"></a>[DOM anchor](#dom-anchor) is used in some APIs as a substitute for the <a id="data&#45;model&#58;focusable&#45;area&#45;4"></a>[focusable area](#focusable-area) when there is no other DOM object to represent the <a id="data&#45;model&#58;focusable&#45;area&#45;5"></a>[focusable area](#focusable-area)&#46;

The following table describes what objects can be <a id="data&#45;model&#58;focusable&#45;area&#45;6"></a>[focusable areas](#focusable-area)&#46; The cells in the left column describe objects that can be <a id="data&#45;model&#58;focusable&#45;area&#45;7"></a>[focusable areas](#focusable-area)&#59; the cells in the right column describe the <a id="data&#45;model&#58;dom&#45;anchor&#45;3"></a>[DOM anchors](#dom-anchor) for those elements&#46; &#40;The cells that span both columns are non&#45;normative examples&#46;&#41;

<table id="table-fa"><thead><tr><th id="th-fa-area"><a href="#focusable-area" id="data-model:focusable-area-8">Focusable area</a>      </th><th id="th-fa-dom-rep"><a href="#dom-anchor" id="data-model:dom-anchor-4">DOM anchor</a>     </th></tr><tr><th colspan="2" id="th-fa-examples">Examples     </th></tr></thead><tbody><tr><th headers="th-fa-area" id="td-fa-1" class="data-header">      Elements that meet all the following criteria&#58;       <ul class="brief"><li>the element&#39;s <a href="#tabindex-value" id="data-model:tabindex-value">tabindex value</a> is non&#45;null&#44; or the element is determined by the       user agent to be focusable&#59;</li><li>the element is either not a <a id="data-model:shadow-host" href="https://dom.spec.whatwg.org/#element-shadow-host" data-x-internal="shadow-host">shadow host</a>&#44; or has a <a href="https://dom.spec.whatwg.org/#concept-element-shadow-root" id="data-model:concept-element-shadow-root" data-x-internal="concept-element-shadow-root">shadow root</a> whose <a id="data-model:delegates-focus" href="https://dom.spec.whatwg.org/#shadowroot-delegates-focus" data-x-internal="delegates-focus">delegates focus</a> is       false&#59;</li><li>the element is not <a href="#concept-element-disabled" id="data-model:concept-element-disabled">actually disabled</a>&#59;</li><li>the element is not <a href="#inert" id="data-model:inert">inert</a>&#59;</li><li>the element is either <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#being-rendered" id="data-model:being-rendered">being rendered</a>&#44; <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#delegating-its-rendering-to-its-children" id="data-model:delegating-its-rendering-to-its-children">delegating its rendering to its       children</a>&#44; or <a href="html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#being-used-as-relevant-canvas-fallback-content" id="data-model:being-used-as-relevant-canvas-fallback-content">being used as relevant canvas fallback content</a>&#46;</li></ul>      </th><td headers="td-fa-1 th-fa-dom-rep">      The element itself&#46;     </td></tr><tr><td headers="td-fa-1 th-fa-examples" colspan="2">      <p class="example"><code id="data-model:the-iframe-element"><a href="html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element">iframe</a></code>&#44; <code id="data-model:the-dialog-element"><a href="html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-dialog-element">dialog</a></code>&#44; <code id="data-model:text-(type=text)-state-and-search-state-(type=search)"><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#text-(type=text)-state-and-search-state-(type=search)">&lt;input type&#61;text&gt;</a></code>&#44; sometimes <code id="data-model:the-a-element"><a href="html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element">&lt;a&#10;     href&#61;&#34;&#34;&gt;</a></code> &#40;depending on platform conventions&#41;&#46;     </p></td></tr></tbody><tbody><tr><th headers="th-fa-area" id="td-fa-2" class="data-header">      The shapes of <code id="data-model:the-area-element"><a href="html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element">area</a></code> elements in an <a href="html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#image-map" id="data-model:image-map">image map</a> associated with an      <code id="data-model:the-img-element"><a href="html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-img-element">img</a></code> element that is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#being-rendered" id="data-model:being-rendered-2">being rendered</a> and is not <a href="#inert" id="data-model:inert-2">inert</a>&#46;      </th><td headers="td-fa-2 th-fa-dom-rep">      The <code id="data-model:the-img-element-2"><a href="html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-img-element">img</a></code> element&#46;     </td></tr><tr><td headers="td-fa-2 th-fa-examples" colspan="2">      <div class="example">       <p>In the following example&#44; the <code id="data-model:the-area-element-2"><a href="html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element">area</a></code> element creates two shapes&#44; one on each       image&#46; The <a href="#dom-anchor" id="data-model:dom-anchor-5">DOM anchor</a> of the first shape is the first <code id="data-model:the-img-element-3"><a href="html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-img-element">img</a></code> element&#44; and the       <a href="#dom-anchor" id="data-model:dom-anchor-6">DOM anchor</a> of the second shape is the second <code id="data-model:the-img-element-4"><a href="html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-img-element">img</a></code> element&#46;</p>       <code>&lt;map id&#61;wallmap&gt;&lt;area alt&#61;&#34;Enter Door&#34; coords&#61;&#34;10&#44;10&#44;100&#44;200&#34; href&#61;&#34;door&#46;html&#34;&gt;&lt;&#47;map&gt;&#10;&#46;&#46;&#46;&#10;&lt;img src&#61;&#34;images&#47;innerwall&#46;jpeg&#34; alt&#61;&#34;There is a white wall here&#44; with a door&#46;&#34; usemap&#61;&#34;&#35;wallmap&#34;&gt;&#10;&#46;&#46;&#46;&#10;&lt;img src&#61;&#34;images&#47;outerwall&#46;jpeg&#34; alt&#61;&#34;There is a red wall here&#44; with a door&#46;&#34; usemap&#61;&#34;&#35;wallmap&#34;&gt;</code>      </div>     </td></tr></tbody><tbody><tr><th headers="th-fa-area" id="td-fa-3" class="data-header">      The user&#45;agent provided subwidgets of elements that are <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#being-rendered" id="data-model:being-rendered-3">being rendered</a> and are not      <a href="#concept-element-disabled" id="data-model:concept-element-disabled-2">actually disabled</a> or <a href="#inert" id="data-model:inert-3">inert</a>&#46;      </th><td headers="td-fa-3 th-fa-dom-rep">      The element for which the <a href="#focusable-area" id="data-model:focusable-area-9">focusable area</a> is a subwidget&#46;     </td></tr><tr><td headers="td-fa-3 th-fa-examples" colspan="2">      <p class="example">The <a href="html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#expose-a-user-interface-to-the-user" id="data-model:expose-a-user-interface-to-the-user">controls in the user      interface</a> for a <code id="data-model:the-video-element"><a href="html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-video-element">video</a></code> element&#44; the up and down buttons in a spin&#45;control      version of <code id="data-model:number-state-(type=number)"><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#number-state-(type=number)">&lt;input type&#61;number&gt;</a></code>&#44; the part of a      <code id="data-model:the-details-element"><a href="html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-details-element">details</a></code> element&#39;s rendering that enables the element to be opened or closed using      keyboard input&#46;</p>     </td></tr></tbody><tbody><tr><th headers="th-fa-area" id="td-fa-4" class="data-header">      The scrollable regions of elements that are <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#being-rendered" id="data-model:being-rendered-4">being rendered</a> and are not      <a href="#inert" id="data-model:inert-4">inert</a>&#46;       </th><td headers="td-fa-4 th-fa-dom-rep">      The element for which the box that the scrollable region scrolls was created&#46;     </td></tr><tr><td headers="td-fa-4 th-fa-examples" colspan="2">      <p class="example">The CSS <a id="data-model:'overflow'" href="https://drafts.csswg.org/css-overflow/#propdef-overflow" data-x-internal="'overflow'">&#39;overflow&#39;</a> property&#39;s &#39;scroll&#39; value typically creates      a scrollable region&#46;</p>     </td></tr></tbody><tbody><tr><th headers="th-fa-area" id="td-fa-5" class="data-header">      The <a id="data-model:viewport" href="https://drafts.csswg.org/css2/#viewport" data-x-internal="viewport">viewport</a> of a <code id="data-model:document-2"><a href="html-language-foundations--877d6146487f--b55e73c31978.md#document">Document</a></code> that has a non&#45;null <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id="data-model:concept-document-bc">browsing context</a> and is not <a href="#inert" id="data-model:inert-5">inert</a>&#46;      </th><td headers="td-fa-5 th-fa-dom-rep">      The <code id="data-model:document-3"><a href="html-language-foundations--877d6146487f--b55e73c31978.md#document">Document</a></code> for which the <a id="data-model:viewport-2" href="https://drafts.csswg.org/css2/#viewport" data-x-internal="viewport">viewport</a> was created&#46;     </td></tr><tr><td headers="td-fa-5 th-fa-examples" colspan="2">      <p class="example">The contents of an <code id="data-model:the-iframe-element-2"><a href="html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element">iframe</a></code>&#46;</p>     </td></tr></tbody><tbody><tr><th headers="th-fa-area" id="td-fa-6" class="data-header">      Any other element or part of an element determined by the user agent to be a focusable area&#44;      especially to aid with accessibility or to better match platform conventions&#46;      </th><td headers="td-fa-6 th-fa-dom-rep">      The element&#46;     </td></tr><tr><td headers="td-fa-6 th-fa-examples" colspan="2">      <p class="example">A user agent could make all list item bullets <a href="#sequentially-focusable" id="data-model:sequentially-focusable">sequentially      focusable</a>&#44; so that a user can more easily navigate lists&#46;</p>      <p class="example">Similarly&#44; a user agent could make all elements with <code id="data-model:attr-title"><a href="html-language-foundations--877d6146487f--b55e73c31978.md#attr-title">title</a></code> attributes <a href="#sequentially-focusable" id="data-model:sequentially-focusable-2">sequentially focusable</a>&#44; so that their      advisory information can be accessed&#46;</p>    </td></tr></tbody></table>

<a id="bc&#45;focus&#45;ergo&#45;bcc&#45;focus"></a>

<strong>Note&#58;</strong>

[](#bc-focus-ergo-bcc-focus)A <a id="data&#45;model&#58;navigable&#45;container"></a>[navigable container](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container) &#40;e&#46;g&#46; an <a id="data&#45;model&#58;the&#45;iframe&#45;element&#45;3"></a><code>[iframe](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element)</code>&#41; is a <a id="data&#45;model&#58;focusable&#45;area&#45;10"></a>[focusable area](#focusable-area)&#44; but key events routed to a <a id="data&#45;model&#58;navigable&#45;container&#45;2"></a>[navigable container](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container) get immediately routed to its <a id="data&#45;model&#58;content&#45;navigable"></a>[content navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable)&#39;s <a id="data&#45;model&#58;nav&#45;document&#45;2"></a>[active document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document)&#46; Similarly&#44; in sequential focus navigation a <a id="data&#45;model&#58;navigable&#45;container&#45;3"></a>[navigable container](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container) essentially acts merely as a placeholder for its <a id="data&#45;model&#58;content&#45;navigable&#45;2"></a>[content navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable)&#39;s <a id="data&#45;model&#58;nav&#45;document&#45;3"></a>[active document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document)&#46;

------------------------------------------------------------------------

One <a id="data&#45;model&#58;focusable&#45;area&#45;11"></a>[focusable area](#focusable-area) in each <a id="data&#45;model&#58;document&#45;4"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> is designated the <a id="focused&#45;area&#45;of&#45;the&#45;document"></a><strong>focused area of the document</strong>&#46; Which control is so designated changes over time&#44; based on algorithms in this specification&#46;

<strong>Note&#58;</strong>

Even if a document is not <a id="data&#45;model&#58;fully&#45;active&#45;2"></a>[fully active](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fully-active) and not shown to the user&#44; it can still have a <a id="data&#45;model&#58;focused&#45;area&#45;of&#45;the&#45;document"></a>[focused area of the document](#focused-area-of-the-document)&#46; If a document&#39;s <a id="data&#45;model&#58;fully&#45;active&#45;3"></a>[fully active](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fully-active) state changes&#44; its <a id="data&#45;model&#58;focused&#45;area&#45;of&#45;the&#45;document&#45;2"></a>[focused area of the document](#focused-area-of-the-document) will stay the same&#46;

<a id="currently&#45;focused&#45;area&#45;of&#45;a&#45;top&#45;level&#45;browsing&#45;context"></a>The <a id="currently&#45;focused&#45;area&#45;of&#45;a&#45;top&#45;level&#45;traversable"></a><strong>currently focused area of a top&#45;level traversable</strong> <var>traversable</var> is the <a id="data&#45;model&#58;focusable&#45;area&#45;12"></a>[focusable area](#focusable-area)&#45;or&#45;null returned by this algorithm&#58;

1.  If <var>traversable</var> does not have <a id="data&#45;model&#58;system&#45;focus&#45;2"></a>[system focus](#system-focus)&#44; then return null&#46;

2.  Let <var>candidate</var> be <var>traversable</var>&#39;s <a id="data&#45;model&#58;nav&#45;document&#45;4"></a>[active document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document)&#46;

3.  While <var>candidate</var>&#39;s <a id="data&#45;model&#58;focused&#45;area&#45;of&#45;the&#45;document&#45;3"></a>[focused area](#focused-area-of-the-document) is a <a id="data&#45;model&#58;navigable&#45;container&#45;4"></a>[navigable container](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container) with a non&#45;null <a id="data&#45;model&#58;content&#45;navigable&#45;3"></a>[content navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable)&#58; set <var>candidate</var> to the <a id="data&#45;model&#58;nav&#45;document&#45;5"></a>[active document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document) of that <a id="data&#45;model&#58;navigable&#45;container&#45;5"></a>[navigable container](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container)&#39;s <a id="data&#45;model&#58;content&#45;navigable&#45;4"></a>[content navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable)&#46;

4.  If <var>candidate</var>&#39;s <a id="data&#45;model&#58;focused&#45;area&#45;of&#45;the&#45;document&#45;4"></a>[focused area](#focused-area-of-the-document) is non&#45;null&#44; set <var>candidate</var> to <var>candidate</var>&#39;s <a id="data&#45;model&#58;focused&#45;area&#45;of&#45;the&#45;document&#45;5"></a>[focused area](#focused-area-of-the-document)&#46;

5.  Return <var>candidate</var>&#46;

<a id="current&#45;focus&#45;chain&#45;of&#45;a&#45;top&#45;level&#45;browsing&#45;context"></a>The <a id="current&#45;focus&#45;chain&#45;of&#45;a&#45;top&#45;level&#45;traversable"></a><strong>current focus chain of a top&#45;level traversable</strong> <var>traversable</var> is the <a id="data&#45;model&#58;focus&#45;chain"></a>[focus chain](#focus-chain) of the <a id="data&#45;model&#58;currently&#45;focused&#45;area&#45;of&#45;a&#45;top&#45;level&#45;traversable"></a>[currently focused area](#currently-focused-area-of-a-top-level-traversable) of <var>traversable</var>&#44; if <var>traversable</var> is non&#45;null&#44; or an empty list otherwise&#46;

An element that is the <a id="data&#45;model&#58;dom&#45;anchor&#45;7"></a>[DOM anchor](#dom-anchor) of a <a id="data&#45;model&#58;focusable&#45;area&#45;13"></a>[focusable area](#focusable-area) is said to <a id="gains&#45;focus"></a><strong>gain focus</strong> when that <a id="data&#45;model&#58;focusable&#45;area&#45;14"></a>[focusable area](#focusable-area) becomes the <a id="data&#45;model&#58;currently&#45;focused&#45;area&#45;of&#45;a&#45;top&#45;level&#45;traversable&#45;2"></a>[currently focused area of a top&#45;level traversable](#currently-focused-area-of-a-top-level-traversable)&#46; When an element is the <a id="data&#45;model&#58;dom&#45;anchor&#45;8"></a>[DOM anchor](#dom-anchor) of a <a id="data&#45;model&#58;focusable&#45;area&#45;15"></a>[focusable area](#focusable-area) of the <a id="data&#45;model&#58;currently&#45;focused&#45;area&#45;of&#45;a&#45;top&#45;level&#45;traversable&#45;3"></a>[currently focused area of a top&#45;level traversable](#currently-focused-area-of-a-top-level-traversable)&#44; it is <a id="focused"></a><strong>focused</strong>&#46;

The <a id="focus&#45;chain"></a><strong>focus chain</strong> of a <a id="data&#45;model&#58;focusable&#45;area&#45;16"></a>[focusable area](#focusable-area) <var>subject</var> is the ordered list constructed as follows&#58;

1.  Let <var>output</var> be an empty <a id="data&#45;model&#58;list"></a>[list](https://infra.spec.whatwg.org/#list)&#46;

2.  Let <var>currentObject</var> be <var>subject</var>&#46;

3.  While true&#58;

    1.  <a id="data&#45;model&#58;list&#45;append"></a>[Append](https://infra.spec.whatwg.org/#list-append) <var>currentObject</var> to <var>output</var>&#46;

    2.  If <var>currentObject</var> is an <a id="data&#45;model&#58;the&#45;area&#45;element&#45;3"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> element&#39;s shape&#44; then <a id="data&#45;model&#58;list&#45;append&#45;2"></a>[append](https://infra.spec.whatwg.org/#list-append) that <a id="data&#45;model&#58;the&#45;area&#45;element&#45;4"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> element to <var>output</var>&#46;

        Otherwise&#44; if <var>currentObject</var>&#39;s <a id="data&#45;model&#58;dom&#45;anchor&#45;9"></a>[DOM anchor](#dom-anchor) is an element that is not <var>currentObject</var> itself&#44; then <a id="data&#45;model&#58;list&#45;append&#45;3"></a>[append](https://infra.spec.whatwg.org/#list-append) <var>currentObject</var>&#39;s <a id="data&#45;model&#58;dom&#45;anchor&#45;10"></a>[DOM anchor](#dom-anchor) to <var>output</var>&#46;

    3.  If <var>currentObject</var> is a <a id="data&#45;model&#58;focusable&#45;area&#45;17"></a>[focusable area](#focusable-area)&#44; then set <var>currentObject</var> to <var>currentObject</var>&#39;s <a id="data&#45;model&#58;dom&#45;anchor&#45;11"></a>[DOM anchor](#dom-anchor)&#39;s <a id="data&#45;model&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46;

        Otherwise&#44; if <var>currentObject</var> is a <a id="data&#45;model&#58;document&#45;5"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> whose <a id="data&#45;model&#58;node&#45;navigable&#45;2"></a>[node navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable)&#39;s <a id="data&#45;model&#58;nav&#45;parent"></a>[parent](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-parent) is non&#45;null&#44; then set <var>currentObject</var> to <var>currentObject</var>&#39;s <a id="data&#45;model&#58;node&#45;navigable&#45;3"></a>[node navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable)&#39;s <a id="data&#45;model&#58;nav&#45;parent&#45;2"></a>[parent](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-parent)&#46;

        Otherwise&#44; <a id="data&#45;model&#58;break"></a>[break](https://infra.spec.whatwg.org/#iteration-break)&#46;

4.  Return <var>output</var>&#46;

    <strong>Note&#58;</strong>

    The chain starts with <var>subject</var> and &#40;if <var>subject</var> is or can be the <a id="data&#45;model&#58;currently&#45;focused&#45;area&#45;of&#45;a&#45;top&#45;level&#45;traversable&#45;4"></a>[currently focused area of a top&#45;level traversable](#currently-focused-area-of-a-top-level-traversable)&#41; continues up the focus hierarchy up to the <a id="data&#45;model&#58;document&#45;6"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> of the <a id="data&#45;model&#58;top&#45;level&#45;traversable&#45;3"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable)&#46;

All elements that are <a id="data&#45;model&#58;focusable&#45;area&#45;18"></a>[focusable areas](#focusable-area) are said to be <a id="focusable"></a><strong>focusable</strong>&#46;

There are two special types of focusability for <a id="data&#45;model&#58;focusable&#45;area&#45;19"></a>[focusable areas](#focusable-area)&#58;

- A <a id="data&#45;model&#58;focusable&#45;area&#45;20"></a>[focusable area](#focusable-area) is said to be <a id="sequentially&#45;focusable"></a><strong>sequentially focusable</strong> if it is included in its <a id="data&#45;model&#58;document&#45;7"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#39;s <a id="data&#45;model&#58;sequential&#45;focus&#45;navigation&#45;order"></a>[sequential focus navigation order](#sequential-focus-navigation-order) and the user agent determines that it is sequentially focusable&#46;

- A <a id="data&#45;model&#58;focusable&#45;area&#45;21"></a>[focusable area](#focusable-area) is said to be <a id="click&#45;focusable"></a><strong>click focusable</strong> if the user agent determines that it is click focusable&#46; User agents should consider focusable areas with non&#45;null <a id="data&#45;model&#58;tabindex&#45;value&#45;2"></a>[tabindex values](#tabindex-value) to be click focusable&#46;

<strong>Note&#58;</strong>

Elements which are not <a id="data&#45;model&#58;focusable"></a>[focusable](#focusable) are not <a id="data&#45;model&#58;focusable&#45;area&#45;22"></a>[focusable areas](#focusable-area)&#44; and thus not <a id="data&#45;model&#58;sequentially&#45;focusable&#45;3"></a>[sequentially focusable](#sequentially-focusable) and not <a id="data&#45;model&#58;click&#45;focusable"></a>[click focusable](#click-focusable)&#46;

<strong>Note&#58;</strong>

Being <a id="data&#45;model&#58;focusable&#45;2"></a>[focusable](#focusable) is a statement about whether an element can be focused programmatically&#44; e&#46;g&#46; via the <a id="data&#45;model&#58;dom&#45;focus"></a><code>[focus&#40;&#41;](#dom-focus)</code> method or <a id="data&#45;model&#58;attr&#45;fe&#45;autofocus"></a><code>[autofocus](#attr-fe-autofocus)</code> attribute&#46; In contrast&#44; <a id="data&#45;model&#58;sequentially&#45;focusable&#45;4"></a>[sequentially focusable](#sequentially-focusable) and <a id="data&#45;model&#58;click&#45;focusable&#45;2"></a>[click focusable](#click-focusable) govern how the user agent responds to user interaction&#58; respectively&#44; to <a id="data&#45;model&#58;sequential&#45;focus&#45;navigation"></a>[sequential focus navigation](#sequential-focus-navigation) and as <a id="data&#45;model&#58;activation&#45;behaviour"></a>[activation behavior](https://dom.spec.whatwg.org/#eventtarget-activation-behavior)&#46;

The user agent might determine that an element is not <a id="data&#45;model&#58;sequentially&#45;focusable&#45;5"></a>[sequentially focusable](#sequentially-focusable) even if it is <a id="data&#45;model&#58;focusable&#45;3"></a>[focusable](#focusable) and is included in its <a id="data&#45;model&#58;document&#45;8"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#39;s <a id="data&#45;model&#58;sequential&#45;focus&#45;navigation&#45;order&#45;2"></a>[sequential focus navigation order](#sequential-focus-navigation-order)&#44; according to user preferences&#46; For example&#44; macOS users can set the user agent to skip non&#45;form control elements&#44; or can skip links when doing <a id="data&#45;model&#58;sequential&#45;focus&#45;navigation&#45;2"></a>[sequential focus navigation](#sequential-focus-navigation) with just the Tab key &#40;as opposed to using both the Option and Tab keys&#41;&#46;

Similarly&#44; the user agent might determine that an element is not <a id="data&#45;model&#58;click&#45;focusable&#45;3"></a>[click focusable](#click-focusable) even if it is <a id="data&#45;model&#58;focusable&#45;4"></a>[focusable](#focusable)&#46; For example&#44; in some user agents&#44; clicking on a non&#45;editable form control does not focus it&#44; i&#46;e&#46; the user agent has determined that such controls are not click focusable&#46;

Thus&#44; an element can be <a id="data&#45;model&#58;focusable&#45;5"></a>[focusable](#focusable)&#44; but neither <a id="data&#45;model&#58;sequentially&#45;focusable&#45;6"></a>[sequentially focusable](#sequentially-focusable) nor <a id="data&#45;model&#58;click&#45;focusable&#45;4"></a>[click focusable](#click-focusable)&#46; For example&#44; in some user agents&#44; a non&#45;editable form&#45;control with a negative&#45;integer <a id="data&#45;model&#58;tabindex&#45;value&#45;3"></a>[tabindex value](#tabindex-value) would not be focusable via user interaction&#44; only via programmatic APIs&#46;

When a user [activates](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#activation) a <a id="data&#45;model&#58;click&#45;focusable&#45;5"></a>[click focusable](#click-focusable) <a id="data&#45;model&#58;focusable&#45;area&#45;23"></a>[focusable area](#focusable-area)&#44; the user agent must run the <a id="data&#45;model&#58;focusing&#45;steps"></a>[focusing steps](#focusing-steps) on the <a id="data&#45;model&#58;focusable&#45;area&#45;24"></a>[focusable area](#focusable-area) with <em>focus trigger</em> set to &#34;<code>click</code>&#34;&#46;

<strong>Note&#58;</strong>

Note that focusing is not an <a id="data&#45;model&#58;activation&#45;behaviour&#45;2"></a>[activation behavior](https://dom.spec.whatwg.org/#eventtarget-activation-behavior)&#44; i&#46;e&#46; calling the <a id="data&#45;model&#58;dom&#45;click"></a><code>[click&#40;&#41;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-click)</code> method on an element or dispatching a synthetic <a id="data&#45;model&#58;event&#45;click"></a><code>[click](https://w3c.github.io/pointerevents/#click)</code> event on it won&#39;t cause the element to get focused&#46;

------------------------------------------------------------------------

A node is a <a id="focus&#45;navigation&#45;scope&#45;owner"></a><strong>focus navigation scope owner</strong> if it is a <a id="data&#45;model&#58;document&#45;9"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#44; a <a id="data&#45;model&#58;shadow&#45;host&#45;2"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host)&#44; a <a id="data&#45;model&#58;the&#45;slot&#45;element"></a>[slot](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-slot-element)&#44; or an element which is the <a id="data&#45;model&#58;popover&#45;trigger"></a>[popover trigger](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#popover-trigger) of an element in the <a id="data&#45;model&#58;popover&#45;showing&#45;state"></a>[popover showing state](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#popover-showing-state)&#46;

Each <a id="data&#45;model&#58;focus&#45;navigation&#45;scope&#45;owner"></a>[focus navigation scope owner](#focus-navigation-scope-owner) has a <a id="focus&#45;navigation&#45;scope"></a><strong>focus navigation scope</strong>&#44; which is a list of elements&#46; Its contents are determined as follows&#58;

Every element <var>element</var> has an <a id="associated&#45;focus&#45;navigation&#45;owner"></a><strong>associated focus navigation owner</strong>&#44; which is either null or a <a id="data&#45;model&#58;focus&#45;navigation&#45;scope&#45;owner&#45;2"></a>[focus navigation scope owner](#focus-navigation-scope-owner)&#46; It is determined by the following algorithm&#58;

1.  If <var>element</var>&#39;s parent is null&#44; then return null&#46;

2.  If <var>element</var>&#39;s parent is a <a id="data&#45;model&#58;shadow&#45;host&#45;3"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host)&#44; then return <var>element</var>&#39;s <a id="data&#45;model&#58;assigned&#45;slot"></a>[assigned slot](https://dom.spec.whatwg.org/#slotable-assigned-slot)&#46;

3.  If <var>element</var>&#39;s parent is a <a id="data&#45;model&#58;shadow&#45;root"></a>[shadow root](https://dom.spec.whatwg.org/#concept-shadow-root)&#44; then return the parent&#39;s <a id="data&#45;model&#58;concept&#45;documentfragment&#45;host"></a>[host](https://dom.spec.whatwg.org/#concept-documentfragment-host)&#46;

4.  If <var>element</var>&#39;s parent is the <a id="data&#45;model&#58;document&#45;element"></a>[document element](https://dom.spec.whatwg.org/#document-element)&#44; then return the parent&#39;s <a id="data&#45;model&#58;node&#45;document&#45;2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46;

5.  If <var>element</var> is in the <a id="data&#45;model&#58;popover&#45;showing&#45;state&#45;2"></a>[popover showing state](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#popover-showing-state) and has a <a id="data&#45;model&#58;popover&#45;trigger&#45;2"></a>[popover trigger](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#popover-trigger) set&#44; then return <var>element</var>&#39;s <a id="data&#45;model&#58;popover&#45;trigger&#45;3"></a>[popover trigger](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#popover-trigger)&#46;

6.  Return <var>element</var>&#39;s parent&#39;s <a id="data&#45;model&#58;associated&#45;focus&#45;navigation&#45;owner"></a>[associated focus navigation owner](#associated-focus-navigation-owner)&#46;

Then&#44; the contents of a given <a id="data&#45;model&#58;focus&#45;navigation&#45;scope&#45;owner&#45;3"></a>[focus navigation scope owner](#focus-navigation-scope-owner) <var>owner</var>&#39;s <a id="data&#45;model&#58;focus&#45;navigation&#45;scope"></a>[focus navigation scope](#focus-navigation-scope) are all elements whose <a id="data&#45;model&#58;associated&#45;focus&#45;navigation&#45;owner&#45;2"></a>[associated focus navigation owner](#associated-focus-navigation-owner) is <var>owner</var>&#46;

<strong>Note&#58;</strong>

The order of elements within a <a id="data&#45;model&#58;focus&#45;navigation&#45;scope&#45;2"></a>[focus navigation scope](#focus-navigation-scope) does not impact any of the algorithms in this specification&#46; Ordering only becomes important for the <a id="data&#45;model&#58;tabindex&#45;ordered&#45;focus&#45;navigation&#45;scope"></a>[tabindex&#45;ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope) and <a id="data&#45;model&#58;flattened&#45;tabindex&#45;ordered&#45;focus&#45;navigation&#45;scope"></a>[flattened tabindex&#45;ordered focus navigation scope](#flattened-tabindex-ordered-focus-navigation-scope) concepts defined below&#46;

A <a id="tabindex&#45;ordered&#45;focus&#45;navigation&#45;scope"></a><strong>tabindex&#45;ordered focus navigation scope</strong> is a list of <a id="data&#45;model&#58;focusable&#45;area&#45;25"></a>[focusable areas](#focusable-area) and <a id="data&#45;model&#58;focus&#45;navigation&#45;scope&#45;owner&#45;4"></a>[focus navigation scope owners](#focus-navigation-scope-owner)&#46; Every <a id="data&#45;model&#58;focus&#45;navigation&#45;scope&#45;owner&#45;5"></a>[focus navigation scope owner](#focus-navigation-scope-owner) <var>owner</var> has <a id="data&#45;model&#58;tabindex&#45;ordered&#45;focus&#45;navigation&#45;scope&#45;2"></a>[tabindex&#45;ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope)&#44; whose contents are determined as follows&#58;

- It contains all elements in <var>owner</var>&#39;s <a id="data&#45;model&#58;focus&#45;navigation&#45;scope&#45;3"></a>[focus navigation scope](#focus-navigation-scope) that are themselves <a id="data&#45;model&#58;focus&#45;navigation&#45;scope&#45;owner&#45;6"></a>[focus navigation scope owners](#focus-navigation-scope-owner)&#44; except the elements whose <a id="data&#45;model&#58;tabindex&#45;value&#45;4"></a>[tabindex value](#tabindex-value) is a negative integer&#46;

- It contains all of the <a id="data&#45;model&#58;focusable&#45;area&#45;26"></a>[focusable areas](#focusable-area) whose <a id="data&#45;model&#58;dom&#45;anchor&#45;12"></a>[DOM anchor](#dom-anchor) is an element in <var>owner</var>&#39;s <a id="data&#45;model&#58;focus&#45;navigation&#45;scope&#45;4"></a>[focus navigation scope](#focus-navigation-scope)&#44; except the <a id="data&#45;model&#58;focusable&#45;area&#45;27"></a>[focusable areas](#focusable-area) whose <a id="data&#45;model&#58;tabindex&#45;value&#45;5"></a>[tabindex value](#tabindex-value) is a negative integer&#46;

The order within a <a id="data&#45;model&#58;tabindex&#45;ordered&#45;focus&#45;navigation&#45;scope&#45;3"></a>[tabindex&#45;ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope) is determined by each element&#39;s <a id="data&#45;model&#58;tabindex&#45;value&#45;6"></a>[tabindex value](#tabindex-value)&#44; as described in the section below&#46;

<strong>Note&#58;</strong>

The rules there do not give a precise ordering&#44; as they are composed mostly of &#34;should&#34; statements and relative orderings&#46;

A <a id="flattened&#45;tabindex&#45;ordered&#45;focus&#45;navigation&#45;scope"></a><strong>flattened tabindex&#45;ordered focus navigation scope</strong> is a list of <a id="data&#45;model&#58;focusable&#45;area&#45;28"></a>[focusable areas](#focusable-area)&#46; Every <a id="data&#45;model&#58;focus&#45;navigation&#45;scope&#45;owner&#45;7"></a>[focus navigation scope owner](#focus-navigation-scope-owner) <var>owner</var> owns a distinct <a id="data&#45;model&#58;flattened&#45;tabindex&#45;ordered&#45;focus&#45;navigation&#45;scope&#45;2"></a>[flattened tabindex&#45;ordered focus navigation scope](#flattened-tabindex-ordered-focus-navigation-scope)&#44; whose contents are determined by the following algorithm&#58;

1.  Let <var>result</var> be a <a id="data&#45;model&#58;list&#45;clone"></a>[clone](https://infra.spec.whatwg.org/#list-clone) of <var>owner</var>&#39;s <a id="data&#45;model&#58;tabindex&#45;ordered&#45;focus&#45;navigation&#45;scope&#45;4"></a>[tabindex&#45;ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope)&#46;

2.  For each <var>item</var> of <var>result</var>&#58;

    1.  If <var>item</var> is not a <a id="data&#45;model&#58;focus&#45;navigation&#45;scope&#45;owner&#45;8"></a>[focus navigation scope owner](#focus-navigation-scope-owner)&#44; then <a id="data&#45;model&#58;continue"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue)&#46;

    2.  If <var>item</var> is not a <a id="data&#45;model&#58;focusable&#45;area&#45;29"></a>[focusable area](#focusable-area)&#44; then replace <var>item</var> with all of the items in <var>item</var>&#39;s <a id="data&#45;model&#58;flattened&#45;tabindex&#45;ordered&#45;focus&#45;navigation&#45;scope&#45;3"></a>[flattened tabindex&#45;ordered focus navigation scope](#flattened-tabindex-ordered-focus-navigation-scope)&#46;

    3.  Otherwise&#44; insert the contents of <var>item</var>&#39;s <a id="data&#45;model&#58;flattened&#45;tabindex&#45;ordered&#45;focus&#45;navigation&#45;scope&#45;4"></a>[flattened tabindex&#45;ordered focus navigation scope](#flattened-tabindex-ordered-focus-navigation-scope) after <var>item</var>&#46;

#### <a id="the&#45;tabindex&#45;attribute"></a>6&#46;6&#46;3 The <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;tabindex"></a><code>[tabindex](#attr-tabindex)</code> attribute[](#the-tabindex-attribute)

<strong>✔</strong>MDN

[Global&#95;attributes&#47;tabindex](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/tabindex)

Support in all current engines&#46;

Firefox1&#46;5&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

The <a id="attr&#45;tabindex"></a><strong><code>tabindex</code></strong> content attribute allows authors to make an element and regions that have the element as its <a id="the&#45;tabindex&#45;attribute&#58;dom&#45;anchor"></a>[DOM anchor](#dom-anchor) be <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area"></a>[focusable areas](#focusable-area)&#44; allow or prevent them from being <a id="the&#45;tabindex&#45;attribute&#58;sequentially&#45;focusable"></a>[sequentially focusable](#sequentially-focusable)&#44; and determine their relative ordering for <a id="the&#45;tabindex&#45;attribute&#58;sequential&#45;focus&#45;navigation"></a>[sequential focus navigation](#sequential-focus-navigation)&#46;

The name &#34;tab index&#34; comes from the common use of the Tab key to navigate through the focusable elements&#46; The term &#34;tabbing&#34; refers to moving forward through <a id="the&#45;tabindex&#45;attribute&#58;sequentially&#45;focusable&#45;2"></a>[sequentially focusable](#sequentially-focusable) <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;2"></a>[focusable areas](#focusable-area)&#46;

The <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;tabindex&#45;2"></a><code>[tabindex](#attr-tabindex)</code> attribute&#44; if specified&#44; must have a value that is a <a id="the&#45;tabindex&#45;attribute&#58;valid&#45;integer"></a>[valid integer](html-language-foundations--877d6146487f--b55e73c31978.md#valid-integer)&#46; Positive numbers specify the relative position of the element&#39;s <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;3"></a>[focusable areas](#focusable-area) in the <a id="the&#45;tabindex&#45;attribute&#58;sequential&#45;focus&#45;navigation&#45;order"></a>[sequential focus navigation order](#sequential-focus-navigation-order)&#44; and negative numbers indicate that the control is not <a id="the&#45;tabindex&#45;attribute&#58;sequentially&#45;focusable&#45;3"></a>[sequentially focusable](#sequentially-focusable)&#46;

Developers should use caution when using values other than 0 or −1 for their <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;tabindex&#45;3"></a><code>[tabindex](#attr-tabindex)</code> attributes as this is complicated to do correctly&#46;

<strong>Note&#58;</strong>

The following provides a non&#45;normative summary of the behaviors of the possible <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;tabindex&#45;4"></a><code>[tabindex](#attr-tabindex)</code> attribute values&#46; The below processing model gives the more precise rules&#46;

<strong>omitted &#40;or non&#45;integer values&#41;</strong>

The user agent will decide whether the element is <a id="the&#45;tabindex&#45;attribute&#58;focusable"></a>[focusable](#focusable)&#44; and if it is&#44; whether it is <a id="the&#45;tabindex&#45;attribute&#58;sequentially&#45;focusable&#45;4"></a>[sequentially focusable](#sequentially-focusable) or <a id="the&#45;tabindex&#45;attribute&#58;click&#45;focusable"></a>[click focusable](#click-focusable) &#40;or both&#41;&#46;

<strong>−1 &#40;or other negative integer values&#41;</strong>

Causes the element to be <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;2"></a>[focusable](#focusable)&#44; and indicates that the author would prefer the element to be <a id="the&#45;tabindex&#45;attribute&#58;click&#45;focusable&#45;2"></a>[click focusable](#click-focusable) but not <a id="the&#45;tabindex&#45;attribute&#58;sequentially&#45;focusable&#45;5"></a>[sequentially focusable](#sequentially-focusable)&#46; The user agent might ignore this preference for click and sequential focusability&#44; e&#46;g&#46;&#44; for specific element types according to platform conventions&#44; or for keyboard&#45;only users&#46;

<strong>0</strong>

Causes the element to be <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;3"></a>[focusable](#focusable)&#44; and indicates that the author would prefer the element to be both <a id="the&#45;tabindex&#45;attribute&#58;click&#45;focusable&#45;3"></a>[click focusable](#click-focusable) and <a id="the&#45;tabindex&#45;attribute&#58;sequentially&#45;focusable&#45;6"></a>[sequentially focusable](#sequentially-focusable)&#46; The user agent might ignore this preference for click and sequential focusability&#46;

<strong>positive integer values</strong>

Behaves the same as 0&#44; but in addition creates a relative ordering within a <a id="the&#45;tabindex&#45;attribute&#58;tabindex&#45;ordered&#45;focus&#45;navigation&#45;scope"></a>[tabindex&#45;ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope)&#44; so that elements with higher <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;tabindex&#45;5"></a><code>[tabindex](#attr-tabindex)</code> attribute value come later&#46;

Note that the <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;tabindex&#45;6"></a><code>[tabindex](#attr-tabindex)</code> attribute cannot be used to make an element non&#45;focusable&#46; The only way a page author can do that is by <a id="the&#45;tabindex&#45;attribute&#58;concept&#45;element&#45;disabled"></a>[disabling](#concept-element-disabled) the element&#44; or making it <a id="the&#45;tabindex&#45;attribute&#58;inert"></a>[inert](#inert)&#46;

------------------------------------------------------------------------

The <a id="tabindex&#45;value"></a><strong>tabindex value</strong> of an element is the value of its <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;tabindex&#45;7"></a><code>[tabindex](#attr-tabindex)</code> attribute&#44; parsed using the <a id="the&#45;tabindex&#45;attribute&#58;rules&#45;for&#45;parsing&#45;integers"></a>[rules for parsing integers](html-language-foundations--877d6146487f--b55e73c31978.md#rules-for-parsing-integers)&#46; If parsing fails or the attribute is not specified&#44; then the <a id="the&#45;tabindex&#45;attribute&#58;tabindex&#45;value"></a>[tabindex value](#tabindex-value) is null&#46;

The <a id="the&#45;tabindex&#45;attribute&#58;tabindex&#45;value&#45;2"></a>[tabindex value](#tabindex-value) of a <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;4"></a>[focusable area](#focusable-area) is the <a id="the&#45;tabindex&#45;attribute&#58;tabindex&#45;value&#45;3"></a>[tabindex value](#tabindex-value) of its <a id="the&#45;tabindex&#45;attribute&#58;dom&#45;anchor&#45;2"></a>[DOM anchor](#dom-anchor)&#46;

The <a id="the&#45;tabindex&#45;attribute&#58;tabindex&#45;value&#45;4"></a>[tabindex value](#tabindex-value) of an element must be interpreted as follows&#58;

<strong>If the value is null</strong>

The user agent should follow platform conventions to determine if the element should be considered as a <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;5"></a>[focusable area](#focusable-area) and if so&#44; whether the element and any <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;6"></a>[focusable areas](#focusable-area) that have the element as their <a id="the&#45;tabindex&#45;attribute&#58;dom&#45;anchor&#45;3"></a>[DOM anchor](#dom-anchor) are <a id="the&#45;tabindex&#45;attribute&#58;sequentially&#45;focusable&#45;7"></a>[sequentially focusable](#sequentially-focusable)&#44; and if so&#44; what their relative position in their <a id="the&#45;tabindex&#45;attribute&#58;tabindex&#45;ordered&#45;focus&#45;navigation&#45;scope&#45;2"></a>[tabindex&#45;ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope) is to be&#46; If the element is a <a id="the&#45;tabindex&#45;attribute&#58;focus&#45;navigation&#45;scope&#45;owner"></a>[focus navigation scope owner](#focus-navigation-scope-owner)&#44; it must be included in its <a id="the&#45;tabindex&#45;attribute&#58;tabindex&#45;ordered&#45;focus&#45;navigation&#45;scope&#45;3"></a>[tabindex&#45;ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope) even if it is not a <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;7"></a>[focusable area](#focusable-area)&#46;

The relative ordering within a <a id="the&#45;tabindex&#45;attribute&#58;tabindex&#45;ordered&#45;focus&#45;navigation&#45;scope&#45;4"></a>[tabindex&#45;ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope) for elements and <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;8"></a>[focusable areas](#focusable-area) that belong to the same <a id="the&#45;tabindex&#45;attribute&#58;focus&#45;navigation&#45;scope"></a>[focus navigation scope](#focus-navigation-scope) and whose <a id="the&#45;tabindex&#45;attribute&#58;tabindex&#45;value&#45;5"></a>[tabindex value](#tabindex-value) is null should be in <a id="the&#45;tabindex&#45;attribute&#58;shadow&#45;including&#45;tree&#45;order"></a>[shadow&#45;including tree order](https://dom.spec.whatwg.org/#concept-shadow-including-tree-order)&#46;

Modulo platform conventions&#44; it is suggested that the following elements should be considered as <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;9"></a>[focusable areas](#focusable-area) and be <a id="the&#45;tabindex&#45;attribute&#58;sequentially&#45;focusable&#45;8"></a>[sequentially focusable](#sequentially-focusable)&#58;

- <a id="the&#45;tabindex&#45;attribute&#58;the&#45;a&#45;element"></a><code>[a](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element)</code> elements that have an <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;hyperlink&#45;href"></a><code>[href](html-language-elements-text-links--877d6146487f--8833a85fd242.md#attr-hyperlink-href)</code> attribute
- <a id="the&#45;tabindex&#45;attribute&#58;the&#45;button&#45;element"></a><code>[button](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-button-element)</code> elements
- <a id="the&#45;tabindex&#45;attribute&#58;the&#45;input&#45;element"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements whose <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;input&#45;type"></a><code>[type](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-type)</code> attribute are not in the <a id="the&#45;tabindex&#45;attribute&#58;hidden&#45;state&#45;&#40;type&#61;hidden&#41;"></a>[Hidden](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#hidden-state-(type=hidden)) state
- <a id="the&#45;tabindex&#45;attribute&#58;the&#45;select&#45;element"></a><code>[select](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-select-element)</code> elements
- <a id="the&#45;tabindex&#45;attribute&#58;the&#45;textarea&#45;element"></a><code>[textarea](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-textarea-element)</code> elements
- <a id="the&#45;tabindex&#45;attribute&#58;the&#45;summary&#45;element"></a><code>[summary](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-summary-element)</code> elements that are the first <a id="the&#45;tabindex&#45;attribute&#58;the&#45;summary&#45;element&#45;2"></a><code>[summary](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-summary-element)</code> element child of a <a id="the&#45;tabindex&#45;attribute&#58;the&#45;details&#45;element"></a><code>[details](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-details-element)</code> element
- Elements with a <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;draggable"></a><code>[draggable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-draggable)</code> attribute set&#44; if that would enable the user agent to allow the user to begin drag operations for those elements without the use of a pointing device
- <a id="the&#45;tabindex&#45;attribute&#58;editing&#45;host"></a>[Editing hosts](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#editing-host)
- <a id="the&#45;tabindex&#45;attribute&#58;navigable&#45;container"></a>[Navigable containers](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container)

<a id="negative&#45;tabindex"></a><strong>If the value is a negative integer</strong>

The user agent must consider the element as a <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;10"></a>[focusable area](#focusable-area)&#44; but should omit the element from any <a id="the&#45;tabindex&#45;attribute&#58;tabindex&#45;ordered&#45;focus&#45;navigation&#45;scope&#45;5"></a>[tabindex&#45;ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope)&#46;

<strong>Note&#58;</strong>

One valid reason to ignore the requirement that sequential focus navigation not allow the author to lead to the element would be if the user&#39;s only mechanism for moving the focus is sequential focus navigation&#46; For instance&#44; a keyboard&#45;only user would be unable to click on a text control with a negative <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;tabindex&#45;8"></a><code>[tabindex](#attr-tabindex)</code>&#44; so that user&#39;s user agent would be well justified in allowing the user to tab to the control regardless&#46;

<strong>If the value is a zero</strong>

The user agent must allow the element to be considered as a <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;11"></a>[focusable area](#focusable-area) and should allow the element and any <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;12"></a>[focusable areas](#focusable-area) that have the element as their <a id="the&#45;tabindex&#45;attribute&#58;dom&#45;anchor&#45;4"></a>[DOM anchor](#dom-anchor) to be <a id="the&#45;tabindex&#45;attribute&#58;sequentially&#45;focusable&#45;9"></a>[sequentially focusable](#sequentially-focusable)&#46;

The relative ordering within a <a id="the&#45;tabindex&#45;attribute&#58;tabindex&#45;ordered&#45;focus&#45;navigation&#45;scope&#45;6"></a>[tabindex&#45;ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope) for elements and <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;13"></a>[focusable areas](#focusable-area) that belong to the same <a id="the&#45;tabindex&#45;attribute&#58;focus&#45;navigation&#45;scope&#45;2"></a>[focus navigation scope](#focus-navigation-scope) and whose <a id="the&#45;tabindex&#45;attribute&#58;tabindex&#45;value&#45;6"></a>[tabindex value](#tabindex-value) is zero should be in <a id="the&#45;tabindex&#45;attribute&#58;shadow&#45;including&#45;tree&#45;order&#45;2"></a>[shadow&#45;including tree order](https://dom.spec.whatwg.org/#concept-shadow-including-tree-order)&#46;

<strong>If the value is greater than zero</strong>

The user agent must allow the element to be considered as a <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;14"></a>[focusable area](#focusable-area) and should allow the element and any <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;15"></a>[focusable areas](#focusable-area) that have the element as their <a id="the&#45;tabindex&#45;attribute&#58;dom&#45;anchor&#45;5"></a>[DOM anchor](#dom-anchor) to be <a id="the&#45;tabindex&#45;attribute&#58;sequentially&#45;focusable&#45;10"></a>[sequentially focusable](#sequentially-focusable)&#44; and should place the element — referenced as <var>candidate</var> below — and the aforementioned <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;16"></a>[focusable areas](#focusable-area) in the <a id="the&#45;tabindex&#45;attribute&#58;tabindex&#45;ordered&#45;focus&#45;navigation&#45;scope&#45;7"></a>[tabindex&#45;ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope) where the element is a part of so that&#44; relative to other elements and <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;17"></a>[focusable areas](#focusable-area) that belong to the same <a id="the&#45;tabindex&#45;attribute&#58;focus&#45;navigation&#45;scope&#45;3"></a>[focus navigation scope](#focus-navigation-scope)&#44; they are&#58;

- before any <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;18"></a>[focusable area](#focusable-area) whose <a id="the&#45;tabindex&#45;attribute&#58;dom&#45;anchor&#45;6"></a>[DOM anchor](#dom-anchor) is an element whose <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;tabindex&#45;9"></a><code>[tabindex](#attr-tabindex)</code> attribute has been omitted or whose value&#44; when parsed&#44; returns an error&#44;
- before any <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;19"></a>[focusable area](#focusable-area) whose <a id="the&#45;tabindex&#45;attribute&#58;dom&#45;anchor&#45;7"></a>[DOM anchor](#dom-anchor) is an element whose <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;tabindex&#45;10"></a><code>[tabindex](#attr-tabindex)</code> attribute has a value less than or equal to zero&#44;
- after any <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;20"></a>[focusable area](#focusable-area) whose <a id="the&#45;tabindex&#45;attribute&#58;dom&#45;anchor&#45;8"></a>[DOM anchor](#dom-anchor) is an element whose <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;tabindex&#45;11"></a><code>[tabindex](#attr-tabindex)</code> attribute has a value greater than zero but less than the value of the <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;tabindex&#45;12"></a><code>[tabindex](#attr-tabindex)</code> attribute on <var>candidate</var>&#44;
- after any <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;21"></a>[focusable area](#focusable-area) whose <a id="the&#45;tabindex&#45;attribute&#58;dom&#45;anchor&#45;9"></a>[DOM anchor](#dom-anchor) is an element whose <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;tabindex&#45;13"></a><code>[tabindex](#attr-tabindex)</code> attribute has a value equal to the value of the <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;tabindex&#45;14"></a><code>[tabindex](#attr-tabindex)</code> attribute on <var>candidate</var> but that is located earlier than <var>candidate</var> in <a id="the&#45;tabindex&#45;attribute&#58;shadow&#45;including&#45;tree&#45;order&#45;3"></a>[shadow&#45;including tree order](https://dom.spec.whatwg.org/#concept-shadow-including-tree-order)&#44;
- before any <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;22"></a>[focusable area](#focusable-area) whose <a id="the&#45;tabindex&#45;attribute&#58;dom&#45;anchor&#45;10"></a>[DOM anchor](#dom-anchor) is an element whose <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;tabindex&#45;15"></a><code>[tabindex](#attr-tabindex)</code> attribute has a value equal to the value of the <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;tabindex&#45;16"></a><code>[tabindex](#attr-tabindex)</code> attribute on <var>candidate</var> but that is located later than <var>candidate</var> in <a id="the&#45;tabindex&#45;attribute&#58;shadow&#45;including&#45;tree&#45;order&#45;4"></a>[shadow&#45;including tree order](https://dom.spec.whatwg.org/#concept-shadow-including-tree-order)&#44; and
- before any <a id="the&#45;tabindex&#45;attribute&#58;focusable&#45;area&#45;23"></a>[focusable area](#focusable-area) whose <a id="the&#45;tabindex&#45;attribute&#58;dom&#45;anchor&#45;11"></a>[DOM anchor](#dom-anchor) is an element whose <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;tabindex&#45;17"></a><code>[tabindex](#attr-tabindex)</code> attribute has a value greater than the value of the <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;tabindex&#45;18"></a><code>[tabindex](#attr-tabindex)</code> attribute on <var>candidate</var>&#46;

<strong>✔</strong>MDN

[HTMLElement&#47;tabIndex](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/tabIndex)

Support in all current engines&#46;

Firefox1&#43;Safari3&#46;1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;18Internet Explorer🔰 5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

The <a id="dom&#45;tabindex"></a><strong><code>tabIndex</code></strong> getter steps are&#58;

1.  Let <var>attribute</var> be <a id="the&#45;tabindex&#45;attribute&#58;this"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;tabindex&#45;attribute&#58;attr&#45;tabindex&#45;19"></a><code>[tabindex](#attr-tabindex)</code> attribute&#46;

2.  If <var>attribute</var> is not null&#58;

    1.  Let <var>parsedValue</var> be the result of <a id="the&#45;tabindex&#45;attribute&#58;rules&#45;for&#45;parsing&#45;integers&#45;2"></a>[integer parsing](html-language-foundations--877d6146487f--b55e73c31978.md#rules-for-parsing-integers) <var>attribute</var>&#39;s <a id="the&#45;tabindex&#45;attribute&#58;concept&#45;attribute&#45;value"></a>[value](https://dom.spec.whatwg.org/#concept-attribute-value)&#46;

    2.  If <var>parsedValue</var> is not an error and is within the <a id="the&#45;tabindex&#45;attribute&#58;idl&#45;long"></a><code>[long](https://webidl.spec.whatwg.org/#idl-long)</code> range&#44; then return <var>parsedValue</var>&#46;

3.  Return 0 if <a id="the&#45;tabindex&#45;attribute&#58;this&#45;2"></a>[this](https://webidl.spec.whatwg.org/#this) is an <a id="the&#45;tabindex&#45;attribute&#58;the&#45;a&#45;element&#45;2"></a><code>[a](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element)</code>&#44; <a id="the&#45;tabindex&#45;attribute&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code>&#44; <a id="the&#45;tabindex&#45;attribute&#58;the&#45;button&#45;element&#45;2"></a><code>[button](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-button-element)</code>&#44; <a id="the&#45;tabindex&#45;attribute&#58;frame"></a><code>[frame](html-language-obsolete--877d6146487f--66870e9cdabe.md#frame)</code>&#44; <a id="the&#45;tabindex&#45;attribute&#58;the&#45;iframe&#45;element"></a><code>[iframe](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element)</code>&#44; <a id="the&#45;tabindex&#45;attribute&#58;the&#45;input&#45;element&#45;2"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code>&#44; <a id="the&#45;tabindex&#45;attribute&#58;the&#45;object&#45;element"></a><code>[object](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-object-element)</code>&#44; <a id="the&#45;tabindex&#45;attribute&#58;the&#45;select&#45;element&#45;2"></a><code>[select](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-select-element)</code>&#44; <a id="the&#45;tabindex&#45;attribute&#58;the&#45;textarea&#45;element&#45;2"></a><code>[textarea](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-textarea-element)</code>&#44; or <a id="the&#45;tabindex&#45;attribute&#58;svg&#45;a"></a>[SVG <code>a</code>](https://w3c.github.io/svgwg/svg2-draft/linking.html#elementdef-a) element&#44; or <a id="the&#45;tabindex&#45;attribute&#58;mathml&#45;a"></a>[MathML <code>a</code>](https://w3c.github.io/mathml-core/#dfn-a) element&#44; or is a <a id="the&#45;tabindex&#45;attribute&#58;the&#45;summary&#45;element&#45;3"></a><code>[summary](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-summary-element)</code> element that is a <a id="the&#45;tabindex&#45;attribute&#58;summary&#45;for&#45;its&#45;parent&#45;details"></a>[summary for its parent details](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#summary-for-its-parent-details)&#59; otherwise −1&#46;

<strong>Note&#58;</strong>

The varying default value based on element type is a historical artifact&#46;

#### <a id="focus&#45;processing&#45;model"></a>6&#46;6&#46;4 <a id="processing&#45;model&#45;5"></a>Processing model[](#focus-processing-model)

To <a id="get&#45;the&#45;focusable&#45;area"></a><strong>get the focusable area</strong> for a <var>focus target</var> that is either an element that is not a <a id="focus&#45;processing&#45;model&#58;focusable&#45;area"></a>[focusable area](#focusable-area)&#44; or is a <a id="focus&#45;processing&#45;model&#58;navigable"></a>[navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable)&#44; given an optional string <var>focus trigger</var> &#40;default &#34;<code>other</code>&#34;&#41;&#44; run the first matching set of steps from the following list&#58;

<strong>If <var>focus target</var> is an <a id="focus&#45;processing&#45;model&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> element with one or more shapes that are <a id="focus&#45;processing&#45;model&#58;focusable&#45;area&#45;2"></a>[focusable areas](#focusable-area)</strong>

Return the shape corresponding to the first <a id="focus&#45;processing&#45;model&#58;the&#45;img&#45;element"></a><code>[img](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-img-element)</code> element in <a id="focus&#45;processing&#45;model&#58;tree&#45;order"></a>[tree order](https://dom.spec.whatwg.org/#concept-tree-order) that uses the image map to which the <a id="focus&#45;processing&#45;model&#58;the&#45;area&#45;element&#45;2"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> element belongs&#46;

<strong>If <var>focus target</var> is an element with one or more scrollable regions that are <a id="focus&#45;processing&#45;model&#58;focusable&#45;area&#45;3"></a>[focusable areas](#focusable-area)</strong>

Return the element&#39;s first scrollable region&#44; according to a pre&#45;order&#44; depth&#45;first traversal of the <a id="focus&#45;processing&#45;model&#58;flat&#45;tree"></a>[flat tree](https://drafts.csswg.org/css-scoping/#flat-tree)&#46; [&#91;CSSSCOPING&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsCSSSCOPING)

<strong>If <var>focus target</var> is the <a id="focus&#45;processing&#45;model&#58;document&#45;element"></a>[document element](https://dom.spec.whatwg.org/#document-element) of its <a id="focus&#45;processing&#45;model&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code></strong>

Return the <a id="focus&#45;processing&#45;model&#58;document&#45;2"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#39;s <a id="focus&#45;processing&#45;model&#58;viewport"></a>[viewport](https://drafts.csswg.org/css2/#viewport)&#46;

<strong>If <var>focus target</var> is a <a id="focus&#45;processing&#45;model&#58;navigable&#45;2"></a>[navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable)</strong>

Return the <a id="focus&#45;processing&#45;model&#58;navigable&#45;3"></a>[navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable)&#39;s <a id="focus&#45;processing&#45;model&#58;nav&#45;document"></a>[active document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document)&#46;

<strong>If <var>focus target</var> is a <a id="focus&#45;processing&#45;model&#58;navigable&#45;container"></a>[navigable container](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container) with a non&#45;null <a id="focus&#45;processing&#45;model&#58;content&#45;navigable"></a>[content navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable)</strong>

Return the <a id="focus&#45;processing&#45;model&#58;navigable&#45;container&#45;2"></a>[navigable container](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container)&#39;s <a id="focus&#45;processing&#45;model&#58;content&#45;navigable&#45;2"></a>[content navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable)&#39;s <a id="focus&#45;processing&#45;model&#58;nav&#45;document&#45;2"></a>[active document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document)&#46;

<strong>If <var>focus target</var> is a <a id="focus&#45;processing&#45;model&#58;shadow&#45;host"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host) whose <a id="focus&#45;processing&#45;model&#58;concept&#45;element&#45;shadow&#45;root"></a>[shadow root](https://dom.spec.whatwg.org/#concept-element-shadow-root)&#39;s <a id="focus&#45;processing&#45;model&#58;delegates&#45;focus"></a>[delegates focus](https://dom.spec.whatwg.org/#shadowroot-delegates-focus) is true</strong>

1.  Let <var>focusedElement</var> be the <a id="focus&#45;processing&#45;model&#58;currently&#45;focused&#45;area&#45;of&#45;a&#45;top&#45;level&#45;traversable"></a>[currently focused area of a top&#45;level traversable](#currently-focused-area-of-a-top-level-traversable)&#39;s <a id="focus&#45;processing&#45;model&#58;dom&#45;anchor"></a>[DOM anchor](#dom-anchor)&#46;

2.  If <var>focus target</var> is a <a id="focus&#45;processing&#45;model&#58;shadow&#45;including&#45;inclusive&#45;ancestor"></a>[shadow&#45;including inclusive ancestor](https://dom.spec.whatwg.org/#concept-shadow-including-inclusive-ancestor) of <var>focusedElement</var>&#44; then return <var>focusedElement</var>&#46;

3.  Return the <a id="focus&#45;processing&#45;model&#58;focus&#45;delegate"></a>[focus delegate](#focus-delegate) for <var>focus target</var> given <var>focus trigger</var>&#46;

<strong>Note&#58;</strong>

For <a id="focus&#45;processing&#45;model&#58;sequentially&#45;focusable"></a>[sequential focusability](#sequentially-focusable)&#44; the handling of <a id="focus&#45;processing&#45;model&#58;shadow&#45;host&#45;2"></a>[shadow hosts](https://dom.spec.whatwg.org/#element-shadow-host) and <a id="focus&#45;processing&#45;model&#58;delegates&#45;focus&#45;2"></a>[delegates focus](https://dom.spec.whatwg.org/#shadowroot-delegates-focus) is done when constructing the <a id="focus&#45;processing&#45;model&#58;sequential&#45;focus&#45;navigation&#45;order"></a>[sequential focus navigation order](#sequential-focus-navigation-order)&#46; That is&#44; the <a id="focus&#45;processing&#45;model&#58;focusing&#45;steps"></a>[focusing steps](#focusing-steps) will never be called on such <a id="focus&#45;processing&#45;model&#58;shadow&#45;host&#45;3"></a>[shadow hosts](https://dom.spec.whatwg.org/#element-shadow-host) as part of sequential focus navigation&#46;

<strong>Otherwise</strong>

Return null&#46;

The <a id="focus&#45;delegate"></a><strong>focus delegate</strong> for a <var>focusTarget</var>&#44; given an optional string <var>focusTrigger</var> &#40;default &#34;<code>other</code>&#34;&#41;&#44; is given by the following steps&#58;

1.  If <var>focusTarget</var> is a <a id="focus&#45;processing&#45;model&#58;shadow&#45;host&#45;4"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host) and its <a id="focus&#45;processing&#45;model&#58;concept&#45;element&#45;shadow&#45;root&#45;2"></a>[shadow root](https://dom.spec.whatwg.org/#concept-element-shadow-root)&#39;s <a id="focus&#45;processing&#45;model&#58;delegates&#45;focus&#45;3"></a>[delegates focus](https://dom.spec.whatwg.org/#shadowroot-delegates-focus) is false&#44; then return null&#46;

2.  Let <var>whereToLook</var> be <var>focusTarget</var>&#46;

3.  If <var>whereToLook</var> is a <a id="focus&#45;processing&#45;model&#58;shadow&#45;host&#45;5"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host)&#44; then set <var>whereToLook</var> to <var>whereToLook</var>&#39;s <a id="focus&#45;processing&#45;model&#58;concept&#45;element&#45;shadow&#45;root&#45;3"></a>[shadow root](https://dom.spec.whatwg.org/#concept-element-shadow-root)&#46;

4.  Let <var>autofocusDelegate</var> be the <a id="focus&#45;processing&#45;model&#58;autofocus&#45;delegate"></a>[autofocus delegate](#autofocus-delegate) for <var>whereToLook</var> given <var>focusTrigger</var>&#46;

5.  If <var>autofocusDelegate</var> is not null&#44; then return <var>autofocusDelegate</var>&#46;

6.  <a id="focus&#45;processing&#45;model&#58;list&#45;iterate"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>descendant</var> of <var>whereToLook</var>&#39;s <a id="focus&#45;processing&#45;model&#58;descendant"></a>[descendants](https://dom.spec.whatwg.org/#concept-tree-descendant)&#44; in <a id="focus&#45;processing&#45;model&#58;tree&#45;order&#45;2"></a>[tree order](https://dom.spec.whatwg.org/#concept-tree-order)&#58;

    1.  Let <var>focusableArea</var> be null&#46;

    2.  If <var>focusTarget</var> is a <a id="focus&#45;processing&#45;model&#58;the&#45;dialog&#45;element"></a><code>[dialog](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-dialog-element)</code> element and <var>descendant</var> is <a id="focus&#45;processing&#45;model&#58;sequentially&#45;focusable&#45;2"></a>[sequentially focusable](#sequentially-focusable)&#44; then set <var>focusableArea</var> to <var>descendant</var>&#46;

    3.  Otherwise&#44; if <var>focusTarget</var> is not a <a id="focus&#45;processing&#45;model&#58;the&#45;dialog&#45;element&#45;2"></a><code>[dialog](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-dialog-element)</code> and <var>descendant</var> is a <a id="focus&#45;processing&#45;model&#58;focusable&#45;area&#45;4"></a>[focusable area](#focusable-area)&#44; set <var>focusableArea</var> to <var>descendant</var>&#46;

    4.  Otherwise&#44; set <var>focusableArea</var> to the result of <a id="focus&#45;processing&#45;model&#58;get&#45;the&#45;focusable&#45;area"></a>[getting the focusable area](#get-the-focusable-area) for <var>descendant</var> given <var>focusTrigger</var>&#46;

        <strong>Note&#58;</strong>

        This step can end up recursing&#44; i&#46;e&#46;&#44; the <a id="focus&#45;processing&#45;model&#58;get&#45;the&#45;focusable&#45;area&#45;2"></a>[get the focusable area](#get-the-focusable-area) steps might return the <a id="focus&#45;processing&#45;model&#58;focus&#45;delegate&#45;2"></a>[focus delegate](#focus-delegate) of <var>descendant</var>&#46;

    5.  If <var>focusableArea</var> is not null&#44; then return <var>focusableArea</var>&#46;

    <strong>Note&#58;</strong>

    It&#39;s important that we are <em>not</em> looking at the <a id="focus&#45;processing&#45;model&#58;shadow&#45;including&#45;descendant"></a>[shadow&#45;including descendants](https://dom.spec.whatwg.org/#concept-shadow-including-descendant) here&#44; but instead only at the <a id="focus&#45;processing&#45;model&#58;descendant&#45;2"></a>[descendants](https://dom.spec.whatwg.org/#concept-tree-descendant)&#46; <a id="focus&#45;processing&#45;model&#58;shadow&#45;host&#45;6"></a>[Shadow hosts](https://dom.spec.whatwg.org/#element-shadow-host) are instead handled by the recursive case mentioned above&#46;

7.  Return null&#46;

<strong>Note&#58;</strong>

The above algorithm essentially returns the first suitable <a id="focus&#45;processing&#45;model&#58;focusable&#45;area&#45;5"></a>[focusable area](#focusable-area) where the path between its <a id="focus&#45;processing&#45;model&#58;dom&#45;anchor&#45;2"></a>[DOM anchor](#dom-anchor) and <var>focusTarget</var> delegates focus at any shadow tree boundaries&#46;

The <a id="autofocus&#45;delegate"></a><strong>autofocus delegate</strong> for a <var>focus target</var> given a <var>focus trigger</var> is given by the following steps&#58;

1.  For each <a id="focus&#45;processing&#45;model&#58;descendant&#45;3"></a>[descendant](https://dom.spec.whatwg.org/#concept-tree-descendant) <var>descendant</var> of <var>focus target</var>&#44; in <a id="focus&#45;processing&#45;model&#58;tree&#45;order&#45;3"></a>[tree order](https://dom.spec.whatwg.org/#concept-tree-order)&#58;

    1.  If <var>descendant</var> does not have an <a id="focus&#45;processing&#45;model&#58;attr&#45;fe&#45;autofocus"></a><code>[autofocus](#attr-fe-autofocus)</code> content attribute&#44; then <a id="focus&#45;processing&#45;model&#58;continue"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue)&#46;

    2.  Let <var>focusable area</var> be <var>descendant</var>&#44; if <var>descendant</var> is a <a id="focus&#45;processing&#45;model&#58;focusable&#45;area&#45;6"></a>[focusable area](#focusable-area)&#59; otherwise let <var>focusable area</var> be the result of <a id="focus&#45;processing&#45;model&#58;get&#45;the&#45;focusable&#45;area&#45;3"></a>[getting the focusable area](#get-the-focusable-area) for <var>descendant</var> given <var>focus trigger</var>&#46;

    3.  If <var>focusable area</var> is null&#44; then <a id="focus&#45;processing&#45;model&#58;continue&#45;2"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue)&#46;

    4.  If <var>focusable area</var> is not <a id="focus&#45;processing&#45;model&#58;click&#45;focusable"></a>[click focusable](#click-focusable) and <var>focus trigger</var> is &#34;<code>click</code>&#34;&#44; then <a id="focus&#45;processing&#45;model&#58;continue&#45;3"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue)&#46;

    5.  Return <var>focusable area</var>&#46;

2.  Return null&#46;

The <a id="focusing&#45;steps"></a><strong>focusing steps</strong> for an object <var>new focus target</var> that is either a <a id="focus&#45;processing&#45;model&#58;focusable&#45;area&#45;7"></a>[focusable area](#focusable-area)&#44; or an element that is not a <a id="focus&#45;processing&#45;model&#58;focusable&#45;area&#45;8"></a>[focusable area](#focusable-area)&#44; or a <a id="focus&#45;processing&#45;model&#58;navigable&#45;4"></a>[navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable)&#44; are as follows&#46; They can optionally be run with a <var>fallback target</var> and a string <var>focus trigger</var>&#46;

1.  If <var>new focus target</var> is not a <a id="focus&#45;processing&#45;model&#58;focusable&#45;area&#45;9"></a>[focusable area](#focusable-area)&#44; then set <var>new focus target</var> to the result of <a id="focus&#45;processing&#45;model&#58;get&#45;the&#45;focusable&#45;area&#45;4"></a>[getting the focusable area](#get-the-focusable-area) for <var>new focus target</var>&#44; given <var>focus trigger</var> if it was passed&#46;

2.  If <var>new focus target</var> is null&#58;

    1.  If no <var>fallback target</var> was specified&#44; then return&#46;

    2.  Otherwise&#44; set <var>new focus target</var> to the <var>fallback target</var>&#46;

3.  If <var>new focus target</var> is a <a id="focus&#45;processing&#45;model&#58;navigable&#45;container&#45;3"></a>[navigable container](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container) with non&#45;null <a id="focus&#45;processing&#45;model&#58;content&#45;navigable&#45;3"></a>[content navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable)&#44; then set <var>new focus target</var> to the <a id="focus&#45;processing&#45;model&#58;content&#45;navigable&#45;4"></a>[content navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable)&#39;s <a id="focus&#45;processing&#45;model&#58;nav&#45;document&#45;3"></a>[active document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document)&#46;

4.  If <var>new focus target</var> is a <a id="focus&#45;processing&#45;model&#58;focusable&#45;area&#45;10"></a>[focusable area](#focusable-area) and its <a id="focus&#45;processing&#45;model&#58;dom&#45;anchor&#45;3"></a>[DOM anchor](#dom-anchor) is <a id="focus&#45;processing&#45;model&#58;inert"></a>[inert](#inert)&#44; then return&#46;

5.  If <var>new focus target</var> is the <a id="focus&#45;processing&#45;model&#58;currently&#45;focused&#45;area&#45;of&#45;a&#45;top&#45;level&#45;traversable&#45;2"></a>[currently focused area of a top&#45;level traversable](#currently-focused-area-of-a-top-level-traversable)&#44; then return&#46;

6.  Let <var>old chain</var> be the <a id="focus&#45;processing&#45;model&#58;current&#45;focus&#45;chain&#45;of&#45;a&#45;top&#45;level&#45;traversable"></a>[current focus chain of the top&#45;level traversable](#current-focus-chain-of-a-top-level-traversable) in which <var>new focus target</var> finds itself&#46;

7.  Let <var>new chain</var> be the <a id="focus&#45;processing&#45;model&#58;focus&#45;chain"></a>[focus chain](#focus-chain) of <var>new focus target</var>&#46;

8.  Run the <a id="focus&#45;processing&#45;model&#58;focus&#45;update&#45;steps"></a>[focus update steps](#focus-update-steps) with <var>old chain</var>&#44; <var>new chain</var>&#44; and <var>new focus target</var> respectively&#46;

User agents must <a id="focus&#45;processing&#45;model&#58;immediately"></a>[immediately](html-language-foundations--877d6146487f--b55e73c31978.md#immediately) run the <a id="focus&#45;processing&#45;model&#58;focusing&#45;steps&#45;2"></a>[focusing steps](#focusing-steps) for a <a id="focus&#45;processing&#45;model&#58;focusable&#45;area&#45;11"></a>[focusable area](#focusable-area) or <a id="focus&#45;processing&#45;model&#58;navigable&#45;5"></a>[navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable) <var>candidate</var> whenever the user attempts to move the focus to <var>candidate</var>&#46;

The <a id="unfocusing&#45;steps"></a><strong>unfocusing steps</strong> for an object <var>old focus target</var> that is either a <a id="focus&#45;processing&#45;model&#58;focusable&#45;area&#45;12"></a>[focusable area](#focusable-area) or an element that is not a <a id="focus&#45;processing&#45;model&#58;focusable&#45;area&#45;13"></a>[focusable area](#focusable-area) are as follows&#58;

1.  If <var>old focus target</var> is a <a id="focus&#45;processing&#45;model&#58;shadow&#45;host&#45;7"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host) whose <a id="focus&#45;processing&#45;model&#58;concept&#45;element&#45;shadow&#45;root&#45;4"></a>[shadow root](https://dom.spec.whatwg.org/#concept-element-shadow-root)&#39;s <a id="focus&#45;processing&#45;model&#58;delegates&#45;focus&#45;4"></a>[delegates focus](https://dom.spec.whatwg.org/#shadowroot-delegates-focus) is true&#44; and <var>old focus target</var>&#39;s <a id="focus&#45;processing&#45;model&#58;concept&#45;element&#45;shadow&#45;root&#45;5"></a>[shadow root](https://dom.spec.whatwg.org/#concept-element-shadow-root) is a <a id="focus&#45;processing&#45;model&#58;shadow&#45;including&#45;inclusive&#45;ancestor&#45;2"></a>[shadow&#45;including inclusive ancestor](https://dom.spec.whatwg.org/#concept-shadow-including-inclusive-ancestor) of the <a id="focus&#45;processing&#45;model&#58;currently&#45;focused&#45;area&#45;of&#45;a&#45;top&#45;level&#45;traversable&#45;3"></a>[currently focused area of a top&#45;level traversable](#currently-focused-area-of-a-top-level-traversable)&#39;s <a id="focus&#45;processing&#45;model&#58;dom&#45;anchor&#45;4"></a>[DOM anchor](#dom-anchor)&#44; then set <var>old focus target</var> to that <a id="focus&#45;processing&#45;model&#58;currently&#45;focused&#45;area&#45;of&#45;a&#45;top&#45;level&#45;traversable&#45;4"></a>[currently focused area of a top&#45;level traversable](#currently-focused-area-of-a-top-level-traversable)&#46;

2.  If <var>old focus target</var> is <a id="focus&#45;processing&#45;model&#58;inert&#45;2"></a>[inert](#inert)&#44; then return&#46;

3.  If <var>old focus target</var> is an <a id="focus&#45;processing&#45;model&#58;the&#45;area&#45;element&#45;3"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> element and one of its shapes is the <a id="focus&#45;processing&#45;model&#58;currently&#45;focused&#45;area&#45;of&#45;a&#45;top&#45;level&#45;traversable&#45;5"></a>[currently focused area of a top&#45;level traversable](#currently-focused-area-of-a-top-level-traversable)&#44; or&#44; if <var>old focus target</var> is an element with one or more scrollable regions&#44; and one of them is the <a id="focus&#45;processing&#45;model&#58;currently&#45;focused&#45;area&#45;of&#45;a&#45;top&#45;level&#45;traversable&#45;6"></a>[currently focused area of a top&#45;level traversable](#currently-focused-area-of-a-top-level-traversable)&#44; then let <var>old focus target</var> be that <a id="focus&#45;processing&#45;model&#58;currently&#45;focused&#45;area&#45;of&#45;a&#45;top&#45;level&#45;traversable&#45;7"></a>[currently focused area of a top&#45;level traversable](#currently-focused-area-of-a-top-level-traversable)&#46;

4.  Let <var>old chain</var> be the <a id="focus&#45;processing&#45;model&#58;current&#45;focus&#45;chain&#45;of&#45;a&#45;top&#45;level&#45;traversable&#45;2"></a>[current focus chain of the top&#45;level traversable](#current-focus-chain-of-a-top-level-traversable) in which <var>old focus target</var> finds itself&#46;

5.  If <var>old focus target</var> is not one of the entries in <var>old chain</var>&#44; then return&#46;

6.  If <var>old focus target</var> is not a <a id="focus&#45;processing&#45;model&#58;focusable&#45;area&#45;14"></a>[focusable area](#focusable-area)&#44; then return&#46;

7.  Let <var>topDocument</var> be <var>old chain</var>&#39;s last entry&#46;

8.  If <var>topDocument</var>&#39;s <a id="focus&#45;processing&#45;model&#58;node&#45;navigable"></a>[node navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable) has <a id="focus&#45;processing&#45;model&#58;system&#45;focus"></a>[system focus](#system-focus)&#44; then run the <a id="focus&#45;processing&#45;model&#58;focusing&#45;steps&#45;3"></a>[focusing steps](#focusing-steps) for <var>topDocument</var>&#39;s <a id="focus&#45;processing&#45;model&#58;viewport&#45;2"></a>[viewport](https://drafts.csswg.org/css2/#viewport)&#46;

    Otherwise&#44; apply any relevant platform&#45;specific conventions for removing <a id="focus&#45;processing&#45;model&#58;system&#45;focus&#45;2"></a>[system focus](#system-focus) from <var>topDocument</var>&#39;s <a id="focus&#45;processing&#45;model&#58;node&#45;navigable&#45;2"></a>[node navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable)&#44; and run the <a id="focus&#45;processing&#45;model&#58;focus&#45;update&#45;steps&#45;2"></a>[focus update steps](#focus-update-steps) given <var>old chain</var>&#44; an empty list&#44; and null&#46;

<strong>Note&#58;</strong>

The <a id="focus&#45;processing&#45;model&#58;unfocusing&#45;steps"></a>[unfocusing steps](#unfocusing-steps) do not always result in the focus changing&#44; even when applied to the <a id="focus&#45;processing&#45;model&#58;currently&#45;focused&#45;area&#45;of&#45;a&#45;top&#45;level&#45;traversable&#45;8"></a>[currently focused area of a top&#45;level traversable](#currently-focused-area-of-a-top-level-traversable)&#46; For example&#44; if the <a id="focus&#45;processing&#45;model&#58;currently&#45;focused&#45;area&#45;of&#45;a&#45;top&#45;level&#45;traversable&#45;9"></a>[currently focused area of a top&#45;level traversable](#currently-focused-area-of-a-top-level-traversable) is a <a id="focus&#45;processing&#45;model&#58;viewport&#45;3"></a>[viewport](https://drafts.csswg.org/css2/#viewport)&#44; then it will usually keep its focus regardless until another <a id="focus&#45;processing&#45;model&#58;focusable&#45;area&#45;15"></a>[focusable area](#focusable-area) is explicitly focused with the <a id="focus&#45;processing&#45;model&#58;focusing&#45;steps&#45;4"></a>[focusing steps](#focusing-steps)&#46;

------------------------------------------------------------------------

The <a id="focus&#45;update&#45;steps"></a><strong>focus update steps</strong>&#44; given an <var>old chain</var>&#44; a <var>new chain</var>&#44; and a <var>new focus target</var> respectively&#44; are as follows&#58;

1.  If the last entry in <var>old chain</var> and the last entry in <var>new chain</var> are the same&#44; pop the last entry from <var>old chain</var> and the last entry from <var>new chain</var> and redo this step&#46;

2.  For each entry <var>entry</var> in <var>old chain</var>&#44; in order&#44; run these substeps&#58;

    1.  If <var>entry</var> is an <a id="focus&#45;processing&#45;model&#58;the&#45;input&#45;element"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> element&#44; and the <a id="focus&#45;processing&#45;model&#58;event&#45;change"></a><code>[change](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-change)</code> event <a id="focus&#45;processing&#45;model&#58;concept&#45;input&#45;apply"></a>[applies](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-input-apply) to the element&#44; and the element does not have a defined <a id="focus&#45;processing&#45;model&#58;activation&#45;behaviour"></a>[activation behavior](https://dom.spec.whatwg.org/#eventtarget-activation-behavior)&#44; and the user has changed the element&#39;s <a id="focus&#45;processing&#45;model&#58;concept&#45;fe&#45;value"></a>[value](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fe-value) or its list of <a id="focus&#45;processing&#45;model&#58;concept&#45;input&#45;type&#45;file&#45;selected"></a>[selected files](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-input-type-file-selected) while the control was focused without committing that change &#40;such that it is different to what it was when the control was first focused&#41;&#58;

        1.  Set <var>entry</var>&#39;s <a id="focus&#45;processing&#45;model&#58;user&#45;validity"></a>[user validity](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#user-validity) to true&#46;

        2.  <a id="unfocus&#45;causes&#45;change&#45;event"></a>

            <a id="focus&#45;processing&#45;model&#58;concept&#45;event&#45;fire"></a>[Fire an event](https://dom.spec.whatwg.org/#concept-event-fire) named <a id="focus&#45;processing&#45;model&#58;event&#45;change&#45;2"></a><code>[change](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-change)</code> at the element&#44; with the <a id="focus&#45;processing&#45;model&#58;dom&#45;event&#45;bubbles"></a><code>[bubbles](https://dom.spec.whatwg.org/#dom-event-bubbles)</code> attribute initialized to true&#46;

    2.  If <var>entry</var> is an element&#44; let <var>blur event target</var> be <var>entry</var>&#46;

        If <var>entry</var> is a <a id="focus&#45;processing&#45;model&#58;document&#45;3"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> object&#44; let <var>blur event target</var> be that <a id="focus&#45;processing&#45;model&#58;document&#45;4"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> object&#39;s <a id="focus&#45;processing&#45;model&#58;concept&#45;relevant&#45;global"></a>[relevant global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global)&#46;

        Otherwise&#44; let <var>blur event target</var> be null&#46;

    3.  If <var>entry</var> is the last entry in <var>old chain</var>&#44; and <var>entry</var> is an <a id="focus&#45;processing&#45;model&#58;element"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code>&#44; and the last entry in <var>new chain</var> is also an <a id="focus&#45;processing&#45;model&#58;element&#45;2"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code>&#44; then let <var>related blur target</var> be the last entry in <var>new chain</var>&#46; Otherwise&#44; let <var>related blur target</var> be null&#46;

    4.  If <var>blur event target</var> is not null&#44; <a id="focus&#45;processing&#45;model&#58;fire&#45;a&#45;focus&#45;event"></a>[fire a focus event](#fire-a-focus-event) named <a id="focus&#45;processing&#45;model&#58;event&#45;blur"></a><code>[blur](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-blur)</code> at <var>blur event target</var>&#44; with <var>related blur target</var> as the related target&#46;

        <a id="note&#45;sometimes&#45;no&#45;blur&#45;event"></a>

        <strong>Note&#58;</strong>

        [](#note-sometimes-no-blur-event)In some cases&#44; e&#46;g&#46;&#44; if <var>entry</var> is an <a id="focus&#45;processing&#45;model&#58;the&#45;area&#45;element&#45;4"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> element&#39;s shape&#44; a scrollable region&#44; or a <a id="focus&#45;processing&#45;model&#58;viewport&#45;4"></a>[viewport](https://drafts.csswg.org/css2/#viewport)&#44; no event is fired&#46;

3.  Apply any relevant platform&#45;specific conventions for focusing <var>new focus target</var>&#46; &#40;For example&#44; some platforms select the contents of a text control when that control is focused&#46;&#41;

4.  For each entry <var>entry</var> in <var>new chain</var>&#44; in reverse order&#44; run these substeps&#58;

    1.  If <var>entry</var> is a <a id="focus&#45;processing&#45;model&#58;focusable&#45;area&#45;16"></a>[focusable area](#focusable-area)&#44; and the <a id="focus&#45;processing&#45;model&#58;focused&#45;area&#45;of&#45;the&#45;document"></a>[focused area of the document](#focused-area-of-the-document) is not <var>entry</var>&#58;

        1.  Set <var>document</var>&#39;s <a id="focus&#45;processing&#45;model&#58;concept&#45;relevant&#45;global&#45;2"></a>[relevant global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global)&#39;s <a id="focus&#45;processing&#45;model&#58;window&#45;navigation&#45;api"></a>[navigation API](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window-navigation-api)&#39;s <a id="focus&#45;processing&#45;model&#58;focus&#45;changed&#45;during&#45;ongoing&#45;navigation"></a>[focus changed during ongoing navigation](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#focus-changed-during-ongoing-navigation) to true&#46;

        2.  Designate <var>entry</var> as the <a id="focus&#45;processing&#45;model&#58;focused&#45;area&#45;of&#45;the&#45;document&#45;2"></a>[focused area of the document](#focused-area-of-the-document)&#46;

    2.  If <var>entry</var> is an element&#44; let <var>focus event target</var> be <var>entry</var>&#46;

        If <var>entry</var> is a <a id="focus&#45;processing&#45;model&#58;document&#45;5"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> object&#44; let <var>focus event target</var> be that <a id="focus&#45;processing&#45;model&#58;document&#45;6"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> object&#39;s <a id="focus&#45;processing&#45;model&#58;concept&#45;relevant&#45;global&#45;3"></a>[relevant global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global)&#46;

        Otherwise&#44; let <var>focus event target</var> be null&#46;

    3.  If <var>entry</var> is the last entry in <var>new chain</var>&#44; and <var>entry</var> is an <a id="focus&#45;processing&#45;model&#58;element&#45;3"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code>&#44; and the last entry in <var>old chain</var> is also an <a id="focus&#45;processing&#45;model&#58;element&#45;4"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code>&#44; then let <var>related focus target</var> be the last entry in <var>old chain</var>&#46; Otherwise&#44; let <var>related focus target</var> be null&#46;

    4.  If <var>focus event target</var> is not null&#44; <a id="focus&#45;processing&#45;model&#58;fire&#45;a&#45;focus&#45;event&#45;2"></a>[fire a focus event](#fire-a-focus-event) named <a id="focus&#45;processing&#45;model&#58;event&#45;focus"></a><code>[focus](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-focus)</code> at <var>focus event target</var>&#44; with <var>related focus target</var> as the related target&#46;

        <strong>Note&#58;</strong>

        In some cases&#44; e&#46;g&#46; if <var>entry</var> is an <a id="focus&#45;processing&#45;model&#58;the&#45;area&#45;element&#45;5"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> element&#39;s shape&#44; a scrollable region&#44; or a <a id="focus&#45;processing&#45;model&#58;viewport&#45;5"></a>[viewport](https://drafts.csswg.org/css2/#viewport)&#44; no event is fired&#46;

To <a id="fire&#45;a&#45;focus&#45;event"></a><strong>fire a focus event</strong> named <var>e</var> at an element or a <a id="focus&#45;processing&#45;model&#58;window"></a><code>[Window](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window)</code> object <var>target</var> given a related target <var>relatedTarget</var>&#58;

1.  Let <var>view</var> be <var>target</var> if <var>target</var> is a <a id="focus&#45;processing&#45;model&#58;window&#45;2"></a><code>[Window](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window)</code> object&#59; otherwise <var>target</var>&#39;s <a id="focus&#45;processing&#45;model&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="focus&#45;processing&#45;model&#58;concept&#45;relevant&#45;global&#45;4"></a>[relevant global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global)&#46;

2.  <a id="focus&#45;processing&#45;model&#58;concept&#45;event&#45;fire&#45;2"></a>[Fire an event](https://dom.spec.whatwg.org/#concept-event-fire) named <var>e</var> at <var>target</var>&#44; using <a id="focus&#45;processing&#45;model&#58;focusevent"></a><code>[FocusEvent](https://w3c.github.io/uievents/#focusevent)</code>&#44; with the <a id="focus&#45;processing&#45;model&#58;dom&#45;focusevent&#45;relatedtarget"></a><code>[relatedTarget](https://w3c.github.io/uievents/#dom-focusevent-relatedtarget)</code> attribute initialized to <var>relatedTarget</var>&#44; the <a id="focus&#45;processing&#45;model&#58;dom&#45;uievent&#45;view"></a><code>[view](https://w3c.github.io/uievents/#dom-uievent-view)</code> attribute initialized to <var>view</var>&#44; and the <a id="focus&#45;processing&#45;model&#58;composed&#45;flag"></a>[composed flag](https://dom.spec.whatwg.org/#composed-flag) set&#46;

------------------------------------------------------------------------

When a key event is to be routed in a <a id="focus&#45;processing&#45;model&#58;top&#45;level&#45;traversable"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable)&#44; the user agent must run the following steps&#58;

1.  Let <var>target area</var> be the <a id="focus&#45;processing&#45;model&#58;currently&#45;focused&#45;area&#45;of&#45;a&#45;top&#45;level&#45;traversable&#45;10"></a>[currently focused area of the top&#45;level traversable](#currently-focused-area-of-a-top-level-traversable)&#46;

2.  <a id="focus&#45;processing&#45;model&#58;assert"></a>[Assert](https://infra.spec.whatwg.org/#assert)&#58; <var>target area</var> is not null&#44; since key events are only routed to <a id="focus&#45;processing&#45;model&#58;top&#45;level&#45;traversable&#45;2"></a>[top&#45;level traversables](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable) that have <a id="focus&#45;processing&#45;model&#58;system&#45;focus&#45;3"></a>[system focus](#system-focus)&#46; Therefore&#44; <var>target area</var> is a <a id="focus&#45;processing&#45;model&#58;focusable&#45;area&#45;17"></a>[focusable area](#focusable-area)&#46;

3.  Let <var>target node</var> be <var>target area</var>&#39;s <a id="focus&#45;processing&#45;model&#58;dom&#45;anchor&#45;5"></a>[DOM anchor](#dom-anchor)&#46;

4.  If <var>target node</var> is a <a id="focus&#45;processing&#45;model&#58;document&#45;7"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> that has a <a id="focus&#45;processing&#45;model&#58;the&#45;body&#45;element&#45;2"></a>[body element](html-language-foundations--877d6146487f--b55e73c31978.md#the-body-element-2)&#44; then let <var>target node</var> be <a id="focus&#45;processing&#45;model&#58;the&#45;body&#45;element&#45;2&#45;2"></a>[the body element](html-language-foundations--877d6146487f--b55e73c31978.md#the-body-element-2) of that <a id="focus&#45;processing&#45;model&#58;document&#45;8"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#46;

    Otherwise&#44; if <var>target node</var> is a <a id="focus&#45;processing&#45;model&#58;document&#45;9"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> object that has a non&#45;null <a id="focus&#45;processing&#45;model&#58;document&#45;element&#45;2"></a>[document element](https://dom.spec.whatwg.org/#document-element)&#44; then let <var>target node</var> be that <a id="focus&#45;processing&#45;model&#58;document&#45;element&#45;3"></a>[document element](https://dom.spec.whatwg.org/#document-element)&#46;

5.  If <var>target node</var> is not <a id="focus&#45;processing&#45;model&#58;inert&#45;3"></a>[inert](#inert)&#58;

    1.  Let <var>canHandle</var> be the result of <a id="focus&#45;processing&#45;model&#58;concept&#45;event&#45;dispatch"></a>[dispatching](https://dom.spec.whatwg.org/#concept-event-dispatch) the key event at <var>target node</var>&#46;

    2.  If <var>canHandle</var> is true&#44; then let <var>target area</var> handle the key event&#46; This might include <a id="focus&#45;processing&#45;model&#58;fire&#45;a&#45;click&#45;event"></a>[firing a <code>click</code> event](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fire-a-click-event) at <var>target node</var>&#46;

------------------------------------------------------------------------

The <a id="has&#45;focus&#45;steps"></a><strong>has focus steps</strong>&#44; given a <a id="focus&#45;processing&#45;model&#58;document&#45;10"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> object <var>target</var>&#44; are as follows&#58;

1.  If <var>target</var>&#39;s <a id="focus&#45;processing&#45;model&#58;node&#45;navigable&#45;3"></a>[node navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable)&#39;s <a id="focus&#45;processing&#45;model&#58;nav&#45;top"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-top) does not have <a id="focus&#45;processing&#45;model&#58;system&#45;focus&#45;4"></a>[system focus](#system-focus)&#44; then return false&#46;

2.  Let <var>candidate</var> be <var>target</var>&#39;s <a id="focus&#45;processing&#45;model&#58;node&#45;navigable&#45;4"></a>[node navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable)&#39;s <a id="focus&#45;processing&#45;model&#58;nav&#45;top&#45;2"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-top)&#39;s <a id="focus&#45;processing&#45;model&#58;active&#45;document"></a>[active document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#active-document)&#46;

3.  While true&#58;

    1.  If <var>candidate</var> is <var>target</var>&#44; then return true&#46;

    2.  If the <a id="focus&#45;processing&#45;model&#58;focused&#45;area&#45;of&#45;the&#45;document&#45;3"></a>[focused area](#focused-area-of-the-document) of <var>candidate</var> is a <a id="focus&#45;processing&#45;model&#58;navigable&#45;container&#45;4"></a>[navigable container](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container) with a non&#45;null <a id="focus&#45;processing&#45;model&#58;content&#45;navigable&#45;5"></a>[content navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable)&#44; then set <var>candidate</var> to the <a id="focus&#45;processing&#45;model&#58;nav&#45;document&#45;4"></a>[active document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document) of that <a id="focus&#45;processing&#45;model&#58;navigable&#45;container&#45;5"></a>[navigable container](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container)&#39;s <a id="focus&#45;processing&#45;model&#58;content&#45;navigable&#45;6"></a>[content navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable)&#46;

    3.  Otherwise&#44; return false&#46;

#### <a id="sequential&#45;focus&#45;navigation"></a>6&#46;6&#46;5 <strong>Sequential focus navigation</strong>[](#sequential-focus-navigation)

Each <a id="sequential&#45;focus&#45;navigation&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> has a <a id="sequential&#45;focus&#45;navigation&#45;order"></a><strong>sequential focus navigation order</strong>&#44; which orders some or all of the <a id="sequential&#45;focus&#45;navigation&#58;focusable&#45;area"></a>[focusable areas](#focusable-area) in the <a id="sequential&#45;focus&#45;navigation&#58;document&#45;2"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> relative to each other&#46; Its contents and ordering are given by the <a id="sequential&#45;focus&#45;navigation&#58;flattened&#45;tabindex&#45;ordered&#45;focus&#45;navigation&#45;scope"></a>[flattened tabindex&#45;ordered focus navigation scope](#flattened-tabindex-ordered-focus-navigation-scope) of the <a id="sequential&#45;focus&#45;navigation&#58;document&#45;3"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#46;

<strong>Note&#58;</strong>

Per the rules defining the <a id="sequential&#45;focus&#45;navigation&#58;flattened&#45;tabindex&#45;ordered&#45;focus&#45;navigation&#45;scope&#45;2"></a>[flattened tabindex&#45;ordered focus navigation scope](#flattened-tabindex-ordered-focus-navigation-scope)&#44; the ordering is not necessarily related to the <a id="sequential&#45;focus&#45;navigation&#58;tree&#45;order"></a>[tree order](https://dom.spec.whatwg.org/#concept-tree-order) of the <a id="sequential&#45;focus&#45;navigation&#58;document&#45;4"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#46;

If a <a id="sequential&#45;focus&#45;navigation&#58;focusable&#45;area&#45;2"></a>[focusable area](#focusable-area) is omitted from the <a id="sequential&#45;focus&#45;navigation&#58;sequential&#45;focus&#45;navigation&#45;order"></a>[sequential focus navigation order](#sequential-focus-navigation-order) of its <a id="sequential&#45;focus&#45;navigation&#58;document&#45;5"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#44; then it is unreachable via <a id="sequential&#45;focus&#45;navigation&#58;sequential&#45;focus&#45;navigation"></a>[sequential focus navigation](#sequential-focus-navigation)&#46;

There can also be a <a id="sequential&#45;focus&#45;navigation&#45;starting&#45;point"></a><strong>sequential focus navigation starting point</strong>&#46; It is initially unset&#46; The user agent may set it when the user indicates that it should be moved&#46;

<strong>Example&#58;</strong>

For example&#44; the user agent could set it to the position of the user&#39;s click if the user clicks on the document contents&#46;

<strong>Note&#58;</strong>

User agents are required to set the <a id="sequential&#45;focus&#45;navigation&#58;sequential&#45;focus&#45;navigation&#45;starting&#45;point"></a>[sequential focus navigation starting point](#sequential-focus-navigation-starting-point) to the <a id="sequential&#45;focus&#45;navigation&#58;target&#45;element"></a>[target element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#target-element) when <a id="sequential&#45;focus&#45;navigation&#58;navigate&#45;fragid"></a>[navigating to a fragment](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigate-fragid)&#46;

A <a id="sequential&#45;focus&#45;direction"></a><strong>sequential focus direction</strong> is one of two possible values&#58; &#34;<a id="sequential&#45;focus&#45;forward"></a><strong><code>forward</code></strong>&#34;&#44; or &#34;<a id="sequential&#45;focus&#45;backward"></a><strong><code>backward</code></strong>&#34;&#46; They are used in the below algorithms to describe the direction in which sequential focus travels at the user&#39;s request&#46;

A <a id="selection&#45;mechanism"></a><strong>selection mechanism</strong> is one of two possible values&#58; &#34;<a id="selection&#45;mechanism&#45;dom"></a><strong><code>DOM</code></strong>&#34;&#44; or &#34;<a id="selection&#45;mechanism&#45;sequential"></a><strong><code>sequential</code></strong>&#34;&#46; They are used to describe how the <a id="sequential&#45;focus&#45;navigation&#58;sequential&#45;navigation&#45;search&#45;algorithm"></a>[sequential navigation search algorithm](#sequential-navigation-search-algorithm) finds the <a id="sequential&#45;focus&#45;navigation&#58;focusable&#45;area&#45;3"></a>[focusable area](#focusable-area) it returns&#46;

When the user requests that focus move from the <a id="sequential&#45;focus&#45;navigation&#58;currently&#45;focused&#45;area&#45;of&#45;a&#45;top&#45;level&#45;traversable"></a>[currently focused area of a top&#45;level traversable](#currently-focused-area-of-a-top-level-traversable) to the next or previous <a id="sequential&#45;focus&#45;navigation&#58;focusable&#45;area&#45;4"></a>[focusable area](#focusable-area) &#40;e&#46;g&#46;&#44; as the default action of pressing the tab key&#41;&#44; or when the user requests that focus sequentially move to a <a id="sequential&#45;focus&#45;navigation&#58;top&#45;level&#45;traversable"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable) in the first place &#40;e&#46;g&#46;&#44; from the browser&#39;s location bar&#41;&#44; the user agent must use the following algorithm&#58;

1.  Let <var>starting point</var> be the <a id="sequential&#45;focus&#45;navigation&#58;currently&#45;focused&#45;area&#45;of&#45;a&#45;top&#45;level&#45;traversable&#45;2"></a>[currently focused area of a top&#45;level traversable](#currently-focused-area-of-a-top-level-traversable)&#44; if the user requested to move focus sequentially from there&#44; or else the <a id="sequential&#45;focus&#45;navigation&#58;top&#45;level&#45;traversable&#45;2"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable) itself&#44; if the user instead requested to move focus from outside the <a id="sequential&#45;focus&#45;navigation&#58;top&#45;level&#45;traversable&#45;3"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable)&#46;

2.  If there is a <a id="sequential&#45;focus&#45;navigation&#58;sequential&#45;focus&#45;navigation&#45;starting&#45;point&#45;2"></a>[sequential focus navigation starting point](#sequential-focus-navigation-starting-point) defined and it is inside <var>starting point</var>&#44; then let <var>starting point</var> be the <a id="sequential&#45;focus&#45;navigation&#58;sequential&#45;focus&#45;navigation&#45;starting&#45;point&#45;3"></a>[sequential focus navigation starting point](#sequential-focus-navigation-starting-point) instead&#46;

3.  Let <var>direction</var> be &#34;<a id="sequential&#45;focus&#45;navigation&#58;sequential&#45;focus&#45;forward"></a><code>[forward](#sequential-focus-forward)</code>&#34; if the user requested the next control&#44; and &#34;<a id="sequential&#45;focus&#45;navigation&#58;sequential&#45;focus&#45;backward"></a><code>[backward](#sequential-focus-backward)</code>&#34; if the user requested the previous control&#46;

    <strong>Note&#58;</strong>

    Typically&#44; pressing tab requests the next control&#44; and pressing shift &#43; tab requests the previous control&#46;

4.  <em>Loop</em>&#58; Let <var>selection mechanism</var> be &#34;<a id="sequential&#45;focus&#45;navigation&#58;selection&#45;mechanism&#45;sequential"></a><code>[sequential](#selection-mechanism-sequential)</code>&#34; if <var>starting point</var> is a <a id="sequential&#45;focus&#45;navigation&#58;navigable"></a>[navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable) or if <var>starting point</var> is in its <a id="sequential&#45;focus&#45;navigation&#58;document&#45;6"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#39;s <a id="sequential&#45;focus&#45;navigation&#58;sequential&#45;focus&#45;navigation&#45;order&#45;2"></a>[sequential focus navigation order](#sequential-focus-navigation-order)&#46;

    Otherwise&#44; <var>starting point</var> is not in its <a id="sequential&#45;focus&#45;navigation&#58;document&#45;7"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#39;s <a id="sequential&#45;focus&#45;navigation&#58;sequential&#45;focus&#45;navigation&#45;order&#45;3"></a>[sequential focus navigation order](#sequential-focus-navigation-order)&#59; let <var>selection mechanism</var> be &#34;<a id="sequential&#45;focus&#45;navigation&#58;selection&#45;mechanism&#45;dom"></a><code>[DOM](#selection-mechanism-dom)</code>&#34;&#46;

5.  Let <var>candidate</var> be the result of running the <a id="sequential&#45;focus&#45;navigation&#58;sequential&#45;navigation&#45;search&#45;algorithm&#45;2"></a>[sequential navigation search algorithm](#sequential-navigation-search-algorithm) with <var>starting point</var>&#44; <var>direction</var>&#44; and <var>selection mechanism</var>&#46;

6.  If <var>candidate</var> is not null&#44; then run the <a id="sequential&#45;focus&#45;navigation&#58;focusing&#45;steps"></a>[focusing steps](#focusing-steps) for <var>candidate</var> and return&#46;

7.  Otherwise&#44; unset the <a id="sequential&#45;focus&#45;navigation&#58;sequential&#45;focus&#45;navigation&#45;starting&#45;point&#45;4"></a>[sequential focus navigation starting point](#sequential-focus-navigation-starting-point)&#46;

8.  If <var>starting point</var> is a <a id="sequential&#45;focus&#45;navigation&#58;top&#45;level&#45;traversable&#45;4"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable)&#44; or a <a id="sequential&#45;focus&#45;navigation&#58;focusable&#45;area&#45;5"></a>[focusable area](#focusable-area) in the <a id="sequential&#45;focus&#45;navigation&#58;top&#45;level&#45;traversable&#45;5"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable)&#44; the user agent should transfer focus to its own controls appropriately &#40;if any&#41;&#44; honouring <var>direction</var>&#44; and then return&#46;

    <strong>Example&#58;</strong>

    For example&#44; if <var>direction</var> is <em>backward</em>&#44; then the last <a id="sequential&#45;focus&#45;navigation&#58;sequentially&#45;focusable"></a>[sequentially focusable](#sequentially-focusable) control before the browser&#39;s rendering area would be the control to focus&#46;

    If the user agent has no <a id="sequential&#45;focus&#45;navigation&#58;sequentially&#45;focusable&#45;2"></a>[sequentially focusable](#sequentially-focusable) controls — a kiosk&#45;mode browser&#44; for instance — then the user agent may instead restart these steps with the <var>starting point</var> being the <a id="sequential&#45;focus&#45;navigation&#58;top&#45;level&#45;traversable&#45;6"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable) itself&#46;

9.  Otherwise&#44; <var>starting point</var> is a <a id="sequential&#45;focus&#45;navigation&#58;focusable&#45;area&#45;6"></a>[focusable area](#focusable-area) in a <a id="sequential&#45;focus&#45;navigation&#58;child&#45;navigable"></a>[child navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#child-navigable)&#46; Set <var>starting point</var> to that <a id="sequential&#45;focus&#45;navigation&#58;child&#45;navigable&#45;2"></a>[child navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#child-navigable)&#39;s <a id="sequential&#45;focus&#45;navigation&#58;nav&#45;parent"></a>[parent](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-parent) and return to the step labeled <em>loop</em>&#46;

The <a id="sequential&#45;navigation&#45;search&#45;algorithm"></a><strong>sequential navigation search algorithm</strong>&#44; given a <a id="sequential&#45;focus&#45;navigation&#58;focusable&#45;area&#45;7"></a>[focusable area](#focusable-area) <var>starting point</var>&#44; <a id="sequential&#45;focus&#45;navigation&#58;sequential&#45;focus&#45;direction"></a>[sequential focus direction](#sequential-focus-direction) <var>direction</var>&#44; and <a id="sequential&#45;focus&#45;navigation&#58;selection&#45;mechanism"></a>[selection mechanism](#selection-mechanism) <var>selection mechanism</var>&#44; consists of the following steps&#46; They return a <a id="sequential&#45;focus&#45;navigation&#58;focusable&#45;area&#45;8"></a>[focusable area](#focusable-area)&#45;or&#45;null&#46;

1.  Pick the appropriate cell from the following table&#44; and follow the instructions in that cell&#46;

    The appropriate cell is the one that is from the column whose header describes <var>direction</var> and from the first row whose header describes <var>starting point</var> and <var>selection mechanism</var>&#46;

    <table><thead><tr><th>        </th><th><var>direction</var> is &#34;<code id="sequential-focus-navigation:sequential-focus-forward-2"><a href="#sequential-focus-forward">forward</a></code>&#34;        </th><th><var>direction</var> is &#34;<code id="sequential-focus-navigation:sequential-focus-backward-2"><a href="#sequential-focus-backward">backward</a></code>&#34;      </th></tr></thead><tbody><tr><th><var>starting point</var> is a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id="sequential-focus-navigation:navigable-2">navigable</a>        </th><td>Let <var>candidate</var> be the first <a href="#suitable-sequentially-focusable-area" id="sequential-focus-navigation:suitable-sequentially-focusable-area">suitable sequentially focusable area</a>        in <var>starting point</var>&#39;s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document" id="sequential-focus-navigation:nav-document">active document</a>&#44; if any&#59; or        else null&#46;        </td><td>Let <var>candidate</var> be the last <a href="#suitable-sequentially-focusable-area" id="sequential-focus-navigation:suitable-sequentially-focusable-area-2">suitable sequentially focusable area</a>        in <var>starting point</var>&#39;s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document" id="sequential-focus-navigation:nav-document-2">active document</a>&#44; if any&#59; or        else null&#46;       </td></tr><tr><th><var>selection mechanism</var> is &#34;<code id="sequential-focus-navigation:selection-mechanism-dom-2"><a href="#selection-mechanism-dom">DOM</a></code>&#34;        </th><td>         <p>Let <var>candidate</var> be the <a href="#suitable-sequentially-focusable-area" id="sequential-focus-navigation:suitable-sequentially-focusable-area-3">suitable sequentially focusable area</a>&#44; that         appears nearest after <var>starting point</var> in <var>starting point</var>&#39;s         <code id="sequential-focus-navigation:document-8"><a href="html-language-foundations--877d6146487f--b55e73c31978.md#document">Document</a></code>&#44; in <a id="sequential-focus-navigation:shadow-including-tree-order" href="https://dom.spec.whatwg.org/#concept-shadow-including-tree-order" data-x-internal="shadow-including-tree-order">shadow&#45;including tree order</a>&#44; if any&#59; or else null&#46;</p>          <p class="note">In this case&#44; <var>starting point</var> does not necessarily belong to its         <code id="sequential-focus-navigation:document-9"><a href="html-language-foundations--877d6146487f--b55e73c31978.md#document">Document</a></code>&#39;s <a href="#sequential-focus-navigation-order" id="sequential-focus-navigation:sequential-focus-navigation-order-4">sequential focus navigation order</a>&#44; so we&#39;ll select the         <a href="#suitable-sequentially-focusable-area" id="sequential-focus-navigation:suitable-sequentially-focusable-area-4">suitable</a> <a href="https://infra.spec.whatwg.org/#list-item" id="sequential-focus-navigation:list-item" data-x-internal="list-item">item</a> from that list that comes <em>after</em> <var>starting point</var> in         <a id="sequential-focus-navigation:shadow-including-tree-order-2" href="https://dom.spec.whatwg.org/#concept-shadow-including-tree-order" data-x-internal="shadow-including-tree-order">shadow&#45;including tree order</a>&#46;</p>        </td><td>Let <var>candidate</var> be the <a href="#suitable-sequentially-focusable-area" id="sequential-focus-navigation:suitable-sequentially-focusable-area-5">suitable sequentially focusable area</a>&#44; that        appears nearest before <var>starting point</var> in <var>starting point</var>&#39;s        <code id="sequential-focus-navigation:document-10"><a href="html-language-foundations--877d6146487f--b55e73c31978.md#document">Document</a></code>&#44; in <a id="sequential-focus-navigation:shadow-including-tree-order-3" href="https://dom.spec.whatwg.org/#concept-shadow-including-tree-order" data-x-internal="shadow-including-tree-order">shadow&#45;including tree order</a>&#44; if any&#59; or else null&#46;       </td></tr><tr><th><var>selection mechanism</var> is &#34;<code id="sequential-focus-navigation:selection-mechanism-sequential-2"><a href="#selection-mechanism-sequential">sequential</a></code>&#34;        </th><td>Let <var>candidate</var> be the first <a href="#suitable-sequentially-focusable-area" id="sequential-focus-navigation:suitable-sequentially-focusable-area-6">suitable sequentially focusable area</a>        after <var>starting point</var>&#44; in <var>starting point</var>&#39;s <code id="sequential-focus-navigation:document-11"><a href="html-language-foundations--877d6146487f--b55e73c31978.md#document">Document</a></code>&#39;s        <a href="#sequential-focus-navigation-order" id="sequential-focus-navigation:sequential-focus-navigation-order-5">sequential focus navigation order</a>&#44; if any&#59; or else null&#46;        </td><td>Let <var>candidate</var> be the last <a href="#suitable-sequentially-focusable-area" id="sequential-focus-navigation:suitable-sequentially-focusable-area-7">suitable sequentially focusable area</a>        before <var>starting point</var>&#44; in <var>starting point</var>&#39;s <code id="sequential-focus-navigation:document-12"><a href="html-language-foundations--877d6146487f--b55e73c31978.md#document">Document</a></code>&#39;s        <a href="#sequential-focus-navigation-order" id="sequential-focus-navigation:sequential-focus-navigation-order-6">sequential focus navigation order</a>&#44; if any&#59; or else null&#46;     </td></tr></tbody></table>

    A <a id="suitable&#45;sequentially&#45;focusable&#45;area"></a><strong>suitable sequentially focusable area</strong> is a <a id="sequential&#45;focus&#45;navigation&#58;focusable&#45;area&#45;9"></a>[focusable area](#focusable-area) whose <a id="sequential&#45;focus&#45;navigation&#58;dom&#45;anchor"></a>[DOM anchor](#dom-anchor) is not <a id="sequential&#45;focus&#45;navigation&#58;inert"></a>[inert](#inert) and is <a id="sequential&#45;focus&#45;navigation&#58;sequentially&#45;focusable&#45;3"></a>[sequentially focusable](#sequentially-focusable)&#46;

2.  If <var>candidate</var> is a <a id="sequential&#45;focus&#45;navigation&#58;navigable&#45;container"></a>[navigable container](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container) with a non&#45;null <a id="sequential&#45;focus&#45;navigation&#58;content&#45;navigable"></a>[content navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable)&#58;

    1.  Let <var>recursive candidate</var> be the result of running the <a id="sequential&#45;focus&#45;navigation&#58;sequential&#45;navigation&#45;search&#45;algorithm&#45;3"></a>[sequential navigation search algorithm](#sequential-navigation-search-algorithm) with <var>candidate</var>&#39;s <a id="sequential&#45;focus&#45;navigation&#58;content&#45;navigable&#45;2"></a>[content navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable)&#44; <var>direction</var>&#44; and &#34;<a id="sequential&#45;focus&#45;navigation&#58;selection&#45;mechanism&#45;sequential&#45;3"></a><code>[sequential](#selection-mechanism-sequential)</code>&#34;&#46;

    2.  If <var>recursive candidate</var> is null&#44; then return the result of running the <a id="sequential&#45;focus&#45;navigation&#58;sequential&#45;navigation&#45;search&#45;algorithm&#45;4"></a>[sequential navigation search algorithm](#sequential-navigation-search-algorithm) with <var>candidate</var>&#44; <var>direction</var>&#44; and <var>selection mechanism</var>&#46;

    3.  Otherwise&#44; set <var>candidate</var> to <var>recursive candidate</var>&#46;

3.  Return <var>candidate</var>&#46;

#### <a id="focus&#45;management&#45;apis"></a>6&#46;6&#46;6 Focus management APIs[](#focus-management-apis)

<a id="focusoptions"></a><a id="focus&#45;management&#45;apis&#58;dom&#45;focusoptions&#45;preventscroll"></a><a id="focus&#45;management&#45;apis&#58;dom&#45;focusoptions&#45;focusvisible"></a>

``` text
dictionary FocusOptions {
  boolean preventScroll = false;
  boolean focusVisible;
};
```

<strong><code><var>documentOrShadowRoot</var>&#46;<a id="dom&#45;documentorshadowroot&#45;activeelement&#45;dev"></a>[activeElement](#dom-documentorshadowroot-activeelement)</code></strong>

<strong>✔</strong>MDN

[Document&#47;activeElement](https://developer.mozilla.org/en-US/docs/Web/API/Document/activeElement)

Support in all current engines&#46;

Firefox3&#43;Safari4&#43;Chrome2&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer6&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android37&#43;Samsung Internet&#63;Opera Android12&#46;1&#43;

[ShadowRoot&#47;activeElement](https://developer.mozilla.org/en-US/docs/Web/API/ShadowRoot/activeElement)

Support in all current engines&#46;

Firefox63&#43;Safari10&#43;Chrome53&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

Returns the deepest element in <var>documentOrShadowRoot</var> through which or to which key events are being routed&#46; This is&#44; roughly speaking&#44; the focused element in the document&#46;

For the purposes of this API&#44; when a <a id="focus&#45;management&#45;apis&#58;child&#45;navigable"></a>[child navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#child-navigable) is focused&#44; its <a id="focus&#45;management&#45;apis&#58;nav&#45;container"></a>[container](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-container) is [focused](#bc-focus-ergo-bcc-focus) within its <a id="focus&#45;management&#45;apis&#58;nav&#45;parent"></a>[parent](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-parent)&#39;s <a id="focus&#45;management&#45;apis&#58;nav&#45;document"></a>[active document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document)&#46; For example&#44; if the user moves the focus to a text control in an <a id="focus&#45;management&#45;apis&#58;the&#45;iframe&#45;element"></a><code>[iframe](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element)</code>&#44; the <a id="focus&#45;management&#45;apis&#58;the&#45;iframe&#45;element&#45;2"></a><code>[iframe](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element)</code> is the element returned by the <a id="focus&#45;management&#45;apis&#58;dom&#45;documentorshadowroot&#45;activeelement"></a><code>[activeElement](#dom-documentorshadowroot-activeelement)</code> API in the <a id="focus&#45;management&#45;apis&#58;the&#45;iframe&#45;element&#45;3"></a><code>[iframe](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-iframe-element)</code>&#39;s <a id="focus&#45;management&#45;apis&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46;

Similarly&#44; when the focused element is in a different <a id="focus&#45;management&#45;apis&#58;node&#45;tree"></a>[node tree](https://dom.spec.whatwg.org/#concept-node-tree) than <var>documentOrShadowRoot</var>&#44; the element returned will be the <a id="focus&#45;management&#45;apis&#58;concept&#45;documentfragment&#45;host"></a>[host](https://dom.spec.whatwg.org/#concept-documentfragment-host) that&#39;s located in the same <a id="focus&#45;management&#45;apis&#58;node&#45;tree&#45;2"></a>[node tree](https://dom.spec.whatwg.org/#concept-node-tree) as <var>documentOrShadowRoot</var> if <var>documentOrShadowRoot</var> is a <a id="focus&#45;management&#45;apis&#58;shadow&#45;including&#45;inclusive&#45;ancestor"></a>[shadow&#45;including inclusive ancestor](https://dom.spec.whatwg.org/#concept-shadow-including-inclusive-ancestor) of the focused element&#44; and null if not&#46;

<strong><code><var>document</var>&#46;<a id="dom&#45;document&#45;hasfocus&#45;dev"></a>[hasFocus](#dom-document-hasfocus)&#40;&#41;</code></strong>

<strong>✔</strong>MDN

[Document&#47;hasFocus](https://developer.mozilla.org/en-US/docs/Web/API/Document/hasFocus)

Support in all current engines&#46;

Firefox3&#43;Safari4&#43;Chrome2&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android37&#43;Samsung Internet&#63;Opera Android&#63;

Returns true if key events are being routed through or to <var>document</var>&#59; otherwise&#44; returns false&#46; Roughly speaking&#44; this corresponds to <var>document</var>&#44; or a document nested inside <var>document</var>&#44; being focused&#46;

<strong><code><var>window</var>&#46;<a id="dom&#45;window&#45;focus&#45;dev"></a>[focus](#dom-window-focus)&#40;&#41;</code></strong>

<strong>✔</strong>MDN

[Window&#47;focus](https://developer.mozilla.org/en-US/docs/Web/API/Window/focus)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer4&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android18&#43;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

Moves the focus to <var>window</var>&#39;s <a id="focus&#45;management&#45;apis&#58;window&#45;navigable"></a>[navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window-navigable)&#44; if any&#46;

<strong><code><var>element</var>&#46;<a id="dom&#45;focus&#45;dev"></a>[focus](#dom-focus)&#40;&#41;</code></strong>

<strong>✔</strong>MDN

[HTMLElement&#47;focus](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/focus)

Support in all current engines&#46;

Firefox1&#46;5&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera8&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android10&#46;1&#43;

<strong><code><var>element</var>&#46;<a id="focus&#45;management&#45;apis&#58;dom&#45;focus"></a>[focus](#dom-focus)&#40;&#123; <a id="focus&#45;management&#45;apis&#58;dom&#45;focusoptions&#45;preventscroll&#45;2"></a>[preventScroll](#dom-focusoptions-preventscroll)&#44; <a id="focus&#45;management&#45;apis&#58;dom&#45;focusoptions&#45;focusvisible&#45;2"></a>[focusVisible](#dom-focusoptions-focusvisible) &#125;&#41;</code></strong>

Moves the focus to <var>element</var>&#46;

If <var>element</var> is a <a id="focus&#45;management&#45;apis&#58;navigable&#45;container"></a>[navigable container](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container)&#44; moves the focus to its <a id="focus&#45;management&#45;apis&#58;content&#45;navigable"></a>[content navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable) instead&#46;

By default&#44; this method also scrolls <var>element</var> into view&#46; Providing the <a id="focus&#45;management&#45;apis&#58;dom&#45;focusoptions&#45;preventscroll&#45;3"></a><code>[preventScroll](#dom-focusoptions-preventscroll)</code> option and setting it to true prevents this behavior&#46;

By default&#44; user agents use <a id="focus&#45;management&#45;apis&#58;implementation&#45;defined"></a>[implementation&#45;defined](https://infra.spec.whatwg.org/#implementation-defined) heuristics to determine whether to <a id="focus&#45;management&#45;apis&#58;indicate&#45;focus"></a>[indicate focus](https://drafts.csswg.org/selectors/#indicate-focus) via a focus ring&#46; Providing the <a id="focus&#45;management&#45;apis&#58;dom&#45;focusoptions&#45;focusvisible&#45;3"></a><code>[focusVisible](#dom-focusoptions-focusvisible)</code> option and setting it to true will ensure the focus ring is always visible&#46;

<strong><code><var>element</var>&#46;<a id="dom&#45;blur&#45;dev"></a>[blur](#dom-blur)&#40;&#41;</code></strong>

<strong>✔</strong>MDN

[HTMLElement&#47;blur](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/blur)

Support in all current engines&#46;

Firefox1&#46;5&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera8&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer5&#46;5&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android10&#46;1&#43;

Moves the focus to the <a id="focus&#45;management&#45;apis&#58;viewport"></a>[viewport](https://drafts.csswg.org/css2/#viewport)&#46; Use of this method is discouraged&#59; if you want to focus the <a id="focus&#45;management&#45;apis&#58;viewport&#45;2"></a>[viewport](https://drafts.csswg.org/css2/#viewport)&#44; call the <a id="focus&#45;management&#45;apis&#58;dom&#45;focus&#45;2"></a><code>[focus&#40;&#41;](#dom-focus)</code> method on the <a id="focus&#45;management&#45;apis&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#39;s <a id="focus&#45;management&#45;apis&#58;document&#45;element"></a>[document element](https://dom.spec.whatwg.org/#document-element)&#46;

Do not use this method to hide the focus ring if you find the focus ring unsightly&#46; Instead&#44; use the <a id="focus&#45;management&#45;apis&#58;&#58;focus&#45;visible"></a><code>[&#58;focus&#45;visible](https://drafts.csswg.org/selectors/#the-focus-visible-pseudo)</code> pseudo&#45;class to override the <a id="focus&#45;management&#45;apis&#58;&#39;outline&#39;"></a>[&#39;outline&#39;](https://drafts.csswg.org/css-ui/#outline) property&#44; and provide a different way to show what element is focused&#46; Be aware that if an alternative focusing style isn&#39;t made available&#44; the page will be significantly less usable for people who primarily navigate pages using a keyboard&#44; or those with reduced vision who use focus outlines to help them navigate the page&#46;

<strong>Example&#58;</strong>

For example&#44; to hide the outline from <a id="focus&#45;management&#45;apis&#58;the&#45;textarea&#45;element"></a><code>[textarea](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-textarea-element)</code> elements and instead use a yellow background to indicate focus&#44; you could use&#58;

``` text
textarea:focus-visible { outline: none; background: yellow; color: black; }
```

<a id="dom&#45;document&#45;activeelement"></a>The <a id="focus&#45;management&#45;apis&#58;documentorshadowroot"></a><code>[DocumentOrShadowRoot](html-language-foundations--877d6146487f--b55e73c31978.md#documentorshadowroot)</code> <a id="dom&#45;documentorshadowroot&#45;activeelement"></a><strong><code>activeElement</code></strong> getter steps are&#58;

1.  Let <var>candidate</var> be <a id="focus&#45;management&#45;apis&#58;this"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="focus&#45;management&#45;apis&#58;node&#45;document&#45;2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#39;s <a id="focus&#45;management&#45;apis&#58;focused&#45;area&#45;of&#45;the&#45;document"></a>[focused area](#focused-area-of-the-document)&#39;s <a id="focus&#45;management&#45;apis&#58;dom&#45;anchor"></a>[DOM anchor](#dom-anchor)&#46;

2.  Set <var>candidate</var> to the result of <a id="focus&#45;management&#45;apis&#58;dom&#45;retarget"></a>[retargeting](https://dom.spec.whatwg.org/#retarget) <var>candidate</var> against <a id="focus&#45;management&#45;apis&#58;this&#45;2"></a>[this](https://webidl.spec.whatwg.org/#this)&#46;

3.  If <var>candidate</var>&#39;s <a id="focus&#45;management&#45;apis&#58;root"></a>[root](https://dom.spec.whatwg.org/#concept-tree-root) is not <a id="focus&#45;management&#45;apis&#58;this&#45;3"></a>[this](https://webidl.spec.whatwg.org/#this)&#44; then return null&#46;

4.  If <var>candidate</var> is not a <a id="focus&#45;management&#45;apis&#58;document&#45;2"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> object&#44; then return <var>candidate</var>&#46;

5.  If <var>candidate</var> has a <a id="focus&#45;management&#45;apis&#58;the&#45;body&#45;element&#45;2"></a>[body element](html-language-foundations--877d6146487f--b55e73c31978.md#the-body-element-2)&#44; then return that <a id="focus&#45;management&#45;apis&#58;the&#45;body&#45;element&#45;2&#45;2"></a>[body element](html-language-foundations--877d6146487f--b55e73c31978.md#the-body-element-2)&#46;

6.  If <var>candidate</var>&#39;s <a id="focus&#45;management&#45;apis&#58;document&#45;element&#45;2"></a>[document element](https://dom.spec.whatwg.org/#document-element) is non&#45;null&#44; then return that <a id="focus&#45;management&#45;apis&#58;document&#45;element&#45;3"></a>[document element](https://dom.spec.whatwg.org/#document-element)&#46;

7.  Return null&#46;

The <a id="focus&#45;management&#45;apis&#58;document&#45;3"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> <a id="dom&#45;document&#45;hasfocus"></a><strong><code>hasFocus&#40;&#41;</code></strong> method steps are to return the result of running the <a id="focus&#45;management&#45;apis&#58;has&#45;focus&#45;steps"></a>[has focus steps](#has-focus-steps) given <a id="focus&#45;management&#45;apis&#58;this&#45;4"></a>[this](https://webidl.spec.whatwg.org/#this)&#46;

------------------------------------------------------------------------

The <a id="focus&#45;management&#45;apis&#58;window"></a><code>[Window](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window)</code> <a id="dom&#45;window&#45;focus"></a><strong><code>focus&#40;&#41;</code></strong> method steps are&#58;

1.  Let <var>current</var> be <a id="focus&#45;management&#45;apis&#58;this&#45;5"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="focus&#45;management&#45;apis&#58;window&#45;navigable&#45;2"></a>[navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window-navigable)&#46;

2.  If <var>current</var> is null&#44; then return&#46;

3.  If the <a id="focus&#45;management&#45;apis&#58;allow&#45;focus&#45;steps"></a>[allow focus steps](#allow-focus-steps) given <var>current</var>&#39;s <a id="focus&#45;management&#45;apis&#58;nav&#45;document&#45;2"></a>[active document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document) return false&#44; then return&#46;

4.  Run the <a id="focus&#45;management&#45;apis&#58;focusing&#45;steps"></a>[focusing steps](#focusing-steps) with <var>current</var>&#46;

5.  If <var>current</var> is a <a id="focus&#45;management&#45;apis&#58;top&#45;level&#45;traversable"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable)&#44; user agents are encouraged to trigger some sort of notification to indicate to the user that the page is attempting to gain focus&#46;

<strong>✔</strong>MDN

[Window&#47;blur](https://developer.mozilla.org/en-US/docs/Web/API/Window/blur)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera12&#46;1&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer4&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android12&#46;1&#43;

The <a id="focus&#45;management&#45;apis&#58;window&#45;2"></a><code>[Window](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window)</code> <a id="dom&#45;window&#45;blur"></a><strong><code>blur&#40;&#41;</code></strong> method steps are to do nothing&#46;

<strong>Note&#58;</strong>

Historically&#44; the <a id="focus&#45;management&#45;apis&#58;dom&#45;window&#45;focus"></a><code>[focus&#40;&#41;](#dom-window-focus)</code> and <a id="focus&#45;management&#45;apis&#58;dom&#45;window&#45;blur"></a><code>[blur&#40;&#41;](#dom-window-blur)</code> methods actually affected the system&#45;level focus of the system widget &#40;e&#46;g&#46;&#44; tab or window&#41; that contained the <a id="focus&#45;management&#45;apis&#58;navigable"></a>[navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable)&#44; but hostile sites widely abuse this behavior to the user&#39;s detriment&#46;

------------------------------------------------------------------------

The <a id="focus&#45;management&#45;apis&#58;htmlorsvgelement"></a><code>[HTMLOrSVGOrMathMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlorsvgelement)</code> <a id="dom&#45;focus"></a><strong><code>focus&#40;<var>options</var>&#41;</code></strong> method steps are&#58;

1.  If the <a id="focus&#45;management&#45;apis&#58;allow&#45;focus&#45;steps&#45;2"></a>[allow focus steps](#allow-focus-steps) given <a id="focus&#45;management&#45;apis&#58;this&#45;6"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="focus&#45;management&#45;apis&#58;node&#45;document&#45;3"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document) return false&#44; then return&#46;

2.  Run the <a id="focus&#45;management&#45;apis&#58;focusing&#45;steps&#45;2"></a>[focusing steps](#focusing-steps) for <a id="focus&#45;management&#45;apis&#58;this&#45;7"></a>[this](https://webidl.spec.whatwg.org/#this)&#46;

3.  If <var>options</var>&#91;&#34;<a id="dom&#45;focusoptions&#45;focusvisible"></a><strong><code>focusVisible</code></strong>&#34;&#93; is true&#44; or does not <a id="focus&#45;management&#45;apis&#58;map&#45;exists"></a>[exist](https://infra.spec.whatwg.org/#map-exists) but in an <a id="focus&#45;management&#45;apis&#58;implementation&#45;defined&#45;2"></a>[implementation&#45;defined](https://infra.spec.whatwg.org/#implementation-defined) way the user agent determines it would be best to do so&#44; then <a id="focus&#45;management&#45;apis&#58;indicate&#45;focus&#45;2"></a>[indicate focus](https://drafts.csswg.org/selectors/#indicate-focus)&#46;

4.  If <var>options</var>&#91;&#34;<a id="dom&#45;focusoptions&#45;preventscroll"></a><strong><code>preventScroll</code></strong>&#34;&#93; is false&#44; then <a id="focus&#45;management&#45;apis&#58;scroll&#45;a&#45;target&#45;into&#45;view"></a>[scroll a target into view](https://drafts.csswg.org/cssom-view/#scroll-a-target-into-view) given <a id="focus&#45;management&#45;apis&#58;this&#45;8"></a>[this](https://webidl.spec.whatwg.org/#this)&#44; &#34;<code>auto</code>&#34;&#44; &#34;<code>center</code>&#34;&#44; and &#34;<code>center</code>&#34;&#46;

The <a id="focus&#45;management&#45;apis&#58;htmlorsvgelement&#45;2"></a><code>[HTMLOrSVGOrMathMLElement](html-language-foundations--877d6146487f--b55e73c31978.md#htmlorsvgelement)</code> <a id="dom&#45;blur"></a><strong><code>blur&#40;&#41;</code></strong> method steps are&#58;

1.  The user agent should run the <a id="focus&#45;management&#45;apis&#58;unfocusing&#45;steps"></a>[unfocusing steps](#unfocusing-steps) given <a id="focus&#45;management&#45;apis&#58;this&#45;9"></a>[this](https://webidl.spec.whatwg.org/#this)&#46;

    User agents may instead selectively or uniformly do nothing&#44; for usability reasons&#46;

<strong>Example&#58;</strong>

For example&#44; if the <a id="focus&#45;management&#45;apis&#58;dom&#45;blur"></a><code>[blur&#40;&#41;](#dom-blur)</code> method is unwisely being used to remove the focus ring for aesthetics reasons&#44; the page would become unusable by keyboard users&#46; Ignoring calls to this method would thus allow keyboard users to interact with the page&#46;

------------------------------------------------------------------------

The <a id="allow&#45;focus&#45;steps"></a><strong>allow focus steps</strong>&#44; given a <a id="focus&#45;management&#45;apis&#58;document&#45;4"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> object <var>target</var>&#44; are&#58;

1.  If <var>target</var> is <a id="focus&#45;management&#45;apis&#58;allowed&#45;to&#45;use"></a>[allowed to use](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#allowed-to-use) the &#34;<a id="focus&#45;management&#45;apis&#58;focus&#45;without&#45;user&#45;activation&#45;feature"></a><code>[focus&#45;without&#45;user&#45;activation](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#focus-without-user-activation-feature)</code>&#34; feature&#44; then return true&#46;

2.  If <var>target</var>&#39;s <a id="focus&#45;management&#45;apis&#58;concept&#45;relevant&#45;global"></a>[relevant global object](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global) has <a id="focus&#45;management&#45;apis&#58;transient&#45;activation"></a>[transient activation](#transient-activation)&#44; then return true&#46;

3.  Return false&#46;

#### <a id="the&#45;autofocus&#45;attribute"></a>6&#46;6&#46;7 <a id="autofocusing&#45;a&#45;form&#45;control&#58;&#45;the&#45;autofocus&#45;attribute"></a>The <a id="the&#45;autofocus&#45;attribute&#58;attr&#45;fe&#45;autofocus"></a><code>[autofocus](#attr-fe-autofocus)</code> attribute[](#the-autofocus-attribute)

The <a id="attr&#45;fe&#45;autofocus"></a><strong><code>autofocus</code></strong> content attribute allows the author to indicate that an element is to be focused as soon as the page is loaded&#44; allowing the user to just start typing without having to manually focus the main element&#46;

When the <a id="the&#45;autofocus&#45;attribute&#58;attr&#45;fe&#45;autofocus&#45;2"></a><code>[autofocus](#attr-fe-autofocus)</code> attribute is specified on an element inside <a id="the&#45;autofocus&#45;attribute&#58;the&#45;dialog&#45;element"></a><code>[dialog](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-dialog-element)</code> elements or <a id="the&#45;autofocus&#45;attribute&#58;html&#45;elements"></a>[HTML elements](html-language-foundations--877d6146487f--b55e73c31978.md#html-elements) whose <a id="the&#45;autofocus&#45;attribute&#58;attr&#45;popover"></a><code>[popover](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-popover)</code> attribute is set&#44; then it will be focused when the dialog or popover becomes shown&#46;

The <a id="the&#45;autofocus&#45;attribute&#58;attr&#45;fe&#45;autofocus&#45;3"></a><code>[autofocus](#attr-fe-autofocus)</code> attribute is a <a id="the&#45;autofocus&#45;attribute&#58;boolean&#45;attribute"></a>[boolean attribute](html-language-foundations--877d6146487f--b55e73c31978.md#boolean-attribute)&#46;

To find the <a id="nearest&#45;ancestor&#45;autofocus&#45;scoping&#45;root&#45;element"></a><strong>nearest ancestor autofocus scoping root element</strong> given an <a id="the&#45;autofocus&#45;attribute&#58;element"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code> <var>element</var>&#58;

1.  If <var>element</var> is a <a id="the&#45;autofocus&#45;attribute&#58;the&#45;dialog&#45;element&#45;2"></a><code>[dialog](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-dialog-element)</code> element&#44; then return <var>element</var>&#46;

2.  If <var>element</var>&#39;s <a id="the&#45;autofocus&#45;attribute&#58;attr&#45;popover&#45;2"></a><code>[popover](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-popover)</code> attribute is not in the <a id="the&#45;autofocus&#45;attribute&#58;attr&#45;popover&#45;none&#45;state"></a>[No Popover](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-popover-none-state) state&#44; then return <var>element</var>&#46;

3.  Let <var>ancestor</var> be <var>element</var>&#46;

4.  While <var>ancestor</var> has a <a id="the&#45;autofocus&#45;attribute&#58;parent&#45;element"></a>[parent element](https://dom.spec.whatwg.org/#parent-element)&#58;

    1.  Set <var>ancestor</var> to <var>ancestor</var>&#39;s <a id="the&#45;autofocus&#45;attribute&#58;parent&#45;element&#45;2"></a>[parent element](https://dom.spec.whatwg.org/#parent-element)&#46;

    2.  If <var>ancestor</var> is a <a id="the&#45;autofocus&#45;attribute&#58;the&#45;dialog&#45;element&#45;3"></a><code>[dialog](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-dialog-element)</code> element&#44; then return <var>ancestor</var>&#46;

    3.  If <var>ancestor</var>&#39;s <a id="the&#45;autofocus&#45;attribute&#58;attr&#45;popover&#45;3"></a><code>[popover](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-popover)</code> attribute is not in the <a id="the&#45;autofocus&#45;attribute&#58;attr&#45;popover&#45;none&#45;state&#45;2"></a>[No Popover](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-popover-none-state) state&#44; then return <var>ancestor</var>&#46;

5.  Return <var>ancestor</var>&#46;

There must not be two elements with the same <a id="the&#45;autofocus&#45;attribute&#58;nearest&#45;ancestor&#45;autofocus&#45;scoping&#45;root&#45;element"></a>[nearest ancestor autofocus scoping root element](#nearest-ancestor-autofocus-scoping-root-element) that both have the <a id="the&#45;autofocus&#45;attribute&#58;attr&#45;fe&#45;autofocus&#45;4"></a><code>[autofocus](#attr-fe-autofocus)</code> attribute specified&#46;

Each <a id="the&#45;autofocus&#45;attribute&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> has an <a id="autofocus&#45;candidates"></a><strong>autofocus candidates</strong> <a id="the&#45;autofocus&#45;attribute&#58;list"></a>[list](https://infra.spec.whatwg.org/#list)&#44; initially empty&#46;

Each <a id="the&#45;autofocus&#45;attribute&#58;document&#45;2"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> has an <a id="autofocus&#45;processed&#45;flag"></a><strong>autofocus processed flag</strong> boolean&#44; initially false&#46;

When an element with the <a id="the&#45;autofocus&#45;attribute&#58;attr&#45;fe&#45;autofocus&#45;5"></a><code>[autofocus](#attr-fe-autofocus)</code> attribute specified is <a id="the&#45;autofocus&#45;attribute&#58;insert&#45;an&#45;element&#45;into&#45;a&#45;document"></a>[inserted into a document](html-language-foundations--877d6146487f--b55e73c31978.md#insert-an-element-into-a-document)&#44; run the following steps&#58;

1.  If the user has indicated &#40;for example&#44; by starting to type in a form control&#41; that they do not wish focus to be changed&#44; then optionally return&#46;

2.  Let <var>target</var> be the element&#39;s <a id="the&#45;autofocus&#45;attribute&#58;node&#45;document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46;

3.  If <var>target</var> is not <a id="the&#45;autofocus&#45;attribute&#58;fully&#45;active"></a>[fully active](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fully-active)&#44; then return&#46;

4.  If <var>target</var>&#39;s <a id="the&#45;autofocus&#45;attribute&#58;active&#45;sandboxing&#45;flag&#45;set"></a>[active sandboxing flag set](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#active-sandboxing-flag-set) has the <a id="the&#45;autofocus&#45;attribute&#58;sandboxed&#45;automatic&#45;features&#45;browsing&#45;context&#45;flag"></a>[sandboxed automatic features browsing context flag](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-automatic-features-browsing-context-flag)&#44; then return&#46;

5.  If the <a id="the&#45;autofocus&#45;attribute&#58;allow&#45;focus&#45;steps"></a>[allow focus steps](#allow-focus-steps) given <var>target</var> return false&#44; then return&#46;

6.  Let <var>topDocument</var> be <var>target</var>&#39;s <a id="the&#45;autofocus&#45;attribute&#58;node&#45;navigable"></a>[node navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable)&#39;s <a id="the&#45;autofocus&#45;attribute&#58;nav&#45;top"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-top)&#39;s <a id="the&#45;autofocus&#45;attribute&#58;nav&#45;document"></a>[active document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document)&#46;

7.  If <var>topDocument</var>&#39;s <a id="the&#45;autofocus&#45;attribute&#58;autofocus&#45;processed&#45;flag"></a>[autofocus processed flag](#autofocus-processed-flag) is false&#44; then <a id="the&#45;autofocus&#45;attribute&#58;list&#45;remove"></a>[remove](https://infra.spec.whatwg.org/#list-remove) the element from <var>topDocument</var>&#39;s <a id="the&#45;autofocus&#45;attribute&#58;autofocus&#45;candidates"></a>[autofocus candidates](#autofocus-candidates)&#44; and <a id="the&#45;autofocus&#45;attribute&#58;list&#45;append"></a>[append](https://infra.spec.whatwg.org/#list-append) the element to <var>topDocument</var>&#39;s <a id="the&#45;autofocus&#45;attribute&#58;autofocus&#45;candidates&#45;2"></a>[autofocus candidates](#autofocus-candidates)&#46;

<strong>Note&#58;</strong>

We do not check if an element is a <a id="the&#45;autofocus&#45;attribute&#58;focusable&#45;area"></a>[focusable area](#focusable-area) before storing it in the <a id="the&#45;autofocus&#45;attribute&#58;autofocus&#45;candidates&#45;3"></a>[autofocus candidates](#autofocus-candidates) list&#44; because even if it is not a focusable area when it is inserted&#44; it could become one by the time <a id="the&#45;autofocus&#45;attribute&#58;flush&#45;autofocus&#45;candidates"></a>[flush autofocus candidates](#flush-autofocus-candidates) sees it&#46;

To <a id="flush&#45;autofocus&#45;candidates"></a><strong>flush autofocus candidates</strong> for a document <var>topDocument</var>&#44; run these steps&#58;

1.  If <var>topDocument</var>&#39;s <a id="the&#45;autofocus&#45;attribute&#58;autofocus&#45;processed&#45;flag&#45;2"></a>[autofocus processed flag](#autofocus-processed-flag) is true&#44; then return&#46;

2.  Let <var>candidates</var> be <var>topDocument</var>&#39;s <a id="the&#45;autofocus&#45;attribute&#58;autofocus&#45;candidates&#45;4"></a>[autofocus candidates](#autofocus-candidates)&#46;

3.  If <var>candidates</var> <a id="the&#45;autofocus&#45;attribute&#58;list&#45;is&#45;empty"></a>[is empty](https://infra.spec.whatwg.org/#list-is-empty)&#44; then return&#46;

4.  If <var>topDocument</var>&#39;s <a id="the&#45;autofocus&#45;attribute&#58;focused&#45;area&#45;of&#45;the&#45;document"></a>[focused area](#focused-area-of-the-document) is not <var>topDocument</var> itself&#44; or <var>topDocument</var> has non&#45;null <a id="the&#45;autofocus&#45;attribute&#58;target&#45;element"></a>[target element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#target-element)&#58;

    1.  <a id="the&#45;autofocus&#45;attribute&#58;list&#45;empty"></a>[Empty](https://infra.spec.whatwg.org/#list-empty) <var>candidates</var>&#46;

    2.  Set <var>topDocument</var>&#39;s <a id="the&#45;autofocus&#45;attribute&#58;autofocus&#45;processed&#45;flag&#45;3"></a>[autofocus processed flag](#autofocus-processed-flag) to true&#46;

    3.  Return&#46;

5.  While <var>candidates</var> is not <a id="the&#45;autofocus&#45;attribute&#58;list&#45;is&#45;empty&#45;2"></a>[empty](https://infra.spec.whatwg.org/#list-is-empty)&#58;

    1.  Let <var>element</var> be <var>candidates</var>&#91;0&#93;&#46;

    2.  Let <var>doc</var> be <var>element</var>&#39;s <a id="the&#45;autofocus&#45;attribute&#58;node&#45;document&#45;2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)&#46;

    3.  If <var>doc</var> is not <a id="the&#45;autofocus&#45;attribute&#58;fully&#45;active&#45;2"></a>[fully active](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fully-active)&#44; then <a id="the&#45;autofocus&#45;attribute&#58;list&#45;remove&#45;2"></a>[remove](https://infra.spec.whatwg.org/#list-remove) <var>element</var> from <var>candidates</var>&#44; and <a id="the&#45;autofocus&#45;attribute&#58;continue"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue)&#46;

    4.  If <var>doc</var>&#39;s <a id="the&#45;autofocus&#45;attribute&#58;node&#45;navigable&#45;2"></a>[node navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable)&#39;s <a id="the&#45;autofocus&#45;attribute&#58;nav&#45;top&#45;2"></a>[top&#45;level traversable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-top) is not the same as <var>topDocument</var>&#39;s <a id="the&#45;autofocus&#45;attribute&#58;node&#45;navigable&#45;3"></a>[node navigable](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable)&#44; then <a id="the&#45;autofocus&#45;attribute&#58;list&#45;remove&#45;3"></a>[remove](https://infra.spec.whatwg.org/#list-remove) <var>element</var> from <var>candidates</var>&#44; and <a id="the&#45;autofocus&#45;attribute&#58;continue&#45;2"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue)&#46;

    5.  If <var>doc</var>&#39;s <a id="the&#45;autofocus&#45;attribute&#58;script&#45;blocking&#45;style&#45;sheet&#45;set"></a>[script&#45;blocking style sheet set](html-language-elements-attributes--877d6146487f--2ca83b5ac984.md#script-blocking-style-sheet-set) is not <a id="the&#45;autofocus&#45;attribute&#58;list&#45;is&#45;empty&#45;3"></a>[empty](https://infra.spec.whatwg.org/#list-is-empty)&#44; then return&#46;

        <strong>Note&#58;</strong>

        In this case&#44; <var>element</var> is the currently&#45;best candidate&#44; but <var>doc</var> is not ready for autofocusing&#46; We&#39;ll try again next time <a id="the&#45;autofocus&#45;attribute&#58;flush&#45;autofocus&#45;candidates&#45;2"></a>[flush autofocus candidates](#flush-autofocus-candidates) is called&#46;

    6.  <a id="the&#45;autofocus&#45;attribute&#58;list&#45;remove&#45;4"></a>[Remove](https://infra.spec.whatwg.org/#list-remove) <var>element</var> from <var>candidates</var>&#46;

    7.  Let <var>inclusiveAncestorDocuments</var> be a <a id="the&#45;autofocus&#45;attribute&#58;list&#45;2"></a>[list](https://infra.spec.whatwg.org/#list) consisting of the <a id="the&#45;autofocus&#45;attribute&#58;nav&#45;document&#45;2"></a>[active document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document) of <var>doc</var>&#39;s <a id="the&#45;autofocus&#45;attribute&#58;inclusive&#45;ancestor&#45;navigables"></a>[inclusive ancestor navigables](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#inclusive-ancestor-navigables)&#46;

    8.  If any <a id="the&#45;autofocus&#45;attribute&#58;document&#45;3"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code> in <var>inclusiveAncestorDocuments</var> has non&#45;null <a id="the&#45;autofocus&#45;attribute&#58;target&#45;element&#45;2"></a>[target element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#target-element)&#44; then <a id="the&#45;autofocus&#45;attribute&#58;continue&#45;3"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue)&#46;

    9.  Let <var>target</var> be <var>element</var>&#46;

    10. If <var>target</var> is not a <a id="the&#45;autofocus&#45;attribute&#58;focusable&#45;area&#45;2"></a>[focusable area](#focusable-area)&#44; then set <var>target</var> to the result of <a id="the&#45;autofocus&#45;attribute&#58;get&#45;the&#45;focusable&#45;area"></a>[getting the focusable area](#get-the-focusable-area) for <var>target</var>&#46;

        <strong>Note&#58;</strong>

        <a id="the&#45;autofocus&#45;attribute&#58;autofocus&#45;candidates&#45;5"></a>[Autofocus candidates](#autofocus-candidates) can <a id="the&#45;autofocus&#45;attribute&#58;list&#45;contains"></a>[contain](https://infra.spec.whatwg.org/#list-contain) elements which are not <a id="the&#45;autofocus&#45;attribute&#58;focusable&#45;area&#45;3"></a>[focusable areas](#focusable-area)&#46; In addition to the special cases handled in the <a id="the&#45;autofocus&#45;attribute&#58;get&#45;the&#45;focusable&#45;area&#45;2"></a>[get the focusable area](#get-the-focusable-area) algorithm&#44; this can happen because a non&#45;<a id="the&#45;autofocus&#45;attribute&#58;focusable&#45;area&#45;4"></a>[focusable area](#focusable-area) element with an <a id="the&#45;autofocus&#45;attribute&#58;attr&#45;fe&#45;autofocus&#45;6"></a><code>[autofocus](#attr-fe-autofocus)</code> attribute was <a id="the&#45;autofocus&#45;attribute&#58;insert&#45;an&#45;element&#45;into&#45;a&#45;document&#45;2"></a>[inserted into a document](html-language-foundations--877d6146487f--b55e73c31978.md#insert-an-element-into-a-document) and it never became focusable&#44; or because the element was focusable but its status changed while it was stored in <a id="the&#45;autofocus&#45;attribute&#58;autofocus&#45;candidates&#45;6"></a>[autofocus candidates](#autofocus-candidates)&#46;

    11. If <var>target</var> is not null&#58;

        1.  <a id="the&#45;autofocus&#45;attribute&#58;list&#45;empty&#45;2"></a>[Empty](https://infra.spec.whatwg.org/#list-empty) <var>candidates</var>&#46;

        2.  Set <var>topDocument</var>&#39;s <a id="the&#45;autofocus&#45;attribute&#58;autofocus&#45;processed&#45;flag&#45;4"></a>[autofocus processed flag](#autofocus-processed-flag) to true&#46;

        3.  Run the <a id="the&#45;autofocus&#45;attribute&#58;focusing&#45;steps"></a>[focusing steps](#focusing-steps) for <var>target</var>&#46;

<strong>Note&#58;</strong>

This handles the automatic focusing during document load&#46; The <a id="the&#45;autofocus&#45;attribute&#58;dom&#45;dialog&#45;show"></a><code>[show&#40;&#41;](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#dom-dialog-show)</code> and <a id="the&#45;autofocus&#45;attribute&#58;dom&#45;dialog&#45;showmodal"></a><code>[showModal&#40;&#41;](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#dom-dialog-showmodal)</code> methods of <a id="the&#45;autofocus&#45;attribute&#58;the&#45;dialog&#45;element&#45;4"></a><code>[dialog](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-dialog-element)</code> elements also processes the <a id="the&#45;autofocus&#45;attribute&#58;attr&#45;fe&#45;autofocus&#45;7"></a><code>[autofocus](#attr-fe-autofocus)</code> attribute&#46;

<strong>Note&#58;</strong>

Focusing the element does not imply that the user agent has to focus the browser window if it has lost focus&#46;

<strong>Example&#58;</strong>

In the following snippet&#44; the text control would be focused when the document was loaded&#46;

``` text
<input maxlength="256" name="q" value="" autofocus>
<input type="submit" value="Search">
```

<strong>Example&#58;</strong>

The <a id="the&#45;autofocus&#45;attribute&#58;attr&#45;fe&#45;autofocus&#45;8"></a><code>[autofocus](#attr-fe-autofocus)</code> attribute applies to all elements&#44; not just to form controls&#46; This allows examples such as the following&#58;

``` text
<div contenteditable autofocus>Edit <strong>me!</strong><div>
```

### <a id="disabled&#45;elements"></a>4&#46;15 Disabled elements[](#disabled-elements)

An element is said to be <a id="concept&#45;element&#45;disabled"></a><strong>actually disabled</strong> if it is one of the following&#58;

- a <a id="disabled&#45;elements&#58;the&#45;button&#45;element"></a><code>[button](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-button-element)</code> element that is <a id="disabled&#45;elements&#58;concept&#45;fe&#45;disabled"></a>[disabled](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fe-disabled)
- an <a id="disabled&#45;elements&#58;the&#45;input&#45;element"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> element that is <a id="disabled&#45;elements&#58;concept&#45;fe&#45;disabled&#45;2"></a>[disabled](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fe-disabled)
- a <a id="disabled&#45;elements&#58;the&#45;select&#45;element"></a><code>[select](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-select-element)</code> element that is <a id="disabled&#45;elements&#58;concept&#45;fe&#45;disabled&#45;3"></a>[disabled](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fe-disabled)
- a <a id="disabled&#45;elements&#58;the&#45;textarea&#45;element"></a><code>[textarea](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-textarea-element)</code> element that is <a id="disabled&#45;elements&#58;concept&#45;fe&#45;disabled&#45;4"></a>[disabled](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fe-disabled)
- an <a id="disabled&#45;elements&#58;the&#45;optgroup&#45;element"></a><code>[optgroup](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-optgroup-element)</code> element that has a <a id="disabled&#45;elements&#58;attr&#45;optgroup&#45;disabled"></a><code>[disabled](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-optgroup-disabled)</code> attribute or whose <a id="disabled&#45;elements&#58;option&#45;element&#45;nearest&#45;ancestor&#45;select"></a>[nearest ancestor <code>select</code>](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#option-element-nearest-ancestor-select) is <a id="disabled&#45;elements&#58;concept&#45;fe&#45;disabled&#45;5"></a>[disabled](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fe-disabled)
- an <a id="disabled&#45;elements&#58;the&#45;option&#45;element"></a><code>[option](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-option-element)</code> element that is <a id="disabled&#45;elements&#58;concept&#45;option&#45;disabled"></a>[disabled](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-option-disabled) or whose <a id="disabled&#45;elements&#58;option&#45;element&#45;nearest&#45;ancestor&#45;select&#45;2"></a>[nearest ancestor <code>select</code>](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#option-element-nearest-ancestor-select) is <a id="disabled&#45;elements&#58;concept&#45;fe&#45;disabled&#45;6"></a>[disabled](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fe-disabled)
- a <a id="disabled&#45;elements&#58;the&#45;fieldset&#45;element"></a><code>[fieldset](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-fieldset-element)</code> element that is a <a id="disabled&#45;elements&#58;concept&#45;fieldset&#45;disabled"></a>[disabled fieldset](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fieldset-disabled)
- a <a id="disabled&#45;elements&#58;form&#45;associated&#45;custom&#45;element"></a>[form&#45;associated custom element](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#form-associated-custom-element) that is <a id="disabled&#45;elements&#58;concept&#45;fe&#45;disabled&#45;7"></a>[disabled](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fe-disabled)

<strong>Note&#58;</strong>

This definition is used to determine what elements are <a id="disabled&#45;elements&#58;focusable"></a>[focusable](#focusable) and which elements match the <a id="disabled&#45;elements&#58;selector&#45;enabled"></a><code>[&#58;enabled](#selector-enabled)</code> and <a id="disabled&#45;elements&#58;selector&#45;disabled"></a><code>[&#58;disabled](#selector-disabled)</code> <a id="disabled&#45;elements&#58;pseudo&#45;class"></a>[pseudo classes](https://drafts.csswg.org/selectors/#pseudo-class)&#46;

### <a id="selectors"></a>4&#46;16 Matching HTML elements using selectors and CSS[](#selectors)

#### <a id="case&#45;sensitivity&#45;of&#45;the&#45;css&#45;&#39;attr&#40;&#41;&#39;&#45;function"></a>4&#46;16&#46;1 Case&#45;sensitivity of the CSS <a id="case&#45;sensitivity&#45;of&#45;the&#45;css&#45;&#39;attr&#40;&#41;&#39;&#45;function&#58;&#39;attr&#40;&#41;&#39;"></a>[&#39;attr&#40;&#41;&#39;](https://drafts.csswg.org/css-values-5/#funcdef-attr) function[](#case-sensitivity-of-the-css-'attr()'-function)

CSS Values and Units leaves the case&#45;sensitivity of attribute names for the purpose of the <a id="case&#45;sensitivity&#45;of&#45;the&#45;css&#45;&#39;attr&#40;&#41;&#39;&#45;function&#58;&#39;attr&#40;&#41;&#39;&#45;2"></a>[&#39;attr&#40;&#41;&#39;](https://drafts.csswg.org/css-values-5/#funcdef-attr) function to be defined by the host language&#46; [&#91;CSSVALUES&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsCSSVALUES)

When comparing the attribute name part of a CSS <a id="case&#45;sensitivity&#45;of&#45;the&#45;css&#45;&#39;attr&#40;&#41;&#39;&#45;function&#58;&#39;attr&#40;&#41;&#39;&#45;3"></a>[&#39;attr&#40;&#41;&#39;](https://drafts.csswg.org/css-values-5/#funcdef-attr) function to the names of namespace&#45;less attributes on <a id="case&#45;sensitivity&#45;of&#45;the&#45;css&#45;&#39;attr&#40;&#41;&#39;&#45;function&#58;html&#45;elements"></a>[HTML elements](html-language-foundations--877d6146487f--b55e73c31978.md#html-elements) in <a id="case&#45;sensitivity&#45;of&#45;the&#45;css&#45;&#39;attr&#40;&#41;&#39;&#45;function&#58;html&#45;documents"></a>[HTML documents](https://dom.spec.whatwg.org/#html-document)&#44; the name part of the CSS <a id="case&#45;sensitivity&#45;of&#45;the&#45;css&#45;&#39;attr&#40;&#41;&#39;&#45;function&#58;&#39;attr&#40;&#41;&#39;&#45;4"></a>[&#39;attr&#40;&#41;&#39;](https://drafts.csswg.org/css-values-5/#funcdef-attr) function must first be <a id="case&#45;sensitivity&#45;of&#45;the&#45;css&#45;&#39;attr&#40;&#41;&#39;&#45;function&#58;converted&#45;to&#45;ascii&#45;lowercase"></a>[converted to ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase)&#46; The same function when compared to other attributes must be compared according to its original case&#46; In both cases&#44; to match&#44; the values must be <a id="case&#45;sensitivity&#45;of&#45;the&#45;css&#45;&#39;attr&#40;&#41;&#39;&#45;function&#58;identical&#45;to"></a>[identical to](https://infra.spec.whatwg.org/#string-is) each other &#40;and therefore the comparison is case sensitive&#41;&#46;

<strong>Note&#58;</strong>

This is the same as comparing the name part of a CSS <a id="case&#45;sensitivity&#45;of&#45;the&#45;css&#45;&#39;attr&#40;&#41;&#39;&#45;function&#58;attribute&#45;selector"></a>[attribute selector](https://drafts.csswg.org/selectors/#attribute-selector)&#44; specified in the next section&#46;

#### <a id="case&#45;sensitivity&#45;of&#45;selectors"></a>4&#46;16&#46;2 Case&#45;sensitivity of selectors[](#case-sensitivity-of-selectors)

Selectors leaves the case&#45;sensitivity of element names&#44; attribute names&#44; and attribute values to be defined by the host language&#46; [&#91;SELECTORS&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsSELECTORS)

When comparing a CSS element <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;type&#45;selector"></a>[type selector](https://drafts.csswg.org/selectors/#type-selector) to the names of <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;html&#45;elements"></a>[HTML elements](html-language-foundations--877d6146487f--b55e73c31978.md#html-elements) in <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;html&#45;documents"></a>[HTML documents](https://dom.spec.whatwg.org/#html-document)&#44; the CSS element <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;type&#45;selector&#45;2"></a>[type selector](https://drafts.csswg.org/selectors/#type-selector) must first be <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;converted&#45;to&#45;ascii&#45;lowercase"></a>[converted to ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase)&#46; The same selector when compared to other elements must be compared according to its original case&#46; In both cases&#44; to match&#44; the values must be <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;identical&#45;to"></a>[identical to](https://infra.spec.whatwg.org/#string-is) each other &#40;and therefore the comparison is case sensitive&#41;&#46;

When comparing the name part of a CSS <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;attribute&#45;selector"></a>[attribute selector](https://drafts.csswg.org/selectors/#attribute-selector) to the names of attributes on <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;html&#45;elements&#45;2"></a>[HTML elements](html-language-foundations--877d6146487f--b55e73c31978.md#html-elements) in <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;html&#45;documents&#45;2"></a>[HTML documents](https://dom.spec.whatwg.org/#html-document)&#44; the name part of the CSS <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;attribute&#45;selector&#45;2"></a>[attribute selector](https://drafts.csswg.org/selectors/#attribute-selector) must first be <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;converted&#45;to&#45;ascii&#45;lowercase&#45;2"></a>[converted to ASCII lowercase](https://infra.spec.whatwg.org/#ascii-lowercase)&#46; The same selector when compared to other attributes must be compared according to its original case&#46; In both cases&#44; the comparison is case&#45;sensitive&#46;

<a id="case&#45;sensitivity&#45;of&#45;selectors&#58;attribute&#45;selector&#45;3"></a>[Attribute selectors](https://drafts.csswg.org/selectors/#attribute-selector) on an <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;html&#45;elements&#45;3"></a>[HTML element](html-language-foundations--877d6146487f--b55e73c31978.md#html-elements) in an <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;html&#45;documents&#45;3"></a>[HTML document](https://dom.spec.whatwg.org/#html-document) must treat the <em>values</em> of attributes with the following names as <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;ascii&#45;case&#45;insensitive"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive)&#58;

- <code>accept</code>
- <code>accept&#45;charset</code>
- <code>align</code>
- <code>alink</code>
- <code>axis</code>
- <code>bgcolor</code>
- <code>charset</code>
- <code>checked</code>
- <code>clear</code>
- <code>codetype</code>
- <code>color</code>
- <code>compact</code>
- <code>declare</code>
- <code>defer</code>
- <code>dir</code>
- <code>direction</code>
- <code>disabled</code>
- <code>enctype</code>
- <code>face</code>
- <code>frame</code>
- <code>hreflang</code>
- <code>http&#45;equiv</code>
- <code>lang</code>
- <code>language</code>
- <code>link</code>
- <code>media</code>
- <code>method</code>
- <code>multiple</code>
- <code>nohref</code>
- <code>noresize</code>
- <code>noshade</code>
- <code>nowrap</code>
- <code>readonly</code>
- <code>rel</code>
- <code>rev</code>
- <code>rules</code>
- <code>scope</code>
- <code>scrolling</code>
- <code>selected</code>
- <code>shape</code>
- <code>target</code>
- <code>text</code>
- <code>type</code>
- <code>valign</code>
- <code>valuetype</code>
- <code>vlink</code>

<strong>Example&#58;</strong>

For example&#44; the selector <code>&#91;bgcolor&#61;&#34;&#35;ffffff&#34;&#93;</code> will match any HTML element with a <code>bgcolor</code> attribute with values including <code>&#35;ffffff</code>&#44; <code>&#35;FFFFFF</code> and <code>&#35;fffFFF</code>&#46; This happens even if <code>bgcolor</code> has no effect for a given element &#40;e&#46;g&#46;&#44; <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;the&#45;div&#45;element"></a><code>[div](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-div-element)</code>&#41;&#46;

The selector <code>&#91;type&#61;a s&#93;</code> will match any HTML element with a <code>type</code> attribute whose value is <code>a</code>&#44; but not whose value is <code>A</code>&#44; due to the <code>s</code> flag&#46;

All other attribute values and everything else must be treated as entirely <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;identical&#45;to&#45;2"></a>[identical to](https://infra.spec.whatwg.org/#string-is) each other for the purposes of selector matching&#46; This includes&#58;

- <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;concept&#45;id"></a>[IDs](https://dom.spec.whatwg.org/#concept-id) and <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;concept&#45;class"></a>[classes](https://dom.spec.whatwg.org/#concept-class) in <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;no&#45;quirks&#45;mode"></a>[no&#45;quirks mode](https://dom.spec.whatwg.org/#concept-document-no-quirks) and <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;limited&#45;quirks&#45;mode"></a>[limited&#45;quirks mode](https://dom.spec.whatwg.org/#concept-document-limited-quirks)

- the names of elements not in the <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;html&#45;namespace&#45;2"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace)

- the names of <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;html&#45;elements&#45;4"></a>[HTML elements](html-language-foundations--877d6146487f--b55e73c31978.md#html-elements) in <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;xml&#45;documents"></a>[XML documents](https://dom.spec.whatwg.org/#xml-document)

- the names of attributes of elements not in the <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;html&#45;namespace&#45;2&#45;2"></a>[HTML namespace](https://infra.spec.whatwg.org/#html-namespace)

- the names of attributes of <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;html&#45;elements&#45;5"></a>[HTML elements](html-language-foundations--877d6146487f--b55e73c31978.md#html-elements) in <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;xml&#45;documents&#45;2"></a>[XML documents](https://dom.spec.whatwg.org/#xml-document)

- the names of attributes that themselves have namespaces

<strong>Note&#58;</strong>

Selectors defines that ID and class selectors &#40;such as <code>&#35;foo</code> and <code>&#46;bar</code>&#41;&#44; when matched against elements in documents that are in <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;quirks&#45;mode"></a>[quirks mode](https://dom.spec.whatwg.org/#concept-document-quirks)&#44; will be matched in an <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;ascii&#45;case&#45;insensitive&#45;2"></a>[ASCII case&#45;insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) manner&#46; However&#44; this does not apply for attribute selectors with &#34;<code>id</code>&#34; or &#34;<code>class</code>&#34; as the name part&#46; The selector <code>&#91;class&#61;&#34;foobar&#34;&#93;</code> will treat its value as case&#45;sensitive even in <a id="case&#45;sensitivity&#45;of&#45;selectors&#58;quirks&#45;mode&#45;2"></a>[quirks mode](https://dom.spec.whatwg.org/#concept-document-quirks)&#46;

#### <a id="pseudo&#45;classes"></a>4&#46;16&#46;3 Pseudo&#45;classes[](#pseudo-classes)

MDN

[Pseudo&#45;classes](https://developer.mozilla.org/en-US/docs/Web/CSS/Pseudo-classes)

There are a number of dynamic selectors that can be used with HTML&#46; This section defines when these selectors match HTML elements&#46; [&#91;SELECTORS&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsSELECTORS) [&#91;CSSUI&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsCSSUI)

<strong><a id="selector&#45;defined"></a><strong><code>&#58;defined</code></strong></strong>

<strong>✔</strong>MDN

[&#58;defined](https://developer.mozilla.org/en-US/docs/Web/CSS/:defined)

Support in all current engines&#46;

Firefox63&#43;Safari10&#43;Chrome54&#43;

------------------------------------------------------------------------

Opera&#63;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

The <a id="pseudo&#45;classes&#58;selector&#45;defined"></a><code>[&#58;defined](#selector-defined)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element that is <a id="pseudo&#45;classes&#58;concept&#45;element&#45;defined"></a>[defined](https://dom.spec.whatwg.org/#concept-element-defined)&#46;

<strong><a id="selector&#45;link"></a><strong><code>&#58;link</code></strong></strong>

<strong>✔</strong>MDN

[&#58;link](https://developer.mozilla.org/en-US/docs/Web/CSS/:link)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera3&#46;5&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer3&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS3&#46;2&#43;Chrome Android&#63;WebView Android1&#46;5&#43;Samsung Internet&#63;Opera Android&#63;

<strong><a id="selector&#45;visited"></a><strong><code>&#58;visited</code></strong></strong>

<strong>✔</strong>MDN

[&#58;visited](https://developer.mozilla.org/en-US/docs/Web/CSS/:visited)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera3&#46;5&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer4&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android10&#46;1&#43;

All <a id="pseudo&#45;classes&#58;the&#45;a&#45;element"></a><code>[a](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element)</code> elements that have an <a id="pseudo&#45;classes&#58;attr&#45;hyperlink&#45;href"></a><code>[href](html-language-elements-text-links--877d6146487f--8833a85fd242.md#attr-hyperlink-href)</code> attribute&#44; and all <a id="pseudo&#45;classes&#58;the&#45;area&#45;element"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> elements that have an <a id="pseudo&#45;classes&#58;attr&#45;hyperlink&#45;href&#45;2"></a><code>[href](html-language-elements-text-links--877d6146487f--8833a85fd242.md#attr-hyperlink-href)</code> attribute&#44; must match one of <a id="pseudo&#45;classes&#58;selector&#45;link"></a><code>[&#58;link](#selector-link)</code> and <a id="pseudo&#45;classes&#58;selector&#45;visited"></a><code>[&#58;visited](#selector-visited)</code>&#46;

Other specifications might apply more specific rules regarding how these elements are to match these <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;2"></a>[pseudo&#45;classes](https://drafts.csswg.org/selectors/#pseudo-class)&#44; to mitigate some privacy concerns that apply with straightforward implementations of this requirement&#46;

<strong><a id="selector&#45;active"></a><strong><code>&#58;active</code></strong></strong>

<strong>✔</strong>MDN

[&#58;active](https://developer.mozilla.org/en-US/docs/Web/CSS/:active)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera5&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer4&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android10&#46;1&#43;

The <a id="pseudo&#45;classes&#58;selector&#45;active"></a><code>[&#58;active](#selector-active)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;3"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) is defined to match an element <q>while an element is <a id="concept&#45;selector&#45;active"></a><strong><em>being activated</em></strong> by the user</q>&#46;

To determine whether a particular element is <a id="pseudo&#45;classes&#58;concept&#45;selector&#45;active"></a><em>[being activated](#concept-selector-active)</em> for the purposes of defining the <a id="pseudo&#45;classes&#58;selector&#45;active&#45;2"></a><code>[&#58;active](#selector-active)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;4"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) only&#44; an HTML user agent must use the first relevant entry in the following list&#46;

<strong>If the element is a <a id="pseudo&#45;classes&#58;the&#45;button&#45;element"></a><code>[button](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-button-element)</code> element</strong>

<strong>If the element is an <a id="pseudo&#45;classes&#58;the&#45;input&#45;element"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> element whose <a id="pseudo&#45;classes&#58;attr&#45;input&#45;type"></a><code>[type](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-type)</code> attribute is in the <a id="pseudo&#45;classes&#58;submit&#45;button&#45;state&#45;&#40;type&#61;submit&#41;"></a>[Submit Button](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#submit-button-state-(type=submit))&#44; <a id="pseudo&#45;classes&#58;image&#45;button&#45;state&#45;&#40;type&#61;image&#41;"></a>[Image Button](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#image-button-state-(type=image))&#44; <a id="pseudo&#45;classes&#58;reset&#45;button&#45;state&#45;&#40;type&#61;reset&#41;"></a>[Reset Button](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#reset-button-state-(type=reset))&#44; or <a id="pseudo&#45;classes&#58;button&#45;state&#45;&#40;type&#61;button&#41;"></a>[Button](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#button-state-(type=button)) state</strong>

<strong>If the element is an <a id="pseudo&#45;classes&#58;the&#45;a&#45;element&#45;2"></a><code>[a](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-a-element)</code> element that has an <a id="pseudo&#45;classes&#58;attr&#45;hyperlink&#45;href&#45;3"></a><code>[href](html-language-elements-text-links--877d6146487f--8833a85fd242.md#attr-hyperlink-href)</code> attribute</strong>

<strong>If the element is an <a id="pseudo&#45;classes&#58;the&#45;area&#45;element&#45;2"></a><code>[area](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#the-area-element)</code> element that has an <a id="pseudo&#45;classes&#58;attr&#45;hyperlink&#45;href&#45;4"></a><code>[href](html-language-elements-text-links--877d6146487f--8833a85fd242.md#attr-hyperlink-href)</code> attribute</strong>

<strong>If the element is <a id="pseudo&#45;classes&#58;focusable"></a>[focusable](#focusable)</strong>

The element is <a id="pseudo&#45;classes&#58;concept&#45;selector&#45;active&#45;2"></a><em>[being activated](#concept-selector-active)</em> if it is <a id="pseudo&#45;classes&#58;in&#45;a&#45;formal&#45;activation&#45;state"></a>[in a formal activation state](#in-a-formal-activation-state)&#46;

<strong>Example&#58;</strong>

For example&#44; if the user is using a keyboard to push a <a id="pseudo&#45;classes&#58;the&#45;button&#45;element&#45;2"></a><code>[button](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-button-element)</code> element by pressing the space bar&#44; the element would match this <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;5"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) in between the time that the element received the <a id="pseudo&#45;classes&#58;event&#45;keydown"></a><code>[keydown](https://w3c.github.io/uievents/#event-type-keydown)</code> event and the time the element received the <a id="pseudo&#45;classes&#58;event&#45;keyup"></a><code>[keyup](https://w3c.github.io/uievents/#event-type-keyup)</code> event&#46;

<strong>If the element is <a id="pseudo&#45;classes&#58;being&#45;actively&#45;pointed&#45;at"></a>[being actively pointed at](#being-actively-pointed-at)</strong>

The element is <a id="pseudo&#45;classes&#58;concept&#45;selector&#45;active&#45;3"></a><em>[being activated](#concept-selector-active)</em>&#46;

An element is said to be <a id="in&#45;a&#45;formal&#45;activation&#45;state"></a><strong>in a formal activation state</strong> between the time the user begins to indicate an intent to trigger the element&#39;s <a id="pseudo&#45;classes&#58;activation&#45;behaviour"></a>[activation behavior](https://dom.spec.whatwg.org/#eventtarget-activation-behavior) and either the time the user stops indicating an intent to trigger the element&#39;s <a id="pseudo&#45;classes&#58;activation&#45;behaviour&#45;2"></a>[activation behavior](https://dom.spec.whatwg.org/#eventtarget-activation-behavior)&#44; or the time the element&#39;s <a id="pseudo&#45;classes&#58;activation&#45;behaviour&#45;3"></a>[activation behavior](https://dom.spec.whatwg.org/#eventtarget-activation-behavior) has finished running&#44; which ever comes first&#46;

An element is said to be <a id="being&#45;actively&#45;pointed&#45;at"></a><strong>being actively pointed at</strong> while the user indicates the element using a pointing device while that pointing device is in the &#34;down&#34; state &#40;e&#46;g&#46; for a mouse&#44; between the time the mouse button is pressed and the time it is released&#59; for a finger in a multitouch environment&#44; while the finger is touching the display surface&#41;&#46;

<strong>Note&#58;</strong>

Per the definition in Selectors&#44; <a id="pseudo&#45;classes&#58;selector&#45;active&#45;3"></a><code>[&#58;active](#selector-active)</code> also matches <a id="pseudo&#45;classes&#58;flat&#45;tree"></a>[flat tree](https://drafts.csswg.org/css-scoping/#flat-tree) ancestors of elements that are <a id="pseudo&#45;classes&#58;concept&#45;selector&#45;active&#45;4"></a><em>[being activated](#concept-selector-active)</em>&#46; [&#91;SELECTORS&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsSELECTORS)

Additionally&#44; any element that is the <a id="pseudo&#45;classes&#58;labeled&#45;control"></a>[labeled control](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#labeled-control) of a <a id="pseudo&#45;classes&#58;the&#45;label&#45;element"></a><code>[label](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-label-element)</code> element that is currently matching <a id="pseudo&#45;classes&#58;selector&#45;active&#45;4"></a><code>[&#58;active](#selector-active)</code>&#44; also matches <a id="pseudo&#45;classes&#58;selector&#45;active&#45;5"></a><code>[&#58;active](#selector-active)</code>&#46; &#40;But&#44; it does not count as being <a id="pseudo&#45;classes&#58;concept&#45;selector&#45;active&#45;5"></a><em>[being activated](#concept-selector-active)</em>&#46;&#41;

<strong><a id="selector&#45;hover"></a><strong><code>&#58;hover</code></strong></strong>

<strong>✔</strong>MDN

[&#58;hover](https://developer.mozilla.org/en-US/docs/Web/CSS/:hover)

Support in all current engines&#46;

Firefox1&#43;Safari2&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera4&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer4&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android10&#46;1&#43;

The <a id="pseudo&#45;classes&#58;selector&#45;hover"></a><code>[&#58;hover](#selector-hover)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;6"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) is defined to match an element <q>while the user <a id="concept&#45;selector&#45;hover"></a><strong><em>designates</em></strong> an element with a pointing device</q>&#46; For the purposes of defining the <a id="pseudo&#45;classes&#58;selector&#45;hover&#45;2"></a><code>[&#58;hover](#selector-hover)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;7"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) only&#44; an HTML user agent must consider an element as being one that the user <a id="pseudo&#45;classes&#58;concept&#45;selector&#45;hover"></a><em>[designates](#concept-selector-hover)</em> if it is an element that the user indicates using a pointing device&#46;

<strong>Note&#58;</strong>

Per the definition in Selectors&#44; <a id="pseudo&#45;classes&#58;selector&#45;hover&#45;3"></a><code>[&#58;hover](#selector-hover)</code> also matches <a id="pseudo&#45;classes&#58;flat&#45;tree&#45;2"></a>[flat tree](https://drafts.csswg.org/css-scoping/#flat-tree) ancestors of elements that are <a id="pseudo&#45;classes&#58;concept&#45;selector&#45;hover&#45;2"></a><em>[designated](#concept-selector-hover)</em>&#46; [&#91;SELECTORS&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsSELECTORS)

Additionally&#44; any element that is the <a id="pseudo&#45;classes&#58;labeled&#45;control&#45;2"></a>[labeled control](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#labeled-control) of a <a id="pseudo&#45;classes&#58;the&#45;label&#45;element&#45;2"></a><code>[label](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-label-element)</code> element that is currently matching <a id="pseudo&#45;classes&#58;selector&#45;hover&#45;4"></a><code>[&#58;hover](#selector-hover)</code>&#44; also matches <a id="pseudo&#45;classes&#58;selector&#45;hover&#45;5"></a><code>[&#58;hover](#selector-hover)</code>&#46; &#40;But&#44; it does not count as being <a id="pseudo&#45;classes&#58;concept&#45;selector&#45;hover&#45;3"></a><em>[designated](#concept-selector-hover)</em>&#46;&#41;

<strong>Example&#58;</strong>

Consider in particular a fragment such as&#58;

``` text
<p> <label for=c> <input id=a> </label> <span id=b> <input id=c> </span> </p>
```

If the user designates the element with ID &#34;<code>a</code>&#34; with their pointing device&#44; then the <a id="pseudo&#45;classes&#58;the&#45;p&#45;element"></a><code>[p](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-p-element)</code> element &#40;and all its ancestors not shown in the snippet above&#41;&#44; the <a id="pseudo&#45;classes&#58;the&#45;label&#45;element&#45;3"></a><code>[label](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-label-element)</code> element&#44; the element with ID &#34;<code>a</code>&#34;&#44; and the element with ID &#34;<code>c</code>&#34; will match the <a id="pseudo&#45;classes&#58;selector&#45;hover&#45;6"></a><code>[&#58;hover](#selector-hover)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;8"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class)&#46; The element with ID &#34;<code>a</code>&#34; matches it by being <a id="pseudo&#45;classes&#58;concept&#45;selector&#45;hover&#45;4"></a><em>[designated](#concept-selector-hover)</em>&#59; the <a id="pseudo&#45;classes&#58;the&#45;label&#45;element&#45;4"></a><code>[label](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-label-element)</code> and <a id="pseudo&#45;classes&#58;the&#45;p&#45;element&#45;2"></a><code>[p](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-p-element)</code> elements match it because of the condition in Selectors about flat tree ancestors&#59; and the element with ID &#34;<code>c</code>&#34; matches it through the additional condition above on <a id="pseudo&#45;classes&#58;labeled&#45;control&#45;3"></a>[labeled controls](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#labeled-control) &#40;i&#46;e&#46;&#44; its <a id="pseudo&#45;classes&#58;the&#45;label&#45;element&#45;5"></a><code>[label](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-label-element)</code> element matches <a id="pseudo&#45;classes&#58;selector&#45;hover&#45;7"></a><code>[&#58;hover](#selector-hover)</code>&#41;&#46; However&#44; the element with ID &#34;<code>b</code>&#34; does <em>not</em> match <a id="pseudo&#45;classes&#58;selector&#45;hover&#45;8"></a><code>[&#58;hover](#selector-hover)</code>&#58; its flat tree descendant is not designated&#44; even though that flat tree descendant matches <a id="pseudo&#45;classes&#58;selector&#45;hover&#45;9"></a><code>[&#58;hover](#selector-hover)</code>&#46;

<strong><a id="selector&#45;focus"></a><strong><code>&#58;focus</code></strong></strong>

<strong>✔</strong>MDN

[&#58;focus](https://developer.mozilla.org/en-US/docs/Web/CSS/:focus)

Support in all current engines&#46;

Firefox1&#43;Safari1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera7&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer8&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android10&#46;1&#43;

For the purposes of the CSS <a id="pseudo&#45;classes&#58;selector&#45;focus"></a><code>[&#58;focus](#selector-focus)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;9"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class)&#44; an <a id="element&#45;has&#45;the&#45;focus"></a><strong>element has the focus</strong> when&#58;

- it is not itself a <a id="pseudo&#45;classes&#58;navigable&#45;container"></a>[navigable container](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container)&#59; and

- any of the following are true&#58;

  - it is one of the elements listed in the <a id="pseudo&#45;classes&#58;current&#45;focus&#45;chain&#45;of&#45;a&#45;top&#45;level&#45;traversable"></a>[current focus chain of the top&#45;level traversable](#current-focus-chain-of-a-top-level-traversable)&#59; or

  - its <a id="pseudo&#45;classes&#58;concept&#45;element&#45;shadow&#45;root"></a>[shadow root](https://dom.spec.whatwg.org/#concept-element-shadow-root) <var>shadowRoot</var> is not null and <var>shadowRoot</var> is the <a id="pseudo&#45;classes&#58;root"></a>[root](https://dom.spec.whatwg.org/#concept-tree-root) of at least one element that <a id="pseudo&#45;classes&#58;element&#45;has&#45;the&#45;focus"></a>[has the focus](#element-has-the-focus)&#46;

<strong><a id="selector&#45;target"></a><strong><code>&#58;target</code></strong></strong>

<strong>✔</strong>MDN

[&#58;target](https://developer.mozilla.org/en-US/docs/Web/CSS/:target)

Support in all current engines&#46;

Firefox1&#43;Safari1&#46;3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera9&#46;5&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer9&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS2&#43;Chrome Android&#63;WebView Android2&#43;Samsung Internet&#63;Opera Android10&#46;1&#43;

For the purposes of the CSS <a id="pseudo&#45;classes&#58;selector&#45;target"></a><code>[&#58;target](#selector-target)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;10"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class)&#44; the <a id="pseudo&#45;classes&#58;document"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#39;s <em>target elements</em> are a list containing the <a id="pseudo&#45;classes&#58;document&#45;2"></a><code>[Document](html-language-foundations--877d6146487f--b55e73c31978.md#document)</code>&#39;s <a id="pseudo&#45;classes&#58;target&#45;element"></a>[target element](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#target-element)&#44; if it is not null&#44; or containing no elements&#44; if it is&#46; [&#91;SELECTORS&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsSELECTORS)

<strong><a id="selector&#45;popover&#45;open"></a><strong><code>&#58;popover&#45;open</code></strong></strong>

<strong>✔</strong>MDN

[&#58;popover&#45;open](https://developer.mozilla.org/en-US/docs/Web/CSS/:popover-open)

Support in all current engines&#46;

Firefox🔰 114&#43;Safaripreview&#43;Chrome114&#43;

------------------------------------------------------------------------

Opera&#63;Edge114&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

The <a id="pseudo&#45;classes&#58;selector&#45;popover&#45;open"></a><code>[&#58;popover&#45;open](#selector-popover-open)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;11"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) is defined to match any <a id="pseudo&#45;classes&#58;html&#45;elements"></a>[HTML element](html-language-foundations--877d6146487f--b55e73c31978.md#html-elements) whose <a id="pseudo&#45;classes&#58;attr&#45;popover"></a><code>[popover](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-popover)</code> attribute is not in the <a id="pseudo&#45;classes&#58;attr&#45;popover&#45;none&#45;state"></a>[No Popover](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-popover-none-state) state and whose <a id="pseudo&#45;classes&#58;popover&#45;visibility&#45;state"></a>[popover visibility state](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#popover-visibility-state) is <a id="pseudo&#45;classes&#58;popover&#45;showing&#45;state"></a>[showing](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#popover-showing-state)&#46;

<strong><a id="selector&#45;enabled"></a><strong><code>&#58;enabled</code></strong></strong>

<strong>✔</strong>MDN

[&#58;enabled](https://developer.mozilla.org/en-US/docs/Web/CSS/:enabled)

Support in all current engines&#46;

Firefox1&#43;Safari3&#46;1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera9&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer9&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android2&#43;Samsung Internet&#63;Opera Android10&#46;1&#43;

The <a id="pseudo&#45;classes&#58;selector&#45;enabled"></a><code>[&#58;enabled](#selector-enabled)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;12"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match any <a id="pseudo&#45;classes&#58;the&#45;button&#45;element&#45;3"></a><code>[button](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-button-element)</code>&#44; <a id="pseudo&#45;classes&#58;the&#45;input&#45;element&#45;2"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code>&#44; <a id="pseudo&#45;classes&#58;the&#45;select&#45;element"></a><code>[select](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-select-element)</code>&#44; <a id="pseudo&#45;classes&#58;the&#45;textarea&#45;element"></a><code>[textarea](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-textarea-element)</code>&#44; <a id="pseudo&#45;classes&#58;the&#45;optgroup&#45;element"></a><code>[optgroup](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-optgroup-element)</code>&#44; <a id="pseudo&#45;classes&#58;the&#45;option&#45;element"></a><code>[option](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-option-element)</code>&#44; <a id="pseudo&#45;classes&#58;the&#45;fieldset&#45;element"></a><code>[fieldset](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-fieldset-element)</code> element&#44; or <a id="pseudo&#45;classes&#58;form&#45;associated&#45;custom&#45;element"></a>[form&#45;associated custom element](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#form-associated-custom-element) that is not <a id="pseudo&#45;classes&#58;concept&#45;element&#45;disabled"></a>[actually disabled](#concept-element-disabled)&#46;

<strong><a id="selector&#45;disabled"></a><strong><code>&#58;disabled</code></strong></strong>

<strong>✔</strong>MDN

[&#58;disabled](https://developer.mozilla.org/en-US/docs/Web/CSS/:disabled)

Support in all current engines&#46;

Firefox1&#43;Safari3&#46;1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera9&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer9&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android2&#43;Samsung Internet&#63;Opera Android10&#46;1&#43;

The <a id="pseudo&#45;classes&#58;selector&#45;disabled"></a><code>[&#58;disabled](#selector-disabled)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;13"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element that is <a id="pseudo&#45;classes&#58;concept&#45;element&#45;disabled&#45;2"></a>[actually disabled](#concept-element-disabled)&#46;

<strong><a id="selector&#45;checked"></a><strong><code>&#58;checked</code></strong></strong>

<strong>✔</strong>MDN

[&#58;checked](https://developer.mozilla.org/en-US/docs/Web/CSS/:checked)

Support in all current engines&#46;

Firefox1&#43;Safari3&#46;1&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera9&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer9&#43;

------------------------------------------------------------------------

Firefox Android4&#43;Safari iOS&#63;Chrome Android18&#43;WebView Android2&#43;Samsung Internet&#63;Opera Android10&#46;1&#43;

The <a id="pseudo&#45;classes&#58;selector&#45;checked"></a><code>[&#58;checked](#selector-checked)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;14"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories&#58;

- <a id="pseudo&#45;classes&#58;the&#45;input&#45;element&#45;3"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements whose <a id="pseudo&#45;classes&#58;attr&#45;input&#45;type&#45;2"></a><code>[type](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-type)</code> attribute is in the <a id="pseudo&#45;classes&#58;checkbox&#45;state&#45;&#40;type&#61;checkbox&#41;"></a>[Checkbox](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#checkbox-state-(type=checkbox)) state and whose <a id="pseudo&#45;classes&#58;concept&#45;fe&#45;checked"></a>[checkedness](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fe-checked) state is true
- <a id="pseudo&#45;classes&#58;the&#45;input&#45;element&#45;4"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements whose <a id="pseudo&#45;classes&#58;attr&#45;input&#45;type&#45;3"></a><code>[type](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-type)</code> attribute is in the <a id="pseudo&#45;classes&#58;radio&#45;button&#45;state&#45;&#40;type&#61;radio&#41;"></a>[Radio Button](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#radio-button-state-(type=radio)) state and whose <a id="pseudo&#45;classes&#58;concept&#45;fe&#45;checked&#45;2"></a>[checkedness](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fe-checked) state is true
- <a id="pseudo&#45;classes&#58;the&#45;option&#45;element&#45;2"></a><code>[option](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-option-element)</code> elements whose <a id="pseudo&#45;classes&#58;concept&#45;option&#45;selectedness"></a>[selectedness](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-option-selectedness) is true

<strong><a id="selector&#45;indeterminate"></a><strong><code>&#58;indeterminate</code></strong></strong>

<strong>✔</strong>MDN

[&#58;indeterminate](https://developer.mozilla.org/en-US/docs/Web/CSS/:indeterminate)

Support in all current engines&#46;

Firefox2&#43;Safari3&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera9&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer10&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS1&#43;Chrome Android&#63;WebView Android37&#43;Samsung Internet&#63;Opera Android10&#46;1&#43;

The <a id="pseudo&#45;classes&#58;selector&#45;indeterminate"></a><code>[&#58;indeterminate](#selector-indeterminate)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;15"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories&#58;

- <a id="pseudo&#45;classes&#58;the&#45;input&#45;element&#45;5"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements whose <a id="pseudo&#45;classes&#58;attr&#45;input&#45;type&#45;4"></a><code>[type](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-type)</code> attribute is in the <a id="pseudo&#45;classes&#58;checkbox&#45;state&#45;&#40;type&#61;checkbox&#41;&#45;2"></a>[Checkbox](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#checkbox-state-(type=checkbox)) state and whose <a id="pseudo&#45;classes&#58;concept&#45;input&#45;indeterminate"></a>[indeterminateness](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-input-indeterminate) is true
- <a id="pseudo&#45;classes&#58;the&#45;input&#45;element&#45;6"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements whose <a id="pseudo&#45;classes&#58;attr&#45;input&#45;type&#45;5"></a><code>[type](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-type)</code> attribute is in the <a id="pseudo&#45;classes&#58;radio&#45;button&#45;state&#45;&#40;type&#61;radio&#41;&#45;2"></a>[Radio Button](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#radio-button-state-(type=radio)) state and whose <a id="pseudo&#45;classes&#58;radio&#45;button&#45;group"></a>[radio button group](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#radio-button-group) contains no <a id="pseudo&#45;classes&#58;the&#45;input&#45;element&#45;7"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements whose <a id="pseudo&#45;classes&#58;concept&#45;fe&#45;checked&#45;3"></a>[checkedness](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fe-checked) state is true&#46;
- <a id="pseudo&#45;classes&#58;the&#45;progress&#45;element"></a><code>[progress](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-progress-element)</code> elements with no <a id="pseudo&#45;classes&#58;attr&#45;progress&#45;value"></a><code>[value](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-progress-value)</code> content attribute

<strong><a id="selector&#45;default"></a><strong><code>&#58;default</code></strong></strong>

<strong>✔</strong>MDN

[&#58;default](https://developer.mozilla.org/en-US/docs/Web/CSS/:default)

Support in all current engines&#46;

Firefox4&#43;Safari5&#43;Chrome10&#43;

------------------------------------------------------------------------

Opera10&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS5&#43;Chrome Android&#63;WebView Android37&#43;Samsung Internet&#63;Opera Android10&#46;1&#43;

The <a id="pseudo&#45;classes&#58;selector&#45;default"></a><code>[&#58;default](#selector-default)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;16"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories&#58;

- <a id="pseudo&#45;classes&#58;concept&#45;submit&#45;button"></a>[Submit buttons](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-submit-button) that are <a id="pseudo&#45;classes&#58;default&#45;button"></a>[default buttons](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#default-button) of their <a id="pseudo&#45;classes&#58;form&#45;owner"></a>[form owner](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#form-owner)&#46;
- <a id="pseudo&#45;classes&#58;the&#45;input&#45;element&#45;8"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements to which the <a id="pseudo&#45;classes&#58;attr&#45;input&#45;checked"></a><code>[checked](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-checked)</code> attribute applies and that have a <a id="pseudo&#45;classes&#58;attr&#45;input&#45;checked&#45;2"></a><code>[checked](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-checked)</code> attribute
- <a id="pseudo&#45;classes&#58;the&#45;option&#45;element&#45;3"></a><code>[option](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-option-element)</code> elements that have a <a id="pseudo&#45;classes&#58;attr&#45;option&#45;selected"></a><code>[selected](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-option-selected)</code> attribute

<strong><a id="selector&#45;placeholder&#45;shown"></a><strong><code>&#58;placeholder&#45;shown</code></strong></strong>

The <a id="pseudo&#45;classes&#58;selector&#45;placeholder&#45;shown"></a><code>[&#58;placeholder&#45;shown](#selector-placeholder-shown)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;17"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories&#58;

- <a id="pseudo&#45;classes&#58;the&#45;input&#45;element&#45;9"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements that have a <a id="pseudo&#45;classes&#58;attr&#45;input&#45;placeholder"></a><code>[placeholder](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-placeholder)</code> attribute whose value is currently being presented to the user
- <a id="pseudo&#45;classes&#58;the&#45;textarea&#45;element&#45;2"></a><code>[textarea](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-textarea-element)</code> elements that have a <a id="pseudo&#45;classes&#58;attr&#45;textarea&#45;placeholder"></a><code>[placeholder](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-textarea-placeholder)</code> attribute whose value is currently being presented to the user

<strong><a id="selector&#45;valid"></a><strong><code>&#58;valid</code></strong></strong>

<strong>✔</strong>MDN

[&#58;valid](https://developer.mozilla.org/en-US/docs/Web/CSS/:valid)

Support in all current engines&#46;

Firefox4&#43;Safari5&#43;Chrome10&#43;

------------------------------------------------------------------------

Opera10&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer10&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS5&#43;Chrome Android&#63;WebView Android37&#43;Samsung Internet&#63;Opera Android10&#46;1&#43;

The <a id="pseudo&#45;classes&#58;selector&#45;valid"></a><code>[&#58;valid](#selector-valid)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;18"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories&#58;

- elements that are <a id="pseudo&#45;classes&#58;candidate&#45;for&#45;constraint&#45;validation"></a>[candidates for constraint validation](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#candidate-for-constraint-validation) and that <a id="pseudo&#45;classes&#58;concept&#45;fv&#45;valid"></a>[satisfy their constraints](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fv-valid)
- <a id="pseudo&#45;classes&#58;the&#45;form&#45;element"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements that are not the <a id="pseudo&#45;classes&#58;form&#45;owner&#45;2"></a>[form owner](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#form-owner) of any elements that themselves are <a id="pseudo&#45;classes&#58;candidate&#45;for&#45;constraint&#45;validation&#45;2"></a>[candidates for constraint validation](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#candidate-for-constraint-validation) but do not <a id="pseudo&#45;classes&#58;concept&#45;fv&#45;valid&#45;2"></a>[satisfy their constraints](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fv-valid)
- <a id="pseudo&#45;classes&#58;the&#45;fieldset&#45;element&#45;2"></a><code>[fieldset](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-fieldset-element)</code> elements that have no descendant elements that themselves are <a id="pseudo&#45;classes&#58;candidate&#45;for&#45;constraint&#45;validation&#45;3"></a>[candidates for constraint validation](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#candidate-for-constraint-validation) but do not <a id="pseudo&#45;classes&#58;concept&#45;fv&#45;valid&#45;3"></a>[satisfy their constraints](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fv-valid)

<strong><a id="selector&#45;invalid"></a><strong><code>&#58;invalid</code></strong></strong>

<strong>✔</strong>MDN

[&#58;invalid](https://developer.mozilla.org/en-US/docs/Web/CSS/:invalid)

Support in all current engines&#46;

Firefox4&#43;Safari5&#43;Chrome10&#43;

------------------------------------------------------------------------

Opera10&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer10&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS5&#43;Chrome Android&#63;WebView Android37&#43;Samsung Internet&#63;Opera Android10&#46;1&#43;

The <a id="pseudo&#45;classes&#58;selector&#45;invalid"></a><code>[&#58;invalid](#selector-invalid)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;19"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories&#58;

- elements that are <a id="pseudo&#45;classes&#58;candidate&#45;for&#45;constraint&#45;validation&#45;4"></a>[candidates for constraint validation](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#candidate-for-constraint-validation) but that do not <a id="pseudo&#45;classes&#58;concept&#45;fv&#45;valid&#45;4"></a>[satisfy their constraints](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fv-valid)
- <a id="pseudo&#45;classes&#58;the&#45;form&#45;element&#45;2"></a><code>[form](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-form-element)</code> elements that are the <a id="pseudo&#45;classes&#58;form&#45;owner&#45;3"></a>[form owner](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#form-owner) of one or more elements that themselves are <a id="pseudo&#45;classes&#58;candidate&#45;for&#45;constraint&#45;validation&#45;5"></a>[candidates for constraint validation](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#candidate-for-constraint-validation) but do not <a id="pseudo&#45;classes&#58;concept&#45;fv&#45;valid&#45;5"></a>[satisfy their constraints](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fv-valid)
- <a id="pseudo&#45;classes&#58;the&#45;fieldset&#45;element&#45;3"></a><code>[fieldset](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-fieldset-element)</code> elements that have of one or more descendant elements that themselves are <a id="pseudo&#45;classes&#58;candidate&#45;for&#45;constraint&#45;validation&#45;6"></a>[candidates for constraint validation](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#candidate-for-constraint-validation) but do not <a id="pseudo&#45;classes&#58;concept&#45;fv&#45;valid&#45;6"></a>[satisfy their constraints](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fv-valid)

<strong><a id="selector&#45;user&#45;valid"></a><strong><code>&#58;user&#45;valid</code></strong></strong>

The <a id="pseudo&#45;classes&#58;selector&#45;user&#45;valid"></a><code>[&#58;user&#45;valid](#selector-user-valid)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;20"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match <a id="pseudo&#45;classes&#58;the&#45;input&#45;element&#45;10"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code>&#44; <a id="pseudo&#45;classes&#58;the&#45;textarea&#45;element&#45;3"></a><code>[textarea](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-textarea-element)</code>&#44; and <a id="pseudo&#45;classes&#58;the&#45;select&#45;element&#45;2"></a><code>[select](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-select-element)</code> elements whose <a id="pseudo&#45;classes&#58;user&#45;validity"></a>[user validity](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#user-validity) is true&#44; are <a id="pseudo&#45;classes&#58;candidate&#45;for&#45;constraint&#45;validation&#45;7"></a>[candidates for constraint validation](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#candidate-for-constraint-validation)&#44; and that <a id="pseudo&#45;classes&#58;concept&#45;fv&#45;valid&#45;7"></a>[satisfy their constraints](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fv-valid)&#46;

<strong><a id="selector&#45;user&#45;invalid"></a><strong><code>&#58;user&#45;invalid</code></strong></strong>

The <a id="pseudo&#45;classes&#58;selector&#45;user&#45;invalid"></a><code>[&#58;user&#45;invalid](#selector-user-invalid)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;21"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match <a id="pseudo&#45;classes&#58;the&#45;input&#45;element&#45;11"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code>&#44; <a id="pseudo&#45;classes&#58;the&#45;textarea&#45;element&#45;4"></a><code>[textarea](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-textarea-element)</code>&#44; and <a id="pseudo&#45;classes&#58;the&#45;select&#45;element&#45;3"></a><code>[select](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-select-element)</code> elements whose <a id="pseudo&#45;classes&#58;user&#45;validity&#45;2"></a>[user validity](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#user-validity) is true&#44; are <a id="pseudo&#45;classes&#58;candidate&#45;for&#45;constraint&#45;validation&#45;8"></a>[candidates for constraint validation](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#candidate-for-constraint-validation) but do not <a id="pseudo&#45;classes&#58;concept&#45;fv&#45;valid&#45;8"></a>[satisfy their constraints](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fv-valid)&#46;

<strong><a id="selector&#45;in&#45;range"></a><strong><code>&#58;in&#45;range</code></strong></strong>

<strong>✔</strong>MDN

[&#58;in&#45;range](https://developer.mozilla.org/en-US/docs/Web/CSS/:in-range)

Support in all current engines&#46;

Firefox29&#43;Safari5&#46;1&#43;Chrome10&#43;

------------------------------------------------------------------------

Opera11&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;13&#43;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android16&#43;Safari iOS&#63;Chrome Android&#63;WebView Android2&#46;2&#43;Samsung Internet1&#46;0&#43;Opera Android11&#43;

The <a id="pseudo&#45;classes&#58;selector&#45;in&#45;range"></a><code>[&#58;in&#45;range](#selector-in-range)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;22"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match all elements that are <a id="pseudo&#45;classes&#58;candidate&#45;for&#45;constraint&#45;validation&#45;9"></a>[candidates for constraint validation](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#candidate-for-constraint-validation)&#44; <a id="pseudo&#45;classes&#58;have&#45;range&#45;limitations"></a>[have range limitations](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#have-range-limitations)&#44; and that are neither <a id="pseudo&#45;classes&#58;suffering&#45;from&#45;an&#45;underflow"></a>[suffering from an underflow](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#suffering-from-an-underflow) nor <a id="pseudo&#45;classes&#58;suffering&#45;from&#45;an&#45;overflow"></a>[suffering from an overflow](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#suffering-from-an-overflow)&#46;

<strong><a id="selector&#45;out&#45;of&#45;range"></a><strong><code>&#58;out&#45;of&#45;range</code></strong></strong>

<strong>✔</strong>MDN

[&#58;out&#45;of&#45;range](https://developer.mozilla.org/en-US/docs/Web/CSS/:out-of-range)

Support in all current engines&#46;

Firefox29&#43;Safari5&#46;1&#43;Chrome10&#43;

------------------------------------------------------------------------

Opera11&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;13&#43;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android16&#43;Safari iOS&#63;Chrome Android&#63;WebView Android2&#46;2&#43;Samsung Internet&#63;Opera Android11&#43;

The <a id="pseudo&#45;classes&#58;selector&#45;out&#45;of&#45;range"></a><code>[&#58;out&#45;of&#45;range](#selector-out-of-range)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;23"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match all elements that are <a id="pseudo&#45;classes&#58;candidate&#45;for&#45;constraint&#45;validation&#45;10"></a>[candidates for constraint validation](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#candidate-for-constraint-validation)&#44; <a id="pseudo&#45;classes&#58;have&#45;range&#45;limitations&#45;2"></a>[have range limitations](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#have-range-limitations)&#44; and that are either <a id="pseudo&#45;classes&#58;suffering&#45;from&#45;an&#45;underflow&#45;2"></a>[suffering from an underflow](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#suffering-from-an-underflow) or <a id="pseudo&#45;classes&#58;suffering&#45;from&#45;an&#45;overflow&#45;2"></a>[suffering from an overflow](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#suffering-from-an-overflow)&#46;

<strong><a id="selector&#45;required"></a><strong><code>&#58;required</code></strong></strong>

<strong>✔</strong>MDN

[&#58;required](https://developer.mozilla.org/en-US/docs/Web/CSS/:required)

Support in all current engines&#46;

Firefox4&#43;Safari5&#43;Chrome10&#43;

------------------------------------------------------------------------

Opera10&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer10&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS5&#43;Chrome Android&#63;WebView Android4&#46;4&#46;3&#43;Samsung Internet&#63;Opera Android10&#46;1&#43;

The <a id="pseudo&#45;classes&#58;selector&#45;required"></a><code>[&#58;required](#selector-required)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;24"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories&#58;

- <a id="pseudo&#45;classes&#58;the&#45;input&#45;element&#45;12"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements that are <a id="pseudo&#45;classes&#58;concept&#45;input&#45;required"></a><em>[required](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-input-required)</em>
- <a id="pseudo&#45;classes&#58;the&#45;select&#45;element&#45;4"></a><code>[select](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-select-element)</code> elements that have a <a id="pseudo&#45;classes&#58;attr&#45;select&#45;required"></a><code>[required](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-select-required)</code> attribute
- <a id="pseudo&#45;classes&#58;the&#45;textarea&#45;element&#45;5"></a><code>[textarea](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-textarea-element)</code> elements that have a <a id="pseudo&#45;classes&#58;attr&#45;textarea&#45;required"></a><code>[required](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-textarea-required)</code> attribute

<strong><a id="selector&#45;optional"></a><strong><code>&#58;optional</code></strong></strong>

<strong>✔</strong>MDN

[&#58;optional](https://developer.mozilla.org/en-US/docs/Web/CSS/:optional)

Support in all current engines&#46;

Firefox4&#43;Safari5&#43;Chrome10&#43;

------------------------------------------------------------------------

Opera10&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;12&#43;Internet Explorer10&#43;

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS5&#43;Chrome Android&#63;WebView Android37&#43;Samsung Internet&#63;Opera Android10&#46;1&#43;

The <a id="pseudo&#45;classes&#58;selector&#45;optional"></a><code>[&#58;optional](#selector-optional)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;25"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories&#58;

- <a id="pseudo&#45;classes&#58;the&#45;input&#45;element&#45;13"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements to which the <a id="pseudo&#45;classes&#58;attr&#45;input&#45;required"></a><code>[required](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-required)</code> attribute applies that are not <a id="pseudo&#45;classes&#58;concept&#45;input&#45;required&#45;2"></a><em>[required](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-input-required)</em>
- <a id="pseudo&#45;classes&#58;the&#45;select&#45;element&#45;5"></a><code>[select](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-select-element)</code> elements that do not have a <a id="pseudo&#45;classes&#58;attr&#45;select&#45;required&#45;2"></a><code>[required](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-select-required)</code> attribute
- <a id="pseudo&#45;classes&#58;the&#45;textarea&#45;element&#45;6"></a><code>[textarea](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-textarea-element)</code> elements that do not have a <a id="pseudo&#45;classes&#58;attr&#45;textarea&#45;required&#45;2"></a><code>[required](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-textarea-required)</code> attribute

<strong><a id="selector&#45;autofill"></a><strong><code>&#58;autofill</code></strong></strong>

MDN

[&#58;autofill](https://developer.mozilla.org/en-US/docs/Web/CSS/:autofill)

Firefox86&#43;Safari15&#43;Chrome🔰 1&#43;

------------------------------------------------------------------------

Opera&#63;Edge🔰 79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS15&#43;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<strong><a id="selector&#45;webkit&#45;autofill"></a><strong><code>&#58;&#45;webkit&#45;autofill</code></strong></strong>

The <a id="pseudo&#45;classes&#58;selector&#45;autofill"></a><code>[&#58;autofill](#selector-autofill)</code> and <a id="pseudo&#45;classes&#58;selector&#45;webkit&#45;autofill"></a><code>[&#58;&#45;webkit&#45;autofill](#selector-webkit-autofill)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;26"></a>[pseudo&#45;classes](https://drafts.csswg.org/selectors/#pseudo-class) must match <a id="pseudo&#45;classes&#58;the&#45;input&#45;element&#45;14"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements which have been autofilled by user agent&#46; These pseudo&#45;classes must stop matching if the user edits the autofilled field&#46;

<strong>Note&#58;</strong>

One way such autofilling might happen is via the <a id="pseudo&#45;classes&#58;attr&#45;fe&#45;autocomplete"></a><code>[autocomplete](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-fe-autocomplete)</code> attribute&#44; but user agents could autofill even without that attribute being involved&#46;

<strong><a id="selector&#45;read&#45;only"></a><strong><code>&#58;read&#45;only</code></strong></strong>

<strong>✔</strong>MDN

[&#58;read&#45;only](https://developer.mozilla.org/en-US/docs/Web/CSS/:read-only)

Support in all current engines&#46;

Firefox78&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera9&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;13&#43;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android🔰 4&#43;Safari iOS&#63;Chrome Android&#63;WebView Android37&#43;Samsung Internet&#63;Opera Android10&#46;1&#43;

<strong><a id="selector&#45;read&#45;write"></a><strong><code>&#58;read&#45;write</code></strong></strong>

<strong>✔</strong>MDN

[&#58;read&#45;write](https://developer.mozilla.org/en-US/docs/Web/CSS/:read-write)

Support in all current engines&#46;

Firefox78&#43;Safari4&#43;Chrome1&#43;

------------------------------------------------------------------------

Opera9&#43;Edge79&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;13&#43;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android🔰 4&#43;Safari iOS&#63;Chrome Android&#63;WebView Android37&#43;Samsung Internet&#63;Opera Android10&#46;1&#43;

The <a id="pseudo&#45;classes&#58;selector&#45;read&#45;write"></a><code>[&#58;read&#45;write](#selector-read-write)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;27"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories&#44; which for the purposes of Selectors are thus considered <em>user&#45;alterable</em>&#58; [&#91;SELECTORS&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsSELECTORS)

- <a id="pseudo&#45;classes&#58;the&#45;input&#45;element&#45;15"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements to which the <a id="pseudo&#45;classes&#58;attr&#45;input&#45;readonly"></a><code>[readonly](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-readonly)</code> attribute applies&#44; and that are <a id="pseudo&#45;classes&#58;concept&#45;fe&#45;mutable"></a>[mutable](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fe-mutable) &#40;i&#46;e&#46; that do not have the <a id="pseudo&#45;classes&#58;attr&#45;input&#45;readonly&#45;2"></a><code>[readonly](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-input-readonly)</code> attribute specified and that are not <a id="pseudo&#45;classes&#58;concept&#45;fe&#45;disabled"></a>[disabled](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fe-disabled)&#41;
- <a id="pseudo&#45;classes&#58;the&#45;textarea&#45;element&#45;7"></a><code>[textarea](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-textarea-element)</code> elements that do not have a <a id="pseudo&#45;classes&#58;attr&#45;textarea&#45;readonly"></a><code>[readonly](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#attr-textarea-readonly)</code> attribute&#44; and that are not <a id="pseudo&#45;classes&#58;concept&#45;fe&#45;disabled&#45;2"></a>[disabled](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#concept-fe-disabled)
- elements that are <a id="pseudo&#45;classes&#58;editing&#45;host"></a>[editing hosts](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#editing-host) or <a id="pseudo&#45;classes&#58;editable"></a>[editable](https://w3c.github.io/editing/docs/execCommand/#editable) and are neither <a id="pseudo&#45;classes&#58;the&#45;input&#45;element&#45;16"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements nor <a id="pseudo&#45;classes&#58;the&#45;textarea&#45;element&#45;8"></a><code>[textarea](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-textarea-element)</code> elements

The <a id="pseudo&#45;classes&#58;selector&#45;read&#45;only"></a><code>[&#58;read&#45;only](#selector-read-only)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;28"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match all other <a id="pseudo&#45;classes&#58;html&#45;elements&#45;2"></a>[HTML elements](html-language-foundations--877d6146487f--b55e73c31978.md#html-elements)&#46;

<strong><a id="selector&#45;modal"></a><strong><code>&#58;modal</code></strong></strong>

The <a id="pseudo&#45;classes&#58;selector&#45;modal"></a><code>[&#58;modal](#selector-modal)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;29"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories&#58;

- <a id="pseudo&#45;classes&#58;the&#45;dialog&#45;element"></a><code>[dialog](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-dialog-element)</code> elements whose <a id="pseudo&#45;classes&#58;is&#45;modal"></a>[is modal](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#is-modal) is true
- elements whose <a id="pseudo&#45;classes&#58;fullscreen&#45;flag"></a>[fullscreen flag](https://fullscreen.spec.whatwg.org/#fullscreen-flag) is true

<strong><a id="selector&#45;ltr"></a><strong><code>&#58;dir&#40;ltr&#41;</code></strong></strong>

MDN

[&#58;dir](https://developer.mozilla.org/en-US/docs/Web/CSS/:dir)

Firefox49&#43;Safari16&#46;4&#43;ChromeNo

------------------------------------------------------------------------

Opera&#63;EdgeNo

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

The <a id="pseudo&#45;classes&#58;selector&#45;ltr"></a><code>[&#58;dir&#40;ltr&#41;](#selector-ltr)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;30"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match all elements whose <a id="pseudo&#45;classes&#58;the&#45;directionality"></a>[directionality](html-language-foundations--877d6146487f--b55e73c31978.md#the-directionality) is &#39;<a id="pseudo&#45;classes&#58;concept&#45;ltr"></a>[ltr](html-language-foundations--877d6146487f--b55e73c31978.md#concept-ltr)&#39;&#46;

<strong><a id="selector&#45;rtl"></a><strong><code>&#58;dir&#40;rtl&#41;</code></strong></strong>

The <a id="pseudo&#45;classes&#58;selector&#45;rtl"></a><code>[&#58;dir&#40;rtl&#41;](#selector-rtl)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;31"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match all elements whose <a id="pseudo&#45;classes&#58;the&#45;directionality&#45;2"></a>[directionality](html-language-foundations--877d6146487f--b55e73c31978.md#the-directionality) is &#39;<a id="pseudo&#45;classes&#58;concept&#45;rtl"></a>[rtl](html-language-foundations--877d6146487f--b55e73c31978.md#concept-rtl)&#39;&#46;

<strong><a id="selector&#45;custom"></a><strong>Custom state pseudo&#45;class</strong></strong>

The <a id="pseudo&#45;classes&#58;selector&#45;custom"></a><code>[&#58;state&#40;<var>identifier</var>&#41;](#selector-custom)</code> pseudo&#45;class must match all <a id="pseudo&#45;classes&#58;custom&#45;element"></a>[custom element](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#custom-element)s whose <a id="pseudo&#45;classes&#58;states&#45;set"></a>[states set](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#states-set)&#39;s <a id="pseudo&#45;classes&#58;set&#45;entries"></a>[set entries](https://webidl.spec.whatwg.org/#dfn-set-entries) contains <var>identifier</var>&#46;

<strong><a id="selector&#45;heading"></a><strong><code>&#58;heading</code></strong></strong>

The <a id="pseudo&#45;classes&#58;selector&#45;heading"></a><code>[&#58;heading](#selector-heading)</code> pseudo&#45;class must match all <a id="pseudo&#45;classes&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements"></a><code>[h1](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; <a id="pseudo&#45;classes&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;2"></a><code>[h2](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; <a id="pseudo&#45;classes&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;3"></a><code>[h3](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; <a id="pseudo&#45;classes&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;4"></a><code>[h4](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; <a id="pseudo&#45;classes&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;5"></a><code>[h5](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; and <a id="pseudo&#45;classes&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;6"></a><code>[h6](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code> elements&#46;

<strong><a id="selector&#45;heading&#45;functional"></a><strong><code>&#58;heading&#40;<var>integer</var>&#35;&#41;</code></strong></strong>

The <a id="pseudo&#45;classes&#58;selector&#45;heading&#45;functional"></a><code>[&#58;heading&#40;<var>integer</var>&#35;&#41;](#selector-heading-functional)</code> pseudo&#45;class must match all <a id="pseudo&#45;classes&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;7"></a><code>[h1](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; <a id="pseudo&#45;classes&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;8"></a><code>[h2](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; <a id="pseudo&#45;classes&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;9"></a><code>[h3](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; <a id="pseudo&#45;classes&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;10"></a><code>[h4](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; <a id="pseudo&#45;classes&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;11"></a><code>[h5](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code>&#44; and <a id="pseudo&#45;classes&#58;the&#45;h1&#44;&#45;h2&#44;&#45;h3&#44;&#45;h4&#44;&#45;h5&#44;&#45;and&#45;h6&#45;elements&#45;12"></a><code>[h6](html-language-elements-text-links--877d6146487f--8833a85fd242.md#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements)</code> elements that have a <a id="pseudo&#45;classes&#58;heading&#45;level"></a>[heading level](html-language-elements-text-links--877d6146487f--8833a85fd242.md#heading-level) of <var>integer</var>&#46; [&#91;CSSSYNTAX&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsCSSSYNTAX) [&#91;CSSVALUES&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsCSSVALUES)

<strong><a id="selector&#45;playing"></a><strong><code>&#58;playing</code></strong></strong>

The <a id="pseudo&#45;classes&#58;selector&#45;playing"></a><code>[&#58;playing](#selector-playing)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;32"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match all <a id="pseudo&#45;classes&#58;media&#45;element"></a>[media elements](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#media-element) whose <a id="pseudo&#45;classes&#58;dom&#45;media&#45;paused"></a><code>[paused](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#dom-media-paused)</code> attribute is false&#46;

<strong><a id="selector&#45;paused"></a><strong><code>&#58;paused</code></strong></strong>

The <a id="pseudo&#45;classes&#58;selector&#45;paused"></a><code>[&#58;paused](#selector-paused)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;33"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match all <a id="pseudo&#45;classes&#58;media&#45;element&#45;2"></a>[media elements](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#media-element) whose <a id="pseudo&#45;classes&#58;dom&#45;media&#45;paused&#45;2"></a><code>[paused](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#dom-media-paused)</code> attribute is true&#46;

<strong><a id="selector&#45;seeking"></a><strong><code>&#58;seeking</code></strong></strong>

The <a id="pseudo&#45;classes&#58;selector&#45;seeking"></a><code>[&#58;seeking](#selector-seeking)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;34"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match all <a id="pseudo&#45;classes&#58;media&#45;element&#45;3"></a>[media elements](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#media-element) whose <a id="pseudo&#45;classes&#58;dom&#45;media&#45;seeking"></a><code>[seeking](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#dom-media-seeking)</code> attribute is true&#46;

<strong><a id="selector&#45;buffering"></a><strong><code>&#58;buffering</code></strong></strong>

The <a id="pseudo&#45;classes&#58;selector&#45;buffering"></a><code>[&#58;buffering](#selector-buffering)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;35"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match all <a id="pseudo&#45;classes&#58;media&#45;element&#45;4"></a>[media elements](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#media-element) whose <a id="pseudo&#45;classes&#58;dom&#45;media&#45;paused&#45;3"></a><code>[paused](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#dom-media-paused)</code> attribute is false&#44; <a id="pseudo&#45;classes&#58;dom&#45;media&#45;networkstate"></a><code>[networkState](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#dom-media-networkstate)</code> attribute is <a id="pseudo&#45;classes&#58;dom&#45;media&#45;network&#95;loading"></a><code>[NETWORK&#95;LOADING](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#dom-media-network_loading)</code>&#44; and ready state is <a id="pseudo&#45;classes&#58;dom&#45;media&#45;have&#95;current&#95;data"></a><code>[HAVE&#95;CURRENT&#95;DATA](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#dom-media-have_current_data)</code> or less&#46;

<strong><a id="selector&#45;stalled"></a><strong><code>&#58;stalled</code></strong></strong>

The <a id="pseudo&#45;classes&#58;selector&#45;stalled"></a><code>[&#58;stalled](#selector-stalled)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;36"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match all <a id="pseudo&#45;classes&#58;media&#45;element&#45;5"></a>[media elements](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#media-element) that match the <a id="pseudo&#45;classes&#58;selector&#45;buffering&#45;2"></a><code>[&#58;buffering](#selector-buffering)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;37"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) and whose <a id="pseudo&#45;classes&#58;is&#45;currently&#45;stalled"></a>[is currently stalled](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#is-currently-stalled) is true&#46;

<strong>Note&#58;</strong>

This pseudo&#45;class is intended for displaying player state UI when video playback has been buffering for too long&#46; Unlike the <a id="pseudo&#45;classes&#58;event&#45;media&#45;stalled"></a><code>[stalled](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#event-media-stalled)</code> event&#44; which fires whenever the network download has stalled regardless of playback or ready state&#46;

<strong><a id="selector&#45;muted"></a><strong><code>&#58;muted</code></strong></strong>

The <a id="pseudo&#45;classes&#58;selector&#45;muted"></a><code>[&#58;muted](#selector-muted)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;38"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match all <a id="pseudo&#45;classes&#58;media&#45;element&#45;6"></a>[media elements](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#media-element) that are <a id="pseudo&#45;classes&#58;concept&#45;media&#45;muted"></a>[muted](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#concept-media-muted)&#46;

<strong><a id="selector&#45;volume&#45;locked"></a><strong><code>&#58;volume&#45;locked</code></strong></strong>

The <a id="pseudo&#45;classes&#58;selector&#45;volume&#45;locked"></a><code>[&#58;volume&#45;locked](#selector-volume-locked)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;39"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match all <a id="pseudo&#45;classes&#58;media&#45;element&#45;7"></a>[media elements](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#media-element) when the user agent&#39;s <a id="pseudo&#45;classes&#58;volume&#45;locked"></a>[volume locked](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md#volume-locked) is true&#46;

<strong><a id="selector&#45;open"></a><strong><code>&#58;open</code></strong></strong>

The <a id="pseudo&#45;classes&#58;selector&#45;open"></a><code>[&#58;open](#selector-open)</code> <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;40"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class) must match any element falling into one of the following categories&#58;

- <a id="pseudo&#45;classes&#58;the&#45;details&#45;element"></a><code>[details](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-details-element)</code> elements that have an <a id="pseudo&#45;classes&#58;attr&#45;details&#45;open"></a><code>[open](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#attr-details-open)</code> attribute

- <a id="pseudo&#45;classes&#58;the&#45;dialog&#45;element&#45;2"></a><code>[dialog](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-dialog-element)</code> elements that have an <a id="pseudo&#45;classes&#58;attr&#45;dialog&#45;open"></a><code>[open](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#attr-dialog-open)</code> attribute

- <a id="pseudo&#45;classes&#58;the&#45;select&#45;element&#45;6"></a><code>[select](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-select-element)</code> elements that are a <a id="pseudo&#45;classes&#58;drop&#45;down&#45;box"></a>[drop&#45;down box](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#drop-down-box) and whose drop&#45;down boxes are open

- <a id="pseudo&#45;classes&#58;the&#45;input&#45;element&#45;17"></a><code>[input](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#the-input-element)</code> elements that <a id="pseudo&#45;classes&#58;input&#45;support&#45;picker"></a>[support a picker](html-language-forms-tables-templates--877d6146487f--2dfc6f632608.md#input-support-picker) and whose pickers are open

<strong>Note&#58;</strong>

This specification does not define when an element matches the <code>&#58;lang&#40;&#41;</code> dynamic <a id="pseudo&#45;classes&#58;pseudo&#45;class&#45;41"></a>[pseudo&#45;class](https://drafts.csswg.org/selectors/#pseudo-class)&#44; as it is defined in sufficient detail in a language&#45;agnostic fashion in Selectors&#46; [&#91;SELECTORS&#93;](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsSELECTORS)
