# DOM: mutation observer queueing and notification source

Bounded Markdown conversion of [DOM §4.3 Mutation observers](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#mutation-observers), immutable revision `3071e5f269448350fdea79bb284eedaf17bc431e`. The retained complete original HTML is `tmp/work/dom-style-contracts/dom/dom-full.html`, 3,180,470 bytes, SHA-256 `9221bf5c72d0399e11749f8c8d5c01b60a45fa045076b02935ac298119ea7be1`; its one-based line 404 identifies that commit (the document title also identifies it at line 7). The source header reads “Commit Snapshot — Last Updated 9 October 2026”. This is the already retained immutable source, extracted offline on 15 October 2026; no new retrieval was performed.

Selected original one-based lines **5604–5698 inclusive**, including the closing algorithm wrapper: the §4.3 heading, three agent-state definitions, complete **queue a mutation observer microtask** algorithm and complete **notify mutation observers** algorithm. Exact selected HTML SHA-256: `937c2b34f9b80a37593a0b8f1b8e84626cc5cfee1fc4aa3b57f555899a63f1c1` (8,077 bytes). Coverage stops before the horizontal rule and registered-observer definitions at lines 5699–5700; it does not include all of §4.3 or its subsections. Definitions outside this range remain linked to their original sources.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose is [Creative Commons Attribution 4.0 International](https://creativecommons.org/licenses/by/4.0/); the unchanged [WHATWG LICENSE](../licenses/whatwg/LICENSE) and [existing attribution](../licenses/whatwg/NOTICE.md#dom-html-web-idl-and-infra-bounded-captures) retain the original terms. Conversion and attribution by Surgeist. No endorsement or implementation-conformance claim is implied.

Conversion uses the existing repository HTML preparation and semantic Lua filter with installed Pandoc 3.1.11.1. Layout whitespace is folded; original prose, ordering, nested algorithm lists, variables, inline code, definition anchors and link identities are retained. Source-only presentation attributes are omitted. Fragment links stay local only for emitted anchors; other DOM fragments resolve to this immutable snapshot, and external links remain external. No scripts are executed. This is a partial source-format conversion, not a rewritten policy, replacement specification or selection of a different edition.

---

<!-- captured-body-start -->

### <a id="mutation-observers"></a>4.3. Mutation observers[](#mutation-observers)

Each <a id="ref-for-similar-origin-window-agent①"></a>[similar-origin window agent](https://html.spec.whatwg.org/multipage/webappapis.html#similar-origin-window-agent) has a <a id="mutation-observer-compound-microtask-queued-flag"></a><strong>mutation observer microtask queued</strong> (a boolean), which is initially false. [\[HTML\]](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#biblio-html)

Each <a id="ref-for-similar-origin-window-agent②"></a>[similar-origin window agent](https://html.spec.whatwg.org/multipage/webappapis.html#similar-origin-window-agent) also has <a id="mutation-observer-list"></a><strong>pending mutation observers</strong> (a <a id="ref-for-ordered-set⑥"></a>[set](https://infra.spec.whatwg.org/#ordered-set) of zero or more <code><a id="ref-for-mutationobserver"></a>[MutationObserver](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#mutationobserver)</code> objects), which is initially empty.

Each <a id="ref-for-similar-origin-window-agent③"></a>[similar-origin window agent](https://html.spec.whatwg.org/multipage/webappapis.html#similar-origin-window-agent) also has a <a id="mutation-observer-counter"></a><strong>mutation observer counter</strong> (a number), which is initially 0.

To <a id="queue-a-mutation-observer-compound-microtask"></a><strong>queue a mutation observer microtask</strong>:

1.  If the <a id="ref-for-surrounding-agent"></a>[surrounding agent](https://tc39.es/ecma262/#surrounding-agent)’s <a id="ref-for-mutation-observer-compound-microtask-queued-flag"></a>[mutation observer microtask queued](#mutation-observer-compound-microtask-queued-flag) is true, then return.

2.  Set the <a id="ref-for-surrounding-agent①"></a>[surrounding agent](https://tc39.es/ecma262/#surrounding-agent)’s <a id="ref-for-mutation-observer-compound-microtask-queued-flag①"></a>[mutation observer microtask queued](#mutation-observer-compound-microtask-queued-flag) to true.

3.  <a id="ref-for-queue-a-microtask"></a>[Queue](https://html.spec.whatwg.org/multipage/webappapis.html#queue-a-microtask) a <a id="ref-for-microtask"></a>[microtask](https://html.spec.whatwg.org/multipage/webappapis.html#microtask) to <a id="ref-for-notify-mutation-observers"></a>[notify mutation observers](#notify-mutation-observers).

To <a id="notify-mutation-observers"></a><strong>notify mutation observers</strong>:

1.  Set the <a id="ref-for-surrounding-agent②"></a>[surrounding agent](https://tc39.es/ecma262/#surrounding-agent)’s <a id="ref-for-mutation-observer-compound-microtask-queued-flag②"></a>[mutation observer microtask queued](#mutation-observer-compound-microtask-queued-flag) to false.

2.  Let <var>notifyList</var> be the result of <a id="ref-for-list-sort-in-ascending-order"></a>[sorting](https://infra.spec.whatwg.org/#list-sort-in-ascending-order) the <a id="ref-for-surrounding-agent③"></a>[surrounding agent](https://tc39.es/ecma262/#surrounding-agent)’s <a id="ref-for-mutation-observer-list"></a>[pending mutation observers](#mutation-observer-list) in ascending order, with <var>a</var> being less than <var>b</var> if <var>a</var>’s <a id="ref-for-mutationobserver-creation-index"></a>[creation index](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#mutationobserver-creation-index) is less than <var>b</var>’s <a id="ref-for-mutationobserver-creation-index①"></a>[creation index](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#mutationobserver-creation-index).

3.  <a id="ref-for-list-empty①"></a>[Empty](https://infra.spec.whatwg.org/#list-empty) the <a id="ref-for-surrounding-agent④"></a>[surrounding agent](https://tc39.es/ecma262/#surrounding-agent)’s <a id="ref-for-mutation-observer-list①"></a>[pending mutation observers](#mutation-observer-list).

4.  Let <var>signalSet</var> be a <a id="ref-for-list-clone①"></a>[clone](https://infra.spec.whatwg.org/#list-clone) of the <a id="ref-for-surrounding-agent⑤"></a>[surrounding agent](https://tc39.es/ecma262/#surrounding-agent)’s <a id="ref-for-signal-slot-list①"></a>[signal slots](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#signal-slot-list).

5.  <a id="ref-for-list-empty②"></a>[Empty](https://infra.spec.whatwg.org/#list-empty) the <a id="ref-for-surrounding-agent⑥"></a>[surrounding agent](https://tc39.es/ecma262/#surrounding-agent)’s <a id="ref-for-signal-slot-list②"></a>[signal slots](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#signal-slot-list).

6.  <a id="ref-for-list-iterate①⑥"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>mo</var> of <var>notifyList</var>:

    1.  Let <var>records</var> be a <a id="ref-for-list-clone②"></a>[clone](https://infra.spec.whatwg.org/#list-clone) of <var>mo</var>’s <a id="ref-for-concept-mo-queue"></a>[record queue](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#concept-mo-queue).

    2.  <a id="ref-for-list-empty③"></a>[Empty](https://infra.spec.whatwg.org/#list-empty) <var>mo</var>’s <a id="ref-for-concept-mo-queue①"></a>[record queue](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#concept-mo-queue).

    3.  <a id="ref-for-remove-transient-registered-observers"></a>[Remove transient registered observers](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#remove-transient-registered-observers) for <var>mo</var>.

    4.  If <var>records</var> <a id="ref-for-list-is-empty④"></a>[is not empty](https://infra.spec.whatwg.org/#list-is-empty), then <a id="ref-for-invoke-a-callback-function"></a>[invoke](https://webidl.spec.whatwg.org/#invoke-a-callback-function) <var>mo</var>’s <a id="ref-for-concept-mo-callback"></a>[callback](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#concept-mo-callback) with « <var>records</var>, <var>mo</var> » and "<code>report</code>", and with <a id="ref-for-dfn-callback-this-value"></a>[callback this value](https://webidl.spec.whatwg.org/#dfn-callback-this-value) <var>mo</var>.

7.  <a id="ref-for-list-iterate①⑦"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>slot</var> of <var>signalSet</var>: <a id="ref-for-concept-event-fire⑧"></a>[fire an event](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#concept-event-fire) named <a id="eventdef-htmlslotelement-slotchange"></a><strong><code>slotchange</code></strong>, with its <code><a id="ref-for-dom-event-bubbles⑧"></a>[bubbles](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#dom-event-bubbles)</code> attribute set to true, at <var>slot</var>.
