() => {
  const qualifyStockSubtree = qualifyRootLocalSubtree;
  try {
    qualifyRootLocalSubtree = (root, unshifted, preparedRanges, compositionImage) => {
      const nodes = Array.from(root.childNodes);
      const fail = () => { throw new Error('Root context: HorizontalBrFontBandSource'); };
      if (root.tagName !== 'DIV' || nodes.length !== 3 ||
          nodes.map(node => node.tagName).join(',') !== 'SPAN,BR,SPAN' ||
          nodes.some(node => node.childNodes.length !== 0) ||
          root.getAttribute('data-surgeist-br-observation') !== 'font-band' ||
          root.getAttribute('data-surgeist-prepared-text') !== 'ahem-ttf-v1' ||
          root.getAttribute('data-surgeist-layout-ready-inline') !== 'true' ||
          preparedRanges || compositionImage) fail();
      for (const [node, display, position] of [
        [root, 'block', 'absolute'], [nodes[0], 'inline-block', 'relative'],
        [nodes[1], 'inline', 'static'], [nodes[2], 'inline-block', 'relative'],
      ]) {
        const style = getComputedStyle(node);
        if (style.display !== display || style.position !== position ||
            style.writingMode !== 'horizontal-tb' || style.cssFloat !== 'none' ||
            style.clear !== 'none' || !unshifted(style) || style.clipPath !== 'none' ||
            node.scrollLeft !== 0 || node.scrollTop !== 0 ||
            !['normal', 'none'].includes(style.content) ||
            ['::before', '::after'].some(pseudo =>
              !['none', 'normal'].includes(getComputedStyle(node, pseudo).content))) fail();
      }
      qualifyLoadedPreparedRangeFace();
    };
    return getTestData();
  } finally {
    qualifyRootLocalSubtree = qualifyStockSubtree;
  }
}
