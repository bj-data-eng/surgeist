fontResourceSha256 => {
  const root = document.getElementById('test-root');

  function read() {
    if (!root || getComputedStyle(root).display !== 'flex' ||
        Array.from(root.childNodes).some(node =>
          node.nodeType !== Node.ELEMENT_NODE &&
          !(node.nodeType === Node.TEXT_NODE && /^[\t\n\r ]*$/.test(node.data)))) {
      throw new Error('Unsupported Flex content source');
    }
    const face = referenceMetricFace();
    if (face.status !== 'loaded') throw new Error('Flex content font is not loaded');
    const nodes = Array.from(root.children, (item, index) => {
      const source = item.firstChild;
      const own = getComputedStyle(item);
      if (item.namespaceURI !== 'http://www.w3.org/1999/xhtml' || item.tagName !== 'DIV' ||
          item.childNodes.length !== 1 || source.nodeType !== Node.TEXT_NODE ||
          !/^[A-Za-z]+(?: [A-Za-z]+)*$/.test(source.data) ||
          item.closest('[lang]')?.lang !== 'en') {
        throw new Error('Unsupported direct Flex text source');
      }
      if (!hasClosedReferenceFont(own) || !document.fonts.check('10px ahem', source.data)) {
        throw new Error('Unsupported direct Flex content font settings');
      }
      const styles = {
        display: ['flex'], flexDirection: ['row'], writingMode: ['horizontal-tb'],
        direction: ['ltr', 'rtl'], lineHeight: ['10px'], whiteSpace: ['normal'],
        textWrapMode: ['wrap'], textWrapStyle: ['auto'], textTransform: ['none'],
        // Each direct text node forms a standalone anonymous item.
        unicodeBidi: ['normal', 'isolate'], textIndent: ['0px'], textAlign: ['start'],
        textAlignLast: ['auto'], lineBreak: ['auto'], wordBreak: ['normal'], overflowWrap: ['normal'],
        textBoxTrim: ['none'], textBoxEdge: ['auto'], dominantBaseline: ['auto'],
        alignmentBaseline: ['auto', 'baseline'],
        hyphens: ['manual'], letterSpacing: ['normal', '0px'], wordSpacing: ['normal', '0px']
      };
      for (const [property, values] of Object.entries(styles)) {
        if (!values.includes(own[property])) {
          throw new Error('Unsupported direct Flex text style ' + property + '=' + own[property]);
        }
      }
      return { item, source, text: source.data, row: {
        child: index, sourceUnits: Array.from(source.data, char => char.charCodeAt(0)),
        direction: own.direction
      } };
    });
    if (!nodes.length) throw new Error('Missing Flex content source');
    return { face, nodes };
  }

  function capture(variant) {
    document.body.className = variant;
    try {
      const before = read();
      const captured = captureRootVariant(root, variant);
      if (captured.unsupportedReason) return captured;
      const after = read();
      if (before.face !== after.face || before.nodes.length !== after.nodes.length ||
          captured.children?.length !== after.nodes.length ||
          after.nodes.some(({item, source, text, row}, index) => {
            const prior = before.nodes[index];
            return item !== prior.item || source !== prior.source || text !== prior.text ||
              row.direction !== prior.row.direction || captured.children[index].textContent !== text;
          })) {
        throw new Error('Flex content source changed during capture');
      }
      return {...captured, flexContentText: {
        fontResourceSha256, rows: after.nodes.map(({row}) => row)
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
