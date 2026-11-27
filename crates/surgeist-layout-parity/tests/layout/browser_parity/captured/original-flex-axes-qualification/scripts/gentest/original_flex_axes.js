(source, baseUrl, baseStyle) => {
  const html = document.documentElement, body = document.body;
  const root = document.getElementById('test-root');
  const originalLink = document.head.querySelector('link');
  const savedClass = body.getAttribute('class');
  const savedProof = rootContextCapture;
  const savedCapture = captureRootContext;
  const fail = reason => { throw new Error('Original Flex axes: ' + reason); };
  const whitespace = node => node.nodeType === Node.COMMENT_NODE ||
    (node.nodeType === Node.TEXT_NODE && /^[\t\n\r ]*$/.test(node.textContent));
  const sides = ['Top', 'Right', 'Bottom', 'Left'];
  const zeroEdges = style => sides.every(side =>
    style['padding' + side] === '0px' && style['border' + side + 'Width'] === '0px');
  const unshifted = style => style.cssFloat === 'none' && style.clear === 'none' &&
    style.transform === 'none' && style.perspective === 'none' && style.filter === 'none' &&
    style.translate === 'none' && style.rotate === 'none' && style.scale === 'none' &&
    style.offsetPath === 'none' && style.contain === 'none' && style.willChange === 'auto' &&
    style.clipPath === 'none' && ['normal', '1'].includes(style.zoom) &&
    style.contentVisibility === 'visible' && style.animationName === 'none' &&
    style.transitionDuration.split(',').every(value => parseFloat(value) === 0) &&
    style.overflowX === 'visible' && style.overflowY === 'visible' &&
    style.scrollbarGutter === 'auto';

  function captureFrame(candidate) {
    const proof = rootContextCapture;
    const sheets = document.styleSheets;
    const bodyShell = body.cloneNode(false);
    bodyShell.removeAttribute('class');
    if (candidate !== root || document.compatMode !== 'CSS1Compat' || window.top !== window ||
        document.scrollingElement !== html || document.baseURI !== baseUrl ||
        !html.cloneNode(false).isEqualNode(proof.html.cloneNode(false)) ||
        !bodyShell.isEqualNode(proof.body.cloneNode(false)) ||
        !root || root.parentElement !== body || root.outerHTML !== proof.sourceRoot ||
        body.children.length !== 1 || document.querySelectorAll('#test-root').length !== 1 ||
        body.childNodes.length !== proof.bodyChildren.length ||
        proof.bodyChildren.some((node, index) => node !== body.childNodes[index]) ||
        Array.from(body.childNodes).some(node => node !== root && !whitespace(node))) {
      fail('first-block source changed');
    }
    if (!rootContextScriptInventory(proof.scriptProfile, proof.scriptOwner) ||
        !rootContextStyleInventory(proof.ownedStyle, proof.preload, proof.preloadSource) ||
        document.adoptedStyleSheets.length !== 0 || sheets.length !== 1 ||
        sheets[0].ownerNode !== proof.ownedStyle || sheets[0].disabled ||
        proof.ownedStyle.parentElement !== document.head ||
        proof.ownedStyle.textContent !== baseStyle ||
        JSON.stringify(Array.from(sheets[0].cssRules, rule => rule.cssText)) !== JSON.stringify(proof.rules)) {
      fail('stylesheet or script changed');
    }
    for (const node of [html, body]) {
      const style = getComputedStyle(node);
      if (style.direction !== 'ltr' || style.writingMode !== 'horizontal-tb' ||
          style.display !== 'block' || style.position !== 'static' || !unshifted(style) ||
          style.whiteSpace !== 'normal' || style.textAlign !== 'start' ||
          style.textIndent !== '0px' || !zeroEdges(style) ||
          sides.some(side => style['margin' + side] !== '0px')) {
        fail('containing frame changed');
      }
    }

    const [boxSizing, direction] = body.className.split(' ');
    const writingMode = root.style.writingMode;
    const flexDirection = root.style.flexDirection;
    if (!['border-box', 'content-box'].includes(boxSizing) || !['ltr', 'rtl'].includes(direction) ||
        !['horizontal-tb', 'vertical-rl', 'vertical-lr', 'sideways-rl', 'sideways-lr'].includes(writingMode) ||
        !['row', 'row-reverse', 'column', 'column-reverse'].includes(flexDirection) ||
        proof.hypotheticalDisplay !== 'flex' || root.children.length !== 2 ||
        Array.from(root.childNodes).some(node => node.nodeType !== Node.ELEMENT_NODE && !whitespace(node))) {
      fail('variant or source box tree changed');
    }
    const rootStyle = getComputedStyle(root);
    if (rootStyle.flexDirection !== flexDirection || rootStyle.flexWrap !== 'nowrap' ||
        rootStyle.justifyContent !== 'space-between' || rootStyle.alignItems !== 'flex-end') {
      fail('Flex alignment changed');
    }
    const [first, second] = root.children;
    for (const [node, display, position, width, height, margins] of [
      [root, 'flex', 'absolute', '180px', '140px', ['0px', '0px', '0px', '0px']],
      [first, 'block', 'relative', '30px', '20px', ['13px', '0px', '0px', '11px']],
      [second, 'block', 'relative', '20px', '35px', ['0px', '7px', '0px', '0px']],
    ]) {
      const style = getComputedStyle(node);
      const marginNames = ['marginInlineStart', 'marginInlineEnd', 'marginBlockStart', 'marginBlockEnd'];
      if (node.tagName !== 'DIV' || (node !== root && node.childNodes.length !== 0) ||
          node.style.width !== width || node.style.height !== height || node.style.display !== display ||
          style.display !== display || style.position !== position ||
          style.writingMode !== writingMode || style.direction !== direction || style.boxSizing !== boxSizing ||
          !unshifted(style) || (node !== root && !isUnshiftedInlinePosition(style)) ||
          style.alignSelf !== 'auto' || style.justifySelf !== 'auto' || style.order !== '0' ||
          node.scrollLeft !== 0 || node.scrollTop !== 0 || !zeroEdges(style) ||
          marginNames.some((name, index) => style[name] !== margins[index]) ||
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
    // The sole block-level Flex box starts at the horizontal Body's content origin.
    return {
      kind: 'absolute-root-local', containingWidth: width, containingHeight: height,
      containingDirection: 'ltr', hypotheticalDirection: 'ltr', hypotheticalDisplay: 'flex',
      hypotheticalAnchor: 'inline-start', hypotheticalInlineOffset: 0, hypotheticalBlockOffset: 0,
      fallback: {
        initialContainingBlock: { width, height }, nearestScrollport: 'none',
        containingLimits: { width: 'unbounded', height: 'unbounded' },
      },
    };
  }

  try {
    if (savedClass !== null || !originalLink || document.baseURI !== baseUrl) fail('original document changed');
    configureRootContextCapture(source, baseStyle,
      '../../scripts/gentest/test_helper.js', '../../scripts/gentest/test_base_style.css');
    if (rootContextCapture.reason) fail(rootContextCapture.reason);
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
    const owner = rootContextCapture?.ownedStyle;
    captureRootContext = savedCapture;
    rootContextCapture = savedProof;
    if (savedClass === null) body.removeAttribute('class');
    else body.setAttribute('class', savedClass);
    if (owner?.parentElement === document.head && owner !== savedProof?.ownedStyle) owner.replaceWith(originalLink);
  }
}
