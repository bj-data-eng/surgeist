() => {
  const root = document.getElementById('test-root');
  const labels = ['Number 1', 'Number 2', 'Number 3', 'Number 4'];
  const widths = ['min-content', 'max-content'];
  const fail = reason => { throw new Error(`Grid Lanes monospace sizing: ${reason}`); };

  function typography(leaf) {
    const style = getComputedStyle(leaf);
    if (style.fontFamily !== 'monospace' || style.fontSize !== '15px' ||
        style.lineHeight !== '15px' || style.fontStyle !== 'normal' ||
        style.fontWeight !== '400' || !['normal', '100%'].includes(style.fontStretch) ||
        style.fontFeatureSettings !== 'normal' || style.fontVariationSettings !== 'normal' ||
        style.fontVariant !== 'normal' || style.fontSizeAdjust !== 'none' ||
        style.display !== 'flex' || style.flexDirection !== 'row' ||
        style.writingMode !== 'horizontal-tb' || !['ltr', 'rtl'].includes(style.direction) ||
        !['normal', 'isolate'].includes(style.unicodeBidi) ||
        style.whiteSpace !== 'normal' || style.textWrapMode !== 'wrap' ||
        style.textWrapStyle !== 'auto' || style.textTransform !== 'none' ||
        style.textIndent !== '0px' || style.textAlign !== 'start' || style.textAlignLast !== 'auto' ||
        style.lineBreak !== 'auto' || style.wordBreak !== 'normal' ||
        style.overflowWrap !== 'normal' || style.hyphens !== 'manual' ||
        !['normal', '0px'].includes(style.letterSpacing) ||
        !['normal', '0px'].includes(style.wordSpacing) ||
        leaf.closest('[lang]')?.lang !== 'en') fail('source typography changed');
    return Object.fromEntries([
      'font-family', 'font-size', 'line-height', 'font-style', 'font-weight', 'font-stretch',
      'font-kerning', 'font-variant', 'font-feature-settings', 'font-variation-settings',
      'font-size-adjust', 'font-synthesis', 'text-rendering', 'writing-mode', 'direction',
      'unicode-bidi', 'white-space', 'text-wrap', 'text-transform', 'text-indent', 'text-align',
      'text-align-last', 'line-break', 'word-break', 'overflow-wrap', 'hyphens', 'letter-spacing',
      'word-spacing',
    ].map(name => [name, style.getPropertyValue(name)]));
  }

  function inspect() {
    if (!root || root.children.length !== widths.length ||
        getComputedStyle(root).display !== 'block' ||
        getComputedStyle(root).position !== 'absolute') fail('original Block root changed');
    return Array.from(root.children).flatMap((group, row) => {
      if (group.children.length !== labels.length ||
          getComputedStyle(group).display !== 'grid-lanes' ||
          group.style.width !== widths[row]) fail('original Grid Lanes group changed');
      return Array.from(group.children).map((leaf, item) => {
        const source = leaf.firstChild;
        if (leaf.tagName !== 'DIV' || leaf.childNodes.length !== 1 ||
            source.nodeType !== Node.TEXT_NODE || source.data !== labels[item] ||
            leaf.style.width !== widths[row]) fail('original Flex leaf changed');
        return {row, item, leaf, source, properties: typography(leaf)};
      });
    });
  }

  // Each sizing probe is outside the Grid Lanes formatting context.
  function measure(source, properties) {
    const probe = document.createElement('section');
    const text = document.createTextNode(source.data);
    probe.lang = 'en';
    probe.append(text);
    Object.assign(probe.style, {
      position: 'absolute', top: '0', left: '0', visibility: 'hidden', display: 'block',
      boxSizing: 'content-box', minWidth: '0', padding: '0', border: '0', margin: '0',
    });
    for (const [name, value] of Object.entries(properties)) probe.style.setProperty(name, value);

    function read(width) {
      probe.style.width = width;
      const {width: inlineSize, height: blockSize} = probe.getBoundingClientRect();
      const tops = [[0, 6], [7, 8]].map(([start, end]) => {
        const range = document.createRange();
        range.setStart(text, start);
        range.setEnd(text, end);
        const rectangles = range.getClientRects();
        if (rectangles.length !== 1 || rectangles[0].width <= 0 ||
            !Number.isFinite(rectangles[0].top)) fail('independent word range changed');
        return rectangles[0].top;
      });
      const lines = [...new Set(tops)].sort((a, b) => a - b);
      return {inlineSize, blockSize, wordLines: tops.map(top => lines.indexOf(top))};
    }

    document.body.append(probe);
    try {
      if (Object.entries(properties).some(([name, value]) =>
        getComputedStyle(probe).getPropertyValue(name) !== value)) fail('probe typography changed');
      const minContent = read('min-content');
      const maxContent = read('max-content');
      const zero = read('0px');
      let low = minContent.inlineSize * 64;
      let high = maxContent.inlineSize * 64;
      if (!Number.isSafeInteger(low) || !Number.isSafeInteger(high) || low <= 0 || high <= low) {
        fail('unexpected intrinsic width domain');
      }

      function atTick(tick) {
        const measurement = read(`${tick / 64}px`);
        if (measurement.inlineSize * 64 !== tick) fail('standalone width changed');
        return measurement;
      }
      function wraps(measurement) {
        const [first, second] = measurement.wordLines;
        if (first !== 0 || ![0, 1].includes(second) ||
            measurement.blockSize !== (second + 1) * 15) fail('standalone line census changed');
        return second === 1;
      }

      let lastWrap = atTick(low);
      let firstFit = atTick(high);
      if (!wraps(lastWrap) || wraps(firstFit)) fail('intrinsic widths do not bracket wrapping');
      while (high - low > 1) {
        const middle = low + Math.floor((high - low) / 2);
        const measurement = atTick(middle);
        if (wraps(measurement)) {
          low = middle;
          lastWrap = measurement;
        } else {
          high = middle;
          firstFit = measurement;
        }
      }
      return {minContent, maxContent, lastWrap, firstFit, zero};
    } finally {
      probe.remove();
    }
  }

  function capture(variant) {
    document.body.className = variant;
    try {
      const before = inspect();
      const items = before.map(({row, item, source, properties}) => ({
        row, item, direction: properties.direction,
        sourceUnits: Array.from(source.data, unit => unit.charCodeAt(0)),
        sizes: measure(source, properties),
      }));
      const captured = captureRootVariant(root, variant);
      if (captured.unsupportedReason) return captured;
      const after = inspect();
      if (after.some(({row, item, leaf, source, properties}, index) => {
        const original = before[index];
        return original.leaf !== leaf || original.source !== source ||
          JSON.stringify(original.properties) !== JSON.stringify(properties) ||
          captured.children?.[row]?.children?.[item]?.textContent !== source.data;
      })) fail('source or typography changed during capture');
      return {...captured, fri08GridLanesText: items};
    } catch (error) {
      return unsupportedTestData(error.message);
    }
  }

  return JSON.stringify({
    schemaVersion: 4,
    borderBoxLtrData: capture('border-box ltr'),
    contentBoxLtrData: capture('content-box ltr'),
    borderBoxRtlData: capture('border-box rtl'),
    contentBoxRtlData: capture('content-box rtl'),
  });
}
