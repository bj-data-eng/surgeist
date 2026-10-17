# HTML dialog modal assertion: current-source comparison

Source: [HTML Living Standard, interactive elements](https://html.spec.whatwg.org/multipage/interactive-elements.html#show-a-modal-dialog).
Retrieved 2026-10-17; the page reports last updated 2026-10-16.
This is a dated comparison, not a replacement for Surgeist's pinned HTML877d behavior.
The normative implementation basis remains the existing retained interactive/scripting reference.

## Selected source clauses

Manually transcribed Markdown excerpts; omitted intervening steps are identified by the original step numbers. No algorithm is presented as complete.

Dialog attribute change steps, step 4:

> If element's node document is not fully active, then return.

Show a modal dialog, steps 7–12, following the cancelable event:

7. If subject has an open attribute, then return.
8. If subject is not connected, then return.
9. If subject is in the popover showing state, then return.
10. Queue a dialog toggle event task given subject, "closed", "open", and source.
11. Add an open attribute to subject, whose value is the empty string.
12. Assert: subject's close watcher is not null.

## Comparison boundary

These selected clauses retain the pinned edition's ordering. They supply no additional activity check between the event and the assertion. The native failure disposition is an explicit Surgeist integration decision in issue #1225, not a correction attributed to this source.

Source attribution: WHATWG HTML Standard contributors. Source text is available under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/); this excerpt changes formatting and removes inline links while preserving the selected wording.
