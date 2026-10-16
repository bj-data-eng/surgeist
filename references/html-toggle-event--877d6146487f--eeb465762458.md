# HTML Standard (Commit Snapshot 877d6146487f15b15bc8dd0e3c625063a37b456a): native event dispatch source capture

Attribution and reformatting notice added for Surgeist on 2026-10-15. Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Original English standard and normative/informative labels remain authoritative. This partial format conversion is implementation support, not a replacement specification or an implementation-conformance claim. No endorsement is implied.

Source: [HTML Standard (Commit Snapshot 877d6146487f15b15bc8dd0e3c625063a37b456a)](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/). Retained source retrieved 2026-10-11; source revision `877d6146487f15b15bc8dd0e3c625063a37b456a`. This is the already retained immutable HTML commit snapshot. No new source edition or retrieval is selected.

The exact upstream LICENSE files at the existing DOM/HTML pins match the already retained [WHATWG LICENSE](../licenses/whatwg/LICENSE), SHA-256 `85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`. Specification prose is Creative Commons Attribution 4.0 International; BSD 3-Clause applies to portions incorporated into source code, not as an alternative license for this Markdown. Existing [WHATWG attribution](../licenses/whatwg/NOTICE.md) records legal provenance.

Complete original HTML: 14,930,614 bytes, SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`. Complete-section/algorithm extraction input: 22,550 bytes, SHA-256 `eeb465762458707a47042dcd76b0d80702df840a698b6b648eaeae5a5c75a7c7`. Complete selected heading sections (including all descendant clauses, bounded at the next equal-or-higher heading): `the-toggleevent-interface`, `ipr`. Original source header and intellectual-property notices are retained. Exact retained ranges and conversion checks are recorded in the [conversion report](CONVERSION-REPORT.md#native-event-dispatch-sources--2026-10-15). Every other clause remains upstream.

Conversion uses the existing repository preparation/semantic filter and already-present Pandoc 3.1.11.1, without software acquisition. Scripts/styles are omitted without execution; normative body text, IDs, headings, inline code/variables, literal blocks, links and table cells/spans are compared against parsed GFM. Source JavaScript/WebIDL bindings remain evidence, not an assignment to implement a JS engine.

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

#### <a id="the&#45;toggleevent&#45;interface"></a>6&#46;5&#46;1 The <a id="the&#45;toggleevent&#45;interface&#58;toggleevent"></a><code>[ToggleEvent](#toggleevent)</code> interface[](#the-toggleevent-interface)

<strong>✔</strong>MDN

[ToggleEvent&#47;ToggleEvent](https://developer.mozilla.org/en-US/docs/Web/API/ToggleEvent/ToggleEvent)

Support in all current engines&#46;

Firefox🔰 114&#43;Safari17&#43;Chrome114&#43;

------------------------------------------------------------------------

Opera&#63;Edge114&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[ToggleEvent](https://developer.mozilla.org/en-US/docs/Web/API/ToggleEvent)

Support in all current engines&#46;

Firefox🔰 114&#43;Safari17&#43;Chrome114&#43;

------------------------------------------------------------------------

Opera&#63;Edge114&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

<a id="toggleevent"></a><a id="the&#45;toggleevent&#45;interface&#58;event"></a><a id="the&#45;toggleevent&#45;interface&#58;toggleeventinit"></a><a id="the&#45;toggleevent&#45;interface&#58;dom&#45;toggleevent&#45;oldstate"></a><a id="the&#45;toggleevent&#45;interface&#58;dom&#45;toggleevent&#45;newstate"></a><a id="the&#45;toggleevent&#45;interface&#58;dom&#45;toggleevent&#45;source"></a><a id="toggleeventinit"></a><a id="the&#45;toggleevent&#45;interface&#58;eventinit"></a>

``` text
[Exposed=Window]
interface ToggleEvent : Event {
  constructor(DOMString type, optional ToggleEventInit eventInitDict = {});
  readonly attribute DOMString oldState;
  readonly attribute DOMString newState;
  readonly attribute Element? source;
};

dictionary ToggleEventInit : EventInit {
  DOMString oldState = "";
  DOMString newState = "";
  Element? source = null;
};
```

<strong><code><var>event</var>&#46;<a id="dom&#45;toggleevent&#45;oldstate&#45;dev"></a>[oldState](#dom-toggleevent-oldstate)</code></strong>

Set to &#34;<code>closed</code>&#34; when transitioning from closed to open&#44; or set to &#34;<code>open</code>&#34; when transitioning from open to closed&#46;

<strong><code><var>event</var>&#46;<a id="dom&#45;toggleevent&#45;newstate&#45;dev"></a>[newState](#dom-toggleevent-newstate)</code></strong>

Set to &#34;<code>open</code>&#34; when transitioning from closed to open&#44; or set to &#34;<code>closed</code>&#34; when transitioning from open to closed&#46;

<strong><code><var>event</var>&#46;<a id="dom&#45;toggleevent&#45;source&#45;dev"></a>[source](#dom-toggleevent-source)</code></strong>

Set to the element which initiated the toggle&#44; which can be set up with the <a id="the&#45;toggleevent&#45;interface&#58;attr&#45;popovertarget"></a><code>[popovertarget](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-popovertarget)</code> and <a id="the&#45;toggleevent&#45;interface&#58;attr&#45;button&#45;commandfor"></a><code>[commandfor](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-button-commandfor)</code> attributes&#46; If there is no source element&#44; then it is set to null&#46;

<strong>✔</strong>MDN

[ToggleEvent&#47;oldState](https://developer.mozilla.org/en-US/docs/Web/API/ToggleEvent/oldState)

Support in all current engines&#46;

Firefox🔰 114&#43;Safari17&#43;Chrome114&#43;

------------------------------------------------------------------------

Opera&#63;Edge114&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

[ToggleEvent&#47;newState](https://developer.mozilla.org/en-US/docs/Web/API/ToggleEvent/newState)

Support in all current engines&#46;

Firefox🔰 114&#43;Safari17&#43;Chrome114&#43;

------------------------------------------------------------------------

Opera&#63;Edge114&#43;

------------------------------------------------------------------------

Edge &#40;Legacy&#41;&#63;Internet ExplorerNo

------------------------------------------------------------------------

Firefox Android&#63;Safari iOS&#63;Chrome Android&#63;WebView Android&#63;Samsung Internet&#63;Opera Android&#63;

The <a id="dom&#45;toggleevent&#45;oldstate"></a><strong><code>oldState</code></strong> and <a id="dom&#45;toggleevent&#45;newstate"></a><strong><code>newState</code></strong> attributes must return the values they are initialized to&#46;

The <a id="dom&#45;toggleevent&#45;source"></a><strong><code>source</code></strong> getter steps are to return the result of <a id="the&#45;toggleevent&#45;interface&#58;dom&#45;retarget"></a>[retargeting](https://dom.spec.whatwg.org/#retarget) <a id="the&#45;toggleevent&#45;interface&#58;dom&#45;toggleevent&#45;source&#45;2"></a><code>[source](#dom-toggleevent-source)</code> against <a id="the&#45;toggleevent&#45;interface&#58;this"></a>[this](https://webidl.spec.whatwg.org/#this)&#39;s <a id="the&#45;toggleevent&#45;interface&#58;dom&#45;event&#45;currenttarget"></a><code>[currentTarget](https://dom.spec.whatwg.org/#dom-event-currenttarget)</code>&#46;

[DOM standard issue &#35;1328](https://github.com/whatwg/dom/issues/1328) tracks how to better standardize associated event data in a way which makes sense on Events&#46; Currently an event attribute initialized to a value cannot also have a getter&#44; and so an internal slot &#40;or map of additional fields&#41; is required to properly specify this&#46;

A <a id="toggle&#45;task&#45;tracker"></a><strong>toggle task tracker</strong> is a <a id="the&#45;toggleevent&#45;interface&#58;struct"></a>[struct](https://infra.spec.whatwg.org/#struct) which has&#58;

<strong><a id="toggle&#45;task&#45;task"></a><strong>task</strong></strong>

A <a id="the&#45;toggleevent&#45;interface&#58;concept&#45;task"></a>[task](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task) which fires a <a id="the&#45;toggleevent&#45;interface&#58;toggleevent&#45;2"></a><code>[ToggleEvent](#toggleevent)</code>&#46;

<strong><a id="toggle&#45;task&#45;old&#45;state"></a><strong>old state</strong></strong>

A string which represents the <a id="the&#45;toggleevent&#45;interface&#58;toggle&#45;task&#45;task"></a>[task](#toggle-task-task)&#39;s event&#39;s value for the <a id="the&#45;toggleevent&#45;interface&#58;dom&#45;toggleevent&#45;oldstate&#45;2"></a><code>[oldState](#dom-toggleevent-oldstate)</code> attribute&#46;

## <a id="ipr"></a>Intellectual property rights[](#ipr)

The image in the introduction is based on [a photo](https://www.flickr.com/photos/wonderlane/2986252088/) by [Wonderlane](https://www.flickr.com/photos/wonderlane/)&#46; &#40;[CC BY 2&#46;0](https://creativecommons.org/licenses/by/2.0/)&#41;

The image of the wolf in the embedded content introduction is based on [a photo](https://commons.wikimedia.org/wiki/File:WolfRunningInSnow.jpg) by [Barry O&#39;Neill](https://commons.wikimedia.org/wiki/File:WolfRunningInSnow.jpg)&#46; &#40;[Public domain](https://en.wikipedia.org/wiki/Public_domain)&#41;

The image of the kettlebell swing in the embedded content introduction is based on [a photo](https://pixabay.com/en/functional-mobility-articular-606568/) by [kokkarina](https://pixabay.com/en/users/506563-506563/)&#46; &#40;[CC0 1&#46;0](https://creativecommons.org/publicdomain/zero/1.0/)&#41;

The Blue Robot Player sprite used in the canvas demo is based on [a work](https://johncolburn.deviantart.com/art/Blue-Robot-Player-Sprite-323813997) by [JohnColburn](https://johncolburn.deviantart.com/)&#46; &#40;[CC BY&#45;SA 3&#46;0](https://creativecommons.org/licenses/by-sa/3.0/)&#41;

The photograph of robot 148 climbing the tower at the FIRST Robotics Competition 2013 Silicon Valley Regional is based on [a work](https://www.flickr.com/photos/lenore-m/8631391979/) by [Lenore Edman](https://www.flickr.com/photos/lenore-m/)&#46; &#40;[CC BY 2&#46;0](https://creativecommons.org/licenses/by/2.0/)&#41;

The diagram showing how <a id="ipr&#58;attr&#45;script&#45;async"></a><code>[async](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-script-async)</code> and <a id="ipr&#58;attr&#45;script&#45;defer"></a><code>[defer](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-script-defer)</code> impact <a id="ipr&#58;the&#45;script&#45;element"></a><code>[script](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-script-element)</code> loading is based on a similar diagram from [a blog post](https://peter.sh/experiments/asynchronous-and-deferred-javascript-execution-explained/) by [Peter Beverloo](https://peter.sh/about/)&#46; &#40;[CC0 1&#46;0](https://creativecommons.org/publicdomain/zero/1.0/)&#41;

The image decoding demo used to demonstrate module&#45;based workers draws on some example code from [a tutorial](https://www.html5rocks.com/en/tutorials/canvas/imagefilters/) by [Ilmari Heikkinen](https://www.fhtr.net/)&#46; &#40;[CC BY 3&#46;0](https://creativecommons.org/licenses/by/3.0/)&#41;

The <code>&#60;flag&#45;icon&#62;</code> example was inspired by [a custom element](https://github.com/stevenrskelton/flag-icon) by [Steven Skelton](http://stevenskelton.ca/)&#46; &#40;[MIT](https://opensource.org/licenses/MIT)&#41;

Part of the revision history of the <a id="ipr&#58;the&#45;picture&#45;element"></a><code>[picture](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-picture-element)</code> element and related features can be found in the [<code>ResponsiveImagesCG&#47;picture&#45;element</code> repository](https://github.com/ResponsiveImagesCG/picture-element)&#44; which is available under the [W3C Software and Document License](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document)&#46;

Part of the revision history of the <a id="ipr&#58;meta&#45;theme&#45;color"></a><code>[theme&#45;color](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#meta-theme-color)</code> metadata name can be found in the [<code>whatwg&#47;meta&#45;theme&#45;color</code> repository](https://github.com/whatwg/meta-theme-color)&#44; which is available under [CC0](https://creativecommons.org/publicdomain/zero/1.0/)&#46;

Part of the revision history of the [custom elements](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#custom-elements) feature can be found in the [<code>w3c&#47;webcomponents</code> repository](https://github.com/w3c/webcomponents)&#44; which is available under the [W3C Software and Document License](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document)&#46;

Part of the revision history of the <a id="ipr&#58;dom&#45;innertext"></a><code>[innerText](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-innertext)</code> getter and setter can be found in the [<code>rocallahan&#47;innerText&#45;spec</code> repository](https://github.com/rocallahan/innerText-spec)&#44; which is available under [CC0](https://creativecommons.org/publicdomain/zero/1.0/)&#46;

Part of the revision history of the [worklets](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#worklets) feature can be found in the [<code>w3c&#47;css&#45;houdini&#45;drafts</code> repository](https://github.com/w3c/css-houdini-drafts)&#44; which is available under the [W3C Software and Document License](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document)&#46;

Part of the revision history of the [import maps](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#import-maps) feature can be found in the [<code>WICG&#47;import&#45;maps</code> repository](https://github.com/WICG/import-maps)&#44; which is available under the [W3C Software and Document License](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document)&#46;

Part of the revision history of the [navigation API](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-api) feature can be found in the [<code>WICG&#47;navigation&#45;api</code> repository](https://github.com/WICG/navigation-api)&#44; which is available under the [W3C Software and Document License](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document)&#46;

Part of the revision history of the [Close requests and close watchers](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#close-requests-and-close-watchers) section can be found in the [<code>WICG&#47;close&#45;watcher</code> repository](https://github.com/WICG/close-watcher)&#44; which is available under the [W3C Software and Document License](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document)&#46;

Part of the revision history of the [Speculative loading](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#speculative-loading) section can be found in the [<code>WICG&#47;nav&#45;speculation</code> repository](https://github.com/WICG/nav-speculation)&#44; which is available under the [W3C Software and Document License](https://www.w3.org/Consortium/Legal/2015/copyright-software-and-document)&#46;

Copyright © WHATWG &#40;Apple&#44; Google&#44; Mozilla&#44; Microsoft&#41;&#46; This work is licensed under a [Creative Commons Attribution 4&#46;0 International License](https://creativecommons.org/licenses/by/4.0/)&#46; To the extent portions of it are incorporated into source code&#44; such portions in the source code are licensed under the [BSD 3&#45;Clause License](https://opensource.org/licenses/BSD-3-Clause) instead&#46;

This is the Living Standard&#46; Those interested in the patent&#45;review version should view the [Living Standard Review Draft](https://html.spec.whatwg.org/review-drafts/2026-07/)&#46;
