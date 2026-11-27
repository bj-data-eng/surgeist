fontResourceSha256 => {
  const root = document.getElementById('test-root');
  const sources = ['HHHH\u200bHHHH', 'HHHHHHHHHHHHHHHH\u200bHHHHHHHHHHHHHHHH'];
  function readLeaf() {
    const children = Array.from(root.children);
    const candidates = children.filter(element => element.tagName === 'DIV' &&
      element.childNodes.length === 1 && element.firstChild.nodeType === Node.TEXT_NODE &&
      sources.includes(element.textContent));
    if (candidates.length !== 1) throw new Error('Measured Grid leaf requires one matching text child');

    const [leaf] = candidates;
    const source = leaf.textContent;
    const own = getComputedStyle(leaf);
    const face = referenceMetricFace();
    if (face.status !== 'loaded' || !hasClosedReferenceFont(own) ||
        own.writingMode !== 'horizontal-tb' || own.whiteSpace !== 'normal' ||
        own.lineHeight !== '10px' || !['normal', '0px'].includes(own.letterSpacing) ||
        !['normal', '0px'].includes(own.wordSpacing)) {
      throw new Error('Measured Grid leaf font or text style changed');
    }

    const canvas = document.createElement('canvas').getContext('2d');
    canvas.font = '10px ahem';
    const witness = {
      nodePath: '0.' + children.indexOf(leaf),
      fontResourceSha256,
      sourceUnits: Array.from(source, char => char.charCodeAt(0)),
      advances: [...source.split('\u200b'), source].map(run => canvas.measureText(run).width),
      direction: own.direction,
      boxSizing: own.boxSizing,
      clientWidth: leaf.clientWidth,
      scrollWidth: leaf.scrollWidth
    };
    return {leaf, face, source, witness};
  }
  function measure(variant) {
    if (!root) return unsupportedTestData('Measured Grid capture requires #test-root');
    document.body.className = variant;
    try {
      const before = readLeaf();
      const captured = captureRootVariant(root, variant);
      if (captured.unsupportedReason) return captured;
      const after = readLeaf();
      const index = Number(before.witness.nodePath.slice(2));
      if (before.leaf !== after.leaf || before.face !== after.face ||
          JSON.stringify(before.witness) !== JSON.stringify(after.witness) ||
          captured.viewport?.rootContext?.kind !== 'absolute-root-local' ||
          captured.children?.[index]?.textContent !== before.source) {
        throw new Error('Measured Grid leaf does not match its captured occurrence');
      }
      return {...captured, measuredLeaf: after.witness};
    } catch (error) {
      return unsupportedTestData(error.message);
    }
  }
  return JSON.stringify({
    schemaVersion: 4,
    borderBoxLtrData: measure('border-box ltr'),
    contentBoxLtrData: measure('content-box ltr'),
    borderBoxRtlData: measure('border-box rtl'),
    contentBoxRtlData: measure('content-box rtl')
  });
}
