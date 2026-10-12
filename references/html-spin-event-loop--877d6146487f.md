# HTML: spin the event loop

Source: [immutable HTML snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#spin-the-event-loop),
revision 877d6146487f15b15bc8dd0e3c625063a37b456a. Complete original:
14,930,614 bytes, SHA-256
877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f.
The complete algorithm wrapper at original raw lines 81620–81655 is retained:
seven outer steps, both nested two-step lists and all four notes. Surrounding
event-loop algorithms are not included. This is a bounded source conversion,
not evidence that Surgeist implements a browser event loop or Document lifecycle.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose:
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/), with the complete local
[WHATWG license](../licenses/whatwg/LICENSE) and
[edition attribution](../licenses/whatwg/NOTICE.md#pinned-html-language-snapshot--2026-10-11).
The original English snapshot remains authoritative; no endorsement is implied.

The supplied conversion was prepared from the existing capture on 2026-10-12
using Pandoc 3.1.11.1. Surgeist adds this bounded provenance wrapper and visible
Note labels, preserves the algorithm marker and published IDs, presents
variables as inline code, and resolves original links to absolute source URLs.
Algorithm text, conditions, nesting, link labels and order are unchanged under
HTML-collapsible whitespace and block-boundary comparison. There are no tables,
diagrams or executable examples in this selection. No source script or linked
resource was executed or acquired.

The normalized bounded HTML conversion input is 4,215 bytes, SHA-256
a9ae6309c8c229a9953f02c9e01d5e826d638ae826a06f454f46154be410a6be.
It quotes attributes, adds explicit closing list tags and resolves links; it is
not a byte-exact original source excerpt. The actual contiguous original
wrapper is 3,016 bytes, SHA-256
a74d317d6c0fe82edf2496cb8e5c998be863cb04b2181618e4e0231b2b9fe028.
These excerpt hashes are separate from the full original source identity above.

The [EOF source mapping](SOURCE-EDITIONS.md#fresh-standalone-eof-lifecycle-sources)
distinguishes the original algorithm from the separately selected closed
construction reduction. The [retained checkpoint](html-microtask-checkpoint--877d6146487f.md)
supplies the operation invoked before waiting; it does not license dropping that
checkpoint when a wait goal is already satisfied.

<div data-algorithm="">

Algorithm steps that say to <span id="spin-the-event-loop" class="dfn">spin the event loop</span> until a condition `goal` is met are equivalent to substituting in the following algorithm steps:

1.  Let `task` be the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id="event-loop-processing-model:event-loop-23">event loop</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#currently-running-task" id="event-loop-processing-model:currently-running-task-5">currently running task</a>.

    **Note:** `task` could be a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#microtask" id="event-loop-processing-model:microtask-2">microtask</a>.

2.  Let `task source` be `task`'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task-source" id="event-loop-processing-model:concept-task-source">source</a>.

3.  Let `old stack` be a copy of the <a href="https://tc39.es/ecma262/#execution-context-stack" id="event-loop-processing-model:javascript-execution-context-stack" data-x-internal="javascript-execution-context-stack">JavaScript execution context stack</a>.

4.  Empty the <a href="https://tc39.es/ecma262/#execution-context-stack" id="event-loop-processing-model:javascript-execution-context-stack-2" data-x-internal="javascript-execution-context-stack">JavaScript execution context stack</a>.

5.  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#perform-a-microtask-checkpoint" id="event-loop-processing-model:perform-a-microtask-checkpoint-4">Perform a microtask checkpoint</a>.

    **Note:** If `task` is a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#microtask" id="event-loop-processing-model:microtask-3">microtask</a> this step will be a no-op due to <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#performing-a-microtask-checkpoint" id="event-loop-processing-model:performing-a-microtask-checkpoint-5">performing a microtask checkpoint</a> being true.

6.  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#in-parallel" id="event-loop-processing-model:in-parallel-5">In parallel</a>:

    1.  Wait until the condition `goal` is met.

    2.  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#queue-a-task" id="event-loop-processing-model:queue-a-task">Queue a task</a> on `task source` to:

        1.  Replace the <a href="https://tc39.es/ecma262/#execution-context-stack" id="event-loop-processing-model:javascript-execution-context-stack-3" data-x-internal="javascript-execution-context-stack">JavaScript execution context stack</a> with `old stack`.

        2.  Perform any steps that appear after this <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#spin-the-event-loop" id="event-loop-processing-model:spin-the-event-loop-2">spin the event loop</a> instance in the original algorithm.

            **Note:** This resumes `task`.

7.  Stop `task`, allowing whatever algorithm that invoked it to resume.

    **Note:** This causes the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#event-loop" id="event-loop-processing-model:event-loop-24">event loop</a>'s main set of steps or the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#perform-a-microtask-checkpoint" id="event-loop-processing-model:perform-a-microtask-checkpoint-5">perform a microtask checkpoint</a> algorithm to continue.

</div>
