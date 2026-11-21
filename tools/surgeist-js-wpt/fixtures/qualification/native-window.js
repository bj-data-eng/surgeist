(() => {
    const fresh = !Object.hasOwn(globalThis, "qualificationState");
    globalThis.qualificationState = false;
    Promise.resolve().then(() => {
        const node = document.appendChild(document.createComment("worker"));
        const event = new Event("qualification", { cancelable: true });
        let observed = false;
        document.addEventListener(event.type, function (delivered) {
            observed = this === document && delivered === event;
            delivered.preventDefault();
        }, { once: true });
        const dispatched = document.dispatchEvent(event);
        globalThis.qualificationState = fresh && window.document === document
            && node.parentNode === document && node.ownerDocument === document
            && observed && !dispatched && event.defaultPrevented;
    });
    return () => globalThis.qualificationState;
})()
