(source, baseUrl, baseStyle) => {
  const original = new DOMParser().parseFromString(source, 'text/html');
  const {documentElement: html, head, body} = document;
  const base = head.firstChild;
  const link = head.querySelector('link');
  const sourceLink = original.head.querySelector('link');
  const sourceParent = original.body.firstElementChild;
  const parent = body.firstElementChild;
  const grid = parent?.firstElementChild;
  const item = grid?.firstElementChild;
  const ratioBox = item?.firstElementChild;
  const boxes = [parent, grid, item, ratioBox];
  const variants = [
    ['borderBoxLtrData', 'border-box', 'ltr'],
    ['contentBoxLtrData', 'content-box', 'ltr'],
    ['borderBoxRtlData', 'border-box', 'rtl'],
    ['contentBoxRtlData', 'content-box', 'rtl'],
  ];
  const savedClass = body.getAttribute('class');
  const savedObservation = rootLocalObservation;
  const style = document.createElement('style');
  style.textContent = baseStyle;
  const rules = sheet => Array.from(sheet.cssRules, rule => rule.cssText).join('\n');
  const fail = reason => { throw new Error('Original percentage Flex parent: ' + reason); };

  function verifyDocument(className, owner, expectedRules) {
    const normalized = html.cloneNode(true);
    normalized.querySelector('base')?.remove();
    const normalizedBody = normalized.querySelector('body');
    if (savedClass === null) normalizedBody.removeAttribute('class');
    else normalizedBody.setAttribute('class', savedClass);
    if (owner === style) normalized.querySelector('style')?.replaceWith(sourceLink.cloneNode(true));
    if (document.compatMode !== 'CSS1Compat' || !document.doctype?.isEqualNode(original.doctype) ||
        window.top !== window || document.scrollingElement !== html ||
        document.adoptedStyleSheets.length !== 0 ||
        base?.tagName !== 'BASE' || base.attributes.length !== 1 || base.href !== baseUrl ||
        head.firstChild !== base || document.querySelectorAll('base').length !== 1 ||
        body.getAttribute('class') !== className || !normalized.isEqualNode(original.documentElement) ||
        body.firstElementChild !== parent || parent.firstElementChild !== grid ||
        grid.firstElementChild !== item || item.firstElementChild !== ratioBox ||
        document.styleSheets.length !== 1 || document.styleSheets[0] !== owner?.sheet ||
        owner.sheet.disabled || !rootContextStyleInventory(owner, null, null) ||
        (owner === style && (style.textContent !== baseStyle || rules(style.sheet) !== expectedRules))) {
      fail('source, variant or stylesheet changed');
    }
  }

  function verifyFrame(boxSizing, direction) {
    const styles = boxes.map(node => getComputedStyle(node));
    const displays = ['flex', 'grid', 'flex', 'flex'];
    if (styles.some((computed, index) => computed.display !== displays[index] ||
        computed.position !== (index === 0 ? 'absolute' : 'relative') ||
        computed.writingMode !== 'horizontal-tb' || computed.direction !== direction ||
        computed.boxSizing !== boxSizing) ||
        styles[0].flexDirection !== 'row' || styles[0].flexWrap !== 'nowrap' ||
        styles[0].alignItems !== 'start' || styles[0].justifyContent !== 'start' ||
        authoredStyleValue(parent, 'width', styles[0]) !== 'max-content' ||
        authoredStyleValue(parent, 'height', styles[0]) !== 'max-content' ||
        styles[1].justifyItems !== 'center' || styles[3].aspectRatio !== '1 / 1' ||
        grid.style.width !== '100%' || grid.style.height !== '40px' ||
        item.style.height !== '100%' || ratioBox.style.height !== '100%') {
      fail('authored sizing or source box roles changed');
    }
    if ([html, body].some(node => {
      const computed = getComputedStyle(node);
      return computed.position !== 'static' || computed.display !== 'block' ||
        computed.writingMode !== 'horizontal-tb' || computed.direction !== 'ltr' ||
        computed.overflowX !== 'visible' || computed.overflowY !== 'visible';
    })) fail('containing frame changed');
    const {clientWidth: width, clientHeight: height} = html;
    const visual = window.visualViewport;
    const rectangle = parent.getBoundingClientRect();
    if (!visual || visual.scale !== 1 || visual.offsetLeft !== 0 || visual.offsetTop !== 0 ||
        window.scrollX !== 0 || window.scrollY !== 0 ||
        width <= 0 || height <= 0 || width !== visual.width || height !== visual.height ||
        body.clientWidth !== width || rectangle.left !== 0 || rectangle.top !== 0) {
      fail('viewport extent or parent origin changed');
    }
    return {kind: 'absolute-root-local', containingWidth: width, containingHeight: height,
      containingDirection: 'ltr', hypotheticalDirection: 'ltr',
      hypotheticalDisplay: 'flex', hypotheticalAnchor: 'inline-start', hypotheticalInlineOffset: 0,
      fallback: {initialContainingBlock: {width, height}, nearestScrollport: 'none',
        containingLimits: {width: 'unbounded', height: 'unbounded'}}};
  }

  try {
    if (savedClass !== original.body.getAttribute('class') ||
        !link?.isEqualNode(sourceLink) || link.disabled || body.children.length !== 1 ||
        boxes.some((node, index) => node?.tagName !== 'DIV' ||
          node.children.length !== (index === boxes.length - 1 ? 0 : 1)) ||
        parent.className !== 'viewport' || grid.id !== 'test-root' ||
        !rootContextBodySequence(body, parent) || ratioBox.textContent !== '') {
      fail('source tree changed');
    }
    verifyDocument(savedClass, link);
    const sheet = new CSSStyleSheet();
    sheet.replaceSync(baseStyle);
    const expectedRules = rules(sheet);
    link.replaceWith(style);

    const result = {schemaVersion: 4};
    for (const [key, boxSizing, direction] of variants) {
      const className = boxSizing + ' ' + direction;
      body.className = className;
      verifyDocument(className, style, expectedRules);
      const context = verifyFrame(boxSizing, direction);
      const rectangle = parent.getBoundingClientRect();
      rootLocalObservation = {root: parent, x: rectangle.left, y: rectangle.top};
      const data = describeElement(parent, sourceParent);
      delete data.children[0].gridAlignmentProfile;
      data.gridAlignmentProfile = completeGridAlignmentProfile(data);
      data.viewport = {rootContext: context};
      verifyDocument(className, style, expectedRules);
      if (JSON.stringify(context) !== JSON.stringify(verifyFrame(boxSizing, direction))) {
        fail('measurement frame changed');
      }
      result[key] = data;
    }
    return JSON.stringify(result);
  } catch (error) {
    return JSON.stringify({schemaVersion: 4, ...Object.fromEntries(
      variants.map(([key]) => [key, unsupportedTestData(error.message)]),
    )});
  } finally {
    rootLocalObservation = savedObservation;
    if (savedClass === null) body.removeAttribute('class');
    else body.setAttribute('class', savedClass);
    if (style.parentElement === head) style.replaceWith(link);
  }
}
