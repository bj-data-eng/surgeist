# Child destruction and queued history: source observations

Inspected on 16 October 2026. This Markdown reference records source provenance
and behavioral summaries. It does not transcribe browser code, ingest an engine,
or replace the retained HTML algorithms.

## Selected standard and reached clauses

HTML remains pinned at `877d6146487f15b15bc8dd0e3c625063a37b456a`;
complete captured source SHA-256
`877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
These existing faithful references retain the used source bodies:

- [Navigable lifecycle](html-hosted-navigable-lifecycle--877d6146487f--3a9658155866.md):
  queued child creation at original raw71126–71135 and child destruction at
  raw71269–71285.
- [History and traversal](html-child-navigable-history--877d6146487f--a9f635b298df.md):
  traversal queue raw72221–72275, entry lookup raw72399–72419, used steps
  raw72473–72491, history application raw74206–74487, live child enumeration
  raw74646–74721.
- [Document destruction](html-hosted-initial-teardown--877d6146487f--c2dd632ac56d.md):
  raw75868–75923, including queued-task removal without execution and the
  asynchronous descendant/Document destruction sequence.
- [Task association](html-element-task-association--877d6146487f.md):
  raw81133–81144 and raw81195–81232 retain task Document, runnability,
  queue-task and actual Global/Document association.
- [Hosted globals](html-hosted-global-agent-settings--877d6146487f--03a314e0aa55.md)
  retains the associated Document and agent/event-loop contracts.

Reading these clauses in order gives separate concerns. If removal precedes
the captured creation-history closure, the synchronous nested-history removal
has no existing matching row. The later creation closure still appends its
captured entry; the later destruction-history closure only applies the history
update. No late pruning or creation-cancellation branch occurs in these bodies.
The historical entry can outlive its Document. This does not make it a live child.

The nonchanging-history wait increments its completion counter only inside
queued task bodies. Document destruction removes associated queued tasks without
running them. No explicit cancellation settlement is supplied there. Child
destruction also clears its container link before scheduling destruction on
the now-inactive Document's Global, while ordinary task runnability requires
a null or fully active task Document.

Recursive destruction adds a second completion obligation. At raw75906–75921,
each captured child receives a queued wrapper which starts recursive destruction
with an increment callback; the parent waits for those callbacks before queuing
its own destruction. With overlapping invocations, actual Document destruction
can remove either another invocation's wrapper or its final task. Removing such
a task does not run its callback. The source bodies do not specify how that
other invocation retains or settles its obligation. This is distinct from the
nonchanging-history counter gap; completion of actual destruction and cancellation
of a task remain different facts.

Two initial children can also be created before either queued creation closure
runs. During the first closure's history update, the helpers at raw74646–74721
and raw74743–74788 can enumerate the second live child before that child's
closure has attached its nested-history row. The entry lookup at raw72399–72419
then has no matching row; the target-entry helper has no pending-initial-entry
branch. Waiting for the second closure would wait behind the current FIFO head.
The actual captured initial entry already exists, with its step initially
pending (raw72024 onward); existence of that entry and attachment to history are
distinct phases. The same source helpers in the current standard were checked
on 16 October 2026 and still contain no special pending-initial-child case.

[WHATWG HTML #12724](https://github.com/whatwg/html/issues/12724), open when
inspected on 16 October 2026, documents a different creation/destruction
bookkeeping conflict with a pending reload. It supplies no adopted fix for the
two-initial-child case. Reload and changing-document traversal are outside the
initial-entry qualification considered here.

There is also a separate parent lookup in the creation closure at raw71126–71129.
If parent child P already has an attached history, creating B inside P queues
C_B. Removing P before C_B runs removes P's nested-history row immediately.
C_B's later lookup of P's entries can then fail even though P's own entry step
was already numeric. Retaining P's identity does not by itself change that
lookup's root-reachable search domain. A native qualification must distinguish
custody of the actual removed entry collection from reinserting it into the
traversable's history; the source bodies do not specify that fallback.

## Newer standard and upstream discussion

The current [child destruction](https://html.spec.whatwg.org/multipage/document-sequences.html#destroy-a-child-navigable)
and [history application](https://html.spec.whatwg.org/multipage/browsing-the-web.html#apply-the-history-step)
were checked on 16 October 2026. They retain the relevant ordering and the
counter wait; this check adopts no new HTML edition.

[WHATWG HTML #10242](https://github.com/whatwg/html/issues/10242), open when
inspected, reports the exact inactive-Document destruction-task problem.
The [latest inspected discussion](https://github.com/whatwg/html/issues/10242#issuecomment-2614972131)
questions an extra destroyed flag and discusses delaying content-link clearing.
That discussion is not an adopted algorithm or authority to keep a removed
container publicly associated.

[WHATWG HTML #9690](https://github.com/whatwg/html/issues/9690) is closed.
The [editor's discussion](https://github.com/whatwg/html/issues/9690#issuecomment-1712832571)
explains that captured navigables can outlive destruction and that the standard's
parallel frame-tree lifetime discipline is incomplete.
The [closure comment](https://github.com/whatwg/html/issues/9690#issuecomment-1742175006)
leaves checks to implementers; closure is not evidence of a normative fix
for the present traversal wait.

These are dated, paraphrased issue observations, not copied discussion transcripts.

## Pinned browser witnesses

WebKit revision: `73aa6c89e2cb77c46184a81aec944e4ab99d114d`.
Chromium/Blink revision: `7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2`.
Git blob identifiers name the complete fetched files; locations are one-based.

| File | Git blob | Lines | Bounded observation |
| --- | --- | --- | --- |
| WebKit [FrameTree.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/page/FrameTree.cpp) | `add3b84805afa72ac2bb576c9dde17bb4be5cfa3` | 59–66, 105–110, 134–147 | Specified-name storage accepts the supplied atom and clearName uses the null atom. Detaching clears the live parent relation; removal also unlinks the sibling chain. |
| WebKit [FrameLoader.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/loader/FrameLoader.cpp) | `535f3f57176fa0574a5bd6af779a1cb7550356b8` | 3450–3508 | Frame removal retains the actual Frame while running cleanup and detaching it from its parent. |
| WebKit [EventLoop.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/dom/EventLoop.cpp) | `3abc026892eaecbcb2cedb800d863fa75a7eeaa8` | 305–310, 335–349, 444–455 | Stopped groups lose queued tasks; stopped or missing groups are skipped without executing task bodies. |
| Chromium [frame_tree.cc](https://github.com/chromium/chromium/blob/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/content/browser/renderer_host/frame_tree.cc) | `0db015a39a1cf3206f7cf17274a8decb63a43ff6` | 578–585 | Frame removal invokes the actual parent's child-removal operation. |
| Chromium [frame_tree_node.cc](https://github.com/chromium/chromium/blob/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/content/browser/renderer_host/frame_tree_node.cc) | `4e4b14ebc84ab4e570ad46bb4622e454a2fbf9d9` | 195–257, 290–302 | Destruction clears live lookup identity and discards pending navigation; separately, a guarded routine can remove script-created frame history. |
| Chromium [navigation_controller_impl.cc](https://github.com/chromium/chromium/blob/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/content/browser/renderer_host/navigation_controller_impl.cc) | `580d1a893d488d255329cddefc9aeaf4bcf8a67a` | 5452–5466 | History index/length propagation broadcasts to available page endpoints, rather than exposing the HTML nonchanging-job counter algorithm. |

These witnesses support separating live tree membership, retained cleanup
ownership, task execution and task cancellation. They do not implement the
exact HTML creation closure/nonchanging counter sequence, and therefore do
not independently settle its missing cancellation rule. Chromium's separate
conditional history-removal routine does not authorize unconditional deletion
of the retained HTML closure's later row. WebKit's synchronous loader/unload
behavior is not adopted as Surgeist scheduling.

Original copyrights and licenses remain in the linked files and the
[WebKit licensing inventory](https://webkit.org/licensing-webkit/) and
[Chromium license](https://github.com/chromium/chromium/blob/7984f9d11800ff86ef6c32f4b44c72b4b2fe8ab2/LICENSE).
No source-code copyright notice is stripped or browser implementation copied.

## Child target-name type conflict

HTML877d raw71115–71120 initializes the child targetName operand to null, reads
the name attribute only after browsing-context/Document creation, and directly
assigns that operand to the initial DocumentState. The general DocumentState
definition at raw72158–72159 instead declares a string initially empty.
Initialization at raw70926–70946 supplies no conversion. All three clauses are
already retained in the linked navigable-lifecycle reference. The current
[child constructor](https://html.spec.whatwg.org/multipage/document-sequences.html#create-a-new-child-navigable)
and [DocumentState definition](https://html.spec.whatwg.org/multipage/browsing-the-web.html#document-state)
were checked on 16 October 2026 and retain this conflict; no newer edition is
adopted. WebKit's specified-name storage above can retain a null atom, but it is
not the HTML DocumentState model and does not establish a normative conversion
between null and an empty string. The selected native initial-state typing
qualification belongs to #1124; the original source text remains unchanged.

Any selected native lifetime, target-name typing, cleanup eligibility or cancellation-settlement
qualification must be recorded separately in the owning
[#1124](https://github.com/bj-data-eng/surgeist/issues/1124) and
[#136](https://github.com/bj-data-eng/surgeist/issues/136) contracts. This source
record alone does not authorize implementation or claim browser conformance.
