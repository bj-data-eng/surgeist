# HTML Standard: close-watcher cancel result

Source: [immutable HTML snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#close-watcher-infrastructure), revision `877d6146487f15b15bc8dd0e3c625063a37b456a`. Extracted on 17 October 2026 from the retained complete HTML, SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`. Source ranges: raw lines 61982–61988 (complete cancel-action item), 62024–62047 (request-to-close algorithm and note), 62050–62058 (close algorithm), and 46630–46632 (Dialog cancel-action item). Other close-watcher machinery remains outside this bounded excerpt.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose is [Creative Commons Attribution 4.0 International](https://creativecommons.org/licenses/by/4.0/), with the [exact WHATWG license](../licenses/whatwg/LICENSE). Manual Markdown conversion normalizes whitespace and restores list boundaries. The contradictory boolean in the source is preserved below. The implementation determination is owned by [Surgeist issue #1225](https://github.com/bj-data-eng/surgeist/issues/1225), not silently substituted into this source transcription.

## Cancel action

A cancel action is an algorithm accepting a boolean argument and returning a boolean. The argument indicates whether or not the cancel action algorithm can prevent the close request from proceeding via the algorithm's return value. If the boolean argument is true, then the algorithm can return either true to indicate that the caller will proceed to the close action, or false to indicate that the caller will bail out. If the argument is false, then the return value is always false. This algorithm can never throw an exception.

## Request to close

To request to close a close watcher *closeWatcher* with boolean *requireHistoryActionActivation*:

1. If *closeWatcher* is not active, then return true.
2. If the result of running *closeWatcher*'s get enabled state is false, then return true.
3. If *closeWatcher*'s is running cancel action is true, then return true.
4. Let *window* be *closeWatcher*'s window.
5. If *window*'s associated Document is not fully active, then return true.
6. Let *canPreventClose* be true if *requireHistoryActionActivation* is false, or if *window*'s close watcher manager's groups's size is less than *window*'s close watcher manager's allowed number of groups, and *window* has history-action activation; otherwise false.
7. Set *closeWatcher*'s is running cancel action to true.
8. Let *shouldContinue* be the result of running *closeWatcher*'s cancel action given *canPreventClose*.
9. Set *closeWatcher*'s is running cancel action to false.
10. If *shouldContinue* is false:
    1. Assert: *canPreventClose* is true.
    2. Consume history-action user activation given *window*.
    3. Return false.

    Note that since these substeps consume history-action user activation, requesting to close a close watcher twice without any intervening user activation will result in *canPreventClose* being false the second time.

11. Close *closeWatcher*.
12. Return true.

## Close

To close a close watcher *closeWatcher*:

1. If *closeWatcher* is not active, then return.
2. If the result of running *closeWatcher*'s get enabled state is false, then return.
3. If *closeWatcher*'s window's associated Document is not fully active, then return.
4. Destroy *closeWatcher*.
5. Run *closeWatcher*'s close action.

## Dialog cancel-action callback

In set the dialog close watcher, the supplied *cancelAction* given *canPreventClose* is to return the result of firing an event named `cancel` at *dialog*, with the `cancelable` attribute initialized to *canPreventClose*.

## Related retained source

[DOM native events](dom-native-events--3071e5f26944--332564d9519c.md) retains event creation, setting the canceled flag, dispatch result and firing algorithms. [HTML interactive/scripting elements](html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md) retains the surrounding Dialog setup and lifecycle algorithms. No language revision or browser override is introduced by this excerpt.
