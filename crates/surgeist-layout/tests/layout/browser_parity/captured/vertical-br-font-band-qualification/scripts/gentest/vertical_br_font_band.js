(source, baseStyle) => {
  const variants = ['borderBoxLtrData', 'contentBoxLtrData', 'borderBoxRtlData', 'contentBoxRtlData'];
  const parsed = new DOMParser().parseFromString(source, 'text/html');
  const html = document.documentElement;
  const body = document.body;
  const root = document.getElementById('test-root');
  const sourceRoot = parsed.getElementById('test-root');
  const baseLink = document.head.querySelector('link');
  const author = document.head.querySelector('style');
  const sourceAuthor = parsed.head.querySelector('style');
  const script = document.scripts[0];
  const expectedScript = parsed.scripts[0];
  const fail = reason => { throw new Error(`Vertical BR capture: ${reason}`); };
  const styleRules = sheet => Array.from(sheet.cssRules, rule => rule.cssText);
  const equalRules = (left, right) => JSON.stringify(styleRules(left)) === JSON.stringify(styleRules(right));
  const sheetFrom = text => {
    const sheet = new CSSStyleSheet();
    sheet.replaceSync(text);
    return sheet;
  };
  const unshifted = (style, position) => style.position === position &&
    style.cssFloat === 'none' && style.clear === 'none' &&
    style.transform === 'none' && style.perspective === 'none' && style.filter === 'none' &&
    style.contain === 'none' && style.willChange === 'auto' && style.clipPath === 'none' &&
    ['normal', '1'].includes(style.zoom) && style.contentVisibility === 'visible' &&
    style.animationName === 'none' && style.transitionDuration.split(',').every(value => parseFloat(value) === 0);
  const zeroEdges = [
    'marginTop', 'marginRight', 'marginBottom', 'marginLeft',
    'paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft',
    'borderTopWidth', 'borderRightWidth', 'borderBottomWidth', 'borderLeftWidth',
  ];

  const savedRootContext = rootContextCapture;
  const savedCapture = captureRootContext;
  try {
    if (!root || !sourceRoot || !baseLink || !author || !sourceAuthor ||
        !script || !expectedScript || document.scripts.length !== 1 ||
        !script.isEqualNode(expectedScript) || !baseLink.isEqualNode(parsed.head.querySelector('link')) ||
        !author.isEqualNode(sourceAuthor) || document.styleSheets.length !== 2 ||
        document.styleSheets[0] !== baseLink.sheet || document.styleSheets[1] !== author.sheet ||
        baseLink.disabled || author.sheet.disabled || document.adoptedStyleSheets.length !== 0 ||
        body.innerHTML !== parsed.body.innerHTML || !body.isEqualNode(parsed.body)) {
      fail('source or stylesheet inventory changed');
    }
    const base = document.createElement('style');
    base.textContent = baseStyle;
    baseLink.replaceWith(base);
    const baseRules = sheetFrom(baseStyle);
    const authorRules = sheetFrom(sourceAuthor.textContent);
    const bodyChildren = Array.from(body.childNodes);
    const writingMode = sourceRoot.style.writingMode;
    const direction = sourceRoot.style.direction;

    function captureFrame(candidate) {
      if (candidate !== root || root.parentElement !== body || body.children.length !== 1 ||
          document.querySelectorAll('#test-root').length !== 1 ||
          document.compatMode !== 'CSS1Compat' || window.top !== window ||
          document.scrollingElement !== html || document.doctype?.name !== 'html' ||
          !html.cloneNode(false).isEqualNode(parsed.documentElement.cloneNode(false)) ||
          body.innerHTML !== parsed.body.innerHTML ||
          bodyChildren.length !== body.childNodes.length ||
          bodyChildren.some((node, index) => body.childNodes[index] !== node) ||
          !root.isEqualNode(sourceRoot) ||
          !['vertical-lr', 'vertical-rl'].includes(writingMode) ||
          !['ltr', 'rtl'].includes(direction)) fail('source or document frame changed');
      if (base.parentElement !== document.head || author.parentElement !== document.head ||
          base.textContent !== baseStyle || !author.isEqualNode(sourceAuthor) ||
          document.styleSheets.length !== 2 || document.styleSheets[0] !== base.sheet ||
          document.styleSheets[1] !== author.sheet || base.sheet.disabled || author.sheet.disabled ||
          document.adoptedStyleSheets.length !== 0 ||
          !equalRules(base.sheet, baseRules) || !equalRules(author.sheet, authorRules)) {
        fail('stylesheet ownership changed');
      }

      const containing = [html, body].map(node => getComputedStyle(node));
      if (containing.some(style =>
          style.direction !== 'ltr' || style.writingMode !== 'horizontal-tb' ||
          style.display !== 'block' || !unshifted(style, 'static') ||
          style.overflowX !== 'visible' || style.overflowY !== 'visible' ||
          style.scrollbarGutter !== 'auto' || style.whiteSpace !== 'normal' ||
          style.textAlign !== 'start' || parseFloat(style.textIndent) !== 0 ||
          zeroEdges.some(property => style[property] !== '0px'))) fail('containing frame changed');

      const [before, br, after] = root.children;
      if (root.children.length !== 3 || before.tagName !== 'SPAN' || br.tagName !== 'BR' ||
          after.tagName !== 'SPAN' || [before, br, after].some(node => node.childNodes.length !== 0) ||
          root.getAttribute('data-surgeist-br-observation') !== 'font-band' ||
          root.getAttribute('data-surgeist-prepared-text') !== 'ahem-ttf-v1') fail('participant shape changed');
      const insets = ['top', 'right', 'bottom', 'left', 'inset-block-start', 'inset-block-end',
        'inset-inline-start', 'inset-inline-end'];
      if (insets.some(property => root.style.getPropertyValue(property) &&
          root.style.getPropertyValue(property) !== 'auto')) fail('authored root insets changed');
      for (const [node, display, position] of [
        [root, 'block', 'absolute'], [before, 'inline-block', 'relative'],
        [br, 'inline', 'static'], [after, 'inline-block', 'relative'],
      ]) {
        const style = getComputedStyle(node);
        if (style.display !== display || !unshifted(style, position) ||
            style.writingMode !== writingMode || style.direction !== direction ||
            (node !== root && !isUnshiftedInlinePosition(style)) ||
            node.scrollLeft !== 0 || node.scrollTop !== 0 ||
            zeroEdges.some(property => style[property] !== '0px') ||
            ['::before', '::after'].some(pseudo =>
              !['none', 'normal'].includes(getComputedStyle(node, pseudo).content))) {
          fail(`participant frame changed: ${node.tagName} ${style.display}/${style.position} ${style.writingMode}/${style.direction}`);
        }
      }
      qualifyLoadedPreparedRangeFace();
      const visual = window.visualViewport;
      const width = html.clientWidth, height = html.clientHeight;
      if (!visual || visual.scale !== 1 || visual.offsetLeft !== 0 || visual.offsetTop !== 0 ||
          window.scrollX !== 0 || window.scrollY !== 0 ||
          ![width, height].every(value => Number.isFinite(value) && value >= 0) ||
          width !== visual.width || height !== visual.height || body.clientWidth !== width) {
        fail('viewport extent authority changed');
      }
      // The sole first block has no preceding in-flow content. The
      // authenticated zero-edge Body therefore supplies a static top of zero.
      if (bodyChildren.some(node => node !== root && node.nodeType !== Node.COMMENT_NODE &&
          !(node.nodeType === Node.TEXT_NODE && /^[\t\n\r ]*$/.test(node.data)))) {
        fail('hypothetical block-start requires the first block');
      }
      return {
        kind: 'absolute-root-local', containingWidth: width, containingHeight: height,
        containingDirection: containing[0].direction, hypotheticalDirection: containing[1].direction,
        hypotheticalDisplay: sourceRoot.style.display, hypotheticalAnchor: 'inline-start', hypotheticalInlineOffset: 0,
        hypotheticalBlockOffset: 0,
        fallback: {
          initialContainingBlock: { width, height }, nearestScrollport: 'none',
          containingLimits: { width: 'unbounded', height: 'unbounded' },
        },
      };
    }
    rootContextCapture = {};
    captureRootContext = captureFrame;
    return getTestData();
  } catch (error) {
    return JSON.stringify({
      schemaVersion: 4,
      ...Object.fromEntries(variants.map(variant => [variant, unsupportedTestData(error.message)])),
    });
  } finally {
    rootContextCapture = savedRootContext;
    captureRootContext = savedCapture;
  }
}
