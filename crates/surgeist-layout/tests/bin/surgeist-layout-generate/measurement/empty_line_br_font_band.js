async (source, baseStyle, fontId, fontDigest, fontBytes) => {
  const variants = ['borderBoxLtrData', 'contentBoxLtrData', 'borderBoxRtlData', 'contentBoxRtlData'];
  const parsed = new DOMParser().parseFromString(source, 'text/html');
  const html = document.documentElement;
  const body = document.body;
  const root = document.getElementById('test-root');
  const sourceRoot = parsed.getElementById('test-root');
  const baseLink = document.head.querySelector('link');
  const author = document.head.querySelector('style');
  const originalAuthor = parsed.head.querySelector('style');
  const fail = reason => { throw new Error(`Empty-line BR capture: ${reason}`); };
  const sheetFrom = css => {
    const sheet = new CSSStyleSheet();
    sheet.replaceSync(css);
    return sheet;
  };
  const rules = sheet => Array.from(sheet.cssRules, rule => rule.cssText);
  const sameRules = (left, right) => JSON.stringify(rules(left)) === JSON.stringify(rules(right));
  const zeroEdges = [
    'marginTop', 'marginRight', 'marginBottom', 'marginLeft',
    'paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft',
    'borderTopWidth', 'borderRightWidth', 'borderBottomWidth', 'borderLeftWidth',
  ];
  const unshifted = style => (style.position === 'absolute' || isUnshiftedInlinePosition(style)) &&
    style.clear === 'none' &&
    style.cssFloat === 'none' && style.transform === 'none' && style.perspective === 'none' &&
    style.filter === 'none' && style.contain === 'none' && style.willChange === 'auto' &&
    style.clipPath === 'none' && ['normal', '1'].includes(style.zoom) &&
    style.contentVisibility === 'visible' && style.animationName === 'none' &&
    style.transitionDuration.split(',').every(value => parseFloat(value) === 0);
  const noGeneratedContent = node => ['::before', '::after'].every(pseudo =>
    ['none', 'normal'].includes(getComputedStyle(node, pseudo).content));

  const savedContext = rootContextCapture;
  const savedCapture = captureRootContext;
  const savedClass = body.className;
  try {
    if (!root || !sourceRoot || !baseLink || !baseLink.isEqualNode(parsed.head.querySelector('link')) ||
        !!author !== !!originalAuthor || (author && !author.isEqualNode(originalAuthor)) ||
        document.scripts.length !== 1 || !document.scripts[0].isEqualNode(parsed.scripts[0]) ||
        !body.isEqualNode(parsed.body) || !html.cloneNode(false).isEqualNode(parsed.documentElement.cloneNode(false)) ||
        document.compatMode !== 'CSS1Compat' || document.doctype?.name !== 'html' || window.top !== window ||
        document.scrollingElement !== html || document.adoptedStyleSheets.length !== 0 ||
        root.parentElement !== body || body.children.length !== 1 ||
        body.childNodes.length !== parsed.body.childNodes.length ||
        Array.from(body.childNodes).some(node => node !== root && node.nodeType !== Node.COMMENT_NODE &&
          !(node.nodeType === Node.TEXT_NODE && /^[\t\n\r ]*$/.test(node.data)))) fail('original document changed');
    const sheets = () => Array.from(document.styleSheets);
    const ownsSheets = base => sheets().length === (author ? 2 : 1) &&
      sheets()[0] === base.sheet && !base.sheet.disabled &&
      (!author || (sheets()[1] === author.sheet && !author.sheet.disabled));
    if (!ownsSheets(baseLink) || baseLink.disabled) fail('stylesheet inventory changed');

    const base = document.createElement('style');
    base.textContent = baseStyle;
    baseLink.replaceWith(base);
    const expectedBase = sheetFrom(baseStyle);
    const expectedAuthor = originalAuthor ? sheetFrom(originalAuthor.textContent) : null;
    const face = referenceMetricFace();
    const fontRules = Array.from(base.sheet.cssRules).filter(rule => rule.type === CSSRule.FONT_FACE_RULE);
    const data = fontRules[0]?.style.src.match(/^url\(["']?data:application\/font-woff2;charset=utf-8;base64,([A-Za-z0-9+/=]+)["']?\) format\(["']?woff2["']?\)$/)?.[1];
    if (fontRules.length !== 1 || !data) fail('stylesheet face must have one embedded WOFF2 source');
    const payload = Uint8Array.from(atob(data), character => character.charCodeAt(0));
    if (payload.length !== fontBytes.length || payload.some((byte, index) => byte !== fontBytes[index]) ||
        await face.load() !== face) fail('stylesheet font resource changed');
    await document.fonts.ready;

    const [first, second] = root.childNodes;
    const insets = ['top', 'right', 'bottom', 'left', 'inset-block-start', 'inset-block-end',
      'inset-inline-start', 'inset-inline-end'];
    if (insets.some(property => root.style.getPropertyValue(property) &&
        root.style.getPropertyValue(property) !== 'auto')) fail('authored root insets changed');
    const writingMode = sourceRoot.style.writingMode || 'horizontal-tb';
    const paragraphs = new Map();
    function captureFrame(candidate) {
      if (candidate !== root || !root.isEqualNode(sourceRoot) ||
          document.getElementById('test-root') !== root || root.parentElement !== body ||
          root.childNodes.length !== 2 || root.childNodes[0] !== first || root.childNodes[1] !== second ||
          [first, second].some(node => node.tagName !== 'BR' || node.childNodes.length !== 0) ||
          root.getAttribute('data-surgeist-br-observation') !== 'font-band') fail('original BR order changed');
      if (!ownsSheets(base) || base.textContent !== baseStyle || !sameRules(base.sheet, expectedBase) ||
          (author && (!author.isEqualNode(originalAuthor) || !sameRules(author.sheet, expectedAuthor))) ||
          referenceMetricFace() !== face || face.status !== 'loaded') fail('loaded stylesheet face changed');

      const containing = [html, body].map(node => getComputedStyle(node));
      if (containing.some(style => style.writingMode !== 'horizontal-tb' ||
          style.position !== 'static' || style.display !== 'block' || !unshifted(style) ||
          style.overflowX !== 'visible' || style.overflowY !== 'visible' ||
          style.scrollbarGutter !== 'auto' || style.whiteSpace !== 'normal' ||
          style.textAlign !== 'start' || parseFloat(style.textIndent) !== 0 ||
          zeroEdges.some(edge => style[edge] !== '0px')) ||
          ![html, body, root, first, second].every(noGeneratedContent)) fail('first-block frame changed');
      const direction = sourceRoot.style.direction || (body.classList.contains('rtl') ? 'rtl' : 'ltr');
      if (!['horizontal-tb', 'vertical-rl'].includes(writingMode) ||
          !['ltr', 'rtl'].includes(direction) || containing[0].direction !== 'ltr' ||
          !['ltr', 'rtl'].includes(containing[1].direction)) fail('source writing frame changed');
      for (const node of [root, first, second]) {
        const style = getComputedStyle(node);
        if (style.writingMode !== writingMode || style.direction !== direction ||
            style.display !== (node === root ? 'block' : 'inline') ||
            style.position !== (node === root ? 'absolute' : 'static') ||
            !unshifted(style) || zeroEdges.some(edge => style[edge] !== '0px') ||
            node.scrollLeft !== 0 || node.scrollTop !== 0 ||
            referenceFontFamily(style.fontFamily) !== 'ahem' || style.fontSize !== '20px' ||
            style.lineHeight !== '30px' || style.fontStyle !== 'normal' || style.fontWeight !== '400' ||
            !['normal', '100%'].includes(style.fontStretch) || style.fontFeatureSettings !== 'normal' ||
            style.fontVariationSettings !== 'normal' || style.fontVariant !== 'normal' ||
            style.fontSizeAdjust !== 'none' || style.textOrientation !== 'mixed' ||
            style.textCombineUpright !== 'none' || style.dominantBaseline !== 'auto' ||
            style.unicodeBidi !== (node === root ? 'isolate' : 'normal') || style.verticalAlign !== 'baseline' ||
            style.whiteSpace !== 'normal' || style.wordBreak !== 'normal' ||
            style.overflowWrap !== 'normal' || style.letterSpacing !== 'normal' ||
            !['normal', '0px'].includes(style.wordSpacing) || style.color !== 'rgb(0, 128, 0)') {
          fail(`source break style changed: ${node.tagName}`);
        }
      }
      const visual = window.visualViewport;
      const width = html.clientWidth, height = html.clientHeight;
      if (!visual || visual.scale !== 1 || visual.offsetLeft !== 0 || visual.offsetTop !== 0 ||
          window.scrollX !== 0 || window.scrollY !== 0 ||
          ![width, height].every(value => Number.isFinite(value) && value >= 0) ||
          width !== visual.width || height !== visual.height || body.clientWidth !== width) {
        fail('viewport extent authority changed');
      }
      paragraphs.set(body.className, {
        font: {id: fontId, sha256: fontDigest, faceIndex: 0, family: 'ahem', style: 'normal', weight: 400,
          stretch: 'normal', features: 'normal', variations: 'normal', synthesis: 'none', loaded: true},
        direction, writingMode,
        baseStyle: {fontSize: 20, lineHeight: 30, whitespace: 'collapse', wrap: 'word',
          letterSpacing: 0, wordSpacing: 0, color: [0, Math.fround(128 / 255), 0, 1], locale: root.closest('[lang]')?.lang ?? null},
        origins: [], connections: [], atomics: [],
        controls: [first, second].map((_, child) => ({child, at: 0, kind: 'forced-break'})),
        order: [first, second].map((_, index) => ({kind: 'control', index, affinity: 'before'})),
      });
      return {
        kind: 'absolute-root-local', containingWidth: width, containingHeight: height,
        containingDirection: containing[0].direction, hypotheticalDirection: containing[1].direction,
        hypotheticalDisplay: 'block', hypotheticalAnchor: 'inline-start', hypotheticalInlineOffset: 0,
        ...(writingMode === 'vertical-rl' ? {hypotheticalBlockOffset: 0} : {}),
        fallback: {
          initialContainingBlock: {width, height}, nearestScrollport: 'none',
          containingLimits: {width: 'unbounded', height: 'unbounded'},
        },
      };
    }

    rootContextCapture = {};
    captureRootContext = captureFrame;
    const response = JSON.parse(getTestData());
    for (const [key, bodyClass] of [
      ['borderBoxLtrData', 'border-box ltr'], ['contentBoxLtrData', 'content-box ltr'],
      ['borderBoxRtlData', 'border-box rtl'], ['contentBoxRtlData', 'content-box rtl'],
    ]) {
      const variant = response[key];
      if (variant.unsupportedReason) continue;
      if (variant.children.length !== 2 || variant.children.some(child =>
          child.tagName !== 'br' || !child.browserBrObservation)) fail('captured break geometry changed');
      variant.layoutReadyInlineRoot = true;
      variant.preparedParagraph = paragraphs.get(bodyClass);
      for (const child of variant.children) {
        delete child.style.inlineMetrics;
        child.lineControlParticipation = {kind: 'forced-break'};
      }
    }
    return JSON.stringify(response);
  } catch (error) {
    return JSON.stringify({schemaVersion: 4,
      ...Object.fromEntries(variants.map(variant => [variant, unsupportedTestData(error.message)]))});
  } finally {
    body.className = savedClass;
    rootContextCapture = savedContext;
    captureRootContext = savedCapture;
  }
}
