# HTML: microtask checkpoint source

Source: [immutable HTML snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/), revision `877d6146487f15b15bc8dd0e3c625063a37b456a`; complete HTML SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`. Retrieved 2026-10-11. Includes queue/reentrancy definitions and the complete checkpoint algorithm, including external cleanup/timing hooks.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose: [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/); [WHATWG license](../licenses/whatwg/LICENSE). Bounded extraction and Markdown conversion by Surgeist, 2026-10-11 using Pandoc 3.1.11.1, absolute links, no scripts executed. No semantic changes or endorsement. This is evidence for a standalone construction adaptation, not a claim of implementing a browser event loop.

<div>

Each <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id="definitions-3:event-loop-6">event loop</a> has a <span id="microtask-queue" class="dfn">microtask queue</span>, which is a <a href="https://infra.spec.whatwg.org/#queue" id="definitions-3:queue-2" data-x-internal="queue">queue</a> of <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#microtask" id="definitions-3:microtask">microtasks</a>, initially empty. A <span id="microtask" class="dfn" export="">microtask</span> is a colloquial way of referring to a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task" id="definitions-3:concept-task-6">task</a> that was created via the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#queue-a-microtask" id="definitions-3:queue-a-microtask">queue a microtask</a> algorithm.

Each <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id="definitions-3:event-loop-7">event loop</a> has a <span id="performing-a-microtask-checkpoint" class="dfn">performing a microtask checkpoint</span> boolean, which is initially false. It is used to prevent reentrant invocation of the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#perform-a-microtask-checkpoint" id="definitions-3:perform-a-microtask-checkpoint">perform a microtask checkpoint</a> algorithm.

<div algorithm="">

When a user agent is to <span id="perform-a-microtask-checkpoint" class="dfn" export="">perform a microtask checkpoint</span>:

1.  If the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id="event-loop-processing-model:event-loop-15">event loop</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#performing-a-microtask-checkpoint" id="event-loop-processing-model:performing-a-microtask-checkpoint">performing a microtask checkpoint</a> is true, then return.

2.  Set the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id="event-loop-processing-model:event-loop-16">event loop</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#performing-a-microtask-checkpoint" id="event-loop-processing-model:performing-a-microtask-checkpoint-2">performing a microtask checkpoint</a> to true.

3.  While the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id="event-loop-processing-model:event-loop-17">event loop</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#microtask-queue" id="event-loop-processing-model:microtask-queue-2">microtask queue</a> is not <a href="https://infra.spec.whatwg.org/#list-is-empty" id="event-loop-processing-model:list-is-empty" data-x-internal="list-is-empty">empty</a>:

    1.  Let `oldestMicrotask` be the result of <a href="https://infra.spec.whatwg.org/#queue-dequeue" id="event-loop-processing-model:dequeue" data-x-internal="dequeue">dequeuing</a> from the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id="event-loop-processing-model:event-loop-18">event loop</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#microtask-queue" id="event-loop-processing-model:microtask-queue-3">microtask queue</a>.

    2.  Set the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id="event-loop-processing-model:event-loop-19">event loop</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#currently-running-task" id="event-loop-processing-model:currently-running-task-3">currently running task</a> to `oldestMicrotask`.

    3.  Run `oldestMicrotask`.

        This might involve invoking scripted callbacks, which eventually calls the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#clean-up-after-running-script" id="event-loop-processing-model:clean-up-after-running-script">clean up after running script</a> steps, which call this <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#perform-a-microtask-checkpoint" id="event-loop-processing-model:perform-a-microtask-checkpoint-3">perform a microtask checkpoint</a> algorithm again, which is why we use the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#performing-a-microtask-checkpoint" id="event-loop-processing-model:performing-a-microtask-checkpoint-3">performing a microtask checkpoint</a> flag to avoid reentrancy.

    4.  Set the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id="event-loop-processing-model:event-loop-20">event loop</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#currently-running-task" id="event-loop-processing-model:currently-running-task-4">currently running task</a> back to null.

4.  For each <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#environment-settings-object" id="event-loop-processing-model:environment-settings-object-2">environment settings object</a> `settingsObject` whose <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#responsible-event-loop" id="event-loop-processing-model:responsible-event-loop">responsible event loop</a> is this <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id="event-loop-processing-model:event-loop-21">event loop</a>, <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#notify-about-rejected-promises" id="event-loop-processing-model:notify-about-rejected-promises">notify about rejected promises</a> given `settingsObject`'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-settings-object-global" id="event-loop-processing-model:concept-settings-object-global-2">global object</a>.

5.  <a href="https://w3c.github.io/IndexedDB/#cleanup-indexed-database-transactions" id="event-loop-processing-model:cleanup-indexed-database-transactions" data-x-internal="cleanup-indexed-database-transactions">Cleanup Indexed Database transactions</a>.

6.  Perform <a href="https://tc39.es/ecma262/#sec-clear-kept-objects" id="event-loop-processing-model:clearkeptobjects" data-x-internal="clearkeptobjects">ClearKeptObjects</a>().

    When <a href="https://tc39.es/ecma262/#sec-weak-ref.prototype.deref" data-x-internal="weakref.prototype.deref()"><code id="event-loop-processing-model:weakref.prototype.deref()">WeakRef.prototype.deref()</code></a> returns an object, that object is kept alive until the next invocation of <a href="https://tc39.es/ecma262/#sec-clear-kept-objects" id="event-loop-processing-model:clearkeptobjects-2" data-x-internal="clearkeptobjects">ClearKeptObjects</a>(), after which it is again subject to garbage collection.

7.  Set the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id="event-loop-processing-model:event-loop-22">event loop</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#performing-a-microtask-checkpoint" id="event-loop-processing-model:performing-a-microtask-checkpoint-4">performing a microtask checkpoint</a> to false.

8.  <a href="https://w3c.github.io/long-animation-frames/#record-timing-info-for-microtask-checkpoint" id="event-loop-processing-model:record-timing-info-for-microtask-checkpoint" data-x-internal="record-timing-info-for-microtask-checkpoint">Record timing info for microtask checkpoint</a>.

</div>

</div>
