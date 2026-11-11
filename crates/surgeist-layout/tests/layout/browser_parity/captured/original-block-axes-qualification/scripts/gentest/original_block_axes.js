(source, baseUrl, baseStyle) => {
  const expected = new DOMParser().parseFromString(source, 'text/html');
  const html = document.documentElement, head = document.head, body = document.body;
  const base = head.firstChild, link = head.querySelector('link');
  const sourceLink = expected.head.querySelector('link');
  const root = document.getElementById('test-root');
  const sourceRoot = expected.getElementById('test-root');
  const savedClass = body.getAttribute('class');
  const savedProof = rootContextCapture;
  const savedCapture = captureRootContext;
  const fail = reason => { throw new Error('Original Block axes: ' + reason); };
  const rules = sheet => JSON.stringify(Array.from(sheet.cssRules, rule => rule.cssText));
  const whitespace = node => node.nodeType === Node.COMMENT_NODE ||
    (node.nodeType === Node.TEXT_NODE && /^[\t\n\r ]*$/.test(node.textContent));
  const edges = ['Top', 'Right', 'Bottom', 'Left'];
  const unshifted = style => style.cssFloat === 'none' && style.clear === 'none' &&
    style.transform === 'none' && style.perspective === 'none' && style.filter === 'none' &&
    style.translate === 'none' && style.rotate === 'none' && style.scale === 'none' &&
    style.offsetPath === 'none' && style.contain === 'none' && style.willChange === 'auto' &&
    style.clipPath === 'none' && ['normal', '1'].includes(style.zoom) &&
    style.contentVisibility === 'visible' && style.animationName === 'none' &&
    style.transitionDuration.split(',').every(value => parseFloat(value) === 0) &&
    style.overflowX === 'visible' && style.overflowY === 'visible' &&
    style.scrollbarGutter === 'auto';

  let ownedStyle, expectedRules;

  function verifySource(owner) {
    const sourceHead = Array.from(expected.head.childNodes);
    const liveHead = Array.from(head.childNodes).slice(1);
    const bodyShell = body.cloneNode(false);
    bodyShell.removeAttribute('class');
    if (document.compatMode !== 'CSS1Compat' || window.top !== window ||
        !document.doctype?.isEqualNode(expected.doctype) || document.scrollingElement !== html ||
        !html.cloneNode(false).isEqualNode(expected.documentElement.cloneNode(false)) ||
        !head.cloneNode(false).isEqualNode(expected.head.cloneNode(false)) ||
        !bodyShell.isEqualNode(expected.body.cloneNode(false)) ||
        !root || root.parentElement !== body || body.children.length !== 1 ||
        !root.isEqualNode(sourceRoot) || document.querySelectorAll('#test-root').length !== 1 ||
        Array.from(body.childNodes).some(node => node !== root && !whitespace(node)) ||
        base?.tagName !== 'BASE' || head.firstChild !== base || base.attributes.length !== 1 ||
        base.href !== baseUrl || liveHead.length !== sourceHead.length ||
        liveHead.some((node, index) => sourceHead[index] === sourceLink
          ? node !== owner : !node.isEqualNode(sourceHead[index])) ||
        document.scripts.length !== 0 || document.adoptedStyleSheets.length !== 0 ||
        document.styleSheets.length !== 1 || document.styleSheets[0] !== owner?.sheet ||
        owner.sheet.disabled) fail('source or stylesheet inventory changed');
    if (owner === ownedStyle &&
        (owner.textContent !== baseStyle || rules(owner.sheet) !== expectedRules)) {
      fail('base stylesheet changed');
    }
  }

  function captureFrame(candidate) {
    verifySource(ownedStyle);
    if (candidate !== root) fail('root identity changed');
    const containing = [html, body].map(node => getComputedStyle(node));
    if (containing.some(style => style.direction !== 'ltr' || style.writingMode !== 'horizontal-tb' ||
        style.display !== 'block' || style.position !== 'static' || !unshifted(style) ||
        style.whiteSpace !== 'normal' || style.textAlign !== 'start' || style.textIndent !== '0px' ||
        edges.some(edge => style['margin' + edge] !== '0px' ||
          style['padding' + edge] !== '0px' || style['border' + edge + 'Width'] !== '0px'))) {
      fail('containing frame changed');
    }
    const [boxSizing, direction] = body.className.split(' ');
    const writingMode = sourceRoot.style.writingMode;
    if (!['border-box', 'content-box'].includes(boxSizing) || !['ltr', 'rtl'].includes(direction) ||
        !['horizontal-tb', 'vertical-rl', 'vertical-lr', 'sideways-rl', 'sideways-lr'].includes(writingMode) ||
        root.children.length !== 2 ||
        Array.from(root.childNodes).some(node => node.nodeType !== Node.ELEMENT_NODE && !whitespace(node))) {
      fail('variant or source box tree changed');
    }
    const [first, second] = root.children;
    for (const [node, position, width, height, margin] of [
      [root, 'absolute', '100px', '100px', '0px'],
      [first, 'relative', '30px', '20px', '13px'],
      [second, 'relative', '20px', '30px', '0px'],
    ]) {
      const style = getComputedStyle(node);
      if (node.tagName !== 'DIV' || (node !== root && node.childNodes.length !== 0) ||
          node.style.width !== width || node.style.height !== height ||
          node.style.display !== 'block' || style.display !== 'block' || style.position !== position ||
          style.writingMode !== writingMode || style.direction !== direction || style.boxSizing !== boxSizing ||
          !unshifted(style) || (node !== root && !isUnshiftedInlinePosition(style)) ||
          style.alignSelf !== 'auto' || style.justifySelf !== 'auto' ||
          node.scrollLeft !== 0 || node.scrollTop !== 0 || style.marginInlineStart !== margin ||
          [style.marginInlineEnd, style.marginBlockStart, style.marginBlockEnd].some(value => value !== '0px') ||
          edges.some(edge => style['padding' + edge] !== '0px' || style['border' + edge + 'Width'] !== '0px') ||
          ['::before', '::after'].some(pseudo => !['none', 'normal'].includes(getComputedStyle(node, pseudo).content))) {
        fail('source box style changed');
      }
    }

    const visual = window.visualViewport;
    const width = html.clientWidth, height = html.clientHeight;
    if (!visual || visual.scale !== 1 || visual.offsetLeft !== 0 || visual.offsetTop !== 0 ||
        window.scrollX !== 0 || window.scrollY !== 0 ||
        ![width, height].every(value => Number.isFinite(value) && value >= 0) ||
        width !== visual.width || height !== visual.height || body.clientWidth !== width) {
      fail('initial containing block changed');
    }
    // The source has one block and no preceding in-flow content. Its zero-edge
    // horizontal Body supplies the inline-start anchor and static top.
    return {
      kind: 'absolute-root-local', containingWidth: width, containingHeight: height,
      containingDirection: 'ltr', hypotheticalDirection: 'ltr', hypotheticalDisplay: 'block',
      hypotheticalAnchor: 'inline-start', hypotheticalInlineOffset: 0, hypotheticalBlockOffset: 0,
      fallback: {
        initialContainingBlock: { width, height }, nearestScrollport: 'none',
        containingLimits: { width: 'unbounded', height: 'unbounded' },
      },
    };
  }

  try {
    if (!link?.isEqualNode(sourceLink) || link.disabled || savedClass !== null) {
      fail('original link or Body changed');
    }
    verifySource(link);
    ownedStyle = document.createElement('style');
    ownedStyle.textContent = baseStyle;
    link.replaceWith(ownedStyle);
    const expectedSheet = new CSSStyleSheet();
    expectedSheet.replaceSync(baseStyle);
    expectedRules = rules(expectedSheet);
    rootContextCapture = {};
    captureRootContext = captureFrame;
    return getTestData();
  } catch (error) {
    return JSON.stringify({
      schemaVersion: 4,
      ...Object.fromEntries(
        ['borderBoxLtrData', 'contentBoxLtrData', 'borderBoxRtlData', 'contentBoxRtlData']
          .map(variant => [variant, unsupportedTestData(error.message)])),
    });
  } finally {
    captureRootContext = savedCapture;
    rootContextCapture = savedProof;
    if (savedClass === null) body.removeAttribute('class');
    else body.setAttribute('class', savedClass);
    if (ownedStyle?.parentElement === head) ownedStyle.replaceWith(link);
  }
}
