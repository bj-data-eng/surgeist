(() => {
    const parent = document.createElement("div");
    const child = document.createTextNode("live");
    parent.appendChild(child);
    if (parent.firstChild !== child || child.parentNode !== parent) {
        throw new Error("native wrapper identity changed");
    }
    child.data = "updated";
    if (parent.textContent !== "updated") {
        throw new Error("a live read returned old native state");
    }
    return parent.removeChild(child) === child && child.parentNode === null;
})()
