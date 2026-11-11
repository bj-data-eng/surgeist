async (source, baseUrl, baseStyle, fontBytes) => {
  const expected = new DOMParser().parseFromString(source, 'text/html');
  const html = document.documentElement;
  const body = document.body;
  const head = document.head;
  const base = head.firstChild;
  const link = head.querySelector('link');
  const script = document.scripts[0];
  const grid = document.getElementById('test-root');
  const parent = grid?.parentElement;
  const sourceGrid = expected.getElementById('test-root');
  const sourceParent = sourceGrid?.parentElement;
  const sourceLink = expected.head.querySelector('link');
  const variantClasses = [
    ['borderBoxLtrData', 'border-box', 'ltr'],
    ['contentBoxLtrData', 'content-box', 'ltr'],
    ['borderBoxRtlData', 'border-box', 'rtl'],
    ['contentBoxRtlData', 'content-box', 'rtl'],
  ];
  const fail = reason => { throw new Error('Original Flex parent: ' + reason); };
  const savedClass = body.getAttribute('class');
  const savedObservation = rootLocalObservation;
  const sameChildren = (actual, original) => actual.length === original.length &&
    actual.every((node, index) => node.isEqualNode(original[index]));
  const rules = sheet => JSON.stringify(Array.from(sheet.cssRules, rule => rule.cssText));
  const zeroEdges = [
    'marginTop', 'marginRight', 'marginBottom', 'marginLeft',
    'paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft',
    'borderTopWidth', 'borderRightWidth', 'borderBottomWidth', 'borderLeftWidth',
  ];
  const insets = ['top', 'right', 'bottom', 'left', 'insetBlockStart', 'insetBlockEnd',
    'insetInlineStart', 'insetInlineEnd'];
  const unshifted = (node, style) => style.cssFloat === 'none' && style.clear === 'none' &&
    insets.every(property => ['', 'auto'].includes(authoredStyleValue(node, property, style))) &&
    style.transform === 'none' && style.perspective === 'none' &&
    style.filter === 'none' && style.contain === 'none' && style.willChange === 'auto' &&
    ['normal', '1'].includes(style.zoom) && style.contentVisibility === 'visible' &&
    style.animationName === 'none' && style.clipPath === 'none' &&
    style.transitionDuration.split(',').every(value => parseFloat(value) === 0);

  let ownedStyle;
  let expectedRules;
  let face;

  function verifySource(className, owner) {
    const bodyShell = body.cloneNode(false);
    bodyShell.removeAttribute('class');
    const sourceHead = Array.from(expected.head.childNodes);
    const liveHead = Array.from(head.childNodes).slice(1);
    if (document.compatMode !== 'CSS1Compat' || document.doctype?.name !== 'html' ||
        !document.doctype.isEqualNode(expected.doctype) || window.top !== window ||
        document.scrollingElement !== html || document.adoptedStyleSheets.length !== 0 ||
        !html.cloneNode(false).isEqualNode(expected.documentElement.cloneNode(false)) ||
        !head.cloneNode(false).isEqualNode(expected.head.cloneNode(false)) ||
        !bodyShell.isEqualNode(expected.body.cloneNode(false)) ||
        body.getAttribute('class') !== className ||
        !sameChildren(Array.from(body.childNodes), Array.from(expected.body.childNodes)) ||
        base?.tagName !== 'BASE' || base.attributes.length !== 1 || base.href !== baseUrl ||
        head.firstChild !== base || document.querySelectorAll('base').length !== 1 ||
        liveHead.length !== sourceHead.length ||
        liveHead.some((node, index) => sourceHead[index] === sourceLink
          ? node !== owner : !node.isEqualNode(sourceHead[index])) ||
        document.scripts.length !== 1 || document.scripts[0] !== script ||
        !script?.isEqualNode(expected.scripts[0]) ||
        document.querySelectorAll('#test-root').length !== 1 ||
        document.getElementById('test-root') !== grid || grid?.parentElement !== parent ||
        parent?.parentElement !== body || body.children.length !== 1 ||
        document.styleSheets.length !== 1 || document.styleSheets[0] !== owner?.sheet ||
        owner.sheet.disabled) {
      fail('source, variant or stylesheet owner changed');
    }
    if (owner === ownedStyle && (owner.textContent !== baseStyle ||
        rules(owner.sheet) !== expectedRules || document.fonts.size !== 1 ||
        referenceMetricFace() !== face || face.status !== 'loaded')) {
      fail('stylesheet rules or loaded face changed');
    }
  }

  function verifyFrame(boxSizing, direction) {
    const containing = [html, body].map(node => getComputedStyle(node));
    if (containing.some((style, index) => style.writingMode !== 'horizontal-tb' ||
        style.position !== 'static' || style.display !== 'block' || style.direction !== 'ltr' ||
        !unshifted(index === 0 ? html : body, style) || style.overflowX !== 'visible' || style.overflowY !== 'visible' ||
        style.scrollbarGutter !== 'auto' || style.whiteSpace !== 'normal' ||
        style.textAlign !== 'start' || parseFloat(style.textIndent) !== 0 ||
        zeroEdges.some(property => style[property] !== '0px'))) fail('containing frame changed');
    const nodes = [parent, grid, ...grid.children];
    const displays = ['flex', 'grid', 'flex', 'flex'];
    if (nodes.some((node, index) => {
      const style = getComputedStyle(node);
      return style.display !== displays[index] ||
        style.position !== (index === 0 ? 'absolute' : 'relative') ||
        style.writingMode !== 'horizontal-tb' || style.direction !== direction ||
        style.boxSizing !== boxSizing || !unshifted(node, style) ||
        style.overflowX !== 'visible' || style.overflowY !== 'visible' ||
        style.scrollbarGutter !== 'auto' || node.scrollLeft !== 0 || node.scrollTop !== 0 ||
        zeroEdges.some(property => style[property] !== '0px') ||
        ['::before', '::after'].some(pseudo => !['none', 'normal'].includes(getComputedStyle(node, pseudo).content)) ||
        (index > 0 && (!hasClosedReferenceFont(style) || style.lineHeight !== '10px' ||
          style.whiteSpace !== 'normal' || style.wordBreak !== 'normal' ||
          style.overflowWrap !== 'normal' || style.letterSpacing !== 'normal' ||
          !['normal', '0px'].includes(style.wordSpacing)));
    })) fail('Flex, Grid or text style changed');
    const own = getComputedStyle(parent);
    if (own.flexDirection !== 'row' || own.flexWrap !== 'nowrap' ||
        own.alignItems !== 'start' || own.justifyContent !== 'start' ||
        authoredStyleValue(parent, 'width', own) !== sourceParent.style.width ||
        authoredStyleValue(parent, 'height', own) !== 'max-content') fail('parent sizing changed');
    const visual = window.visualViewport;
    const width = html.clientWidth;
    const height = html.clientHeight;
    if (!visual || visual.scale !== 1 || visual.offsetLeft !== 0 || visual.offsetTop !== 0 ||
        window.scrollX !== 0 || window.scrollY !== 0 ||
        ![width, height].every(value => Number.isFinite(value) && value >= 0) ||
        width !== visual.width || height !== visual.height || body.clientWidth !== width) {
      fail('viewport extent changed');
    }
    return {kind: 'absolute-root-local', containingWidth: width, containingHeight: height,
      containingDirection: 'ltr', hypotheticalDirection: 'ltr',
      hypotheticalDisplay: 'flex', hypotheticalAnchor: 'inline-start', hypotheticalInlineOffset: 0,
      fallback: {initialContainingBlock: {width, height}, nearestScrollport: 'none',
        containingLimits: {width: 'unbounded', height: 'unbounded'}}};
  }

  try {
    if (savedClass !== expected.body.getAttribute('class') ||
        !sourceParent || sourceParent.className !== 'viewport' ||
        rootContextLiteralPx(sourceParent.style.width) === undefined ||
        !link?.isEqualNode(sourceLink) || link.disabled || !parent || !grid ||
        parent.tagName !== 'DIV' || grid.tagName !== 'DIV' || parent.children.length !== 1 ||
        grid.children.length !== 2 || !rootContextBodySequence(body, parent) ||
        Array.from(grid.children).some(node => node.tagName !== 'DIV' ||
          node.childNodes.length !== 1 || node.firstChild.nodeType !== Node.TEXT_NODE ||
          !/^H+(?:\u200bH+)+$/.test(node.textContent))) fail('original source roles changed');
    verifySource(savedClass, link);
    ownedStyle = document.createElement('style');
    ownedStyle.textContent = baseStyle;
    link.replaceWith(ownedStyle);
    const sheet = new CSSStyleSheet();
    sheet.replaceSync(baseStyle);
    expectedRules = rules(sheet);
    const fontRules = Array.from(ownedStyle.sheet.cssRules).filter(rule => rule.type === CSSRule.FONT_FACE_RULE);
    const encoded = fontRules[0]?.style.src.match(/^url\(["']?data:application\/font-woff2;charset=utf-8;base64,([A-Za-z0-9+/=]+)["']?\) format\(["']?woff2["']?\)$/)?.[1];
    if (fontRules.length !== 1 || !encoded) fail('embedded font source changed');
    const bytes = Uint8Array.from(atob(encoded), character => character.charCodeAt(0));
    face = referenceMetricFace();
    if (bytes.length !== fontBytes.length || bytes.some((byte, index) => byte !== fontBytes[index]) ||
        await face.load() !== face) fail('embedded Ahem changed');
    await document.fonts.ready;

    const result = {schemaVersion: 4};
    for (const [key, boxSizing, direction] of variantClasses) {
      const className = boxSizing + ' ' + direction;
      body.className = className;
      verifySource(className, ownedStyle);
      const context = verifyFrame(boxSizing, direction);
      const rectangle = parent.getBoundingClientRect();
      rootLocalObservation = {root: parent, x: rectangle.left, y: rectangle.top};
      const data = describeElement(parent, sourceParent);
      if (data.children.length !== 1 || data.children[0].children.length !== 2) {
        fail('measured source tree changed');
      }
      delete data.children[0].gridAlignmentProfile;
      data.gridAlignmentProfile = completeGridAlignmentProfile(data);
      data.viewport = {rootContext: context};
      verifySource(className, ownedStyle);
      if (JSON.stringify(context) !== JSON.stringify(verifyFrame(boxSizing, direction))) {
        fail('measurement frame changed');
      }
      result[key] = data;
    }
    return JSON.stringify(result);
  } catch (error) {
    return JSON.stringify({schemaVersion: 4, ...Object.fromEntries(
      variantClasses.map(([key]) => [key, unsupportedTestData(error.message)]),
    )});
  } finally {
    rootLocalObservation = savedObservation;
    if (savedClass === null) body.removeAttribute('class');
    else body.setAttribute('class', savedClass);
    if (ownedStyle?.parentElement === head) ownedStyle.replaceWith(link);
  }
}
