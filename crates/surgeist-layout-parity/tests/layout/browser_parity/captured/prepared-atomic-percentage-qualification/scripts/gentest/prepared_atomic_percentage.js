async fontBytes => {
  const root = document.getElementById('test-root');
  const fail = reason => { throw new Error(`Prepared atomic percentage: ${reason}`); };
  const marker = 'data-surgeist-prepared-text';
  const fontRule = '#test-root { font-family: SurgeistParityAhem !important; font-synthesis: none; }';
  const sameFrame = (left, right) => JSON.stringify(left) === JSON.stringify(right);
  const nativeSize = (value, unit) => value.endsWith(unit) &&
    Number.isFinite(Number(value.slice(0, -unit.length))) && Number(value.slice(0, -unit.length)) >= 0;
  const names = ['borderBoxLtrData', 'contentBoxLtrData', 'borderBoxRtlData', 'contentBoxRtlData'];

  function sourceParts() {
    const [before, atomic, after] = root?.childNodes ?? [];
    if (root?.tagName !== 'DIV' || root.childNodes.length !== 3 ||
        before?.nodeType !== Node.TEXT_NODE || after?.nodeType !== Node.TEXT_NODE ||
        atomic?.tagName !== 'SPAN' || atomic.childNodes.length !== 0 ||
        root.getAttribute('data-surgeist-layout-ready-inline') !== 'true' ||
        !nativeSize(root.style.width, 'px') || !nativeSize(root.style.height, 'px') ||
        !nativeSize(atomic.style.width, 'px') || !nativeSize(atomic.style.height, '%')) {
      fail('original authored box and text source changed');
    }
    return { before, atomic, after };
  }

  function qualifySubtree(candidate, unshifted) {
    if (candidate !== root) fail('source root changed');
    const { atomic } = sourceParts();
    for (const [node, display, position] of [[root, 'block', 'absolute'], [atomic, 'inline-block', 'relative']]) {
      const style = getComputedStyle(node);
      if (style.display !== display || style.position !== position || style.writingMode !== 'horizontal-tb' ||
          style.cssFloat !== 'none' || style.clear !== 'none' || !unshifted(style) ||
          style.clipPath !== 'none' || node.scrollLeft !== 0 || node.scrollTop !== 0 ||
          !['normal', 'none'].includes(style.content) ||
          ['top', 'right', 'bottom', 'left'].some(property => node.style[property] && node.style[property] !== 'auto') ||
          ['::before', '::after'].some(pseudo => !['none', 'normal'].includes(getComputedStyle(node, pseudo).content))) {
        fail('source box is outside the in-flow atomic capture profile');
      }
    }
  }

  function captureRootTextStyle() {
    const style = getComputedStyle(root);
    // The absolute Block creates its own paragraph. Its root isolation does
    // not create a nested bidi boundary among that paragraph's source items.
    const required = {
      fontSize: '16px', lineHeight: '20px', fontStyle: 'normal', fontWeight: '400',
      fontFeatureSettings: 'normal', fontVariationSettings: 'normal', fontSynthesis: 'none',
      writingMode: 'horizontal-tb', whiteSpace: 'normal', wordBreak: 'normal',
      overflowWrap: 'normal', letterSpacing: 'normal', wordSpacing: '0px',
      textTransform: 'none', textDecorationLine: 'none', fontVariant: 'normal',
      hyphens: 'manual', color: 'rgb(0, 128, 0)',
    };
    for (const [property, value] of Object.entries(required)) {
      if (style[property] !== value) fail(`unsupported root text property ${property}: ${style[property]}`);
    }
    if (referenceFontFamily(style.fontFamily) !== referenceFontFamily(PINNED_TEXT_FONT.family) ||
        !['normal', '100%'].includes(style.fontStretch) ||
        !['normal', 'isolate'].includes(style.unicodeBidi) ||
        !['ltr', 'rtl'].includes(style.direction) || root.closest('[lang]')?.lang !== 'en') {
      fail('root text face, bidi boundary or language changed');
    }
    return {
      fontSize: 16, lineHeight: 20, color: [0, Math.fround(128 / 255), 0, 1],
      locale: 'en', whitespace: 'collapse', wrap: 'word', letterSpacing: 0, wordSpacing: 0,
    };
  }

  function captureSource(candidate) {
    if (candidate !== root) fail('prepared source owner changed');
    const { before, atomic, after } = sourceParts();
    const style = captureRootTextStyle();
    const firstLength = before.length;
    const finalLength = firstLength + after.length;
    const authoredSize = { width: atomic.style.width, height: atomic.style.height };
    const paragraph = {
      font: { ...PINNED_TEXT_FONT, loaded: true },
      direction: getComputedStyle(root).direction,
      writingMode: 'horizontal-tb',
      baseStyle: style,
      origins: [],
      connections: [{
        preceding: 0, following: 1, kind: 'continuous',
        precedingWrap: style.wrap, followingWrap: style.wrap, ancestorWrap: style.wrap,
      }],
      atomics: [{
        child: 1, at: firstLength, kind: 'in-flow', authoredSize,
        precedingWrap: style.wrap, followingWrap: style.wrap, ancestorWrap: style.wrap,
      }],
      controls: [],
      order: [
        { kind: 'origin', index: 0, affinity: 'before' },
        { kind: 'atomic', index: 0, affinity: 'before' },
        { kind: 'origin', index: 1, affinity: 'before' },
      ],
    };
    function captureText(node, child, origin, start, end) {
      const content = node.data;
      for (const character of content) {
        const codePoint = character.codePointAt(0);
        if (!glyphFreeSourceControl(codePoint) && !pinnedTextNominalGlyph(codePoint)) fail('pinned face lacks original source coverage');
      }
      const units = Array.from({ length: content.length }, (_, index) => content.charCodeAt(index));
      paragraph.origins.push({ child, units, start, end, style });
      const range = document.createRange();
      range.selectNodeContents(node);
      const rect = preparedRangeRootRect(root);
      const rangeFragments = Array.from(range.getClientRects(), fragment => ({
        origin, line: layoutReadyRangeLineIndex(root, fragment), physicalStartEdge: 'left',
        start: fragment.left - rect.left, advance: fragment.width,
      }));
      return { layoutInput: 'prepared-text', preparedOccurrence: origin, rangeFragments, children: [] };
    }
    const first = captureText(before, 0, 0, 0, firstLength);
    const box = describeElement(atomic);
    delete box.atomicInlineParticipation;
    const second = captureText(after, 2, 1, firstLength, finalLength);
    return { paragraph, children: [first, box, second] };
  }

  const stockSubtree = qualifyRootLocalSubtree;
  const stockPrepare = capturePreparedChildren;
  const savedObservation = rootLocalObservation;
  try {
    qualifyRootLocalSubtree = qualifySubtree;
    captureRootContext(root);
    if (root.hasAttribute(marker)) fail('unexpected authored capture marker');
    root.setAttribute(marker, 'ahem-ttf-v1');
    try { await preparePinnedTextFont(fontBytes); }
    finally { root.removeAttribute(marker); }
    qualifyLoadedPreparedRangeFace();
    function measure(variant) {
      document.body.className = variant;
      const context = captureRootContext(root);
      if (context.kind !== 'absolute-root-local' || document.adoptedStyleSheets.length !== 0) fail('wrong root observation frame');
      const controlledFont = new CSSStyleSheet();
      controlledFont.replaceSync(fontRule);
      const fontRules = Array.from(controlledFont.cssRules, rule => rule.cssText);
      document.adoptedStyleSheets = [controlledFont];
      const stableRect = root.getBoundingClientRect();
      rootLocalObservation = { root, x: stableRect.left, y: stableRect.top, stableRect };
      capturePreparedChildren = captureSource;
      root.setAttribute(marker, 'ahem-ttf-v1');
      let data;
      try {
        qualifyLoadedPreparedRangeFace();
        data = describeElement(root);
        data.style.position = getComputedStyle(root).position;
        const after = root.getBoundingClientRect();
        if (['left', 'top', 'width', 'height'].some(edge => stableRect[edge] !== after[edge]) ||
            document.adoptedStyleSheets.length !== 1 ||
            document.adoptedStyleSheets[0] !== controlledFont ||
            !sameFrame(fontRules, Array.from(controlledFont.cssRules, rule => rule.cssText))) {
          fail('observation frame or controlled font rules changed');
        }
      } finally {
        root.removeAttribute(marker);
        capturePreparedChildren = stockPrepare;
        rootLocalObservation = savedObservation;
        document.adoptedStyleSheets = [];
      }
      if (!sameFrame(context, captureRootContext(root))) fail('source authority changed during capture');
      data.viewport = { rootContext: context };
      data.preparedRangeFrame = PREPARED_RANGE_FRAME;
      return data;
    }
    const values = ['border-box ltr', 'content-box ltr', 'border-box rtl', 'content-box rtl'].map(measure);
    return JSON.stringify({ schemaVersion: 4, ...Object.fromEntries(names.map((name, index) => [name, values[index]])) });
  } catch (error) {
    return JSON.stringify({ schemaVersion: 4, ...Object.fromEntries(names.map(name => [name, unsupportedTestData(error.message)])) });
  } finally {
    qualifyRootLocalSubtree = stockSubtree;
    capturePreparedChildren = stockPrepare;
    rootLocalObservation = savedObservation;
  }
}
