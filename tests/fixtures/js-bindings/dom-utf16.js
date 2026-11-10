(() => {
    const text = document.createTextNode("\ud800A\udfff");
    return text.data.length === 3
        && text.data.charCodeAt(0) === 0xd800
        && text.data.charCodeAt(1) === 0x41
        && text.data.charCodeAt(2) === 0xdfff;
})()
