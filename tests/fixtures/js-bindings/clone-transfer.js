(() => {
    function check(condition, message) {
        if (!condition) throw new Error(message);
    }
    function dataCloneError(action) {
        try { action(); } catch (error) { return error.name === "DataCloneError"; }
        return false;
    }
    const duplicate = new ArrayBuffer(4);
    check(dataCloneError(() => structuredClone({}, { transfer: [duplicate, duplicate] })),
        "duplicate transfer entries were accepted");
    check(duplicate.byteLength === 4, "duplicate validation detached the source");

    const failed = new ArrayBuffer(4);
    const failure = new Error("serialization failure");
    let observedFailure;
    try {
        structuredClone({ failed, get stop() { throw failure; } }, { transfer: [failed] });
    } catch (error) { observedFailure = error; }
    check(observedFailure === failure && failed.byteLength === 4,
        "serialization failure changed transfer custody");

    const unreferenced = new ArrayBuffer(4);
    structuredClone({}, { transfer: [unreferenced] });
    check(unreferenced.byteLength === 0, "an unreferenced transfer entry was skipped");

    const shared = new ArrayBuffer(4);
    new Uint8Array(shared)[0] = 37;
    const copy = structuredClone({ first: shared, second: shared }, { transfer: [shared] });
    check(shared.byteLength === 0 && copy.first === copy.second
        && new Uint8Array(copy.first)[0] === 37, "transfer lost aliases or bytes");

    const first = new ArrayBuffer(4);
    const later = new ArrayBuffer(4);
    let beforeTransfer;
    const source = { first, get detachLater() {
        beforeTransfer = first.byteLength;
        structuredClone(later, { transfer: [later] });
        return true;
    } };
    check(dataCloneError(() => structuredClone(source, { transfer: [first, later] })),
        "a reentrantly detached later transfer was accepted");
    check(beforeTransfer === 4 && first.byteLength === 0 && later.byteLength === 0,
        "transfer list order or accepted-prefix effects were lost");
    return true;
})()
