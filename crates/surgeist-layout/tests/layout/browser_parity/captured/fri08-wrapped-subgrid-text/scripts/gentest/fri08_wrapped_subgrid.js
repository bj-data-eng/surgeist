async (fontResourceSha256, fontBytes) => {
  const root = document.getElementById('test-root');
  const fail = reason => { throw new Error(`Wrapped Subgrid text: ${reason}`); };

  function readFace() {
    const [sheet] = document.styleSheets;
    if (document.styleSheets.length !== 1 || document.adoptedStyleSheets.length ||
        sheet.disabled || sheet.ownerNode?.parentElement !== document.head) fail('stylesheet ownership changed');
    const faces = Array.from(sheet.cssRules).filter(rule => rule.type === CSSRule.FONT_FACE_RULE);
    const source = faces[0]?.style.getPropertyValue('src') ?? '';
    const base64 = source.match(/data:application\/font-woff2;charset=utf-8;base64,([A-Za-z0-9+/=]+)/)?.[1];
    if (faces.length !== 1 || !base64 || !/\bformat\(["']?woff2["']?\)/.test(source)) {
      fail('stylesheet font rule changed');
    }
    const bytes = Uint8Array.from(atob(base64), character => character.charCodeAt(0));
    if (bytes.length !== fontBytes.length || !bytes.every((byte, index) => byte === fontBytes[index])) {
      fail('stylesheet font bytes changed');
    }
    return referenceMetricFace();
  }

  function readRanges(source, rootRect) {
    const fragments = [];
    for (let start = 0; start < source.length; start++) {
      if (source.data[start] === ' ') continue;
      const range = document.createRange();
      range.setStart(source, start);
      range.setEnd(source, start + 1);
      const rects = range.getClientRects();
      if (rects.length !== 1) fail('an ASCII source unit must occupy one line fragment');
      const {left, top, right, bottom} = rects[0];
      if (![left, top, right, bottom].every(Number.isFinite) || left >= right || top >= bottom) {
        fail('invalid source Range rectangle');
      }
      const previous = fragments.at(-1);
      if (previous && previous.end === start && previous.top === top && previous.bottom === bottom &&
          (previous.right === left || right === previous.left)) {
        previous.end++;
        previous.left = Math.min(previous.left, left);
        previous.right = Math.max(previous.right, right);
      } else {
        fragments.push({start, end: start + 1, left, top, right, bottom});
      }
    }
    const tops = [...new Set(fragments.map(range => range.top))].sort((a, b) => a - b);
    return fragments.map(({start, end, left, top, right, bottom}) => ({
      start, end, line: tops.indexOf(top),
      x: left - rootRect.left, y: top - rootRect.top, width: right - left, height: bottom - top,
    }));
  }

  function read() {
    if (!root || root.children.length !== 3 || getComputedStyle(root).display !== 'grid' ||
        getComputedStyle(root).position !== 'absolute') fail('original absolute Grid root changed');
    const face = readFace();
    if (face.status !== 'loaded') fail('stylesheet face is not loaded');
    const rootRect = root.getBoundingClientRect();
    const items = Array.from(root.children).flatMap((card, cardIndex) => {
      const [header, footer] = card.children;
      if (card.children.length !== 2 || footer.children.length !== 1 ||
          [card, footer].some(node => getComputedStyle(node).display !== 'grid' ||
            node.style.gridTemplateRows !== 'subgrid')) fail('nested row Subgrid topology changed');
      return [header, footer.children[0]].map((item, itemIndex) => {
        const source = item.firstChild;
        const style = getComputedStyle(item);
        const hidden = itemIndex === 1;
        if (item.tagName !== 'DIV' || item.childNodes.length !== 1 ||
            source.nodeType !== Node.TEXT_NODE || !/^[A-Za-z:]+(?: [A-Za-z:]+)*$/.test(source.data) ||
            item.closest('[lang]')?.lang !== 'en' || !hasClosedReferenceFont(style) ||
            !document.fonts.check('10px ahem', source.data) ||
            style.display !== 'flex' || style.flexDirection !== 'row' ||
            style.writingMode !== 'horizontal-tb' || style.lineHeight !== '10px' ||
            style.whiteSpace !== 'normal' || style.textWrapMode !== 'wrap' ||
            style.textWrapStyle !== 'auto' || style.textTransform !== 'none' || style.textIndent !== '0px' ||
            style.textAlign !== 'start' || style.textAlignLast !== 'auto' ||
            !['ltr', 'rtl'].includes(style.direction) || !['normal', 'isolate'].includes(style.unicodeBidi) ||
            style.lineBreak !== 'auto' || style.wordBreak !== 'normal' || style.overflowWrap !== 'normal' ||
            style.hyphens !== 'manual' || !['normal', '0px'].includes(style.letterSpacing) ||
            !['normal', '0px'].includes(style.wordSpacing) ||
            style.overflowX !== (hidden ? 'hidden' : 'visible') ||
            style.overflowY !== (hidden ? 'hidden' : 'visible')) fail('original Flex leaf style changed');
        const witness = {
          card: cardIndex, item: itemIndex, direction: style.direction,
          sourceUnits: Array.from(source.data, unit => unit.charCodeAt(0)),
          ranges: readRanges(source, rootRect),
        };
        if (hidden) {
          for (const edge of ['Top', 'Right', 'Bottom', 'Left']) {
            if (style[`padding${edge}`] !== '0px' || style[`border${edge}Width`] !== '0px') {
              fail('hidden leaf edges changed');
            }
          }
          witness.clip = [item.clientWidth, item.clientHeight, item.scrollWidth, item.scrollHeight];
        }
        return {source, witness};
      });
    });
    return {face, items};
  }

  function capture(variant) {
    document.body.className = variant;
    try {
      const before = read();
      const captured = captureRootVariant(root, variant);
      if (captured.unsupportedReason) return captured;
      const after = read();
      if (before.face !== after.face || after.items.some(({source, witness}, index) => {
        const previous = before.items[index];
        const card = captured.children?.[witness.card];
        const item = witness.item === 0 ? card?.children?.[0] : card?.children?.[1]?.children?.[0];
        return previous.source !== source || JSON.stringify(previous.witness) !== JSON.stringify(witness) ||
          item?.textContent !== source.data;
      })) fail('source or Text observations changed during capture');
      return {...captured, fri08WrappedSubgrid: {
        fontResourceSha256, items: after.items.map(({witness}) => witness)
      }};
    } catch (error) {
      return unsupportedTestData(error.message);
    }
  }

  let result;
  try {
    const face = readFace();
    if (await face.load() !== face) fail('stylesheet face identity changed');
    await document.fonts.ready;
    if (face !== readFace() || face.status !== 'loaded') fail('stylesheet face loading changed');
    result = {
      borderBoxLtrData: capture('border-box ltr'),
      contentBoxLtrData: capture('content-box ltr'),
      borderBoxRtlData: capture('border-box rtl'),
      contentBoxRtlData: capture('content-box rtl'),
    };
  } catch (error) {
    const data = unsupportedTestData(error.message);
    result = {borderBoxLtrData: data, contentBoxLtrData: data,
      borderBoxRtlData: data, contentBoxRtlData: data};
  }
  return JSON.stringify({schemaVersion: 4, ...result});
}
