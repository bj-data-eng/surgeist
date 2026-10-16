# HTML hosted navigable lifecycle clauses

Bounded source retention for the selected hosted lifecycle contribution.
This reference does not establish implementation of hosted behavior.

Source: [immutable WHATWG HTML snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/), revision
`877d6146487f15b15bc8dd0e3c625063a37b456a` (9 October 2026).
Extracted offline on 16 October 2026 from the already-retained complete HTML:
14,930,614 bytes, SHA-256
`877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
No new retrieval, edition, browser implementation or software acquisition is used.

The exact selected original fragments, in table order and joined by one extra LF,
have SHA-256 `3a96581558660a74c558d5a3ca46e360be9f4192f044a4eee52edbf800e72c14`.
The filename suffix identifies this bounded original input, not the final
Markdown file. Each range includes complete named definitions, algorithms and
their retained adjacent notes; it is not a transitive closure of linked services.

| Original raw lines | Complete retained unit | Original fragment SHA-256 |
| --- | --- | --- |
| 70833–70874 | Navigable fields and current versus active entries | `759f67ff54a220f2d266d01ec24c855a3ad5ec3ba8d80416fac05e0df2dc8868` |
| 70882–70943 | Active-context, global and node-navigable relations; initialization | `8a3bd7cc284b17ecc7bea72839f0ce3fb5028cdf9740edebfddb58a3ceccbbf5` |
| 70946–71022 | Traversable definitions and complete new top-level creation algorithm | `506c6fbefcf7e0be60c132f38ae125e89be7e066c957b11981ed297a3c73ef85` |
| 71041–71137 | Container relations and complete child creation algorithm | `a0c6d7e9d7557b583f51e456688be4ce85390f19f92f20fbd5b1e37511de7e51` |
| 71267–71304 | Complete child and top-level destruction algorithms | `50fddfd48dfb095a7ff665900db8fdfd34b3e2c020066705bc80f554085d3e0d` |
| 72024–72055 | Complete session-history-entry items and Document getter | `491b6692b1a0bddace08ea84cd2e7c536f460c46aef0baffd14f25764967daf1` |
| 72094–72169 | Complete Document-state items and retention notes | `667e625464b627a58ab2fead5cf0f36511ba39d9d366f2c27af1617481791b7f` |
| 72189–72198 | Complete nested-history definition and source issue note | `8d4a21c52d756f7453078f631987e6086754c3b774980f622881feb3460addc6` |
| 74892–74900 | Complete make-active synchronization algorithm | `01ad68f0c31818ba61b628089e0d3d2853e297aa59cb4cf6ff2e5dc41352b09e` |

Reuse the existing [active-Document, browsing-context and fully-active clauses](html-hosted-document-activity--877d6146487f--f8564ea4758e.md)
and [task/global/element association and runnability clauses](html-element-task-association--877d6146487f.md).
Their bodies are not duplicated here. In particular, the active-Document
algorithm at raw70876–70880 remains in the first linked reference; the adjacent
read-safety note at70882–70885 is retained below with that explicit antecedent.
These links describe source, not implementation capability or task completion.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose is
[Creative Commons Attribution 4.0 International](https://creativecommons.org/licenses/by/4.0/).
The existing [complete WHATWG license](../licenses/whatwg/LICENSE) and
[pinned HTML attribution](../licenses/whatwg/NOTICE.md#pinned-html-language-snapshot--2026-10-11)
apply. The license file SHA-256 is
`85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`.
The upstream BSD 3-Clause provision applies to portions incorporated into source
code, not as an alternative license for this retained specification prose.

Changes by Surgeist on 16 October 2026: bounded offline extraction, explanatory
headings/provenance, and absolute same-edition fragment links. Selected source
bodies remain literal semantic HTML in Markdown: wording, conditions, order,
IDs, definition/list boundaries, informative notes, examples and source issue
markers are preserved. Only `href` values beginning with `#` are rewritten to
the pinned snapshot, with normal quoted attribute spelling. No source scripts
are executed. Original normative/informative distinctions remain authoritative;
no endorsement or complete browser-conformance claim is implied.

## Selection limits and actual remaining source dependencies

This retains all branches of the selected new top-level and child creation and
destruction algorithms, including opener/storage, traversal-queue, origin,
Navigation API and WebDriver calls. Those calls are not erased or declared
fulfilled by a lifecycle label. The next `create a fresh top-level traversable`
algorithm (71024–71038) initiates navigation and is not selected here; the complete
`create a new top-level traversable` algorithm ends at71022.

The complete session-history-entry and Document-state definitions are retained
rather than replacing them with a fabricated Document-only entry. This does not
select execution of their serialization, navigation, cache or resource services.
The nested-history source issue note and the top-level destruction's unresolved
iteration-order marker are retained literally; this reference chooses no missing
upstream order or application policy.

The selected algorithms still call these actual external prerequisites:

- [Create a new browsing context and Document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#creating-a-new-browsing-context),
  whose complete algorithm is raw71653–71719 and establishes real origin, policy,
  environment/global and Document state. Its body is outside this excerpt.
- [Make active](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#make-active), retained completely below, synchronizes the
  browsing context, WindowProxy and relevant settings object. Retaining it does
  not implement Window/global services or the calling active-entry/history-step
  transition ([update document for history step application](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#update-document-for-history-step-application), beginning at74793). [Reactivate a Document](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#reactivate-a-document) is a separate
  algorithm beginning at74902 and is not selected here.
- [Destroy a Document and its descendants](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#destroy-a-document-and-its-descendants),
  complete raw75892–75923, queues real navigation/traversal work and invokes
  Document destruction. Its body, the underlying Document destruction steps,
  unload/close cancellation, and the linked history/Navigation API updates are
  outside this excerpt. Rust arena retirement remains the separately selected
  #136/#1124 policy; it is not interchangeable with these source algorithms.

No arbitrary active Boolean, native-to-hosted conversion, null task-Document
substitute, omitted callback or implied physical platform success is established
by this retention. The existing operative model and issue owners retain all
implementation/admission decisions.



## Navigable fields and current versus active entries

<!-- selected-source-fragment:70833-70874 -->
  <h4 id=navigables><span class=secno>7.3.1</span> Navigables<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigables" class=self-link></a></h4>

  <p>A <dfn id=navigable data-export="">navigable</dfn> presents a <code id=navigables:document><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> to the user via its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-active-history-entry" id=navigables:nav-active-history-entry>active session history entry</a>. Each navigable has:</p>

  <ul><li><p>An <dfn id=nav-id>id</dfn>, a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#new-unique-internal-value" id=navigables:new-unique-internal-value>new unique internal value</a>.<li><p id=parent-browsing-context>A <dfn data-dfn-for=navigable id=nav-parent data-export="">parent</dfn>,
   a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=navigables:navigable>navigable</a> or null.<li>
    <p id=current-entry>A <dfn id=nav-current-history-entry>current session history
    entry</dfn>, a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#session-history-entry" id=navigables:session-history-entry>session history entry</a>.</p>

    <p>This can only be modified within the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#tn-session-history-traversal-queue" id=navigables:tn-session-history-traversal-queue>session history traversal queue</a> of the
    parent <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#traversable-navigable" id=navigables:traversable-navigable>traversable navigable</a>.</p>
   <li>
    <p>An <dfn id=nav-active-history-entry>active session history entry</dfn>, a
    <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#session-history-entry" id=navigables:session-history-entry-2>session history entry</a>.</p>

    <p>This can only be modified from the event loop of the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-active-history-entry" id=navigables:nav-active-history-entry-2>active session history entry</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#she-document" id=navigables:she-document>document</a>.</p>
   <li>
    <p>An <dfn id=is-closing>is closing</dfn> boolean, initially false.</p>

    <p class=note>This is only ever set to true for <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable" id=navigables:top-level-traversable>top-level traversable navigables</a>.</p>
   <li>
    <p>An <dfn id=delaying-load-events-mode>is delaying <code>load</code>
    events</dfn> boolean, initially false.</p>

    <p class=note>This is only ever set to true in cases where the navigable's <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-parent" id=navigables:nav-parent>parent</a> is non-null.</p>
   <li>
    <p>An <dfn id=allowed-to-perform-a-navigation-or-history-update>allowed to perform a navigation or history update</dfn> algorithm, which is
    <a id=navigables:implementation-defined href=https://infra.spec.whatwg.org/#implementation-defined data-x-internal=implementation-defined>implementation-defined</a> and returns either <a id=navigables:allowed href=https://infra.spec.whatwg.org/#allowed data-x-internal=allowed>allowed</a> or
    <a id=navigables:blocked href=https://infra.spec.whatwg.org/#blocked data-x-internal=blocked>blocked</a>.</p>

    <p class=note>For example, this can return <a id=navigables:blocked-2 href=https://infra.spec.whatwg.org/#blocked data-x-internal=blocked>blocked</a> if invoked too many times
    times within a certain timespan.</p>
   </ul>

  <p>The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-current-history-entry" id=navigables:nav-current-history-entry>current session history entry</a> and the
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-active-history-entry" id=navigables:nav-active-history-entry-3>active session history entry</a> are usually the same,
  but they get out of sync when:</p>

  <ul><li><p>Synchronous navigations are performed. This causes the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-active-history-entry" id=navigables:nav-active-history-entry-4>active session history entry</a> to temporarily step ahead
   of the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-current-history-entry" id=navigables:nav-current-history-entry-2>current session history entry</a>.<li><p>A non-displayable, non-error response is received when <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#apply-the-history-step" id=navigables:apply-the-history-step>applying the history step</a>. This updates the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-current-history-entry" id=navigables:nav-current-history-entry-3>current session history entry</a> but leaves the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-active-history-entry" id=navigables:nav-active-history-entry-5>active session history entry</a> as-is.</ul>

  <hr>


## Active-context, global and node-navigable relations; initialization

<!-- selected-source-fragment:70882-70943 -->
  <p class=note>This can be safely read from within the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#tn-session-history-traversal-queue" id=navigables:tn-session-history-traversal-queue-2>session history traversal queue</a> of the
  navigable's <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-top" id=navigables:nav-top>top-level traversable</a>. Although a
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=navigables:navigable-3>navigable</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-active-history-entry" id=navigables:nav-active-history-entry-7>active history entry</a> can
  change synchronously, the new entry will always have the same <code id=navigables:document-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>.</p>

  <div data-algorithm="">
  <p>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=navigables:navigable-4>navigable</a>'s <dfn data-dfn-for=navigable id=nav-bc data-export="">active browsing
  context</dfn> is its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document" id=navigables:nav-document>active document</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id=navigables:concept-document-bc>browsing context</a>. If this <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=navigables:navigable-5>navigable</a> is a
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#traversable-navigable" id=navigables:traversable-navigable-2>traversable navigable</a>, then its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-bc" id=navigables:nav-bc>active browsing context</a>
  will be a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-browsing-context" id=navigables:top-level-browsing-context>top-level browsing context</a>.</p>
  </div>

  <div data-algorithm="">
  <p>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=navigables:navigable-6>navigable</a>'s <dfn id=nav-wp>active <code>WindowProxy</code></dfn> is its
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-bc" id=navigables:nav-bc-2>active browsing context</a>'s associated <code id=navigables:windowproxy><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#windowproxy">WindowProxy</a></code>.</p>
  </div>

  <div data-algorithm="">
  <p>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=navigables:navigable-7>navigable</a>'s <dfn data-dfn-for=navigable id=nav-window data-export="">active window</dfn>
  is its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-wp" id=navigables:nav-wp>active <code>WindowProxy</code></a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-windowproxy-window" id=navigables:concept-windowproxy-window>[[Window]]</a>.</p>
  </div>

  <p class=note>This will always equal the navigable's <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document" id=navigables:nav-document-2>active
  document</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global" id=navigables:concept-relevant-global>relevant global object</a>; this is kept in sync by the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#make-active" id=navigables:make-active>make
  active</a> algorithm.</p>

  <div data-algorithm="">
  <p id=browsing-context-name>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=navigables:navigable-8>navigable</a>'s <dfn id=nav-target>target
  name</dfn> is its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-active-history-entry" id=navigables:nav-active-history-entry-8>active session history entry</a>'s
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#she-document-state" id=navigables:she-document-state>document state</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-nav-target-name" id=navigables:document-state-nav-target-name>navigable target name</a>.</p>
  </div>

  <hr>

  <div data-algorithm="">
  <p>To get the <dfn id=node-navigable data-export="">node navigable</dfn> of a <a id=navigables:node href=https://dom.spec.whatwg.org/#interface-node data-x-internal=node>node</a> <var>node</var>, return the
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=navigables:navigable-9>navigable</a> whose <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document" id=navigables:nav-document-3>active document</a> is
  <var>node</var>'s <a id=navigables:node-document href=https://dom.spec.whatwg.org/#concept-node-document data-x-internal=node-document>node document</a>, or null if there is no such
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=navigables:navigable-10>navigable</a>.</p>
  </div>

  <hr>

  <div data-algorithm="">
  <p>To <dfn id=initialize-the-navigable>initialize the navigable</dfn> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=navigables:navigable-11>navigable</a> <var>navigable</var>, given a
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-2" id=navigables:document-state-2>document state</a> <var>documentState</var> and an optional
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=navigables:navigable-12>navigable</a>-or-null <var>parent</var> (default null):</p>

  <ol><li><p><a id=navigables:assert href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: <var>documentState</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-document" id=navigables:document-state-document>document</a> is non-null.<li>
    <p>Let <var>entry</var> be a new <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#session-history-entry" id=navigables:session-history-entry-3>session history entry</a>, with</p>

    <dl class=props><dt><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#she-url" id=navigables:she-url>URL</a><dd><var>documentState</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-document" id=navigables:document-state-document-2>document</a>'s <a href=https://dom.spec.whatwg.org/#concept-document-url id="navigables:the-document's-address" data-x-internal="the-document's-address">URL</a><dt><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#she-document-state" id=navigables:she-document-state-2>document state</a><dd><var>documentState</var></dl>

    <p class=note>The caller of this algorithm is responsible for initializing <var>entry</var>'s
    <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#she-step" id=navigables:she-step>step</a>; it will be left as "<code>pending</code>" until
    that is complete.</p>
   <li><p>Set <var>navigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-current-history-entry" id=navigables:nav-current-history-entry-4>current session
   history entry</a> to <var>entry</var>.<li><p>Set <var>navigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-active-history-entry" id=navigables:nav-active-history-entry-9>active session history
   entry</a> to <var>entry</var>.<li><p>Set <var>navigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-parent" id=navigables:nav-parent-2>parent</a> to
   <var>parent</var>.<li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#set-the-initial-visibility-state" id=navigables:set-the-initial-visibility-state>Set the initial visibility state</a> of <var>documentState</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-document" id=navigables:document-state-document-3>document</a> to
   <var>navigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-traversable" id=navigables:nav-traversable>traversable navigable</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#system-visibility-state" id=navigables:system-visibility-state>system visibility state</a>.</ol>
  </div>


## Traversable definitions and complete new top-level creation algorithm

<!-- selected-source-fragment:70946-71022 -->
  <h5 id=traversable-navigables><span class=secno>7.3.1.1</span> Traversable navigables<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#traversable-navigables" class=self-link></a></h5>

  <p>A <dfn id=traversable-navigable data-export="">traversable navigable</dfn> is a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=traversable-navigables:navigable>navigable</a> that also controls which
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#session-history-entry" id=traversable-navigables:session-history-entry>session history entry</a> should be the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-current-history-entry" id=traversable-navigables:nav-current-history-entry>current
  session history entry</a> and <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-active-history-entry" id=traversable-navigables:nav-active-history-entry>active session history
  entry</a> for itself and its descendant <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=traversable-navigables:navigable-2>navigables</a>.</p>

  <p>In addition to the properties of a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=traversable-navigables:navigable-3>navigable</a>, a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#traversable-navigable" id=traversable-navigables:traversable-navigable>traversable
  navigable</a> has:</p>

  <ul><li><p>A <dfn id=tn-current-session-history-step data-export="">current session history
   step</dfn>, a number, initially 0.<li><p id=session-history><dfn id=tn-session-history-entries>Session history
   entries</dfn>, a <a id=traversable-navigables:list href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a> of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#session-history-entry" id=traversable-navigables:session-history-entry-2>session history
   entries</a>, initially a new <a id=traversable-navigables:list-2 href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a>.<li><p>A <dfn id=tn-session-history-traversal-queue>session history traversal queue</dfn>,
   a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#session-history-traversal-parallel-queue" id=traversable-navigables:session-history-traversal-parallel-queue>session history traversal parallel queue</a>, the result of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#starting-a-new-session-history-traversal-parallel-queue" id=traversable-navigables:starting-a-new-session-history-traversal-parallel-queue>starting a new
   session history traversal parallel queue</a>.<li><p>A <dfn id=tn-running-nested-apply-history-step>running nested apply history
   step</dfn> boolean, initially false.<li>
    <p>A <dfn id=system-visibility-state>system visibility state</dfn>, which is either "<code>hidden</code>" or
    "<code>visible</code>".</p>

    <p>See the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#page-visibility">page visibility</a> section for the requirements on this
    item.</p>
   <li><p>An <dfn id=is-created-by-web-content>is created by web content</dfn> boolean, initially false.</ul>

  <div data-algorithm="">
  <p>To get the <dfn data-dfn-for=navigable id=nav-traversable data-export="">traversable navigable</dfn> of
  a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=traversable-navigables:navigable-4>navigable</a> <var>inputNavigable</var>:</p>

  <ol><li><p>Let <var>navigable</var> be <var>inputNavigable</var>.<li><p>While <var>navigable</var> is not a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#traversable-navigable" id=traversable-navigables:traversable-navigable-2>traversable navigable</a>, set
   <var>navigable</var> to <var>navigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-parent" id=traversable-navigables:nav-parent>parent</a>.<li><p>Return <var>navigable</var>.</ol>
  </div>

  <h5 id=top-level-traversables><span class=secno>7.3.1.2</span> Top-level traversables<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversables" class=self-link></a></h5>

  <div data-algorithm="">
  <p>A <dfn id=top-level-traversable data-export="">top-level traversable</dfn> is a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#traversable-navigable" id=top-level-traversables:traversable-navigable>traversable navigable</a> with a null
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-parent" id=top-level-traversables:nav-parent>parent</a>.</p>
  </div>

  <p class=note>Currently, all <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#traversable-navigable" id=top-level-traversables:traversable-navigable-2>traversable navigables</a>
  are <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable" id=top-level-traversables:top-level-traversable>top-level traversables</a>. Future proposals envision
  introducing non-top-level traversables.</p>

  <p>A user agent holds a <dfn data-dfn-for="user agent" id=top-level-traversable-set data-export="">top-level traversable set</dfn> (a
  <a id=top-level-traversables:set href=https://infra.spec.whatwg.org/#ordered-set data-x-internal=set>set</a> of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable" id=top-level-traversables:top-level-traversable-2>top-level traversables</a>). These are
  typically presented to the user in the form of browser windows or browser tabs.</p>

  <div data-algorithm="">
  <p>To get the <dfn data-dfn-for=navigable id=nav-top data-export="">top-level traversable</dfn> of a
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=top-level-traversables:navigable>navigable</a> <var>inputNavigable</var>:</p>

  <ol><li><p>Let <var>navigable</var> be <var>inputNavigable</var>.<li><p>While <var>navigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-parent" id=top-level-traversables:nav-parent-2>parent</a> is not null, set
   <var>navigable</var> to <var>navigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-parent" id=top-level-traversables:nav-parent-3>parent</a>.<li><p>Return <var>navigable</var>.</ol>
  </div>

  <div data-algorithm="">
  <p>To <dfn id=creating-a-new-top-level-traversable>create a new top-level traversable</dfn>
  given a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=top-level-traversables:browsing-context>browsing context</a>-or-null <var>opener</var>, a string
  <var>targetName</var>, and an optional <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=top-level-traversables:navigable-2>navigable</a>
  <var>openerNavigableForWebDriver</var>:</p>

  <ol><li><p>Let <var>document</var> be null.<li><p>If <var>opener</var> is null, then set <var>document</var> to the second return value of
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#creating-a-new-top-level-browsing-context" id=top-level-traversables:creating-a-new-top-level-browsing-context>creating a new top-level browsing context and document</a>.<li><p>Otherwise, set <var>document</var> to the second return value of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#creating-a-new-auxiliary-browsing-context" id=top-level-traversables:creating-a-new-auxiliary-browsing-context>creating a new
   auxiliary browsing context and document</a> given <var>opener</var>.<li>
    <p>Let <var>documentState</var> be a new <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-2" id=top-level-traversables:document-state-2>document state</a>, with</p>

    <dl class=props><dt><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-document" id=top-level-traversables:document-state-document>document</a><dd><var>document</var><dt><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-initiator-origin" id=top-level-traversables:document-state-initiator-origin>initiator origin</a><dd>null if <var>opener</var> is null; otherwise, <var>document</var>'s <a href=https://dom.spec.whatwg.org/#concept-document-origin id=top-level-traversables:concept-document-origin data-x-internal=concept-document-origin>origin</a><dt><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-origin" id=top-level-traversables:document-state-origin>origin</a><dd><var>document</var>'s <a href=https://dom.spec.whatwg.org/#concept-document-origin id=top-level-traversables:concept-document-origin-2 data-x-internal=concept-document-origin>origin</a><dt><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-nav-target-name" id=top-level-traversables:document-state-nav-target-name>navigable target name</a><dd><var>targetName</var><dt><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-about-base-url" id=top-level-traversables:document-state-about-base-url>about base URL</a><dd><var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-about-base-url" id=top-level-traversables:concept-document-about-base-url>about base
     URL</a></dl>
   <li><p>Let <var>traversable</var> be a new <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#traversable-navigable" id=top-level-traversables:traversable-navigable-3>traversable navigable</a>.<li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#initialize-the-navigable" id=top-level-traversables:initialize-the-navigable>Initialize the navigable</a> <var>traversable</var> given
   <var>documentState</var>.<li><p>Let <var>initialHistoryEntry</var> be <var>traversable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-active-history-entry" id=top-level-traversables:nav-active-history-entry>active session history entry</a>.<li><p>Set <var>initialHistoryEntry</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#she-step" id=top-level-traversables:she-step>step</a> to 0.<li><p><a href=https://infra.spec.whatwg.org/#list-append id=top-level-traversables:list-append data-x-internal=list-append>Append</a> <var>initialHistoryEntry</var> to
   <var>traversable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#tn-session-history-entries" id=top-level-traversables:tn-session-history-entries>session history
   entries</a>.<li id=copy-session-storage><p>If <var>opener</var> is non-null, then <a id=top-level-traversables:legacy-clone-a-traversable-storage-shed href=https://storage.spec.whatwg.org/#legacy-clone-a-traversable-storage-shed data-x-internal=legacy-clone-a-traversable-storage-shed>legacy-clone a
   traversable storage shed</a> given <var>opener</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#bc-traversable" id=top-level-traversables:bc-traversable>top-level
   traversable</a> and <var>traversable</var>. <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsSTORAGE">[STORAGE]</a><li><p><a href=https://infra.spec.whatwg.org/#list-append id=top-level-traversables:list-append-2 data-x-internal=list-append>Append</a> <var>traversable</var> to the user agent's
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable-set" id=top-level-traversables:top-level-traversable-set>top-level traversable set</a>.<li><p>Invoke <a id=top-level-traversables:webdriver-bidi-navigable-created href=https://w3c.github.io/webdriver-bidi/#webdriver-bidi-navigable-created data-x-internal=webdriver-bidi-navigable-created>WebDriver BiDi navigable created</a> with <var>traversable</var>
   and <var>openerNavigableForWebDriver</var>.<li><p>Return <var>traversable</var>.</ol>
  </div>


## Container relations and complete child creation algorithm

<!-- selected-source-fragment:71041-71137 -->
  <h5 id=child-navigables><span class=secno>7.3.1.3</span> Child navigables<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#child-navigables" class=self-link></a></h5>

  <p id=browsing-context-container>Certain elements (for example, <code id=child-navigables:the-iframe-element><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-iframe-element">iframe</a></code> elements)
  can present a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=child-navigables:navigable>navigable</a> to the user. These elements are called <dfn id=navigable-container data-lt="navigable container" data-export="">navigable containers</dfn>.</p>

  <p id=nested-browsing-context>Each <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container" id=child-navigables:navigable-container>navigable container</a> has a <dfn data-dfn-for="navigable container" id=content-navigable data-export="">content navigable</dfn>, which is either a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=child-navigables:navigable-2>navigable</a> or
  null. It is initially null.</p>

  <div data-algorithm="">
  <p id=bc-container>The <dfn data-dfn-for=navigable id=nav-container data-export="">container</dfn> of a
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=child-navigables:navigable-3>navigable</a> <var>navigable</var> is the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container" id=child-navigables:navigable-container-2>navigable container</a> whose
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable" id=child-navigables:content-navigable>content navigable</a> is <var>navigable</var>, or null if there is no such element.</p>
  </div>

  <div data-algorithm="">
  <p id=bc-container-document>The <dfn data-dfn-for=navigable id=nav-container-document data-export="">container document</dfn> of a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=child-navigables:navigable-4>navigable</a>
  <var>navigable</var> is the result of running these steps:</p>

  <ol><li><p>If <var>navigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-container" id=child-navigables:nav-container>container</a> is null, then
   return null.<li>
    <p>Return <var>navigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-container" id=child-navigables:nav-container-2>container</a>'s <a id=child-navigables:node-document href=https://dom.spec.whatwg.org/#concept-node-document data-x-internal=node-document>node
    document</a>.</p>

    <p class=note>This is equal to <var>navigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-container" id=child-navigables:nav-container-3>container</a>'s <a id=child-navigables:shadow-including-root href=https://dom.spec.whatwg.org/#concept-shadow-including-root data-x-internal=shadow-including-root>shadow-including root</a> as
    <var>navigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-container" id=child-navigables:nav-container-4>container</a> has to be
    <a id=child-navigables:connected href=https://dom.spec.whatwg.org/#connected data-x-internal=connected>connected</a>.</p>
   </ol>
  </div>

  <div data-algorithm="">
  <p>The <dfn id=doc-container-document>container document</dfn> of a <code id=child-navigables:document><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>
  <var>document</var> is the result of running these steps:</p>

  <ol><li><p>If <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable" id=child-navigables:node-navigable>node navigable</a> is null, then return null.<li><p>Return <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable" id=child-navigables:node-navigable-2>node navigable</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-container-document" id=child-navigables:nav-container-document>container document</a>.</ol>
  </div>

  <div data-algorithm="">
  <p id=child-browsing-context>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=child-navigables:navigable-5>navigable</a> <var>navigable</var> is a <dfn id=child-navigable data-export="">child navigable</dfn> of another navigable <var>potentialParent</var> when
  <var>navigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-parent" id=child-navigables:nav-parent>parent</a> is <var>potentialParent</var>. We
  can also just say that a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=child-navigables:navigable-6>navigable</a> "is a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#child-navigable" id=child-navigables:child-navigable>child navigable</a>", which means
  that its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-parent" id=child-navigables:nav-parent-2>parent</a> is non-null.</p>
  </div>

  <p class=note>All <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#child-navigable" id=child-navigables:child-navigable-2>child navigables</a> are the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable" id=child-navigables:content-navigable-2>content
  navigable</a> of their <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-container" id=child-navigables:nav-container-5>container</a>.</p>

  <div data-algorithm="">
  <p>The <dfn id=concept-bcc-content-document>content document</dfn> of a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container" id=child-navigables:navigable-container-3>navigable
  container</a> <var>container</var> is the result of running these steps:</p>

  <ol><li><p>If <var>container</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable" id=child-navigables:content-navigable-3>content navigable</a> is null, then return
   null.<li><p>Let <var>document</var> be <var>container</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable" id=child-navigables:content-navigable-4>content navigable</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document" id=child-navigables:nav-document>active document</a>.<li><p>If <var>document</var>'s <a href=https://dom.spec.whatwg.org/#concept-document-origin id=child-navigables:concept-document-origin data-x-internal=concept-document-origin>origin</a> and
   <var>container</var>'s <a id=child-navigables:node-document-2 href=https://dom.spec.whatwg.org/#concept-node-document data-x-internal=node-document>node document</a>'s <a href=https://dom.spec.whatwg.org/#concept-document-origin id=child-navigables:concept-document-origin-2 data-x-internal=concept-document-origin>origin</a> are not <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#same-origin-domain" id=child-navigables:same-origin-domain>same origin-domain</a>, then
   return null.<li><p>Return <var>document</var>.</ol>
  </div>

  <div data-algorithm="">
  <p>The <dfn id=content-window>content window</dfn> of a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container" id=child-navigables:navigable-container-4>navigable container</a> <var>container</var> is the
  result of running these steps:</p>

  <ol><li><p>If <var>container</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable" id=child-navigables:content-navigable-5>content navigable</a> is null, then return
   null.<li><p>Return <var>container</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable" id=child-navigables:content-navigable-6>content navigable</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-wp" id=child-navigables:nav-wp>active <code>WindowProxy</code></a>'s object.</ol>
  </div>

  <hr>

  <div data-algorithm="">
  <p id=creating-a-new-nested-browsing-context>To <dfn id=create-a-new-child-navigable>create a new child navigable</dfn>, given
  an element <var>element</var>:</p>

  <ol><li><p>Let <var>parentNavigable</var> be <var>element</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable" id=child-navigables:node-navigable-3>node
   navigable</a>.<li><p>Let <var>group</var> be <var>element</var>'s <a id=child-navigables:node-document-3 href=https://dom.spec.whatwg.org/#concept-node-document data-x-internal=node-document>node document</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id=child-navigables:concept-document-bc>browsing context</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#bc-tlbc" id=child-navigables:bc-tlbc>top-level browsing
   context</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#tlbc-group" id=child-navigables:tlbc-group>group</a>.<li><p>Let <var>browsingContext</var> and <var>document</var> be the result of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#creating-a-new-browsing-context" id=child-navigables:creating-a-new-browsing-context>creating a
   new browsing context and document</a> given <var>element</var>'s <a id=child-navigables:node-document-4 href=https://dom.spec.whatwg.org/#concept-node-document data-x-internal=node-document>node document</a>,
   <var>element</var>, and <var>group</var>.<li><p>Let <var>targetName</var> be null.<li><p>If <var>element</var> has a <code>name</code> content attribute, then set
   <var>targetName</var> to the value of that attribute.<li>
    <p>Let <var>documentState</var> be a new <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-2" id=child-navigables:document-state-2>document state</a>, with</p>

    <dl class=props><dt><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-document" id=child-navigables:document-state-document>document</a><dd><var>document</var><dt><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-initiator-origin" id=child-navigables:document-state-initiator-origin>initiator origin</a><dd><var>document</var>'s <a href=https://dom.spec.whatwg.org/#concept-document-origin id=child-navigables:concept-document-origin-3 data-x-internal=concept-document-origin>origin</a><dt><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-origin" id=child-navigables:document-state-origin>origin</a><dd><var>document</var>'s <a href=https://dom.spec.whatwg.org/#concept-document-origin id=child-navigables:concept-document-origin-4 data-x-internal=concept-document-origin>origin</a><dt><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-nav-target-name" id=child-navigables:document-state-nav-target-name>navigable target name</a><dd><var>targetName</var><dt><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-about-base-url" id=child-navigables:document-state-about-base-url>about base URL</a><dd><var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-about-base-url" id=child-navigables:concept-document-about-base-url>about base
     URL</a></dl>
   <li><p>Let <var>navigable</var> be a new <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=child-navigables:navigable-7>navigable</a>.<li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#initialize-the-navigable" id=child-navigables:initialize-the-navigable>Initialize the navigable</a> <var>navigable</var> given <var>documentState</var>
   and <var>parentNavigable</var>.<li><p>Set <var>element</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable" id=child-navigables:content-navigable-7>content navigable</a> to <var>navigable</var>.<li><p>Let <var>historyEntry</var> be <var>navigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-active-history-entry" id=child-navigables:nav-active-history-entry>active session history entry</a>.<li><p>Let <var>traversable</var> be <var>parentNavigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-traversable" id=child-navigables:nav-traversable>traversable navigable</a>.<li>
    <p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#tn-append-session-history-traversal-steps" id=child-navigables:tn-append-session-history-traversal-steps>Append the following session
    history traversal steps</a> to <var>traversable</var>:</p>

    <ol><li><p>Let <var>parentDocState</var> be <var>parentNavigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-active-history-entry" id=child-navigables:nav-active-history-entry-2>active session history entry</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#she-document-state" id=child-navigables:she-document-state>document state</a>.<li><p>Let <var>parentNavigableEntries</var> be the result of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#getting-session-history-entries" id=child-navigables:getting-session-history-entries>getting session history
     entries</a> for <var>parentNavigable</var>.<li><p>Let <var>targetStepSHE</var> be the first <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#session-history-entry" id=child-navigables:session-history-entry>session history entry</a> in
     <var>parentNavigableEntries</var> whose <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#she-document-state" id=child-navigables:she-document-state-2>document
     state</a> equals <var>parentDocState</var>.<li><p>Set <var>historyEntry</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#she-step" id=child-navigables:she-step>step</a> to
     <var>targetStepSHE</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#she-step" id=child-navigables:she-step-2>step</a>.<li><p>Let <var>nestedHistory</var> be a new <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nested-history" id=child-navigables:nested-history>nested history</a> whose <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nested-history-id" id=child-navigables:nested-history-id>id</a> is <var>navigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-id" id=child-navigables:nav-id>id</a>
     and <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nested-history-entries" id=child-navigables:nested-history-entries>entries list</a> is « <var>historyEntry</var>
     ».<li><p><a href=https://infra.spec.whatwg.org/#list-append id=child-navigables:list-append data-x-internal=list-append>Append</a> <var>nestedHistory</var> to
     <var>parentDocState</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-nested-histories" id=child-navigables:document-state-nested-histories>nested
     histories</a>.<li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#update-for-navigable-creation/destruction" id=child-navigables:update-for-navigable-creation/destruction>Update for navigable creation/destruction</a> given
     <var>traversable</var>.</ol>
   <li><p>Invoke <a id=child-navigables:webdriver-bidi-navigable-created href=https://w3c.github.io/webdriver-bidi/#webdriver-bidi-navigable-created data-x-internal=webdriver-bidi-navigable-created>WebDriver BiDi navigable created</a> with <var>traversable</var>.</ol>
  </div>


## Complete child and top-level destruction algorithms

<!-- selected-source-fragment:71267-71304 -->
  <h5 id=garbage-collection-and-browsing-contexts><span class=secno>7.3.1.6</span> Navigable destruction<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#garbage-collection-and-browsing-contexts" class=self-link></a></h5>

  <div data-algorithm="">
  <p>To <dfn id=destroy-a-child-navigable>destroy a child navigable</dfn> given a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container" id=garbage-collection-and-browsing-contexts:navigable-container>navigable container</a>
  <var>container</var>:</p>

  <ol><li><p>Let <var>navigable</var> be <var>container</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable" id=garbage-collection-and-browsing-contexts:content-navigable>content
   navigable</a>.<li><p>If <var>navigable</var> is null, then return.<li><p>Set <var>container</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable" id=garbage-collection-and-browsing-contexts:content-navigable-2>content navigable</a> to null.<li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#inform-the-navigation-api-about-child-navigable-destruction" id=garbage-collection-and-browsing-contexts:inform-the-navigation-api-about-child-navigable-destruction>Inform the navigation API about child navigable destruction</a> given
   <var>navigable</var>.<li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#destroy-a-document-and-its-descendants" id=garbage-collection-and-browsing-contexts:destroy-a-document-and-its-descendants>Destroy a document and its descendants</a> given <var>navigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document" id=garbage-collection-and-browsing-contexts:nav-document>active document</a>.<li><p>Let <var>parentDocState</var> be <var>container</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable" id=garbage-collection-and-browsing-contexts:node-navigable>node navigable</a>'s
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-active-history-entry" id=garbage-collection-and-browsing-contexts:nav-active-history-entry>active session history entry</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#she-document-state" id=garbage-collection-and-browsing-contexts:she-document-state>document state</a>.<li><p><a href=https://infra.spec.whatwg.org/#list-remove id=garbage-collection-and-browsing-contexts:list-remove data-x-internal=list-remove>Remove</a> the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nested-history" id=garbage-collection-and-browsing-contexts:nested-history>nested history</a> from
   <var>parentDocState</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-nested-histories" id=garbage-collection-and-browsing-contexts:document-state-nested-histories>nested
   histories</a> whose <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nested-history-id" id=garbage-collection-and-browsing-contexts:nested-history-id>id</a> equals
   <var>navigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-id" id=garbage-collection-and-browsing-contexts:nav-id>id</a>.<li><p>Let <var>traversable</var> be <var>container</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable" id=garbage-collection-and-browsing-contexts:node-navigable-2>node navigable</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-traversable" id=garbage-collection-and-browsing-contexts:nav-traversable>traversable navigable</a>.<li>
    <p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#tn-append-session-history-traversal-steps" id=garbage-collection-and-browsing-contexts:tn-append-session-history-traversal-steps>Append the following session
    history traversal steps</a> to <var>traversable</var>:</p>

    <ol><li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#update-for-navigable-creation/destruction" id=garbage-collection-and-browsing-contexts:update-for-navigable-creation/destruction>Update for navigable creation/destruction</a> given
     <var>traversable</var>.</ol>
   <li><p>Invoke <a id=garbage-collection-and-browsing-contexts:webdriver-bidi-navigable-destroyed href=https://w3c.github.io/webdriver-bidi/#webdriver-bidi-navigable-destroyed data-x-internal=webdriver-bidi-navigable-destroyed>WebDriver BiDi navigable destroyed</a> with <var>navigable</var>.</ol>
  </div>

  <div data-algorithm="">
  <p>To <dfn id=destroy-a-top-level-traversable>destroy</dfn> a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable" id=garbage-collection-and-browsing-contexts:top-level-traversable>top-level
  traversable</a> <var>traversable</var>:</p>

  <ol><li><p>Let <var>browsingContext</var> be <var>traversable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-bc" id=garbage-collection-and-browsing-contexts:nav-bc>active browsing context</a>.<li>
    <p><a href=https://infra.spec.whatwg.org/#list-iterate id=garbage-collection-and-browsing-contexts:list-iterate data-x-internal=list-iterate>For each</a> <var>historyEntry</var> in
    <var>traversable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#tn-session-history-entries" id=garbage-collection-and-browsing-contexts:tn-session-history-entries>session history
    entries</a> <span class=XXX>in what order?</span>:</p>

    <ol><li><p>Let <var>document</var> be <var>historyEntry</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#she-document" id=garbage-collection-and-browsing-contexts:she-document>document</a>.<li><p>If <var>document</var> is not null, then <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#destroy-a-document-and-its-descendants" id=garbage-collection-and-browsing-contexts:destroy-a-document-and-its-descendants-2>destroy a document and its
     descendants</a> given <var>document</var>.</ol>
   <li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#bcg-remove" id=garbage-collection-and-browsing-contexts:bcg-remove>Remove</a> <var>browsingContext</var>.<li><p>Remove <var>traversable</var> from the user interface (e.g., close or hide its tab in
   a tabbed browser).<li><p><a href=https://infra.spec.whatwg.org/#list-remove id=garbage-collection-and-browsing-contexts:list-remove-2 data-x-internal=list-remove>Remove</a> <var>traversable</var> from the user agent's
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable-set" id=garbage-collection-and-browsing-contexts:top-level-traversable-set>top-level traversable set</a>.<li><p>Invoke <a id=garbage-collection-and-browsing-contexts:webdriver-bidi-navigable-destroyed-2 href=https://w3c.github.io/webdriver-bidi/#webdriver-bidi-navigable-destroyed data-x-internal=webdriver-bidi-navigable-destroyed>WebDriver BiDi navigable destroyed</a> with <var>traversable</var>.</ol>
  </div>

  <p>User agents may <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#destroy-a-top-level-traversable" id=garbage-collection-and-browsing-contexts:destroy-a-top-level-traversable>destroy a top-level traversable</a> at any time (typically, <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-traversal-ui">in response to user requests</a>).</p>


## Complete session-history-entry items and Document getter

<!-- selected-source-fragment:72024-72055 -->
  <h5 id=session-history-entries><span class=secno>7.4.1.1</span> Session history entries<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#session-history-entries" class=self-link></a></h5>

  <p>A <dfn id=session-history-entry>session history entry</dfn> is a <a id=session-history-entries:struct href=https://infra.spec.whatwg.org/#struct data-x-internal=struct>struct</a> with the following <a href=https://infra.spec.whatwg.org/#struct-item id=session-history-entries:struct-item data-x-internal=struct-item>items</a>:</p>

  <ul><li><p><dfn id=she-step>step</dfn>, a non-negative integer or "<code>pending</code>", initially "<code>pending</code>".<li><p><dfn id=she-url>URL</dfn>, a <a id=session-history-entries:url href=https://url.spec.whatwg.org/#concept-url data-x-internal=url>URL</a><li><p><dfn id=she-document-state>document state</dfn>, a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-2" id=session-history-entries:document-state-2>document
   state</a>.<li><p id=she-serialized-state><dfn id=she-classic-history-api-state>classic history API
   state</dfn>, which is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#serialized-state" id=session-history-entries:serialized-state>serialized state</a>, initially
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#structuredserializeforstorage" id=session-history-entries:structuredserializeforstorage>StructuredSerializeForStorage</a>(null).<li><p><dfn id=she-navigation-api-state>navigation API state</dfn>, which is a
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#serialized-state" id=session-history-entries:serialized-state-2>serialized state</a>, initially
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#structuredserializeforstorage" id=session-history-entries:structuredserializeforstorage-2>StructuredSerializeForStorage</a>(undefined).<li><p><dfn id=she-navigation-api-key>navigation API key</dfn>, which is a string,
   initially set to the result of <a id=session-history-entries:generating-a-random-uuid href=https://w3c.github.io/webcrypto/#dfn-generate-a-random-uuid data-x-internal=generating-a-random-uuid>generating a random UUID</a>.<li><p><dfn id=she-navigation-api-id>navigation API ID</dfn>, which is a string, initially
   set to the result of <a id=session-history-entries:generating-a-random-uuid-2 href=https://w3c.github.io/webcrypto/#dfn-generate-a-random-uuid data-x-internal=generating-a-random-uuid>generating a random UUID</a>.<li><p><dfn id=she-scroll-restoration-mode>scroll restoration mode</dfn>, a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#scroll-restoration-mode" id=session-history-entries:scroll-restoration-mode>scroll
   restoration mode</a>, initially "<code id=session-history-entries:dom-scrollrestoration-auto><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-scrollrestoration-auto">auto</a></code>".<li><p><dfn id=she-scroll-position>scroll position data</dfn>, which is scroll position
   data for the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#she-document" id=session-history-entries:she-document>document</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#restorable-scrollable-regions" id=session-history-entries:restorable-scrollable-regions>restorable scrollable
   regions</a>.<li>
    <p id=an-entry-with-persisted-user-state><dfn id=she-other>persisted user state</dfn>,
    which is <a id=session-history-entries:implementation-defined href=https://infra.spec.whatwg.org/#implementation-defined data-x-internal=implementation-defined>implementation-defined</a>, initially null</p>

    <p class=example>For example, some user agents might want to persist the values of form
    controls.</p>

    <p class=note>User agents that persist the value of form controls are encouraged to also
    persist their directionality (the value of the element's <code id=session-history-entries:attr-dir><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-dir">dir</a></code>
    attribute). This prevents values from being displayed incorrectly after a history traversal
    when the user had originally entered the values with an explicit, non-default
    directionality.</p>
   </ul>

  <div data-algorithm="">
  <p>To get a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#session-history-entry" id=session-history-entries:session-history-entry>session history entry</a>'s <dfn id=she-document>document</dfn>,
  return its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#she-document-state" id=session-history-entries:she-document-state>document state</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-document" id=session-history-entries:document-state-document>document</a>.</p>
  </div>


## Complete Document-state items and retention notes

<!-- selected-source-fragment:72094-72169 -->
  <h5 id=document-state><span class=secno>7.4.1.2</span> Document state<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state" class=self-link></a></h5>

  <p><dfn id=document-state-2>Document state</dfn> holds state inside a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#session-history-entry" id=document-state:session-history-entry>session history entry</a> regarding
  how to present and, if necessary, recreate, a <code id=document-state:document><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>. It has:</p>

  <ul><li>
    <p>A <dfn data-dfn-for="document state" id=document-state-document>document</dfn>, a
    <code id=document-state:document-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> or null, initially null.</p>

    <div id=note-bfcache class=note><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#note-bfcache" class=self-link></a>
     <p>When a history entry is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-active-history-entry" id=document-state:nav-active-history-entry>active</a>, it has a
     <code id=document-state:document-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> in its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#she-document-state" id=document-state:she-document-state>document state</a>. However,
     when a <code id=document-state:document-4><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> is not <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fully-active" id=document-state:fully-active>fully active</a>, it's possible for it to be
     <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#destroy-a-document" id=document-state:destroy-a-document>destroyed</a> to free resources. In such cases, this
     <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-document" id=document-state:document-state-document>document</a> item will be nulled out. The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#she-url" id=document-state:she-url>URL</a> and other data in the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#session-history-entry" id=document-state:session-history-entry-2>session history entry</a> and <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#she-document-state" id=document-state:she-document-state-2>document state</a> is then used to bring a new
     <code id=document-state:document-5><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> into being to take the place of the original, in the case where the user
     agent finds itself having to traverse to the entry.</p>

     <p>If the <code id=document-state:document-6><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> is <em>not</em> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#destroy-a-document" id=document-state:destroy-a-document-2>destroyed</a>, then during <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#traverse-the-history-by-a-delta" id=document-state:traverse-the-history-by-a-delta>history
     traversal</a>, it can be <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#reactivate-a-document" id=document-state:reactivate-a-document>reactivated</a>. The cache
     in which browsers store such <code id=document-state:document-7><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>s is often called a <em>back-forward
     cache</em>, or <em>bfcache</em> (or perhaps <a href="https://bugzilla.mozilla.org/show_bug.cgi?id=274784">"blazingly fast"</a> cache).</p>
    </div>
   <li><p id=she-policy-container>A <dfn data-dfn-for="document state" id=document-state-history-policy-container>history policy container</dfn>, a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container" id=document-state:policy-container>policy container</a> or
   null, initially null.<li><p>A <dfn data-dfn-for="document state" id=document-state-request-referrer>request
   referrer</dfn>, which is "<code>no-referrer</code>", "<code>client</code>",
   or a <a id=document-state:url href=https://url.spec.whatwg.org/#concept-url data-x-internal=url>URL</a>, initially "<code>client</code>".<li>
    <p>A <dfn data-dfn-for="document state" id=document-state-request-referrer-policy>request
    referrer policy</dfn>, which is a <a id=document-state:referrer-policy href=https://w3c.github.io/webappsec-referrer-policy/#referrer-policy data-x-internal=referrer-policy>referrer policy</a>, initially the <a id=document-state:default-referrer-policy href=https://w3c.github.io/webappsec-referrer-policy/#default-referrer-policy data-x-internal=default-referrer-policy>default
    referrer policy</a>.</p>

    <p class=note>The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-request-referrer-policy" id=document-state:document-state-request-referrer-policy>request referrer
    policy</a> is distinct from the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-history-policy-container" id=document-state:document-state-history-policy-container>history policy container</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#policy-container-referrer-policy" id=document-state:policy-container-referrer-policy>referrer policy</a>. The former is used for
    fetches <em>of</em> this document, whereas the latter controls fetches <em>by</em> this
    document.</p>
   <li>
    <p>An <dfn data-dfn-for="document state" id=document-state-initiator-origin>initiator origin</dfn>,
    which is an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin" id=document-state:concept-origin>origin</a> or null, initially null.</p>
   <li>
    <p>An <dfn data-dfn-for="document state" id=document-state-origin>origin</dfn>, which is an
    <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin" id=document-state:concept-origin-2>origin</a> or null, initially null.</p>

    <p class=note>This is the origin that we set "<code>about:</code>"-schemed
    <code id=document-state:document-8><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>s' <a href=https://dom.spec.whatwg.org/#concept-document-origin id=document-state:concept-document-origin data-x-internal=concept-document-origin>origin</a> to. We store it
    here because it is also used when restoring these <code id=document-state:document-9><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>s during traversal,
    since they are reconstructed locally without visiting the network. It is also used to compare
    the origin before and after the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#session-history-entry" id=document-state:session-history-entry-3>session history entry</a> is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attempt-to-populate-the-history-entry's-document" id="document-state:attempt-to-populate-the-history-entry's-document">repopulated</a>. If the origins change, the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-nav-target-name" id=document-state:document-state-nav-target-name>navigable target name</a> is cleared.</p>
   <li>
    <p>An <dfn data-dfn-for="document state" id=document-state-about-base-url>about base URL</dfn>,
    which is a <a id=document-state:url-2 href=https://url.spec.whatwg.org/#concept-url data-x-internal=url>URL</a> or null, initially null.</p>

    <p class=note>This will be populated only for "<code>about:</code>"-schemed
    <code id=document-state:document-10><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>s and will be the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fallback-base-url" id=document-state:fallback-base-url>fallback base URL</a> for those
    <code id=document-state:document-11><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>s. It is a snapshot of the initiator <code id=document-state:document-12><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-base-url" id=document-state:document-base-url>document
    base URL</a>.</p>
   <li><p><dfn data-dfn-for="document state" id=document-state-nested-histories>Nested histories</dfn>,
   a <a id=document-state:list href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a> of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nested-history" id=document-state:nested-history>nested histories</a>, initially an empty
   <a id=document-state:list-2 href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a>.<li>
    <p>A <dfn data-dfn-for="document state" id=document-state-resource>resource</dfn>, a string,
    <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#post-resource" id=document-state:post-resource>POST resource</a> or null, initially null.</p>

    <p class=note>A string is treated as HTML. It's used to store the source of an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#an-iframe-srcdoc-document" id=document-state:an-iframe-srcdoc-document><code>iframe</code> <code>srcdoc</code> document</a>.</p>
   <li><p>A <dfn data-dfn-for="document state" id=document-state-reload-pending>reload pending</dfn>
   boolean, initially false.<li><p>An <dfn data-dfn-for="document state" id=document-state-ever-populated>ever populated</dfn>
   boolean, initially false.<li><p id=she-bc-name>A <dfn data-dfn-for="document
   state" id=document-state-nav-target-name>navigable target name</dfn> string, initially the empty string.<li><p>A <dfn data-dfn-for="document state" id=document-state-not-restored-reasons>not restored
   reasons</dfn>, a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nrr-struct" id=document-state:nrr-struct>not restored reasons</a> or null, initially
   null.</ul>

  <p>User agents may <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#destroy-a-document-and-its-descendants" id=document-state:destroy-a-document-and-its-descendants>destroy a document and its descendants</a> given the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-document" id=document-state:document-state-document-2>documents</a> of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-2" id=document-state:document-state-2>document
  states</a> with non-null <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-document" id=document-state:document-state-document-3>documents</a>, as long as
  the <code id=document-state:document-13><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> is not <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fully-active" id=document-state:fully-active-2>fully active</a>.</p>

  <p>Apart from that restriction, this standard does not specify when user agents should destroy
  the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-document" id=document-state:document-state-document-4>document</a> stored in a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-2" id=document-state:document-state-2-2>document
  state</a>, versus keeping it cached.</p>


## Complete nested-history definition and source issue note

<!-- selected-source-fragment:72189-72198 -->
  <p>A <dfn id=nested-history>nested history</dfn> has:</p>

  <ul><li>
    <p>An <dfn id=nested-history-id>id</dfn>, a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unique-internal-value" id=document-state:unique-internal-value>unique internal value</a>.</p>

    <p class=note>This is used to associate the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nested-history" id=document-state:nested-history-2>nested history</a> with a
    <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=document-state:navigable>navigable</a>.</p>
   <li><p><dfn id=nested-history-entries>Entries</dfn>, a <a id=document-state:list-3 href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a> of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#session-history-entry" id=document-state:session-history-entry-4>session history entries</a>.</ul>

  <p class=XXX>This will later contain ways to identify a child navigable across reloads.</p>


## Complete make-active synchronization algorithm

<!-- selected-source-fragment:74892-74900 -->
  <div data-algorithm="">
  <p>To <dfn id=make-active>make active</dfn> a <code id=updating-the-document:document-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> <var>document</var>:</p>

  <ol><li><p>Let <var>window</var> be <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global" id=updating-the-document:concept-relevant-global-5>relevant global
   object</a>.<li><p>Set <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id=updating-the-document:concept-document-bc>browsing context</a>'s
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#active-document" id=updating-the-document:active-document>active document</a> to <var>document</var>.<li><p>Set <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id=updating-the-document:concept-document-bc-2>browsing context</a>'s
   <code id=updating-the-document:windowproxy><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#windowproxy">WindowProxy</a></code>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-windowproxy-window" id=updating-the-document:concept-windowproxy-window>[[Window]]</a> internal
   slot value to <var>window</var>.<li><p>Set <var>window</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant-settings-object" id=updating-the-document:relevant-settings-object>relevant settings object</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-environment-execution-ready-flag" id=updating-the-document:concept-environment-execution-ready-flag>execution ready flag</a>.</ol>
  </div>
