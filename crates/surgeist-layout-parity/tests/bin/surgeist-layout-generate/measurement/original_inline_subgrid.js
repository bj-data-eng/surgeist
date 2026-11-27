(source, baseUrl, subgridWritingMode) => {
  const expected = new DOMParser().parseFromString(source, 'text/html');
  const html = document.documentElement;
  const body = document.body;
  const root = document.getElementById('test-root');
  const base = document.head.firstChild;
  const authored = document.head.querySelector('style');
  const expectedStyle = expected.head.querySelector('style');
  const zeroEdges = [
    'marginTop', 'marginRight', 'marginBottom', 'marginLeft',
    'paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft',
    'borderTopWidth', 'borderRightWidth', 'borderBottomWidth', 'borderLeftWidth',
  ];
  const rules = sheet => Array.from(sheet.cssRules, rule => rule.cssText);
  const fail = reason => { throw new Error(`Original inline Subgrid: ${reason}`); };

  function verifyDocument() {
    const originalHead = Array.from(expected.head.childNodes);
    const liveHead = Array.from(document.head.childNodes).slice(1);
    if (document.compatMode !== 'CSS1Compat' || !document.doctype?.isEqualNode(expected.doctype) ||
        window.top !== window || document.scrollingElement !== html ||
        !html.cloneNode(false).isEqualNode(expected.documentElement.cloneNode(false)) ||
        !document.head.cloneNode(false).isEqualNode(expected.head.cloneNode(false)) ||
        !body.isEqualNode(expected.body) || root?.parentElement !== body || body.children.length !== 1 ||
        document.querySelectorAll('#test-root').length !== 1 || document.scripts.length !== 0 ||
        base?.tagName !== 'BASE' || base.attributes.length !== 1 || base.href !== baseUrl ||
        document.querySelectorAll('base').length !== 1 || liveHead.length !== originalHead.length ||
        !liveHead.every((node, index) => node.isEqualNode(originalHead[index]))) {
      fail('source or document frame changed');
    }
    const sheet = authored?.sheet;
    if (!sheet || !expectedStyle || authored.textContent !== expectedStyle.textContent ||
        sheet.disabled || document.styleSheets.length !== 1 || document.styleSheets[0] !== sheet ||
        document.adoptedStyleSheets.length !== 0) {
      fail('authored stylesheet inventory changed');
    }
    const expectedSheet = new CSSStyleSheet();
    expectedSheet.replaceSync(expectedStyle.textContent);
    if (JSON.stringify(rules(sheet)) !== JSON.stringify(rules(expectedSheet))) {
      fail('authored stylesheet rules changed');
    }
  }

  function staticBox(element, display) {
    const style = getComputedStyle(element);
    if (style.position !== 'static' || style.display !== display || style.direction !== 'ltr' ||
        style.boxSizing !== 'content-box' || style.cssFloat !== 'none' || style.clear !== 'none' ||
        ['top', 'right', 'bottom', 'left'].some(property => style[property] !== 'auto') ||
        style.transform !== 'none' || style.perspective !== 'none' || style.filter !== 'none' ||
        style.contain !== 'none' || style.willChange !== 'auto' || !['normal', '1'].includes(style.zoom) ||
        style.contentVisibility !== 'visible' || style.animationName !== 'none' ||
        style.transitionDuration.split(',').some(value => parseFloat(value) !== 0) ||
        style.overflowX !== 'visible' || style.overflowY !== 'visible' ||
        style.scrollbarGutter !== 'auto' ||
        zeroEdges.some(property => style[property] !== '0px') ||
        ['::before', '::after'].some(pseudo => !['none', 'normal'].includes(getComputedStyle(element, pseudo).content))) {
      fail('source box or static formatting role changed');
    }
    return style;
  }

  function containingFrame() {
    const nested = root.firstElementChild;
    const item = nested?.firstElementChild;
    if (!nested || !item || root.children.length !== 1 || nested.children.length !== 1 ||
        item.children.length !== 0 ||
        [root, nested, item].some(element => element.tagName !== 'DIV')) {
      fail('inline Grid tree changed');
    }
    if ([staticBox(html, 'block'), staticBox(body, 'block'), staticBox(root, 'inline-grid')]
        .some(style => style.writingMode !== 'horizontal-tb') ||
        staticBox(nested, 'grid').writingMode !== subgridWritingMode ||
        staticBox(item, 'block').writingMode !== subgridWritingMode ||
        root.computedStyleMap().get('width')?.toString() !== 'auto' ||
        root.computedStyleMap().get('height')?.toString() !== 'auto') {
      fail('inline sizing inputs changed');
    }
    const visual = window.visualViewport;
    const width = html.clientWidth;
    const height = html.clientHeight;
    const position = root.getBoundingClientRect();
    if (window.scrollX !== 0 || window.scrollY !== 0 || !visual ||
        visual.scale !== 1 || visual.offsetLeft !== 0 || visual.offsetTop !== 0 ||
        ![width, height].every(value => Number.isFinite(value) && value >= 0) ||
        width !== visual.width || height !== visual.height || width !== body.clientWidth ||
        position.x !== 0 || position.y !== 0) {
      fail('inline start or containing frame changed');
    }
    return { kind: 'inline-grid', parentInlineExtent: width, parentDirection: 'ltr',
      fallback: { initialContainingBlock: { width, height }, nearestScrollport: 'none',
        containingLimits: { width: 'unbounded', height: 'unbounded' } } };
  }

  try {
    verifyDocument();
    const context = containingFrame();
    const data = describeElement(root);
    data.style.position = 'static';
    data.gridAlignmentProfile = completeGridAlignmentProfile(data);
    data.viewport = { rootContext: context };
    verifyDocument();
    if (JSON.stringify(context) !== JSON.stringify(containingFrame())) {
      fail('inline containing frame changed during capture');
    }
    // The four retained labels all describe the original LTR content-box tree.
    return JSON.stringify({ schemaVersion: 4,
      borderBoxLtrData: data, contentBoxLtrData: data, borderBoxRtlData: data, contentBoxRtlData: data });
  } catch (error) {
    const data = unsupportedTestData(error.message);
    return JSON.stringify({ schemaVersion: 4,
      borderBoxLtrData: data, contentBoxLtrData: data, borderBoxRtlData: data, contentBoxRtlData: data });
  }
}
