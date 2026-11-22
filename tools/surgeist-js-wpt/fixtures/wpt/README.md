# Initial upstream Window fixtures

The files below are unchanged from Web Platform Tests at commit
[`69f2bba61ce7e9469ccbce53de4644a89f6b7dff`](https://github.com/web-platform-tests/wpt/tree/69f2bba61ce7e9469ccbce53de4644a89f6b7dff).
The pinned [manifest rules](https://github.com/web-platform-tests/wpt/blob/69f2bba61ce7e9469ccbce53de4644a89f6b7dff/tools/manifest/sourcefile.py#L74-L170)
select the `.any.html` Window variant for both EventTarget tests. The worker inventory
identifies each case by the commit, source path and exact variant URL.

| Retained source | Window variant URL | Git blob |
| --- | --- | --- |
| [EventTarget-addEventListener.any.js](dom/events/EventTarget-addEventListener.any.js) | `/dom/events/EventTarget-addEventListener.any.html` | `e22da4aff850a2f0042c5ecda4c844cc1852f2ec` |
| [EventTarget-add-remove-listener.any.js](dom/events/EventTarget-add-remove-listener.any.js) | `/dom/events/EventTarget-add-remove-listener.any.html` | `b1d7ffb3e07fbe2038f0e33a8e63173edf12234c` |
| [HTMLCollection-as-prototype.html](dom/collections/HTMLCollection-as-prototype.html) | `/dom/collections/HTMLCollection-as-prototype.html` | `d572d35c041496904901a9eb900e84f2f5b63704` |
| [testharness.js](resources/testharness.js) | Shared harness resource | `dd841a084662637b32b7f064b1816ab0b35f89d1` |
| [testharnessreport.js](resources/testharnessreport.js) | HTMLCollection harness support resource | `405a2d8b06f00fe8292e0e9d5b917e193cfd416c` |

The [retained BSD-3-Clause license](../../../../licenses/web-platform-tests/LICENSE.md)
is byte-identical to the [license at the selected revision](https://github.com/web-platform-tests/wpt/blob/69f2bba61ce7e9469ccbce53de4644a89f6b7dff/LICENSE.md),
Git blob `39c46d03ac2988226f949ee7ab3c7347d5481bd8`. The web-platform-tests
contributors own these upstream files.

The HTMLCollection page runs on a native attached Boa Window. The worker
serves the two embedded scripts at their exact `/resources/` URLs, evaluates
each script at its original parser pause and records the harness's public
completion callback after the native Window load task and timer.

The two generated EventTarget Window variants remain in the inventory without
execution support.
