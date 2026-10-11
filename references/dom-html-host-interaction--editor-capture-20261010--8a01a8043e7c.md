# HTML Standard: bounded DOM foundation source capture

Attribution and reformatting notice added for Surgeist on 2026-10-10. Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Original English standard and normative/informative labels remain authoritative; this partial format conversion is implementation support, not a replacement specification or an implementation-conformance claim. No endorsement is implied.

Source: [HTML Standard](https://html.spec.whatwg.org/multipage/interaction.html). Retrieved 2026-10-10. Displayed status: HTMLLiving Standard — Last Updated 9 October 2026.

The exact upstream repository LICENSE files from DOM `3071e5f269448350fdea79bb284eedaf17bc431e`, HTML `877d6146487f15b15bc8dd0e3c625063a37b456a`, Web IDL `0620cf171d1b97138847088748e363cae4005156` and Infra `c67ae1fb1d161b23d0f8c9bf9917ea1993f7b19a` all match the already retained [WHATWG LICENSE](../licenses/whatwg/LICENSE) byte-for-byte, SHA-256 `85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`. Specification prose is Creative Commons Attribution 4.0 International; the BSD 3-Clause scope applies when portions are incorporated into code, not as an alternative license for this Markdown.

Full retrieved HTML: 453,780 bytes, SHA-256 `87d5bedffbeb165e8c5c2b5765de86db90277fcd0fd2f47829d99fe90f64a3c4`. Complete-section extraction input: 210,713 bytes, SHA-256 `8a01a8043e7c1080cd1b8cc7a224a9d293756bb63fe4f6d17ea2e49803e6fe55`. Source revision metadata: `No revision metadata in this chapter; GitHub HEAD association is not proven.`.

Retained complete heading sections, including descendant clauses and bounded at the next equal-or-higher-level heading: `the-hidden-attribute`, `inert-subtrees`, `user-activation-data-model`, `focus`. Original source header is also retained. Every other clause and all cross-chapter references remain upstream. These are explicitly partial captures. HTML commit-snapshot chapter URLs for the stated repository HEAD returned HTTP 404; exact fetched chapter body hashes identify the captures without asserting that repository HEAD generated them.

Conversion uses existing repository HTML preparation/table-span models, approved Pandoc 3.1.11.1 and the semantic Lua filter. Source scripts/styles are omitted without execution. Source headings, IDs, links, prose, inline code, variables, literal blocks, images and every selected table cell/span are compared against parsed GFM. Selected tables remain literal semantic HTML, preserving every original table cell, code/link/variable, row/column span and anchor; table flattening is not used. Source examples and Web IDL remain evidence even where the corresponding JavaScript API is outside Surgeist's native Rust contract.

---

<!-- captured-body-start -->

<a id="head"></a>

[![WHATWG](https://resources.whatwg.org/logo.svg)](https://whatwg.org/)

# HTML

<a id="living-standard"></a>Living Standard — Last Updated 9 October 2026

### <a id="the-hidden-attribute"></a>6.1 The <a id="the-hidden-attribute&#58;attr-hidden"></a><code>[hidden](#attr-hidden)</code> attribute[](#the-hidden-attribute)

<strong>⚠</strong>MDN

[Global_attributes/hidden](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/hidden)

Support in one engine only.

FirefoxNoSafariNoChrome102+

------------------------------------------------------------------------

OperaNoEdge102+

------------------------------------------------------------------------

Edge (Legacy)?Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android?Samsung Internet?Opera Android?

[Global_attributes/hidden](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/hidden)

Support in all current engines.

Firefox4+Safari5.1+Chrome10+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer11

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android4+Samsung Internet?Opera Android?

All <a id="the-hidden-attribute&#58;html-elements"></a>[HTML elements](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements) may have the <a id="attr-hidden"></a><strong><code>hidden</code></strong> content attribute set. The <a id="the-hidden-attribute&#58;attr-hidden-2"></a><code>[hidden](#attr-hidden)</code> attribute is an <a id="the-hidden-attribute&#58;enumerated-attribute"></a>[enumerated attribute](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#enumerated-attribute) with the following keywords and states:

<table><thead><tr><th>Keyword      </th><th>State      </th><th>Brief description    </th></tr></thead><tbody><tr><td><dfn data-dfn-for="html-global/hidden" id="attr-hidden-hidden" data-dfn-type="attr-value"><code>hidden</code></dfn>      </td><td><dfn id="attr-hidden-hidden-state">Hidden</dfn>      </td><td>Will not be rendered.     </td></tr><tr><td><dfn data-dfn-for="html-global/hidden" id="attr-hidden-until-found" data-dfn-type="attr-value"><code>until-found</code></dfn>      </td><td><dfn id="attr-hidden-until-found-state">Hidden Until Found</dfn>      </td><td>Will not be rendered, but content inside will be accessible to <a href="https://html.spec.whatwg.org/multipage/interaction.html#find-in-page-2" id="the-hidden-attribute:find-in-page-2">find-in-page</a>      and <a href="https://html.spec.whatwg.org/multipage/browsing-the-web.html#navigate-fragid" id="the-hidden-attribute:navigate-fragid">fragment navigation</a>.   </td></tr></tbody></table>

The attribute's <a id="the-hidden-attribute&#58;missing-value-default"></a><em>[missing value default](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#missing-value-default)</em> is the <a id="attr-hidden-not-hidden-state"></a><strong>Not Hidden</strong> state, and its <a id="the-hidden-attribute&#58;invalid-value-default"></a><em>[invalid value default](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#invalid-value-default)</em> and <a id="the-hidden-attribute&#58;empty-value-default"></a><em>[empty value default](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#empty-value-default)</em> are both the <a id="the-hidden-attribute&#58;attr-hidden-hidden-state"></a>[Hidden](#attr-hidden-hidden-state) state.

When an element has the <a id="the-hidden-attribute&#58;attr-hidden-3"></a><code>[hidden](#attr-hidden)</code> attribute in the <a id="the-hidden-attribute&#58;attr-hidden-hidden-state-2"></a>[Hidden](#attr-hidden-hidden-state) state, it indicates that the element is not yet, or is no longer, directly relevant to the page's current state, or that it is being used to declare content to be reused by other parts of the page as opposed to being directly accessed by the user. User agents should not render elements that are in the <a id="the-hidden-attribute&#58;attr-hidden-hidden-state-3"></a>[Hidden](#attr-hidden-hidden-state) state.

<strong>Note:</strong>

The requirement for user agents not to render elements that are in the <a id="the-hidden-attribute&#58;attr-hidden-hidden-state-4"></a>[Hidden](#attr-hidden-hidden-state) state can be implemented indirectly through the style layer. For example, a web browser could implement these requirements [using the rules suggested in the Rendering section](https://html.spec.whatwg.org/multipage/rendering.html#hiddenCSS).

When an element has the <a id="the-hidden-attribute&#58;attr-hidden-4"></a><code>[hidden](#attr-hidden)</code> attribute in the <a id="the-hidden-attribute&#58;attr-hidden-until-found-state"></a>[Hidden Until Found](#attr-hidden-until-found-state) state, it indicates that the element is hidden like the <a id="the-hidden-attribute&#58;attr-hidden-hidden-state-5"></a>[Hidden](#attr-hidden-hidden-state) state but the content inside the element will be accessible to <a id="the-hidden-attribute&#58;find-in-page-2-2"></a>[find-in-page](https://html.spec.whatwg.org/multipage/interaction.html#find-in-page-2) and <a id="the-hidden-attribute&#58;navigate-fragid-2"></a>[fragment navigation](https://html.spec.whatwg.org/multipage/browsing-the-web.html#navigate-fragid). When these features attempt to scroll to a target which is in the element's subtree, the user agent will remove the <a id="the-hidden-attribute&#58;attr-hidden-5"></a><code>[hidden](#attr-hidden)</code> attribute in order to reveal the content before scrolling to it by running the <a id="the-hidden-attribute&#58;ancestor-revealing-algorithm"></a>[ancestor revealing algorithm](#ancestor-revealing-algorithm) on the target node.

<strong>Note:</strong>

Web browsers will use 'content-visibility: hidden' instead of 'display: none' when the <a id="the-hidden-attribute&#58;attr-hidden-6"></a><code>[hidden](#attr-hidden)</code> attribute is in the <a id="the-hidden-attribute&#58;attr-hidden-until-found-state-2"></a>[Hidden Until Found](#attr-hidden-until-found-state) state, as specified in the [Rendering section](https://html.spec.whatwg.org/multipage/rendering.html#hiddenCSS).

<strong>Note:</strong>

Because this attribute is typically implemented using CSS, it's also possible to override it using CSS. For instance, a rule that applies 'display: block' to all elements will cancel the effects of the <a id="the-hidden-attribute&#58;attr-hidden-hidden-state-6"></a>[Hidden](#attr-hidden-hidden-state) state. Authors therefore have to take care when writing their style sheets to make sure that the attribute is still styled as expected. In addition, legacy user agents which don't support the <a id="the-hidden-attribute&#58;attr-hidden-until-found-state-3"></a>[Hidden Until Found](#attr-hidden-until-found-state) state will have 'display: none' instead of 'content-visibility: hidden', so authors are encouraged to make sure that their style sheets don't change the 'display' or 'content-visibility' properties of <a id="the-hidden-attribute&#58;attr-hidden-until-found-state-4"></a>[Hidden Until Found](#attr-hidden-until-found-state) elements.

Since elements with the <a id="the-hidden-attribute&#58;attr-hidden-7"></a><code>[hidden](#attr-hidden)</code> attribute in the <a id="the-hidden-attribute&#58;attr-hidden-until-found-state-5"></a>[Hidden Until Found](#attr-hidden-until-found-state) state use 'content-visibility: hidden' instead of 'display: none', there are two caveats of the <a id="the-hidden-attribute&#58;attr-hidden-until-found-state-6"></a>[Hidden Until Found](#attr-hidden-until-found-state) state that make it different from the <a id="the-hidden-attribute&#58;attr-hidden-hidden-state-7"></a>[Hidden](#attr-hidden-hidden-state) state:

1.  The element needs to be affected by <a id="the-hidden-attribute&#58;layout-containment"></a>[layout containment](https://drafts.csswg.org/css-contain/#containment-layout) in order to be revealed by find-in-page. This means that if the element in the <a id="the-hidden-attribute&#58;attr-hidden-until-found-state-7"></a>[Hidden Until Found](#attr-hidden-until-found-state) state has a 'display' value of 'none', 'contents', or 'inline', then the element will not be revealed by find-in-page.

2.  The element will still have a <a id="the-hidden-attribute&#58;'visibility'"></a>[generated box](https://drafts.csswg.org/css2/#propdef-visibility) when in the <a id="the-hidden-attribute&#58;attr-hidden-until-found-state-8"></a>[Hidden Until Found](#attr-hidden-until-found-state) state, which means that borders, margin, and padding will still be rendered around the element.

<strong>Example:</strong>

In the following skeletal example, the attribute is used to hide the web game's main screen until the user logs in:

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

The <a id="the-hidden-attribute&#58;attr-hidden-8"></a><code>[hidden](#attr-hidden)</code> attribute must not be used to hide content that could legitimately be shown in another presentation. For example, it is incorrect to use <a id="the-hidden-attribute&#58;attr-hidden-9"></a><code>[hidden](#attr-hidden)</code> to hide panels in a tabbed dialog, because the tabbed interface is merely a kind of overflow presentation — one could equally well just show all the form controls in one big page with a scrollbar. It is similarly incorrect to use this attribute to hide content just from one presentation — if something is marked <a id="the-hidden-attribute&#58;attr-hidden-10"></a><code>[hidden](#attr-hidden)</code>, it is hidden from all presentations, including, for instance, screen readers.

Elements that are not themselves <a id="the-hidden-attribute&#58;attr-hidden-11"></a><code>[hidden](#attr-hidden)</code> must not <a id="the-hidden-attribute&#58;hyperlink"></a>[hyperlink](https://html.spec.whatwg.org/multipage/links.html#hyperlink) to elements that are <a id="the-hidden-attribute&#58;attr-hidden-12"></a><code>[hidden](#attr-hidden)</code>. The <code>for</code> attributes of <a id="the-hidden-attribute&#58;the-label-element"></a><code>[label](https://html.spec.whatwg.org/multipage/forms.html#the-label-element)</code> and <a id="the-hidden-attribute&#58;the-output-element"></a><code>[output](https://html.spec.whatwg.org/multipage/form-elements.html#the-output-element)</code> elements that are not themselves <a id="the-hidden-attribute&#58;attr-hidden-13"></a><code>[hidden](#attr-hidden)</code> must similarly not refer to elements that are <a id="the-hidden-attribute&#58;attr-hidden-14"></a><code>[hidden](#attr-hidden)</code>. In both cases, such references would cause user confusion.

Elements and scripts may, however, refer to elements that are <a id="the-hidden-attribute&#58;attr-hidden-15"></a><code>[hidden](#attr-hidden)</code> in other contexts.

<strong>Example:</strong>

For example, it would be incorrect to use the <a id="the-hidden-attribute&#58;attr-hyperlink-href"></a><code>[href](https://html.spec.whatwg.org/multipage/links.html#attr-hyperlink-href)</code> attribute to link to a section marked with the <a id="the-hidden-attribute&#58;attr-hidden-16"></a><code>[hidden](#attr-hidden)</code> attribute. If the content is not applicable or relevant, then there is no reason to link to it.

It would be fine, however, to use the ARIA <a id="the-hidden-attribute&#58;attr-aria-describedby"></a><code>[aria-describedby](https://w3c.github.io/aria/#aria-describedby)</code> attribute to refer to descriptions that are themselves <a id="the-hidden-attribute&#58;attr-hidden-17"></a><code>[hidden](#attr-hidden)</code>. While hiding the descriptions implies that they are not useful alone, they could be written in such a way that they are useful in the specific context of being referenced from the elements that they describe.

Similarly, a <a id="the-hidden-attribute&#58;the-canvas-element"></a><code>[canvas](https://html.spec.whatwg.org/multipage/canvas.html#the-canvas-element)</code> element with the <a id="the-hidden-attribute&#58;attr-hidden-18"></a><code>[hidden](#attr-hidden)</code> attribute could be used by a scripted graphics engine as an off-screen buffer, and a form control could refer to a hidden <a id="the-hidden-attribute&#58;the-form-element"></a><code>[form](https://html.spec.whatwg.org/multipage/forms.html#the-form-element)</code> element using its <a id="the-hidden-attribute&#58;attr-fae-form"></a><code>[form](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#attr-fae-form)</code> attribute.

Elements in a section hidden by the <a id="the-hidden-attribute&#58;attr-hidden-19"></a><code>[hidden](#attr-hidden)</code> attribute are still active, e.g. scripts and form controls in such sections still execute and submit respectively. Only their presentation to the user changes.

<strong>✔</strong>MDN

[HTMLElement/hidden](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/hidden)

Support in all current engines.

Firefox4+Safari5.1+Chrome6+

------------------------------------------------------------------------

Opera11.6+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer11

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android3+Samsung Internet?Opera Android12+

The <a id="dom-hidden"></a><strong><code>hidden</code></strong> getter steps are:

1.  If the <a id="the-hidden-attribute&#58;attr-hidden-20"></a><code>[hidden](#attr-hidden)</code> attribute is in the <a id="the-hidden-attribute&#58;attr-hidden-until-found-state-9"></a>[Hidden Until Found](#attr-hidden-until-found-state) state, then return "<a id="the-hidden-attribute&#58;attr-hidden-until-found"></a><code>[until-found](#attr-hidden-until-found)</code>".

2.  If the <a id="the-hidden-attribute&#58;attr-hidden-21"></a><code>[hidden](#attr-hidden)</code> attribute is set, then return true.

3.  Return false.

The <a id="the-hidden-attribute&#58;dom-hidden"></a><code>[hidden](#dom-hidden)</code> setter steps are:

1.  If the given value is a string that is an <a id="the-hidden-attribute&#58;ascii-case-insensitive"></a>[ASCII case-insensitive](https://infra.spec.whatwg.org/#ascii-case-insensitive) match for "<a id="the-hidden-attribute&#58;attr-hidden-until-found-2"></a><code>[until-found](#attr-hidden-until-found)</code>", then set the <a id="the-hidden-attribute&#58;attr-hidden-22"></a><code>[hidden](#attr-hidden)</code> attribute to "<a id="the-hidden-attribute&#58;attr-hidden-until-found-3"></a><code>[until-found](#attr-hidden-until-found)</code>".

2.  Otherwise, if the given value is false, then remove the <a id="the-hidden-attribute&#58;attr-hidden-23"></a><code>[hidden](#attr-hidden)</code> attribute.

3.  Otherwise, if the given value is the empty string, then remove the <a id="the-hidden-attribute&#58;attr-hidden-24"></a><code>[hidden](#attr-hidden)</code> attribute.

4.  Otherwise, if the given value is null, then remove the <a id="the-hidden-attribute&#58;attr-hidden-25"></a><code>[hidden](#attr-hidden)</code> attribute.

5.  Otherwise, if the given value is 0, then remove the <a id="the-hidden-attribute&#58;attr-hidden-26"></a><code>[hidden](#attr-hidden)</code> attribute.

6.  Otherwise, if the given value is NaN, then remove the <a id="the-hidden-attribute&#58;attr-hidden-27"></a><code>[hidden](#attr-hidden)</code> attribute.

7.  Otherwise, set the <a id="the-hidden-attribute&#58;attr-hidden-28"></a><code>[hidden](#attr-hidden)</code> attribute to the empty string.

An <a id="ancestor-reveal-pair"></a><strong>ancestor reveal pair</strong> is a <a id="the-hidden-attribute&#58;tuple"></a>[tuple](https://infra.spec.whatwg.org/#tuple) consisting of a <a id="ancestor-reveal-pair-node"></a><strong>node</strong> and a <a id="ancestor-reveal-pair-string"></a><strong>string</strong>.

The <a id="ancestor-revealing-algorithm"></a><strong>ancestor revealing algorithm</strong> given a node <var>target</var> is:

1.  Let <var>ancestorsToReveal</var> be « ».

2.  Let <var>ancestor</var> be <var>target</var>.

3.  While <var>ancestor</var> has a parent node within the <a id="the-hidden-attribute&#58;flat-tree"></a>[flat tree](https://drafts.csswg.org/css-scoping/#flat-tree):

    1.  If <var>ancestor</var> has a <a id="the-hidden-attribute&#58;attr-hidden-29"></a><code>[hidden](#attr-hidden)</code> attribute in the <a id="the-hidden-attribute&#58;attr-hidden-until-found-state-10"></a>[Hidden Until Found](#attr-hidden-until-found-state) state, then <a id="the-hidden-attribute&#58;list-append"></a>[append](https://infra.spec.whatwg.org/#list-append) (<var>ancestor</var>, "<code>until-found</code>") to <var>ancestorsToReveal</var>.

    2.  If <var>ancestor</var> is slotted into the second slot of a <a id="the-hidden-attribute&#58;the-details-element"></a><code>[details](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-details-element)</code> element which does not have an <a id="the-hidden-attribute&#58;attr-details-open"></a><code>[open](https://html.spec.whatwg.org/multipage/interactive-elements.html#attr-details-open)</code> attribute, then <a id="the-hidden-attribute&#58;list-append-2"></a>[append](https://infra.spec.whatwg.org/#list-append) (<var>ancestor</var>'s parent node, "<code>details</code>") to <var>ancestorsToReveal</var>.

    3.  Set <var>ancestor</var> to the parent node of <var>ancestor</var> within the <a id="the-hidden-attribute&#58;flat-tree-2"></a>[flat tree](https://drafts.csswg.org/css-scoping/#flat-tree).

4.  For each (<var>ancestorToReveal</var>, <var>revealType</var>) of <var>ancestorsToReveal</var>:

    1.  If <var>ancestorToReveal</var> is not <a id="the-hidden-attribute&#58;connected"></a>[connected](https://dom.spec.whatwg.org/#connected), then return.

    2.  If <var>revealType</var> is "<code>until-found</code>":

        1.  If <var>ancestorToReveal</var>'s <a id="the-hidden-attribute&#58;attr-hidden-30"></a><code>[hidden](#attr-hidden)</code> attribute is not in the <a id="the-hidden-attribute&#58;attr-hidden-until-found-state-11"></a>[Hidden Until Found](#attr-hidden-until-found-state) state, then return.

        2.  <a id="the-hidden-attribute&#58;concept-event-fire"></a>[Fire an event](https://dom.spec.whatwg.org/#concept-event-fire) named <a id="the-hidden-attribute&#58;event-beforematch"></a><code>[beforematch](https://html.spec.whatwg.org/multipage/indices.html#event-beforematch)</code> at <var>ancestorToReveal</var> with the <a id="the-hidden-attribute&#58;dom-event-bubbles"></a><code>[bubbles](https://dom.spec.whatwg.org/#dom-event-bubbles)</code> attribute initialized to true.

        3.  If <var>ancestorToReveal</var> is not <a id="the-hidden-attribute&#58;connected-2"></a>[connected](https://dom.spec.whatwg.org/#connected), then return.

        4.  If <var>ancestorToReveal</var>'s <a id="the-hidden-attribute&#58;attr-hidden-31"></a><code>[hidden](#attr-hidden)</code> attribute is not in the <a id="the-hidden-attribute&#58;attr-hidden-until-found-state-12"></a>[Hidden Until Found](#attr-hidden-until-found-state) state, then return.

        5.  Remove the <a id="the-hidden-attribute&#58;attr-hidden-32"></a><code>[hidden](#attr-hidden)</code> attribute from <var>ancestorToReveal</var>.

    3.  Otherwise:

        1.  <a id="the-hidden-attribute&#58;assert"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>revealType</var> is "<code>details</code>".

        2.  If <var>ancestorToReveal</var> has an <a id="the-hidden-attribute&#58;attr-details-open-2"></a><code>[open](https://html.spec.whatwg.org/multipage/interactive-elements.html#attr-details-open)</code> attribute, then return.

        3.  Set <var>ancestorToReveal</var>'s <a id="the-hidden-attribute&#58;attr-details-open-3"></a><code>[open](https://html.spec.whatwg.org/multipage/interactive-elements.html#attr-details-open)</code> attribute to the empty string.

### <a id="inert-subtrees"></a>6.3 Inert subtrees[](#inert-subtrees)

<strong>Note:</strong>

See also <a id="inert-subtrees&#58;the-inert-attribute"></a><code>[inert](#the-inert-attribute)</code> for an explanation of the attribute of the same name.

A node (in particular elements and text nodes) can be <a id="inert"></a><strong>inert</strong>. When a node is <a id="inert-subtrees&#58;inert"></a>[inert](#inert):

- Hit-testing must act as if the <a id="inert-subtrees&#58;'pointer-events'"></a>['pointer-events'](https://drafts.csswg.org/css-ui-4/#pointer-events-control) CSS property were set to 'none'.

- Text selection functionality must act as if the <a id="inert-subtrees&#58;'user-select'"></a>['user-select'](https://drafts.csswg.org/css-ui-4/#content-selection) CSS property were set to 'none'.

- If it is <a id="inert-subtrees&#58;editable"></a>[editable](https://w3c.github.io/editing/docs/execCommand/#editable), the node behaves as if it were non-editable.

- The user agent should ignore the node for the purposes of <a id="inert-subtrees&#58;find-in-page-2"></a>[find-in-page](https://html.spec.whatwg.org/multipage/interaction.html#find-in-page-2).

<strong>Note:</strong>

Inert nodes generally cannot be focused, and user agents do not expose the inert nodes to accessibility APIs or assistive technologies. Inert nodes that are <a id="inert-subtrees&#58;concept-command"></a>[commands](https://html.spec.whatwg.org/multipage/interactive-elements.html#concept-command) will become inoperable to users, in the manner described above.

User agents may allow the user to override the restrictions on <a id="inert-subtrees&#58;find-in-page-2-2"></a>[find-in-page](https://html.spec.whatwg.org/multipage/interaction.html#find-in-page-2) and text selection, however.

By default, a node is not <a id="inert-subtrees&#58;inert-2"></a>[inert](#inert).

#### <a id="modal-dialogs-and-inert-subtrees"></a>6.3.1 Modal dialogs and inert subtrees[](#modal-dialogs-and-inert-subtrees)

A <a id="modal-dialogs-and-inert-subtrees&#58;document"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> <var>document</var> is <a id="blocked-by-a-modal-dialog"></a><strong>blocked by a modal dialog</strong> <var>subject</var> if <var>subject</var> is the topmost <a id="modal-dialogs-and-inert-subtrees&#58;the-dialog-element"></a><code>[dialog](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element)</code> element in <var>document</var>'s <a id="modal-dialogs-and-inert-subtrees&#58;top-layer"></a>[top layer](https://drafts.csswg.org/css-position-4/#document-top-layer). While <var>document</var> is so blocked, every node that is <a id="modal-dialogs-and-inert-subtrees&#58;connected"></a>[connected](https://dom.spec.whatwg.org/#connected) to <var>document</var>, with the exception of the <var>subject</var> element and its <a id="modal-dialogs-and-inert-subtrees&#58;flat-tree"></a>[flat tree](https://drafts.csswg.org/css-scoping/#flat-tree) descendants, must become <a id="modal-dialogs-and-inert-subtrees&#58;inert"></a>[inert](#inert).

<var>subject</var> can additionally become <a id="modal-dialogs-and-inert-subtrees&#58;inert-2"></a>[inert](#inert) via the <a id="modal-dialogs-and-inert-subtrees&#58;the-inert-attribute"></a><code>[inert](#the-inert-attribute)</code> attribute, but only if specified on <var>subject</var> itself (i.e., <var>subject</var> escapes inertness of ancestors); <var>subject</var>'s <a id="modal-dialogs-and-inert-subtrees&#58;flat-tree-2"></a>[flat tree](https://drafts.csswg.org/css-scoping/#flat-tree) descendants can become <a id="modal-dialogs-and-inert-subtrees&#58;inert-3"></a>[inert](#inert) in a similar fashion.

<strong>Note:</strong>

The <a id="modal-dialogs-and-inert-subtrees&#58;the-dialog-element-2"></a><code>[dialog](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element)</code> element's <a id="modal-dialogs-and-inert-subtrees&#58;dom-dialog-showmodal"></a><code>[showModal()](https://html.spec.whatwg.org/multipage/interactive-elements.html#dom-dialog-showmodal)</code> method causes this mechanism to trigger, by <a id="modal-dialogs-and-inert-subtrees&#58;add-an-element-to-the-top-layer"></a>[adding](https://drafts.csswg.org/css-position-4/#add-an-element-to-the-top-layer) the <a id="modal-dialogs-and-inert-subtrees&#58;the-dialog-element-3"></a><code>[dialog](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element)</code> element to its <a id="modal-dialogs-and-inert-subtrees&#58;node-document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)'s <a id="modal-dialogs-and-inert-subtrees&#58;top-layer-2"></a>[top layer](https://drafts.csswg.org/css-position-4/#document-top-layer).

#### <a id="the-inert-attribute"></a>6.3.2 The <strong><code>inert</code></strong> attribute[](#the-inert-attribute)

<strong>✔</strong>MDN

[Global_attributes/inert](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/inert)

Support in all current engines.

Firefox112+Safari15.5+Chrome102+

------------------------------------------------------------------------

Opera?Edge102+

------------------------------------------------------------------------

Edge (Legacy)?Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android?Samsung Internet?Opera Android?

The <a id="the-inert-attribute&#58;the-inert-attribute"></a><code>[inert](#the-inert-attribute)</code> attribute is a <a id="the-inert-attribute&#58;boolean-attribute"></a>[boolean attribute](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#boolean-attribute) that indicates, by its presence, that the element and all its <a id="the-inert-attribute&#58;flat-tree"></a>[flat tree](https://drafts.csswg.org/css-scoping/#flat-tree) descendants which don't otherwise escape inertness (such as modal dialogs) are to be made <a id="the-inert-attribute&#58;inert"></a>[inert](#inert) by the user agent.

An inert subtree should not contain any content or controls which are critical to understanding or using aspects of the page which are not in the inert state. Content in an inert subtree will not be perceivable by all users, or interactive. Authors should not specify elements as inert unless the content they represent are also visually obscured in some way. In most cases, authors should not specify the <a id="the-inert-attribute&#58;inert-2"></a><code>[inert](#inert)</code> attribute on individual form controls. In these instances, the <a id="the-inert-attribute&#58;attr-fe-disabled"></a><code>[disabled](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#attr-fe-disabled)</code> attribute is probably more appropriate.

<strong>Example:</strong>

The following example shows how to mark partially loaded content, visually obscured by a "loading" message, as inert.

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

![Screenshot of Population by City content with an overlaid loading message which visually obscures the form controls and data table which have not fully rendered, and thus are in the inert state.](https://html.spec.whatwg.org/images/inert-example-loading-section.png)

The "loading" overlay obscures the inert content, making it visually apparent that the inert content is not presently accessible. Notice that the heading and "loading" text are not descendants of the element with the <a id="the-inert-attribute&#58;inert-3"></a><code>[inert](#inert)</code> attribute. This will ensure this text is accessible to all users, while the inert content cannot be interacted with by anyone.

<strong>Note:</strong>

By default, there is no persistent visual indication of an element or its subtree being inert. Appropriate visual styles for such content is often context-dependent. For instance, an inert off-screen navigation panel would not require a default style, as its off-screen position visually obscures the content. Similarly, a modal <a id="the-inert-attribute&#58;the-dialog-element"></a><code>[dialog](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element)</code> element's backdrop will serve as the means to visually obscure the inert content of the web page, rather than styling the inert content specifically.

However, for many other situations authors are strongly encouraged to clearly mark what parts of their document are active and which are inert, to avoid user confusion. In particular, it is worth remembering that not all users can see all parts of a page at once; for example, users of screen readers, users on small devices or with magnifiers, and even users using particularly small windows might not be able to see the active part of a page and might get frustrated if inert sections are not obviously inert.

#### <a id="user-activation-data-model"></a>6.4.1 Data model[](#user-activation-data-model)

For the purpose of tracking user activation, each <a id="user-activation-data-model&#58;window"></a><code>[Window](https://html.spec.whatwg.org/multipage/nav-history-apis.html#window)</code> <var>W</var> has two relevant values:

- A <a id="last-activation-timestamp"></a><strong>last activation timestamp</strong>, which is either a <a id="user-activation-data-model&#58;domhighrestimestamp"></a><code>[DOMHighResTimeStamp](https://w3c.github.io/hr-time/#dom-domhighrestimestamp)</code>, positive infinity (indicating that <var>W</var> has never been activated), or negative infinity (indicating that the activation has been <a id="user-activation-data-model&#58;consume-user-activation"></a>[consumed](https://html.spec.whatwg.org/multipage/interaction.html#consume-user-activation)). Initially positive infinity.

- A <a id="last-history-action-activation-timestamp"></a><strong>last history-action activation timestamp</strong>, which is either a <a id="user-activation-data-model&#58;domhighrestimestamp-2"></a><code>[DOMHighResTimeStamp](https://w3c.github.io/hr-time/#dom-domhighrestimestamp)</code> or positive infinity, initially positive infinity.

A user agent also defines a <a id="transient-activation-duration"></a><strong>transient activation duration</strong>, which is a constant number indicating how long a user activation is available for certain [user activation-gated APIs](https://html.spec.whatwg.org/multipage/interaction.html#user-activation-gated-apis) (e.g., for opening popups).

<strong>Note:</strong>

The <a id="user-activation-data-model&#58;transient-activation-duration"></a>[transient activation duration](#transient-activation-duration) is expected be at most a few seconds, so that the user can possibly perceive the link between an interaction with the page and the page calling the activation-gated API.

We then have the following boolean user activation states for <var>W</var>:

<strong><a id="sticky-activation"></a><strong>Sticky activation</strong></strong>

When the <a id="user-activation-data-model&#58;current-high-resolution-time"></a>[current high resolution time](https://w3c.github.io/hr-time/#dfn-current-high-resolution-time) given <var>W</var> is greater than or equal to the <a id="user-activation-data-model&#58;last-activation-timestamp"></a>[last activation timestamp](#last-activation-timestamp) in <var>W</var>, <var>W</var> is said to have <a id="user-activation-data-model&#58;sticky-activation"></a>[sticky activation](#sticky-activation).

This is <var>W</var>'s historical activation state, indicating whether the user has ever interacted in <var>W</var>. It starts false, then changes to true (and never changes back to false) when <var>W</var> gets the very first <a id="user-activation-data-model&#58;activation-notification"></a>[activation notification](https://html.spec.whatwg.org/multipage/interaction.html#activation-notification).

<strong><a id="transient-activation"></a><strong>Transient activation</strong></strong>

When the <a id="user-activation-data-model&#58;current-high-resolution-time-2"></a>[current high resolution time](https://w3c.github.io/hr-time/#dfn-current-high-resolution-time) given <var>W</var> is greater than or equal to the <a id="user-activation-data-model&#58;last-activation-timestamp-2"></a>[last activation timestamp](#last-activation-timestamp) in <var>W</var>, and less than the <a id="user-activation-data-model&#58;last-activation-timestamp-3"></a>[last activation timestamp](#last-activation-timestamp) in <var>W</var> plus the <a id="user-activation-data-model&#58;transient-activation-duration-2"></a>[transient activation duration](#transient-activation-duration), then <var>W</var> is said to have <a id="user-activation-data-model&#58;transient-activation"></a>[transient activation](#transient-activation).

This is <var>W</var>'s current activation state, indicating whether the user has interacted in <var>W</var> recently. This starts with a false value, and remains true for a limited time after every <a id="user-activation-data-model&#58;activation-notification-2"></a>[activation notification](https://html.spec.whatwg.org/multipage/interaction.html#activation-notification) <var>W</var> gets.

The <a id="user-activation-data-model&#58;transient-activation-2"></a>[transient activation](#transient-activation) state is considered <a id="activation-expiry"></a><strong>expired</strong> if it becomes false because the <a id="user-activation-data-model&#58;transient-activation-duration-3"></a>[transient activation duration](#transient-activation-duration) time has elapsed since the last user activation. Note that it can become false even before the expiry time through an <a id="user-activation-data-model&#58;consume-user-activation-2"></a>[activation consumption](https://html.spec.whatwg.org/multipage/interaction.html#consume-user-activation).

<strong><a id="history-action-activation"></a><strong>History-action activation</strong></strong>

When the <a id="user-activation-data-model&#58;last-history-action-activation-timestamp"></a>[last history-action activation timestamp](#last-history-action-activation-timestamp) of <var>W</var> is not equal to the <a id="user-activation-data-model&#58;last-activation-timestamp-4"></a>[last activation timestamp](#last-activation-timestamp) of <var>W</var>, then <var>W</var> is said to have <a id="user-activation-data-model&#58;history-action-activation"></a>[history-action activation](#history-action-activation).

This is a special variant of user activation, used to allow access to certain session history APIs which, if used too frequently, would make it harder for the user to traverse back using [browser UI](https://html.spec.whatwg.org/multipage/speculative-loading.html#nav-traversal-ui). It starts with a false value, and becomes true whenever the user interacts with <var>W</var>, but is reset to false through <a id="user-activation-data-model&#58;consume-history-action-user-activation"></a>[history-action activation consumption](https://html.spec.whatwg.org/multipage/interaction.html#consume-history-action-user-activation). This ensures such APIs cannot be used multiple times in a row without an intervening user activation. But unlike <a id="user-activation-data-model&#58;transient-activation-3"></a>[transient activation](#transient-activation), there is no time limit within which such APIs must be used.

<strong>Note:</strong>

The <a id="user-activation-data-model&#58;last-activation-timestamp-5"></a>[last activation timestamp](#last-activation-timestamp) and <a id="user-activation-data-model&#58;last-history-action-activation-timestamp-2"></a>[last history-action activation timestamp](#last-history-action-activation-timestamp) are retained even after the <a id="user-activation-data-model&#58;document"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> changes its <a id="user-activation-data-model&#58;fully-active"></a>[fully active](https://html.spec.whatwg.org/multipage/document-sequences.html#fully-active) status (e.g., after navigating away from a <a id="user-activation-data-model&#58;document-2"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code>, or navigating to a cached <a id="user-activation-data-model&#58;document-3"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code>). This means <a id="user-activation-data-model&#58;sticky-activation-2"></a>[sticky activation](#sticky-activation) state spans multiple navigations as long as the same <a id="user-activation-data-model&#58;document-4"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> gets reused. For the transient activation state, the original <a id="user-activation-data-model&#58;activation-expiry"></a>[expiry](#activation-expiry) time remains unchanged (i.e., the state still expires within the <a id="user-activation-data-model&#58;transient-activation-duration-4"></a>[transient activation duration](#transient-activation-duration) limit from the original <a id="user-activation-data-model&#58;activation-triggering-input-event"></a>[activation triggering input event](https://html.spec.whatwg.org/multipage/interaction.html#activation-triggering-input-event)). It is important to consider this when deciding whether to base certain things off <a id="user-activation-data-model&#58;sticky-activation-3"></a>[sticky activation](#sticky-activation) or <a id="user-activation-data-model&#58;transient-activation-4"></a>[transient activation](#transient-activation).

### <a id="focus"></a>6.6 Focus[](#focus)

#### <a id="introduction-8"></a>6.6.1 Introduction[](#introduction-8)

<em>This section is non-normative.</em>

An HTML user interface typically consists of multiple interactive widgets, such as form controls, scrollable regions, links, dialog boxes, browser tabs, and so forth. These widgets form a hierarchy, with some (e.g. browser tabs, dialog boxes) containing others (e.g. links, form controls).

When interacting with an interface using a keyboard, key input is channeled from the system, through the hierarchy of interactive widgets, to an active widget, which is said to be <a id="introduction-8&#58;focused"></a>[focused](#focused).

<strong>Example:</strong>

Consider an HTML application running in a browser tab running in a graphical environment. Suppose this application had a page with some text controls and links, and was currently showing a modal dialog, which itself had a text control and a button.

The hierarchy of focusable widgets, in this scenario, would include the browser window, which would have, amongst its children, the browser tab containing the HTML application. The tab itself would have as its children the various links and text controls, as well as the dialog. The dialog itself would have as its children the text control and the button.

![](https://html.spec.whatwg.org/images/focus-tree.png)

If the widget with <a id="introduction-8&#58;focused-2"></a>[focus](#focused) in this example was the text control in the dialog box, then key input would be channeled from the graphical system to ① the web browser, then to ② the tab, then to ③ the dialog, and finally to ④ the text control.

Keyboard <em>events</em> are always targeted at this <a id="introduction-8&#58;focused-3"></a>[focused](#focused) element.

#### <a id="data-model"></a>6.6.2 Data model[](#data-model)

<a id="tlbc-system-focus"></a>A <a id="data-model&#58;top-level-traversable"></a>[top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#top-level-traversable) has <a id="system-focus"></a><strong>system focus</strong> when it can receive keyboard input channeled from the operating system, possibly targeted at one of its <a id="data-model&#58;nav-document"></a>[active document](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-document)'s <a id="data-model&#58;descendant-navigables"></a>[descendant navigables](https://html.spec.whatwg.org/multipage/document-sequences.html#descendant-navigables).

A <a id="data-model&#58;top-level-traversable-2"></a>[top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#top-level-traversable) has <a id="user-attention"></a><strong>user attention</strong> when its <a id="data-model&#58;system-visibility-state"></a>[system visibility state](https://html.spec.whatwg.org/multipage/document-sequences.html#system-visibility-state) is "<code>visible</code>", and it either has <a id="data-model&#58;system-focus"></a>[system focus](#system-focus) or user agent widgets directly related to it can receive keyboard input channeled from the operating system.

<strong>Note:</strong>

User attention is lost when a browser window loses focus, whereas system focus might also be lost to other system widgets in the browser window such as a location bar.

A <a id="data-model&#58;document"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> <var>d</var> is a <a id="fully-active-descendant-of-a-top-level-traversable-with-user-attention"></a><strong>fully active descendant of a top-level traversable with user attention</strong> when <var>d</var> is <a id="data-model&#58;fully-active"></a>[fully active](https://html.spec.whatwg.org/multipage/document-sequences.html#fully-active) and <var>d</var>'s <a id="data-model&#58;node-navigable"></a>[node navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#node-navigable)'s <a id="data-model&#58;nav-top"></a>[top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-top) has <a id="data-model&#58;user-attention"></a>[user attention](#user-attention).

The term <a id="focusable-area"></a><strong>focusable area</strong> is used to refer to regions of the interface that can further become the target of such keyboard input. Focusable areas can be elements, parts of elements, or other regions managed by the user agent.

Each <a id="data-model&#58;focusable-area"></a>[focusable area](#focusable-area) has a <a id="dom-anchor"></a><strong>DOM anchor</strong>, which is a <a id="data-model&#58;node"></a><code>[Node](https://dom.spec.whatwg.org/#interface-node)</code> object that represents the position of the <a id="data-model&#58;focusable-area-2"></a>[focusable area](#focusable-area) in the DOM. (When the <a id="data-model&#58;focusable-area-3"></a>[focusable area](#focusable-area) is itself a <a id="data-model&#58;node-2"></a><code>[Node](https://dom.spec.whatwg.org/#interface-node)</code>, it is its own <a id="data-model&#58;dom-anchor"></a>[DOM anchor](#dom-anchor).) The <a id="data-model&#58;dom-anchor-2"></a>[DOM anchor](#dom-anchor) is used in some APIs as a substitute for the <a id="data-model&#58;focusable-area-4"></a>[focusable area](#focusable-area) when there is no other DOM object to represent the <a id="data-model&#58;focusable-area-5"></a>[focusable area](#focusable-area).

The following table describes what objects can be <a id="data-model&#58;focusable-area-6"></a>[focusable areas](#focusable-area). The cells in the left column describe objects that can be <a id="data-model&#58;focusable-area-7"></a>[focusable areas](#focusable-area); the cells in the right column describe the <a id="data-model&#58;dom-anchor-3"></a>[DOM anchors](#dom-anchor) for those elements. (The cells that span both columns are non-normative examples.)

<table id="table-fa"><thead><tr><th id="th-fa-area"><a href="#focusable-area" id="data-model:focusable-area-8">Focusable area</a>      </th><th id="th-fa-dom-rep"><a href="#dom-anchor" id="data-model:dom-anchor-4">DOM anchor</a>     </th></tr><tr><th colspan="2" id="th-fa-examples">Examples     </th></tr></thead><tbody><tr><th headers="th-fa-area" id="td-fa-1" class="data-header">      Elements that meet all the following criteria:       <ul class="brief"><li>the element's <a href="#tabindex-value" id="data-model:tabindex-value">tabindex value</a> is non-null, or the element is determined by the       user agent to be focusable;</li><li>the element is either not a <a id="data-model:shadow-host" href="https://dom.spec.whatwg.org/#element-shadow-host" data-x-internal="shadow-host">shadow host</a>, or has a <a href="https://dom.spec.whatwg.org/#concept-element-shadow-root" id="data-model:concept-element-shadow-root" data-x-internal="concept-element-shadow-root">shadow root</a> whose <a id="data-model:delegates-focus" href="https://dom.spec.whatwg.org/#shadowroot-delegates-focus" data-x-internal="delegates-focus">delegates focus</a> is       false;</li><li>the element is not <a href="https://html.spec.whatwg.org/multipage/semantics-other.html#concept-element-disabled" id="data-model:concept-element-disabled">actually disabled</a>;</li><li>the element is not <a href="#inert" id="data-model:inert">inert</a>;</li><li>the element is either <a id="data-model:being-rendered" href="https://html.spec.whatwg.org/multipage/rendering.html#being-rendered">being rendered</a>, <a id="data-model:delegating-its-rendering-to-its-children" href="https://html.spec.whatwg.org/multipage/rendering.html#delegating-its-rendering-to-its-children">delegating its rendering to its       children</a>, or <a id="data-model:being-used-as-relevant-canvas-fallback-content" href="https://html.spec.whatwg.org/multipage/canvas.html#being-used-as-relevant-canvas-fallback-content">being used as relevant canvas fallback content</a>.</li></ul>      </th><td headers="td-fa-1 th-fa-dom-rep">      The element itself.     </td></tr><tr><td colspan="2" headers="td-fa-1 th-fa-examples">      <p class="example"><code id="data-model:the-iframe-element"><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code>, <code id="data-model:the-dialog-element"><a href="https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element">dialog</a></code>, <code id="data-model:text-(type=text)-state-and-search-state-(type=search)"><a href="https://html.spec.whatwg.org/multipage/input.html#text-(type=text)-state-and-search-state-(type=search)">&lt;input type=text&gt;</a></code>, sometimes <code id="data-model:the-a-element"><a href="https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element">&lt;a&#10;     href=""&gt;</a></code> (depending on platform conventions).     </p></td></tr></tbody><tbody><tr><th headers="th-fa-area" id="td-fa-2" class="data-header">      The shapes of <code id="data-model:the-area-element"><a href="https://html.spec.whatwg.org/multipage/image-maps.html#the-area-element">area</a></code> elements in an <a id="data-model:image-map" href="https://html.spec.whatwg.org/multipage/image-maps.html#image-map">image map</a> associated with an      <code id="data-model:the-img-element"><a href="https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element">img</a></code> element that is <a id="data-model:being-rendered-2" href="https://html.spec.whatwg.org/multipage/rendering.html#being-rendered">being rendered</a> and is not <a href="#inert" id="data-model:inert-2">inert</a>.      </th><td headers="td-fa-2 th-fa-dom-rep">      The <code id="data-model:the-img-element-2"><a href="https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element">img</a></code> element.     </td></tr><tr><td colspan="2" headers="td-fa-2 th-fa-examples">      <div class="example">       <p>In the following example, the <code id="data-model:the-area-element-2"><a href="https://html.spec.whatwg.org/multipage/image-maps.html#the-area-element">area</a></code> element creates two shapes, one on each       image. The <a href="#dom-anchor" id="data-model:dom-anchor-5">DOM anchor</a> of the first shape is the first <code id="data-model:the-img-element-3"><a href="https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element">img</a></code> element, and the       <a href="#dom-anchor" id="data-model:dom-anchor-6">DOM anchor</a> of the second shape is the second <code id="data-model:the-img-element-4"><a href="https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element">img</a></code> element.</p>       <code>&lt;map id=wallmap&gt;&lt;area alt="Enter Door" coords="10,10,100,200" href="door.html"&gt;&lt;/map&gt;&#10;...&#10;&lt;img src="images/innerwall.jpeg" alt="There is a white wall here, with a door." usemap="#wallmap"&gt;&#10;...&#10;&lt;img src="images/outerwall.jpeg" alt="There is a red wall here, with a door." usemap="#wallmap"&gt;</code>      </div>     </td></tr></tbody><tbody><tr><th headers="th-fa-area" id="td-fa-3" class="data-header">      The user-agent provided subwidgets of elements that are <a id="data-model:being-rendered-3" href="https://html.spec.whatwg.org/multipage/rendering.html#being-rendered">being rendered</a> and are not      <a href="https://html.spec.whatwg.org/multipage/semantics-other.html#concept-element-disabled" id="data-model:concept-element-disabled-2">actually disabled</a> or <a href="#inert" id="data-model:inert-3">inert</a>.      </th><td headers="td-fa-3 th-fa-dom-rep">      The element for which the <a href="#focusable-area" id="data-model:focusable-area-9">focusable area</a> is a subwidget.     </td></tr><tr><td colspan="2" headers="td-fa-3 th-fa-examples">      <p class="example">The <a href="https://html.spec.whatwg.org/multipage/media.html#expose-a-user-interface-to-the-user" id="data-model:expose-a-user-interface-to-the-user">controls in the user      interface</a> for a <code id="data-model:the-video-element"><a href="https://html.spec.whatwg.org/multipage/media.html#the-video-element">video</a></code> element, the up and down buttons in a spin-control      version of <code id="data-model:number-state-(type=number)"><a href="https://html.spec.whatwg.org/multipage/input.html#number-state-(type=number)">&lt;input type=number&gt;</a></code>, the part of a      <code id="data-model:the-details-element"><a href="https://html.spec.whatwg.org/multipage/interactive-elements.html#the-details-element">details</a></code> element's rendering that enables the element to be opened or closed using      keyboard input.</p>     </td></tr></tbody><tbody><tr><th headers="th-fa-area" id="td-fa-4" class="data-header">      The scrollable regions of elements that are <a id="data-model:being-rendered-4" href="https://html.spec.whatwg.org/multipage/rendering.html#being-rendered">being rendered</a> and are not      <a href="#inert" id="data-model:inert-4">inert</a>.       </th><td headers="td-fa-4 th-fa-dom-rep">      The element for which the box that the scrollable region scrolls was created.     </td></tr><tr><td colspan="2" headers="td-fa-4 th-fa-examples">      <p class="example">The CSS <a id="data-model:'overflow'" href="https://drafts.csswg.org/css-overflow/#propdef-overflow" data-x-internal="'overflow'">'overflow'</a> property's 'scroll' value typically creates      a scrollable region.</p>     </td></tr></tbody><tbody><tr><th headers="th-fa-area" id="td-fa-5" class="data-header">      The <a id="data-model:viewport" href="https://drafts.csswg.org/css2/#viewport" data-x-internal="viewport">viewport</a> of a <code id="data-model:document-2"><a href="https://html.spec.whatwg.org/multipage/dom.html#document">Document</a></code> that has a non-null <a href="https://html.spec.whatwg.org/multipage/document-sequences.html#concept-document-bc" id="data-model:concept-document-bc">browsing context</a> and is not <a href="#inert" id="data-model:inert-5">inert</a>.      </th><td headers="td-fa-5 th-fa-dom-rep">      The <code id="data-model:document-3"><a href="https://html.spec.whatwg.org/multipage/dom.html#document">Document</a></code> for which the <a id="data-model:viewport-2" href="https://drafts.csswg.org/css2/#viewport" data-x-internal="viewport">viewport</a> was created.     </td></tr><tr><td colspan="2" headers="td-fa-5 th-fa-examples">      <p class="example">The contents of an <code id="data-model:the-iframe-element-2"><a href="https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element">iframe</a></code>.</p>     </td></tr></tbody><tbody><tr><th headers="th-fa-area" id="td-fa-6" class="data-header">      Any other element or part of an element determined by the user agent to be a focusable area,      especially to aid with accessibility or to better match platform conventions.      </th><td headers="td-fa-6 th-fa-dom-rep">      The element.     </td></tr><tr><td colspan="2" headers="td-fa-6 th-fa-examples">      <p class="example">A user agent could make all list item bullets <a href="#sequentially-focusable" id="data-model:sequentially-focusable">sequentially      focusable</a>, so that a user can more easily navigate lists.</p>      <p class="example">Similarly, a user agent could make all elements with <code id="data-model:attr-title"><a href="https://html.spec.whatwg.org/multipage/dom.html#attr-title">title</a></code> attributes <a href="#sequentially-focusable" id="data-model:sequentially-focusable-2">sequentially focusable</a>, so that their      advisory information can be accessed.</p>    </td></tr></tbody></table>

<a id="bc-focus-ergo-bcc-focus"></a>

<strong>Note:</strong>

[](#bc-focus-ergo-bcc-focus)A <a id="data-model&#58;navigable-container"></a>[navigable container](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable-container) (e.g. an <a id="data-model&#58;the-iframe-element-3"></a><code>[iframe](https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element)</code>) is a <a id="data-model&#58;focusable-area-10"></a>[focusable area](#focusable-area), but key events routed to a <a id="data-model&#58;navigable-container-2"></a>[navigable container](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable-container) get immediately routed to its <a id="data-model&#58;content-navigable"></a>[content navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#content-navigable)'s <a id="data-model&#58;nav-document-2"></a>[active document](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-document). Similarly, in sequential focus navigation a <a id="data-model&#58;navigable-container-3"></a>[navigable container](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable-container) essentially acts merely as a placeholder for its <a id="data-model&#58;content-navigable-2"></a>[content navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#content-navigable)'s <a id="data-model&#58;nav-document-3"></a>[active document](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-document).

------------------------------------------------------------------------

One <a id="data-model&#58;focusable-area-11"></a>[focusable area](#focusable-area) in each <a id="data-model&#58;document-4"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> is designated the <a id="focused-area-of-the-document"></a><strong>focused area of the document</strong>. Which control is so designated changes over time, based on algorithms in this specification.

<strong>Note:</strong>

Even if a document is not <a id="data-model&#58;fully-active-2"></a>[fully active](https://html.spec.whatwg.org/multipage/document-sequences.html#fully-active) and not shown to the user, it can still have a <a id="data-model&#58;focused-area-of-the-document"></a>[focused area of the document](#focused-area-of-the-document). If a document's <a id="data-model&#58;fully-active-3"></a>[fully active](https://html.spec.whatwg.org/multipage/document-sequences.html#fully-active) state changes, its <a id="data-model&#58;focused-area-of-the-document-2"></a>[focused area of the document](#focused-area-of-the-document) will stay the same.

<a id="currently-focused-area-of-a-top-level-browsing-context"></a>The <a id="currently-focused-area-of-a-top-level-traversable"></a><strong>currently focused area of a top-level traversable</strong> <var>traversable</var> is the <a id="data-model&#58;focusable-area-12"></a>[focusable area](#focusable-area)-or-null returned by this algorithm:

1.  If <var>traversable</var> does not have <a id="data-model&#58;system-focus-2"></a>[system focus](#system-focus), then return null.

2.  Let <var>candidate</var> be <var>traversable</var>'s <a id="data-model&#58;nav-document-4"></a>[active document](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-document).

3.  While <var>candidate</var>'s <a id="data-model&#58;focused-area-of-the-document-3"></a>[focused area](#focused-area-of-the-document) is a <a id="data-model&#58;navigable-container-4"></a>[navigable container](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable-container) with a non-null <a id="data-model&#58;content-navigable-3"></a>[content navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#content-navigable): set <var>candidate</var> to the <a id="data-model&#58;nav-document-5"></a>[active document](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-document) of that <a id="data-model&#58;navigable-container-5"></a>[navigable container](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable-container)'s <a id="data-model&#58;content-navigable-4"></a>[content navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#content-navigable).

4.  If <var>candidate</var>'s <a id="data-model&#58;focused-area-of-the-document-4"></a>[focused area](#focused-area-of-the-document) is non-null, set <var>candidate</var> to <var>candidate</var>'s <a id="data-model&#58;focused-area-of-the-document-5"></a>[focused area](#focused-area-of-the-document).

5.  Return <var>candidate</var>.

<a id="current-focus-chain-of-a-top-level-browsing-context"></a>The <a id="current-focus-chain-of-a-top-level-traversable"></a><strong>current focus chain of a top-level traversable</strong> <var>traversable</var> is the <a id="data-model&#58;focus-chain"></a>[focus chain](#focus-chain) of the <a id="data-model&#58;currently-focused-area-of-a-top-level-traversable"></a>[currently focused area](#currently-focused-area-of-a-top-level-traversable) of <var>traversable</var>, if <var>traversable</var> is non-null, or an empty list otherwise.

An element that is the <a id="data-model&#58;dom-anchor-7"></a>[DOM anchor](#dom-anchor) of a <a id="data-model&#58;focusable-area-13"></a>[focusable area](#focusable-area) is said to <a id="gains-focus"></a><strong>gain focus</strong> when that <a id="data-model&#58;focusable-area-14"></a>[focusable area](#focusable-area) becomes the <a id="data-model&#58;currently-focused-area-of-a-top-level-traversable-2"></a>[currently focused area of a top-level traversable](#currently-focused-area-of-a-top-level-traversable). When an element is the <a id="data-model&#58;dom-anchor-8"></a>[DOM anchor](#dom-anchor) of a <a id="data-model&#58;focusable-area-15"></a>[focusable area](#focusable-area) of the <a id="data-model&#58;currently-focused-area-of-a-top-level-traversable-3"></a>[currently focused area of a top-level traversable](#currently-focused-area-of-a-top-level-traversable), it is <a id="focused"></a><strong>focused</strong>.

The <a id="focus-chain"></a><strong>focus chain</strong> of a <a id="data-model&#58;focusable-area-16"></a>[focusable area](#focusable-area) <var>subject</var> is the ordered list constructed as follows:

1.  Let <var>output</var> be an empty <a id="data-model&#58;list"></a>[list](https://infra.spec.whatwg.org/#list).

2.  Let <var>currentObject</var> be <var>subject</var>.

3.  While true:

    1.  <a id="data-model&#58;list-append"></a>[Append](https://infra.spec.whatwg.org/#list-append) <var>currentObject</var> to <var>output</var>.

    2.  If <var>currentObject</var> is an <a id="data-model&#58;the-area-element-3"></a><code>[area](https://html.spec.whatwg.org/multipage/image-maps.html#the-area-element)</code> element's shape, then <a id="data-model&#58;list-append-2"></a>[append](https://infra.spec.whatwg.org/#list-append) that <a id="data-model&#58;the-area-element-4"></a><code>[area](https://html.spec.whatwg.org/multipage/image-maps.html#the-area-element)</code> element to <var>output</var>.

        Otherwise, if <var>currentObject</var>'s <a id="data-model&#58;dom-anchor-9"></a>[DOM anchor](#dom-anchor) is an element that is not <var>currentObject</var> itself, then <a id="data-model&#58;list-append-3"></a>[append](https://infra.spec.whatwg.org/#list-append) <var>currentObject</var>'s <a id="data-model&#58;dom-anchor-10"></a>[DOM anchor](#dom-anchor) to <var>output</var>.

    3.  If <var>currentObject</var> is a <a id="data-model&#58;focusable-area-17"></a>[focusable area](#focusable-area), then set <var>currentObject</var> to <var>currentObject</var>'s <a id="data-model&#58;dom-anchor-11"></a>[DOM anchor](#dom-anchor)'s <a id="data-model&#58;node-document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document).

        Otherwise, if <var>currentObject</var> is a <a id="data-model&#58;document-5"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> whose <a id="data-model&#58;node-navigable-2"></a>[node navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#node-navigable)'s <a id="data-model&#58;nav-parent"></a>[parent](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-parent) is non-null, then set <var>currentObject</var> to <var>currentObject</var>'s <a id="data-model&#58;node-navigable-3"></a>[node navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#node-navigable)'s <a id="data-model&#58;nav-parent-2"></a>[parent](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-parent).

        Otherwise, <a id="data-model&#58;break"></a>[break](https://infra.spec.whatwg.org/#iteration-break).

4.  Return <var>output</var>.

    <strong>Note:</strong>

    The chain starts with <var>subject</var> and (if <var>subject</var> is or can be the <a id="data-model&#58;currently-focused-area-of-a-top-level-traversable-4"></a>[currently focused area of a top-level traversable](#currently-focused-area-of-a-top-level-traversable)) continues up the focus hierarchy up to the <a id="data-model&#58;document-6"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> of the <a id="data-model&#58;top-level-traversable-3"></a>[top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#top-level-traversable).

All elements that are <a id="data-model&#58;focusable-area-18"></a>[focusable areas](#focusable-area) are said to be <a id="focusable"></a><strong>focusable</strong>.

There are two special types of focusability for <a id="data-model&#58;focusable-area-19"></a>[focusable areas](#focusable-area):

- A <a id="data-model&#58;focusable-area-20"></a>[focusable area](#focusable-area) is said to be <a id="sequentially-focusable"></a><strong>sequentially focusable</strong> if it is included in its <a id="data-model&#58;document-7"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code>'s <a id="data-model&#58;sequential-focus-navigation-order"></a>[sequential focus navigation order](#sequential-focus-navigation-order) and the user agent determines that it is sequentially focusable.

- A <a id="data-model&#58;focusable-area-21"></a>[focusable area](#focusable-area) is said to be <a id="click-focusable"></a><strong>click focusable</strong> if the user agent determines that it is click focusable. User agents should consider focusable areas with non-null <a id="data-model&#58;tabindex-value-2"></a>[tabindex values](#tabindex-value) to be click focusable.

<strong>Note:</strong>

Elements which are not <a id="data-model&#58;focusable"></a>[focusable](#focusable) are not <a id="data-model&#58;focusable-area-22"></a>[focusable areas](#focusable-area), and thus not <a id="data-model&#58;sequentially-focusable-3"></a>[sequentially focusable](#sequentially-focusable) and not <a id="data-model&#58;click-focusable"></a>[click focusable](#click-focusable).

<strong>Note:</strong>

Being <a id="data-model&#58;focusable-2"></a>[focusable](#focusable) is a statement about whether an element can be focused programmatically, e.g. via the <a id="data-model&#58;dom-focus"></a><code>[focus()](#dom-focus)</code> method or <a id="data-model&#58;attr-fe-autofocus"></a><code>[autofocus](#attr-fe-autofocus)</code> attribute. In contrast, <a id="data-model&#58;sequentially-focusable-4"></a>[sequentially focusable](#sequentially-focusable) and <a id="data-model&#58;click-focusable-2"></a>[click focusable](#click-focusable) govern how the user agent responds to user interaction: respectively, to <a id="data-model&#58;sequential-focus-navigation"></a>[sequential focus navigation](#sequential-focus-navigation) and as <a id="data-model&#58;activation-behaviour"></a>[activation behavior](https://dom.spec.whatwg.org/#eventtarget-activation-behavior).

The user agent might determine that an element is not <a id="data-model&#58;sequentially-focusable-5"></a>[sequentially focusable](#sequentially-focusable) even if it is <a id="data-model&#58;focusable-3"></a>[focusable](#focusable) and is included in its <a id="data-model&#58;document-8"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code>'s <a id="data-model&#58;sequential-focus-navigation-order-2"></a>[sequential focus navigation order](#sequential-focus-navigation-order), according to user preferences. For example, macOS users can set the user agent to skip non-form control elements, or can skip links when doing <a id="data-model&#58;sequential-focus-navigation-2"></a>[sequential focus navigation](#sequential-focus-navigation) with just the Tab key (as opposed to using both the Option and Tab keys).

Similarly, the user agent might determine that an element is not <a id="data-model&#58;click-focusable-3"></a>[click focusable](#click-focusable) even if it is <a id="data-model&#58;focusable-4"></a>[focusable](#focusable). For example, in some user agents, clicking on a non-editable form control does not focus it, i.e. the user agent has determined that such controls are not click focusable.

Thus, an element can be <a id="data-model&#58;focusable-5"></a>[focusable](#focusable), but neither <a id="data-model&#58;sequentially-focusable-6"></a>[sequentially focusable](#sequentially-focusable) nor <a id="data-model&#58;click-focusable-4"></a>[click focusable](#click-focusable). For example, in some user agents, a non-editable form-control with a negative-integer <a id="data-model&#58;tabindex-value-3"></a>[tabindex value](#tabindex-value) would not be focusable via user interaction, only via programmatic APIs.

When a user [activates](https://html.spec.whatwg.org/multipage/interaction.html#activation) a <a id="data-model&#58;click-focusable-5"></a>[click focusable](#click-focusable) <a id="data-model&#58;focusable-area-23"></a>[focusable area](#focusable-area), the user agent must run the <a id="data-model&#58;focusing-steps"></a>[focusing steps](#focusing-steps) on the <a id="data-model&#58;focusable-area-24"></a>[focusable area](#focusable-area) with <em>focus trigger</em> set to "<code>click</code>".

<strong>Note:</strong>

Note that focusing is not an <a id="data-model&#58;activation-behaviour-2"></a>[activation behavior](https://dom.spec.whatwg.org/#eventtarget-activation-behavior), i.e. calling the <a id="data-model&#58;dom-click"></a><code>[click()](https://html.spec.whatwg.org/multipage/interaction.html#dom-click)</code> method on an element or dispatching a synthetic <a id="data-model&#58;event-click"></a><code>[click](https://w3c.github.io/pointerevents/#click)</code> event on it won't cause the element to get focused.

------------------------------------------------------------------------

A node is a <a id="focus-navigation-scope-owner"></a><strong>focus navigation scope owner</strong> if it is a <a id="data-model&#58;document-9"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code>, a <a id="data-model&#58;shadow-host-2"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host), a <a id="data-model&#58;the-slot-element"></a>[slot](https://html.spec.whatwg.org/multipage/scripting.html#the-slot-element), or an element which is the <a id="data-model&#58;popover-trigger"></a>[popover trigger](https://html.spec.whatwg.org/multipage/popover.html#popover-trigger) of an element in the <a id="data-model&#58;popover-showing-state"></a>[popover showing state](https://html.spec.whatwg.org/multipage/popover.html#popover-showing-state).

Each <a id="data-model&#58;focus-navigation-scope-owner"></a>[focus navigation scope owner](#focus-navigation-scope-owner) has a <a id="focus-navigation-scope"></a><strong>focus navigation scope</strong>, which is a list of elements. Its contents are determined as follows:

Every element <var>element</var> has an <a id="associated-focus-navigation-owner"></a><strong>associated focus navigation owner</strong>, which is either null or a <a id="data-model&#58;focus-navigation-scope-owner-2"></a>[focus navigation scope owner](#focus-navigation-scope-owner). It is determined by the following algorithm:

1.  If <var>element</var>'s parent is null, then return null.

2.  If <var>element</var>'s parent is a <a id="data-model&#58;shadow-host-3"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host), then return <var>element</var>'s <a id="data-model&#58;assigned-slot"></a>[assigned slot](https://dom.spec.whatwg.org/#slotable-assigned-slot).

3.  If <var>element</var>'s parent is a <a id="data-model&#58;shadow-root"></a>[shadow root](https://dom.spec.whatwg.org/#concept-shadow-root), then return the parent's <a id="data-model&#58;concept-documentfragment-host"></a>[host](https://dom.spec.whatwg.org/#concept-documentfragment-host).

4.  If <var>element</var>'s parent is the <a id="data-model&#58;document-element"></a>[document element](https://dom.spec.whatwg.org/#document-element), then return the parent's <a id="data-model&#58;node-document-2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document).

5.  If <var>element</var> is in the <a id="data-model&#58;popover-showing-state-2"></a>[popover showing state](https://html.spec.whatwg.org/multipage/popover.html#popover-showing-state) and has a <a id="data-model&#58;popover-trigger-2"></a>[popover trigger](https://html.spec.whatwg.org/multipage/popover.html#popover-trigger) set, then return <var>element</var>'s <a id="data-model&#58;popover-trigger-3"></a>[popover trigger](https://html.spec.whatwg.org/multipage/popover.html#popover-trigger).

6.  Return <var>element</var>'s parent's <a id="data-model&#58;associated-focus-navigation-owner"></a>[associated focus navigation owner](#associated-focus-navigation-owner).

Then, the contents of a given <a id="data-model&#58;focus-navigation-scope-owner-3"></a>[focus navigation scope owner](#focus-navigation-scope-owner) <var>owner</var>'s <a id="data-model&#58;focus-navigation-scope"></a>[focus navigation scope](#focus-navigation-scope) are all elements whose <a id="data-model&#58;associated-focus-navigation-owner-2"></a>[associated focus navigation owner](#associated-focus-navigation-owner) is <var>owner</var>.

<strong>Note:</strong>

The order of elements within a <a id="data-model&#58;focus-navigation-scope-2"></a>[focus navigation scope](#focus-navigation-scope) does not impact any of the algorithms in this specification. Ordering only becomes important for the <a id="data-model&#58;tabindex-ordered-focus-navigation-scope"></a>[tabindex-ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope) and <a id="data-model&#58;flattened-tabindex-ordered-focus-navigation-scope"></a>[flattened tabindex-ordered focus navigation scope](#flattened-tabindex-ordered-focus-navigation-scope) concepts defined below.

A <a id="tabindex-ordered-focus-navigation-scope"></a><strong>tabindex-ordered focus navigation scope</strong> is a list of <a id="data-model&#58;focusable-area-25"></a>[focusable areas](#focusable-area) and <a id="data-model&#58;focus-navigation-scope-owner-4"></a>[focus navigation scope owners](#focus-navigation-scope-owner). Every <a id="data-model&#58;focus-navigation-scope-owner-5"></a>[focus navigation scope owner](#focus-navigation-scope-owner) <var>owner</var> has <a id="data-model&#58;tabindex-ordered-focus-navigation-scope-2"></a>[tabindex-ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope), whose contents are determined as follows:

- It contains all elements in <var>owner</var>'s <a id="data-model&#58;focus-navigation-scope-3"></a>[focus navigation scope](#focus-navigation-scope) that are themselves <a id="data-model&#58;focus-navigation-scope-owner-6"></a>[focus navigation scope owners](#focus-navigation-scope-owner), except the elements whose <a id="data-model&#58;tabindex-value-4"></a>[tabindex value](#tabindex-value) is a negative integer.

- It contains all of the <a id="data-model&#58;focusable-area-26"></a>[focusable areas](#focusable-area) whose <a id="data-model&#58;dom-anchor-12"></a>[DOM anchor](#dom-anchor) is an element in <var>owner</var>'s <a id="data-model&#58;focus-navigation-scope-4"></a>[focus navigation scope](#focus-navigation-scope), except the <a id="data-model&#58;focusable-area-27"></a>[focusable areas](#focusable-area) whose <a id="data-model&#58;tabindex-value-5"></a>[tabindex value](#tabindex-value) is a negative integer.

The order within a <a id="data-model&#58;tabindex-ordered-focus-navigation-scope-3"></a>[tabindex-ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope) is determined by each element's <a id="data-model&#58;tabindex-value-6"></a>[tabindex value](#tabindex-value), as described in the section below.

<strong>Note:</strong>

The rules there do not give a precise ordering, as they are composed mostly of "should" statements and relative orderings.

A <a id="flattened-tabindex-ordered-focus-navigation-scope"></a><strong>flattened tabindex-ordered focus navigation scope</strong> is a list of <a id="data-model&#58;focusable-area-28"></a>[focusable areas](#focusable-area). Every <a id="data-model&#58;focus-navigation-scope-owner-7"></a>[focus navigation scope owner](#focus-navigation-scope-owner) <var>owner</var> owns a distinct <a id="data-model&#58;flattened-tabindex-ordered-focus-navigation-scope-2"></a>[flattened tabindex-ordered focus navigation scope](#flattened-tabindex-ordered-focus-navigation-scope), whose contents are determined by the following algorithm:

1.  Let <var>result</var> be a <a id="data-model&#58;list-clone"></a>[clone](https://infra.spec.whatwg.org/#list-clone) of <var>owner</var>'s <a id="data-model&#58;tabindex-ordered-focus-navigation-scope-4"></a>[tabindex-ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope).

2.  For each <var>item</var> of <var>result</var>:

    1.  If <var>item</var> is not a <a id="data-model&#58;focus-navigation-scope-owner-8"></a>[focus navigation scope owner](#focus-navigation-scope-owner), then <a id="data-model&#58;continue"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    2.  If <var>item</var> is not a <a id="data-model&#58;focusable-area-29"></a>[focusable area](#focusable-area), then replace <var>item</var> with all of the items in <var>item</var>'s <a id="data-model&#58;flattened-tabindex-ordered-focus-navigation-scope-3"></a>[flattened tabindex-ordered focus navigation scope](#flattened-tabindex-ordered-focus-navigation-scope).

    3.  Otherwise, insert the contents of <var>item</var>'s <a id="data-model&#58;flattened-tabindex-ordered-focus-navigation-scope-4"></a>[flattened tabindex-ordered focus navigation scope](#flattened-tabindex-ordered-focus-navigation-scope) after <var>item</var>.

#### <a id="the-tabindex-attribute"></a>6.6.3 The <a id="the-tabindex-attribute&#58;attr-tabindex"></a><code>[tabindex](#attr-tabindex)</code> attribute[](#the-tabindex-attribute)

<strong>✔</strong>MDN

[Global_attributes/tabindex](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/tabindex)

Support in all current engines.

Firefox1.5+Safari4+Chrome1+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet ExplorerYes

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android?Samsung Internet?Opera Android?

The <a id="attr-tabindex"></a><strong><code>tabindex</code></strong> content attribute allows authors to make an element and regions that have the element as its <a id="the-tabindex-attribute&#58;dom-anchor"></a>[DOM anchor](#dom-anchor) be <a id="the-tabindex-attribute&#58;focusable-area"></a>[focusable areas](#focusable-area), allow or prevent them from being <a id="the-tabindex-attribute&#58;sequentially-focusable"></a>[sequentially focusable](#sequentially-focusable), and determine their relative ordering for <a id="the-tabindex-attribute&#58;sequential-focus-navigation"></a>[sequential focus navigation](#sequential-focus-navigation).

The name "tab index" comes from the common use of the Tab key to navigate through the focusable elements. The term "tabbing" refers to moving forward through <a id="the-tabindex-attribute&#58;sequentially-focusable-2"></a>[sequentially focusable](#sequentially-focusable) <a id="the-tabindex-attribute&#58;focusable-area-2"></a>[focusable areas](#focusable-area).

The <a id="the-tabindex-attribute&#58;attr-tabindex-2"></a><code>[tabindex](#attr-tabindex)</code> attribute, if specified, must have a value that is a <a id="the-tabindex-attribute&#58;valid-integer"></a>[valid integer](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#valid-integer). Positive numbers specify the relative position of the element's <a id="the-tabindex-attribute&#58;focusable-area-3"></a>[focusable areas](#focusable-area) in the <a id="the-tabindex-attribute&#58;sequential-focus-navigation-order"></a>[sequential focus navigation order](#sequential-focus-navigation-order), and negative numbers indicate that the control is not <a id="the-tabindex-attribute&#58;sequentially-focusable-3"></a>[sequentially focusable](#sequentially-focusable).

Developers should use caution when using values other than 0 or −1 for their <a id="the-tabindex-attribute&#58;attr-tabindex-3"></a><code>[tabindex](#attr-tabindex)</code> attributes as this is complicated to do correctly.

<strong>Note:</strong>

The following provides a non-normative summary of the behaviors of the possible <a id="the-tabindex-attribute&#58;attr-tabindex-4"></a><code>[tabindex](#attr-tabindex)</code> attribute values. The below processing model gives the more precise rules.

<strong>omitted (or non-integer values)</strong>

The user agent will decide whether the element is <a id="the-tabindex-attribute&#58;focusable"></a>[focusable](#focusable), and if it is, whether it is <a id="the-tabindex-attribute&#58;sequentially-focusable-4"></a>[sequentially focusable](#sequentially-focusable) or <a id="the-tabindex-attribute&#58;click-focusable"></a>[click focusable](#click-focusable) (or both).

<strong>−1 (or other negative integer values)</strong>

Causes the element to be <a id="the-tabindex-attribute&#58;focusable-2"></a>[focusable](#focusable), and indicates that the author would prefer the element to be <a id="the-tabindex-attribute&#58;click-focusable-2"></a>[click focusable](#click-focusable) but not <a id="the-tabindex-attribute&#58;sequentially-focusable-5"></a>[sequentially focusable](#sequentially-focusable). The user agent might ignore this preference for click and sequential focusability, e.g., for specific element types according to platform conventions, or for keyboard-only users.

<strong>0</strong>

Causes the element to be <a id="the-tabindex-attribute&#58;focusable-3"></a>[focusable](#focusable), and indicates that the author would prefer the element to be both <a id="the-tabindex-attribute&#58;click-focusable-3"></a>[click focusable](#click-focusable) and <a id="the-tabindex-attribute&#58;sequentially-focusable-6"></a>[sequentially focusable](#sequentially-focusable). The user agent might ignore this preference for click and sequential focusability.

<strong>positive integer values</strong>

Behaves the same as 0, but in addition creates a relative ordering within a <a id="the-tabindex-attribute&#58;tabindex-ordered-focus-navigation-scope"></a>[tabindex-ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope), so that elements with higher <a id="the-tabindex-attribute&#58;attr-tabindex-5"></a><code>[tabindex](#attr-tabindex)</code> attribute value come later.

Note that the <a id="the-tabindex-attribute&#58;attr-tabindex-6"></a><code>[tabindex](#attr-tabindex)</code> attribute cannot be used to make an element non-focusable. The only way a page author can do that is by <a id="the-tabindex-attribute&#58;concept-element-disabled"></a>[disabling](https://html.spec.whatwg.org/multipage/semantics-other.html#concept-element-disabled) the element, or making it <a id="the-tabindex-attribute&#58;inert"></a>[inert](#inert).

------------------------------------------------------------------------

The <a id="tabindex-value"></a><strong>tabindex value</strong> of an element is the value of its <a id="the-tabindex-attribute&#58;attr-tabindex-7"></a><code>[tabindex](#attr-tabindex)</code> attribute, parsed using the <a id="the-tabindex-attribute&#58;rules-for-parsing-integers"></a>[rules for parsing integers](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#rules-for-parsing-integers). If parsing fails or the attribute is not specified, then the <a id="the-tabindex-attribute&#58;tabindex-value"></a>[tabindex value](#tabindex-value) is null.

The <a id="the-tabindex-attribute&#58;tabindex-value-2"></a>[tabindex value](#tabindex-value) of a <a id="the-tabindex-attribute&#58;focusable-area-4"></a>[focusable area](#focusable-area) is the <a id="the-tabindex-attribute&#58;tabindex-value-3"></a>[tabindex value](#tabindex-value) of its <a id="the-tabindex-attribute&#58;dom-anchor-2"></a>[DOM anchor](#dom-anchor).

The <a id="the-tabindex-attribute&#58;tabindex-value-4"></a>[tabindex value](#tabindex-value) of an element must be interpreted as follows:

<strong>If the value is null</strong>

The user agent should follow platform conventions to determine if the element should be considered as a <a id="the-tabindex-attribute&#58;focusable-area-5"></a>[focusable area](#focusable-area) and if so, whether the element and any <a id="the-tabindex-attribute&#58;focusable-area-6"></a>[focusable areas](#focusable-area) that have the element as their <a id="the-tabindex-attribute&#58;dom-anchor-3"></a>[DOM anchor](#dom-anchor) are <a id="the-tabindex-attribute&#58;sequentially-focusable-7"></a>[sequentially focusable](#sequentially-focusable), and if so, what their relative position in their <a id="the-tabindex-attribute&#58;tabindex-ordered-focus-navigation-scope-2"></a>[tabindex-ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope) is to be. If the element is a <a id="the-tabindex-attribute&#58;focus-navigation-scope-owner"></a>[focus navigation scope owner](#focus-navigation-scope-owner), it must be included in its <a id="the-tabindex-attribute&#58;tabindex-ordered-focus-navigation-scope-3"></a>[tabindex-ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope) even if it is not a <a id="the-tabindex-attribute&#58;focusable-area-7"></a>[focusable area](#focusable-area).

The relative ordering within a <a id="the-tabindex-attribute&#58;tabindex-ordered-focus-navigation-scope-4"></a>[tabindex-ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope) for elements and <a id="the-tabindex-attribute&#58;focusable-area-8"></a>[focusable areas](#focusable-area) that belong to the same <a id="the-tabindex-attribute&#58;focus-navigation-scope"></a>[focus navigation scope](#focus-navigation-scope) and whose <a id="the-tabindex-attribute&#58;tabindex-value-5"></a>[tabindex value](#tabindex-value) is null should be in <a id="the-tabindex-attribute&#58;shadow-including-tree-order"></a>[shadow-including tree order](https://dom.spec.whatwg.org/#concept-shadow-including-tree-order).

Modulo platform conventions, it is suggested that the following elements should be considered as <a id="the-tabindex-attribute&#58;focusable-area-9"></a>[focusable areas](#focusable-area) and be <a id="the-tabindex-attribute&#58;sequentially-focusable-8"></a>[sequentially focusable](#sequentially-focusable):

- <a id="the-tabindex-attribute&#58;the-a-element"></a><code>[a](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element)</code> elements that have an <a id="the-tabindex-attribute&#58;attr-hyperlink-href"></a><code>[href](https://html.spec.whatwg.org/multipage/links.html#attr-hyperlink-href)</code> attribute
- <a id="the-tabindex-attribute&#58;the-button-element"></a><code>[button](https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element)</code> elements
- <a id="the-tabindex-attribute&#58;the-input-element"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code> elements whose <a id="the-tabindex-attribute&#58;attr-input-type"></a><code>[type](https://html.spec.whatwg.org/multipage/input.html#attr-input-type)</code> attribute are not in the <a id="the-tabindex-attribute&#58;hidden-state-(type=hidden)"></a>[Hidden](https://html.spec.whatwg.org/multipage/input.html#hidden-state-(type=hidden)) state
- <a id="the-tabindex-attribute&#58;the-select-element"></a><code>[select](https://html.spec.whatwg.org/multipage/form-elements.html#the-select-element)</code> elements
- <a id="the-tabindex-attribute&#58;the-textarea-element"></a><code>[textarea](https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element)</code> elements
- <a id="the-tabindex-attribute&#58;the-summary-element"></a><code>[summary](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-summary-element)</code> elements that are the first <a id="the-tabindex-attribute&#58;the-summary-element-2"></a><code>[summary](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-summary-element)</code> element child of a <a id="the-tabindex-attribute&#58;the-details-element"></a><code>[details](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-details-element)</code> element
- Elements with a <a id="the-tabindex-attribute&#58;attr-draggable"></a><code>[draggable](https://html.spec.whatwg.org/multipage/dnd.html#attr-draggable)</code> attribute set, if that would enable the user agent to allow the user to begin drag operations for those elements without the use of a pointing device
- <a id="the-tabindex-attribute&#58;editing-host"></a>[Editing hosts](https://html.spec.whatwg.org/multipage/interaction.html#editing-host)
- <a id="the-tabindex-attribute&#58;navigable-container"></a>[Navigable containers](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable-container)

<a id="negative-tabindex"></a><strong>If the value is a negative integer</strong>

The user agent must consider the element as a <a id="the-tabindex-attribute&#58;focusable-area-10"></a>[focusable area](#focusable-area), but should omit the element from any <a id="the-tabindex-attribute&#58;tabindex-ordered-focus-navigation-scope-5"></a>[tabindex-ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope).

<strong>Note:</strong>

One valid reason to ignore the requirement that sequential focus navigation not allow the author to lead to the element would be if the user's only mechanism for moving the focus is sequential focus navigation. For instance, a keyboard-only user would be unable to click on a text control with a negative <a id="the-tabindex-attribute&#58;attr-tabindex-8"></a><code>[tabindex](#attr-tabindex)</code>, so that user's user agent would be well justified in allowing the user to tab to the control regardless.

<strong>If the value is a zero</strong>

The user agent must allow the element to be considered as a <a id="the-tabindex-attribute&#58;focusable-area-11"></a>[focusable area](#focusable-area) and should allow the element and any <a id="the-tabindex-attribute&#58;focusable-area-12"></a>[focusable areas](#focusable-area) that have the element as their <a id="the-tabindex-attribute&#58;dom-anchor-4"></a>[DOM anchor](#dom-anchor) to be <a id="the-tabindex-attribute&#58;sequentially-focusable-9"></a>[sequentially focusable](#sequentially-focusable).

The relative ordering within a <a id="the-tabindex-attribute&#58;tabindex-ordered-focus-navigation-scope-6"></a>[tabindex-ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope) for elements and <a id="the-tabindex-attribute&#58;focusable-area-13"></a>[focusable areas](#focusable-area) that belong to the same <a id="the-tabindex-attribute&#58;focus-navigation-scope-2"></a>[focus navigation scope](#focus-navigation-scope) and whose <a id="the-tabindex-attribute&#58;tabindex-value-6"></a>[tabindex value](#tabindex-value) is zero should be in <a id="the-tabindex-attribute&#58;shadow-including-tree-order-2"></a>[shadow-including tree order](https://dom.spec.whatwg.org/#concept-shadow-including-tree-order).

<strong>If the value is greater than zero</strong>

The user agent must allow the element to be considered as a <a id="the-tabindex-attribute&#58;focusable-area-14"></a>[focusable area](#focusable-area) and should allow the element and any <a id="the-tabindex-attribute&#58;focusable-area-15"></a>[focusable areas](#focusable-area) that have the element as their <a id="the-tabindex-attribute&#58;dom-anchor-5"></a>[DOM anchor](#dom-anchor) to be <a id="the-tabindex-attribute&#58;sequentially-focusable-10"></a>[sequentially focusable](#sequentially-focusable), and should place the element — referenced as <var>candidate</var> below — and the aforementioned <a id="the-tabindex-attribute&#58;focusable-area-16"></a>[focusable areas](#focusable-area) in the <a id="the-tabindex-attribute&#58;tabindex-ordered-focus-navigation-scope-7"></a>[tabindex-ordered focus navigation scope](#tabindex-ordered-focus-navigation-scope) where the element is a part of so that, relative to other elements and <a id="the-tabindex-attribute&#58;focusable-area-17"></a>[focusable areas](#focusable-area) that belong to the same <a id="the-tabindex-attribute&#58;focus-navigation-scope-3"></a>[focus navigation scope](#focus-navigation-scope), they are:

- before any <a id="the-tabindex-attribute&#58;focusable-area-18"></a>[focusable area](#focusable-area) whose <a id="the-tabindex-attribute&#58;dom-anchor-6"></a>[DOM anchor](#dom-anchor) is an element whose <a id="the-tabindex-attribute&#58;attr-tabindex-9"></a><code>[tabindex](#attr-tabindex)</code> attribute has been omitted or whose value, when parsed, returns an error,
- before any <a id="the-tabindex-attribute&#58;focusable-area-19"></a>[focusable area](#focusable-area) whose <a id="the-tabindex-attribute&#58;dom-anchor-7"></a>[DOM anchor](#dom-anchor) is an element whose <a id="the-tabindex-attribute&#58;attr-tabindex-10"></a><code>[tabindex](#attr-tabindex)</code> attribute has a value less than or equal to zero,
- after any <a id="the-tabindex-attribute&#58;focusable-area-20"></a>[focusable area](#focusable-area) whose <a id="the-tabindex-attribute&#58;dom-anchor-8"></a>[DOM anchor](#dom-anchor) is an element whose <a id="the-tabindex-attribute&#58;attr-tabindex-11"></a><code>[tabindex](#attr-tabindex)</code> attribute has a value greater than zero but less than the value of the <a id="the-tabindex-attribute&#58;attr-tabindex-12"></a><code>[tabindex](#attr-tabindex)</code> attribute on <var>candidate</var>,
- after any <a id="the-tabindex-attribute&#58;focusable-area-21"></a>[focusable area](#focusable-area) whose <a id="the-tabindex-attribute&#58;dom-anchor-9"></a>[DOM anchor](#dom-anchor) is an element whose <a id="the-tabindex-attribute&#58;attr-tabindex-13"></a><code>[tabindex](#attr-tabindex)</code> attribute has a value equal to the value of the <a id="the-tabindex-attribute&#58;attr-tabindex-14"></a><code>[tabindex](#attr-tabindex)</code> attribute on <var>candidate</var> but that is located earlier than <var>candidate</var> in <a id="the-tabindex-attribute&#58;shadow-including-tree-order-3"></a>[shadow-including tree order](https://dom.spec.whatwg.org/#concept-shadow-including-tree-order),
- before any <a id="the-tabindex-attribute&#58;focusable-area-22"></a>[focusable area](#focusable-area) whose <a id="the-tabindex-attribute&#58;dom-anchor-10"></a>[DOM anchor](#dom-anchor) is an element whose <a id="the-tabindex-attribute&#58;attr-tabindex-15"></a><code>[tabindex](#attr-tabindex)</code> attribute has a value equal to the value of the <a id="the-tabindex-attribute&#58;attr-tabindex-16"></a><code>[tabindex](#attr-tabindex)</code> attribute on <var>candidate</var> but that is located later than <var>candidate</var> in <a id="the-tabindex-attribute&#58;shadow-including-tree-order-4"></a>[shadow-including tree order](https://dom.spec.whatwg.org/#concept-shadow-including-tree-order), and
- before any <a id="the-tabindex-attribute&#58;focusable-area-23"></a>[focusable area](#focusable-area) whose <a id="the-tabindex-attribute&#58;dom-anchor-11"></a>[DOM anchor](#dom-anchor) is an element whose <a id="the-tabindex-attribute&#58;attr-tabindex-17"></a><code>[tabindex](#attr-tabindex)</code> attribute has a value greater than the value of the <a id="the-tabindex-attribute&#58;attr-tabindex-18"></a><code>[tabindex](#attr-tabindex)</code> attribute on <var>candidate</var>.

<strong>✔</strong>MDN

[HTMLElement/tabIndex](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/tabIndex)

Support in all current engines.

Firefox1+Safari3.1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)18Internet Explorer🔰 5.5+

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android?Samsung Internet?Opera Android12.1+

The <a id="dom-tabindex"></a><strong><code>tabIndex</code></strong> getter steps are:

1.  Let <var>attribute</var> be <a id="the-tabindex-attribute&#58;this"></a>[this](https://webidl.spec.whatwg.org/#this)'s <a id="the-tabindex-attribute&#58;attr-tabindex-19"></a><code>[tabindex](#attr-tabindex)</code> attribute.

2.  If <var>attribute</var> is not null:

    1.  Let <var>parsedValue</var> be the result of <a id="the-tabindex-attribute&#58;rules-for-parsing-integers-2"></a>[integer parsing](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#rules-for-parsing-integers) <var>attribute</var>'s <a id="the-tabindex-attribute&#58;concept-attribute-value"></a>[value](https://dom.spec.whatwg.org/#concept-attribute-value).

    2.  If <var>parsedValue</var> is not an error and is within the <a id="the-tabindex-attribute&#58;idl-long"></a><code>[long](https://webidl.spec.whatwg.org/#idl-long)</code> range, then return <var>parsedValue</var>.

3.  Return 0 if <a id="the-tabindex-attribute&#58;this-2"></a>[this](https://webidl.spec.whatwg.org/#this) is an <a id="the-tabindex-attribute&#58;the-a-element-2"></a><code>[a](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element)</code>, <a id="the-tabindex-attribute&#58;the-area-element"></a><code>[area](https://html.spec.whatwg.org/multipage/image-maps.html#the-area-element)</code>, <a id="the-tabindex-attribute&#58;the-button-element-2"></a><code>[button](https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element)</code>, <a id="the-tabindex-attribute&#58;frame"></a><code>[frame](https://html.spec.whatwg.org/multipage/obsolete.html#frame)</code>, <a id="the-tabindex-attribute&#58;the-iframe-element"></a><code>[iframe](https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element)</code>, <a id="the-tabindex-attribute&#58;the-input-element-2"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code>, <a id="the-tabindex-attribute&#58;the-object-element"></a><code>[object](https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-object-element)</code>, <a id="the-tabindex-attribute&#58;the-select-element-2"></a><code>[select](https://html.spec.whatwg.org/multipage/form-elements.html#the-select-element)</code>, <a id="the-tabindex-attribute&#58;the-textarea-element-2"></a><code>[textarea](https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element)</code>, or <a id="the-tabindex-attribute&#58;svg-a"></a>[SVG <code>a</code>](https://w3c.github.io/svgwg/svg2-draft/linking.html#elementdef-a) element, or <a id="the-tabindex-attribute&#58;mathml-a"></a>[MathML <code>a</code>](https://w3c.github.io/mathml-core/#dfn-a) element, or is a <a id="the-tabindex-attribute&#58;the-summary-element-3"></a><code>[summary](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-summary-element)</code> element that is a <a id="the-tabindex-attribute&#58;summary-for-its-parent-details"></a>[summary for its parent details](https://html.spec.whatwg.org/multipage/interactive-elements.html#summary-for-its-parent-details); otherwise −1.

<strong>Note:</strong>

The varying default value based on element type is a historical artifact.

#### <a id="focus-processing-model"></a>6.6.4 <a id="processing-model-5"></a>Processing model[](#focus-processing-model)

To <a id="get-the-focusable-area"></a><strong>get the focusable area</strong> for a <var>focus target</var> that is either an element that is not a <a id="focus-processing-model&#58;focusable-area"></a>[focusable area](#focusable-area), or is a <a id="focus-processing-model&#58;navigable"></a>[navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable), given an optional string <var>focus trigger</var> (default "<code>other</code>"), run the first matching set of steps from the following list:

<strong>If <var>focus target</var> is an <a id="focus-processing-model&#58;the-area-element"></a><code>[area](https://html.spec.whatwg.org/multipage/image-maps.html#the-area-element)</code> element with one or more shapes that are <a id="focus-processing-model&#58;focusable-area-2"></a>[focusable areas](#focusable-area)</strong>

Return the shape corresponding to the first <a id="focus-processing-model&#58;the-img-element"></a><code>[img](https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element)</code> element in <a id="focus-processing-model&#58;tree-order"></a>[tree order](https://dom.spec.whatwg.org/#concept-tree-order) that uses the image map to which the <a id="focus-processing-model&#58;the-area-element-2"></a><code>[area](https://html.spec.whatwg.org/multipage/image-maps.html#the-area-element)</code> element belongs.

<strong>If <var>focus target</var> is an element with one or more scrollable regions that are <a id="focus-processing-model&#58;focusable-area-3"></a>[focusable areas](#focusable-area)</strong>

Return the element's first scrollable region, according to a pre-order, depth-first traversal of the <a id="focus-processing-model&#58;flat-tree"></a>[flat tree](https://drafts.csswg.org/css-scoping/#flat-tree). [\[CSSSCOPING\]](https://html.spec.whatwg.org/multipage/references.html#refsCSSSCOPING)

<strong>If <var>focus target</var> is the <a id="focus-processing-model&#58;document-element"></a>[document element](https://dom.spec.whatwg.org/#document-element) of its <a id="focus-processing-model&#58;document"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code></strong>

Return the <a id="focus-processing-model&#58;document-2"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code>'s <a id="focus-processing-model&#58;viewport"></a>[viewport](https://drafts.csswg.org/css2/#viewport).

<strong>If <var>focus target</var> is a <a id="focus-processing-model&#58;navigable-2"></a>[navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable)</strong>

Return the <a id="focus-processing-model&#58;navigable-3"></a>[navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable)'s <a id="focus-processing-model&#58;nav-document"></a>[active document](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-document).

<strong>If <var>focus target</var> is a <a id="focus-processing-model&#58;navigable-container"></a>[navigable container](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable-container) with a non-null <a id="focus-processing-model&#58;content-navigable"></a>[content navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#content-navigable)</strong>

Return the <a id="focus-processing-model&#58;navigable-container-2"></a>[navigable container](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable-container)'s <a id="focus-processing-model&#58;content-navigable-2"></a>[content navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#content-navigable)'s <a id="focus-processing-model&#58;nav-document-2"></a>[active document](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-document).

<strong>If <var>focus target</var> is a <a id="focus-processing-model&#58;shadow-host"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host) whose <a id="focus-processing-model&#58;concept-element-shadow-root"></a>[shadow root](https://dom.spec.whatwg.org/#concept-element-shadow-root)'s <a id="focus-processing-model&#58;delegates-focus"></a>[delegates focus](https://dom.spec.whatwg.org/#shadowroot-delegates-focus) is true</strong>

1.  Let <var>focusedElement</var> be the <a id="focus-processing-model&#58;currently-focused-area-of-a-top-level-traversable"></a>[currently focused area of a top-level traversable](#currently-focused-area-of-a-top-level-traversable)'s <a id="focus-processing-model&#58;dom-anchor"></a>[DOM anchor](#dom-anchor).

2.  If <var>focus target</var> is a <a id="focus-processing-model&#58;shadow-including-inclusive-ancestor"></a>[shadow-including inclusive ancestor](https://dom.spec.whatwg.org/#concept-shadow-including-inclusive-ancestor) of <var>focusedElement</var>, then return <var>focusedElement</var>.

3.  Return the <a id="focus-processing-model&#58;focus-delegate"></a>[focus delegate](#focus-delegate) for <var>focus target</var> given <var>focus trigger</var>.

<strong>Note:</strong>

For <a id="focus-processing-model&#58;sequentially-focusable"></a>[sequential focusability](#sequentially-focusable), the handling of <a id="focus-processing-model&#58;shadow-host-2"></a>[shadow hosts](https://dom.spec.whatwg.org/#element-shadow-host) and <a id="focus-processing-model&#58;delegates-focus-2"></a>[delegates focus](https://dom.spec.whatwg.org/#shadowroot-delegates-focus) is done when constructing the <a id="focus-processing-model&#58;sequential-focus-navigation-order"></a>[sequential focus navigation order](#sequential-focus-navigation-order). That is, the <a id="focus-processing-model&#58;focusing-steps"></a>[focusing steps](#focusing-steps) will never be called on such <a id="focus-processing-model&#58;shadow-host-3"></a>[shadow hosts](https://dom.spec.whatwg.org/#element-shadow-host) as part of sequential focus navigation.

<strong>Otherwise</strong>

Return null.

The <a id="focus-delegate"></a><strong>focus delegate</strong> for a <var>focusTarget</var>, given an optional string <var>focusTrigger</var> (default "<code>other</code>"), is given by the following steps:

1.  If <var>focusTarget</var> is a <a id="focus-processing-model&#58;shadow-host-4"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host) and its <a id="focus-processing-model&#58;concept-element-shadow-root-2"></a>[shadow root](https://dom.spec.whatwg.org/#concept-element-shadow-root)'s <a id="focus-processing-model&#58;delegates-focus-3"></a>[delegates focus](https://dom.spec.whatwg.org/#shadowroot-delegates-focus) is false, then return null.

2.  Let <var>whereToLook</var> be <var>focusTarget</var>.

3.  If <var>whereToLook</var> is a <a id="focus-processing-model&#58;shadow-host-5"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host), then set <var>whereToLook</var> to <var>whereToLook</var>'s <a id="focus-processing-model&#58;concept-element-shadow-root-3"></a>[shadow root](https://dom.spec.whatwg.org/#concept-element-shadow-root).

4.  Let <var>autofocusDelegate</var> be the <a id="focus-processing-model&#58;autofocus-delegate"></a>[autofocus delegate](#autofocus-delegate) for <var>whereToLook</var> given <var>focusTrigger</var>.

5.  If <var>autofocusDelegate</var> is not null, then return <var>autofocusDelegate</var>.

6.  <a id="focus-processing-model&#58;list-iterate"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>descendant</var> of <var>whereToLook</var>'s <a id="focus-processing-model&#58;descendant"></a>[descendants](https://dom.spec.whatwg.org/#concept-tree-descendant), in <a id="focus-processing-model&#58;tree-order-2"></a>[tree order](https://dom.spec.whatwg.org/#concept-tree-order):

    1.  Let <var>focusableArea</var> be null.

    2.  If <var>focusTarget</var> is a <a id="focus-processing-model&#58;the-dialog-element"></a><code>[dialog](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element)</code> element and <var>descendant</var> is <a id="focus-processing-model&#58;sequentially-focusable-2"></a>[sequentially focusable](#sequentially-focusable), then set <var>focusableArea</var> to <var>descendant</var>.

    3.  Otherwise, if <var>focusTarget</var> is not a <a id="focus-processing-model&#58;the-dialog-element-2"></a><code>[dialog](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element)</code> and <var>descendant</var> is a <a id="focus-processing-model&#58;focusable-area-4"></a>[focusable area](#focusable-area), set <var>focusableArea</var> to <var>descendant</var>.

    4.  Otherwise, set <var>focusableArea</var> to the result of <a id="focus-processing-model&#58;get-the-focusable-area"></a>[getting the focusable area](#get-the-focusable-area) for <var>descendant</var> given <var>focusTrigger</var>.

        <strong>Note:</strong>

        This step can end up recursing, i.e., the <a id="focus-processing-model&#58;get-the-focusable-area-2"></a>[get the focusable area](#get-the-focusable-area) steps might return the <a id="focus-processing-model&#58;focus-delegate-2"></a>[focus delegate](#focus-delegate) of <var>descendant</var>.

    5.  If <var>focusableArea</var> is not null, then return <var>focusableArea</var>.

    <strong>Note:</strong>

    It's important that we are <em>not</em> looking at the <a id="focus-processing-model&#58;shadow-including-descendant"></a>[shadow-including descendants](https://dom.spec.whatwg.org/#concept-shadow-including-descendant) here, but instead only at the <a id="focus-processing-model&#58;descendant-2"></a>[descendants](https://dom.spec.whatwg.org/#concept-tree-descendant). <a id="focus-processing-model&#58;shadow-host-6"></a>[Shadow hosts](https://dom.spec.whatwg.org/#element-shadow-host) are instead handled by the recursive case mentioned above.

7.  Return null.

<strong>Note:</strong>

The above algorithm essentially returns the first suitable <a id="focus-processing-model&#58;focusable-area-5"></a>[focusable area](#focusable-area) where the path between its <a id="focus-processing-model&#58;dom-anchor-2"></a>[DOM anchor](#dom-anchor) and <var>focusTarget</var> delegates focus at any shadow tree boundaries.

The <a id="autofocus-delegate"></a><strong>autofocus delegate</strong> for a <var>focus target</var> given a <var>focus trigger</var> is given by the following steps:

1.  For each <a id="focus-processing-model&#58;descendant-3"></a>[descendant](https://dom.spec.whatwg.org/#concept-tree-descendant) <var>descendant</var> of <var>focus target</var>, in <a id="focus-processing-model&#58;tree-order-3"></a>[tree order](https://dom.spec.whatwg.org/#concept-tree-order):

    1.  If <var>descendant</var> does not have an <a id="focus-processing-model&#58;attr-fe-autofocus"></a><code>[autofocus](#attr-fe-autofocus)</code> content attribute, then <a id="focus-processing-model&#58;continue"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    2.  Let <var>focusable area</var> be <var>descendant</var>, if <var>descendant</var> is a <a id="focus-processing-model&#58;focusable-area-6"></a>[focusable area](#focusable-area); otherwise let <var>focusable area</var> be the result of <a id="focus-processing-model&#58;get-the-focusable-area-3"></a>[getting the focusable area](#get-the-focusable-area) for <var>descendant</var> given <var>focus trigger</var>.

    3.  If <var>focusable area</var> is null, then <a id="focus-processing-model&#58;continue-2"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    4.  If <var>focusable area</var> is not <a id="focus-processing-model&#58;click-focusable"></a>[click focusable](#click-focusable) and <var>focus trigger</var> is "<code>click</code>", then <a id="focus-processing-model&#58;continue-3"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    5.  Return <var>focusable area</var>.

2.  Return null.

The <a id="focusing-steps"></a><strong>focusing steps</strong> for an object <var>new focus target</var> that is either a <a id="focus-processing-model&#58;focusable-area-7"></a>[focusable area](#focusable-area), or an element that is not a <a id="focus-processing-model&#58;focusable-area-8"></a>[focusable area](#focusable-area), or a <a id="focus-processing-model&#58;navigable-4"></a>[navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable), are as follows. They can optionally be run with a <var>fallback target</var> and a string <var>focus trigger</var>.

1.  If <var>new focus target</var> is not a <a id="focus-processing-model&#58;focusable-area-9"></a>[focusable area](#focusable-area), then set <var>new focus target</var> to the result of <a id="focus-processing-model&#58;get-the-focusable-area-4"></a>[getting the focusable area](#get-the-focusable-area) for <var>new focus target</var>, given <var>focus trigger</var> if it was passed.

2.  If <var>new focus target</var> is null:

    1.  If no <var>fallback target</var> was specified, then return.

    2.  Otherwise, set <var>new focus target</var> to the <var>fallback target</var>.

3.  If <var>new focus target</var> is a <a id="focus-processing-model&#58;navigable-container-3"></a>[navigable container](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable-container) with non-null <a id="focus-processing-model&#58;content-navigable-3"></a>[content navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#content-navigable), then set <var>new focus target</var> to the <a id="focus-processing-model&#58;content-navigable-4"></a>[content navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#content-navigable)'s <a id="focus-processing-model&#58;nav-document-3"></a>[active document](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-document).

4.  If <var>new focus target</var> is a <a id="focus-processing-model&#58;focusable-area-10"></a>[focusable area](#focusable-area) and its <a id="focus-processing-model&#58;dom-anchor-3"></a>[DOM anchor](#dom-anchor) is <a id="focus-processing-model&#58;inert"></a>[inert](#inert), then return.

5.  If <var>new focus target</var> is the <a id="focus-processing-model&#58;currently-focused-area-of-a-top-level-traversable-2"></a>[currently focused area of a top-level traversable](#currently-focused-area-of-a-top-level-traversable), then return.

6.  Let <var>old chain</var> be the <a id="focus-processing-model&#58;current-focus-chain-of-a-top-level-traversable"></a>[current focus chain of the top-level traversable](#current-focus-chain-of-a-top-level-traversable) in which <var>new focus target</var> finds itself.

7.  Let <var>new chain</var> be the <a id="focus-processing-model&#58;focus-chain"></a>[focus chain](#focus-chain) of <var>new focus target</var>.

8.  Run the <a id="focus-processing-model&#58;focus-update-steps"></a>[focus update steps](#focus-update-steps) with <var>old chain</var>, <var>new chain</var>, and <var>new focus target</var> respectively.

User agents must <a id="focus-processing-model&#58;immediately"></a>[immediately](https://html.spec.whatwg.org/multipage/infrastructure.html#immediately) run the <a id="focus-processing-model&#58;focusing-steps-2"></a>[focusing steps](#focusing-steps) for a <a id="focus-processing-model&#58;focusable-area-11"></a>[focusable area](#focusable-area) or <a id="focus-processing-model&#58;navigable-5"></a>[navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable) <var>candidate</var> whenever the user attempts to move the focus to <var>candidate</var>.

The <a id="unfocusing-steps"></a><strong>unfocusing steps</strong> for an object <var>old focus target</var> that is either a <a id="focus-processing-model&#58;focusable-area-12"></a>[focusable area](#focusable-area) or an element that is not a <a id="focus-processing-model&#58;focusable-area-13"></a>[focusable area](#focusable-area) are as follows:

1.  If <var>old focus target</var> is a <a id="focus-processing-model&#58;shadow-host-7"></a>[shadow host](https://dom.spec.whatwg.org/#element-shadow-host) whose <a id="focus-processing-model&#58;concept-element-shadow-root-4"></a>[shadow root](https://dom.spec.whatwg.org/#concept-element-shadow-root)'s <a id="focus-processing-model&#58;delegates-focus-4"></a>[delegates focus](https://dom.spec.whatwg.org/#shadowroot-delegates-focus) is true, and <var>old focus target</var>'s <a id="focus-processing-model&#58;concept-element-shadow-root-5"></a>[shadow root](https://dom.spec.whatwg.org/#concept-element-shadow-root) is a <a id="focus-processing-model&#58;shadow-including-inclusive-ancestor-2"></a>[shadow-including inclusive ancestor](https://dom.spec.whatwg.org/#concept-shadow-including-inclusive-ancestor) of the <a id="focus-processing-model&#58;currently-focused-area-of-a-top-level-traversable-3"></a>[currently focused area of a top-level traversable](#currently-focused-area-of-a-top-level-traversable)'s <a id="focus-processing-model&#58;dom-anchor-4"></a>[DOM anchor](#dom-anchor), then set <var>old focus target</var> to that <a id="focus-processing-model&#58;currently-focused-area-of-a-top-level-traversable-4"></a>[currently focused area of a top-level traversable](#currently-focused-area-of-a-top-level-traversable).

2.  If <var>old focus target</var> is <a id="focus-processing-model&#58;inert-2"></a>[inert](#inert), then return.

3.  If <var>old focus target</var> is an <a id="focus-processing-model&#58;the-area-element-3"></a><code>[area](https://html.spec.whatwg.org/multipage/image-maps.html#the-area-element)</code> element and one of its shapes is the <a id="focus-processing-model&#58;currently-focused-area-of-a-top-level-traversable-5"></a>[currently focused area of a top-level traversable](#currently-focused-area-of-a-top-level-traversable), or, if <var>old focus target</var> is an element with one or more scrollable regions, and one of them is the <a id="focus-processing-model&#58;currently-focused-area-of-a-top-level-traversable-6"></a>[currently focused area of a top-level traversable](#currently-focused-area-of-a-top-level-traversable), then let <var>old focus target</var> be that <a id="focus-processing-model&#58;currently-focused-area-of-a-top-level-traversable-7"></a>[currently focused area of a top-level traversable](#currently-focused-area-of-a-top-level-traversable).

4.  Let <var>old chain</var> be the <a id="focus-processing-model&#58;current-focus-chain-of-a-top-level-traversable-2"></a>[current focus chain of the top-level traversable](#current-focus-chain-of-a-top-level-traversable) in which <var>old focus target</var> finds itself.

5.  If <var>old focus target</var> is not one of the entries in <var>old chain</var>, then return.

6.  If <var>old focus target</var> is not a <a id="focus-processing-model&#58;focusable-area-14"></a>[focusable area](#focusable-area), then return.

7.  Let <var>topDocument</var> be <var>old chain</var>'s last entry.

8.  If <var>topDocument</var>'s <a id="focus-processing-model&#58;node-navigable"></a>[node navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#node-navigable) has <a id="focus-processing-model&#58;system-focus"></a>[system focus](#system-focus), then run the <a id="focus-processing-model&#58;focusing-steps-3"></a>[focusing steps](#focusing-steps) for <var>topDocument</var>'s <a id="focus-processing-model&#58;viewport-2"></a>[viewport](https://drafts.csswg.org/css2/#viewport).

    Otherwise, apply any relevant platform-specific conventions for removing <a id="focus-processing-model&#58;system-focus-2"></a>[system focus](#system-focus) from <var>topDocument</var>'s <a id="focus-processing-model&#58;node-navigable-2"></a>[node navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#node-navigable), and run the <a id="focus-processing-model&#58;focus-update-steps-2"></a>[focus update steps](#focus-update-steps) given <var>old chain</var>, an empty list, and null.

<strong>Note:</strong>

The <a id="focus-processing-model&#58;unfocusing-steps"></a>[unfocusing steps](#unfocusing-steps) do not always result in the focus changing, even when applied to the <a id="focus-processing-model&#58;currently-focused-area-of-a-top-level-traversable-8"></a>[currently focused area of a top-level traversable](#currently-focused-area-of-a-top-level-traversable). For example, if the <a id="focus-processing-model&#58;currently-focused-area-of-a-top-level-traversable-9"></a>[currently focused area of a top-level traversable](#currently-focused-area-of-a-top-level-traversable) is a <a id="focus-processing-model&#58;viewport-3"></a>[viewport](https://drafts.csswg.org/css2/#viewport), then it will usually keep its focus regardless until another <a id="focus-processing-model&#58;focusable-area-15"></a>[focusable area](#focusable-area) is explicitly focused with the <a id="focus-processing-model&#58;focusing-steps-4"></a>[focusing steps](#focusing-steps).

------------------------------------------------------------------------

The <a id="focus-update-steps"></a><strong>focus update steps</strong>, given an <var>old chain</var>, a <var>new chain</var>, and a <var>new focus target</var> respectively, are as follows:

1.  If the last entry in <var>old chain</var> and the last entry in <var>new chain</var> are the same, pop the last entry from <var>old chain</var> and the last entry from <var>new chain</var> and redo this step.

2.  For each entry <var>entry</var> in <var>old chain</var>, in order, run these substeps:

    1.  If <var>entry</var> is an <a id="focus-processing-model&#58;the-input-element"></a><code>[input](https://html.spec.whatwg.org/multipage/input.html#the-input-element)</code> element, and the <a id="focus-processing-model&#58;event-change"></a><code>[change](https://html.spec.whatwg.org/multipage/indices.html#event-change)</code> event <a id="focus-processing-model&#58;concept-input-apply"></a>[applies](https://html.spec.whatwg.org/multipage/input.html#concept-input-apply) to the element, and the element does not have a defined <a id="focus-processing-model&#58;activation-behaviour"></a>[activation behavior](https://dom.spec.whatwg.org/#eventtarget-activation-behavior), and the user has changed the element's <a id="focus-processing-model&#58;concept-fe-value"></a>[value](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-fe-value) or its list of <a id="focus-processing-model&#58;concept-input-type-file-selected"></a>[selected files](https://html.spec.whatwg.org/multipage/input.html#concept-input-type-file-selected) while the control was focused without committing that change (such that it is different to what it was when the control was first focused):

        1.  Set <var>entry</var>'s <a id="focus-processing-model&#58;user-validity"></a>[user validity](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#user-validity) to true.

        2.  <a id="unfocus-causes-change-event"></a>

            <a id="focus-processing-model&#58;concept-event-fire"></a>[Fire an event](https://dom.spec.whatwg.org/#concept-event-fire) named <a id="focus-processing-model&#58;event-change-2"></a><code>[change](https://html.spec.whatwg.org/multipage/indices.html#event-change)</code> at the element, with the <a id="focus-processing-model&#58;dom-event-bubbles"></a><code>[bubbles](https://dom.spec.whatwg.org/#dom-event-bubbles)</code> attribute initialized to true.

    2.  If <var>entry</var> is an element, let <var>blur event target</var> be <var>entry</var>.

        If <var>entry</var> is a <a id="focus-processing-model&#58;document-3"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> object, let <var>blur event target</var> be that <a id="focus-processing-model&#58;document-4"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> object's <a id="focus-processing-model&#58;concept-relevant-global"></a>[relevant global object](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global).

        Otherwise, let <var>blur event target</var> be null.

    3.  If <var>entry</var> is the last entry in <var>old chain</var>, and <var>entry</var> is an <a id="focus-processing-model&#58;element"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code>, and the last entry in <var>new chain</var> is also an <a id="focus-processing-model&#58;element-2"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code>, then let <var>related blur target</var> be the last entry in <var>new chain</var>. Otherwise, let <var>related blur target</var> be null.

    4.  If <var>blur event target</var> is not null, <a id="focus-processing-model&#58;fire-a-focus-event"></a>[fire a focus event](#fire-a-focus-event) named <a id="focus-processing-model&#58;event-blur"></a><code>[blur](https://html.spec.whatwg.org/multipage/indices.html#event-blur)</code> at <var>blur event target</var>, with <var>related blur target</var> as the related target.

        <a id="note-sometimes-no-blur-event"></a>

        <strong>Note:</strong>

        [](#note-sometimes-no-blur-event)In some cases, e.g., if <var>entry</var> is an <a id="focus-processing-model&#58;the-area-element-4"></a><code>[area](https://html.spec.whatwg.org/multipage/image-maps.html#the-area-element)</code> element's shape, a scrollable region, or a <a id="focus-processing-model&#58;viewport-4"></a>[viewport](https://drafts.csswg.org/css2/#viewport), no event is fired.

3.  Apply any relevant platform-specific conventions for focusing <var>new focus target</var>. (For example, some platforms select the contents of a text control when that control is focused.)

4.  For each entry <var>entry</var> in <var>new chain</var>, in reverse order, run these substeps:

    1.  If <var>entry</var> is a <a id="focus-processing-model&#58;focusable-area-16"></a>[focusable area](#focusable-area), and the <a id="focus-processing-model&#58;focused-area-of-the-document"></a>[focused area of the document](#focused-area-of-the-document) is not <var>entry</var>:

        1.  Set <var>document</var>'s <a id="focus-processing-model&#58;concept-relevant-global-2"></a>[relevant global object](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global)'s <a id="focus-processing-model&#58;window-navigation-api"></a>[navigation API](https://html.spec.whatwg.org/multipage/nav-history-apis.html#window-navigation-api)'s <a id="focus-processing-model&#58;focus-changed-during-ongoing-navigation"></a>[focus changed during ongoing navigation](https://html.spec.whatwg.org/multipage/nav-history-apis.html#focus-changed-during-ongoing-navigation) to true.

        2.  Designate <var>entry</var> as the <a id="focus-processing-model&#58;focused-area-of-the-document-2"></a>[focused area of the document](#focused-area-of-the-document).

    2.  If <var>entry</var> is an element, let <var>focus event target</var> be <var>entry</var>.

        If <var>entry</var> is a <a id="focus-processing-model&#58;document-5"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> object, let <var>focus event target</var> be that <a id="focus-processing-model&#58;document-6"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> object's <a id="focus-processing-model&#58;concept-relevant-global-3"></a>[relevant global object](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global).

        Otherwise, let <var>focus event target</var> be null.

    3.  If <var>entry</var> is the last entry in <var>new chain</var>, and <var>entry</var> is an <a id="focus-processing-model&#58;element-3"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code>, and the last entry in <var>old chain</var> is also an <a id="focus-processing-model&#58;element-4"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code>, then let <var>related focus target</var> be the last entry in <var>old chain</var>. Otherwise, let <var>related focus target</var> be null.

    4.  If <var>focus event target</var> is not null, <a id="focus-processing-model&#58;fire-a-focus-event-2"></a>[fire a focus event](#fire-a-focus-event) named <a id="focus-processing-model&#58;event-focus"></a><code>[focus](https://html.spec.whatwg.org/multipage/indices.html#event-focus)</code> at <var>focus event target</var>, with <var>related focus target</var> as the related target.

        <strong>Note:</strong>

        In some cases, e.g. if <var>entry</var> is an <a id="focus-processing-model&#58;the-area-element-5"></a><code>[area](https://html.spec.whatwg.org/multipage/image-maps.html#the-area-element)</code> element's shape, a scrollable region, or a <a id="focus-processing-model&#58;viewport-5"></a>[viewport](https://drafts.csswg.org/css2/#viewport), no event is fired.

To <a id="fire-a-focus-event"></a><strong>fire a focus event</strong> named <var>e</var> at an element or a <a id="focus-processing-model&#58;window"></a><code>[Window](https://html.spec.whatwg.org/multipage/nav-history-apis.html#window)</code> object <var>target</var> given a related target <var>relatedTarget</var>:

1.  Let <var>view</var> be <var>target</var> if <var>target</var> is a <a id="focus-processing-model&#58;window-2"></a><code>[Window](https://html.spec.whatwg.org/multipage/nav-history-apis.html#window)</code> object; otherwise <var>target</var>'s <a id="focus-processing-model&#58;node-document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)'s <a id="focus-processing-model&#58;concept-relevant-global-4"></a>[relevant global object](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global).

2.  <a id="focus-processing-model&#58;concept-event-fire-2"></a>[Fire an event](https://dom.spec.whatwg.org/#concept-event-fire) named <var>e</var> at <var>target</var>, using <a id="focus-processing-model&#58;focusevent"></a><code>[FocusEvent](https://w3c.github.io/uievents/#focusevent)</code>, with the <a id="focus-processing-model&#58;dom-focusevent-relatedtarget"></a><code>[relatedTarget](https://w3c.github.io/uievents/#dom-focusevent-relatedtarget)</code> attribute initialized to <var>relatedTarget</var>, the <a id="focus-processing-model&#58;dom-uievent-view"></a><code>[view](https://w3c.github.io/uievents/#dom-uievent-view)</code> attribute initialized to <var>view</var>, and the <a id="focus-processing-model&#58;composed-flag"></a>[composed flag](https://dom.spec.whatwg.org/#composed-flag) set.

------------------------------------------------------------------------

When a key event is to be routed in a <a id="focus-processing-model&#58;top-level-traversable"></a>[top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#top-level-traversable), the user agent must run the following steps:

1.  Let <var>target area</var> be the <a id="focus-processing-model&#58;currently-focused-area-of-a-top-level-traversable-10"></a>[currently focused area of the top-level traversable](#currently-focused-area-of-a-top-level-traversable).

2.  <a id="focus-processing-model&#58;assert"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>target area</var> is not null, since key events are only routed to <a id="focus-processing-model&#58;top-level-traversable-2"></a>[top-level traversables](https://html.spec.whatwg.org/multipage/document-sequences.html#top-level-traversable) that have <a id="focus-processing-model&#58;system-focus-3"></a>[system focus](#system-focus). Therefore, <var>target area</var> is a <a id="focus-processing-model&#58;focusable-area-17"></a>[focusable area](#focusable-area).

3.  Let <var>target node</var> be <var>target area</var>'s <a id="focus-processing-model&#58;dom-anchor-5"></a>[DOM anchor](#dom-anchor).

4.  If <var>target node</var> is a <a id="focus-processing-model&#58;document-7"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> that has a <a id="focus-processing-model&#58;the-body-element-2"></a>[body element](https://html.spec.whatwg.org/multipage/dom.html#the-body-element-2), then let <var>target node</var> be <a id="focus-processing-model&#58;the-body-element-2-2"></a>[the body element](https://html.spec.whatwg.org/multipage/dom.html#the-body-element-2) of that <a id="focus-processing-model&#58;document-8"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code>.

    Otherwise, if <var>target node</var> is a <a id="focus-processing-model&#58;document-9"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> object that has a non-null <a id="focus-processing-model&#58;document-element-2"></a>[document element](https://dom.spec.whatwg.org/#document-element), then let <var>target node</var> be that <a id="focus-processing-model&#58;document-element-3"></a>[document element](https://dom.spec.whatwg.org/#document-element).

5.  If <var>target node</var> is not <a id="focus-processing-model&#58;inert-3"></a>[inert](#inert):

    1.  Let <var>canHandle</var> be the result of <a id="focus-processing-model&#58;concept-event-dispatch"></a>[dispatching](https://dom.spec.whatwg.org/#concept-event-dispatch) the key event at <var>target node</var>.

    2.  If <var>canHandle</var> is true, then let <var>target area</var> handle the key event. This might include <a id="focus-processing-model&#58;fire-a-click-event"></a>[firing a <code>click</code> event](https://html.spec.whatwg.org/multipage/webappapis.html#fire-a-click-event) at <var>target node</var>.

------------------------------------------------------------------------

The <a id="has-focus-steps"></a><strong>has focus steps</strong>, given a <a id="focus-processing-model&#58;document-10"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> object <var>target</var>, are as follows:

1.  If <var>target</var>'s <a id="focus-processing-model&#58;node-navigable-3"></a>[node navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#node-navigable)'s <a id="focus-processing-model&#58;nav-top"></a>[top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-top) does not have <a id="focus-processing-model&#58;system-focus-4"></a>[system focus](#system-focus), then return false.

2.  Let <var>candidate</var> be <var>target</var>'s <a id="focus-processing-model&#58;node-navigable-4"></a>[node navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#node-navigable)'s <a id="focus-processing-model&#58;nav-top-2"></a>[top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-top)'s <a id="focus-processing-model&#58;active-document"></a>[active document](https://html.spec.whatwg.org/multipage/document-sequences.html#active-document).

3.  While true:

    1.  If <var>candidate</var> is <var>target</var>, then return true.

    2.  If the <a id="focus-processing-model&#58;focused-area-of-the-document-3"></a>[focused area](#focused-area-of-the-document) of <var>candidate</var> is a <a id="focus-processing-model&#58;navigable-container-4"></a>[navigable container](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable-container) with a non-null <a id="focus-processing-model&#58;content-navigable-5"></a>[content navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#content-navigable), then set <var>candidate</var> to the <a id="focus-processing-model&#58;nav-document-4"></a>[active document](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-document) of that <a id="focus-processing-model&#58;navigable-container-5"></a>[navigable container](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable-container)'s <a id="focus-processing-model&#58;content-navigable-6"></a>[content navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#content-navigable).

    3.  Otherwise, return false.

#### <a id="sequential-focus-navigation"></a>6.6.5 <strong>Sequential focus navigation</strong>[](#sequential-focus-navigation)

Each <a id="sequential-focus-navigation&#58;document"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> has a <a id="sequential-focus-navigation-order"></a><strong>sequential focus navigation order</strong>, which orders some or all of the <a id="sequential-focus-navigation&#58;focusable-area"></a>[focusable areas](#focusable-area) in the <a id="sequential-focus-navigation&#58;document-2"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> relative to each other. Its contents and ordering are given by the <a id="sequential-focus-navigation&#58;flattened-tabindex-ordered-focus-navigation-scope"></a>[flattened tabindex-ordered focus navigation scope](#flattened-tabindex-ordered-focus-navigation-scope) of the <a id="sequential-focus-navigation&#58;document-3"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code>.

<strong>Note:</strong>

Per the rules defining the <a id="sequential-focus-navigation&#58;flattened-tabindex-ordered-focus-navigation-scope-2"></a>[flattened tabindex-ordered focus navigation scope](#flattened-tabindex-ordered-focus-navigation-scope), the ordering is not necessarily related to the <a id="sequential-focus-navigation&#58;tree-order"></a>[tree order](https://dom.spec.whatwg.org/#concept-tree-order) of the <a id="sequential-focus-navigation&#58;document-4"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code>.

If a <a id="sequential-focus-navigation&#58;focusable-area-2"></a>[focusable area](#focusable-area) is omitted from the <a id="sequential-focus-navigation&#58;sequential-focus-navigation-order"></a>[sequential focus navigation order](#sequential-focus-navigation-order) of its <a id="sequential-focus-navigation&#58;document-5"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code>, then it is unreachable via <a id="sequential-focus-navigation&#58;sequential-focus-navigation"></a>[sequential focus navigation](#sequential-focus-navigation).

There can also be a <a id="sequential-focus-navigation-starting-point"></a><strong>sequential focus navigation starting point</strong>. It is initially unset. The user agent may set it when the user indicates that it should be moved.

<strong>Example:</strong>

For example, the user agent could set it to the position of the user's click if the user clicks on the document contents.

<strong>Note:</strong>

User agents are required to set the <a id="sequential-focus-navigation&#58;sequential-focus-navigation-starting-point"></a>[sequential focus navigation starting point](#sequential-focus-navigation-starting-point) to the <a id="sequential-focus-navigation&#58;target-element"></a>[target element](https://html.spec.whatwg.org/multipage/browsing-the-web.html#target-element) when <a id="sequential-focus-navigation&#58;navigate-fragid"></a>[navigating to a fragment](https://html.spec.whatwg.org/multipage/browsing-the-web.html#navigate-fragid).

A <a id="sequential-focus-direction"></a><strong>sequential focus direction</strong> is one of two possible values: "<a id="sequential-focus-forward"></a><strong><code>forward</code></strong>", or "<a id="sequential-focus-backward"></a><strong><code>backward</code></strong>". They are used in the below algorithms to describe the direction in which sequential focus travels at the user's request.

A <a id="selection-mechanism"></a><strong>selection mechanism</strong> is one of two possible values: "<a id="selection-mechanism-dom"></a><strong><code>DOM</code></strong>", or "<a id="selection-mechanism-sequential"></a><strong><code>sequential</code></strong>". They are used to describe how the <a id="sequential-focus-navigation&#58;sequential-navigation-search-algorithm"></a>[sequential navigation search algorithm](#sequential-navigation-search-algorithm) finds the <a id="sequential-focus-navigation&#58;focusable-area-3"></a>[focusable area](#focusable-area) it returns.

When the user requests that focus move from the <a id="sequential-focus-navigation&#58;currently-focused-area-of-a-top-level-traversable"></a>[currently focused area of a top-level traversable](#currently-focused-area-of-a-top-level-traversable) to the next or previous <a id="sequential-focus-navigation&#58;focusable-area-4"></a>[focusable area](#focusable-area) (e.g., as the default action of pressing the tab key), or when the user requests that focus sequentially move to a <a id="sequential-focus-navigation&#58;top-level-traversable"></a>[top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#top-level-traversable) in the first place (e.g., from the browser's location bar), the user agent must use the following algorithm:

1.  Let <var>starting point</var> be the <a id="sequential-focus-navigation&#58;currently-focused-area-of-a-top-level-traversable-2"></a>[currently focused area of a top-level traversable](#currently-focused-area-of-a-top-level-traversable), if the user requested to move focus sequentially from there, or else the <a id="sequential-focus-navigation&#58;top-level-traversable-2"></a>[top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#top-level-traversable) itself, if the user instead requested to move focus from outside the <a id="sequential-focus-navigation&#58;top-level-traversable-3"></a>[top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#top-level-traversable).

2.  If there is a <a id="sequential-focus-navigation&#58;sequential-focus-navigation-starting-point-2"></a>[sequential focus navigation starting point](#sequential-focus-navigation-starting-point) defined and it is inside <var>starting point</var>, then let <var>starting point</var> be the <a id="sequential-focus-navigation&#58;sequential-focus-navigation-starting-point-3"></a>[sequential focus navigation starting point](#sequential-focus-navigation-starting-point) instead.

3.  Let <var>direction</var> be "<a id="sequential-focus-navigation&#58;sequential-focus-forward"></a><code>[forward](#sequential-focus-forward)</code>" if the user requested the next control, and "<a id="sequential-focus-navigation&#58;sequential-focus-backward"></a><code>[backward](#sequential-focus-backward)</code>" if the user requested the previous control.

    <strong>Note:</strong>

    Typically, pressing tab requests the next control, and pressing shift + tab requests the previous control.

4.  <em>Loop</em>: Let <var>selection mechanism</var> be "<a id="sequential-focus-navigation&#58;selection-mechanism-sequential"></a><code>[sequential](#selection-mechanism-sequential)</code>" if <var>starting point</var> is a <a id="sequential-focus-navigation&#58;navigable"></a>[navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable) or if <var>starting point</var> is in its <a id="sequential-focus-navigation&#58;document-6"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code>'s <a id="sequential-focus-navigation&#58;sequential-focus-navigation-order-2"></a>[sequential focus navigation order](#sequential-focus-navigation-order).

    Otherwise, <var>starting point</var> is not in its <a id="sequential-focus-navigation&#58;document-7"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code>'s <a id="sequential-focus-navigation&#58;sequential-focus-navigation-order-3"></a>[sequential focus navigation order](#sequential-focus-navigation-order); let <var>selection mechanism</var> be "<a id="sequential-focus-navigation&#58;selection-mechanism-dom"></a><code>[DOM](#selection-mechanism-dom)</code>".

5.  Let <var>candidate</var> be the result of running the <a id="sequential-focus-navigation&#58;sequential-navigation-search-algorithm-2"></a>[sequential navigation search algorithm](#sequential-navigation-search-algorithm) with <var>starting point</var>, <var>direction</var>, and <var>selection mechanism</var>.

6.  If <var>candidate</var> is not null, then run the <a id="sequential-focus-navigation&#58;focusing-steps"></a>[focusing steps](#focusing-steps) for <var>candidate</var> and return.

7.  Otherwise, unset the <a id="sequential-focus-navigation&#58;sequential-focus-navigation-starting-point-4"></a>[sequential focus navigation starting point](#sequential-focus-navigation-starting-point).

8.  If <var>starting point</var> is a <a id="sequential-focus-navigation&#58;top-level-traversable-4"></a>[top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#top-level-traversable), or a <a id="sequential-focus-navigation&#58;focusable-area-5"></a>[focusable area](#focusable-area) in the <a id="sequential-focus-navigation&#58;top-level-traversable-5"></a>[top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#top-level-traversable), the user agent should transfer focus to its own controls appropriately (if any), honouring <var>direction</var>, and then return.

    <strong>Example:</strong>

    For example, if <var>direction</var> is <em>backward</em>, then the last <a id="sequential-focus-navigation&#58;sequentially-focusable"></a>[sequentially focusable](#sequentially-focusable) control before the browser's rendering area would be the control to focus.

    If the user agent has no <a id="sequential-focus-navigation&#58;sequentially-focusable-2"></a>[sequentially focusable](#sequentially-focusable) controls — a kiosk-mode browser, for instance — then the user agent may instead restart these steps with the <var>starting point</var> being the <a id="sequential-focus-navigation&#58;top-level-traversable-6"></a>[top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#top-level-traversable) itself.

9.  Otherwise, <var>starting point</var> is a <a id="sequential-focus-navigation&#58;focusable-area-6"></a>[focusable area](#focusable-area) in a <a id="sequential-focus-navigation&#58;child-navigable"></a>[child navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#child-navigable). Set <var>starting point</var> to that <a id="sequential-focus-navigation&#58;child-navigable-2"></a>[child navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#child-navigable)'s <a id="sequential-focus-navigation&#58;nav-parent"></a>[parent](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-parent) and return to the step labeled <em>loop</em>.

The <a id="sequential-navigation-search-algorithm"></a><strong>sequential navigation search algorithm</strong>, given a <a id="sequential-focus-navigation&#58;focusable-area-7"></a>[focusable area](#focusable-area) <var>starting point</var>, <a id="sequential-focus-navigation&#58;sequential-focus-direction"></a>[sequential focus direction](#sequential-focus-direction) <var>direction</var>, and <a id="sequential-focus-navigation&#58;selection-mechanism"></a>[selection mechanism](#selection-mechanism) <var>selection mechanism</var>, consists of the following steps. They return a <a id="sequential-focus-navigation&#58;focusable-area-8"></a>[focusable area](#focusable-area)-or-null.

1.  Pick the appropriate cell from the following table, and follow the instructions in that cell.

    The appropriate cell is the one that is from the column whose header describes <var>direction</var> and from the first row whose header describes <var>starting point</var> and <var>selection mechanism</var>.

    <table><thead><tr><th>        </th><th><var>direction</var> is "<code id="sequential-focus-navigation:sequential-focus-forward-2"><a href="#sequential-focus-forward">forward</a></code>"        </th><th><var>direction</var> is "<code id="sequential-focus-navigation:sequential-focus-backward-2"><a href="#sequential-focus-backward">backward</a></code>"      </th></tr></thead><tbody><tr><th><var>starting point</var> is a <a id="sequential-focus-navigation:navigable-2" href="https://html.spec.whatwg.org/multipage/document-sequences.html#navigable">navigable</a>        </th><td>Let <var>candidate</var> be the first <a href="#suitable-sequentially-focusable-area" id="sequential-focus-navigation:suitable-sequentially-focusable-area">suitable sequentially focusable area</a>        in <var>starting point</var>'s <a href="https://html.spec.whatwg.org/multipage/document-sequences.html#nav-document" id="sequential-focus-navigation:nav-document">active document</a>, if any; or        else null.        </td><td>Let <var>candidate</var> be the last <a href="#suitable-sequentially-focusable-area" id="sequential-focus-navigation:suitable-sequentially-focusable-area-2">suitable sequentially focusable area</a>        in <var>starting point</var>'s <a href="https://html.spec.whatwg.org/multipage/document-sequences.html#nav-document" id="sequential-focus-navigation:nav-document-2">active document</a>, if any; or        else null.       </td></tr><tr><th><var>selection mechanism</var> is "<code id="sequential-focus-navigation:selection-mechanism-dom-2"><a href="#selection-mechanism-dom">DOM</a></code>"        </th><td>         <p>Let <var>candidate</var> be the <a href="#suitable-sequentially-focusable-area" id="sequential-focus-navigation:suitable-sequentially-focusable-area-3">suitable sequentially focusable area</a>, that         appears nearest after <var>starting point</var> in <var>starting point</var>'s         <code id="sequential-focus-navigation:document-8"><a href="https://html.spec.whatwg.org/multipage/dom.html#document">Document</a></code>, in <a id="sequential-focus-navigation:shadow-including-tree-order" href="https://dom.spec.whatwg.org/#concept-shadow-including-tree-order" data-x-internal="shadow-including-tree-order">shadow-including tree order</a>, if any; or else null.</p>          <p class="note">In this case, <var>starting point</var> does not necessarily belong to its         <code id="sequential-focus-navigation:document-9"><a href="https://html.spec.whatwg.org/multipage/dom.html#document">Document</a></code>'s <a href="#sequential-focus-navigation-order" id="sequential-focus-navigation:sequential-focus-navigation-order-4">sequential focus navigation order</a>, so we'll select the         <a href="#suitable-sequentially-focusable-area" id="sequential-focus-navigation:suitable-sequentially-focusable-area-4">suitable</a> <a href="https://infra.spec.whatwg.org/#list-item" id="sequential-focus-navigation:list-item" data-x-internal="list-item">item</a> from that list that comes <em>after</em> <var>starting point</var> in         <a id="sequential-focus-navigation:shadow-including-tree-order-2" href="https://dom.spec.whatwg.org/#concept-shadow-including-tree-order" data-x-internal="shadow-including-tree-order">shadow-including tree order</a>.</p>        </td><td>Let <var>candidate</var> be the <a href="#suitable-sequentially-focusable-area" id="sequential-focus-navigation:suitable-sequentially-focusable-area-5">suitable sequentially focusable area</a>, that        appears nearest before <var>starting point</var> in <var>starting point</var>'s        <code id="sequential-focus-navigation:document-10"><a href="https://html.spec.whatwg.org/multipage/dom.html#document">Document</a></code>, in <a id="sequential-focus-navigation:shadow-including-tree-order-3" href="https://dom.spec.whatwg.org/#concept-shadow-including-tree-order" data-x-internal="shadow-including-tree-order">shadow-including tree order</a>, if any; or else null.       </td></tr><tr><th><var>selection mechanism</var> is "<code id="sequential-focus-navigation:selection-mechanism-sequential-2"><a href="#selection-mechanism-sequential">sequential</a></code>"        </th><td>Let <var>candidate</var> be the first <a href="#suitable-sequentially-focusable-area" id="sequential-focus-navigation:suitable-sequentially-focusable-area-6">suitable sequentially focusable area</a>        after <var>starting point</var>, in <var>starting point</var>'s <code id="sequential-focus-navigation:document-11"><a href="https://html.spec.whatwg.org/multipage/dom.html#document">Document</a></code>'s        <a href="#sequential-focus-navigation-order" id="sequential-focus-navigation:sequential-focus-navigation-order-5">sequential focus navigation order</a>, if any; or else null.        </td><td>Let <var>candidate</var> be the last <a href="#suitable-sequentially-focusable-area" id="sequential-focus-navigation:suitable-sequentially-focusable-area-7">suitable sequentially focusable area</a>        before <var>starting point</var>, in <var>starting point</var>'s <code id="sequential-focus-navigation:document-12"><a href="https://html.spec.whatwg.org/multipage/dom.html#document">Document</a></code>'s        <a href="#sequential-focus-navigation-order" id="sequential-focus-navigation:sequential-focus-navigation-order-6">sequential focus navigation order</a>, if any; or else null.     </td></tr></tbody></table>

    A <a id="suitable-sequentially-focusable-area"></a><strong>suitable sequentially focusable area</strong> is a <a id="sequential-focus-navigation&#58;focusable-area-9"></a>[focusable area](#focusable-area) whose <a id="sequential-focus-navigation&#58;dom-anchor"></a>[DOM anchor](#dom-anchor) is not <a id="sequential-focus-navigation&#58;inert"></a>[inert](#inert) and is <a id="sequential-focus-navigation&#58;sequentially-focusable-3"></a>[sequentially focusable](#sequentially-focusable).

2.  If <var>candidate</var> is a <a id="sequential-focus-navigation&#58;navigable-container"></a>[navigable container](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable-container) with a non-null <a id="sequential-focus-navigation&#58;content-navigable"></a>[content navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#content-navigable):

    1.  Let <var>recursive candidate</var> be the result of running the <a id="sequential-focus-navigation&#58;sequential-navigation-search-algorithm-3"></a>[sequential navigation search algorithm](#sequential-navigation-search-algorithm) with <var>candidate</var>'s <a id="sequential-focus-navigation&#58;content-navigable-2"></a>[content navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#content-navigable), <var>direction</var>, and "<a id="sequential-focus-navigation&#58;selection-mechanism-sequential-3"></a><code>[sequential](#selection-mechanism-sequential)</code>".

    2.  If <var>recursive candidate</var> is null, then return the result of running the <a id="sequential-focus-navigation&#58;sequential-navigation-search-algorithm-4"></a>[sequential navigation search algorithm](#sequential-navigation-search-algorithm) with <var>candidate</var>, <var>direction</var>, and <var>selection mechanism</var>.

    3.  Otherwise, set <var>candidate</var> to <var>recursive candidate</var>.

3.  Return <var>candidate</var>.

#### <a id="focus-management-apis"></a>6.6.6 Focus management APIs[](#focus-management-apis)

<a id="focusoptions"></a><a id="focus-management-apis&#58;dom-focusoptions-preventscroll"></a><a id="focus-management-apis&#58;dom-focusoptions-focusvisible"></a>

``` text
dictionary FocusOptions {
  boolean preventScroll = false;
  boolean focusVisible;
};
```

<strong><code><var>documentOrShadowRoot</var>.<a id="dom-documentorshadowroot-activeelement-dev"></a>[activeElement](#dom-documentorshadowroot-activeelement)</code></strong>

<strong>✔</strong>MDN

[Document/activeElement](https://developer.mozilla.org/en-US/docs/Web/API/Document/activeElement)

Support in all current engines.

Firefox3+Safari4+Chrome2+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer6+

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android37+Samsung Internet?Opera Android12.1+

[ShadowRoot/activeElement](https://developer.mozilla.org/en-US/docs/Web/API/ShadowRoot/activeElement)

Support in all current engines.

Firefox63+Safari10+Chrome53+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)?Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android?Samsung Internet?Opera Android?

Returns the deepest element in <var>documentOrShadowRoot</var> through which or to which key events are being routed. This is, roughly speaking, the focused element in the document.

For the purposes of this API, when a <a id="focus-management-apis&#58;child-navigable"></a>[child navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#child-navigable) is focused, its <a id="focus-management-apis&#58;nav-container"></a>[container](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-container) is [focused](#bc-focus-ergo-bcc-focus) within its <a id="focus-management-apis&#58;nav-parent"></a>[parent](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-parent)'s <a id="focus-management-apis&#58;nav-document"></a>[active document](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-document). For example, if the user moves the focus to a text control in an <a id="focus-management-apis&#58;the-iframe-element"></a><code>[iframe](https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element)</code>, the <a id="focus-management-apis&#58;the-iframe-element-2"></a><code>[iframe](https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element)</code> is the element returned by the <a id="focus-management-apis&#58;dom-documentorshadowroot-activeelement"></a><code>[activeElement](#dom-documentorshadowroot-activeelement)</code> API in the <a id="focus-management-apis&#58;the-iframe-element-3"></a><code>[iframe](https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element)</code>'s <a id="focus-management-apis&#58;node-document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document).

Similarly, when the focused element is in a different <a id="focus-management-apis&#58;node-tree"></a>[node tree](https://dom.spec.whatwg.org/#concept-node-tree) than <var>documentOrShadowRoot</var>, the element returned will be the <a id="focus-management-apis&#58;concept-documentfragment-host"></a>[host](https://dom.spec.whatwg.org/#concept-documentfragment-host) that's located in the same <a id="focus-management-apis&#58;node-tree-2"></a>[node tree](https://dom.spec.whatwg.org/#concept-node-tree) as <var>documentOrShadowRoot</var> if <var>documentOrShadowRoot</var> is a <a id="focus-management-apis&#58;shadow-including-inclusive-ancestor"></a>[shadow-including inclusive ancestor](https://dom.spec.whatwg.org/#concept-shadow-including-inclusive-ancestor) of the focused element, and null if not.

<strong><code><var>document</var>.<a id="dom-document-hasfocus-dev"></a>[hasFocus](#dom-document-hasfocus)()</code></strong>

<strong>✔</strong>MDN

[Document/hasFocus](https://developer.mozilla.org/en-US/docs/Web/API/Document/hasFocus)

Support in all current engines.

Firefox3+Safari4+Chrome2+

------------------------------------------------------------------------

Opera?Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer5.5+

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android37+Samsung Internet?Opera Android?

Returns true if key events are being routed through or to <var>document</var>; otherwise, returns false. Roughly speaking, this corresponds to <var>document</var>, or a document nested inside <var>document</var>, being focused.

<strong><code><var>window</var>.<a id="dom-window-focus-dev"></a>[focus](#dom-window-focus)()</code></strong>

<strong>✔</strong>MDN

[Window/focus](https://developer.mozilla.org/en-US/docs/Web/API/Window/focus)

Support in all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer4+

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android18+WebView Android?Samsung Internet?Opera Android12.1+

Moves the focus to <var>window</var>'s <a id="focus-management-apis&#58;window-navigable"></a>[navigable](https://html.spec.whatwg.org/multipage/nav-history-apis.html#window-navigable), if any.

<strong><code><var>element</var>.<a id="dom-focus-dev"></a>[focus](#dom-focus)()</code></strong>

<strong>✔</strong>MDN

[HTMLElement/focus](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/focus)

Support in all current engines.

Firefox1.5+Safari3+Chrome1+

------------------------------------------------------------------------

Opera8+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer5.5+

------------------------------------------------------------------------

Firefox Android?Safari iOS1+Chrome Android?WebView Android?Samsung Internet?Opera Android10.1+

<strong><code><var>element</var>.<a id="focus-management-apis&#58;dom-focus"></a>[focus](#dom-focus)({ <a id="focus-management-apis&#58;dom-focusoptions-preventscroll-2"></a>[preventScroll](#dom-focusoptions-preventscroll), <a id="focus-management-apis&#58;dom-focusoptions-focusvisible-2"></a>[focusVisible](#dom-focusoptions-focusvisible) })</code></strong>

Moves the focus to <var>element</var>.

If <var>element</var> is a <a id="focus-management-apis&#58;navigable-container"></a>[navigable container](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable-container), moves the focus to its <a id="focus-management-apis&#58;content-navigable"></a>[content navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#content-navigable) instead.

By default, this method also scrolls <var>element</var> into view. Providing the <a id="focus-management-apis&#58;dom-focusoptions-preventscroll-3"></a><code>[preventScroll](#dom-focusoptions-preventscroll)</code> option and setting it to true prevents this behavior.

By default, user agents use <a id="focus-management-apis&#58;implementation-defined"></a>[implementation-defined](https://infra.spec.whatwg.org/#implementation-defined) heuristics to determine whether to <a id="focus-management-apis&#58;indicate-focus"></a>[indicate focus](https://drafts.csswg.org/selectors/#indicate-focus) via a focus ring. Providing the <a id="focus-management-apis&#58;dom-focusoptions-focusvisible-3"></a><code>[focusVisible](#dom-focusoptions-focusvisible)</code> option and setting it to true will ensure the focus ring is always visible.

<strong><code><var>element</var>.<a id="dom-blur-dev"></a>[blur](#dom-blur)()</code></strong>

<strong>✔</strong>MDN

[HTMLElement/blur](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/blur)

Support in all current engines.

Firefox1.5+Safari3+Chrome1+

------------------------------------------------------------------------

Opera8+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer5.5+

------------------------------------------------------------------------

Firefox Android?Safari iOS1+Chrome Android?WebView Android?Samsung Internet?Opera Android10.1+

Moves the focus to the <a id="focus-management-apis&#58;viewport"></a>[viewport](https://drafts.csswg.org/css2/#viewport). Use of this method is discouraged; if you want to focus the <a id="focus-management-apis&#58;viewport-2"></a>[viewport](https://drafts.csswg.org/css2/#viewport), call the <a id="focus-management-apis&#58;dom-focus-2"></a><code>[focus()](#dom-focus)</code> method on the <a id="focus-management-apis&#58;document"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code>'s <a id="focus-management-apis&#58;document-element"></a>[document element](https://dom.spec.whatwg.org/#document-element).

Do not use this method to hide the focus ring if you find the focus ring unsightly. Instead, use the <a id="focus-management-apis&#58;&#58;focus-visible"></a><code>[:focus-visible](https://drafts.csswg.org/selectors/#the-focus-visible-pseudo)</code> pseudo-class to override the <a id="focus-management-apis&#58;'outline'"></a>['outline'](https://drafts.csswg.org/css-ui/#outline) property, and provide a different way to show what element is focused. Be aware that if an alternative focusing style isn't made available, the page will be significantly less usable for people who primarily navigate pages using a keyboard, or those with reduced vision who use focus outlines to help them navigate the page.

<strong>Example:</strong>

For example, to hide the outline from <a id="focus-management-apis&#58;the-textarea-element"></a><code>[textarea](https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element)</code> elements and instead use a yellow background to indicate focus, you could use:

``` text
textarea:focus-visible { outline: none; background: yellow; color: black; }
```

<a id="dom-document-activeelement"></a>The <a id="focus-management-apis&#58;documentorshadowroot"></a><code>[DocumentOrShadowRoot](https://html.spec.whatwg.org/multipage/dom.html#documentorshadowroot)</code> <a id="dom-documentorshadowroot-activeelement"></a><strong><code>activeElement</code></strong> getter steps are:

1.  Let <var>candidate</var> be <a id="focus-management-apis&#58;this"></a>[this](https://webidl.spec.whatwg.org/#this)'s <a id="focus-management-apis&#58;node-document-2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document)'s <a id="focus-management-apis&#58;focused-area-of-the-document"></a>[focused area](#focused-area-of-the-document)'s <a id="focus-management-apis&#58;dom-anchor"></a>[DOM anchor](#dom-anchor).

2.  Set <var>candidate</var> to the result of <a id="focus-management-apis&#58;dom-retarget"></a>[retargeting](https://dom.spec.whatwg.org/#retarget) <var>candidate</var> against <a id="focus-management-apis&#58;this-2"></a>[this](https://webidl.spec.whatwg.org/#this).

3.  If <var>candidate</var>'s <a id="focus-management-apis&#58;root"></a>[root](https://dom.spec.whatwg.org/#concept-tree-root) is not <a id="focus-management-apis&#58;this-3"></a>[this](https://webidl.spec.whatwg.org/#this), then return null.

4.  If <var>candidate</var> is not a <a id="focus-management-apis&#58;document-2"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> object, then return <var>candidate</var>.

5.  If <var>candidate</var> has a <a id="focus-management-apis&#58;the-body-element-2"></a>[body element](https://html.spec.whatwg.org/multipage/dom.html#the-body-element-2), then return that <a id="focus-management-apis&#58;the-body-element-2-2"></a>[body element](https://html.spec.whatwg.org/multipage/dom.html#the-body-element-2).

6.  If <var>candidate</var>'s <a id="focus-management-apis&#58;document-element-2"></a>[document element](https://dom.spec.whatwg.org/#document-element) is non-null, then return that <a id="focus-management-apis&#58;document-element-3"></a>[document element](https://dom.spec.whatwg.org/#document-element).

7.  Return null.

The <a id="focus-management-apis&#58;document-3"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> <a id="dom-document-hasfocus"></a><strong><code>hasFocus()</code></strong> method steps are to return the result of running the <a id="focus-management-apis&#58;has-focus-steps"></a>[has focus steps](#has-focus-steps) given <a id="focus-management-apis&#58;this-4"></a>[this](https://webidl.spec.whatwg.org/#this).

------------------------------------------------------------------------

The <a id="focus-management-apis&#58;window"></a><code>[Window](https://html.spec.whatwg.org/multipage/nav-history-apis.html#window)</code> <a id="dom-window-focus"></a><strong><code>focus()</code></strong> method steps are:

1.  Let <var>current</var> be <a id="focus-management-apis&#58;this-5"></a>[this](https://webidl.spec.whatwg.org/#this)'s <a id="focus-management-apis&#58;window-navigable-2"></a>[navigable](https://html.spec.whatwg.org/multipage/nav-history-apis.html#window-navigable).

2.  If <var>current</var> is null, then return.

3.  If the <a id="focus-management-apis&#58;allow-focus-steps"></a>[allow focus steps](#allow-focus-steps) given <var>current</var>'s <a id="focus-management-apis&#58;nav-document-2"></a>[active document](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-document) return false, then return.

4.  Run the <a id="focus-management-apis&#58;focusing-steps"></a>[focusing steps](#focusing-steps) with <var>current</var>.

5.  If <var>current</var> is a <a id="focus-management-apis&#58;top-level-traversable"></a>[top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#top-level-traversable), user agents are encouraged to trigger some sort of notification to indicate to the user that the page is attempting to gain focus.

<strong>✔</strong>MDN

[Window/blur](https://developer.mozilla.org/en-US/docs/Web/API/Window/blur)

Support in all current engines.

Firefox1+Safari1+Chrome1+

------------------------------------------------------------------------

Opera12.1+Edge79+

------------------------------------------------------------------------

Edge (Legacy)12+Internet Explorer4+

------------------------------------------------------------------------

Firefox Android?Safari iOS?Chrome Android?WebView Android?Samsung Internet?Opera Android12.1+

The <a id="focus-management-apis&#58;window-2"></a><code>[Window](https://html.spec.whatwg.org/multipage/nav-history-apis.html#window)</code> <a id="dom-window-blur"></a><strong><code>blur()</code></strong> method steps are to do nothing.

<strong>Note:</strong>

Historically, the <a id="focus-management-apis&#58;dom-window-focus"></a><code>[focus()](#dom-window-focus)</code> and <a id="focus-management-apis&#58;dom-window-blur"></a><code>[blur()](#dom-window-blur)</code> methods actually affected the system-level focus of the system widget (e.g., tab or window) that contained the <a id="focus-management-apis&#58;navigable"></a>[navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable), but hostile sites widely abuse this behavior to the user's detriment.

------------------------------------------------------------------------

The <a id="focus-management-apis&#58;htmlorsvgelement"></a><code>[HTMLOrSVGOrMathMLElement](https://html.spec.whatwg.org/multipage/dom.html#htmlorsvgelement)</code> <a id="dom-focus"></a><strong><code>focus(<var>options</var>)</code></strong> method steps are:

1.  If the <a id="focus-management-apis&#58;allow-focus-steps-2"></a>[allow focus steps](#allow-focus-steps) given <a id="focus-management-apis&#58;this-6"></a>[this](https://webidl.spec.whatwg.org/#this)'s <a id="focus-management-apis&#58;node-document-3"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document) return false, then return.

2.  Run the <a id="focus-management-apis&#58;focusing-steps-2"></a>[focusing steps](#focusing-steps) for <a id="focus-management-apis&#58;this-7"></a>[this](https://webidl.spec.whatwg.org/#this).

3.  If <var>options</var>\["<a id="dom-focusoptions-focusvisible"></a><strong><code>focusVisible</code></strong>"\] is true, or does not <a id="focus-management-apis&#58;map-exists"></a>[exist](https://infra.spec.whatwg.org/#map-exists) but in an <a id="focus-management-apis&#58;implementation-defined-2"></a>[implementation-defined](https://infra.spec.whatwg.org/#implementation-defined) way the user agent determines it would be best to do so, then <a id="focus-management-apis&#58;indicate-focus-2"></a>[indicate focus](https://drafts.csswg.org/selectors/#indicate-focus).

4.  If <var>options</var>\["<a id="dom-focusoptions-preventscroll"></a><strong><code>preventScroll</code></strong>"\] is false, then <a id="focus-management-apis&#58;scroll-a-target-into-view"></a>[scroll a target into view](https://drafts.csswg.org/cssom-view/#scroll-a-target-into-view) given <a id="focus-management-apis&#58;this-8"></a>[this](https://webidl.spec.whatwg.org/#this), "<code>auto</code>", "<code>center</code>", and "<code>center</code>".

The <a id="focus-management-apis&#58;htmlorsvgelement-2"></a><code>[HTMLOrSVGOrMathMLElement](https://html.spec.whatwg.org/multipage/dom.html#htmlorsvgelement)</code> <a id="dom-blur"></a><strong><code>blur()</code></strong> method steps are:

1.  The user agent should run the <a id="focus-management-apis&#58;unfocusing-steps"></a>[unfocusing steps](#unfocusing-steps) given <a id="focus-management-apis&#58;this-9"></a>[this](https://webidl.spec.whatwg.org/#this).

    User agents may instead selectively or uniformly do nothing, for usability reasons.

<strong>Example:</strong>

For example, if the <a id="focus-management-apis&#58;dom-blur"></a><code>[blur()](#dom-blur)</code> method is unwisely being used to remove the focus ring for aesthetics reasons, the page would become unusable by keyboard users. Ignoring calls to this method would thus allow keyboard users to interact with the page.

------------------------------------------------------------------------

The <a id="allow-focus-steps"></a><strong>allow focus steps</strong>, given a <a id="focus-management-apis&#58;document-4"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> object <var>target</var>, are:

1.  If <var>target</var> is <a id="focus-management-apis&#58;allowed-to-use"></a>[allowed to use](https://html.spec.whatwg.org/multipage/iframe-embed-object.html#allowed-to-use) the "<a id="focus-management-apis&#58;focus-without-user-activation-feature"></a><code>[focus-without-user-activation](https://html.spec.whatwg.org/multipage/infrastructure.html#focus-without-user-activation-feature)</code>" feature, then return true.

2.  If <var>target</var>'s <a id="focus-management-apis&#58;concept-relevant-global"></a>[relevant global object](https://html.spec.whatwg.org/multipage/webappapis.html#concept-relevant-global) has <a id="focus-management-apis&#58;transient-activation"></a>[transient activation](#transient-activation), then return true.

3.  Return false.

#### <a id="the-autofocus-attribute"></a>6.6.7 <a id="autofocusing-a-form-control&#58;-the-autofocus-attribute"></a>The <a id="the-autofocus-attribute&#58;attr-fe-autofocus"></a><code>[autofocus](#attr-fe-autofocus)</code> attribute[](#the-autofocus-attribute)

The <a id="attr-fe-autofocus"></a><strong><code>autofocus</code></strong> content attribute allows the author to indicate that an element is to be focused as soon as the page is loaded, allowing the user to just start typing without having to manually focus the main element.

When the <a id="the-autofocus-attribute&#58;attr-fe-autofocus-2"></a><code>[autofocus](#attr-fe-autofocus)</code> attribute is specified on an element inside <a id="the-autofocus-attribute&#58;the-dialog-element"></a><code>[dialog](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element)</code> elements or <a id="the-autofocus-attribute&#58;html-elements"></a>[HTML elements](https://html.spec.whatwg.org/multipage/infrastructure.html#html-elements) whose <a id="the-autofocus-attribute&#58;attr-popover"></a><code>[popover](https://html.spec.whatwg.org/multipage/popover.html#attr-popover)</code> attribute is set, then it will be focused when the dialog or popover becomes shown.

The <a id="the-autofocus-attribute&#58;attr-fe-autofocus-3"></a><code>[autofocus](#attr-fe-autofocus)</code> attribute is a <a id="the-autofocus-attribute&#58;boolean-attribute"></a>[boolean attribute](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#boolean-attribute).

To find the <a id="nearest-ancestor-autofocus-scoping-root-element"></a><strong>nearest ancestor autofocus scoping root element</strong> given an <a id="the-autofocus-attribute&#58;element"></a><code>[Element](https://dom.spec.whatwg.org/#interface-element)</code> <var>element</var>:

1.  If <var>element</var> is a <a id="the-autofocus-attribute&#58;the-dialog-element-2"></a><code>[dialog](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element)</code> element, then return <var>element</var>.

2.  If <var>element</var>'s <a id="the-autofocus-attribute&#58;attr-popover-2"></a><code>[popover](https://html.spec.whatwg.org/multipage/popover.html#attr-popover)</code> attribute is not in the <a id="the-autofocus-attribute&#58;attr-popover-none-state"></a>[No Popover](https://html.spec.whatwg.org/multipage/popover.html#attr-popover-none-state) state, then return <var>element</var>.

3.  Let <var>ancestor</var> be <var>element</var>.

4.  While <var>ancestor</var> has a <a id="the-autofocus-attribute&#58;parent-element"></a>[parent element](https://dom.spec.whatwg.org/#parent-element):

    1.  Set <var>ancestor</var> to <var>ancestor</var>'s <a id="the-autofocus-attribute&#58;parent-element-2"></a>[parent element](https://dom.spec.whatwg.org/#parent-element).

    2.  If <var>ancestor</var> is a <a id="the-autofocus-attribute&#58;the-dialog-element-3"></a><code>[dialog](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element)</code> element, then return <var>ancestor</var>.

    3.  If <var>ancestor</var>'s <a id="the-autofocus-attribute&#58;attr-popover-3"></a><code>[popover](https://html.spec.whatwg.org/multipage/popover.html#attr-popover)</code> attribute is not in the <a id="the-autofocus-attribute&#58;attr-popover-none-state-2"></a>[No Popover](https://html.spec.whatwg.org/multipage/popover.html#attr-popover-none-state) state, then return <var>ancestor</var>.

5.  Return <var>ancestor</var>.

There must not be two elements with the same <a id="the-autofocus-attribute&#58;nearest-ancestor-autofocus-scoping-root-element"></a>[nearest ancestor autofocus scoping root element](#nearest-ancestor-autofocus-scoping-root-element) that both have the <a id="the-autofocus-attribute&#58;attr-fe-autofocus-4"></a><code>[autofocus](#attr-fe-autofocus)</code> attribute specified.

Each <a id="the-autofocus-attribute&#58;document"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> has an <a id="autofocus-candidates"></a><strong>autofocus candidates</strong> <a id="the-autofocus-attribute&#58;list"></a>[list](https://infra.spec.whatwg.org/#list), initially empty.

Each <a id="the-autofocus-attribute&#58;document-2"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> has an <a id="autofocus-processed-flag"></a><strong>autofocus processed flag</strong> boolean, initially false.

When an element with the <a id="the-autofocus-attribute&#58;attr-fe-autofocus-5"></a><code>[autofocus](#attr-fe-autofocus)</code> attribute specified is <a id="the-autofocus-attribute&#58;insert-an-element-into-a-document"></a>[inserted into a document](https://html.spec.whatwg.org/multipage/infrastructure.html#insert-an-element-into-a-document), run the following steps:

1.  If the user has indicated (for example, by starting to type in a form control) that they do not wish focus to be changed, then optionally return.

2.  Let <var>target</var> be the element's <a id="the-autofocus-attribute&#58;node-document"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document).

3.  If <var>target</var> is not <a id="the-autofocus-attribute&#58;fully-active"></a>[fully active](https://html.spec.whatwg.org/multipage/document-sequences.html#fully-active), then return.

4.  If <var>target</var>'s <a id="the-autofocus-attribute&#58;active-sandboxing-flag-set"></a>[active sandboxing flag set](https://html.spec.whatwg.org/multipage/browsers.html#active-sandboxing-flag-set) has the <a id="the-autofocus-attribute&#58;sandboxed-automatic-features-browsing-context-flag"></a>[sandboxed automatic features browsing context flag](https://html.spec.whatwg.org/multipage/browsers.html#sandboxed-automatic-features-browsing-context-flag), then return.

5.  If the <a id="the-autofocus-attribute&#58;allow-focus-steps"></a>[allow focus steps](#allow-focus-steps) given <var>target</var> return false, then return.

6.  Let <var>topDocument</var> be <var>target</var>'s <a id="the-autofocus-attribute&#58;node-navigable"></a>[node navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#node-navigable)'s <a id="the-autofocus-attribute&#58;nav-top"></a>[top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-top)'s <a id="the-autofocus-attribute&#58;nav-document"></a>[active document](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-document).

7.  If <var>topDocument</var>'s <a id="the-autofocus-attribute&#58;autofocus-processed-flag"></a>[autofocus processed flag](#autofocus-processed-flag) is false, then <a id="the-autofocus-attribute&#58;list-remove"></a>[remove](https://infra.spec.whatwg.org/#list-remove) the element from <var>topDocument</var>'s <a id="the-autofocus-attribute&#58;autofocus-candidates"></a>[autofocus candidates](#autofocus-candidates), and <a id="the-autofocus-attribute&#58;list-append"></a>[append](https://infra.spec.whatwg.org/#list-append) the element to <var>topDocument</var>'s <a id="the-autofocus-attribute&#58;autofocus-candidates-2"></a>[autofocus candidates](#autofocus-candidates).

<strong>Note:</strong>

We do not check if an element is a <a id="the-autofocus-attribute&#58;focusable-area"></a>[focusable area](#focusable-area) before storing it in the <a id="the-autofocus-attribute&#58;autofocus-candidates-3"></a>[autofocus candidates](#autofocus-candidates) list, because even if it is not a focusable area when it is inserted, it could become one by the time <a id="the-autofocus-attribute&#58;flush-autofocus-candidates"></a>[flush autofocus candidates](#flush-autofocus-candidates) sees it.

To <a id="flush-autofocus-candidates"></a><strong>flush autofocus candidates</strong> for a document <var>topDocument</var>, run these steps:

1.  If <var>topDocument</var>'s <a id="the-autofocus-attribute&#58;autofocus-processed-flag-2"></a>[autofocus processed flag](#autofocus-processed-flag) is true, then return.

2.  Let <var>candidates</var> be <var>topDocument</var>'s <a id="the-autofocus-attribute&#58;autofocus-candidates-4"></a>[autofocus candidates](#autofocus-candidates).

3.  If <var>candidates</var> <a id="the-autofocus-attribute&#58;list-is-empty"></a>[is empty](https://infra.spec.whatwg.org/#list-is-empty), then return.

4.  If <var>topDocument</var>'s <a id="the-autofocus-attribute&#58;focused-area-of-the-document"></a>[focused area](#focused-area-of-the-document) is not <var>topDocument</var> itself, or <var>topDocument</var> has non-null <a id="the-autofocus-attribute&#58;target-element"></a>[target element](https://html.spec.whatwg.org/multipage/browsing-the-web.html#target-element):

    1.  <a id="the-autofocus-attribute&#58;list-empty"></a>[Empty](https://infra.spec.whatwg.org/#list-empty) <var>candidates</var>.

    2.  Set <var>topDocument</var>'s <a id="the-autofocus-attribute&#58;autofocus-processed-flag-3"></a>[autofocus processed flag](#autofocus-processed-flag) to true.

    3.  Return.

5.  While <var>candidates</var> is not <a id="the-autofocus-attribute&#58;list-is-empty-2"></a>[empty](https://infra.spec.whatwg.org/#list-is-empty):

    1.  Let <var>element</var> be <var>candidates</var>\[0\].

    2.  Let <var>doc</var> be <var>element</var>'s <a id="the-autofocus-attribute&#58;node-document-2"></a>[node document](https://dom.spec.whatwg.org/#concept-node-document).

    3.  If <var>doc</var> is not <a id="the-autofocus-attribute&#58;fully-active-2"></a>[fully active](https://html.spec.whatwg.org/multipage/document-sequences.html#fully-active), then <a id="the-autofocus-attribute&#58;list-remove-2"></a>[remove](https://infra.spec.whatwg.org/#list-remove) <var>element</var> from <var>candidates</var>, and <a id="the-autofocus-attribute&#58;continue"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    4.  If <var>doc</var>'s <a id="the-autofocus-attribute&#58;node-navigable-2"></a>[node navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#node-navigable)'s <a id="the-autofocus-attribute&#58;nav-top-2"></a>[top-level traversable](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-top) is not the same as <var>topDocument</var>'s <a id="the-autofocus-attribute&#58;node-navigable-3"></a>[node navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#node-navigable), then <a id="the-autofocus-attribute&#58;list-remove-3"></a>[remove](https://infra.spec.whatwg.org/#list-remove) <var>element</var> from <var>candidates</var>, and <a id="the-autofocus-attribute&#58;continue-2"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    5.  If <var>doc</var>'s <a id="the-autofocus-attribute&#58;script-blocking-style-sheet-set"></a>[script-blocking style sheet set](https://html.spec.whatwg.org/multipage/semantics.html#script-blocking-style-sheet-set) is not <a id="the-autofocus-attribute&#58;list-is-empty-3"></a>[empty](https://infra.spec.whatwg.org/#list-is-empty), then return.

        <strong>Note:</strong>

        In this case, <var>element</var> is the currently-best candidate, but <var>doc</var> is not ready for autofocusing. We'll try again next time <a id="the-autofocus-attribute&#58;flush-autofocus-candidates-2"></a>[flush autofocus candidates](#flush-autofocus-candidates) is called.

    6.  <a id="the-autofocus-attribute&#58;list-remove-4"></a>[Remove](https://infra.spec.whatwg.org/#list-remove) <var>element</var> from <var>candidates</var>.

    7.  Let <var>inclusiveAncestorDocuments</var> be a <a id="the-autofocus-attribute&#58;list-2"></a>[list](https://infra.spec.whatwg.org/#list) consisting of the <a id="the-autofocus-attribute&#58;nav-document-2"></a>[active document](https://html.spec.whatwg.org/multipage/document-sequences.html#nav-document) of <var>doc</var>'s <a id="the-autofocus-attribute&#58;inclusive-ancestor-navigables"></a>[inclusive ancestor navigables](https://html.spec.whatwg.org/multipage/document-sequences.html#inclusive-ancestor-navigables).

    8.  If any <a id="the-autofocus-attribute&#58;document-3"></a><code>[Document](https://html.spec.whatwg.org/multipage/dom.html#document)</code> in <var>inclusiveAncestorDocuments</var> has non-null <a id="the-autofocus-attribute&#58;target-element-2"></a>[target element](https://html.spec.whatwg.org/multipage/browsing-the-web.html#target-element), then <a id="the-autofocus-attribute&#58;continue-3"></a>[continue](https://infra.spec.whatwg.org/#iteration-continue).

    9.  Let <var>target</var> be <var>element</var>.

    10. If <var>target</var> is not a <a id="the-autofocus-attribute&#58;focusable-area-2"></a>[focusable area](#focusable-area), then set <var>target</var> to the result of <a id="the-autofocus-attribute&#58;get-the-focusable-area"></a>[getting the focusable area](#get-the-focusable-area) for <var>target</var>.

        <strong>Note:</strong>

        <a id="the-autofocus-attribute&#58;autofocus-candidates-5"></a>[Autofocus candidates](#autofocus-candidates) can <a id="the-autofocus-attribute&#58;list-contains"></a>[contain](https://infra.spec.whatwg.org/#list-contain) elements which are not <a id="the-autofocus-attribute&#58;focusable-area-3"></a>[focusable areas](#focusable-area). In addition to the special cases handled in the <a id="the-autofocus-attribute&#58;get-the-focusable-area-2"></a>[get the focusable area](#get-the-focusable-area) algorithm, this can happen because a non-<a id="the-autofocus-attribute&#58;focusable-area-4"></a>[focusable area](#focusable-area) element with an <a id="the-autofocus-attribute&#58;attr-fe-autofocus-6"></a><code>[autofocus](#attr-fe-autofocus)</code> attribute was <a id="the-autofocus-attribute&#58;insert-an-element-into-a-document-2"></a>[inserted into a document](https://html.spec.whatwg.org/multipage/infrastructure.html#insert-an-element-into-a-document) and it never became focusable, or because the element was focusable but its status changed while it was stored in <a id="the-autofocus-attribute&#58;autofocus-candidates-6"></a>[autofocus candidates](#autofocus-candidates).

    11. If <var>target</var> is not null:

        1.  <a id="the-autofocus-attribute&#58;list-empty-2"></a>[Empty](https://infra.spec.whatwg.org/#list-empty) <var>candidates</var>.

        2.  Set <var>topDocument</var>'s <a id="the-autofocus-attribute&#58;autofocus-processed-flag-4"></a>[autofocus processed flag](#autofocus-processed-flag) to true.

        3.  Run the <a id="the-autofocus-attribute&#58;focusing-steps"></a>[focusing steps](#focusing-steps) for <var>target</var>.

<strong>Note:</strong>

This handles the automatic focusing during document load. The <a id="the-autofocus-attribute&#58;dom-dialog-show"></a><code>[show()](https://html.spec.whatwg.org/multipage/interactive-elements.html#dom-dialog-show)</code> and <a id="the-autofocus-attribute&#58;dom-dialog-showmodal"></a><code>[showModal()](https://html.spec.whatwg.org/multipage/interactive-elements.html#dom-dialog-showmodal)</code> methods of <a id="the-autofocus-attribute&#58;the-dialog-element-4"></a><code>[dialog](https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element)</code> elements also processes the <a id="the-autofocus-attribute&#58;attr-fe-autofocus-7"></a><code>[autofocus](#attr-fe-autofocus)</code> attribute.

<strong>Note:</strong>

Focusing the element does not imply that the user agent has to focus the browser window if it has lost focus.

<strong>Example:</strong>

In the following snippet, the text control would be focused when the document was loaded.

``` text
<input maxlength="256" name="q" value="" autofocus>
<input type="submit" value="Search">
```

<strong>Example:</strong>

The <a id="the-autofocus-attribute&#58;attr-fe-autofocus-8"></a><code>[autofocus](#attr-fe-autofocus)</code> attribute applies to all elements, not just to form controls. This allows examples such as the following:

``` text
<div contenteditable autofocus>Edit <strong>me!</strong><div>
```
