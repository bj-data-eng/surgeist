# HTML hosted initial state and top-level teardown clauses

Bounded source retention for the native hosted lifecycle's initial no-opener,
no-child top-level unit. Complete selected source bodies remain authoritative;
retention does not implement JavaScript, navigation, platform cleanup or a full
browser event loop, and does not modify the separately selected native projection.

## Source basis and attribution

Source: [immutable WHATWG HTML snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/),
revision `877d6146487f15b15bc8dd0e3c625063a37b456a` (9 October 2026).
Extracted offline on 16 October 2026 from the retained complete HTML,
14,930,614 bytes, SHA-256
`877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
No retrieval, edition change, executable or software acquisition is used.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose is
[Creative Commons Attribution 4.0 International](https://creativecommons.org/licenses/by/4.0/).
The [complete WHATWG license](../licenses/whatwg/LICENSE), SHA-256
`85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`,
and [pinned HTML attribution](../licenses/whatwg/NOTICE.md#pinned-html-language-snapshot--2026-10-11)
apply. The upstream BSD 3-Clause provision concerns portions incorporated into
source code, not an alternative license for retained specification prose.

Changes by Surgeist on 16 October 2026: bounded offline extraction, explanatory
headings/provenance and absolute same-edition fragment links. Source bodies remain
literal semantic HTML in Markdown: every word, branch, order, ID, definition/list
boundary, note and issue marker is preserved. Only `href` values beginning with
`#` are rewritten to the pinned snapshot with normal quoted attribute spelling.
No source scripts are executed. Normative/informative distinctions remain;
no endorsement or full browser-conformance claim is implied.

The exact original fragments below, in ascending source order and joined by
one extra LF, have SHA-256
`c2dd632ac56d753b0610a68c53f8f1ab6f97b9e8d0f346eec8faef48ef056bf6`.
The filename suffix identifies those original fragments, not the final file.

| Original raw lines | Complete retained unit | Original fragment SHA-256 |
| --- | --- | --- |
| 68218–68229 | Complete Window Navigation creation and getter | `2c937e250126b7d156cde5b2ddfe0fc429ae5c1c577a06ace76b1b4648446d01` |
| 68249–68290 | Navigation entry defaults, current entry, disabled guard and index lookup | `6a8ab0ea356742d6238ce66aee6d14fd0af7b118bbc26cdfd4dbb82cc6324d9e` |
| 68323–68347 | Complete new-Document Navigation-entry initialization | `e0c6400562a8f57ea83a42237c733924f8f874b85e3c23fd0a2dea070df27ba5` |
| 69080–69091 | Navigation ongoing and upcoming initial state, with note | `b0a9a2751b8e759079651ff134c5726a5fd8e5dd980c1efd510c4f63243b09e0` |
| 69343–69349 | Navigation transition initial state and complete getter | `e49739db28bc7a9f63ac185df2d09e43cf34035ff4cac3f3482b2d24cd31c180` |
| 71183–71264 | Complete related navigable collections and ordering notes | `631eb72bc55b46d9f4827fde38fd3652054a7204c540cbf444e94e36a3d314e5` |
| 75724–75742 | Complete unloading-state field definitions | `4aa65414468aa2ca8e5f3ead03789549542cd32d3b785dfb44c5939f120fa92a` |
| 75829–75949 | Complete unloading cleanup, source issue note, Document destruction and abort | `4be20f93e2e85f1301b3bfb98260b5f938f7801285f5f9c1c48ac0070b4db5c8` |
| 81757–81780 | Complete generic task-source definitions, including navigation and traversal | `ec3470bf8b8df87cd39a018c9cb9d0c890270ccd26a03ba96c1c12104326d46c` |

## Source call boundaries and first-case relevance

Reuse the unchanged [navigable lifecycle](html-hosted-navigable-lifecycle--877d6146487f--3a9658155866.md)
for complete top-level/child destruction callers, actual history-entry and
Document-state definitions, and make-active. Reuse
[global/agent/settings and initial construction](html-hosted-global-agent-settings--877d6146487f--03a314e0aa55.md)
for BC/group/agent/global ownership, original group removal and event-loop state;
[activity](html-hosted-document-activity--877d6146487f--f8564ea4758e.md) and
[task association/queuing](html-element-task-association--877d6146487f.md) retain
actual fully-active and task/global/element relations.

The requested Navigation bodies are complete, including both guard outcomes and
the reachable source-defined empty entry list/current index −1. Its disabled
guard tests actual not-fully-active, initial-about:blank and opaque-origin facts;
it does not ask whether JavaScript is supported. The full initialization body
retains every later allocation and index step despite its guarded return in an
admitted initial-about:blank case. These definitions are not a navigation API,
public replacement/history operation or proof that a constructor calls entry
initialization where the source has no such call.

Top-level destruction's caller iterates actual session-history entries and calls
destroy-Document-and-descendants for each non-null Document. It then removes
the original BC/group, UI presentation and top-level-set identity and invokes
the original WebDriver notification. It does not call the child-destruction
history traversal/update tail. That separate tail and its apply-history helpers
are outside this excerpt and remain later child-lifecycle requirements.

Destroy-Document-and-descendants runs its not-fully-active branch conditionally,
then collects actual child navigables, initializes numberDestroyed and visits
children. A native first-unit claim of no children must follow from actual
relations, not missing JS or a hard-coded empty substitute. The body still
queues the Document's actual global task, even when the child iteration has no
members. Destroy-Document calls abort-Document and **unloading document cleanup**;
it does not invoke the separate unload-a-Document algorithm at raw75744 onward.
The latter algorithm is not selected merely because the field definitions and
cleanup hook share its source section.

### Pinned wait-variable discrepancy

At raw75916, the source literally says to wait until numberDestroyed equals
**singular childNavigable's size**. It follows the closed child foreach at75915;
it is not inside that iteration or the not-fully-active branch. For an actual
fully-active initial Document with an empty child collection, the foreach is
skipped but this wait step is still reached; no loop iteration has bound the
singular variable. The separate Navigation disabled guard does not guard this
destruction wait. The exact pinned wording and adjacent unspecified-iteration
issue marker are preserved below.

Editorial native qualification: Architecture's
[destroy-descendants wait determination in #136](https://github.com/bj-data-eng/surgeist/issues/136),
also referenced by [#1124](https://github.com/bj-data-eng/surgeist/issues/1124),
uses the actual captured plural childNavigables.size, including zero, as the
native completion target. The captured count remains owned across Pending.
Zero children satisfies only this wait: the actual parent-global navigation and
traversal task, Document destruction/cleanup and afterAllDestruction steps still
run in source order. This is an explicit native qualification of the pinned
variable discrepancy, not an upstream correction, new edition or alteration of
the retained source body.

### External cleanup and task execution

Complete abort/cleanup/destruction bodies preserve their original fetch/parser,
task removal, Worker/worklet, WebSocket/WebTransport/EventSource/timer, external
cleanup, WebDriver and inactive-Document branches. The source issue lists
cross-spec cleanup ordering as unspecified. An actual empty supported resource
domain can justify the corresponding no-effect branch; noJS/noBC alone cannot.
Unavailable required service must not become fabricated completion or a null
task Document.

The navigation-and-traversal source definition is retained with its complete
surrounding generic-source list. Existing task queuing retains the actual global,
event loop and Document. This does not select a whole browser's continual event
loop, rendering, microtask engine or JS stack machinery. If actual native
execution later consumes the main processing model (raw81279–81390), microtask
checkpoint (81572–81601) or spin-the-event-loop macro (81620–81660), retain their
complete bodies and qualify that concrete consumer before claiming fulfillment.
The in-parallel destruction wait alone does not invoke the JS spin macro by name.
No child-history traversal, replacement, reactivation or blanket transitive
source closure is claimed.

## Complete Window Navigation creation and getter

<!-- selected-source-fragment:68218-68229 -->
  <div data-algorithm="">
  <p>Each <code id=navigation-interface:window><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> has an associated <dfn id=window-navigation-api>navigation
  API</dfn>, which is a <code id=navigation-interface:navigation-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation">Navigation</a></code> object. Upon creation of the <code id=navigation-interface:window-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code>
  object, its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window-navigation-api" id=navigation-interface:window-navigation-api>navigation API</a> must be set to a
  <a id=navigation-interface:new href=https://webidl.spec.whatwg.org/#new data-x-internal=new>new</a> <code id=navigation-interface:navigation-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation">Navigation</a></code> object created in the <code id=navigation-interface:window-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window">Window</a></code> object's <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-realm" id=navigation-interface:concept-relevant-realm>relevant realm</a>.</p>
  </div>

  <div data-algorithm="">
  <p>The <dfn data-dfn-for=Window id=dom-navigation data-dfn-type=attribute><code>navigation</code></dfn> getter
  steps are to return <a id=navigation-interface:this href=https://webidl.spec.whatwg.org/#this data-x-internal=this>this</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#window-navigation-api" id=navigation-interface:window-navigation-api-2>navigation
  API</a>.</p>
  </div>


## Navigation entry defaults, current entry, disabled guard and index lookup

<!-- selected-source-fragment:68249-68290 -->
  <h5 id=navigation-api-core><span class=secno>7.2.6.3</span> Core infrastructure<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-api-core" class=self-link></a></h5>

  

  <p>Each <code id=navigation-api-core:navigation><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation">Navigation</a></code> has an associated <dfn id=navigation-entry-list>entry
  list</dfn>, a <a id=navigation-api-core:list href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a> of <code id=navigation-api-core:navigationhistoryentry><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigationhistoryentry">NavigationHistoryEntry</a></code> objects, initially
  empty.</p>

  <p>Each <code id=navigation-api-core:navigation-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation">Navigation</a></code> has an associated <dfn id=navigation-current-entry-index>current entry index</dfn>, an integer, initially
  −1.</p>

  <div data-algorithm="">
  <p>The <dfn id=navigation-current-entry>current entry</dfn> of a <code id=navigation-api-core:navigation-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation">Navigation</a></code>
  <var>navigation</var> is the result of running the following steps:</p>

  <ol><li><p>If <var>navigation</var> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#has-entries-and-events-disabled" id=navigation-api-core:has-entries-and-events-disabled>has entries and events disabled</a>, then return
   null.<li><p><a id=navigation-api-core:assert href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: <var>navigation</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-current-entry-index" id=navigation-api-core:navigation-current-entry-index>current entry index</a> is not −1.<li><p>Return <var>navigation</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-entry-list" id=navigation-api-core:navigation-entry-list>entry
   list</a>[<var>navigation</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-current-entry-index" id=navigation-api-core:navigation-current-entry-index-2>current entry
   index</a>].</ol>
  </div>

  <div data-algorithm="">
  <p>A <code id=navigation-api-core:navigation-4><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation">Navigation</a></code> <var>navigation</var> <dfn id=has-entries-and-events-disabled>has entries and events disabled</dfn> if
  the following steps return true:</p>

  <ol><li><p>Let <var>document</var> be <var>navigation</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global" id=navigation-api-core:concept-relevant-global>relevant global object</a>'s
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-window" id=navigation-api-core:concept-document-window>associated <code>Document</code></a>.<li><p>If <var>document</var> is not <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fully-active" id=navigation-api-core:fully-active>fully active</a>, then return true.<li><p>If <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#is-initial-about:blank" id=navigation-api-core:is-initial-about:blank>is initial <code>about:blank</code></a> is true, then
   return true.<li><p>If <var>document</var>'s <a href=https://dom.spec.whatwg.org/#concept-document-origin id=navigation-api-core:concept-document-origin data-x-internal=concept-document-origin>origin</a> is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-origin-opaque" id=navigation-api-core:concept-origin-opaque>opaque</a>, then return true.<li><p>Return false.</ol>
  </div>

  <div data-algorithm="">
  <p>To <dfn id=getting-the-navigation-api-entry-index>get the navigation API entry
  index</dfn> of a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#session-history-entry" id=navigation-api-core:session-history-entry>session history entry</a> <var>she</var> within a
  <code id=navigation-api-core:navigation-5><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation">Navigation</a></code> <var>navigation</var>:</p>

  <ol><li><p>Let <var>index</var> be 0.<li>
    <p><a href=https://infra.spec.whatwg.org/#list-iterate id=navigation-api-core:list-iterate data-x-internal=list-iterate>For each</a> <var>nhe</var> of <var>navigation</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-entry-list" id=navigation-api-core:navigation-entry-list-2>entry list</a>:</p>

    <ol><li><p>If <var>nhe</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nhe-she" id=navigation-api-core:nhe-she>session history entry</a> is equal to
     <var>she</var>, then return <var>index</var>.<li><p>Increment <var>index</var> by 1.</ol>
   <li><p>Return −1.</ol>
  </div>


## Complete new-Document Navigation-entry initialization

<!-- selected-source-fragment:68323-68347 -->
  <h5 id=navigation-api-entry-updates><span class=secno>7.2.6.4</span> Initializing and updating the entry list<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-api-entry-updates" class=self-link></a></h5>

  <div data-algorithm="">
  <p>To <dfn id=initialize-the-navigation-api-entries-for-a-new-document>initialize the navigation API entries for a new document</dfn> given a
  <code id=navigation-api-entry-updates:navigation><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation">Navigation</a></code> <var>navigation</var>, a <a id=navigation-api-entry-updates:list href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a> of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#session-history-entry" id=navigation-api-entry-updates:session-history-entry>session history entries</a> <var>newSHEs</var>, and a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#session-history-entry" id=navigation-api-entry-updates:session-history-entry-2>session history
  entry</a> <var>initialSHE</var>:</p>

  <ol><li><p><a id=navigation-api-entry-updates:assert href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: <var>navigation</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-entry-list" id=navigation-api-entry-updates:navigation-entry-list>entry
   list</a> <a href=https://infra.spec.whatwg.org/#list-is-empty id=navigation-api-entry-updates:list-is-empty data-x-internal=list-is-empty>is empty</a>.<li><p><a id=navigation-api-entry-updates:assert-2 href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: <var>navigation</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-current-entry-index" id=navigation-api-entry-updates:navigation-current-entry-index>current entry index</a> is −1.<li><p>If <var>navigation</var> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#has-entries-and-events-disabled" id=navigation-api-entry-updates:has-entries-and-events-disabled>has entries and events disabled</a>, then
   return.<li>
    <p><a href=https://infra.spec.whatwg.org/#list-iterate id=navigation-api-entry-updates:list-iterate data-x-internal=list-iterate>For each</a> <var>newSHE</var> of <var>newSHEs</var>:</p>

    <ol><li><p>Let <var>newNHE</var> be a <a id=navigation-api-entry-updates:new href=https://webidl.spec.whatwg.org/#new data-x-internal=new>new</a> <code id=navigation-api-entry-updates:navigationhistoryentry><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigationhistoryentry">NavigationHistoryEntry</a></code> created
     in the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-realm" id=navigation-api-entry-updates:concept-relevant-realm>relevant realm</a> of
     <var>navigation</var>.<li><p>Set <var>newNHE</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nhe-she" id=navigation-api-entry-updates:nhe-she>session history entry</a> to
     <var>newSHE</var>.<li><p><a href=https://infra.spec.whatwg.org/#list-append id=navigation-api-entry-updates:list-append data-x-internal=list-append>Append</a> <var>newNHE</var> to <var>navigation</var>'s
     <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-entry-list" id=navigation-api-entry-updates:navigation-entry-list-2>entry list</a>.</ol>

    <p class=note><var>newSHEs</var> will have originally come from <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#getting-session-history-entries-for-the-navigation-api" id=navigation-api-entry-updates:getting-session-history-entries-for-the-navigation-api>getting session history
    entries for the navigation API</a>, and thus each <var>newSHE</var> will be contiguous <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#same-origin" id=navigation-api-entry-updates:same-origin>same</a> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-origin" id=navigation-api-entry-updates:document-state-origin>origin</a> with
    <var>initialSHE</var>.</p>
   <li><p>Set <var>navigation</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-current-entry-index" id=navigation-api-entry-updates:navigation-current-entry-index-2>current entry
   index</a> to the result of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#getting-the-navigation-api-entry-index" id=navigation-api-entry-updates:getting-the-navigation-api-entry-index>getting the navigation API entry index</a> of
   <var>initialSHE</var> within <var>navigation</var>.</ol>
  </div>


## Navigation ongoing and upcoming initial state, with note

<!-- selected-source-fragment:69080-69091 -->
  <p>We end up accomplishing all this by associating the following with each
  <code id=ongoing-navigation-tracking:navigation-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation">Navigation</a></code>:</p>

  <ul><li><p><dfn id=ongoing-navigate-event>Ongoing <code>navigate</code> event</dfn>, a
   <code id=ongoing-navigation-tracking:navigateevent-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigateevent">NavigateEvent</a></code> or null, initially null.<li><p><dfn id=focus-changed-during-ongoing-navigation>Focus changed during ongoing navigation</dfn>, a boolean, initially false.<li><p><dfn id=suppress-normal-scroll-restoration-during-ongoing-navigation>Suppress normal scroll restoration during ongoing navigation</dfn>, a boolean,
   initially false.<li><p><dfn id=ongoing-api-method-tracker>Ongoing API method tracker</dfn>, a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-api-method-tracker" id=ongoing-navigation-tracking:navigation-api-method-tracker>navigation API method tracker</a> or
   null, initially null.<li><p><dfn id=upcoming-traverse-api-method-trackers>Upcoming traverse API method trackers</dfn>, an <a id=ongoing-navigation-tracking:ordered-map href=https://infra.spec.whatwg.org/#ordered-map data-x-internal=ordered-map>ordered map</a> from strings
   to <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-api-method-tracker" id=ongoing-navigation-tracking:navigation-api-method-tracker-2>navigation API method trackers</a>, initially
   empty.</ul>

  <p class=note>The state here that is not stored in <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-api-method-tracker" id=ongoing-navigation-tracking:navigation-api-method-tracker-3>navigation API method trackers</a> is state which needs to be tracked even for
  navigations that are not initiated via navigation API methods.</p>


## Navigation transition initial state and complete getter

<!-- selected-source-fragment:69343-69349 -->
  <p>Each <code id=ongoing-navigation-tracking:navigation-7><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation">Navigation</a></code> has a <dfn id=concept-navigation-transition>transition</dfn>, which is a
  <code id=ongoing-navigation-tracking:navigationtransition-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigationtransition">NavigationTransition</a></code> or null, initially null.</p>

  <div data-algorithm="">
  <p>The <dfn data-dfn-for=Navigation id=dom-navigation-transition data-dfn-type=attribute><code>transition</code></dfn> getter steps are to return
  <a id=ongoing-navigation-tracking:this href=https://webidl.spec.whatwg.org/#this data-x-internal=this>this</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-navigation-transition" id=ongoing-navigation-tracking:concept-navigation-transition-5>transition</a>.</p>
  </div>


## Complete related navigable collections and ordering notes

<!-- selected-source-fragment:71183-71264 -->
  <h5 id=related-navigable-collections><span class=secno>7.3.1.5</span> Related navigable collections<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#related-navigable-collections" class=self-link></a></h5>

  <p>It is often helpful in this standard's algorithms to look at collections of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=related-navigable-collections:navigable>navigables</a> starting at a given <code id=related-navigable-collections:document><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>. This section
  contains a curated set of algorithms for collecting those navigables.</p>

  <p class=note>The return values of these algorithms are ordered so that parents appears before
  their children. Callers rely on this ordering.</p>

  <p class=note>Starting with a <code id=related-navigable-collections:document-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>, rather than a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=related-navigable-collections:navigable-2>navigable</a>, is
  generally better because it makes the caller cognizant of whether they are starting with a
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fully-active" id=related-navigable-collections:fully-active>fully active</a> <code id=related-navigable-collections:document-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> or not. Although non-<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fully-active" id=related-navigable-collections:fully-active-2>fully active</a>
  <code id=related-navigable-collections:document-4><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>s do have ancestor and descendant navigables, they often behave as if they
  don't (e.g., in the <code id=related-navigable-collections:dom-parent><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-parent">window.parent</a></code> getter).</p>

  <div data-algorithm="">
  <p>The <dfn data-dfn-for=Document id=ancestor-navigables data-export="">ancestor navigables</dfn> of a <code id=related-navigable-collections:document-5><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>
  <var>document</var> are given by these steps:</p>

  <ol><li><p>Let <var>navigable</var> be <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable" id=related-navigable-collections:node-navigable>node navigable</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-parent" id=related-navigable-collections:nav-parent>parent</a>.<li><p>Let <var>ancestors</var> be an empty list.</p>

   <li>
    <p>While <var>navigable</var> is not null:</p>

    <ol><li><p><a href=https://infra.spec.whatwg.org/#list-prepend id=related-navigable-collections:list-prepend data-x-internal=list-prepend>Prepend</a> <var>navigable</var> to
     <var>ancestors</var>.<li><p>Set <var>navigable</var> to <var>navigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-parent" id=related-navigable-collections:nav-parent-2>parent</a>.</ol>
   <li><p>Return <var>ancestors</var>.</ol>
  </div>

  <div data-algorithm="">
  <p>The <dfn data-dfn-for=Document id=inclusive-ancestor-navigables data-export="">inclusive ancestor navigables</dfn> of a <code id=related-navigable-collections:document-6><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>
  <var>document</var> are given by these steps:</p>

  <ol><li><p>Let <var>navigables</var> be <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#ancestor-navigables" id=related-navigable-collections:ancestor-navigables>ancestor
   navigables</a>.<li><p><a href=https://infra.spec.whatwg.org/#list-append id=related-navigable-collections:list-append data-x-internal=list-append>Append</a> <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable" id=related-navigable-collections:node-navigable-2>node
   navigable</a> to <var>navigables</var>.<li><p>Return <var>navigables</var>.</ol>
  </div>

  <div data-algorithm="">
  <p id=list-of-the-descendant-browsing-contexts>The <dfn data-dfn-for=Document id=descendant-navigables data-export="">descendant
  navigables</dfn> of a <code id=related-navigable-collections:document-7><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> <var>document</var> are given by these steps:</p>

  <ol><li><p>Let <var>navigables</var> be new <a id=related-navigable-collections:list href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a>.<li><p>Let <var>navigableContainers</var> be a <a id=related-navigable-collections:list-2 href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a> of all <a href=https://dom.spec.whatwg.org/#concept-shadow-including-descendant id=related-navigable-collections:shadow-including-descendant data-x-internal=shadow-including-descendant>shadow-including descendants</a> of <var>document</var>
   that are <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container" id=related-navigable-collections:navigable-container>navigable containers</a>, in
   <a id=related-navigable-collections:shadow-including-tree-order href=https://dom.spec.whatwg.org/#concept-shadow-including-tree-order data-x-internal=shadow-including-tree-order>shadow-including tree order</a>.<li>
    <p><a href=https://infra.spec.whatwg.org/#list-iterate id=related-navigable-collections:list-iterate data-x-internal=list-iterate>For each</a> <var>navigableContainer</var> of
    <var>navigableContainers</var>:</p>

    <ol><li><p>If <var>navigableContainer</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable" id=related-navigable-collections:content-navigable>content navigable</a> is null, then
     continue.<li><p><a href=https://infra.spec.whatwg.org/#list-extend id=related-navigable-collections:list-extend data-x-internal=list-extend>Extend</a> <var>navigables</var> with
     <var>navigableContainer</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable" id=related-navigable-collections:content-navigable-2>content navigable</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document" id=related-navigable-collections:nav-document>active document</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#inclusive-descendant-navigables" id=related-navigable-collections:inclusive-descendant-navigables>inclusive descendant
     navigables</a>.</ol>
   <li><p>Return <var>navigables</var>.</ol>
  </div>

  <div data-algorithm="">
  <p>The <dfn data-dfn-for=Document id=inclusive-descendant-navigables data-export="">inclusive descendant navigables</dfn> of a
  <code id=related-navigable-collections:document-8><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> <var>document</var> are given by these steps:</p>

  <ol><li><p>Let <var>navigables</var> be « <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable" id=related-navigable-collections:node-navigable-3>node navigable</a>
   ».<li><p><a href=https://infra.spec.whatwg.org/#list-extend id=related-navigable-collections:list-extend-2 data-x-internal=list-extend>Extend</a> <var>navigables</var> with <var>document</var>'s
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#descendant-navigables" id=related-navigable-collections:descendant-navigables>descendant navigables</a>.<li><p>Return <var>navigables</var>.</ol>
  </div>

  <p class=note>These descendant-collecting algorithms are described as looking at the DOM tree
  of descendant <code id=related-navigable-collections:document-9><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> objects. In reality, this is often not feasible since the DOM
  tree can be in another process from the caller of the algorithm. Instead, implementations
  generally replicate the appropriate trees across processes.</p>

  <div data-algorithm="">
  <p id=document-tree-child-browsing-context>The <dfn id=document-tree-child-navigables>document-tree child navigables</dfn> of a
  <code id=related-navigable-collections:document-10><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> <var>document</var> are given by these steps:</p>

  <ol><li><p>If <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable" id=related-navigable-collections:node-navigable-4>node navigable</a> is null, then return the empty
   list.<li><p>Let <var>navigables</var> be new <a id=related-navigable-collections:list-3 href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a>.<li><p>Let <var>navigableContainers</var> be a <a id=related-navigable-collections:list-4 href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a> of all <a href=https://dom.spec.whatwg.org/#concept-tree-descendant id=related-navigable-collections:descendant data-x-internal=descendant>descendants</a> of <var>document</var> that are <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable-container" id=related-navigable-collections:navigable-container-2>navigable containers</a>, in <a id=related-navigable-collections:tree-order href=https://dom.spec.whatwg.org/#concept-tree-order data-x-internal=tree-order>tree order</a>.<li>
    <p><a href=https://infra.spec.whatwg.org/#list-iterate id=related-navigable-collections:list-iterate-2 data-x-internal=list-iterate>For each</a> <var>navigableContainer</var> of
    <var>navigableContainers</var>:</p>

    <ol><li><p>If <var>navigableContainer</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable" id=related-navigable-collections:content-navigable-3>content navigable</a> is null, then
     <a id=related-navigable-collections:continue href=https://infra.spec.whatwg.org/#iteration-continue data-x-internal=continue>continue</a>.<li><p><a href=https://infra.spec.whatwg.org/#list-append id=related-navigable-collections:list-append-2 data-x-internal=list-append>Append</a> <var>navigableContainer</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#content-navigable" id=related-navigable-collections:content-navigable-4>content
     navigable</a> to <var>navigables</var>.</ol>
   <li><p>Return <var>navigables</var>.</ol>
  </div>


## Complete unloading-state field definitions

<!-- selected-source-fragment:75724-75742 -->
  <h4 id=unloading-documents><span class=secno>7.5.9</span> Unloading documents<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unloading-documents" class=self-link></a></h4>

  <p>A <code id=unloading-documents:document><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> has a <dfn id=concept-document-salvageable><i>salvageable</i></dfn> state, which must initially be
  true, and a <dfn id=page-showing>page showing</dfn> boolean, which is initially false. The <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#page-showing" id=unloading-documents:page-showing>page
  showing</a> boolean is used to ensure that scripts receive <code id=unloading-documents:event-pageshow><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-pageshow">pageshow</a></code> and <code id=unloading-documents:event-pagehide><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-pagehide">pagehide</a></code> events
  in a consistent manner (e.g., that they never receive two <code id=unloading-documents:event-pagehide-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-pagehide">pagehide</a></code> events in a row without an intervening <code id=unloading-documents:event-pageshow-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-pageshow">pageshow</a></code>, or vice versa).</p>

  <p>A <code id=unloading-documents:document-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> has a <code id=unloading-documents:domhighrestimestamp><a data-x-internal=domhighrestimestamp href=https://w3c.github.io/hr-time/#dom-domhighrestimestamp>DOMHighResTimeStamp</a></code> <dfn id=suspension-time>suspension time</dfn>,
  initially 0.</p>

  <p>A <code id=unloading-documents:document-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> has a <a id=unloading-documents:list href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a> of <dfn id=suspended-timer-handles>suspended timer handles</dfn>,
  initially empty.</p>

  <p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id=unloading-documents:event-loop>Event loops</a> have a <dfn id=termination-nesting-level>termination nesting level</dfn>
  counter, which must initially be 0.</p>

  <p id=ignore-opens-during-unload-counter><code id=unloading-documents:document-4><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> objects have an <dfn id=unload-counter>unload
  counter</dfn>, which is used to ignore certain operations while the below algorithms run.
  Initially, the counter must be set to zero.</p>


## Complete unloading cleanup, source issue note, Document destruction and abort

<!-- selected-source-fragment:75829-75949 -->
  <div data-algorithm="">
  <p>This specification defines the following <dfn id=unloading-document-cleanup-steps data-export="">unloading document cleanup steps</dfn>.
  Other specifications can define more. Given a <code id=unloading-documents:document-9><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> <var>document</var>:</p>

  <ol><li><p>Let <var>window</var> be <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global" id=unloading-documents:concept-relevant-global-7>relevant global
   object</a>.<li>
    <p>For each <code id=unloading-documents:websocket-2><a data-x-internal=websocket-2 href=https://websockets.spec.whatwg.org/#websocket>WebSocket</a></code> object <var>webSocket</var> whose <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global" id=unloading-documents:concept-relevant-global-8>relevant global
    object</a> is <var>window</var>, <a id=unloading-documents:make-disappear href=https://websockets.spec.whatwg.org/#make-disappear data-x-internal=make-disappear>make disappear</a> <var>webSocket</var>.</p>

    <p>If this affected any <code id=unloading-documents:websocket-2-2><a data-x-internal=websocket-2 href=https://websockets.spec.whatwg.org/#websocket>WebSocket</a></code> objects, then <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#make-document-unsalvageable" id=unloading-documents:make-document-unsalvageable>make document
    unsalvageable</a> given <var>document</var> and "<code id=unloading-documents:blocking-websocket><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#blocking-websocket">websocket</a></code>".</p>
   <li><p>For each <code id=unloading-documents:webtransport><a data-x-internal=webtransport href=https://w3c.github.io/webtransport/#webtransport>WebTransport</a></code> object <var>transport</var> whose <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global" id=unloading-documents:concept-relevant-global-9>relevant global
   object</a> is <var>window</var>, run the <a id=unloading-documents:context-cleanup-steps href=https://w3c.github.io/webtransport/#context-cleanup-steps data-x-internal=context-cleanup-steps>context cleanup steps</a> given
   <var>transport</var>.<li>
    <p>If <var>document</var>'s <i id=unloading-documents:concept-document-salvageable-7><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-salvageable">salvageable</a></i>
    state is false:</p>

    <ol><li><p>For each <code id=unloading-documents:eventsource><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#eventsource">EventSource</a></code> object <var>eventSource</var> whose <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global" id=unloading-documents:concept-relevant-global-10>relevant
     global object</a> is equal to <var>window</var>, <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-eventsource-forcibly-close" id=unloading-documents:concept-eventsource-forcibly-close>forcibly close</a>
     <var>eventSource</var>.<li><p><a href=https://infra.spec.whatwg.org/#map-clear id=unloading-documents:map-clear data-x-internal=map-clear>Clear</a> <var>window</var>'s
     <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#map-of-active-timers" id=unloading-documents:map-of-active-timers-2>map of active timers</a>.</ol>
   </ol>
  </div>

  <div class=XXX>
   <p>It would be better if specification authors sent a pull request to add calls from here into
   their specifications directly, instead of using the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unloading-document-cleanup-steps" id=unloading-documents:unloading-document-cleanup-steps-2>unloading document cleanup
   steps</a> hook, to ensure well-defined cross-specification call order. As of the time of this
   writing the following specifications are known to have <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unloading-document-cleanup-steps" id=unloading-documents:unloading-document-cleanup-steps-3>unloading document cleanup
   steps</a>, which will be run in an unspecified order: <cite>Fullscreen API</cite>, <cite>Web
   NFC</cite>, <cite>WebDriver BiDi</cite>, <cite>Compute Pressure</cite>, <cite>File API</cite>,
   <cite>Media Capture and Streams</cite>, <cite>Picture-in-Picture</cite>, <cite>Screen
   Orientation</cite>, <cite>Service Workers</cite>, <cite>WebLocks API</cite>, <cite>WebAudio
   API</cite>, <cite>WebRTC</cite>. <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsFULLSCREEN">[FULLSCREEN]</a> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsWEBNFC">[WEBNFC]</a> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsWEBDRIVERBIDI">[WEBDRIVERBIDI]</a>
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsCOMPUTEPRESSURE">[COMPUTEPRESSURE]</a> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsFILEAPI">[FILEAPI]</a> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsMEDIASTREAM">[MEDIASTREAM]</a> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsPICTUREINPICTURE">[PICTUREINPICTURE]</a>
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsSCREENORIENTATION">[SCREENORIENTATION]</a> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsSW">[SW]</a> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsWEBLOCKS">[WEBLOCKS]</a> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsWEBAUDIO">[WEBAUDIO]</a>
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsWEBRTC">[WEBRTC]</a>

   <p><a href=https://github.com/whatwg/html/issues/8906>Issue #8906</a> tracks the work to make
   the order of these steps clear.</p>
  </div>

  <h4 id=destroying-documents><span class=secno>7.5.10</span> Destroying documents<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#destroying-documents" class=self-link></a></h4>

  <div data-algorithm="">
  <p id=discard-a-document>To <dfn id=destroy-a-document>destroy</dfn> a
  <code id=destroying-documents:document><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> <var>document</var>:</p>

  <ol><li><p><a id=destroying-documents:assert href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: this is running as part of a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task" id=destroying-documents:concept-task>task</a>
   queued on <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant-agent" id=destroying-documents:relevant-agent>relevant agent</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-agent-event-loop" id=destroying-documents:concept-agent-event-loop>event loop</a>.<li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#abort-a-document" id=destroying-documents:abort-a-document>Abort</a> <var>document</var>.<li><p>Set <var>document</var>'s <i id=destroying-documents:concept-document-salvageable><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-salvageable">salvageable</a></i> state
   to false.<li><p>Run any <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#unloading-document-cleanup-steps" id=destroying-documents:unloading-document-cleanup-steps>unloading document cleanup steps</a> for <var>document</var> that
   are defined by this specification and <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#other-applicable-specifications" id=destroying-documents:other-applicable-specifications>other applicable specifications</a>.<li><p>Remove any <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task" id=destroying-documents:concept-task-2>tasks</a> whose <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task-document" id=destroying-documents:concept-task-document>document</a> is <var>document</var> from any <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#task-queue" id=destroying-documents:task-queue>task
   queue</a> (without running those tasks).<li><p>Set <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id=destroying-documents:concept-document-bc>browsing context</a> to
   null.<li><p>Set <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable" id=destroying-documents:node-navigable>node navigable</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-active-history-entry" id=destroying-documents:nav-active-history-entry>active session history entry</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#she-document-state" id=destroying-documents:she-document-state>document state</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document-state-document" id=destroying-documents:document-state-document>document</a> to null.<li><p><a href=https://infra.spec.whatwg.org/#list-remove id=destroying-documents:list-remove data-x-internal=list-remove>Remove</a> <var>document</var> from the
   <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-WorkerGlobalScope-owner-set" id=destroying-documents:concept-WorkerGlobalScope-owner-set>owner set</a> of each <code id=destroying-documents:workerglobalscope><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#workerglobalscope">WorkerGlobalScope</a></code> object whose set <a href=https://infra.spec.whatwg.org/#list-contain id=destroying-documents:list-contains data-x-internal=list-contains>contains</a> <var>document</var>.<li><p><a href=https://infra.spec.whatwg.org/#list-iterate id=destroying-documents:list-iterate data-x-internal=list-iterate>For each</a> <var>workletGlobalScope</var> in
   <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-worklet-global-scopes" id=destroying-documents:concept-document-worklet-global-scopes>worklet global
   scopes</a>, <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#terminate-a-worklet-global-scope" id=destroying-documents:terminate-a-worklet-global-scope>terminate</a>
   <var>workletGlobalScope</var>.</ol>
  </div>

  <p class=note>Even after destruction, the <code id=destroying-documents:document-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> object itself might still be
  accessible to script, in the case where we are <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#destroy-a-child-navigable" id=destroying-documents:destroy-a-child-navigable>destroying a child navigable</a>.</p>

  <div data-algorithm="">
  <p>To <dfn id=destroy-a-document-and-its-descendants>destroy a document and its descendants</dfn> given a <code id=destroying-documents:document-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>
  <var>document</var> and an optional set of steps <var>afterAllDestruction</var>, perform the
  following steps <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#in-parallel" id=destroying-documents:in-parallel>in parallel</a>:</p>

  <ol><li>
    <p>If <var>document</var> is not <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fully-active" id=destroying-documents:fully-active>fully active</a>:</p>

    <ol><li><p>Let <var>reason</var> be a string from <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#ua-specific-blocking-reasons" id=destroying-documents:ua-specific-blocking-reasons>user-agent specific blocking reasons</a>. If none
     apply, then let <var>reason</var> be "<code id=destroying-documents:blocking-masked><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#blocking-masked">masked</a></code>".<li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#make-document-unsalvageable" id=destroying-documents:make-document-unsalvageable>Make document unsalvageable</a> given <var>document</var> and
     <var>reason</var>.<li><p>If <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable" id=destroying-documents:node-navigable-2>node navigable</a> is a
     <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable" id=destroying-documents:top-level-traversable>top-level traversable</a>,
     <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#build-not-restored-reasons-for-a-top-level-traversable-and-its-descendants" id=destroying-documents:build-not-restored-reasons-for-a-top-level-traversable-and-its-descendants>build not restored reasons for a top-level traversable and its descendants</a>
     given <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable" id=destroying-documents:node-navigable-3>node navigable</a>.</ol>
   <li><p>Let <var>childNavigables</var> be <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#child-navigable" id=destroying-documents:child-navigable>child navigables</a>.<li><p>Let <var>numberDestroyed</var> be 0.<li>
    <p><a href=https://infra.spec.whatwg.org/#list-iterate id=destroying-documents:list-iterate-2 data-x-internal=list-iterate>For each</a> <var>childNavigable</var> of
    <var>childNavigables</var> <span class=XXX>in what order?</span>, <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#queue-a-global-task" id=destroying-documents:queue-a-global-task>queue a
    global task</a> on the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-and-traversal-task-source" id=destroying-documents:navigation-and-traversal-task-source>navigation and traversal task source</a> given
    <var>childNavigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-window" id=destroying-documents:nav-window>active window</a> to perform the
    following steps:</p>

    <ol><li><p>Let <var>incrementDestroyed</var> be an algorithm step which increments
     <var>numberDestroyed</var>.<li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#destroy-a-document-and-its-descendants" id=destroying-documents:destroy-a-document-and-its-descendants>Destroy a document and its descendants</a> given <var>childNavigable</var>'s
     <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document" id=destroying-documents:nav-document>active document</a> and <var>incrementDestroyed</var>.</ol>
   <li><p>Wait until <var>numberDestroyed</var> equals <var>childNavigable</var>'s <a href=https://infra.spec.whatwg.org/#list-size id=destroying-documents:list-size data-x-internal=list-size>size</a>.<li>
    <p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#queue-a-global-task" id=destroying-documents:queue-a-global-task-2>Queue a global task</a> on the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigation-and-traversal-task-source" id=destroying-documents:navigation-and-traversal-task-source-2>navigation and traversal task source</a>
    given <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-relevant-global" id=destroying-documents:concept-relevant-global>relevant global object</a> to perform the following
    steps:</p>

    <ol><li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#destroy-a-document" id=destroying-documents:destroy-a-document>Destroy</a> <var>document</var>.<li><p>If <var>afterAllDestruction</var> was given, then run it.</ol>
   </ol>
  </div>

  <h4 id=aborting-a-document-load><span class=secno>7.5.11</span> Aborting a document load<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#aborting-a-document-load" class=self-link></a></h4>

  <div data-algorithm="">
  <p>To <dfn id=abort-a-document>abort</dfn> a <code id=aborting-a-document-load:document><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> <var>document</var>:</p>

  <ol><li><p><a id=aborting-a-document-load:assert href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: this is running as part of a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task" id=aborting-a-document-load:concept-task>task</a>
   queued on <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#relevant-agent" id=aborting-a-document-load:relevant-agent>relevant agent</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-agent-event-loop" id=aborting-a-document-load:concept-agent-event-loop>event loop</a>.<li><p>Cancel any instances of the <a href=https://fetch.spec.whatwg.org/#concept-fetch id=aborting-a-document-load:concept-fetch data-x-internal=concept-fetch>fetch</a> algorithm in the
   context of <var>document</var>, discarding any <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task" id=aborting-a-document-load:concept-task-2>tasks</a> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#queue-a-task" id=aborting-a-document-load:queue-a-task>queued</a> for them, and discarding any further data received from the
   network for them. If this resulted in any instances of the <a href=https://fetch.spec.whatwg.org/#concept-fetch id=aborting-a-document-load:concept-fetch-2 data-x-internal=concept-fetch>fetch</a> algorithm being canceled or any <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#queue-a-task" id=aborting-a-document-load:queue-a-task-2>queued</a> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task" id=aborting-a-document-load:concept-task-3>tasks</a> or any network data getting
   discarded, then <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#make-document-unsalvageable" id=aborting-a-document-load:make-document-unsalvageable>make document unsalvageable</a> given <var>document</var> and
   "<code id=aborting-a-document-load:blocking-fetch><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#blocking-fetch">fetch</a></code>".<li>
    <p>If <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-navigation-id" id=aborting-a-document-load:concept-document-navigation-id>during-loading
    navigation ID for WebDriver BiDi</a> is non-null:</p>

    <ol><li><p>Invoke <a id=aborting-a-document-load:webdriver-bidi-navigation-aborted href=https://w3c.github.io/webdriver-bidi/#webdriver-bidi-navigation-aborted data-x-internal=webdriver-bidi-navigation-aborted>WebDriver BiDi navigation aborted</a> with <var>document</var>'s
     <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#node-navigable" id=aborting-a-document-load:node-navigable>node navigable</a> and a new <a id=aborting-a-document-load:webdriver-bidi-navigation-status href=https://w3c.github.io/webdriver-bidi/#webdriver-bidi-navigation-status data-x-internal=webdriver-bidi-navigation-status>WebDriver BiDi navigation status</a> whose <a href=https://w3c.github.io/webdriver-bidi/#navigation-status-id id=aborting-a-document-load:navigation-status-id data-x-internal=navigation-status-id>id</a> is <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-navigation-id" id=aborting-a-document-load:concept-document-navigation-id-2>during-loading navigation ID for WebDriver BiDi</a>,
     <a href=https://w3c.github.io/webdriver-bidi/#navigation-status-status id=aborting-a-document-load:navigation-status-status data-x-internal=navigation-status-status>status</a> is "<code id=aborting-a-document-load:navigation-status-canceled><a data-x-internal=navigation-status-canceled href=https://w3c.github.io/webdriver-bidi/#navigation-status-canceled>canceled</a></code>", and <a href=https://w3c.github.io/webdriver-bidi/#navigation-status-url id=aborting-a-document-load:navigation-status-url data-x-internal=navigation-status-url>url</a> is <var>document</var>'s <a href=https://dom.spec.whatwg.org/#concept-document-url id="aborting-a-document-load:the-document's-address" data-x-internal="the-document's-address">URL</a>.<li><p>Set <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-navigation-id" id=aborting-a-document-load:concept-document-navigation-id-3>during-loading
     navigation ID for WebDriver BiDi</a> to null.</ol>
   <li>
    <p>If <var>document</var> has an <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#active-parser" id=aborting-a-document-load:active-parser>active parser</a>:</p>

    <ol><li><p>Set <var>document</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#active-parser-was-aborted" id=aborting-a-document-load:active-parser-was-aborted>active parser was aborted</a> to true.<li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#abort-a-parser" id=aborting-a-document-load:abort-a-parser>Abort that parser</a>.<li><p><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#make-document-unsalvageable" id=aborting-a-document-load:make-document-unsalvageable-2>Make document unsalvageable</a> given <var>document</var> and
     "<code id=aborting-a-document-load:blocking-parser-aborted><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#blocking-parser-aborted">parser-aborted</a></code>".</ol>
   </ol>
  </div>


## Complete generic task-source definitions, including navigation and traversal

<!-- selected-source-fragment:81757-81780 -->
  <h5 id=generic-task-sources><span class=secno>8.1.7.4</span> Generic task sources<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#generic-task-sources" class=self-link></a></h5>

  <p>The following <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#task-source" id=generic-task-sources:task-source>task sources</a> are used by a number of mostly
  unrelated features in this and other specifications.</p>

  <dl><dt>The <dfn id=dom-manipulation-task-source data-export="">DOM manipulation task source</dfn><dd>
    <p>This <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#task-source" id=generic-task-sources:task-source-2>task source</a> is used for features that react to DOM manipulations, such as
    things that happen in a non-blocking fashion when an element is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#insert-an-element-into-a-document" id=generic-task-sources:insert-an-element-into-a-document>inserted into the document</a>.</p>
   <dt>The <dfn id=user-interaction-task-source data-export="">user interaction task source</dfn><dd>
    <p>This <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#task-source" id=generic-task-sources:task-source-3>task source</a> is used for features that react to user interaction, for
    example keyboard or mouse input.</p>

    <p>Events sent in response to user input (e.g., <code id=generic-task-sources:event-click><a data-x-internal=event-click href=https://w3c.github.io/pointerevents/#click>click</a></code> events)
    must be fired using <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task" id=generic-task-sources:concept-task>tasks</a> <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#queue-a-task" id=generic-task-sources:queue-a-task>queued</a> with the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#user-interaction-task-source" id=generic-task-sources:user-interaction-task-source>user interaction task source</a>. <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#refsUIEVENTS">[UIEVENTS]</a></p>
    
   <dt>The <dfn id=networking-task-source data-export="">networking task source</dfn><dd>
    <p>This <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#task-source" id=generic-task-sources:task-source-4>task source</a> is used for features that trigger in response to network
    activity.</p>
   <dt id=history-traversal-task-source>The <dfn id=navigation-and-traversal-task-source>navigation and traversal task source</dfn><dd>
    <p>This <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#task-source" id=generic-task-sources:task-source-5>task source</a> is used to queue tasks involved in <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigate" id=generic-task-sources:navigate>navigation</a> and <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#apply-the-traverse-history-step" id=generic-task-sources:apply-the-traverse-history-step>history
    traversal</a>.</p>
   <dt>The <dfn id=rendering-task-source>rendering task source</dfn><dd>
    <p>This <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#task-source" id=generic-task-sources:task-source-6>task source</a> is used solely to <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#update-the-rendering" id=generic-task-sources:update-the-rendering>update the rendering</a>.</p>
   </dl>
