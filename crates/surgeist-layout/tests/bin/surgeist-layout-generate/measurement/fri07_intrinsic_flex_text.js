fontResourceSha256 => {
  const root = document.getElementById('test-root');
  const isDiv = node => node.namespaceURI === 'http://www.w3.org/1999/xhtml' && node.tagName === 'DIV';
  const isWhitespace = node => node.nodeType === Node.TEXT_NODE && /^[\t\n\r ]*$/.test(node.data);

  function flexChildren(node, flexDirection) {
    const style = getComputedStyle(node);
    if (!isDiv(node) || style.display !== 'flex' || style.flexDirection !== flexDirection ||
        style.writingMode !== 'horizontal-tb' || node.children.length !== 2 ||
        Array.from(node.childNodes).some(child => child.nodeType !== Node.ELEMENT_NODE && !isWhitespace(child))) {
      throw new Error('Unexpected original Flex row topology');
    }
    return Array.from(node.children);
  }

  function readWords(source, rootRect) {
    const words = Array.from(source.data.matchAll(/[A-Za-z]+/g), match => {
      const start = match.index;
      const end = start + match[0].length;
      const range = document.createRange();
      range.setStart(source, start);
      range.setEnd(source, end);
      const rects = range.getClientRects();
      if (rects.length !== 1) throw new Error('A word must occupy one line fragment');
      const {left, top, width, height} = rects[0];
      if (![left, top, width, height, rootRect.left, rootRect.top].every(Number.isFinite) ||
          width <= 0 || height <= 0) {
        throw new Error('Invalid Flex word Range rectangle');
      }
      return {start, end, top, x: left - rootRect.left, y: top - rootRect.top, width, height};
    });
    const tops = [...new Set(words.map(word => word.top))].sort((a, b) => a - b);
    return words.map(({top, ...word}) => ({...word, line: tops.indexOf(top)}));
  }

  function read() {
    if (!root || getComputedStyle(root).position !== 'absolute') {
      throw new Error('Expected the original absolute Flex root');
    }
    const face = referenceMetricFace();
    if (face.status !== 'loaded') throw new Error('Flex content font is not loaded');
    const rootRect = root.getBoundingClientRect();
    const items = flexChildren(root, 'column').flatMap((row, rowIndex) =>
      flexChildren(row, 'row').map((item, itemIndex) => {
        const source = item.firstChild;
        const style = getComputedStyle(item);
        if (!isDiv(item) || item.childNodes.length !== 1 || source.nodeType !== Node.TEXT_NODE ||
            !/^[A-Za-z]+(?: [A-Za-z]+)*$/.test(source.data) || item.closest('[lang]')?.lang !== 'en' ||
            !hasClosedReferenceFont(style) || !document.fonts.check('10px ahem', source.data) ||
            style.display !== 'flex' || style.flexDirection !== 'row' || style.writingMode !== 'horizontal-tb' ||
            style.flexBasis !== (itemIndex === 0 ? 'min-content' : 'max-content') ||
            style.flexGrow !== '1' || style.flexShrink !== '1' || style.minWidth !== '0px' ||
            style.lineHeight !== '10px' || style.whiteSpace !== 'normal' || style.textWrapMode !== 'wrap' ||
            style.textWrapStyle !== 'auto' || style.textTransform !== 'none' || style.textIndent !== '0px' ||
            style.textAlign !== 'start' || style.textAlignLast !== 'auto' ||
            !['ltr', 'rtl'].includes(style.direction) || !['normal', 'isolate'].includes(style.unicodeBidi) ||
            style.lineBreak !== 'auto' || style.wordBreak !== 'normal' || style.overflowWrap !== 'normal' ||
            style.hyphens !== 'manual' || !['normal', '0px'].includes(style.letterSpacing) ||
            !['normal', '0px'].includes(style.wordSpacing)) {
          throw new Error('Unexpected original Flex label or typography');
        }
        const witness = {
          row: rowIndex, item: itemIndex, direction: style.direction,
          sourceUnits: Array.from(source.data, char => char.charCodeAt(0)),
          words: readWords(source, rootRect)
        };
        return {row, item, source, text: source.data, witness};
      })
    );
    return {face, items};
  }

  function capture(variant) {
    document.body.className = variant;
    try {
      const before = read();
      const captured = captureRootVariant(root, variant);
      if (captured.unsupportedReason) return captured;
      const after = read();
      if (before.face !== after.face || after.items.some(({row, item, source, text, witness}, index) => {
        const previous = before.items[index];
        return previous.row !== row || previous.item !== item || previous.source !== source ||
          previous.text !== text || JSON.stringify(previous.witness) !== JSON.stringify(witness) ||
          captured.children?.[witness.row]?.children?.[witness.item]?.textContent !== text;
      })) {
        throw new Error('Intrinsic Flex source or word Range changed during capture');
      }
      return {...captured, fri07IntrinsicFlexText: {
        fontResourceSha256, items: after.items.map(({witness}) => witness)
      }};
    } catch (error) {
      return unsupportedTestData(error.message);
    }
  }

  return JSON.stringify({
    schemaVersion: 4,
    borderBoxLtrData: capture('border-box ltr'),
    contentBoxLtrData: capture('content-box ltr'),
    borderBoxRtlData: capture('border-box rtl'),
    contentBoxRtlData: capture('content-box rtl')
  });
}
