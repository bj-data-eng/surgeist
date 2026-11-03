// Real-browser qualification appended only to an invocation-owned helper copy.
// The original fixture and production helper resource remain unchanged.
(() => {
  const configure = configureRootContextCapture;
  const measure = getTestData;
  let configuredArguments;
  configureRootContextCapture = function (...args) {
    configuredArguments = args.slice();
    return configure.apply(this, args);
  };

  const keys = [
    'borderBoxLtrData', 'contentBoxLtrData',
    'borderBoxRtlData', 'contentBoxRtlData',
  ];
  function require(condition, message) {
    if (!condition) throw new Error('Stylesheet custody qualification: ' + message);
  }
  function envelope(raw) {
    require(typeof raw === 'string', 'measurement must retain JSON-string transport');
    const value = JSON.parse(raw);
    require(value.schemaVersion === 4, 'measurement schema');
    require(JSON.stringify(Object.keys(value).sort()) ===
      JSON.stringify([...keys, 'schemaVersion'].sort()), 'complete variant envelope');
    return value;
  }
  function supported(raw) {
    const value = envelope(raw);
    for (const key of keys) {
      require(value[key]?.layoutInput === 'box', key + ': genuine supported root');
      require(value[key].viewport?.rootContext, key + ': actual root context');
    }
  }
  function refused(reason) {
    const value = envelope(measure());
    for (const key of keys) {
      require(value[key]?.layoutInput === 'unsupported', key + ': missing refusal');
      require(value[key].unsupportedReason === reason,
        key + ': expected ' + reason + ', got ' + value[key].unsupportedReason);
      require(JSON.stringify(Object.keys(value[key]).sort()) ===
        JSON.stringify(['layoutInput', 'unsupportedReason']), key + ': exact refusal fields');
    }
  }

  getTestData = function () {
    require(configuredArguments?.length === 4, 'real adapter configure arguments');
    const args = configuredArguments.slice();
    const pristine = measure();
    supported(pristine);
    require(document.styleSheets.length === 1, 'one authenticated stylesheet');
    const owner = document.styleSheets[0].ownerNode;
    require(owner?.tagName === 'STYLE' && owner.parentElement === document.head,
      'actual configured STYLE owner');
    require(owner.getAttribute('data-surgeist-root-source') === 'true', 'owned resource marker');
    const originalText = owner.textContent;
    require(originalText === args[1], 'actual configured resource bytes');

    function sameOwner() {
      require(owner.isConnected && owner.parentElement === document.head,
        'retained owner remains attached');
      require(document.styleSheets.length === 1 &&
        document.styleSheets[0].ownerNode === owner, 'reconfiguration must retain owner identity');
      require(owner.textContent === originalText, 'original resource text retained');
    }
    function repeat() {
      configureRootContextCapture(...args);
      sameOwner();
    }
    function pristineAgain() {
      // Only invoke configure after the actual original owner/rules are restored.
      // No new LINK, replacement owner or fabricated proof repairs a failed test.
      repeat();
      const actual = measure();
      supported(actual);
      require(actual === pristine, 'complete pristine envelope after restoration');
      return actual;
    }

    repeat();
    require(measure() === pristine, 'successful repeated setup preserves complete envelope');

    const sheet = owner.sheet;
    const ruleIndex = sheet.cssRules.length;
    const inserted = sheet.insertRule(':root { --surgeist-custody-probe: cssom; }', ruleIndex);
    try {
      require(inserted === ruleIndex && sheet.cssRules.length === ruleIndex + 1,
        'actual CSSOM mutation executed');
      require(owner.textContent === originalText, 'CSSOM mutation leaves source text unchanged');
      refused('Root context: StylesheetChanged');
      repeat();
      require(owner.sheet === sheet && sheet.cssRules.length === ruleIndex + 1,
        'repeated setup must not replace the sheet or erase the mutation');
      refused('Root context: StylesheetChanged');
    } finally {
      sheet.deleteRule(inserted);
    }
    pristineAgain();

    const extra = document.createElement('style');
    extra.textContent = ':root { --surgeist-custody-probe: unknown; }';
    document.head.appendChild(extra);
    try {
      require(document.styleSheets.length === 2, 'unknown stylesheet is live');
      refused('Root context: StylesheetInventory');
    } finally {
      extra.remove();
    }
    pristineAgain();

    const clone = owner.cloneNode(true);
    require(clone !== owner && clone.outerHTML === owner.outerHTML,
      'identical resource bytes and marker do not establish owner identity');
    owner.replaceWith(clone);
    try {
      require(document.styleSheets.length === 1 &&
        document.styleSheets[0].ownerNode === clone, 'cloned owner is actually selected');
      refused('Root context: StylesheetSource');
    } finally {
      clone.replaceWith(owner);
    }
    pristineAgain();

    const previousAdopted = Array.from(document.adoptedStyleSheets);
    require(previousAdopted.length === 0, 'baseline has no adopted stylesheet');
    const adopted = new CSSStyleSheet();
    adopted.replaceSync(':root { --surgeist-custody-probe: adopted; }');
    document.adoptedStyleSheets = [...previousAdopted, adopted];
    try {
      require(document.adoptedStyleSheets.includes(adopted), 'constructed stylesheet is adopted');
      refused('Root context: AdoptedStylesheet');
    } finally {
      document.adoptedStyleSheets = previousAdopted;
    }
    return pristineAgain();
  };
})();
