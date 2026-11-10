// Invocation-only qualification of actual stylesheet or loaded-font custody.
// Installation never changes original fixture/helper resources or invents results.
function installCustodyProbe(mode, expectedParagraphs, expectedGrids) {
  const configure = configureRootContextCapture;
  const measure = getTestData;
  const prepareFont = prepareNamedReferenceFont;
  let configuredArguments;
  const label = mode === 'font' ? 'Font custody qualification: ' : 'Stylesheet custody qualification: ';
  function require(condition, message) {
    if (!condition) throw new Error(label + message);
  }
  require(mode === 'font' || mode === 'stylesheet', 'closed custody mode');
  require(Number.isSafeInteger(expectedParagraphs) && expectedParagraphs >= 0 &&
    Number.isSafeInteger(expectedGrids) && expectedGrids >= 0 &&
    (mode === 'font' ? expectedParagraphs > 0 || expectedGrids > 0 : expectedParagraphs === 0 && expectedGrids === 0), 'declared custody applicability');
  configureRootContextCapture = function (...args) {
    configuredArguments = args.slice();
    return configure.apply(this, args);
  };

  const keys = [
    'borderBoxLtrData', 'contentBoxLtrData',
    'borderBoxRtlData', 'contentBoxRtlData',
  ];
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
      require(value[key]?.layoutInput === 'box', key + ': supported root; got ' +
        (value[key]?.unsupportedReason ?? value[key]?.layoutInput ?? 'missing'));
      require(value[key].viewport?.rootContext, key + ': actual root context');
      require(value[key].gridAlignmentProfile?.profile === 1 &&
        value[key].gridAlignmentProfile.owners?.length === expectedGrids, key + ': complete source-owned Grid reference profile');
      const paragraphs = [], grids = [];
      function collect(node) {
        if (node.atomicParagraphMetrics) paragraphs.push(node.atomicParagraphMetrics);
        if (node.gridAlignmentReference) grids.push(node.gridAlignmentReference);
        for (const child of node.children || []) collect(child);
      }
      collect(value[key]);
      require(paragraphs.length === expectedParagraphs,
        key + ': declared named paragraph certificates; expected ' + expectedParagraphs + ', got ' + paragraphs.length);
      require(grids.length === expectedGrids, key + ': declared own Grid references; expected ' + expectedGrids + ', got ' + grids.length);
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
  function context() {
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
    return { args, pristine, owner, originalText };
  }
  function sameOwner({ owner, originalText }) {
    require(owner.isConnected && owner.parentElement === document.head,
      'retained owner remains attached');
    require(document.styleSheets.length === 1 &&
      document.styleSheets[0].ownerNode === owner, 'reconfiguration must retain owner identity');
    require(owner.textContent === originalText, 'original resource text retained');
  }
  function repeat(ctx) {
    configureRootContextCapture(...ctx.args);
    sameOwner(ctx);
  }
  function actualPristine(ctx) {
    const actual = measure();
    supported(actual);
    require(actual === ctx.pristine, 'complete pristine envelope after restoration');
    return actual;
  }
  function pristineAgain(ctx) {
    // Configure only after the actual original owner/rules have been restored.
    // No replacement owner or fabricated proof repairs an invalid capture.
    repeat(ctx);
    return actualPristine(ctx);
  }

  function foreignNamespaceScriptCustody(ctx) {
    const scriptCount = document.querySelectorAll('script').length;
    const htmlScriptCount = document.scripts.length;
    const extra = document.createElementNS('http://www.w3.org/2000/svg', 'script');
    extra.setAttribute('type', 'application/json');
    extra.textContent = '{}';
    document.head.appendChild(extra);
    try {
      require(extra.isConnected && document.querySelectorAll('script').length === scriptCount + 1 &&
        Array.from(document.querySelectorAll('script')).includes(extra) &&
        document.scripts.length === htmlScriptCount,
        'actual SVG script changes the full script census without changing the HTML script census');
      refused('Root context: ScriptChanged');
      repeat(ctx);
      refused('Root context: ScriptChanged');
    } finally {
      extra.remove();
    }
    pristineAgain(ctx);
  }

  function scriptlessSourceCustody(ctx) {
    const source = new DOMParser().parseFromString(ctx.args[0], 'text/html');
    if (source.querySelectorAll('script').length !== 0) return;
    require(document.scripts.length === 0, 'scriptless source retains no live script elements');
    repeat(ctx);
    actualPristine(ctx);

    // Even the expected helper URL cannot authorize a DOM script when the
    // original source has none. This data block never executes or loads code.
    const extra = document.createElement('script');
    extra.type = 'application/json';
    extra.src = new URL(ctx.args[2], document.baseURI).href;
    document.head.appendChild(extra);
    try {
      require(document.scripts.length === 1 && document.scripts[0] === extra,
        'actual script census mutation executed');
      refused('Root context: ScriptChanged');
    } finally {
      extra.remove();
    }
    actualPristine(ctx);
    pristineAgain(ctx);
  }

  if (mode === 'stylesheet') {
    getTestData = function () {
      const ctx = context();
      foreignNamespaceScriptCustody(ctx);
      const { owner, originalText, pristine } = ctx;
      repeat(ctx);
      require(measure() === pristine, 'successful repeated setup preserves complete envelope');
      scriptlessSourceCustody(ctx);

      const sheet = owner.sheet;
      const ruleIndex = sheet.cssRules.length;
      const inserted = sheet.insertRule(':root { --surgeist-custody-probe: cssom; }', ruleIndex);
      try {
        require(inserted === ruleIndex && sheet.cssRules.length === ruleIndex + 1,
          'actual CSSOM mutation executed');
        require(owner.textContent === originalText, 'CSSOM mutation leaves source text unchanged');
        refused('Root context: StylesheetChanged');
        repeat(ctx);
        require(owner.sheet === sheet && sheet.cssRules.length === ruleIndex + 1,
          'repeated setup must not replace the sheet or erase the mutation');
        refused('Root context: StylesheetChanged');
      } finally {
        sheet.deleteRule(inserted);
      }
      pristineAgain(ctx);

      const extra = document.createElement('style');
      extra.textContent = ':root { --surgeist-custody-probe: unknown; }';
      document.head.appendChild(extra);
      try {
        require(document.styleSheets.length === 2, 'unknown stylesheet is live');
        refused('Root context: StylesheetInventory');
      } finally {
        extra.remove();
      }
      pristineAgain(ctx);

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
      pristineAgain(ctx);

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
      return pristineAgain(ctx);
    };
    return;
  }

  require(typeof prepareFont === 'function', 'actual font-preparation owner');
  let completed;
  prepareNamedReferenceFont = async function (...args) {
    completed = undefined;
    await prepareFont.apply(this, args);
    const ctx = context();
    const state = namedReferenceFontState;
    let firstReference;
    function firstDemand(node) {
      for (const child of node.children || []) firstDemand(child);
      if (!firstReference && node.atomicParagraphMetrics) firstReference = 'Atomic paragraph metrics: ';
      if (!firstReference && node.gridAlignmentReference) firstReference = 'Grid alignment reference: ';
    }
    firstDemand(envelope(ctx.pristine)[keys[0]]);
    require(firstReference, 'actual source-owned named reference demand');
    const face = state?.face;
    require(state?.owner === ctx.owner && state.loaded && !state.error &&
      face?.status === 'loaded' && document.fonts.has(face), 'actual retained CSS face is loaded');
    function sameFace() {
      sameOwner(ctx);
      require(namedReferenceFontState === state && state.face === face && state.loaded && !state.error &&
        face.status === 'loaded' && document.fonts.has(face), 'same original loaded face/state without repair');
    }
    async function repeatFont() {
      repeat(ctx);
      await document.fonts.ready;
      // Repeat the actual preparation owner, not a surrogate configure-only path.
      await prepareFont();
      sameFace();
    }

    // Observe the real retained face's actual load method. Delegation preserves
    // native behavior; any repeated load is a forbidden phase operation here.
    const ownLoad = Object.getOwnPropertyDescriptor(face, 'load');
    const load = face.load;
    let reloads = 0;
    Object.defineProperty(face, 'load', {
      configurable: true, writable: true,
      value: function (...loadArgs) {
        if (this === face) reloads += 1;
        return load.apply(this, loadArgs);
      },
    });
    try {
      await repeatFont();
      actualPristine(ctx);
      const rule = Array.from(ctx.owner.sheet.cssRules).find(rule =>
        rule.type === CSSRule.FONT_FACE_RULE &&
        rule.style.fontFamily.replace(/^['"]|['"]$/g, '').toLowerCase() === 'ahem');
      require(rule, 'actual owned Ahem face rule');
      const competitor = new FontFace('ahem', rule.style.getPropertyValue('src'), {
        style: 'normal', weight: '400', stretch: 'normal',
      });
      require(competitor !== face, 'independent actual competing face');
      document.fonts.add(competitor);
      try {
        require(document.fonts.has(competitor) && document.fonts.has(face), 'both real faces are present');
        refused(firstReference + 'CompetingFontResource');
        await repeatFont();
        refused(firstReference + 'CompetingFontResource');
      } finally {
        require(document.fonts.delete(competitor), 'remove only invocation-owned competing face');
      }
      require(!document.fonts.has(competitor) && document.fonts.has(face), 'actual pristine FontFaceSet restored');
      await repeatFont();
      actualPristine(ctx);
      sameFace();
      require(reloads === 0, 'repeated setup and recovery must not reload the retained face');
    } finally {
      if (ownLoad) Object.defineProperty(face, 'load', ownLoad);
      else delete face.load;
    }
    sameFace();
    actualPristine(ctx);
    completed = { ctx, state, face };
  };
  getTestData = function () {
    require(completed, 'real font custody setup completed before measurement');
    const { ctx, state, face } = completed;
    sameOwner(ctx);
    require(namedReferenceFontState === state && state.face === face && state.loaded && !state.error &&
      face.status === 'loaded' && document.fonts.has(face), 'measurement retains original loaded identity');
    // Return a fresh real capture after cleanup, never the earlier snapshot.
    return actualPristine(ctx);
  };
}
