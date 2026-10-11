# HTML: sandboxing flag definitions and directive parsing

Source: [immutable HTML snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxing), revision `877d6146487f15b15bc8dd0e3c625063a37b456a`; complete original HTML SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`. Retrieved 2026-10-11. Complete §7.1.5, original lines 65599–65796, ending immediately before §7.1.6 at line 65797. Includes all sixteen individual flag definitions, the complete three-step directive algorithm with sixteen flag entries at lines 65688–65748 (defining paragraph and steps at 65689–65746), and the popup, iframe, active, CSP-derived and creation flag-set clauses. Surrounding policy/navigation algorithms remain linked to the same selected edition.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose: [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/); [WHATWG license](../licenses/whatwg/LICENSE). Bounded extraction and Markdown conversion by Surgeist, 2026-10-11 using Pandoc 3.1.11.1. Links are absolute; inert wrappers preserve code/paragraph IDs and informative-note classes, and the original semantic heading is retained. No scripts executed, semantic changes or endorsement. This representation supplies source evidence; it does not claim browser sandbox enforcement, navigation, CSP evaluation or runtime composition.

<div>

<h4 id="sandboxing"><span class="secno">7.1.5</span> Sandboxing<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxing" class="self-link"></a></h4>

A <span id="sandboxing-flag-set" class="dfn" export="">sandboxing flag set</span> is a set of zero or more of the following flags, which are used to restrict the abilities that potentially untrusted resources have:

The <span id="sandboxed-navigation-browsing-context-flag" class="dfn" export="">sandboxed navigation browsing context flag</span>\
This flag [prevents content from navigating browsing contexts other than the sandboxed browsing context itself](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxLinks) (or browsing contexts further nested inside it), <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#auxiliary-browsing-context" id="sandboxing:auxiliary-browsing-context">auxiliary browsing contexts</a> (which are protected by the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-auxiliary-navigation-browsing-context-flag" id="sandboxing:sandboxed-auxiliary-navigation-browsing-context-flag">sandboxed auxiliary navigation browsing context flag</a> defined next), and the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-browsing-context" id="sandboxing:top-level-browsing-context">top-level browsing context</a> (which is protected by the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-top-level-navigation-without-user-activation-browsing-context-flag" id="sandboxing:sandboxed-top-level-navigation-without-user-activation-browsing-context-flag">sandboxed top-level navigation without user activation browsing context flag</a> and <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-top-level-navigation-with-user-activation-browsing-context-flag" id="sandboxing:sandboxed-top-level-navigation-with-user-activation-browsing-context-flag">sandboxed top-level navigation with user activation browsing context flag</a> defined below).

If the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-auxiliary-navigation-browsing-context-flag" id="sandboxing:sandboxed-auxiliary-navigation-browsing-context-flag-2">sandboxed auxiliary navigation browsing context flag</a> is not set, then in certain cases the restrictions nonetheless allow popups (new <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-browsing-context" id="sandboxing:top-level-browsing-context-2">top-level browsing contexts</a>) to be opened. These <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id="sandboxing:browsing-context">browsing contexts</a> always have <span id="one-permitted-sandboxed-navigator" class="dfn">one permitted sandboxed navigator</span>, set when the browsing context is created, which allows the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id="sandboxing:browsing-context-2">browsing context</a> that created them to actually navigate them. (Otherwise, the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-navigation-browsing-context-flag" id="sandboxing:sandboxed-navigation-browsing-context-flag">sandboxed navigation browsing context flag</a> would prevent them from being navigated even if they were opened.)

The <span id="sandboxed-auxiliary-navigation-browsing-context-flag" class="dfn" export="">sandboxed auxiliary navigation browsing context flag</span>\
This flag [prevents content from creating new auxiliary browsing contexts](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxWindowOpen), e.g. using the <span id="sandboxing:attr-hyperlink-target">[`target`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-hyperlink-target)</span> attribute or the <span id="sandboxing:dom-open">[`window.open()`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-open)</span> method.

The <span id="sandboxed-top-level-navigation-without-user-activation-browsing-context-flag" class="dfn" export="">sandboxed top-level navigation without user activation browsing context flag</span>\
This flag [prevents content from navigating their top-level browsing context](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxLinks) and [prevents content from closing their top-level browsing context](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxClose). It is consulted only when the sandboxed browsing context's <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#active-window" id="sandboxing:active-window">active window</a> does not have <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#transient-activation" id="sandboxing:transient-activation">transient activation</a>.

When the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-top-level-navigation-without-user-activation-browsing-context-flag" id="sandboxing:sandboxed-top-level-navigation-without-user-activation-browsing-context-flag-2">sandboxed top-level navigation without user activation browsing context flag</a> is *not* set, content can navigate its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-browsing-context" id="sandboxing:top-level-browsing-context-3">top-level browsing context</a>, but other <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id="sandboxing:browsing-context-3">browsing contexts</a> are still protected by the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-navigation-browsing-context-flag" id="sandboxing:sandboxed-navigation-browsing-context-flag-2">sandboxed navigation browsing context flag</a> and possibly the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-auxiliary-navigation-browsing-context-flag" id="sandboxing:sandboxed-auxiliary-navigation-browsing-context-flag-3">sandboxed auxiliary navigation browsing context flag</a>.

The <span id="sandboxed-top-level-navigation-with-user-activation-browsing-context-flag" class="dfn" export="">sandboxed top-level navigation with user activation browsing context flag</span>\
This flag [prevents content from navigating their top-level browsing context](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxLinks) and [prevents content from closing their top-level browsing context](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxClose). It is consulted only when the sandboxed browsing context's <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#active-window" id="sandboxing:active-window-2">active window</a> has <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#transient-activation" id="sandboxing:transient-activation-2">transient activation</a>.

As with the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-top-level-navigation-without-user-activation-browsing-context-flag" id="sandboxing:sandboxed-top-level-navigation-without-user-activation-browsing-context-flag-3">sandboxed top-level navigation without user activation browsing context flag</a>, this flag only affects the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-browsing-context" id="sandboxing:top-level-browsing-context-4">top-level browsing context</a>; if it is not set, other <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id="sandboxing:browsing-context-4">browsing contexts</a> might still be protected by other flags.

The <span id="sandboxed-origin-browsing-context-flag" class="dfn" export="">sandboxed origin browsing context flag</span>\
This flag [forces content into an opaque origin](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxOrigin), thus preventing it from accessing other content from the same <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin" id="sandboxing:concept-origin">origin</a>.

This flag also [prevents script from reading from or writing to the `document.cookie` IDL attribute](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxCookies), and blocks access to <span id="sandboxing:dom-localstorage">[`localStorage`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-localstorage)</span>.

The <span id="sandboxed-forms-browsing-context-flag" class="dfn" export="">sandboxed forms browsing context flag</span>\
This flag [blocks form submission](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxSubmitBlocked).

The <span id="sandboxed-pointer-lock-browsing-context-flag" class="dfn" export="">sandboxed pointer lock browsing context flag</span>\
This flag disables the Pointer Lock API. [\[POINTERLOCK\]](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsPOINTERLOCK)

The <span id="sandboxed-scripts-browsing-context-flag" class="dfn" export="">sandboxed scripts browsing context flag</span>\
This flag [blocks script execution](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxScriptBlocked).

The <span id="sandboxed-automatic-features-browsing-context-flag" class="dfn" export="">sandboxed automatic features browsing context flag</span>\
This flag blocks features that trigger automatically, such as <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-media-autoplay" id="sandboxing:attr-media-autoplay">automatically playing a video</a> or <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-fe-autofocus" id="sandboxing:attr-fe-autofocus">automatically focusing a form control</a>.

The <span id="sandboxed-document.domain-browsing-context-flag" class="dfn" export="">sandboxed `document.domain` browsing context flag</span>\
This flag prevents content from using the <span id="sandboxing:dom-document-domain">[`document.domain`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-document-domain)</span> setter.

The <span id="sandbox-propagates-to-auxiliary-browsing-contexts-flag" class="dfn" export="">sandbox propagates to auxiliary browsing contexts flag</span>\
This flag prevents content from escaping the sandbox by ensuring that any <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#auxiliary-browsing-context" id="sandboxing:auxiliary-browsing-context-2">auxiliary browsing context</a> it creates inherits the content's <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#active-sandboxing-flag-set" id="sandboxing:active-sandboxing-flag-set">active sandboxing flag set</a>.

The <span id="sandboxed-modals-flag" class="dfn" export="">sandboxed modals flag</span>\
This flag prevents content from using any of the following features to produce modal dialogs:

- <span id="sandboxing:dom-alert">[`window.alert()`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-alert)</span>
- <span id="sandboxing:dom-confirm">[`window.confirm()`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-confirm)</span>
- <span id="sandboxing:dom-print">[`window.print()`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-print)</span>
- <span id="sandboxing:dom-prompt">[`window.prompt()`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-prompt)</span>
- the <span id="sandboxing:event-beforeunload">[`beforeunload`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-beforeunload)</span> event

The <span id="sandboxed-orientation-lock-browsing-context-flag" class="dfn" export="">sandboxed orientation lock browsing context flag</span>\
This flag disables the ability to lock the screen orientation. [\[SCREENORIENTATION\]](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsSCREENORIENTATION)

The <span id="sandboxed-presentation-browsing-context-flag" class="dfn" export="">sandboxed presentation browsing context flag</span>\
This flag disables the Presentation API. [\[PRESENTATION\]](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsPRESENTATION)

The <span id="sandboxed-downloads-browsing-context-flag" class="dfn" export="">sandboxed downloads browsing context flag</span>\
This flag prevents content from initiating or instantiating downloads, whether through <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#downloading-hyperlinks" id="sandboxing:downloading-hyperlinks">downloading hyperlinks</a> or through [navigation](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-as-a-download) that gets <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#handle-as-a-download" id="sandboxing:handle-as-a-download">handled as a download</a>.

The <span id="sandboxed-custom-protocols-navigation-browsing-context-flag" class="dfn" export="">sandboxed custom protocols navigation browsing context flag</span>\
This flag prevents navigations toward non <a href="https://fetch.spec.whatwg.org/#fetch-scheme" id="sandboxing:fetch-scheme" data-x-internal="fetch-scheme">fetch schemes</a> from being <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#hand-off-to-external-software" id="sandboxing:hand-off-to-external-software">handed off to external software</a>.

<div algorithm="">

When the user agent is to <span id="parse-a-sandboxing-directive" class="dfn" export="">parse a sandboxing directive</span>, given a string `input` and a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxing-flag-set" id="sandboxing:sandboxing-flag-set">sandboxing flag set</a> `output`, it must run the following steps:

1.  <a href="https://infra.spec.whatwg.org/#split-on-ascii-whitespace" id="sandboxing:split-a-string-on-spaces" data-x-internal="split-a-string-on-spaces">Split <var>input</var> on ASCII whitespace</a>, to obtain `tokens`.

2.  Let `output` be empty.

3.  Add the following flags to `output`:

    - The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-navigation-browsing-context-flag" id="sandboxing:sandboxed-navigation-browsing-context-flag-3">sandboxed navigation browsing context flag</a>.

    - The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-auxiliary-navigation-browsing-context-flag" id="sandboxing:sandboxed-auxiliary-navigation-browsing-context-flag-4">sandboxed auxiliary navigation browsing context flag</a>, unless `tokens` contains the <span id="attr-iframe-sandbox-allow-popups" class="dfn" dfn-for="iframe/sandbox" dfn-type="attr-value">`allow-popups`</span> keyword.

    - The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-top-level-navigation-without-user-activation-browsing-context-flag" id="sandboxing:sandboxed-top-level-navigation-without-user-activation-browsing-context-flag-4">sandboxed top-level navigation without user activation browsing context flag</a>, unless `tokens` contains the <span id="attr-iframe-sandbox-allow-top-navigation" class="dfn" dfn-for="iframe/sandbox" dfn-type="attr-value">`allow-top-navigation`</span> keyword.

    - The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-top-level-navigation-with-user-activation-browsing-context-flag" id="sandboxing:sandboxed-top-level-navigation-with-user-activation-browsing-context-flag-2">sandboxed top-level navigation with user activation browsing context flag</a>, unless `tokens` contains either the <span id="attr-iframe-sandbox-allow-top-navigation-by-user-activation" class="dfn" dfn-for="iframe/sandbox" dfn-type="attr-value">`allow-top-navigation-by-user-activation`</span> keyword or the <span id="sandboxing:attr-iframe-sandbox-allow-top-navigation">[`allow-top-navigation`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-top-navigation)</span> keyword.

      <div class="note">

      This means that if the <span id="sandboxing:attr-iframe-sandbox-allow-top-navigation-2">[`allow-top-navigation`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-top-navigation)</span> is present, the <span id="sandboxing:attr-iframe-sandbox-allow-top-navigation-by-user-activation">[`allow-top-navigation-by-user-activation`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-top-navigation-by-user-activation)</span> keyword will have no effect. For this reason, specifying both is a document conformance error.

      </div>

    - The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-origin-browsing-context-flag" id="sandboxing:sandboxed-origin-browsing-context-flag">sandboxed origin browsing context flag</a>, unless the `tokens` contains the <span id="attr-iframe-sandbox-allow-same-origin" class="dfn" dfn-for="iframe/sandbox" dfn-type="attr-value">`allow-same-origin`</span> keyword.

      <div class="note">

      The <span id="sandboxing:attr-iframe-sandbox-allow-same-origin">[`allow-same-origin`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-same-origin)</span> keyword is intended for two cases.

      First, it can be used to allow content from the same site to be sandboxed to disable scripting, while still allowing access to the DOM of the sandboxed content.

      Second, it can be used to embed content from a third-party site, sandboxed to prevent that site from opening popups, etc, without preventing the embedded page from communicating back to its originating site, using the database APIs to store data, etc.

      </div>

    - The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-forms-browsing-context-flag" id="sandboxing:sandboxed-forms-browsing-context-flag">sandboxed forms browsing context flag</a>, unless `tokens` contains the <span id="attr-iframe-sandbox-allow-forms" class="dfn" dfn-for="iframe/sandbox" dfn-type="attr-value">`allow-forms`</span> keyword.

    - The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-pointer-lock-browsing-context-flag" id="sandboxing:sandboxed-pointer-lock-browsing-context-flag">sandboxed pointer lock browsing context flag</a>, unless `tokens` contains the <span id="attr-iframe-sandbox-allow-pointer-lock" class="dfn" dfn-for="iframe/sandbox" dfn-type="attr-value">`allow-pointer-lock`</span> keyword.

    - The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-scripts-browsing-context-flag" id="sandboxing:sandboxed-scripts-browsing-context-flag">sandboxed scripts browsing context flag</a>, unless `tokens` contains the <span id="attr-iframe-sandbox-allow-scripts" class="dfn" dfn-for="iframe/sandbox" dfn-type="attr-value">`allow-scripts`</span> keyword.

    - The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-automatic-features-browsing-context-flag" id="sandboxing:sandboxed-automatic-features-browsing-context-flag">sandboxed automatic features browsing context flag</a>, unless `tokens` contains the <span id="sandboxing:attr-iframe-sandbox-allow-scripts">[`allow-scripts`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-scripts)</span> keyword (defined above).

      <div class="note">

      This flag is relaxed by the same keyword as scripts, because when scripts are enabled these features are trivially possible anyway, and it would be unfortunate to force authors to use script to do them when sandboxed rather than allowing them to use the declarative features.

      </div>

    - The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-document.domain-browsing-context-flag" id="sandboxing:sandboxed-document.domain-browsing-context-flag">sandboxed <code>document.domain</code> browsing context flag</a>.

    - The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandbox-propagates-to-auxiliary-browsing-contexts-flag" id="sandboxing:sandbox-propagates-to-auxiliary-browsing-contexts-flag">sandbox propagates to auxiliary browsing contexts flag</a>, unless `tokens` contains the <span id="attr-iframe-sandbox-allow-popups-to-escape-sandbox" class="dfn" dfn-for="iframe/sandbox" dfn-type="attr-value">`allow-popups-to-escape-sandbox`</span> keyword.

    - The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-modals-flag" id="sandboxing:sandboxed-modals-flag">sandboxed modals flag</a>, unless `tokens` contains the <span id="attr-iframe-sandbox-allow-modals" class="dfn" dfn-for="iframe/sandbox" dfn-type="attr-value">`allow-modals`</span> keyword.

    - The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-orientation-lock-browsing-context-flag" id="sandboxing:sandboxed-orientation-lock-browsing-context-flag">sandboxed orientation lock browsing context flag</a>, unless `tokens` contains the <span id="attr-iframe-sandbox-allow-orientation-lock" class="dfn" dfn-for="iframe/sandbox" dfn-type="attr-value">`allow-orientation-lock`</span> keyword.

    - The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-presentation-browsing-context-flag" id="sandboxing:sandboxed-presentation-browsing-context-flag">sandboxed presentation browsing context flag</a>, unless `tokens` contains the <span id="attr-iframe-sandbox-allow-presentation" class="dfn" dfn-for="iframe/sandbox" dfn-type="attr-value">`allow-presentation`</span> keyword.

    - The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-downloads-browsing-context-flag" id="sandboxing:sandboxed-downloads-browsing-context-flag">sandboxed downloads browsing context flag</a>, unless `tokens` contains the <span id="attr-iframe-sandbox-allow-downloads" class="dfn" dfn-for="iframe/sandbox" dfn-type="attr-value">`allow-downloads`</span> keyword.

    - The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxed-custom-protocols-navigation-browsing-context-flag" id="sandboxing:sandboxed-custom-protocols-navigation-browsing-context-flag">sandboxed custom protocols navigation browsing context flag</a>, unless `tokens` contains either the <span id="attr-iframe-sandbox-allow-top-navigation-to-custom-protocols" class="dfn" dfn-for="iframe/sandbox" dfn-type="attr-value">`allow-top-navigation-to-custom-protocols`</span> keyword, the <span id="sandboxing:attr-iframe-sandbox-allow-popups">[`allow-popups`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-popups)</span> keyword, or the <span id="sandboxing:attr-iframe-sandbox-allow-top-navigation-3">[`allow-top-navigation`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox-allow-top-navigation)</span> keyword.

</div>

------------------------------------------------------------------------

Every <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-browsing-context" id="sandboxing:top-level-browsing-context-5">top-level browsing context</a> has a <span id="popup-sandboxing-flag-set" class="dfn">popup sandboxing flag set</span>, which is a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxing-flag-set" id="sandboxing:sandboxing-flag-set-2">sandboxing flag set</a>. When a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id="sandboxing:browsing-context-5">browsing context</a> is created, its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#popup-sandboxing-flag-set" id="sandboxing:popup-sandboxing-flag-set">popup sandboxing flag set</a> must be empty. It is populated by <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-rules-for-choosing-a-navigable" id="sandboxing:the-rules-for-choosing-a-navigable">the rules for choosing a navigable</a> and the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#obtain-browsing-context-navigation" id="sandboxing:obtain-browsing-context-navigation">obtain a browsing context to use for a navigation response</a> algorithm.

Every <span id="sandboxing:the-iframe-element">[`iframe`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-iframe-element)</span> element has an <span id="iframe-sandboxing-flag-set" class="dfn">`iframe` sandboxing flag set</span>, which is a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxing-flag-set" id="sandboxing:sandboxing-flag-set-3">sandboxing flag set</a>. Which flags in an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#iframe-sandboxing-flag-set" id="sandboxing:iframe-sandboxing-flag-set"><code>iframe</code> sandboxing flag set</a> are set at any particular time is determined by the <span id="sandboxing:the-iframe-element-2">[`iframe`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-iframe-element)</span> element's <span id="sandboxing:attr-iframe-sandbox">[`sandbox`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-iframe-sandbox)</span> attribute.

Every <span id="sandboxing:document">[`Document`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document)</span> has an <span id="active-sandboxing-flag-set" class="dfn" dfn-for="Document" export="">active sandboxing flag set</span>, which is a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxing-flag-set" id="sandboxing:sandboxing-flag-set-4">sandboxing flag set</a>. When the <span id="sandboxing:document-2">[`Document`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document)</span> is created, its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#active-sandboxing-flag-set" id="sandboxing:active-sandboxing-flag-set-2">active sandboxing flag set</a> must be empty. It is populated by the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigate" id="sandboxing:navigate">navigation algorithm</a>.

<div algorithm="">

<div id="forced-sandboxing-flag-set">

Every <a href="https://w3c.github.io/webappsec-csp/#csp-list" id="sandboxing:concept-csp-list" data-x-internal="concept-csp-list">CSP list</a> `cspList` has <span id="csp-derived-sandboxing-flags" class="dfn">CSP-derived sandboxing flags</span>, which is a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxing-flag-set" id="sandboxing:sandboxing-flag-set-5">sandboxing flag set</a>. It is the return value of the following algorithm:

</div>

1.  Let `directives` be an empty <a href="https://infra.spec.whatwg.org/#ordered-set" id="sandboxing:set" data-x-internal="set">ordered set</a>.

2.  <a href="https://infra.spec.whatwg.org/#list-iterate" id="sandboxing:list-iterate" data-x-internal="list-iterate">For each</a> policy in `cspList`:

    1.  If `policy`'s <a href="https://w3c.github.io/webappsec-csp/#policy-disposition" id="sandboxing:csp-disposition" data-x-internal="csp-disposition">disposition</a> is not "`enforce`", then <a href="https://infra.spec.whatwg.org/#iteration-continue" id="sandboxing:continue" data-x-internal="continue">continue</a>.

    2.  If `policy`'s <a href="https://w3c.github.io/webappsec-csp/#policy-directive-set" id="sandboxing:csp-directive-set" data-x-internal="csp-directive-set">directive set</a> <a href="https://infra.spec.whatwg.org/#list-contain" id="sandboxing:list-contains" data-x-internal="list-contains">contains</a> a <a href="https://w3c.github.io/webappsec-csp/#directives" id="sandboxing:content-security-policy-directive" data-x-internal="content-security-policy-directive">directive</a> whose name is "<span id="sandboxing:sandbox-directive"><a href="https://w3c.github.io/webappsec-csp/#sandbox" data-x-internal="sandbox-directive"><code>sandbox</code></a></span>", then <a href="https://infra.spec.whatwg.org/#list-append" id="sandboxing:list-append" data-x-internal="list-append">append</a> that directive to `directives`.

3.  If `directives` is empty, then return an empty <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxing-flag-set" id="sandboxing:sandboxing-flag-set-6">sandboxing flag set</a>.

4.  Let `directive` be `directives`\[`directives`'s <a href="https://infra.spec.whatwg.org/#list-size" id="sandboxing:list-size" data-x-internal="list-size">size</a> − 1\].

5.  Return the result of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#parse-a-sandboxing-directive" id="sandboxing:parse-a-sandboxing-directive">parsing the sandboxing directive</a> `directive`.

</div>

------------------------------------------------------------------------

<div algorithm="">

To <span id="determining-the-creation-sandboxing-flags" class="dfn">determine the creation sandboxing flags</span> for a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id="sandboxing:concept-document-bc">browsing context</a> `browsing context`, given null or an element `embedder`, return the <a href="https://infra.spec.whatwg.org/#set-union" id="sandboxing:set-union" data-x-internal="set-union">union</a> of the flags that are present in the following <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#sandboxing-flag-set" id="sandboxing:sandboxing-flag-set-7">sandboxing flag sets</a>:

- If `embedder` is null, then the flags set on `browsing context`'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#popup-sandboxing-flag-set" id="sandboxing:popup-sandboxing-flag-set-2">popup sandboxing flag set</a>.

- If `embedder` is an element, then the flags set on `embedder`'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#iframe-sandboxing-flag-set" id="sandboxing:iframe-sandboxing-flag-set-2"><code>iframe</code> sandboxing flag set</a>.

- If `embedder` is an element, then the flags set on `embedder`'s <a href="https://dom.spec.whatwg.org/#concept-node-document" id="sandboxing:node-document" data-x-internal="node-document">node document</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#active-sandboxing-flag-set" id="sandboxing:active-sandboxing-flag-set-3">active sandboxing flag set</a>.

</div>

</div>
