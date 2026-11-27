(source, baseUrl) => {
  const original = new DOMParser().parseFromString(source, 'text/html');
  const html = document.documentElement;
  const body = document.body;
  const grid = document.getElementById('test-root');
  const base = document.head.firstChild;
  const authored = document.head.querySelector('style');
  const originalStyle = original.head.querySelector('style');
  const insets = ['top', 'right', 'bottom', 'left'];
  const zeroEdges = [
    'marginTop', 'marginRight', 'marginBottom', 'marginLeft',
    'paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft',
    'borderTopWidth', 'borderRightWidth', 'borderBottomWidth', 'borderLeftWidth',
  ];
  const rules = sheet => Array.from(sheet.cssRules, rule => rule.cssText);
  const fail = reason => { throw new Error(`Original relative RTL Subgrid: ${reason}`); };

  function verifyDocument() {
    const originalHead = Array.from(original.head.childNodes);
    const liveHead = Array.from(document.head.childNodes).slice(1);
    if (document.compatMode !== 'CSS1Compat' || !document.doctype?.isEqualNode(original.doctype) ||
        window.top !== window || document.scrollingElement !== html ||
        !html.cloneNode(false).isEqualNode(original.documentElement.cloneNode(false)) ||
        !document.head.cloneNode(false).isEqualNode(original.head.cloneNode(false)) ||
        !body.isEqualNode(original.body) || grid?.parentElement !== body || body.children.length !== 1 ||
        document.querySelectorAll('#test-root').length !== 1 || document.scripts.length !== 0 ||
        base?.tagName !== 'BASE' || base.attributes.length !== 1 || base.href !== baseUrl ||
        document.querySelectorAll('base').length !== 1 || liveHead.length !== originalHead.length ||
        !liveHead.every((node, index) => node.isEqualNode(originalHead[index]))) {
      fail('source or document frame changed');
    }
    const sheet = authored?.sheet;
    if (!sheet || !originalStyle || authored.textContent !== originalStyle.textContent ||
        sheet.disabled || document.styleSheets.length !== 1 || document.styleSheets[0] !== sheet ||
        document.adoptedStyleSheets.length !== 0) {
      fail('authored stylesheet inventory changed');
    }
    const expectedSheet = new CSSStyleSheet();
    expectedSheet.replaceSync(originalStyle.textContent);
    if (JSON.stringify(rules(sheet)) !== JSON.stringify(rules(expectedSheet))) {
      fail('authored stylesheet rules changed');
    }
  }

  function boxStyle(element, display, position, direction) {
    const style = getComputedStyle(element);
    const inset = position === 'absolute' ? '0px' : 'auto';
    if (style.display !== display || style.position !== position || style.direction !== direction ||
        style.writingMode !== 'horizontal-tb' || style.boxSizing !== 'content-box' ||
        style.fontFamily !== 'monospace' || style.fontSize !== '12px' || style.lineHeight !== '12px' ||
        style.cssFloat !== 'none' || style.clear !== 'none' ||
        insets.some(property => element.computedStyleMap().get(property)?.toString() !== inset) ||
        style.transform !== 'none' || style.perspective !== 'none' || style.filter !== 'none' ||
        style.contain !== 'none' || style.willChange !== 'auto' || !['normal', '1'].includes(style.zoom) ||
        style.contentVisibility !== 'visible' || style.animationName !== 'none' ||
        style.transitionDuration.split(',').some(value => parseFloat(value) !== 0) ||
        style.overflowX !== 'visible' || style.overflowY !== 'visible' ||
        style.scrollbarGutter !== 'auto' ||
        ['::before', '::after'].some(pseudo => !['none', 'normal'].includes(getComputedStyle(element, pseudo).content))) {
      fail('source box, positioning or typography changed');
    }
    return style;
  }

  function verifyText(leaf, text, position, direction) {
    const style = boxStyle(leaf, 'block', position, direction);
    if (leaf.tagName !== 'DIV' || leaf.children.length !== 0 || leaf.textContent !== text ||
        zeroEdges.some(property => style[property] !== '0px')) {
      fail('text leaf or box edges changed');
    }
    const range = document.createRange();
    range.selectNodeContents(leaf);
    const lines = Array.from(range.getClientRects());
    const box = leaf.getBoundingClientRect();
    // The fixed columns and authored line height determine the box. Glyph
    // advances are only checked for a single line within its assigned span.
    if (lines.length !== 1 || lines[0].width <= 0 || lines[0].width >= box.width ||
        lines[0].left < box.left || lines[0].right > box.right) {
      fail('text no longer fits one line in its assigned span');
    }
  }

  function containingFrame() {
    for (const element of [html, body]) {
      const style = boxStyle(element, 'block', 'static', 'ltr');
      if (zeroEdges.some(property => style[property] !== '0px')) fail('Body edges changed');
    }
    const outer = boxStyle(grid, 'grid', 'static', 'ltr');
    if (outer.width !== '500px' || grid.style.gridTemplateColumns !== 'repeat(10, 30px)' ||
        [outer.paddingTop, outer.paddingRight, outer.paddingBottom, outer.paddingLeft].join(' ') !== '1px 2px 1px 2px' ||
        ['Top', 'Right', 'Bottom', 'Left'].some(edge => outer[`border${edge}Width`] !== '1px')) {
      fail('outer Grid sizing or edges changed');
    }
    const children = Array.from(grid.children);
    const subgrid = children[10];
    const positioned = subgrid?.firstElementChild;
    if (children.length !== 11 || !positioned || subgrid.children.length !== 1 ||
        subgrid.tagName !== 'DIV' || subgrid.style.gridTemplateColumns !== 'subgrid' ||
        positioned.style.gridColumnStart !== '3' || positioned.style.gridColumnEnd !== '5' ||
        positioned.offsetParent !== subgrid) {
      fail('relative Subgrid or positioned line placement changed');
    }
    boxStyle(subgrid, 'grid', 'relative', 'rtl');
    children.slice(0, 10).forEach((leaf, index) => verifyText(leaf, String(index + 1), 'static', 'ltr'));
    verifyText(positioned, 'x', 'absolute', 'rtl');

    const visual = window.visualViewport;
    const width = html.clientWidth;
    const height = html.clientHeight;
    const rootPosition = grid.getBoundingClientRect();
    if (window.scrollX !== 0 || window.scrollY !== 0 || !visual ||
        visual.scale !== 1 || visual.offsetLeft !== 0 || visual.offsetTop !== 0 ||
        ![width, height].every(value => Number.isFinite(value) && value >= 0) ||
        width !== visual.width || height !== visual.height || width !== body.clientWidth ||
        rootPosition.x !== 0 || rootPosition.y !== 0) {
      fail('Body containing frame or Grid origin changed');
    }
    return { kind: 'block-flow-auto', parentInlineExtent: width, parentDirection: 'ltr',
      fallback: { initialContainingBlock: { width, height }, nearestScrollport: 'none',
        containingLimits: { width: 'unbounded', height: 'unbounded' } } };
  }

  let data;
  try {
    verifyDocument();
    const context = containingFrame();
    data = describeElement(body);
    if (data.children.length !== 1) fail('Body to Grid mapping changed');
    delete data.children[0].gridAlignmentProfile;
    data.gridAlignmentProfile = completeGridAlignmentProfile(data);
    data.style.position = 'static';
    data.viewport = { rootContext: context };
    verifyDocument();
    if (JSON.stringify(context) !== JSON.stringify(containingFrame())) {
      fail('Body containing frame changed during capture');
    }
  } catch (error) {
    data = unsupportedTestData(error.message);
  }
  // All four historical labels represent the same authored LTR outer tree.
  return JSON.stringify({ schemaVersion: 4,
    borderBoxLtrData: data, contentBoxLtrData: data, borderBoxRtlData: data, contentBoxRtlData: data });
}
