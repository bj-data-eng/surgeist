globalThis.bindingEvents = ["sync-start"];
queueMicrotask(() => {
    bindingEvents.push("outer");
    queueMicrotask(() => bindingEvents.push("inner"));
});
queueMicrotask(() => bindingEvents.push("peer"));
bindingEvents.push("sync-end");
JSON.stringify(bindingEvents)
