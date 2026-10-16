# HTML child navigable history and traversal clauses

Bounded source retention for child creation/destruction support from the
existing pinned HTML source. Retention supplies source evidence, not an
implementation release, expanded native execution or browser-conformance claim.

## Source basis and attribution

Source: [immutable WHATWG HTML snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/), revision
`877d6146487f15b15bc8dd0e3c625063a37b456a` (9 October 2026).
Extracted offline on 16 October 2026 from the already-retained complete HTML:
14,930,614 bytes, SHA-256
`877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
No retrieval, changed edition, executable or software acquisition is used.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose is
[Creative Commons Attribution 4.0 International](https://creativecommons.org/licenses/by/4.0/).
The [complete WHATWG license](../licenses/whatwg/LICENSE), SHA-256
`85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`,
and [pinned HTML attribution](../licenses/whatwg/NOTICE.md#pinned-html-language-snapshot--2026-10-11)
apply. The upstream BSD 3-Clause provision applies when portions are incorporated
into source code, not as an alternative license for retained specification prose.

Changes by Surgeist on 16 October 2026: bounded offline extraction and added
Markdown headings, original-range/fragment-hash provenance, attribution and
separately identified scope notes. The original HTML is **byte-identical**
inside fenced text: wording, whitespace, tags, IDs, variables, conditions,
algorithm/list order, all branches, notes and issue markers remain unchanged.
No href or source citation is rewritten. Source scripts are neither acquired
nor run. These are inert source blocks, not a rendered or replacement technical
specification, browser conformance assertion or execution of JavaScript steps.

The fragments in ascending original-source order, joined by one extra LF,
have SHA-256 `a9f635b298df880ce64558a848b6d4b7131ad28907ad05d444dca2e9f266cb3f`.
The filename suffix identifies that joined original input, not this wrapper.

| Original raw lines | Complete retained unit | Original fragment SHA-256 |
| --- | --- | --- |
| 67712–67734 | Document History object, getter, initial state and explanatory note | `793843651674903cbc4c9d6631af3464aa87010b800e78da2ecbd26dc5fcaae6` |
| 69258–69305 | Complete Navigation aborting and child-destruction notification algorithms | `faa237a827232b8f289b887ee247bab1b053458446f3b42cc52f4c360aff77c6` |
| 72214–72218 | Document latest-entry definition and complete note | `445e96494fcbc244d2371ba49fadfa1d4994ca11675c60f0928361fc8f856abc` |
| 72221–72275 | Traversal queue definition, both append algorithms, complete runner and queue-jumping note | `bc55aa33e331995e5bc34450875869cc348ad4a7a028194d87f284aaf5343fea` |
| 72399–72419 | Complete session-history entries lookup algorithm | `6597360f459a39cdd802980e1f07a62b404fa6f945ab0fd669db2215f52050dc` |
| 72473–72491 | Complete all-used-history-steps algorithm | `10f65a43f2102a50f408b25634c3fb09d033c8f5459d696b4345957c0e953c5f` |
| 74141–74147 | Complete creation/destruction history-update caller | `e67d16e6bd503a2dad5b32a9ac4ce96c4738e24eb4cb3d8942250f9c17eab9b4` |
| 74206–74487 | Complete apply-history-step algorithm, all branches | `b013e744aa45e575ac107cb25df9ad11a4652569194e564217d69d84cdfe9699` |
| 74612–74644 | Complete used-step and History length/index algorithms | `516514292ec0b507c8a49479dec6c1c35684a86e6918b168f535e3dabca70bef` |
| 74646–74721 | Complete changing, nonchanging and target-entry lookup algorithms | `39e45614dffb967b63de4332c7e572a87af2c2001a8d5551a647eb9c6a1deea2` |
| 74743–74788 | Complete potentially-cross-document navigables algorithm | `4946ebc14bcc518a4b44f03aefde6939563058906836666f98667ed02e01d0fe` |

## Editorial reuse and immediate child boundary

Reuse the unchanged [navigable lifecycle clauses](html-hosted-navigable-lifecycle--877d6146487f--3a9658155866.md)
for child creation/destruction callers and actual Entry/DocumentState/nested
history records. Reuse the [initial constructor/global clauses](html-hosted-global-agent-settings--877d6146487f--03a314e0aa55.md)
for the real creator/embedder-aware BC, origin, agent, settings, registry and
Document construction recipe. [Initial teardown clauses](html-hosted-initial-teardown--877d6146487f--c2dd632ac56d.md)
retain related navigable collections, actual Document/descendant destruction,
abort, cleanup, task sources and the literal raw75916 wait-variable discrepancy.
That discrepancy's separate #136 captured-plural-child-count qualification is
not a modification of the retained body, and nonempty captured child counts
still require actual child tasks and completions.

The complete [Iframe hooks and attribute processing](html-language-elements-embedded--877d6146487f--48a1a13e5cfe.md)
and [Frame hooks](html-language-obsolete--877d6146487f--66870e9cdabe.md)
already exist at the same pin. Complete traversal work is required even for the
initial-entry-only domain: compute actual target/changing/cross-document and
nonchanging sets, queue the real nonchanging global tasks, update each actual
History object, wait for completed tasks and set the traversable step. Retaining
all apply-history-step branches does not admit replacement/reload/back-forward
or fulfill external execution by treating JavaScript exclusions as no-ops.

History's source state begins null, length0 and index0. A later computed
length1/index0 is an effect of reached traversal work, not an invented initial
default. Child Entry.step remains Pending until the original queued creation
steps assign it. Empty Navigation event/trackers must follow from actual state;
the complete aborting/child-destruction bodies and relevant-agent/global task
branches remain visible below.

### Literal argument discrepancy and earlier URL guard

The retained shared Frame/Iframe algorithm raw23996 names the inclusive ancestor
navigables of an element's **node navigable**, while its linked, sole definition
raw71211–71219 is explicitly for a **Document**. Both remain literal in their
existing references. Architecture's separately recorded
[#1124 caller-type qualification](https://github.com/bj-data-eng/surgeist/issues/1124),
“Frame/Iframe inclusive-ancestor guard: caller type qualification”, invokes
that Document helper on the element's actual current node Document D, obtaining
D.nodeNavigable N and the source ancestor list with N appended. N is the
containing parent, not the newly created child/content navigable. Missing or
retired N is not an empty ancestor chain. The comparison uses actual URL value
equality with only fragments excluded. This editorial native qualification is
not an upstream fix, new edition, altered literal source or invented overload.

Under the parent-inclusive relation, the exact no-srcdoc, absent/empty-src
initial-about:blank-parent case reaches the earlier ancestor-URL match
raw23996–23998 and returns null. It returns before URL/history update and before
the subsequent Frame/Iframe initial load branch or lazy predicate. This is an
argument-derived boundary, not an interpretation of the later prose note as an
executable no-op. Different admitted arguments require their actually reached
source chain. The qualification is attributed separately above rather than
applied as an edit to the retained original.

The earlier Permissions Policy phase is distinct: #1218's declared-origin
algorithm checks a **present** src, so present-empty src may still require actual
URL parsing/origin evidence even though shared Frame/Iframe processing skips
empty src. This later null return does not establish that the whole child
constructor needs no URL service or may waive previously reached policy work.

### Conditional source not selected by this input

A nonmatching ancestor URL and a matching-about:blank candidate can reach
URL/history update raw73487–73532. That body creates a new Entry, changes
latest/active, calls Navigation same-document update raw68408–68478 and appends
synchronous finalization raw73126–73162; its replace branch calls the
push/replace wrapper raw74149–74158 even for replacement. These bodies and any
then-reached update-Document/restore-History branches are **not adopted or
retained by this reference**. Raw4834–4847's lazy predicate is likewise a separate
source need if a later admitted src/srcdoc path actually reaches it. Retained
external calls do not certify the implementations or transitive source closure
of WebDriver, navigation, fetching, event execution or other browser services.

## Literal source bodies

### Document History object, getter, initial state and explanatory note

<!-- selected-source-fragment:67712-67734 -->
```text
  <p>A <code id=the-history-interface:document-3><a href=#document>Document</a></code> has a <dfn id=doc-history>history object</dfn>, a
  <code id=the-history-interface:history-3-2><a href=#history-3>History</a></code> object.</p>

  <div data-algorithm="">
  <p>The <dfn data-dfn-for=Window id=dom-history data-dfn-type=attribute><code>history</code></dfn> getter steps
  are to return <a id=the-history-interface:this href=https://webidl.spec.whatwg.org/#this data-x-internal=this>this</a>'s <a href=#concept-document-window id=the-history-interface:concept-document-window>associated
  <code>Document</code></a>'s <a href=#doc-history id=the-history-interface:doc-history>history object</a>.</p>
  </div>

  <hr>

  <p>Each <code id=the-history-interface:history-3-3><a href=#history-3>History</a></code> object has <dfn id=concept-history-state>state</dfn>,
  initially null.</p>

  <p>Each <code id=the-history-interface:history-3-4><a href=#history-3>History</a></code> object has a <dfn id=concept-history-length>length</dfn>, a
  non-negative integer, initially 0.</p>

  <p>Each <code id=the-history-interface:history-3-5><a href=#history-3>History</a></code> object has an <dfn id=concept-history-index>index</dfn>, a
  non-negative integer, initially 0.</p>

  <p class=note>Although the <a href=#concept-history-index id=the-history-interface:concept-history-index>index</a> is not directly
  exposed, it can be inferred from changes to the <a href=#concept-history-length id=the-history-interface:concept-history-length>length</a> during synchronous navigations. In fact, that is
  what it's used for.</p>
```

### Complete Navigation aborting and child-destruction notification algorithms

<!-- selected-source-fragment:69258-69305 -->
```text
  <div data-algorithm="">
  <p>To <dfn id=inform-the-navigation-api-about-aborting-navigation>inform the navigation API about aborting navigation</dfn> in a <a href=#navigable id=ongoing-navigation-tracking:navigable>navigable</a>
  <var>navigable</var>:</p>

  <ol><li><p>If this algorithm is running on <var>navigable</var>'s <a href=#nav-window id=ongoing-navigation-tracking:nav-window>active
   window</a>'s <a href=#relevant-agent id=ongoing-navigation-tracking:relevant-agent>relevant agent</a>'s <a href=#concept-agent-event-loop id=ongoing-navigation-tracking:concept-agent-event-loop>event
   loop</a>, then continue on to the following steps. Otherwise, <a href=#queue-a-global-task id=ongoing-navigation-tracking:queue-a-global-task>queue a global task</a>
   on the <a href=#navigation-and-traversal-task-source id=ongoing-navigation-tracking:navigation-and-traversal-task-source>navigation and traversal task source</a> given <var>navigable</var>'s <a href=#nav-window id=ongoing-navigation-tracking:nav-window-2>active window</a> to run the following steps.<li><p>Let <var>navigation</var> be <var>navigable</var>'s <a href=#nav-window id=ongoing-navigation-tracking:nav-window-3>active
   window</a>'s <a href=#window-navigation-api id=ongoing-navigation-tracking:window-navigation-api-2>navigation API</a>.<li>
    <p><a id=ongoing-navigation-tracking:while href=https://infra.spec.whatwg.org/#iteration-while data-x-internal=while>While</a> <var>navigation</var>'s <a href=#ongoing-navigate-event id=ongoing-navigation-tracking:ongoing-navigate-event-3>ongoing <code>navigate</code> event</a> is not null:</p>

    <ol><li><p><a href=#abort-the-ongoing-navigation id=ongoing-navigation-tracking:abort-the-ongoing-navigation>Abort the ongoing navigation</a> given <var>navigation</var>.</ol>

    <div class=note>
     <p>If there is an ongoing cross-document navigation, this means it will be signaled to the
     navigation API as aborted, e.g., by firing <code id=ongoing-navigation-tracking:event-navigateerror-3><a href=#event-navigateerror>navigateerror</a></code> events. This is somewhat accurate, since the
     next navigation the <code id=ongoing-navigation-tracking:document><a href=#document>Document</a></code> experiences will be this same-document navigation, so
     a developer which was expecting the next navigation completion to be that of the cross-document
     navigation gets a useful signal that this did not happen. However, it is also somewhat
     inaccurate, as the browser will continue to process the ongoing cross-document navigation
     (applying it after this same-document one synchronously finishes).</p>

     <p>Ultimately, the navigation API gets a bit messy with overlapping cross- and same-document
     navigations, as the <a href=#ongoing-navigation-tracking>ongoing navigation tracking</a>
     machinery and APIs are built to expose only a single ongoing navigation. Web developers will be
     best-served if they do not create such overlapping situations, e.g., by <code>await</code>ing promises returned from <code id=ongoing-navigation-tracking:dom-navigation-navigate-2><a href=#dom-navigation-navigate>navigation.navigate()</a></code> before starting new
     navigations.</p>
    </div>

    <p class=note>This is a loop, since <a href=#abort-the-ongoing-navigation id=ongoing-navigation-tracking:abort-the-ongoing-navigation-2>abort the ongoing navigation</a> can run
    JavaScript (e.g., via the <code id=ongoing-navigation-tracking:event-navigateerror-4><a href=#event-navigateerror>navigateerror</a></code> event), which
    might start a new navigation. Since such a newly-started navigation will be superseded by the
    completion of this navigation, it gets signaled to the navigation API as aborted.</p>
   </ol>
  </div>

  <div data-algorithm="">
  <p>To <dfn id=inform-the-navigation-api-about-child-navigable-destruction>inform the navigation API about child navigable destruction</dfn> given a
  <a href=#navigable id=ongoing-navigation-tracking:navigable-2>navigable</a> <var>navigable</var>:</p>

  <ol><li><p><a href=#inform-the-navigation-api-about-aborting-navigation id=ongoing-navigation-tracking:inform-the-navigation-api-about-aborting-navigation>Inform the navigation API about aborting navigation</a> in
   <var>navigable</var>.<li><p>Let <var>navigation</var> be <var>navigable</var>'s <a href=#nav-window id=ongoing-navigation-tracking:nav-window-4>active
   window</a>'s <a href=#window-navigation-api id=ongoing-navigation-tracking:window-navigation-api-3>navigation API</a>.<li><p>Let <var>traversalAPIMethodTrackers</var> be a <a href=https://infra.spec.whatwg.org/#list-clone id=ongoing-navigation-tracking:list-clone data-x-internal=list-clone>clone</a> of
   <var>navigation</var>'s <a href=#upcoming-traverse-api-method-trackers id=ongoing-navigation-tracking:upcoming-traverse-api-method-trackers-4>upcoming traverse API method trackers</a>.<li><p><a href=https://infra.spec.whatwg.org/#list-iterate id=ongoing-navigation-tracking:list-iterate data-x-internal=list-iterate>For each</a> <var>apiMethodTracker</var> of
   <var>traversalAPIMethodTrackers</var>: <a href=#reject-the-finished-promise id=ongoing-navigation-tracking:reject-the-finished-promise-2>reject the finished promise</a> for
   <var>apiMethodTracker</var> with a new <a id=ongoing-navigation-tracking:aborterror-2 href=https://webidl.spec.whatwg.org/#aborterror data-x-internal=aborterror>"<code>AbortError</code>"</a>
   <code id=ongoing-navigation-tracking:domexception-3><a data-x-internal=domexception href=https://webidl.spec.whatwg.org/#dfn-DOMException>DOMException</a></code> created in <var>navigation</var>'s <a href=#concept-relevant-realm id=ongoing-navigation-tracking:concept-relevant-realm-4>relevant realm</a>.</ol>
  </div>
```

### Document latest-entry definition and complete note

<!-- selected-source-fragment:72214-72218 -->
```text
  <p>A <code id=document-state:document-14><a href=#document>Document</a></code> has a <dfn id=latest-entry>latest entry</dfn>, a <a href=#session-history-entry id=document-state:session-history-entry-5>session history entry</a> or
  null.</p>

  <p class=note>This is the entry that was most recently represented by a given
  <code id=document-state:document-15><a href=#document>Document</a></code>. A single <code id=document-state:document-16><a href=#document>Document</a></code> can represent many <a href=#session-history-entry id=document-state:session-history-entry-6>session history entries</a> over time, as many contiguous <a href=#session-history-entry id=document-state:session-history-entry-7>session history entries</a> can share the same <a href=#she-document-state id=document-state:she-document-state-5>document state</a> as explained above.</p>
```

### Traversal queue definition, both append algorithms, complete runner and queue-jumping note

<!-- selected-source-fragment:72221-72275 -->
```text
  <h5 id=centralized-modifications-of-session-history><span class=secno>7.4.1.3</span> Centralized modifications of session history<a href=#centralized-modifications-of-session-history class=self-link></a></h5>

  <p>To maintain a single source of truth, all modifications to a <a href=#traversable-navigable id=centralized-modifications-of-session-history:traversable-navigable>traversable
  navigable</a>'s <a href=#tn-session-history-entries id=centralized-modifications-of-session-history:tn-session-history-entries>session history entries</a> need
  to be synchronized. This is especially important due to how session history is influenced by all
  of the descendant <a href=#navigable id=centralized-modifications-of-session-history:navigable>navigables</a>, and thus by multiple <a href=#event-loop id=centralized-modifications-of-session-history:event-loop>event loops</a>. To accomplish this, we use the <a href=#session-history-traversal-parallel-queue id=centralized-modifications-of-session-history:session-history-traversal-parallel-queue>session history
  traversal parallel queue</a> structure.</p>

  <p id=session-history-traversal-queue>A <dfn id=session-history-traversal-parallel-queue>session history traversal parallel queue</dfn> is
  very similar to a <a href=#parallel-queue id=centralized-modifications-of-session-history:parallel-queue>parallel queue</a>. It has an <dfn id=session-history-traversal-parallel-queue-algorithm-set>algorithm set</dfn>, an <a href=https://infra.spec.whatwg.org/#ordered-set id=centralized-modifications-of-session-history:set data-x-internal=set>ordered set</a>.</p>

  <p>The <a href=https://infra.spec.whatwg.org/#list-item id=centralized-modifications-of-session-history:list-item data-x-internal=list-item>items</a> in a <a href=#session-history-traversal-parallel-queue id=centralized-modifications-of-session-history:session-history-traversal-parallel-queue-2>session history traversal parallel
  queue</a>'s <a href=#session-history-traversal-parallel-queue-algorithm-set id=centralized-modifications-of-session-history:session-history-traversal-parallel-queue-algorithm-set>algorithm
  set</a> are either algorithm steps, or <dfn id=session-history-traversal-parallel-queue-sync-nav-steps>synchronous navigation
  steps</dfn>, which are a particular brand of algorithm steps involving a <dfn id=session-history-traversal-parallel-queue-sync-nav-steps-target-nav>target
  navigable</dfn> (a <a href=#navigable id=centralized-modifications-of-session-history:navigable-2>navigable</a>).</p>

  <div data-algorithm="">
  <p>To <dfn id=tn-append-session-history-traversal-steps>append session history traversal
  steps</dfn> to a <a href=#traversable-navigable id=centralized-modifications-of-session-history:traversable-navigable-2>traversable navigable</a> <var>traversable</var> given algorithm steps
  <var>steps</var>, <a href=https://infra.spec.whatwg.org/#list-append id=centralized-modifications-of-session-history:list-append data-x-internal=list-append>append</a> <var>steps</var> to
  <var>traversable</var>'s <a href=#tn-session-history-traversal-queue id=centralized-modifications-of-session-history:tn-session-history-traversal-queue>session history
  traversal queue</a>'s <a href=#session-history-traversal-parallel-queue-algorithm-set id=centralized-modifications-of-session-history:session-history-traversal-parallel-queue-algorithm-set-2>algorithm set</a>.</p>
  </div>

  <div data-algorithm="">
  <p>To <dfn id=tn-append-session-history-sync-nav-steps>append session history synchronous
  navigation steps</dfn> involving a <a href=#navigable id=centralized-modifications-of-session-history:navigable-3>navigable</a> <var>targetNavigable</var> to a
  <a href=#traversable-navigable id=centralized-modifications-of-session-history:traversable-navigable-3>traversable navigable</a> <var>traversable</var> given algorithm steps <var>steps</var>,
  <a href=https://infra.spec.whatwg.org/#list-append id=centralized-modifications-of-session-history:list-append-2 data-x-internal=list-append>append</a> <var>steps</var> as <a href=#session-history-traversal-parallel-queue-sync-nav-steps id=centralized-modifications-of-session-history:session-history-traversal-parallel-queue-sync-nav-steps>synchronous navigation
  steps</a> targeting <a href=#session-history-traversal-parallel-queue-sync-nav-steps-target-nav id=centralized-modifications-of-session-history:session-history-traversal-parallel-queue-sync-nav-steps-target-nav>target
  navigable</a> <var>targetNavigable</var> to <var>traversable</var>'s <a href=#tn-session-history-traversal-queue id=centralized-modifications-of-session-history:tn-session-history-traversal-queue-2>session history traversal queue</a>'s <a href=#session-history-traversal-parallel-queue-algorithm-set id=centralized-modifications-of-session-history:session-history-traversal-parallel-queue-algorithm-set-3>algorithm set</a>.</p>
  </div>

  <div data-algorithm="">
  <p id=session-history-event-loop>To <dfn id=starting-a-new-session-history-traversal-parallel-queue>start a new session history traversal parallel queue</dfn>:</p>

  <ol><li><p>Let <var>sessionHistoryTraversalQueue</var> be a new <a href=#session-history-traversal-parallel-queue id=centralized-modifications-of-session-history:session-history-traversal-parallel-queue-3>session history traversal
   parallel queue</a>.<li>
    <p>Run the following steps <a href=#in-parallel id=centralized-modifications-of-session-history:in-parallel>in parallel</a>:</p>

    <ol><li>
      <p>While true:</p>

      <ol><li><p>If <var>sessionHistoryTraversalQueue</var>'s <a href=#session-history-traversal-parallel-queue-algorithm-set id=centralized-modifications-of-session-history:session-history-traversal-parallel-queue-algorithm-set-4>algorithm set</a> is
       empty, then <a id=centralized-modifications-of-session-history:continue href=https://infra.spec.whatwg.org/#iteration-continue data-x-internal=continue>continue</a>.<li><p>Let <var>steps</var> be the result of <a href=https://infra.spec.whatwg.org/#queue-dequeue id=centralized-modifications-of-session-history:dequeue data-x-internal=dequeue>dequeuing</a> from
       <var>sessionHistoryTraversalQueue</var>'s <a href=#session-history-traversal-parallel-queue-algorithm-set id=centralized-modifications-of-session-history:session-history-traversal-parallel-queue-algorithm-set-5>algorithm
       set</a>.<li><p>Run <var>steps</var>.</ol>
     </ol>
   <li><p>Return <var>sessionHistoryTraversalQueue</var>.</ol>
  </div>

  <p id=sync-navigation-steps-queue-jumping-examples><a href=#session-history-traversal-parallel-queue-sync-nav-steps id=centralized-modifications-of-session-history:session-history-traversal-parallel-queue-sync-nav-steps-2>Synchronous navigation
  steps</a> are tagged in the <a href=#session-history-traversal-parallel-queue-algorithm-set id=centralized-modifications-of-session-history:session-history-traversal-parallel-queue-algorithm-set-6>algorithm set</a> to allow
  them to conditionally "jump the queue". This is handled <a href=#sync-navigations-jump-queue>within apply the history step</a>.</p>
```

### Complete session-history entries lookup algorithm

<!-- selected-source-fragment:72399-72419 -->
```text
  <div data-algorithm="">
  <p>To <dfn id=getting-session-history-entries>get session history entries</dfn> of a
  <a href=#navigable id=low-level-operations-on-session-history:navigable>navigable</a> <var>navigable</var>:</p>

  <ol><li><p>Let <var>traversable</var> be <var>navigable</var>'s <a href=#nav-traversable id=low-level-operations-on-session-history:nav-traversable>traversable navigable</a>.<li><p><a id=low-level-operations-on-session-history:assert href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: this is running within <var>traversable</var>'s <a href=#tn-session-history-traversal-queue id=low-level-operations-on-session-history:tn-session-history-traversal-queue>session history traversal queue</a>.<li><p>If <var>navigable</var> is <var>traversable</var>, return <var>traversable</var>'s <a href=#tn-session-history-entries id=low-level-operations-on-session-history:tn-session-history-entries>session history entries</a>.<li><p>Let <var>docStates</var> be an empty <a href=https://infra.spec.whatwg.org/#ordered-set id=low-level-operations-on-session-history:set data-x-internal=set>ordered set</a> of <a href=#document-state-2 id=low-level-operations-on-session-history:document-state-2>document states</a>.<li><p>For each <var>entry</var> of <var>traversable</var>'s <a href=#tn-session-history-entries id=low-level-operations-on-session-history:tn-session-history-entries-2>session history entries</a>, <a href=https://infra.spec.whatwg.org/#set-append id=low-level-operations-on-session-history:set-append data-x-internal=set-append>append</a> <var>entry</var>'s <a href=#she-document-state id=low-level-operations-on-session-history:she-document-state>document state</a>
   to <var>docStates</var>.<li>
    <p><a href=https://infra.spec.whatwg.org/#list-iterate id=low-level-operations-on-session-history:list-iterate data-x-internal=list-iterate>For each</a> <var>docState</var> of <var>docStates</var>:</p>

    <ol><li>
      <p><a href=https://infra.spec.whatwg.org/#list-iterate id=low-level-operations-on-session-history:list-iterate-2 data-x-internal=list-iterate>For each</a> <var>nestedHistory</var> of
      <var>docState</var>'s <a href=#document-state-nested-histories id=low-level-operations-on-session-history:document-state-nested-histories>nested
      histories</a>:</p>

      <ol><li><p>If <var>nestedHistory</var>'s <a href=#nested-history-id id=low-level-operations-on-session-history:nested-history-id>id</a> equals
       <var>navigable</var>'s <a href=#nav-id id=low-level-operations-on-session-history:nav-id>id</a>, return <var>nestedHistory</var>'s
       <a href=#nested-history-entries id=low-level-operations-on-session-history:nested-history-entries>entries</a>.<li><p>For each <var>entry</var> of <var>nestedHistory</var>'s <a href=#nested-history-entries id=low-level-operations-on-session-history:nested-history-entries-2>entries</a>, <a href=https://infra.spec.whatwg.org/#set-append id=low-level-operations-on-session-history:set-append-2 data-x-internal=set-append>append</a>
       <var>entry</var>'s <a href=#she-document-state id=low-level-operations-on-session-history:she-document-state-2>document state</a> to
       <var>docStates</var>.</ol>
     </ol>
   <li><p><a id=low-level-operations-on-session-history:assert-2 href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: this step is not reached.</ol>
  </div>
```

### Complete all-used-history-steps algorithm

<!-- selected-source-fragment:72473-72491 -->
```text
  <div data-algorithm="">
  <p>To <dfn id=getting-all-used-history-steps>get all used history steps</dfn> that are part
  of <a href=#traversable-navigable id=low-level-operations-on-session-history:traversable-navigable-2>traversable navigable</a> <var>traversable</var>:</p>

  <ol><li><p><a id=low-level-operations-on-session-history:assert-4 href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: this is running within <var>traversable</var>'s <a href=#tn-session-history-traversal-queue id=low-level-operations-on-session-history:tn-session-history-traversal-queue-3>session history traversal queue</a>.<li><p>Let <var>steps</var> be an empty <a href=https://infra.spec.whatwg.org/#ordered-set id=low-level-operations-on-session-history:set-3 data-x-internal=set>ordered set</a> of non-negative
   integers.<li><p>Let <var>entryLists</var> be the <a href=https://infra.spec.whatwg.org/#ordered-set id=low-level-operations-on-session-history:set-4 data-x-internal=set>ordered set</a> «
   <var>traversable</var>'s <a href=#tn-session-history-entries id=low-level-operations-on-session-history:tn-session-history-entries-4>session history
   entries</a> ».<li>
    <p><a href=https://infra.spec.whatwg.org/#list-iterate id=low-level-operations-on-session-history:list-iterate-6 data-x-internal=list-iterate>For each</a> <var>entryList</var> of
    <var>entryLists</var>:</p>

    <ol><li>
      <p><a href=https://infra.spec.whatwg.org/#list-iterate id=low-level-operations-on-session-history:list-iterate-7 data-x-internal=list-iterate>For each</a> <var>entry</var> of <var>entryList</var>:</p>

      <ol><li><p><a href=https://infra.spec.whatwg.org/#set-append id=low-level-operations-on-session-history:set-append-4 data-x-internal=set-append>Append</a> <var>entry</var>'s <a href=#she-step id=low-level-operations-on-session-history:she-step-3>step</a> to <var>steps</var>.<li><p><a href=https://infra.spec.whatwg.org/#list-iterate id=low-level-operations-on-session-history:list-iterate-8 data-x-internal=list-iterate>For each</a> <var>nestedHistory</var> of
       <var>entry</var>'s <a href=#she-document-state id=low-level-operations-on-session-history:she-document-state-7>document state</a>'s <a href=#document-state-nested-histories id=low-level-operations-on-session-history:document-state-nested-histories-3>nested histories</a>, <a href=https://infra.spec.whatwg.org/#set-append id=low-level-operations-on-session-history:set-append-5 data-x-internal=set-append>append</a> <var>nestedHistory</var>'s <a href=#nested-history-entries id=low-level-operations-on-session-history:nested-history-entries-4>entries list</a> to <var>entryLists</var>.</ol>
     </ol>
   <li><p>Return <var>steps</var>, <a href=https://infra.spec.whatwg.org/#list-sort-in-ascending-order id=low-level-operations-on-session-history:list-sort data-x-internal=list-sort>sorted</a>.</ol>
  </div>
```

### Complete creation/destruction history-update caller

<!-- selected-source-fragment:74141-74147 -->
```text
  <div data-algorithm="">
  <p>To <dfn id=update-for-navigable-creation/destruction>update for navigable creation/destruction</dfn> given a <a href=#traversable-navigable id=updating-the-traversable:traversable-navigable-3>traversable
  navigable</a> <var>traversable</var>:</p>

  <ol><li><p>Let <var>step</var> be <var>traversable</var>'s <a href=#tn-current-session-history-step id=updating-the-traversable:tn-current-session-history-step>current session history step</a>.<li><p>Return the result of <a href=#apply-the-history-step id=updating-the-traversable:apply-the-history-step-3>applying the history
   step</a> <var>step</var> to <var>traversable</var> given false, null, null, "<code id=updating-the-traversable:uni-none><a href=#uni-none>none</a></code>", and null.</ol>
  </div>
```

### Complete apply-history-step algorithm, all branches

<!-- selected-source-fragment:74206-74487 -->
```text
  <div data-algorithm="">
  <p id=update-the-session-history-with-the-new-page>To <dfn id=apply-the-history-step>apply the history step</dfn> given a
  non-negative integer <var>step</var> to a <a href=#traversable-navigable id=updating-the-traversable:traversable-navigable-8>traversable navigable</a>
  <var>traversable</var>, with boolean <var>checkForCancelation</var>, <a href=#source-snapshot-params id=updating-the-traversable:source-snapshot-params-4>source snapshot
  params</a>-or-null <var>sourceSnapshotParams</var>, <a href=#navigable id=updating-the-traversable:navigable-7>navigable</a>-or-null
  <var>initiatorToCheck</var>, <a href=#user-navigation-involvement id=updating-the-traversable:user-navigation-involvement-5>user navigation involvement</a> <var>userInvolvement</var>,
  and <code id=updating-the-traversable:navigationtype><a href=#navigationtype>NavigationType</a></code>-or-null <var>navigationType</var>, perform the following steps.
  They return "<code>initiator-disallowed</code>", "<code>canceled-by-beforeunload</code>", "<code>canceled-by-navigate</code>", or
  "<code>applied</code>".</p>

  <ol><li><p><a id=updating-the-traversable:assert href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: This is running within <var>traversable</var>'s <a href=#tn-session-history-traversal-queue id=updating-the-traversable:tn-session-history-traversal-queue>session history traversal queue</a>.<li><p>Let <var>targetStep</var> be the result of <a href=#getting-the-used-step id=updating-the-traversable:getting-the-used-step>getting the used step</a> given
   <var>traversable</var> and <var>step</var>.<li>
    <p>If <var>initiatorToCheck</var> is not null:</p>

    <ol><li><p><a id=updating-the-traversable:assert-2 href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: <var>sourceSnapshotParams</var> is not null.<li><p><a href=https://infra.spec.whatwg.org/#list-iterate id=updating-the-traversable:list-iterate data-x-internal=list-iterate>For each</a> <var>navigable</var> of <a href=#get-all-navigables-whose-current-session-history-entry-will-change-or-reload id=updating-the-traversable:get-all-navigables-whose-current-session-history-entry-will-change-or-reload>get all
     navigables whose current session history entry will change or reload</a>: if
     <var>initiatorToCheck</var> is not <a href=#allowed-to-navigate id=updating-the-traversable:allowed-to-navigate>allowed by sandboxing to navigate</a>
     <var>navigable</var> given <var>sourceSnapshotParams</var>, then return "<code>initiator-disallowed</code>".</ol>
   <li><p>Let <var>navigablesCrossingDocuments</var> be the result of <a href=#getting-all-navigables-that-might-experience-a-cross-document-traversal id=updating-the-traversable:getting-all-navigables-that-might-experience-a-cross-document-traversal>getting all navigables
   that might experience a cross-document traversal</a> given <var>traversable</var> and
   <var>targetStep</var>.<li><p>If <var>checkForCancelation</var> is true, and the result of <a href=#checking-if-unloading-is-canceled id=updating-the-traversable:checking-if-unloading-is-canceled>checking if unloading
   is canceled</a> given <var>navigablesCrossingDocuments</var>, <var>traversable</var>,
   <var>targetStep</var>, and <var>userInvolvement</var> is not "<code>continue</code>",
   then return that result.<li><p>Let <var>changingNavigables</var> be the result of <a href=#get-all-navigables-whose-current-session-history-entry-will-change-or-reload id=updating-the-traversable:get-all-navigables-whose-current-session-history-entry-will-change-or-reload-2>get all navigables whose current
   session history entry will change or reload</a> given <var>traversable</var> and
   <var>targetStep</var>.<li><p>Let <var>nonchangingNavigablesThatStillNeedUpdates</var> be the result of <a href=#getting-all-navigables-that-only-need-history-object-length/index-update id=updating-the-traversable:getting-all-navigables-that-only-need-history-object-length/index-update>getting
   all navigables that only need history object length/index update</a> given
   <var>traversable</var> and <var>targetStep</var>.<li>
    <p><a href=https://infra.spec.whatwg.org/#list-iterate id=updating-the-traversable:list-iterate-2 data-x-internal=list-iterate>For each</a> <var>navigable</var> of
    <var>changingNavigables</var>:</p>

    <ol><li><p>Let <var>targetEntry</var> be the result of <a href=#getting-the-target-history-entry id=updating-the-traversable:getting-the-target-history-entry>getting the target history
     entry</a> given <var>navigable</var> and <var>targetStep</var>.<li><p>Set <var>navigable</var>'s <a href=#nav-current-history-entry id=updating-the-traversable:nav-current-history-entry>current session
     history entry</a> to <var>targetEntry</var>.<li>
      <p>If <var>targetEntry</var>'s <a href=#she-document id=updating-the-traversable:she-document>document</a> is not
      <var>navigable</var>'s <a href=#nav-document id=updating-the-traversable:nav-document>active document</a>, then <a href=#queue-a-global-task id=updating-the-traversable:queue-a-global-task>queue a
      global task</a> on the <a href=#navigation-and-traversal-task-source id=updating-the-traversable:navigation-and-traversal-task-source>navigation and traversal task source</a> of
      <var>navigable</var>'s <a href=#nav-window id=updating-the-traversable:nav-window>active window</a> to run these steps:</p>

      <ol><li><p>Let <var>navigation</var> be <var>navigable</var>'s <a href=#nav-window id=updating-the-traversable:nav-window-2>active
       window</a>'s <a href=#window-navigation-api id=updating-the-traversable:window-navigation-api>navigation API</a>.<li><p>Set <var>navigation</var>'s <a href=#ongoing-navigate-event id=updating-the-traversable:ongoing-navigate-event>ongoing <code>navigate</code> event</a> to null.<li><p>Set <var>navigation</var>'s <a href=#ongoing-api-method-tracker id=updating-the-traversable:ongoing-api-method-tracker>ongoing API method tracker</a> to null.</ol>

      <p class=note>This prevents the <code id=updating-the-traversable:event-navigateerror><a href=#event-navigateerror>navigateerror</a></code>
      event and the corresponding <a href=#dom-navigateevent-signal id=updating-the-traversable:dom-navigateevent-signal>signal</a> from firing
      as a result of a successful cross-document navigation.</p>
     <li><p>Set <var>navigable</var>'s <a href=#ongoing-navigation id=updating-the-traversable:ongoing-navigation>ongoing navigation</a> to "<code>traversal</code>".</ol>
   <li><p>Let <var>totalChangeJobs</var> be the <a href=https://infra.spec.whatwg.org/#list-size id=updating-the-traversable:list-size data-x-internal=list-size>size</a> of
   <var>changingNavigables</var>.<li><p>Let <var>completedChangeJobs</var> be 0.<li>
    <p>Let <var>changingNavigableContinuations</var> be an empty <a id=updating-the-traversable:queue href=https://infra.spec.whatwg.org/#queue data-x-internal=queue>queue</a> of <a href=#changing-navigable-continuation-state id=updating-the-traversable:changing-navigable-continuation-state>changing navigable continuation
    states</a>.</p>

    <p class=note>This queue is used to split the operations on <var>changingNavigables</var>
    into two parts. Specifically, <var>changingNavigableContinuations</var> holds data for the <a href=#continuations-dequeue>second part</a>.</p>
   <li>
    <p><a href=https://infra.spec.whatwg.org/#list-iterate id=updating-the-traversable:list-iterate-3 data-x-internal=list-iterate>For each</a> <var>navigable</var> of
    <var>changingNavigables</var>, <a href=#queue-a-global-task id=updating-the-traversable:queue-a-global-task-2>queue a global task</a> on the <a href=#navigation-and-traversal-task-source id=updating-the-traversable:navigation-and-traversal-task-source-2>navigation and
    traversal task source</a> of <var>navigable</var>'s <a href=#nav-window id=updating-the-traversable:nav-window-3>active
    window</a> to run the steps:</p>

    <p class=note>This set of steps are split into two parts to allow synchronous navigations to
    be processed before documents unload. State is stored in
    <var>changingNavigableContinuations</var> for the <a href=#continuations-dequeue>second
    part</a>.</p>

    <ol><li><p>Let <var>displayedEntry</var> be <var>navigable</var>'s <a href=#nav-active-history-entry id=updating-the-traversable:nav-active-history-entry>active session history entry</a>.<li><p>Let <var>targetEntry</var> be <var>navigable</var>'s <a href=#nav-current-history-entry id=updating-the-traversable:nav-current-history-entry-2>current session history entry</a>.<li>
      <p>Let <var>changingNavigableContinuation</var> be a <a href=#changing-navigable-continuation-state id=updating-the-traversable:changing-navigable-continuation-state-2>changing navigable continuation
      state</a> with:</p>

      <dl class=props><dt><a href=#changing-nav-continuation-displayed-document id=updating-the-traversable:changing-nav-continuation-displayed-document>displayed document</a><dd><var>displayedEntry</var>'s <a href=#she-document id=updating-the-traversable:she-document-2>document</a><dt><a href=#changing-nav-continuation-target-entry id=updating-the-traversable:changing-nav-continuation-target-entry>target entry</a><dd><var>targetEntry</var><dt><a href=#changing-nav-continuation-navigable id=updating-the-traversable:changing-nav-continuation-navigable>navigable</a><dd><var>navigable</var><dt><a href=#changing-nav-continuation-update-only id=updating-the-traversable:changing-nav-continuation-update-only>update-only</a><dd>false</dl>
     <li>
      <p>If <var>displayedEntry</var> is <var>targetEntry</var> and <var>targetEntry</var>'s <a href=#she-document-state id=updating-the-traversable:she-document-state>document state</a>'s <a href=#document-state-reload-pending id=updating-the-traversable:document-state-reload-pending>reload pending</a> is false:</p>

      <ol><li><p>Set <var>changingNavigableContinuation</var>'s <a href=#changing-nav-continuation-update-only id=updating-the-traversable:changing-nav-continuation-update-only-2>update-only</a> to true.<li><p><a id=updating-the-traversable:enqueue href=https://infra.spec.whatwg.org/#queue-enqueue data-x-internal=enqueue>Enqueue</a> <var>changingNavigableContinuation</var> on
       <var>changingNavigableContinuations</var>.<li><p>Abort these steps.</ol>

      <p class=note>This case occurs due to a <a href=#finalize-a-same-document-navigation id=updating-the-traversable:finalize-a-same-document-navigation>synchronous navigation</a> which already updated the <a href=#nav-active-history-entry id=updating-the-traversable:nav-active-history-entry-2>active session history entry</a>.</p>
     <li>
      <p>Switch on <var>navigationType</var>:</p>

      <dl class=switch><dt>"<code id=updating-the-traversable:dom-navigationtype-reload-2><a href=#dom-navigationtype-reload>reload</a></code>"<dd><p><a id=updating-the-traversable:assert-3 href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: <var>targetEntry</var>'s <a href=#she-document-state id=updating-the-traversable:she-document-state-2>document state</a>'s <a href=#document-state-reload-pending id=updating-the-traversable:document-state-reload-pending-2>reload pending</a> is true.<dt>"<code id=updating-the-traversable:dom-navigationtype-traverse-3><a href=#dom-navigationtype-traverse>traverse</a></code>"<dd><p><a id=updating-the-traversable:assert-4 href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: <var>targetEntry</var>'s
       <a href=#she-document-state id=updating-the-traversable:she-document-state-3>document state</a>'s <a href=#document-state-ever-populated id=updating-the-traversable:document-state-ever-populated>ever populated</a> is true.<dt>"<code id=updating-the-traversable:dom-navigationtype-replace><a href=#dom-navigationtype-replace>replace</a></code>"<dd><p><a id=updating-the-traversable:assert-5 href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: <var>targetEntry</var>'s <a href=#she-step id=updating-the-traversable:she-step>step</a>
       is <var>displayedEntry</var>'s <a href=#she-step id=updating-the-traversable:she-step-2>step</a> and
       <var>targetEntry</var>'s <a href=#she-document-state id=updating-the-traversable:she-document-state-4>document state</a>'s <a href=#document-state-ever-populated id=updating-the-traversable:document-state-ever-populated-2>ever populated</a> is false.<dt>"<code id=updating-the-traversable:dom-navigationtype-push><a href=#dom-navigationtype-push>push</a></code>"<dd><p><a id=updating-the-traversable:assert-6 href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: <var>targetEntry</var>'s <a href=#she-step id=updating-the-traversable:she-step-3>step</a> is
       <var>displayedEntry</var>'s <a href=#she-step id=updating-the-traversable:she-step-4>step</a> + 1 and
       <var>targetEntry</var>'s <a href=#she-document-state id=updating-the-traversable:she-document-state-5>document state</a>'s <a href=#document-state-ever-populated id=updating-the-traversable:document-state-ever-populated-3>ever populated</a> is false.</dl>
     <li><p>Let <var>oldOrigin</var> be <var>targetEntry</var>'s <a href=#she-document-state id=updating-the-traversable:she-document-state-6>document state</a>'s <a href=#document-state-origin id=updating-the-traversable:document-state-origin>origin</a>.<li id=descendant-navigable-traversal-navigate-events>
      <p>If all of the following are true:</p>

      <ul><li><p><var>navigationType</var> is "<code id=updating-the-traversable:dom-navigationtype-traverse-4><a href=#dom-navigationtype-traverse>traverse</a></code>";<li><p><var>navigable</var> is not <var>traversable</var>;<li><p><var>targetEntry</var> is not <var>displayedEntry</var>; and<li><p><var>oldOrigin</var> is the <a href=#same-origin id=updating-the-traversable:same-origin>same</a> as
       <var>displayedEntry</var>'s <a href=#she-document-state id=updating-the-traversable:she-document-state-7>document state</a>'s <a href=#document-state-origin id=updating-the-traversable:document-state-origin-2>origin</a>,</ul>

      <p>then:</p>

      <ol><li><p>Let <var>navigation</var> be <var>navigable</var>'s <a href=#nav-window id=updating-the-traversable:nav-window-4>active window</a>'s <a href=#window-navigation-api id=updating-the-traversable:window-navigation-api-2>navigation API</a>.<li><p><a href=#fire-a-traverse-navigate-event id=updating-the-traversable:fire-a-traverse-navigate-event>Fire a traverse <code>navigate</code> event</a> at
       <var>navigation</var> given <var>targetEntry</var> and <var>userInvolvement</var>.</ol>
     <li>
      <p>If <var>targetEntry</var>'s <a href=#she-document id=updating-the-traversable:she-document-3>document</a> is null, or
      <var>targetEntry</var>'s <a href=#she-document-state id=updating-the-traversable:she-document-state-8>document state</a>'s <a href=#document-state-reload-pending id=updating-the-traversable:document-state-reload-pending-3>reload pending</a> is true:</p>

      <ol><li><p>Let <var>navTimingType</var> be "<code id=updating-the-traversable:dom-navigationtimingtype-back_forward><a data-x-internal=dom-navigationtimingtype-back_forward href=https://w3c.github.io/navigation-timing/#dom-navigationtimingtype-back_forward>back_forward</a></code>" if
       <var>targetEntry</var>'s <a href=#she-document id=updating-the-traversable:she-document-4>document</a> is null; otherwise
       "<code id=updating-the-traversable:dom-navigationtimingtype-reload><a data-x-internal=dom-navigationtimingtype-reload href=https://w3c.github.io/navigation-timing/#dom-navigationtimingtype-reload>reload</a></code>".<li><p>Let <var>targetSnapshotParams</var> be the result of <a href=#snapshotting-target-snapshot-params id=updating-the-traversable:snapshotting-target-snapshot-params>snapshotting target
       snapshot params</a> given <var>navigable</var>.<li><p>Let <var>potentiallyTargetSpecificSourceSnapshotParams</var> be
       <var>sourceSnapshotParams</var>.<li>
        <p>If <var>potentiallyTargetSpecificSourceSnapshotParams</var> is null, then set it to the
        result of <a href=#snapshotting-source-snapshot-params id=updating-the-traversable:snapshotting-source-snapshot-params>snapshotting source snapshot params</a> given <var>navigable</var>'s
        <a href=#nav-document id=updating-the-traversable:nav-document-2>active document</a>.</p>

        <p class=note>In this case there is no clear source of the traversal/reload. We treat this
        situation as if <var>navigable</var> navigated itself, but note that some properties of
        <var>targetEntry</var>'s original initiator are preserved in <var>targetEntry</var>'s <a href=#she-document-state id=updating-the-traversable:she-document-state-9>document state</a>, such as the <a href=#document-state-initiator-origin id=updating-the-traversable:document-state-initiator-origin>initiator origin</a> and <a href=#document-state-request-referrer id=updating-the-traversable:document-state-request-referrer>referrer</a>, which will appropriately influence
        the navigation.</p>
       <li><p>Set <var>targetEntry</var>'s <a href=#she-document-state id=updating-the-traversable:she-document-state-10>document
       state</a>'s <a href=#document-state-reload-pending id=updating-the-traversable:document-state-reload-pending-4>reload pending</a> to
       false.<li><p>Let <var>allowPOST</var> be <var>targetEntry</var>'s <a href=#she-document-state id=updating-the-traversable:she-document-state-11>document state</a>'s <a href=#document-state-reload-pending id=updating-the-traversable:document-state-reload-pending-5>reload pending</a>.</p>

       <li><p><a href=#in-parallel id=updating-the-traversable:in-parallel>In parallel</a>, <a href="#attempt-to-populate-the-history-entry's-document" id="updating-the-traversable:attempt-to-populate-the-history-entry's-document">attempt to populate the history entry's
       document</a> for <var>targetEntry</var>, given <var>navigable</var>,
       <var>potentiallyTargetSpecificSourceSnapshotParams</var>, <var>targetSnapshotParams</var>,
       <var>userInvolvement</var>, with <i id=updating-the-traversable:attempt-to-populate-allow-post><a href=#attempt-to-populate-allow-post>allowPOST</a></i> set
       to <var>allowPOST</var> and <a href=#attempt-to-populate-completion-steps id=updating-the-traversable:attempt-to-populate-completion-steps><i>completionSteps</i></a> set to
       <a href=#queue-a-global-task id=updating-the-traversable:queue-a-global-task-3>queue a global task</a> on the <a href=#navigation-and-traversal-task-source id=updating-the-traversable:navigation-and-traversal-task-source-3>navigation and traversal task source</a>
       given <var>navigable</var>'s <a href=#nav-window id=updating-the-traversable:nav-window-5>active window</a> to run
       <var>afterDocumentPopulated</var>.</ol>

      <p>Otherwise, run <var>afterDocumentPopulated</var> <a href=#immediately id=updating-the-traversable:immediately>immediately</a>.</p>

      <p>In both cases, let <var>afterDocumentPopulated</var> be the following steps:</p>

      <ol><li>
        <p>If <var>targetEntry</var>'s <a href=#she-document id=updating-the-traversable:she-document-5>document</a> is null, then
        set <var>changingNavigableContinuation</var>'s <a href=#changing-nav-continuation-update-only id=updating-the-traversable:changing-nav-continuation-update-only-3>update-only</a> to true.</p>

        <p class=note>This means we tried to populate the document, but were unable to do so,
        e.g. because of the server returning a 204.</p>

        <div class=note>
         <p>These kinds of failed navigations or traversals will not be signaled to the <a href=#navigation-api>navigation API</a> (e.g., through the promises of any
         <a href=#navigation-api-method-tracker id=updating-the-traversable:navigation-api-method-tracker>navigation API method tracker</a>, or the <code id=updating-the-traversable:event-navigateerror-2><a href=#event-navigateerror>navigateerror</a></code> event). Doing so would leak information
         about the timing of responses from other origins, in the cross-origin case, and providing
         different results in the cross-origin vs. same-origin cases was deemed too confusing.</p>

         <p>However, implementations could use this opportunity to clear any promise handlers for
         the <code id=updating-the-traversable:dom-navigationtransition-finished><a href=#dom-navigationtransition-finished>navigation.transition.finished</a></code>
         promise, as they are guaranteed at this point to never run. And, they might wish to
         <a id=updating-the-traversable:report-a-warning-to-the-console href=https://console.spec.whatwg.org/#report-a-warning-to-the-console data-x-internal=report-a-warning-to-the-console>report a warning to the console</a> if any part of the navigation API initiated
         these navigations, to make it clear to the web developer why their promises will never
         settle and events will never fire.</p>
        </div>
       <li>
        <p>If <var>targetEntry</var>'s <a href=#she-document id=updating-the-traversable:she-document-6>document</a>'s <a href=https://dom.spec.whatwg.org/#concept-document-origin id=updating-the-traversable:concept-document-origin data-x-internal=concept-document-origin>origin</a> is not <a href=#same-origin id=updating-the-traversable:same-origin-2>same origin</a> with
        <var>oldOrigin</var>, then set <var>targetEntry</var>'s <a href=#she-classic-history-api-state id=updating-the-traversable:she-classic-history-api-state>classic history API state</a> to
        <a href=#structuredserializeforstorage id=updating-the-traversable:structuredserializeforstorage>StructuredSerializeForStorage</a>(null).</p>

        <p class=note>This clears history state when the origin changed vs a previous load of
        <var>targetEntry</var> without a redirect occuring. This can happen due to a change in CSP
        sandbox headers.</p>
       <li>
        <p>If all of the following are true:</p>

        <ul><li><p><var>navigable</var>'s <a href=#nav-parent id=updating-the-traversable:nav-parent>parent</a> is null;<li><p><var>targetEntry</var>'s <a href=#she-document id=updating-the-traversable:she-document-7>document</a>'s <a href=#concept-document-bc id=updating-the-traversable:concept-document-bc>browsing context</a> is not an <a href=#auxiliary-browsing-context id=updating-the-traversable:auxiliary-browsing-context>auxiliary browsing
         context</a> whose <a href=#opener-browsing-context id=updating-the-traversable:opener-browsing-context>opener browsing context</a> is non-null; and<li><p><var>targetEntry</var>'s <a href=#she-document id=updating-the-traversable:she-document-8>document</a>'s <a href=https://dom.spec.whatwg.org/#concept-document-origin id=updating-the-traversable:concept-document-origin-2 data-x-internal=concept-document-origin>origin</a> is not <a href=#same-origin id=updating-the-traversable:same-origin-3>same origin</a> with
         <var>oldOrigin</var>,</ul>

        <p>then set <var>targetEntry</var>'s <a href=#she-document-state id=updating-the-traversable:she-document-state-12>document
        state</a>'s <a href=#document-state-nav-target-name id=updating-the-traversable:document-state-nav-target-name>navigable target name</a>
        to the empty string.</p>
       <li>
        <p><a id=updating-the-traversable:enqueue-2 href=https://infra.spec.whatwg.org/#queue-enqueue data-x-internal=enqueue>Enqueue</a> <var>changingNavigableContinuation</var> on
        <var>changingNavigableContinuations</var>.</p>

        <p class=note>The rest of this job <a href=#continuations-dequeue>runs later</a> in
        this algorithm.</p>
       </ol>
     </ol>
   <li><p>Let <var>navigablesThatMustWaitBeforeHandlingSyncNavigation</var> be an empty
   <a id=updating-the-traversable:set href=https://infra.spec.whatwg.org/#ordered-set data-x-internal=set>set</a>.<li>
    <p>While <var>completedChangeJobs</var> does not equal <var>totalChangeJobs</var>:</p>

    <ol><li>
      <p>If <var>traversable</var>'s <a href=#tn-running-nested-apply-history-step id=updating-the-traversable:tn-running-nested-apply-history-step>running
      nested apply history step</a> is false:</p>

      <ol><li id=sync-navigations-jump-queue>
        <p>While <var>traversable</var>'s <a href=#tn-session-history-traversal-queue id=updating-the-traversable:tn-session-history-traversal-queue-2>session
        history traversal queue</a>'s <a href=#session-history-traversal-parallel-queue-algorithm-set id=updating-the-traversable:session-history-traversal-parallel-queue-algorithm-set>algorithm set</a> <a href=https://infra.spec.whatwg.org/#list-contain id=updating-the-traversable:list-contains data-x-internal=list-contains>contains</a> one or more <a href=#session-history-traversal-parallel-queue-sync-nav-steps id=updating-the-traversable:session-history-traversal-parallel-queue-sync-nav-steps>synchronous navigation
        steps</a> with a <a href=#session-history-traversal-parallel-queue-sync-nav-steps-target-nav id=updating-the-traversable:session-history-traversal-parallel-queue-sync-nav-steps-target-nav>target
        navigable</a> not <a href=https://infra.spec.whatwg.org/#list-contain id=updating-the-traversable:list-contains-2 data-x-internal=list-contains>contained</a> in
        <var>navigablesThatMustWaitBeforeHandlingSyncNavigation</var>:</p>

        <ol><li><p>Let <var>steps</var> be the first <a href=https://infra.spec.whatwg.org/#list-item id=updating-the-traversable:list-item data-x-internal=list-item>item</a> in
         <var>traversable</var>'s <a href=#tn-session-history-traversal-queue id=updating-the-traversable:tn-session-history-traversal-queue-3>session history
         traversal queue</a>'s <a href=#session-history-traversal-parallel-queue-algorithm-set id=updating-the-traversable:session-history-traversal-parallel-queue-algorithm-set-2>algorithm set</a> that
         is <a href=#session-history-traversal-parallel-queue-sync-nav-steps id=updating-the-traversable:session-history-traversal-parallel-queue-sync-nav-steps-2>synchronous
         navigation steps</a> with a <a href=#session-history-traversal-parallel-queue-sync-nav-steps-target-nav id=updating-the-traversable:session-history-traversal-parallel-queue-sync-nav-steps-target-nav-2>target
         navigable</a> not <a href=https://infra.spec.whatwg.org/#list-contain id=updating-the-traversable:list-contains-3 data-x-internal=list-contains>contained</a> in
         <var>navigablesThatMustWaitBeforeHandlingSyncNavigation</var>.<li><p><a href=https://infra.spec.whatwg.org/#list-remove id=updating-the-traversable:list-remove data-x-internal=list-remove>Remove</a> <var>steps</var> from
         <var>traversable</var>'s <a href=#tn-session-history-traversal-queue id=updating-the-traversable:tn-session-history-traversal-queue-4>session history
         traversal queue</a>'s <a href=#session-history-traversal-parallel-queue-algorithm-set id=updating-the-traversable:session-history-traversal-parallel-queue-algorithm-set-3>algorithm
         set</a>.<li><p>Set <var>traversable</var>'s <a href=#tn-running-nested-apply-history-step id=updating-the-traversable:tn-running-nested-apply-history-step-2>running nested apply history step</a> to
         true.<li><p>Run <var>steps</var>.<li><p>Set <var>traversable</var>'s <a href=#tn-running-nested-apply-history-step id=updating-the-traversable:tn-running-nested-apply-history-step-3>running nested apply history step</a> to
         false.</ol>

        <p class=note>Synchronous navigations that are intended to take place before this
        traversal jump the queue at this point, so they can be added to the correct place in
        <var>traversable</var>'s <a href=#tn-session-history-entries id=updating-the-traversable:tn-session-history-entries>session history
        entries</a> before this traversal potentially unloads their document. <a href=#sync-navigation-steps-queue-jumping-examples>More details can be found
        here</a>.</p>
       </ol>
     <li id=continuations-dequeue><p>Let <var>changingNavigableContinuation</var> be the result
     of <a href=https://infra.spec.whatwg.org/#queue-dequeue id=updating-the-traversable:dequeue data-x-internal=dequeue>dequeuing</a> from
     <var>changingNavigableContinuations</var>.<li><p>If <var>changingNavigableContinuation</var> is nothing, then
     <a id=updating-the-traversable:continue href=https://infra.spec.whatwg.org/#iteration-continue data-x-internal=continue>continue</a>.<li><p>Let <var>displayedDocument</var> be <var>changingNavigableContinuation</var>'s <a href=#changing-nav-continuation-displayed-document id=updating-the-traversable:changing-nav-continuation-displayed-document-2>displayed document</a>.<li><p>Let <var>targetEntry</var> be <var>changingNavigableContinuation</var>'s <a href=#changing-nav-continuation-target-entry id=updating-the-traversable:changing-nav-continuation-target-entry-2>target entry</a>.<li><p>Let <var>navigable</var> be <var>changingNavigableContinuation</var>'s <a href=#changing-nav-continuation-navigable id=updating-the-traversable:changing-nav-continuation-navigable-2>navigable</a>.<li>
      <p>Let (<var>scriptHistoryLength</var>, <var>scriptHistoryIndex</var>) be the result of
      <a href=#getting-the-history-object-length-and-index id=updating-the-traversable:getting-the-history-object-length-and-index>getting the history object length and index</a> given <var>traversable</var> and
      <var>targetStep</var>.</p>

      <p class=note>These values might have changed since they were last calculated.</p>
     <li>
      <p><a href=https://infra.spec.whatwg.org/#list-append id=updating-the-traversable:list-append data-x-internal=list-append>Append</a> <var>navigable</var> to
      <var>navigablesThatMustWaitBeforeHandlingSyncNavigation</var>.</p>

      <p class=note>Once a navigable has reached this point in traversal, additionally queued
      synchronous navigation steps are likely to be intended to occur after this traversal rather
      than before it, so they no longer jump the queue. <a href=#sync-navigation-steps-queue-jumping-examples>More details can be found here</a>.</p>
     <li><p>Let <var>entriesForNavigationAPI</var> be the result of <a href=#getting-session-history-entries-for-the-navigation-api id=updating-the-traversable:getting-session-history-entries-for-the-navigation-api>getting session history
     entries for the navigation API</a> given <var>navigable</var> and
     <var>targetStep</var>.<li>
      <p>If <var>changingNavigableContinuation</var>'s <a href=#changing-nav-continuation-update-only id=updating-the-traversable:changing-nav-continuation-update-only-4>update-only</a> is true, or
      <var>targetEntry</var>'s <a href=#she-document id=updating-the-traversable:she-document-9>document</a> is
      <var>displayedDocument</var>:</p>

      <p class=note>This is a same-document navigation: we proceed without unloading.</p>

      <ol><li>
        <p>Set <var>navigable</var>'s <a href=#ongoing-navigation id=updating-the-traversable:ongoing-navigation-2>ongoing navigation</a> to null.</p>

        <p class=note>This allows new <a href=#navigate id=updating-the-traversable:navigate-2>navigations</a> of
        <var>navigable</var> to start, whereas during the traversal they were blocked.</p>
       <li><p><a href=#queue-a-global-task id=updating-the-traversable:queue-a-global-task-4>Queue a global task</a> on the <a href=#navigation-and-traversal-task-source id=updating-the-traversable:navigation-and-traversal-task-source-4>navigation and traversal task
       source</a> given <var>navigable</var>'s <a href=#nav-window id=updating-the-traversable:nav-window-6>active window</a> to
       perform <var>afterPotentialUnloads</var>.</ol>
     <li>
      <p>Otherwise:</p>

      <ol><li><p><a id=updating-the-traversable:assert-7 href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: <var>navigationType</var> is not null.<li><p><a href=#deactivate-a-document-for-a-cross-document-navigation id=updating-the-traversable:deactivate-a-document-for-a-cross-document-navigation>Deactivate</a>
       <var>displayedDocument</var>, given <var>userInvolvement</var>, <var>targetEntry</var>,
       <var>navigationType</var>, and <var>afterPotentialUnloads</var>.</ol>
     <li>
      <p>In both cases, let <var>afterPotentialUnloads</var> be the following steps:</p>

      <ol><li><p>Let <var>previousEntry</var> be <var>navigable</var>'s <a href=#nav-active-history-entry id=updating-the-traversable:nav-active-history-entry-3>active session history entry</a>.<li><p>If <var>changingNavigableContinuation</var>'s <a href=#changing-nav-continuation-update-only id=updating-the-traversable:changing-nav-continuation-update-only-5>update-only</a> is false, then
       <a href=#activate-history-entry id=updating-the-traversable:activate-history-entry>activate history entry</a> <var>targetEntry</var> for
       <var>navigable</var>.<li><p>Let <var>updateDocument</var> be an algorithm step which performs <a href=#update-document-for-history-step-application id=updating-the-traversable:update-document-for-history-step-application>update
       document for history step application</a> given <var>targetEntry</var>'s <a href=#she-document id=updating-the-traversable:she-document-10>document</a>, <var>targetEntry</var>,
       <var>changingNavigableContinuation</var>'s <a href=#changing-nav-continuation-update-only id=updating-the-traversable:changing-nav-continuation-update-only-6>update-only</a>,
       <var>scriptHistoryLength</var>, <var>scriptHistoryIndex</var>, <var>navigationType</var>,
       <var>entriesForNavigationAPI</var>, and <var>previousEntry</var>.<li><p>If <var>targetEntry</var>'s <a href=#she-document id=updating-the-traversable:she-document-11>document</a> is equal to
       <var>displayedDocument</var>, then perform <var>updateDocument</var>.<li><p>Otherwise, <a href=#queue-a-global-task id=updating-the-traversable:queue-a-global-task-5>queue a global task</a> on the <a href=#navigation-and-traversal-task-source id=updating-the-traversable:navigation-and-traversal-task-source-5>navigation and traversal task
       source</a> given <var>targetEntry</var>'s <a href=#she-document id=updating-the-traversable:she-document-12>document</a>'s
       <a href=#concept-relevant-global id=updating-the-traversable:concept-relevant-global>relevant global object</a> to perform <var>updateDocument</var>.<li><p>Increment <var>completedChangeJobs</var>.</ol>
     </ol>
   <li>
    <p>Let <var>totalNonchangingJobs</var> be the <a href=https://infra.spec.whatwg.org/#list-size id=updating-the-traversable:list-size-2 data-x-internal=list-size>size</a> of
    <var>nonchangingNavigablesThatStillNeedUpdates</var>.</p>

    <p class=note>This step onwards deliberately waits for all the previous operations to
    complete, as they include <a href=#sync-navigations-jump-queue>processing synchronous
    navigations</a> which will also post tasks to update history length and index.</p>
   <li><p>Let <var>completedNonchangingJobs</var> be 0.<li><p>Let (<var>scriptHistoryLength</var>, <var>scriptHistoryIndex</var>) be the result of
   <a href=#getting-the-history-object-length-and-index id=updating-the-traversable:getting-the-history-object-length-and-index-2>getting the history object length and index</a> given <var>traversable</var> and
   <var>targetStep</var>.<li>
    <p><a href=https://infra.spec.whatwg.org/#list-iterate id=updating-the-traversable:list-iterate-4 data-x-internal=list-iterate>For each</a> <var>navigable</var> of
    <var>nonchangingNavigablesThatStillNeedUpdates</var>, <a href=#queue-a-global-task id=updating-the-traversable:queue-a-global-task-6>queue a global task</a> on the
    <a href=#navigation-and-traversal-task-source id=updating-the-traversable:navigation-and-traversal-task-source-6>navigation and traversal task source</a> given <var>navigable</var>'s <a href=#nav-window id=updating-the-traversable:nav-window-7>active window</a> to run the steps:</p>

    <ol><li><p>Let <var>document</var> be <var>navigable</var>'s <a href=#nav-document id=updating-the-traversable:nav-document-3>active
     document</a>.<li><p>Set <var>document</var>'s <a href=#doc-history id=updating-the-traversable:doc-history>history object</a>'s <a href=#concept-history-index id=updating-the-traversable:concept-history-index>index</a> to <var>scriptHistoryIndex</var>.<li><p>Set <var>document</var>'s <a href=#doc-history id=updating-the-traversable:doc-history-2>history object</a>'s <a href=#concept-history-length id=updating-the-traversable:concept-history-length>length</a> to <var>scriptHistoryLength</var>.<li><p>Increment <var>completedNonchangingJobs</var>.</ol>
   <li><p>Wait for <var>completedNonchangingJobs</var> to equal
   <var>totalNonchangingJobs</var>.<li><p>Set <var>traversable</var>'s <a href=#tn-current-session-history-step id=updating-the-traversable:tn-current-session-history-step-3>current
   session history step</a> to <var>targetStep</var>.<li><p>Return "<code>applied</code>".</ol>
  </div>
```

### Complete used-step and History length/index algorithms

<!-- selected-source-fragment:74612-74644 -->
```text
  <div data-algorithm="">
  <p>To <dfn id=getting-the-used-step>get the used step</dfn> given a <a href=#traversable-navigable id=updating-the-traversable:traversable-navigable-9>traversable
  navigable</a> <var>traversable</var>, and a non-negative integer <var>step</var>, perform the
  following steps. They return a non-negative integer.</p>

  <ol><li><p>Let <var>steps</var> be the result of <a href=#getting-all-used-history-steps id=updating-the-traversable:getting-all-used-history-steps>getting all used history steps</a> within
   <var>traversable</var>.<li>
    <p>Return the greatest <a href=https://infra.spec.whatwg.org/#list-item id=updating-the-traversable:list-item-2 data-x-internal=list-item>item</a> in <var>steps</var> that is less
    than or equal to <var>step</var>.</p>

    <p class=note>This caters for situations where there's no <a href=#session-history-entry id=updating-the-traversable:session-history-entry-5>session history entry</a>
    with <a href=#she-step id=updating-the-traversable:she-step-5>step</a> <var>step</var>, due to the removal of a
    <a href=#navigable id=updating-the-traversable:navigable-10>navigable</a>.</p>
   </ol>
  </div>

  <div data-algorithm="">
  <p>To <dfn id=getting-the-history-object-length-and-index>get the history object length and
  index</dfn> given a <a href=#traversable-navigable id=updating-the-traversable:traversable-navigable-10>traversable navigable</a> <var>traversable</var>, and a non-negative
  integer <var>step</var>, perform the following steps. They return a <a id=updating-the-traversable:tuple href=https://infra.spec.whatwg.org/#tuple data-x-internal=tuple>tuple</a> of two
  non-negative integers.</p>

  <ol><li><p>Let <var>steps</var> be the result of <a href=#getting-all-used-history-steps id=updating-the-traversable:getting-all-used-history-steps-2>getting all used history steps</a> within
   <var>traversable</var>.<li><p>Let <var>scriptHistoryLength</var> be the <a href=https://infra.spec.whatwg.org/#list-size id=updating-the-traversable:list-size-3 data-x-internal=list-size>size</a> of
   <var>steps</var>.<li>
    <p><a id=updating-the-traversable:assert-10 href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: <var>steps</var> <a href=https://infra.spec.whatwg.org/#list-contain id=updating-the-traversable:list-contains-4 data-x-internal=list-contains>contains</a>
    <var>step</var>.</p>

    <p class=note>It is assumed that <var>step</var> has been adjusted by <a href=#getting-the-used-step id=updating-the-traversable:getting-the-used-step-2>getting the used
    step</a>.</p>
   <li><p>Let <var>scriptHistoryIndex</var> be the index of <var>step</var> in
   <var>steps</var>.<li><p>Return (<var>scriptHistoryLength</var>, <var>scriptHistoryIndex</var>).</ol>
  </div>
```

### Complete changing, nonchanging and target-entry lookup algorithms

<!-- selected-source-fragment:74646-74721 -->
```text
  <div data-algorithm="">
  <p>To <dfn id=get-all-navigables-whose-current-session-history-entry-will-change-or-reload>get all navigables whose current session history entry will change or reload</dfn>
  given a <a href=#traversable-navigable id=updating-the-traversable:traversable-navigable-11>traversable navigable</a> <var>traversable</var>, and a non-negative integer
  <var>targetStep</var>, perform the following steps. They return a <a id=updating-the-traversable:list href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a> of <a href=#navigable id=updating-the-traversable:navigable-11>navigables</a>.</p>

  <ol><li><p>Let <var>results</var> be an empty <a id=updating-the-traversable:list-2 href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a>.<li>
    <p>Let <var>navigablesToCheck</var> be « <var>traversable</var> ».</p>

    <p class=note>This list is extended in the loop below.</p>
   <li>
    <p><a href=https://infra.spec.whatwg.org/#list-iterate id=updating-the-traversable:list-iterate-5 data-x-internal=list-iterate>For each</a> <var>navigable</var> of
    <var>navigablesToCheck</var>:</p>

    <ol><li><p>Let <var>targetEntry</var> be the result of <a href=#getting-the-target-history-entry id=updating-the-traversable:getting-the-target-history-entry-2>getting the target history
     entry</a> given <var>navigable</var> and <var>targetStep</var>.<li><p>If <var>targetEntry</var> is not <var>navigable</var>'s <a href=#nav-current-history-entry id=updating-the-traversable:nav-current-history-entry-3>current session history entry</a> or
     <var>targetEntry</var>'s <a href=#she-document-state id=updating-the-traversable:she-document-state-13>document state</a>'s <a href=#document-state-reload-pending id=updating-the-traversable:document-state-reload-pending-6>reload pending</a> is true, then <a href=https://infra.spec.whatwg.org/#list-append id=updating-the-traversable:list-append-2 data-x-internal=list-append>append</a> <var>navigable</var> to <var>results</var>.<li>
      <p>If <var>targetEntry</var>'s <a href=#she-document id=updating-the-traversable:she-document-21>document</a> is
      <var>navigable</var>'s <a href=#nav-document id=updating-the-traversable:nav-document-4>active document</a>, and
      <var>targetEntry</var>'s <a href=#she-document-state id=updating-the-traversable:she-document-state-14>document state</a>'s <a href=#document-state-reload-pending id=updating-the-traversable:document-state-reload-pending-7>reload pending</a> is false, then <a href=https://infra.spec.whatwg.org/#list-extend id=updating-the-traversable:list-extend data-x-internal=list-extend>extend</a> <var>navigablesToCheck</var> with the <a href=#child-navigable id=updating-the-traversable:child-navigable>child navigables</a> of <var>navigable</var>.</p>

      <p class=note>Adding <a href=#child-navigable id=updating-the-traversable:child-navigable-2>child navigables</a> to
      <var>navigablesToCheck</var> means those navigables will also be checked by this loop. <a href=#child-navigable id=updating-the-traversable:child-navigable-3>Child navigables</a> are only checked if the
      <var>navigable</var>'s <a href=#nav-document id=updating-the-traversable:nav-document-5>active document</a> will not change as
      part of this traversal.</p>
     </ol>
   <li><p>Return <var>results</var>.</ol>
  </div>

  <div data-algorithm="">
  <p>To <dfn id=getting-all-navigables-that-only-need-history-object-length/index-update>get
  all navigables that only need history object length/index update</dfn> given a <a href=#traversable-navigable id=updating-the-traversable:traversable-navigable-12>traversable
  navigable</a> <var>traversable</var>, and a non-negative integer <var>targetStep</var>,
  perform the following steps. They return a <a id=updating-the-traversable:list-3 href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a> of <a href=#navigable id=updating-the-traversable:navigable-12>navigables</a>.</p>

  <p class=note>Other <a href=#navigable id=updating-the-traversable:navigable-13>navigables</a> might not be impacted by the
  traversal. For example, if the response is a 204, the currently active document will remain.
  Additionally, going 'back' after a 204 will change the <a href=#nav-current-history-entry id=updating-the-traversable:nav-current-history-entry-4>current session history entry</a>, but the <a href=#nav-active-history-entry id=updating-the-traversable:nav-active-history-entry-6>active session history entry</a> will already be
  correct.</p>

  <ol><li><p>Let <var>results</var> be an empty <a id=updating-the-traversable:list-4 href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a>.<li>
    <p>Let <var>navigablesToCheck</var> be « <var>traversable</var> ».</p>

    <p class=note>This list is extended in the loop below.</p>
   <li>
    <p><a href=https://infra.spec.whatwg.org/#list-iterate id=updating-the-traversable:list-iterate-6 data-x-internal=list-iterate>For each</a> <var>navigable</var> of
    <var>navigablesToCheck</var>:</p>

    <ol><li><p>Let <var>targetEntry</var> be the result of <a href=#getting-the-target-history-entry id=updating-the-traversable:getting-the-target-history-entry-3>getting the target history
     entry</a> given <var>navigable</var> and <var>targetStep</var>.<li>
      <p>If <var>targetEntry</var> is <var>navigable</var>'s <a href=#nav-current-history-entry id=updating-the-traversable:nav-current-history-entry-5>current session history entry</a> and
      <var>targetEntry</var>'s <a href=#she-document-state id=updating-the-traversable:she-document-state-15>document state</a>'s <a href=#document-state-reload-pending id=updating-the-traversable:document-state-reload-pending-8>reload pending</a> is false:</p>

      <ol><li><p><a href=https://infra.spec.whatwg.org/#list-append id=updating-the-traversable:list-append-3 data-x-internal=list-append>Append</a> <var>navigable</var> to
       <var>results</var>.<li>
        <p><a href=https://infra.spec.whatwg.org/#list-extend id=updating-the-traversable:list-extend-2 data-x-internal=list-extend>Extend</a> <var>navigablesToCheck</var> with
        <var>navigable</var>'s <a href=#child-navigable id=updating-the-traversable:child-navigable-4>child navigables</a>.</p>

        <p class=note>Adding <a href=#child-navigable id=updating-the-traversable:child-navigable-5>child navigables</a> to
        <var>navigablesToCheck</var> means those navigables will also be checked by this loop.
        <a href=#child-navigable id=updating-the-traversable:child-navigable-6>child navigables</a> are only checked if the
        <var>navigable</var>'s <a href=#nav-document id=updating-the-traversable:nav-document-6>active document</a> will not change
        as part of this traversal.</p>
       </ol>
     </ol>
   <li><p>Return <var>results</var>.</ol>
  </div>

  <div data-algorithm="">
  <p>To <dfn id=getting-the-target-history-entry>get the target history entry</dfn> given a
  <a href=#navigable id=updating-the-traversable:navigable-14>navigable</a> <var>navigable</var>, and a non-negative integer <var>step</var>, perform
  the following steps. They return a <a href=#session-history-entry id=updating-the-traversable:session-history-entry-6>session history entry</a>.</p>

  <ol><li><p>Let <var>entries</var> be the result of <a href=#getting-session-history-entries id=updating-the-traversable:getting-session-history-entries>getting session history entries</a> for
   <var>navigable</var>.<li><p>Return the <a href=https://infra.spec.whatwg.org/#list-item id=updating-the-traversable:list-item-3 data-x-internal=list-item>item</a> in <var>entries</var> that has the
   greatest <a href=#she-step id=updating-the-traversable:she-step-6>step</a> less than or equal to <var>step</var>.</ol>
  </div>
```

### Complete potentially-cross-document navigables algorithm

<!-- selected-source-fragment:74743-74788 -->
```text
  <div data-algorithm="">
  <p>To <dfn id=getting-all-navigables-that-might-experience-a-cross-document-traversal>get
  all navigables that might experience a cross-document traversal</dfn> given a <a href=#traversable-navigable id=updating-the-traversable:traversable-navigable-13>traversable
  navigable</a> <var>traversable</var>, and a non-negative integer <var>targetStep</var>,
  perform the following steps. They return a <a id=updating-the-traversable:list-5 href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a> of <a href=#navigable id=updating-the-traversable:navigable-15>navigables</a>.</p>

  <div class=note>
   <p>From <var>traversable</var>'s <a href=#tn-session-history-traversal-queue id=updating-the-traversable:tn-session-history-traversal-queue-5>session
   history traversal queue</a>'s perspective, these documents are candidates for going
   cross-document during the traversal described by <var>targetStep</var>. They will not experience
   a cross-document traversal if the status code for their target document is HTTP 204 No Content.</p>

   <p>Note that if a given <a href=#navigable id=updating-the-traversable:navigable-16>navigable</a> might experience a cross-document traversal, this
   algorithm will return <a href=#navigable id=updating-the-traversable:navigable-17>navigable</a> but not its <a href=#child-navigable id=updating-the-traversable:child-navigable-7>child
   navigables</a>. Those would end up <a href=#unload-a-document id=updating-the-traversable:unload-a-document>unloaded</a>, not
   traversed.</p>
  </div>

  <ol><li><p>Let <var>results</var> be an empty <a id=updating-the-traversable:list-6 href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a>.<li>
    <p>Let <var>navigablesToCheck</var> be « <var>traversable</var> ».</p>

    <p class=note>This list is extended in the loop below.</p>
   <li>
    <p><a href=https://infra.spec.whatwg.org/#list-iterate id=updating-the-traversable:list-iterate-7 data-x-internal=list-iterate>For each</a> <var>navigable</var> of
    <var>navigablesToCheck</var>:</p>

    <ol><li><p>Let <var>targetEntry</var> be the result of <a href=#getting-the-target-history-entry id=updating-the-traversable:getting-the-target-history-entry-5>getting the target history
     entry</a> given <var>navigable</var> and <var>targetStep</var>.<li>
      <p>If <var>targetEntry</var>'s <a href=#she-document id=updating-the-traversable:she-document-22>document</a> is not
      <var>navigable</var>'s <a href=#nav-document id=updating-the-traversable:nav-document-7>active document</a> or
      <var>targetEntry</var>'s <a href=#she-document-state id=updating-the-traversable:she-document-state-16>document state</a>'s <a href=#document-state-reload-pending id=updating-the-traversable:document-state-reload-pending-9>reload pending</a> is true, then <a href=https://infra.spec.whatwg.org/#list-append id=updating-the-traversable:list-append-4 data-x-internal=list-append>append</a> <var>navigable</var> to <var>results</var>.</p>

      <p class=note>Although <var>navigable</var>'s <a href=#nav-active-history-entry id=updating-the-traversable:nav-active-history-entry-7>active history entry</a> can change synchronously, the
      new entry will always have the same <code id=updating-the-traversable:document-5><a href=#document>Document</a></code>, so accessing
      <var>navigable</var>'s <a href=#nav-document id=updating-the-traversable:nav-document-8>active document</a> is reliable.</p>
     <li>
      <p>Otherwise, <a href=https://infra.spec.whatwg.org/#list-extend id=updating-the-traversable:list-extend-3 data-x-internal=list-extend>extend</a> <var>navigablesToCheck</var> with
      <var>navigable</var>'s <a href=#child-navigable id=updating-the-traversable:child-navigable-8>child navigables</a>.</p>

      <p class=note>Adding <a href=#child-navigable id=updating-the-traversable:child-navigable-9>child navigables</a> to
      <var>navigablesToCheck</var> means those navigables will also be checked by this loop. <a href=#child-navigable id=updating-the-traversable:child-navigable-10>Child navigables</a> are only checked if the
      <var>navigable</var>'s <a href=#nav-document id=updating-the-traversable:nav-document-9>active document</a> will not change as
      part of this traversal.</p>
     </ol>
   <li><p>Return <var>results</var>.</ol>
  </div>
```
