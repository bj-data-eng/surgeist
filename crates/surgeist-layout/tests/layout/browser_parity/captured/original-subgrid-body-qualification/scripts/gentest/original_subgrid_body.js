(source, baseUrl) => {
  const expected = new DOMParser().parseFromString(source, 'text/html');
  const html = document.documentElement;
  const body = document.body;
  const root = document.getElementById('test-root');
  const base = document.head.firstChild;
  const authoredStyle = document.head.querySelector('style');
  const expectedStyle = expected.head.querySelector('style');
  const authoredSheet = authoredStyle?.sheet;
  const zeroEdges = [
    'marginTop', 'marginRight', 'marginBottom', 'marginLeft',
    'paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft',
    'borderTopWidth', 'borderRightWidth', 'borderBottomWidth', 'borderLeftWidth',
  ];
  const insets = ['top', 'right', 'bottom', 'left'];
  const fail = reason => { throw new Error(`Original subgrid body: ${reason}`); };
  const sameChildren = (live, original) =>
    live.length === original.length && live.every((node, index) => node.isEqualNode(original[index]));
  const equalRules = (left, right) => {
    const rules = sheet => Array.from(sheet.cssRules, rule => rule.cssText);
    return JSON.stringify(rules(left)) === JSON.stringify(rules(right));
  };

  function verifySource(variantSheet) {
    if (document.compatMode !== 'CSS1Compat' || document.doctype?.name !== 'html' ||
        !document.doctype.isEqualNode(expected.doctype) || window.top !== window ||
        document.scrollingElement !== html || !html.cloneNode(false).isEqualNode(expected.documentElement.cloneNode(false)) ||
        !body.isEqualNode(expected.body) || root?.parentElement !== body || body.children.length !== 1 ||
        document.getElementById('test-root') !== root || document.querySelectorAll('#test-root').length !== 1 ||
        !document.head.cloneNode(false).isEqualNode(expected.head.cloneNode(false)) ||
        base?.tagName !== 'BASE' || document.head.firstChild !== base ||
        base.attributes.length !== 1 || base.href !== baseUrl ||
        !sameChildren(Array.from(document.head.childNodes).slice(1), Array.from(expected.head.childNodes)) ||
        document.scripts.length !== 0 || document.querySelectorAll('base').length !== 1) {
      fail('source or document frame changed');
    }
    const adopted = document.adoptedStyleSheets;
    if (!authoredSheet || !expectedStyle || authoredStyle !== document.head.querySelector('style') ||
        authoredStyle.textContent !== expectedStyle.textContent || authoredSheet.disabled ||
        document.styleSheets.length !== 1 || document.styleSheets[0] !== authoredSheet ||
        adopted.length !== (variantSheet ? 1 : 0) || (variantSheet && adopted[0] !== variantSheet)) {
      fail('stylesheet inventory changed');
    }
    const parsedSheet = new CSSStyleSheet();
    parsedSheet.replaceSync(expectedStyle.textContent);
    if (!equalRules(authoredSheet, parsedSheet)) fail('authored stylesheet rules changed');
  }

  function staticFlow(style) {
    return style.writingMode === 'horizontal-tb' && style.position === 'static' &&
      style.cssFloat === 'none' && style.clear === 'none' &&
      insets.every(property => style[property] === 'auto') &&
      style.transform === 'none' && style.perspective === 'none' && style.filter === 'none' &&
      style.contain === 'none' && style.willChange === 'auto' &&
      ['normal', '1'].includes(style.zoom) && style.contentVisibility === 'visible' &&
      style.animationName === 'none' &&
      style.transitionDuration.split(',').every(value => parseFloat(value) === 0);
  }

  function containingFrame(direction, boxSizing) {
    for (const [element, flow] of [[html, 'ltr'], [body, direction]]) {
      const style = getComputedStyle(element);
      if (!staticFlow(style) || style.display !== 'block' || style.direction !== flow ||
          style.overflowX !== 'visible' || style.overflowY !== 'visible' ||
          style.scrollbarGutter !== 'auto' || zeroEdges.some(property => style[property] !== '0px')) {
        fail('body or containing frame is outside the static zero-edge profile');
      }
    }
    const boxes = [root, ...root.querySelectorAll('*')];
    if (boxes.length !== 16 || getComputedStyle(root).display !== 'grid' || boxes.some(element => {
      const style = getComputedStyle(element);
      return element.tagName !== 'DIV' || !staticFlow(style) ||
        style.direction !== direction || style.boxSizing !== boxSizing ||
        ['::before', '::after'].some(pseudo => !['none', 'normal'].includes(getComputedStyle(element, pseudo).content));
    })) fail('Grid subtree is outside the static box profile');

    const visual = window.visualViewport;
    const width = html.clientWidth;
    const height = html.clientHeight;
    if (window.scrollX !== 0 || window.scrollY !== 0 || !visual ||
        visual.scale !== 1 || visual.offsetLeft !== 0 || visual.offsetTop !== 0 ||
        ![width, height].every(value => Number.isFinite(value) && value >= 0) ||
        width !== visual.width || height !== visual.height || body.clientWidth !== width) {
      fail('viewport extent authority changed');
    }
    return { kind: 'block-flow-auto', parentInlineExtent: width, parentDirection: 'ltr',
      fallback: { initialContainingBlock: { width, height }, nearestScrollport: 'none',
        containingLimits: { width: 'unbounded', height: 'unbounded' } } };
  }

  function measure(direction, boxSizing) {
    let variantSheet;
    try {
      verifySource();
      const rules = `body { direction: ${direction}; } #test-root, #test-root * { box-sizing: ${boxSizing}; }`;
      variantSheet = new CSSStyleSheet();
      variantSheet.replaceSync(rules);
      document.adoptedStyleSheets = [variantSheet];
      verifySource(variantSheet);
      const context = containingFrame(direction, boxSizing);
      const data = describeElement(body);
      if (data.children.length !== 1 || data.children[0].tagName !== 'div') fail('body to Grid mapping changed');
      delete data.children[0].gridAlignmentProfile;
      data.gridAlignmentProfile = completeGridAlignmentProfile(data);
      data.style.position = 'static';
      data.viewport = { rootContext: context };
      verifySource(variantSheet);
      const control = new CSSStyleSheet();
      control.replaceSync(rules);
      if (!equalRules(variantSheet, control) ||
          JSON.stringify(context) !== JSON.stringify(containingFrame(direction, boxSizing))) {
        fail('variant stylesheet or frame changed during capture');
      }
      return data;
    } catch (error) {
      return unsupportedTestData(error.message);
    } finally {
      if (variantSheet && document.adoptedStyleSheets[0] === variantSheet) {
        document.adoptedStyleSheets = [];
      }
    }
  }

  return JSON.stringify({
    schemaVersion: 4,
    borderBoxLtrData: measure('ltr', 'border-box'),
    contentBoxLtrData: measure('ltr', 'content-box'),
    borderBoxRtlData: measure('rtl', 'border-box'),
    contentBoxRtlData: measure('rtl', 'content-box'),
  });
}
