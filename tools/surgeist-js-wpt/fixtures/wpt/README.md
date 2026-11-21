# Initial upstream Window fixtures

The files below are unchanged from Web Platform Tests at commit
[`69f2bba61ce7e9469ccbce53de4644a89f6b7dff`](https://github.com/web-platform-tests/wpt/tree/69f2bba61ce7e9469ccbce53de4644a89f6b7dff).
The pinned [manifest rules](https://github.com/web-platform-tests/wpt/blob/69f2bba61ce7e9469ccbce53de4644a89f6b7dff/tools/manifest/sourcefile.py#L74-L170)
select the `.any.html` Window variant for both tests. The worker inventory
identifies each case by the commit, source path and exact variant URL.

| Retained source | Window variant URL | Git blob |
| --- | --- | --- |
| [EventTarget-addEventListener.any.js](dom/events/EventTarget-addEventListener.any.js) | `/dom/events/EventTarget-addEventListener.any.html` | `e22da4aff850a2f0042c5ecda4c844cc1852f2ec` |
| [EventTarget-add-remove-listener.any.js](dom/events/EventTarget-add-remove-listener.any.js) | `/dom/events/EventTarget-add-remove-listener.any.html` | `b1d7ffb3e07fbe2038f0e33a8e63173edf12234c` |
| [testharness.js](resources/testharness.js) | Shared harness resource | `dd841a084662637b32b7f064b1816ab0b35f89d1` |

The [retained BSD-3-Clause license](../../../../licenses/web-platform-tests/LICENSE.md)
is byte-identical to the [license at the selected revision](https://github.com/web-platform-tests/wpt/blob/69f2bba61ce7e9469ccbce53de4644a89f6b7dff/LICENSE.md),
Git blob `39c46d03ac2988226f949ee7ab3c7347d5481bd8`. The web-platform-tests
contributors own these upstream files.

Both cases use native EventTarget behavior. The [attached startup evidence](https://github.com/bj-data-eng/surgeist/issues/3095)
and the [native Window load task](https://github.com/bj-data-eng/surgeist/issues/2462)
are separate prerequisites for isolated harness completion.
