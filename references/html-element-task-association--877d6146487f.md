# HTML Standard: element-task association and runnability

Source: [immutable HTML snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#queue-an-element-task), revision `877d6146487f15b15bc8dd0e3c625063a37b456a`. Extracted on 16 October 2026 from the retained complete HTML, SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`. Source ranges: raw lines 81133–81144 and 81195–81232. This excerpt preserves task fields, runnability, and task/global-task/element-task queue algorithms; it does not include the complete event-loop model.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose is [Creative Commons Attribution 4.0 International](https://creativecommons.org/licenses/by/4.0/), with the [exact WHATWG license](../licenses/whatwg/LICENSE). Manual Markdown conversion normalizes whitespace and restores the source definition-list and algorithm-list boundaries. No semantic change or endorsement is implied.

## Task definition and runnability

Formally, a **task** is a struct which has:

- **Steps:** A series of steps specifying the work to be done by the task.
- **A source:** One of the task sources, used to group and serialize related tasks.
- **A document:** A Document associated with the task, or null for tasks that are not in a window event loop.
- **A script evaluation environment settings object set:** A set of environment settings objects used for tracking script evaluation during the task.

A task is **runnable** if its document is either null or fully active.

## Queue a task

To **queue a task** on a task source *source*, which performs a series of steps *steps*, optionally given an event loop *event loop* and a document *document*:

1. If *event loop* was not given, set *event loop* to the implied event loop.
2. If *document* was not given, set *document* to the implied document.
3. Let *task* be a new task.
4. Set *task*'s steps to *steps*.
5. Set *task*'s source to *source*.
6. Set *task*'s document to *document*.
7. Set *task*'s script evaluation environment settings object set to an empty set.
8. Let *queue* be the task queue to which *source* is associated on *event loop*.
9. Append *task* to *queue*.

Failing to pass an event loop and document to queue a task means relying on the ambiguous and poorly-specified implied event loop and implied document concepts. Specification authors should either always pass these values, or use the wrapper algorithms queue a global task or queue an element task instead. Using the wrapper algorithms is recommended.

## Queue a global task

To **queue a global task** on a task source *source*, with a global object *global* and a series of steps *steps*:

1. Let *event loop* be *global*'s relevant agent's event loop.
2. Let *document* be *global*'s associated Document, if *global* is a Window object; otherwise null.
3. Queue a task given *source*, *event loop*, *document*, and *steps*.

## Queue an element task

To **queue an element task** on a task source *source*, with an element *element* and a series of steps *steps*:

1. Let *global* be *element*'s relevant global object.
2. Queue a global task given *source*, *global*, and *steps*.

## Scope and source links

The immutable source defines [task fields](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task), [runnability](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-task-runnable), [queue a task](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#queue-a-task), [queue a global task](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#queue-a-global-task), and [queue an element task](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#queue-an-element-task). These clauses do not define Rust canonical identity retirement. The independently selected Dialog retirement policy belongs to [#136](https://github.com/bj-data-eng/surgeist/issues/136). This excerpt does not authorize JavaScript execution, a Window implementation, fake null Document associations, a new task scheduler, or completion of hosted lifecycle requirements.
