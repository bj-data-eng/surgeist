# HTML Standard: Window close-watcher manager

Source: [immutable HTML snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#close-watcher-infrastructure), revision `877d6146487f15b15bc8dd0e3c625063a37b456a`.
Complete original HTML SHA-256: `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
Extracted offline on 17 October 2026. Each selected source unit remains byte-identical in its inert HTML fence, including original links, IDs, whitespace and branches. No source script or renderer ran.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). The unchanged complete [WHATWG license](../licenses/whatwg/LICENSE) supplies CC BY 4.0 terms for specification prose and a separate BSD 3-Clause provision for portions incorporated into code. The [modification notice](../licenses/whatwg/NOTICE.md#close-watcher-manager--2026-10-17) records this bounded selection and added Markdown framing.

## Reading guide

These complete selected units define the original Window manager defaults, watcher fields and active-membership test, establishment and destruction. Anti-abuse explanation, user-activation notification, request-close, close and platform processing remain outside this capture. The [cancel-result reference](html-close-watcher-cancel-result--877d6146487f.md) separately retains the reached request-close/close clauses and literal contradictory descriptive boolean; [issue #1225](https://github.com/bj-data-eng/surgeist/issues/1225) owns the narrow interpretation. No source boolean is corrected inside these literal blocks. Retention supplies source evidence and does not establish implementation or full browser conformance.

Fragment links inside the inert fences preserve their original source spelling; resolve them against the immutable complete snapshot above. Markdown headings and the reading guide are editorial additions.

## Window manager defaults

Inclusive raw lines 61929–61937, 1,256 bytes; SHA-256 `9eb6d311bd1fe34753584adbfb4f4f84f09c16cf9b9ad8f4509dbea0e6c1aa5f`.

```html
  <h4 id=close-watcher-infrastructure><span class=secno>6.10.2</span> Close watcher infrastructure<a href=#close-watcher-infrastructure class=self-link></a></h4>

  <p>Each <code id=close-watcher-infrastructure:window><a href=#window>Window</a></code> has a <dfn id=close-watcher-manager>close watcher manager</dfn>, which is a
  <a id=close-watcher-infrastructure:struct href=https://infra.spec.whatwg.org/#struct data-x-internal=struct>struct</a> with the following <a href=https://infra.spec.whatwg.org/#struct-item id=close-watcher-infrastructure:struct-item data-x-internal=struct-item>items</a>:</p>

  <ul><li><p><dfn id=close-watcher-manager-groups>Groups</dfn>, a <a id=close-watcher-infrastructure:list href=https://infra.spec.whatwg.org/#list data-x-internal=list>list</a> of <a href=https://infra.spec.whatwg.org/#list id=close-watcher-infrastructure:list-2 data-x-internal=list>lists</a> of <a href=#close-watcher id=close-watcher-infrastructure:close-watcher>close watchers</a>, initially
   empty.<li><p><dfn id=close-watcher-manager-allowed-number-of-groups>Allowed number of
   groups</dfn>, a number, initially 1.<li><p><dfn id=close-watcher-manager-next>Next user interaction allows a new group</dfn>,
   a boolean, initially true.</ul>
```

## Watcher definition, activity and establishment

Inclusive raw lines 61980–62022, 6,501 bytes; SHA-256 `1d1801d55815ba0a2d512eb3ba67916301a939aab25f80f5ba8d9580cae6ec5b`.

```html
  <p>A <dfn id=close-watcher data-export="">close watcher</dfn> is a <a id=close-watcher-infrastructure:struct-2 href=https://infra.spec.whatwg.org/#struct data-x-internal=struct>struct</a> with the following <a href=https://infra.spec.whatwg.org/#struct-item id=close-watcher-infrastructure:struct-item-2 data-x-internal=struct-item>items</a>:</p>

  <ul><li><p>A <dfn id=close-watcher-window>window</dfn>, a <code id=close-watcher-infrastructure:window-3><a href=#window>Window</a></code>.<li><p>A <dfn id=close-watcher-cancel-action>cancel action</dfn>, an algorithm accepting a
   boolean argument and returning a boolean. The argument indicates whether or not the cancel
   action algorithm can prevent the close request from proceeding via the algorithm's return value.
   If the boolean argument is true, then the algorithm can return either true to indicate that the
   caller will proceed to the <a href=#close-watcher-close-action id=close-watcher-infrastructure:close-watcher-close-action>close action</a>, or
   false to indicate that the caller will bail out. If the argument is false, then the return value
   is always false. This algorithm can never throw an exception.<li><p>A <dfn id=close-watcher-close-action>close action</dfn>, an algorithm accepting no
   arguments and returning nothing. This algorithm can never throw an exception.<li><p>An <dfn id=close-watcher-is-running-cancel>is running cancel action</dfn>
   boolean.<li><p>A <dfn id=close-watcher-get-enabled-state>get enabled state</dfn>, an algorithm
   accepting no arguments and returning a boolean. This algorithm can never throw an
   exception.</ul>

  <div data-algorithm="">
  <p>A <a href=#close-watcher id=close-watcher-infrastructure:close-watcher-4>close watcher</a> <var>closeWatcher</var> <dfn id=close-watcher-active>is
  active</dfn> if <var>closeWatcher</var>'s <a href=#close-watcher-window id=close-watcher-infrastructure:close-watcher-window>window</a>'s
  <a href=#close-watcher-manager id=close-watcher-infrastructure:close-watcher-manager-3>close watcher manager</a> <a href=https://infra.spec.whatwg.org/#list-contain id=close-watcher-infrastructure:list-contains data-x-internal=list-contains>contains</a> any list which
  <a href=https://infra.spec.whatwg.org/#list-contain id=close-watcher-infrastructure:list-contains-2 data-x-internal=list-contains>contains</a> <var>closeWatcher</var>.</p>
  </div>

  <hr>

  <div data-algorithm="">
  <p>To <dfn id=establish-a-close-watcher>establish a close watcher</dfn> given a <code id=close-watcher-infrastructure:window-4><a href=#window>Window</a></code> <var>window</var>, a list
  of steps <dfn id=create-close-watcher-cancelaction><var>cancelAction</var></dfn>, a
  list of steps <dfn id=create-close-watcher-closeaction><var>closeAction</var></dfn>,
  and an algorithm that returns a boolean <dfn id=create-close-watcher-getenabledstate><var>getEnabledState</var></dfn>:</p>

  <ol><li><p><a id=close-watcher-infrastructure:assert href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: <var>window</var>'s <a href=#concept-document-window id=close-watcher-infrastructure:concept-document-window>associated <code>Document</code></a> is <a href=#fully-active id=close-watcher-infrastructure:fully-active>fully
   active</a>.<li>
    <p>Let <var>closeWatcher</var> be a new <a href=#close-watcher id=close-watcher-infrastructure:close-watcher-5>close watcher</a>, with</p>

    <dl class=props><dt><a href=#close-watcher-window id=close-watcher-infrastructure:close-watcher-window-2>window</a><dd><var>window</var><dt><a href=#close-watcher-cancel-action id=close-watcher-infrastructure:close-watcher-cancel-action>cancel action</a><dd><var>cancelAction</var><dt><a href=#close-watcher-close-action id=close-watcher-infrastructure:close-watcher-close-action-2>close action</a><dd><var>closeAction</var><dt><a href=#close-watcher-is-running-cancel id=close-watcher-infrastructure:close-watcher-is-running-cancel>is running cancel action</a><dd>false<dt><a href=#close-watcher-get-enabled-state id=close-watcher-infrastructure:close-watcher-get-enabled-state>get enabled state</a><dd><var>getEnabledState</var></dl>
   <li><p>Let <var>manager</var> be <var>window</var>'s <a href=#close-watcher-manager id=close-watcher-infrastructure:close-watcher-manager-4>close watcher manager</a>.<li><p>If <var>manager</var>'s <a href=#close-watcher-manager-groups id=close-watcher-infrastructure:close-watcher-manager-groups>groups</a>'s <a href=https://infra.spec.whatwg.org/#list-size id=close-watcher-infrastructure:list-size data-x-internal=list-size>size</a> is less than <var>manager</var>'s <a href=#close-watcher-manager-allowed-number-of-groups id=close-watcher-infrastructure:close-watcher-manager-allowed-number-of-groups-3>allowed number of groups</a>, then <a href=https://infra.spec.whatwg.org/#list-append id=close-watcher-infrastructure:list-append data-x-internal=list-append>append</a> « <var>closeWatcher</var> » to <var>manager</var>'s <a href=#close-watcher-manager-groups id=close-watcher-infrastructure:close-watcher-manager-groups-2>groups</a>.<li>
    <p>Otherwise:</p>

    <ol><li><p><a id=close-watcher-infrastructure:assert-2 href=https://infra.spec.whatwg.org/#assert data-x-internal=assert>Assert</a>: <var>manager</var>'s <a href=#close-watcher-manager-groups id=close-watcher-infrastructure:close-watcher-manager-groups-3>groups</a>'s <a href=https://infra.spec.whatwg.org/#list-size id=close-watcher-infrastructure:list-size-2 data-x-internal=list-size>size</a> is at least 1 in this branch,
     since <var>manager</var>'s <a href=#close-watcher-manager-allowed-number-of-groups id=close-watcher-infrastructure:close-watcher-manager-allowed-number-of-groups-4>allowed number of groups</a> is always at least 1.<li><p><a href=https://infra.spec.whatwg.org/#list-append id=close-watcher-infrastructure:list-append-2 data-x-internal=list-append>Append</a> <var>closeWatcher</var> to
     <var>manager</var>'s <a href=#close-watcher-manager-groups id=close-watcher-infrastructure:close-watcher-manager-groups-4>groups</a>'s last <a href=https://infra.spec.whatwg.org/#list-item id=close-watcher-infrastructure:list-item data-x-internal=list-item>item</a>.</ol>
   <li><p>Set <var>manager</var>'s <a href=#close-watcher-manager-next id=close-watcher-infrastructure:close-watcher-manager-next-4>next user interaction
   allows a new group</a> to true.<li><p>Return <var>closeWatcher</var>.</ol>
  </div>
```

## Destruction

Inclusive raw lines 62060–62067, 1,415 bytes; SHA-256 `6d08391f947d16e20c2ed188949fcb9ff260e73c7b472dd01d392e88e03ae5b9`.

```html
  <div data-algorithm="">
  <p>To <dfn id=close-watcher-destroy>destroy</dfn> a <a href=#close-watcher id=close-watcher-infrastructure:close-watcher-9>close watcher</a>
  <var>closeWatcher</var>:</p>

  <ol><li><p>Let <var>manager</var> be <var>closeWatcher</var>'s <a href=#close-watcher-window id=close-watcher-infrastructure:close-watcher-window-5>window</a>'s <a href=#close-watcher-manager id=close-watcher-infrastructure:close-watcher-manager-7>close watcher manager</a>.<li><p><a href=https://infra.spec.whatwg.org/#list-iterate id=close-watcher-infrastructure:list-iterate data-x-internal=list-iterate>For each</a> <var>group</var> of <var>manager</var>'s
   <a href=#close-watcher-manager-groups id=close-watcher-infrastructure:close-watcher-manager-groups-6>groups</a>: <a href=https://infra.spec.whatwg.org/#list-remove id=close-watcher-infrastructure:list-remove data-x-internal=list-remove>remove</a> <var>closeWatcher</var> from <var>group</var>.<li><p><a href=https://infra.spec.whatwg.org/#list-remove id=close-watcher-infrastructure:list-remove-2 data-x-internal=list-remove>Remove</a> any item from <var>manager</var>'s <a href=#close-watcher-manager-groups id=close-watcher-infrastructure:close-watcher-manager-groups-7>groups</a> that <a href=https://infra.spec.whatwg.org/#list-is-empty id=close-watcher-infrastructure:list-is-empty data-x-internal=list-is-empty>is
   empty</a>.</ol>
  </div>
```
