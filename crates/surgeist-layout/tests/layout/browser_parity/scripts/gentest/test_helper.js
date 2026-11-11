
function getScrollBarWidth() {
  let el = document.createElement("div");
  el.style.cssText = "overflow:scroll; visibility:hidden; position:absolute;";
  document.body.appendChild(el);
  let width = el.offsetWidth - el.clientWidth;
  el.remove();
  return width;
}

class TrackSizingParser {
  static INITIAL_CHAR_REGEX = /[a-z-A-Z0-9]/;
  static TOKEN_CHAR_REGEX = /[-\.a-z-A-Z0-9%]/;

  constructor(input, options = { allowFrUnits: true }) {
    this.input = input;
    this.index = 0;
    this.options = options;
  }

  parseList() {
    return this._parseItemList(' ', null);
  }

  parseSingleItem() {
    return this._parseItem();
  }

  _parseItemList(separator, terminator = null) {
    if (!separator) throw new Error('No terminator passed');
    let tokenList = [];
    // console.debug('Parse List', this.index, this.input.slice(this.index));

    while (this.index < this.input.length) {
      const char = this.input[this.index];
      // console.debug(this.index, char);

      // Skip whitespace
      if (char === ' ') { this.index++; continue; }

      if (terminator && char === terminator) {
        return tokenList;
      }

      if (char === '[') {
        const names = this._parseLineNames();
        const previous = tokenList[tokenList.length - 1];
        if (previous?.kind === 'subgrid') previous.lineNames.push(names);
        else tokenList.push({ kind: 'line-names', names });
        continue;
      }

      if (TrackSizingParser.INITIAL_CHAR_REGEX.test(char)) {
        const token = this._parseItem();
        tokenList.push(token);

        const nextChar = this.input[this.index];
        if ((terminator && nextChar === terminator) || !terminator && !nextChar) {
          return tokenList;
        } else {
          this.index++;
          continue;
        }
      }

      throw new Error(`Invalid start of token ${char}`);
    }

    return tokenList;
  }

  _parseItem() {
    let token = '';
    // console.debug('Parse Item', this.index, this.input.slice(this.index));

    while (this.index < this.input.length) {
      const char = this.input[this.index];
      // console.debug(this.index, char);

      if (TrackSizingParser.TOKEN_CHAR_REGEX.test(char)) {
        token += char;
        this.index++;
        continue;
      }

      if (char === '(') {
        if (['calc', 'min', 'max', 'clamp'].includes(token)) {
          return { kind: 'scalar', ...this._parseSizingItem(token) };
        }
        if (!['fit-content', 'minmax', 'repeat'].includes(token)) {
          throw new Error(`Unsupported grid track sizing function ${token}`);
        }
        this.index++;
        const args = this._parseItemList(',', ')');
        this.index++;
        return { kind: 'function', name: token, arguments: args };
      }

      if (token === 'subgrid') return { kind: 'subgrid', lineNames: [] };
      return { kind: 'scalar', ...this._parseScalarItem(token) };
    }
    if (token === 'subgrid') return { kind: 'subgrid', lineNames: [] };
    return { kind: 'scalar', ...this._parseScalarItem(token) };

  }

  _parseSizingItem(name) {
    const body = this._parseBalancedParenthesized();
    const dimension = parseSizingDimension(`${name}(${body})`, { allowFrUnits: this.options.allowFrUnits });
    if (!dimension) throw new Error(`Invalid scalar grid track sizing function ${name}(${body})`);
    return dimension;
  }

  _parseBalancedParenthesized() {
    if (this.input[this.index] !== '(') throw new Error('Expected parenthesized calc value');
    this.index++;
    let depth = 1;
    let body = '';
    while (this.index < this.input.length) {
      const char = this.input[this.index];
      this.index++;
      if (char === '(') {
        depth++;
        body += char;
        continue;
      }
      if (char === ')') {
        depth--;
        if (depth === 0) return body;
        body += char;
        continue;
      }
      body += char;
    }
    throw new Error('Unterminated calc grid track sizing function');
  }

  _parseLineNames() {
    this.index++;
    let names = '';
    while (this.index < this.input.length && this.input[this.index] !== ']') {
      names += this.input[this.index];
      this.index++;
    }
    if (this.input[this.index] !== ']') throw new Error('Unterminated grid line name list');
    this.index++;
    return names.trim() === '' ? [] : names.trim().split(/\s+/);
  }

  _parseScalarItem(item) {
    const res = parseRepetition(item) || parseSizingDimension(item, { allowFrUnits: this.options.allowFrUnits });
    if (!res) throw new Error(`Invalid scalar grid track sizing function ${item}`);
    return res;
  }

}


// The capture profile is source-qualified, not a CSS cascade implementation.
// Its producer is the adapter holding the exact HTML/base/helper input bytes.
let rootContextCapture;
// One synchronous observation occurrence; restored after every variant.
let rootLocalObservation;

// Closed source facts for the two additional root-local capabilities. These
// predicates do not read used rectangles, body height, fonts or line metrics.
function rootContextLiteralPx(value) {
  if (!/^(?:\d+(?:\.\d*)?|\.\d+)px$/.test(value)) return undefined;
  const number = Number(value.slice(0, -2));
  return Number.isFinite(number) && number >= 0 ? number : undefined;
}

function rootContextBodyIndent(body) {
  if (!body.style.length) return 0;
  if (body.style.length !== 1 || body.style.item(0) !== 'text-indent') return undefined;
  return rootContextLiteralPx(body.style.textIndent);
}

function rootContextStaticSpacer(node) {
  if (node.nodeType !== Node.ELEMENT_NODE || node.tagName !== 'DIV' ||
      node.childNodes.length || Array.from(node.attributes).some(attribute => attribute.name !== 'style')) return false;
  const properties = Array.from(node.style);
  return properties.length === 4 &&
    properties.every(property => ['display', 'position', 'width', 'height'].includes(property)) &&
    node.style.display === 'block' && node.style.position === 'static' &&
    rootContextLiteralPx(node.style.width) === 0 && rootContextLiteralPx(node.style.height) > 0;
}

function rootContextBodySequence(body, root) {
  let reachedRoot = false;
  for (const node of body.childNodes) {
    if (node === root) { reachedRoot = true; continue; }
    if (node.nodeType === Node.COMMENT_NODE ||
        (node.nodeType === Node.TEXT_NODE && /^[\t\n\r ]*$/.test(node.textContent))) continue;
    if (reachedRoot || !rootContextStaticSpacer(node)) return false;
  }
  return reachedRoot;
}

function qualifyRootLiteralIndent(root, proof, parentStyle, htmlStyle, width) {
  const fail = () => { throw new Error('Root context: UnrepresentedInheritedIndent'); };
  const indent = rootContextBodyIndent(proof.body);
  if (indent === undefined || indent > width || htmlStyle.direction !== 'ltr' ||
      parentStyle.direction !== 'ltr' || !['flex', 'inline-flex'].includes(proof.hypotheticalDisplay) ||
      Array.from(proof.body.children).some(node => node.id !== 'test-root')) fail();
  // text-indent inherits. Only empty fixed native Flex items are admitted here:
  // no descendant inline line can consume a fact omitted by its normalized input.
  if (Array.from(root.childNodes).some(node => node.nodeType !== Node.ELEMENT_NODE &&
      node.nodeType !== Node.COMMENT_NODE &&
      !(node.nodeType === Node.TEXT_NODE && /^[\t\n\r ]*$/.test(node.textContent)))) fail();
  if (!root.children.length || getComputedStyle(root).display !== 'flex') fail();
  for (const child of root.children) {
    if (child.tagName !== 'DIV' || child.childNodes.length ||
        Array.from(child.attributes).some(attribute => attribute.name !== 'style') ||
        Array.from(child.style).some(property => !['width', 'height', 'flex-shrink'].includes(property)) ||
        !(rootContextLiteralPx(child.style.width) > 0) ||
        !(rootContextLiteralPx(child.style.height) > 0) || child.style.flexShrink !== '0') fail();
  }
  if (rootContextLiteralPx(parentStyle.textIndent) !== indent) fail();
  return proof.hypotheticalDisplay === 'inline-flex' ? indent : 0;
}

// Closed font-wide metric evidence for atomic-only paragraphs. This is not a
// registered Text font, a Prepared paragraph or a provider-issued identity.
const REFERENCE_FONT_SHA256 = 'e450dd6e1bc6fb10ebc6edecef1182dcd8cad6e12e54c363ad4ba26f6b5db46c';
let namedReferenceFontState;
function referenceFontFamily(value) { return value.replace(/^['"]|['"]$/g, '').toLowerCase(); }
function isUnshiftedInlinePosition(style) {
  return ['static', 'relative'].includes(style.position) &&
    (style.position !== 'relative' || ['top', 'right', 'bottom', 'left'].every(edge =>
      style[edge] === 'auto' || parseCssPx(style[edge]) === 0));
}
function atomicParagraphSource(element) {
  if (!hasLayoutReadyInlineFixture(element)) return undefined;
  const style = getComputedStyle(element);
  if (element.tagName !== 'DIV' || style.display !== 'block' ||
      !['vertical-rl', 'vertical-lr'].includes(style.writingMode) || parseCssPx(style.lineHeight) !== 0) return undefined;
  if (element.closest('[data-surgeist-prepared-text]')) return undefined;
  const order = Array.from(element.childNodes).map((node, source) => ({ node, source })).filter(({node}) =>
    node.nodeType !== Node.COMMENT_NODE &&
    !(node.nodeType === Node.TEXT_NODE && /^[\t\n\r ]*$/.test(node.textContent)));
  if (order.length !== 3 || order[1].node.tagName !== 'BR' ||
      ![order[0], order[2]].every(({node}) => node.nodeType === Node.ELEMENT_NODE && isLoweredAtomicInline(node))) return undefined;
  return order;
}
function hasDefaultFontMetricDescriptors(face) {
  return face.unicodeRange.toUpperCase() === 'U+0-10FFFF' &&
    face.ascentOverride === 'normal' && face.descentOverride === 'normal' &&
    face.lineGapOverride === 'normal' && face.sizeAdjust === '100%';
}
function referenceMetricFace() {
  const faces = Array.from(document.fonts).filter(face => referenceFontFamily(face.family) === 'ahem');
  if (faces.length !== 1) throw new Error(faces.length ? 'CompetingFontResource' : 'MissingFontResource');
  const face = faces[0];
  if (face.style !== 'normal' || !['normal', '400'].includes(face.weight) ||
      !['normal', '100%'].includes(face.stretch) || !hasDefaultFontMetricDescriptors(face) ||
      face.featureSettings !== 'normal' || face.variationSettings !== 'normal') throw new Error('UnsupportedFontResourceSettings');
  return face;
}
function gridAlignmentSource(element) {
  if (!hasLayoutReadyInlineFixture(element)) return undefined;
  const own = getComputedStyle(element);
  if (!['grid', 'inline-grid'].includes(own.display) ||
      !['vertical-rl', 'vertical-lr'].includes(own.writingMode)) return undefined;
  return own;
}
function hasClosedReferenceFont(own) {
  return referenceFontFamily(own.fontFamily) === 'ahem' && own.fontSize === '10px' &&
    own.fontStyle === 'normal' && own.fontWeight === '400' && ['normal', '100%'].includes(own.fontStretch) &&
    own.fontFeatureSettings === 'normal' && own.fontVariationSettings === 'normal' &&
    own.fontVariant === 'normal' && own.fontSizeAdjust === 'none';
}
function gridAlignmentReferenceForElement(element) {
  const own = gridAlignmentSource(element);
  if (!own) return undefined;
  const fail = reason => { throw new Error('Grid alignment reference: ' + reason); };
  if (element.namespaceURI !== 'http://www.w3.org/1999/xhtml' || !['DIV', 'SPAN'].includes(element.tagName)) fail('UnsupportedGridSourceRole');
  for (let node = element; node; node = node.parentElement) {
    if (['dominant-baseline', 'alignment-baseline'].some(property => node.style.getPropertyValue(property))) fail('UnsupportedReferenceOverride');
    if (node.id === 'test-root') break;
  }
  if (!['ltr', 'rtl'].includes(own.direction) || !['mixed', 'upright'].includes(own.textOrientation) ||
      own.dominantBaseline !== 'auto' || !['auto', 'baseline'].includes(own.alignmentBaseline) ||
      own.textCombineUpright !== 'none') fail('UnsupportedReferenceContext');
  if (!hasClosedReferenceFont(own)) fail('UnsupportedFirstFontAuthority');
  try { requireNamedReferenceFont(); } catch (error) { fail(error.message); }
  return {profile:1, display:own.display, writingMode:own.writingMode, direction:own.direction,
    textOrientation:own.textOrientation, dominantBaseline:own.dominantBaseline, alignmentBaseline:own.alignmentBaseline,
    font:{sha256:REFERENCE_FONT_SHA256,family:'ahem',size:10,style:'normal',weight:400,stretch:'normal',loaded:true},reference:'central'};
}
function completeGridAlignmentProfile(root) {
  const owners = [];
  function visit(node, path) {
    if (node.gridAlignmentReference) owners.push(path);
    (node.children || []).forEach((child, index) => visit(child, path === '/' ? '/' + index : path + '/' + index));
  }
  visit(root, '/');
  return {profile:1, owners};
}

async function prepareNamedReferenceFont() {
  const proof = rootContextCapture;
  if (!proof || proof.reason) return;
  const root = document.getElementById('test-root');
  if (!root || ![root, ...root.querySelectorAll('*')].some(element => atomicParagraphSource(element) || gridAlignmentSource(element))) return;
  if (namedReferenceFontState?.owner === proof.ownedStyle) {
    // Preserve the original load outcome and face. Per-capture checks reject a
    // transient competitor/missing face; repeated setup cannot reload or repair it.
    return;
  }
  const state = { owner: proof.ownedStyle, face: undefined, loaded: false, error: undefined };
  namedReferenceFontState = state;
  try {
    state.face = referenceMetricFace();
    let loaded;
    try { loaded = await state.face.load(); } catch (_) { throw new Error('FontLoadFailed'); }
    if (loaded !== state.face) throw new Error('ForeignLoadedFontResource');
    await document.fonts.ready;
    if (state.face.status !== 'loaded' || referenceMetricFace() !== state.face) throw new Error('FontResourceChanged');
    state.loaded = true;
  } catch (error) { state.error = error.message || 'FontLoadFailed'; }
}
function requireNamedReferenceFont() {
  const state = namedReferenceFontState;
  if (!state || state.owner !== rootContextCapture?.ownedStyle) throw new Error('MissingLoadedFontAuthority');
  if (state.error) throw new Error(state.error);
  if (!state.loaded || state.face.status !== 'loaded') throw new Error('FontNotLoaded');
  if (referenceMetricFace() !== state.face) throw new Error('FontResourceChanged');
}
function atomicParagraphMetricsForElement(element, children) {
  const order = atomicParagraphSource(element);
  if (!order) return undefined;
  const fail = reason => { throw new Error('Atomic paragraph metrics: ' + reason); };
  try { requireNamedReferenceFont(); } catch (error) { fail(error.message); }
  const style = getComputedStyle(element);
  for (let node = element; node; node = node.parentElement) {
    if (['line-fit-edge', 'text-box-trim', 'text-box-edge', 'text-fit', 'dominant-baseline', 'alignment-baseline'].some(property => node.style.getPropertyValue(property)) ||
        ['data-surgeist-inline-struts', 'data-surgeist-inline-breaks', 'data-surgeist-inline-bidi-levels'].some(attribute => node.hasAttribute(attribute))) fail('UnsupportedMetricOverride');
    if (node.id === 'test-root') break;
  }
  if (element.tagName !== 'DIV' || style.display !== 'block' ||
      !['ltr', 'rtl'].includes(style.direction) || !['mixed', 'upright'].includes(style.textOrientation) ||
      (style.textOrientation === 'upright' && style.direction !== 'ltr') ||
      style.dominantBaseline !== 'auto' || !['auto', 'baseline'].includes(style.alignmentBaseline) ||
      style.verticalAlign !== 'baseline' ||
      style.textCombineUpright !== 'none' || !['normal', 'isolate'].includes(style.unicodeBidi) ||
      style.whiteSpace !== 'normal' || style.textEmphasisStyle !== 'none') fail('UnsupportedReferenceContext');
  const matchingFont = own => hasClosedReferenceFont(own) && parseCssPx(own.lineHeight) === 0;
  if (!matchingFont(style) || children.length !== 3) fail('UnsupportedFirstFontMetrics');
  const level = style.direction === 'rtl' ? 1 : 0;
  for (const [index, row] of order.entries()) {
    const own = getComputedStyle(row.node), described = children[index];
    if (['line-fit-edge', 'text-box-trim', 'text-box-edge', 'text-fit', 'dominant-baseline', 'alignment-baseline'].some(property => row.node.style.getPropertyValue(property))) fail('UnsupportedMetricOverride');
    if (!matchingFont(own) || own.writingMode !== style.writingMode || own.direction !== style.direction ||
        own.textOrientation !== style.textOrientation || own.verticalAlign !== 'baseline' ||
        own.cssFloat !== 'none' || !['static', 'relative'].includes(own.position) ||
        !['auto', 'baseline'].includes(own.alignmentBaseline) ||
        own.dominantBaseline !== 'auto' || own.textCombineUpright !== 'none' ||
        own.unicodeBidi !== 'normal' || own.textEmphasisStyle !== 'none') fail('MismatchedOccurrenceReference');
    if (row.node.childNodes.length || !isUnshiftedInlinePosition(own)) fail('UnsupportedOccurrence');
    if (index === 1) {
      if (own.display !== 'inline' || described.lineControlParticipation?.kind !== 'forced-break' ||
          described.style?.inlineMetrics?.baseline !== 0 || described.style?.inlineMetrics?.lineHeight !== 0) fail('MismatchedBreakMetrics');
    } else if (row.node.tagName !== 'SPAN' || own.display !== 'inline-block' ||
        described.atomicInlineParticipation?.bidiLevel !== level ||
        described.atomicInlineParticipation?.followingBreak !== 'prohibited') fail('MismatchedAtomicOccurrence');
  }
  // Actual WOFF2 Ahem1.1: UPEM1000, hhea/OS2 800/-200, no BASE/vhea/MVAR.
  // Used Central at10px is5/5; H0 half-leading is-5/-5. No glyph or rectangle.
  return { profile: 1, font: { sha256: REFERENCE_FONT_SHA256, family: 'ahem', size: 10,
      style: 'normal', weight: 400, stretch: 'normal', loaded: true },
    writingMode: style.writingMode, direction: style.direction, textOrientation: style.textOrientation,
    dominantBaseline: 'auto', alignmentBaseline: style.alignmentBaseline, verticalAlign: style.verticalAlign,
    lineFit: 'leading', reference: 'central',
    before: 0, after: 0, lineHeight: 0,
    order: order.map((row, child) => child === 1
      ? { kind: 'break', child, source: row.source, reference: 'central', before: 0, after: 0 }
      : { kind: 'atomic', child, source: row.source, bidiLevel: level }) };
}

const PREPARED_RANGE_FRAME = 'prepared-range-root-border-v1';

function preparedRangePreload(link, head, fontUrl) {
  return link?.parentElement === head && link.attributes.length === 5 &&
    link.getAttribute('rel') === 'preload' && link.getAttribute('as') === 'font' &&
    link.getAttribute('type') === 'font/ttf' && link.getAttribute('crossorigin') === '' &&
    link.hasAttribute('href') && new URL(link.getAttribute('href'), document.baseURI).href === fontUrl;
}

function rootContextStyleInventory(ownedStyle, preload, preloadSource) {
  const resources = Array.from(document.querySelectorAll('style, link'));
  return resources.length === (preload ? 2 : 1) && resources.includes(ownedStyle) &&
    (!preload || (resources.includes(preload) && preload.parentElement === document.head &&
      preload.outerHTML === preloadSource));
}

function rootContextScriptAttributes(script) {
  return JSON.stringify(Array.from(script.attributes, attribute =>
    [attribute.name, attribute.value]).sort(([left], [right]) => left.localeCompare(right)));
}

function rootContextSourceScriptProfile(parsed, helperUrl) {
  const scripts = Array.from(parsed.querySelectorAll('script'));
  if (scripts.length === 0) return { kind: 'absent' };
  const script = scripts[0];
  if (scripts.length !== 1 || script.parentElement !== parsed.head ||
      script.attributes.length !== 1 || script.attributes[0].name !== 'src' ||
      script.textContent.trim()) return null;
  try {
    if (new URL(script.getAttribute('src'), document.baseURI).href !== helperUrl) return null;
  } catch (_) {
    return null;
  }
  return { kind: 'helper', attributes: rootContextScriptAttributes(script),
    text: script.textContent, url: helperUrl };
}

function rootContextScriptInventory(profile, owner) {
  const scripts = Array.from(document.querySelectorAll('script'));
  if (profile.kind === 'absent') return owner === null && scripts.length === 0;
  return profile.kind === 'helper' && scripts.length === 1 && scripts[0] === owner &&
    owner.isConnected && owner.parentElement === document.head &&
    rootContextScriptAttributes(owner) === profile.attributes &&
    owner.textContent === profile.text && owner.src === profile.url;
}

function configureRootContextCapture(source, baseStyle, helperReference, styleReference, preparedFontReference, compositionProfile) {
  const previous = rootContextCapture;
  // Failed reconfiguration retains the authenticated owners for revalidation
  // after restoration, but the refusal prevents every intervening capture.
  const reject = reason => {
    rootContextCapture = previous?.ownedStyle ? { ...previous, reason } : { reason };
  };
  try {
    const parsed = new DOMParser().parseFromString(source, 'text/html');
    if (parsed.doctype?.name.toLowerCase() !== 'html' ||
        parsed.querySelectorAll('#test-root').length !== 1 ||
        parsed.querySelectorAll('body').length !== 1) return reject('SourceDocument');
    if (parsed.documentElement.style.length || rootContextBodyIndent(parsed.body) === undefined) return reject('AuthoredBodyOrHtmlStyle');
    if ([parsed.documentElement, parsed.body].some(node =>
        Array.from(node.attributes).some(attribute =>
          !['lang', 'dir', 'class'].includes(attribute.name) && !(node === parsed.body && attribute.name === 'style')))) return reject('SourceAncestorAttribute');
    if (parsed.querySelector('style, iframe, frame, object, embed, base')) return reject('SourceFrameOrStyle');
    if (Array.from(parsed.querySelectorAll('*')).some(node =>
        Array.from(node.attributes).some(attribute => /^on/i.test(attribute.name)))) return reject('SourceScript');
    const links = Array.from(parsed.querySelectorAll('link'));
    const preparedRanges = typeof preparedFontReference === 'string';
    const flexComposition = compositionProfile === 'fixed-sideways-root-v1';
    if (compositionProfile !== undefined && (!flexComposition || preparedRanges)) return reject('SourceObservationProfile');
    const fontUrl = preparedRanges ? new URL(preparedFontReference, document.baseURI).href : null;
    const sourcePreload = preparedRanges ? links.find(link => link.rel === 'preload') : null;
    if (preparedRanges && !preparedRangePreload(sourcePreload, parsed.head, fontUrl)) return reject('SourceResources');
    const stylesheets = links.filter(link => link !== sourcePreload);
    const helperUrl = new URL(helperReference, document.baseURI).href;
    const styleUrl = new URL(styleReference, document.baseURI).href;
    const scriptProfile = rootContextSourceScriptProfile(parsed, helperUrl);
    if (!scriptProfile || stylesheets.length !== 1 || stylesheets[0].rel !== 'stylesheet' ||
        Array.from(stylesheets[0].attributes).some(attribute => !['rel', 'type', 'href'].includes(attribute.name)) ||
        (stylesheets[0].hasAttribute('type') && stylesheets[0].getAttribute('type') !== 'text/css') ||
        stylesheets[0].parentElement !== parsed.head ||
        new URL(stylesheets[0].getAttribute('href') || '', document.baseURI).href !== styleUrl) return reject('SourceResources');
    const sourceRoot = parsed.getElementById('test-root');
    if (sourceRoot.parentElement !== parsed.body) return reject('SourceParent');
    if (!rootContextBodySequence(parsed.body, sourceRoot)) return reject('SourceBodySiblings');
    if (document.body.innerHTML !== parsed.body.innerHTML) return reject('SourceBodyChanged');
    const expectedSheet = new CSSStyleSheet();
    expectedSheet.replaceSync(baseStyle);
    const rules = Array.from(expectedSheet.cssRules, rule => rule.cssText);
    const sameProof = previous?.ownedStyle && previous.source === source &&
      previous.baseStyle === baseStyle && previous.helperUrl === helperUrl &&
      previous.styleUrl === styleUrl && previous.fontUrl === fontUrl && previous.flexComposition === flexComposition;
    const scriptOwner = sameProof ? previous.scriptOwner :
      scriptProfile.kind === 'helper' ? document.scripts[0] : null;
    if (!rootContextScriptInventory(scriptProfile, scriptOwner)) return reject('ScriptChanged');
    let ownedStyle;
    let bodyChildren;
    let preload;
    if (sameProof) {
      // Repeated setup retains the same authenticated owner; a marker alone
      // never authorizes a replacement sheet or repairs a changed document.
      ownedStyle = previous.ownedStyle;
      bodyChildren = previous.bodyChildren;
      preload = previous.preload;
      if (!bodyChildren || bodyChildren.length !== document.body.childNodes.length ||
          bodyChildren.some((node, index) => document.body.childNodes[index] !== node)) return reject('SourceBodyChanged');
      if (!ownedStyle.isConnected || ownedStyle.parentElement !== document.head ||
          ownedStyle.textContent !== baseStyle ||
          !rootContextStyleInventory(ownedStyle, preload, sourcePreload?.outerHTML) ||
          document.styleSheets.length !== 1 ||
          document.styleSheets[0].ownerNode !== ownedStyle ||
          document.adoptedStyleSheets.length !== 0) return reject('StylesheetChanged');
    } else {
      bodyChildren = Array.from(document.body.childNodes);
      const liveLinks = Array.from(document.querySelectorAll('link'));
      preload = preparedRanges ? liveLinks.find(link => link.rel === 'preload') : null;
      if (preparedRanges && (!preparedRangePreload(preload, document.head, fontUrl) ||
          preload.outerHTML !== sourcePreload.outerHTML)) return reject('SourceResources');
      const liveStylesheets = liveLinks.filter(link => link !== preload);
      if (liveStylesheets.length !== 1 || liveStylesheets[0].parentElement !== document.head ||
          liveStylesheets[0].outerHTML !== stylesheets[0].outerHTML || liveStylesheets[0].disabled ||
          document.querySelectorAll('style').length !== 0 ||
          document.styleSheets.length !== 1 ||
          document.styleSheets[0].ownerNode !== liveStylesheets[0] ||
          document.styleSheets[0].disabled || document.adoptedStyleSheets.length !== 0) return reject('StylesheetSource');
      // The file:// LINK's cssRules may be opaque. Replace only the exact
      // source-authenticated owned resource, in its original cascade position.
      // Its bytes (including the data-URI font) remain the captured input.
      ownedStyle = document.createElement('style');
      ownedStyle.setAttribute('data-surgeist-root-source', 'true');
      ownedStyle.textContent = baseStyle;
      liveStylesheets[0].replaceWith(ownedStyle);
    }
    // Only this exact base sheet supplies the DIV default. Restoring position,
    // float and clear must not use the live absolute box's blockified display.
    const hypotheticalDisplay = sourceRoot.style.getPropertyValue('all') ? null :
      sourceRoot.style.display || (sourceRoot.tagName === 'DIV' ? 'flex' : null);
    rootContextCapture = { source, baseStyle, ownedStyle, hypotheticalDisplay,
      sourceRoot: sourceRoot.outerHTML, html: parsed.documentElement,
      body: parsed.body, bodyChildren, rules, helperUrl, styleUrl, fontUrl, preload,
      preloadSource: sourcePreload?.outerHTML, preparedRanges, flexComposition,
      scriptProfile, scriptOwner };
  } catch (_) {
    reject('SourceQualificationUnavailable');
  }
}

async function prepareFlexCompositionImage() {
  const proof = rootContextCapture;
  if (!proof?.flexComposition || proof.reason) return;
  const image = document.getElementById('test-root')?.children[1];
  if (image?.tagName !== 'IMG') {
    proof.reason = 'FlexCompositionImageSource';
    return;
  }
  try {
    await image.decode();
    proof.compositionImage = image;
  } catch (_) {
    proof.reason = 'FlexCompositionImageDecode';
  }
}

function qualifyFlexCompositionSubtree(root, proof) {
  const [opposite, image, collapsed, scroller] = root.children;
  const style = getComputedStyle(root);
  if (root.tagName !== 'DIV' || root.children.length !== 4 ||
      style.display !== 'flex' || style.writingMode !== 'sideways-rl' ||
      root.style.width !== '120px' || root.style.height !== '180px' ||
      [style.paddingTop, style.paddingRight, style.paddingBottom, style.paddingLeft,
        style.borderTopWidth, style.borderRightWidth, style.borderBottomWidth, style.borderLeftWidth]
        .some(value => value !== '0px') ||
      [opposite, collapsed, scroller].some(node => node?.tagName !== 'DIV') ||
      opposite.children.length !== 0 ||
      [collapsed, scroller].some(node => node.children.length !== 1 ||
        node.firstElementChild.tagName !== 'DIV' || node.firstElementChild.children.length !== 0) ||
      getComputedStyle(opposite).writingMode !== 'vertical-lr') {
    throw new Error('Root context: FlexCompositionSourceShape');
  }
  if (image !== proof.compositionImage || image?.tagName !== 'IMG' || image.children.length !== 0 ||
      !image.getAttribute('src')?.startsWith('data:image/svg+xml,') || image.currentSrc !== image.src ||
      !image.complete || image.naturalWidth !== 80 || image.naturalHeight !== 40 ||
      image.style.width !== '40px' || image.style.height !== '80px') {
    throw new Error('Root context: FlexCompositionImageAuthority');
  }
  const elements = [root, ...root.querySelectorAll('*')];
  if (elements.some(node =>
      Array.from(node.childNodes).some(child => child.nodeType === Node.TEXT_NODE && child.textContent.trim()) ||
      ['::before', '::after'].some(pseudo => !['normal', 'none'].includes(getComputedStyle(node, pseudo).content)))) {
    throw new Error('Root context: FlexCompositionAdditionalContent');
  }
  return image;
}

function captureRootContext(root) {
  const fail = reason => { throw new Error('Root context: ' + reason); };
  const proof = rootContextCapture;
  if (!proof || proof.reason) fail(proof?.reason || 'MissingSourceAuthority');
  const html = document.documentElement, body = document.body;
  if (document.compatMode !== 'CSS1Compat' || window.top !== window ||
      document.scrollingElement !== html || root.parentElement !== body ||
      document.querySelectorAll('#test-root').length !== 1 ||
      root.outerHTML !== proof.sourceRoot) fail('DocumentFrameChanged');
  for (const [live, original] of [[html, proof.html], [body, proof.body]]) {
    // Body class is the generator-owned four-variant switch. Everything else
    // affecting the ancestor source must remain the captured original.
    const attrs = element => Array.from(element.attributes)
      .filter(attribute => live === body ? attribute.name !== 'class' : true)
      .map(attribute => [attribute.name, attribute.value]).sort();
    if (JSON.stringify(attrs(live)) !== JSON.stringify(attrs(original))) fail('AncestorSourceChanged');
  }
  if (!rootContextBodySequence(body, root) || body.innerHTML !== proof.body.innerHTML ||
      proof.bodyChildren.length !== body.childNodes.length ||
      proof.bodyChildren.some((node, index) => body.childNodes[index] !== node)) fail('BodySiblings');
  if (!rootContextScriptInventory(proof.scriptProfile, proof.scriptOwner)) fail('ScriptChanged');
  const sheets = Array.from(document.styleSheets);
  if (document.adoptedStyleSheets.length !== 0) fail('AdoptedStylesheet');
  if (sheets.length !== 1 ||
      !rootContextStyleInventory(proof.ownedStyle, proof.preload, proof.preloadSource)) fail('StylesheetInventory');
  if (proof.preparedRanges) qualifyLoadedPreparedRangeFace();
  for (const sheet of sheets) {
    const owner = sheet.ownerNode;
    if (owner !== proof.ownedStyle || owner?.tagName !== 'STYLE' ||
        owner.parentElement !== document.head ||
        owner.getAttribute('data-surgeist-root-source') !== 'true' ||
        owner.textContent !== proof.baseStyle || sheet.disabled) fail('StylesheetSource');
    let rules;
    try { rules = Array.from(sheet.cssRules, rule => rule.cssText); }
    catch (_) { fail('OpaqueStylesheet'); }
    if (JSON.stringify(rules) !== JSON.stringify(proof.rules)) fail('StylesheetChanged');
  }
  const zero = value => /^[-+]?0(?:\.0+)?px$/.test(value);
  const marginProperties = ['marginTop', 'marginRight', 'marginBottom', 'marginLeft'];
  const paddingBorderProperties = ['paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft',
    'borderTopWidth', 'borderRightWidth', 'borderBottomWidth', 'borderLeftWidth'];
  const edgeProperties = [...marginProperties, ...paddingBorderProperties];
  const unshifted = style => style.transform === 'none' && style.perspective === 'none' &&
    style.filter === 'none' && style.contain === 'none' && style.willChange === 'auto' &&
    ['normal', '1'].includes(style.zoom) && style.contentVisibility === 'visible' &&
    style.animationName === 'none' && style.transitionProperty === 'all' &&
    style.transitionDuration.split(',').every(value => parseFloat(value) === 0);
  const parentStyle = getComputedStyle(body), htmlStyle = getComputedStyle(html);
  for (const style of [parentStyle, htmlStyle]) {
    if (style.writingMode !== 'horizontal-tb' || style.display !== 'block' ||
        style.position !== 'static' || style.cssFloat !== 'none' || style.clear !== 'none' ||
        style.overflowX !== 'visible' || style.overflowY !== 'visible' ||
        style.scrollbarGutter !== 'auto' || !unshifted(style) ||
        edgeProperties.some(property => !zero(style[property]))) fail('ContainingFrame');
  }
  // The only admitted body declaration is a separately qualified literal indent.
  // Neither it nor a fixed spacer establishes a body block-size percentage basis.
  if (rootContextBodyIndent(body) === undefined || html.style.length) fail('AncestorInlineStyleChanged');
  if (![parentStyle.direction, htmlStyle.direction].every(value => ['ltr', 'rtl'].includes(value))) fail('ContainingDirection');
  if (window.scrollX !== 0 || window.scrollY !== 0) fail('ScrolledFrame');
  const visual = window.visualViewport;
  if (!visual || visual.scale !== 1 || visual.offsetLeft !== 0 || visual.offsetTop !== 0) fail('ViewportScale');
  const width = html.clientWidth, height = html.clientHeight;
  if (![width, height].every(value => Number.isFinite(value) && value >= 0) ||
      width !== visual.width || height !== visual.height ||
      body.clientWidth !== width) fail('ViewportExtentAuthority');
  const hasSpacers = Array.from(body.children).some(node => node !== root);
  for (const spacer of body.children) {
    if (spacer === root) continue;
    const spacerStyle = getComputedStyle(spacer);
    if (spacerStyle.display !== 'block' || spacerStyle.position !== 'static' ||
        spacerStyle.writingMode !== 'horizontal-tb' || spacerStyle.cssFloat !== 'none' ||
        spacerStyle.clear !== 'none' || !unshifted(spacerStyle) ||
        spacerStyle.clipPath !== 'none' || !['normal', 'none'].includes(spacerStyle.content) ||
        edgeProperties.some(property => !zero(spacerStyle[property]))) fail('BodySiblings');
  }
  const style = getComputedStyle(root);
  if (hasSpacers && style.position !== 'absolute') fail('SourceBodySiblings');
  if (body.style.length && style.position !== 'absolute') fail('AuthoredBodyOrHtmlStyle');
  if (root.tagName !== 'DIV' || !['normal', 'none'].includes(style.content)) fail('RootBoxRole');
  if (marginProperties.some(property => root.style[property] && !/^(?:0|[-+]?0(?:\.0+)?px)$/.test(root.style[property]))) fail('AuthoredRootEdges');
  if (!unshifted(style) || style.cssFloat !== 'none' || style.clear !== 'none' ||
      marginProperties.some(property => !zero(style[property]))) fail('RootEdgesOrDisplacement');
  const authoredPaddingBorder = paddingBorderProperties.some(property =>
    root.style[property] && !/^(?:0|[-+]?0(?:\.0+)?px)$/.test(root.style[property]));
  const usedPaddingBorder = paddingBorderProperties.some(property => !zero(style[property]));
  // The in-flow certificate still promises the initial normal-flow origin.
  // Only the separate absolute-local capability can admit nonzero box edges.
  if (style.position !== 'absolute') {
    if (authoredPaddingBorder) fail('AuthoredRootEdges');
    if (usedPaddingBorder) fail('RootEdgesOrDisplacement');
  }
  const physicalInsets = ['top', 'right', 'bottom', 'left'];
  const logicalInsets = ['inset-block-start', 'inset-block-end', 'inset-inline-start', 'inset-inline-end'];
  if (physicalInsets.some(property => root.style[property] && root.style[property] !== 'auto') ||
      logicalInsets.some(property => root.style.getPropertyValue(property) &&
        root.style.getPropertyValue(property) !== 'auto')) fail('AuthoredInsets');
  if (!['block', 'flow-root', 'flex', 'grid', 'grid-lanes'].includes(style.display)) fail('RootDisplay');
  const nonCollapsingBoundary = ['flow-root', 'flex', 'grid', 'grid-lanes'].includes(style.display) ||
    (style.display === 'block' && [style.overflowX, style.overflowY].some(value => ['hidden', 'auto', 'scroll'].includes(value)));
  const fallback = { initialContainingBlock: { width, height }, nearestScrollport: 'none',
    containingLimits: { width: 'unbounded', height: 'unbounded' } };
  if (style.position === 'absolute') {
    if ((style.writingMode !== 'horizontal-tb' && !proof.flexComposition) ||
        style.aspectRatio !== 'auto' || style.alignSelf !== 'auto' ||
        style.justifySelf !== 'auto' || ['IMG', 'VIDEO', 'CANVAS', 'IFRAME', 'OBJECT', 'EMBED', 'INPUT'].includes(root.tagName)) {
      fail('PositionedRootProfile');
    }
    if (authoredPaddingBorder || usedPaddingBorder) {
      // Reuse the descriptor's physical/logical preferred-size projection.
      // Computed used width and the observed root rectangle are not proof of a
      // fixed own preferred size. The full authenticated CB, not W-s, supplies
      // a percentage width's basis. General calc/intrinsic/Auto edge-sensitive
      // shrink-to-fit has not been qualified by this source profile.
      const preferred = parseElementSize(property => {
        const input = root.style[property];
        return requireSizingProjection(input, parseSizeDimension(input));
      }, style)?.width;
      const definite = preferred?.unit === 'px' ? preferred.value :
        preferred?.unit === 'percent' ? preferred.value * width : undefined;
      if (!Number.isFinite(definite) || definite < 0) fail('AuthoredRootEdges');
    }
    const hypotheticalDisplay = proof.hypotheticalDisplay;
    const blockified = { 'inline-flex': 'flex', 'inline-grid': 'grid' };
    if (!['block', 'flow-root', 'flex', 'inline-flex', 'grid', 'inline-grid', 'grid-lanes'].includes(hypotheticalDisplay) ||
        style.display !== (blockified[hypotheticalDisplay] || hypotheticalDisplay)) fail('HypotheticalDisplaySource');
    // A qualified spacer changes only the omitted common block translation.
    // No font/strut or used root/body rectangle supplies a static-position operand.
    if (parentStyle.whiteSpace !== 'normal' || parentStyle.textAlign !== 'start' ||
        ![parentStyle.unicodeBidi, style.unicodeBidi].every(value => ['normal', 'isolate'].includes(value))) fail('HypotheticalInlineStartUnavailable');
    // Actual absolute positioning supplies the root's independent formatting
    // context even for visible-overflow Block. Local observation does not
    // promise its hypothetical block origin; complete subtree independence
    // below remains mandatory before that translation can be omitted.
    const image = proof.flexComposition ? qualifyFlexCompositionSubtree(root, proof) : null;
    qualifyRootLocalSubtree(root, unshifted, proof.preparedRanges, image);
    let hypotheticalInlineOffset = 0;
    if (body.style.length) {
      hypotheticalInlineOffset = qualifyRootLiteralIndent(root, proof, parentStyle, htmlStyle, width);
    } else if (!zero(parentStyle.textIndent)) {
      fail('HypotheticalInlineStartUnavailable');
    }
    return { kind: 'absolute-root-local', containingWidth: width,
      containingHeight: height, containingDirection: htmlStyle.direction,
      hypotheticalDirection: parentStyle.direction, hypotheticalDisplay,
      hypotheticalAnchor: 'inline-start', hypotheticalInlineOffset, fallback };
  }
  if (style.position === 'static' || style.position === 'relative') {
    if (!nonCollapsingBoundary) fail('UnqualifiedInitialBlockOrigin');
    return { kind: 'block-flow-auto', parentInlineExtent: width,
      parentDirection: parentStyle.direction, fallback };
  }
  fail('UnsupportedPosition');
}


// This is a closed source/capture capability, not a CSS cascade or an assumed
// zero block origin. Every accepted producer must commute with a common root
// translation before publication rounding. The exact base/inline declarations
// and complete source/live subtree equality are authenticated above.
function qualifyRootLocalSubtree(root, unshifted, preparedRanges, compositionImage) {
  const fail = reason => { throw new Error('Root context: ' + reason); };
  const blockProperties = ['height', 'min-height', 'max-height',
    'block-size', 'min-block-size', 'max-block-size'];
  for (const property of blockProperties) {
    const value = root.style.getPropertyValue(property);
    if (/(?:^|[^a-z-])(?:stretch|fit-content|(?:-webkit-)?fill-available|available|contain)(?:$|[^a-z-])/i.test(value)) fail('RootLocalBlockSizeDependency');
  }
  const logicalBlockEdges = ['padding-block-start', 'padding-block-end',
    'inset-block-start', 'inset-block-end'];
  const inline = preparedRanges ? qualifyPreparedRangeSubtree(root) : null;
  const pending = [root];
  const localBoxes = [];
  let fixedOrthogonal = false;
  while (pending.length) {
    const node = pending.pop();
    const style = getComputedStyle(node);
    localBoxes.push([node, style]);
    if (logicalBlockEdges.some(property => styleDeclarationValue(node.style, property))) {
      fail('UnrepresentedLogicalBlockEdges');
    }
    if (node !== root && !['static', 'relative', 'absolute'].includes(style.position)) fail('RootLocalExternalFrame');
    if (!unshifted(style) || style.clipPath !== 'none' ||
        !['normal', 'none'].includes(style.content)) fail('RootLocalExternalFrame');
    const expectedMode = compositionImage
      ? (node === root.firstElementChild ? 'vertical-lr' : 'sideways-rl')
      : 'horizontal-tb';
    if (style.writingMode !== expectedMode) {
      if (node === root || preparedRanges || compositionImage ||
          !['vertical-rl', 'vertical-lr', 'sideways-rl', 'sideways-lr'].includes(style.writingMode)) {
        fail('RootLocalWritingMode');
      }
      // Source/live subtree equality authenticates these inline declarations.
      // A computed or measured finite size cannot supply missing source input.
      const fixed = property => {
        const value = styleDeclarationValue(node.style, property);
        return /^(?:\d+(?:\.\d*)?|\.\d+)px$/.test(value) &&
          Number.isFinite(Number(value.slice(0, -2))) && Number(value.slice(0, -2)) >= 0;
      };
      if (!fixed('width') || !fixed('height')) fail('RootLocalOrthogonalPhysicalSize');
      fixedOrthogonal = true;
    }
    // These elements have no border-box observation that can share the root's
    // omitted translation. A world-zero rectangle is not a local zero box.
    if (['none', 'contents'].includes(style.display)) fail('RootLocalBoxlessObservation');
    if (node !== inline && node !== compositionImage && (!['DIV', 'SPAN'].includes(node.tagName) ||
        !['block', 'flow-root', 'flex', 'grid', 'grid-lanes'].includes(style.display))) fail('RootLocalBoxProfile');
    // Unlike variant box-sizing/direction rules, .viewport supplies dimensions
    // absent from this inline-declaration-only normalized sizing channel.
    if (node === compositionImage && style.display !== 'block') fail('RootLocalBoxProfile');
    if (node.classList.contains('viewport')) fail('RootLocalUnrepresentedClass');
    const allowedMarkers = node === inline ? ['data-surgeist-transparent-inline-container'] :
      preparedRanges && node === root ? ['data-surgeist-layout-ready-inline', 'data-surgeist-prepared-text'] : [];
    if (Array.from(node.attributes).some(({name}) => name.startsWith('data-surgeist-') &&
        !allowedMarkers.includes(name))) fail('RootLocalObservationChannel');
    if (node.scrollLeft !== 0 || node.scrollTop !== 0) fail('RootLocalScrolledBox');
    for (const property of Array.from(node.style)) {
      const value = node.style.getPropertyValue(property);
      if (property === 'all' || property.startsWith('anchor-') || property.startsWith('position-') ||
          property === 'container-type' || property === 'container-name' ||
          /(?:anchor(?:-size)?|env)\s*\(/i.test(value) ||
          /(?:^|[^a-z-])(?:[sld]?v[whib]|[sld]?vmin|[sld]?vmax)(?:$|[^a-z-])/i.test(value)) fail('RootLocalExternalSize');
    }
    // Absolute descendants have a containing box inside this actual absolute
    // root; local hidden/scroll clipping remains in that same moving subtree.
    // Fixed/sticky, transformed, viewport-dependent and typed inline producers
    // have deliberately not been qualified by this first capability.
    pending.push(...node.children);
  }
  if (fixedOrthogonal) qualifyRootLocalFixedPhysicalScope(root, localBoxes);
}

function qualifyRootLocalFixedPhysicalScope(root, boxes) {
  const fail = reason => { throw new Error('Root context: ' + reason); };
  const properties = new Set(['display', 'writing-mode', 'direction', 'width', 'height',
    'grid-auto-flow', 'grid-template-columns', 'grid-template-rows',
    'grid-row', 'grid-row-start', 'grid-row-end',
    'grid-column', 'grid-column-start', 'grid-column-end']);
  const edges = ['marginTop', 'marginRight', 'marginBottom', 'marginLeft',
    'paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft',
    'borderTopWidth', 'borderRightWidth', 'borderBottomWidth', 'borderLeftWidth'];
  for (const [node, style] of boxes) {
    // Every local producer shares the root translation. This source profile
    // admits no intervening positioned, clipped or scrolling dependency.
    // Auto insets follow from the authenticated base and absence of source
    // overrides below, not CSSOM's resolved absolute-position coordinates.
    if ((node !== root && !['static', 'relative'].includes(style.position)) ||
        style.overflowX !== 'visible' || style.overflowY !== 'visible' ||
        style.translate !== 'none' || style.rotate !== 'none' || style.scale !== 'none' || style.offsetPath !== 'none' ||
        style.aspectRatio !== 'auto' || edges.some(property => !/^[-+]?0(?:\.0+)?px$/.test(style[property]))) {
      fail('RootLocalOrthogonalFrame');
    }
    if (style.whiteSpace !== 'normal') fail('RootLocalOrthogonalTextProducer');
    for (const property of Array.from(node.style)) {
      // Only represented source inputs participate in this profile. Other
      // declarations require their own proof rather than a property blacklist.
      if (!properties.has(property)) fail('RootLocalOrthogonalSourceOverride');
    }
    if (Array.from(node.childNodes).some(child => child.nodeType === Node.TEXT_NODE && /[^\t\n\f\r ]/.test(child.textContent))) {
      fail('RootLocalOrthogonalTextProducer');
    }
  }
}

function qualifyPreparedRangeSubtree(root) {
  const text = node => node?.nodeType === Node.TEXT_NODE && node.length > 0;
  const [first, inline, last] = root.childNodes;
  const rootStyle = getComputedStyle(root);
  if (rootStyle.display !== 'block' || root.getAttribute('data-surgeist-layout-ready-inline') !== 'true' ||
      root.getAttribute('data-surgeist-prepared-text') !== 'ahem-ttf-v1' || root.childNodes.length !== 3 ||
      !text(first) || inline?.tagName !== 'SPAN' || !text(last) || inline.childNodes.length !== 1 ||
      !text(inline.firstChild) || inline.getAttribute('data-surgeist-transparent-inline-container') !== 'true') {
    throw new Error('Root context: PreparedRangeSourceShape');
  }
  const inlineStyle = getComputedStyle(inline);
  if (inlineStyle.display !== 'inline' || inlineStyle.direction !== rootStyle.direction ||
      rootStyle.fontSizeAdjust !== 'none' || inlineStyle.fontSizeAdjust !== 'none' ||
      [root, inline].some(node => ['::before', '::after'].some(pseudo =>
        !['normal', 'none'].includes(getComputedStyle(node, pseudo).content)))) {
    throw new Error('Root context: PreparedRangeInlineFrame');
  }
  qualifyPreparedFrame(inline, rootStyle);
  qualifyTransparentPreparedInline(inline, root);
  return inline;
}

function qualifyAnalyticCaptureText(node, containingMode, inheritedOrthogonal = false) {
  if (node.preparedParagraph) return;
  const mode = node.style?.writingMode || containingMode;
  const orthogonal = inheritedOrthogonal ||
    ((mode === 'horizontal-tb') !== (containingMode === 'horizontal-tb'));
  if (orthogonal && typeof node.textContent === 'string' && node.textContent.length) {
    throw new Error('Root context: UnsupportedAnalyticOrthogonalTextProfile');
  }
  for (const child of node.children || []) qualifyAnalyticCaptureText(child, mode, orthogonal);
}

function captureRootVariant(root, className) {
  document.body.className = className;
  const previousObservation = rootLocalObservation;
  try {
    const before = captureRootContext(root);
    const fixedFrame = rootContextCapture.preparedRanges || rootContextCapture.flexComposition;
    if (rootContextCapture.flexComposition && before.kind !== 'absolute-root-local') {
      throw new Error('Root context: FlexCompositionRequiresAbsoluteRoot');
    }
    if (rootContextCapture.preparedRanges && before.kind !== 'absolute-root-local') {
      throw new Error('Root context: PreparedRangeRequiresAbsoluteRoot');
    }
    if (before.kind === 'absolute-root-local') {
      const rectangle = root.getBoundingClientRect();
      // These coordinates are observation operands only, never source/static
      // position evidence or inputs to the root sizing context above.
      if (fixedFrame &&
          ![rectangle.left, rectangle.top, rectangle.width, rectangle.height].every(Number.isFinite)) {
        throw new Error(rootContextCapture.preparedRanges ?
          'Root context: NonfinitePreparedRangeFrame' : 'Root context: NonfiniteFlexCompositionFrame');
      }
      rootLocalObservation = { root, x: rectangle.left, y: rectangle.top,
        stableRect: fixedFrame ? rectangle : undefined };
    } else {
      rootLocalObservation = undefined;
    }
    const data = describeElement(root);
    // Base-CSS positioning is real normalized input, not an omitted authored value.
    data.style.position = getComputedStyle(root).position;
    qualifyAnalyticCaptureText(data, getComputedStyle(document.body).writingMode);
    const after = captureRootContext(root);
    if (JSON.stringify(before) !== JSON.stringify(after)) throw new Error('Root context: UnstableContainingFrame');
    const stableRect = rootLocalObservation?.stableRect;
    if (stableRect) {
      const afterRect = root.getBoundingClientRect();
      if (['left', 'top', 'width', 'height'].some(key => stableRect[key] !== afterRect[key])) {
        throw new Error(rootContextCapture.preparedRanges ?
          'Root context: UnstablePreparedRangeFrame' : 'Root context: UnstableFlexCompositionFrame');
      }
      if (!data.unsupportedReason && rootContextCapture.preparedRanges) {
        data.preparedRangeFrame = PREPARED_RANGE_FRAME;
      }
      if (!data.unsupportedReason && rootContextCapture.flexComposition) {
        const {naturalWidth, naturalHeight} = rootContextCapture.compositionImage;
        data.flexCompositionImage = {naturalWidth, naturalHeight};
      }
    }
    data.viewport = { rootContext: before };
    return data;
  } catch (error) {
    return unsupportedTestData(error.message);
  } finally {
    rootLocalObservation = previousObservation;
  }
}


function parseRepetition(input) {
  if (input === "auto-fill") return { unit: 'auto-fill' };
  if (input === "auto-fit") return { unit: 'auto-fit' };
  if (/^[0-9]*$/.test(input)) return { 'unit': 'integer', value: parseInt(input, 10) };
  return undefined;
}

function parseDimension(input, options = { allowFrUnits: false }) {
  if (!input) return undefined;
  if (typeof input === 'object') return parseTypedOmDimension(input) || input;
  const calc = parseCalcDimension(input);
  if (calc) return calc;
  if (options.allowFrUnits && input.endsWith('fr')) return { unit: 'fraction', value: parseFloat(input.replace('fr', '')) };
  if (input.endsWith('px')) return { unit: 'px', value: parseFloat(input.replace('px', '')) };
  if (input.endsWith('%')) return { unit: 'percent', value: parseFloat(input.replace('%', '')) / 100 };
  if (input === 'auto') return { unit: 'auto' };
  if (input === 'min-content') return { unit: 'min-content' };
  if (input === 'max-content') return { unit: 'max-content' };
  return undefined;
}

const MAX_OWNED_SIZING_FUNCTION_DEPTH = 64;
const OWNED_SIZING_KEYWORDS = new Set([
  'auto',
  'none',
  'content',
  'min-content',
  'max-content',
  'stretch',
  'fit-content',
  'contain',
]);
const OWNED_CALC_SIZE_BASES = new Set([
  'any',
  '100%',
  'auto',
  'none',
  'content',
  'min-content',
  'max-content',
  'stretch',
  'fit-content',
  'contain',
]);
const OWNED_COMPLETE_CSS_NUMBER = /^[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?$/;

function parseSizingDimension(input, options = { allowFrUnits: false }) {
  if (!input) return undefined;
  if (typeof input === 'object') {
    return canonicalSizingPercentage(input) || parseTypedOmSizingDimension(input, options);
  }

  const value = input.trim();
  if (!ownedSizingTokenIsValid(value, options.allowFrUnits)) return undefined;

  if (value.endsWith('px')) return { unit: 'px', value: Number(value.slice(0, -2)) };
  if (value.endsWith('%') && !value.includes('(')) {
    return { unit: 'percent', value: Number(value.slice(0, -1)) / 100 };
  }
  if (options.allowFrUnits && value.endsWith('fr')) {
    return { unit: 'fraction', value: Number(value.slice(0, -2)) };
  }
  if (OWNED_SIZING_KEYWORDS.has(value)) return { unit: value };
  if (value.startsWith('calc(')) return { unit: 'calc', value };
  return { unit: 'sizing', value };
}

function canonicalSizingPercentage(value) {
  const prototype = Object.getPrototypeOf(value);
  const isPlainObject = prototype === Object.prototype || prototype === null;
  if (!isPlainObject || value.unit !== 'percent' || !Number.isFinite(value.value)) {
    return undefined;
  }
  return value;
}

function parseTypedOmSizingDimension(value, options) {
  if (!value) return undefined;
  if (value.unit === 'percent' && Number.isFinite(value.value)) {
    return { unit: 'percent', value: value.value / 100 };
  }
  if (value.unit === 'px' && Number.isFinite(value.value)) {
    return { unit: 'px', value: value.value };
  }
  if (value.unit === 'fr' && options.allowFrUnits && Number.isFinite(value.value) && value.value >= 0) {
    return { unit: 'fraction', value: value.value };
  }
  if ((value.unit === 'calc' || value.unit === 'sizing') && typeof value.value === 'string') {
    return parseSizingDimension(value.value, options);
  }
  return parseSizingDimension(value.toString ? value.toString() : '', options);
}

function ownedSizingTokenIsValid(value, allowFrUnits) {
  if (!value) return false;
  if (ownedLengthPercentageIsValid(value)) return true;
  if (OWNED_SIZING_KEYWORDS.has(value)) return true;
  if (allowFrUnits && ownedTrackFlexIsValid(value)) return true;

  const sizingFunction = parseOwnedSizingFunction(value);
  if (!sizingFunction) return false;
  if (sizingFunction.name === 'fit-content') {
    const arguments = splitOwnedSizingArguments(sizingFunction.body);
    return arguments?.length === 1 && ownedSizingCalculationIsValid(arguments[0]);
  }
  if (sizingFunction.name === 'calc-size') {
    const arguments = splitOwnedSizingArguments(sizingFunction.body);
    if (arguments?.length !== 2 || !OWNED_CALC_SIZE_BASES.has(arguments[0])) return false;
    const calculation = ownedCalcSizeCalculation(arguments[1]);
    return calculation.valid && !(arguments[0] === 'any' && calculation.usesSize);
  }
  return ownedSizingCalculationIsValid(value);
}

function ownedLengthPercentageIsValid(value) {
  if (value.endsWith('px')) return ownedFiniteCssNumber(value.slice(0, -2));
  if (value.endsWith('%')) return ownedFiniteCssNumber(value.slice(0, -1));
  return false;
}

function ownedTrackFlexIsValid(value) {
  if (!value.endsWith('fr') || !ownedFiniteCssNumber(value.slice(0, -2))) return false;
  return Number(value.slice(0, -2)) >= 0;
}

function ownedFiniteCssNumber(value) {
  return ownedFixtureNumber(value) !== undefined;
}

function ownedFixtureNumber(value) {
  if (!OWNED_COMPLETE_CSS_NUMBER.test(value)) return undefined;
  const number = Math.fround(Number(value));
  return Number.isFinite(number) ? number : undefined;
}

function parseOwnedSizingFunction(value) {
  const openIndex = value.indexOf('(');
  if (openIndex <= 0 || !/^[a-z-]+$/.test(value.slice(0, openIndex))) return undefined;

  let depth = 0;
  for (let index = openIndex; index < value.length; index++) {
    const char = value[index];
    if (char === '[' || char === ']') return undefined;
    if (char === '(') {
      depth++;
      if (depth > MAX_OWNED_SIZING_FUNCTION_DEPTH) return undefined;
      continue;
    }
    if (char !== ')') continue;
    depth--;
    if (depth < 0) return undefined;
    if (depth === 0) {
      if (index + 1 !== value.length) return undefined;
      return {
        name: value.slice(0, openIndex),
        body: value.slice(openIndex + 1, index),
      };
    }
  }
  return undefined;
}

function splitOwnedSizingArguments(body) {
  const arguments = [];
  let depth = 0;
  let start = 0;
  for (let index = 0; index < body.length; index++) {
    const char = body[index];
    if (char === '(') depth++;
    if (char === ')') {
      depth--;
      if (depth < 0) return undefined;
    }
    if (char === ',' && depth === 0) {
      arguments.push(body.slice(start, index).trim());
      start = index + 1;
    }
  }
  if (depth !== 0) return undefined;
  arguments.push(body.slice(start).trim());
  return arguments.some(argument => !argument) ? undefined : arguments;
}

function ownedSizingCalculationIsValid(value) {
  if (ownedLengthPercentageIsValid(value)) return true;
  const sizingFunction = parseOwnedSizingFunction(value);
  if (!sizingFunction) return false;

  if (sizingFunction.name === 'calc') {
    return ownedAffineCalculation(sizingFunction.body, false, false).valid;
  }
  const arguments = splitOwnedSizingArguments(sizingFunction.body);
  if (!arguments) return false;
  if (sizingFunction.name === 'min' || sizingFunction.name === 'max') {
    return arguments.every(ownedSizingCalculationIsValid);
  }
  if (sizingFunction.name === 'clamp') {
    return arguments.length === 3
      && arguments[1] !== 'none'
      && (arguments[0] === 'none' || ownedSizingCalculationIsValid(arguments[0]))
      && ownedSizingCalculationIsValid(arguments[1])
      && (arguments[2] === 'none' || ownedSizingCalculationIsValid(arguments[2]));
  }
  return false;
}

function ownedCalcSizeCalculation(value) {
  const affine = ownedAffineCalculation(value, true, true);
  if (affine.valid) return affine;

  const sizingFunction = parseOwnedSizingFunction(value);
  if (!sizingFunction) return { valid: false, usesSize: false };
  if (sizingFunction.name === 'calc') {
    return ownedAffineCalculation(sizingFunction.body, true, false);
  }
  const arguments = splitOwnedSizingArguments(sizingFunction.body);
  if (!arguments) return { valid: false, usesSize: false };
  if (sizingFunction.name === 'min' || sizingFunction.name === 'max') {
    const calculations = arguments.map(ownedCalcSizeCalculation);
    return {
      valid: calculations.every(calculation => calculation.valid),
      usesSize: calculations.some(calculation => calculation.usesSize),
    };
  }
  if (sizingFunction.name === 'clamp' && arguments.length === 3 && arguments[1] !== 'none') {
    const calculations = arguments.map((argument, index) => {
      if (argument === 'none' && index !== 1) return { valid: true, usesSize: false };
      return ownedCalcSizeCalculation(argument);
    });
    return {
      valid: calculations.every(calculation => calculation.valid),
      usesSize: calculations.some(calculation => calculation.usesSize),
    };
  }
  return { valid: false, usesSize: false };
}

function ownedAffineCalculation(value, allowSize, allowUnitless) {
  const trimmed = value.trim();
  if (!trimmed) return { valid: false, usesSize: false };

  const tokens = trimmed.split(/\s+/);
  let absolutePx = 0;
  let percentFraction = 0;
  let sizeFraction = 0;
  let usesSize = false;
  let start = 0;
  let sign = 1;

  while (start < tokens.length) {
    const operatorOffset = tokens.slice(start).findIndex(token => token === '+' || token === '-');
    const end = operatorOffset === -1 ? tokens.length : start + operatorOffset;
    if (end === start) return { valid: false, usesSize: false };

    const term = ownedAffineTerm(tokens.slice(start, end), allowSize, allowUnitless);
    if (!term.valid) return { valid: false, usesSize: false };

    absolutePx = Math.fround(absolutePx + Math.fround(term.absolutePx * sign));
    percentFraction = Math.fround(percentFraction + Math.fround(term.percentFraction * sign));
    sizeFraction = Math.fround(sizeFraction + Math.fround(term.sizeFraction * sign));
    if (![absolutePx, percentFraction, sizeFraction].every(Number.isFinite)) {
      return { valid: false, usesSize: false };
    }
    usesSize ||= term.usesSize;

    if (end === tokens.length) break;
    sign = tokens[end] === '+' ? 1 : -1;
    start = end + 1;
    if (start === tokens.length) return { valid: false, usesSize: false };
  }

  return { valid: true, usesSize };
}

function ownedAffineTerm(tokens, allowSize, allowUnitless) {
  if (tokens.length === 1) {
    return ownedAffineAtom(tokens[0], allowSize, allowUnitless);
  }
  if (tokens.length !== 3 || tokens[1] !== '*' || !allowSize) {
    return { valid: false, usesSize: false };
  }

  const factor = tokens[0] === 'size'
    ? ownedFixtureNumber(tokens[2])
    : tokens[2] === 'size'
      ? ownedFixtureNumber(tokens[0])
      : undefined;
  if (factor === undefined) return { valid: false, usesSize: false };
  return {
    valid: true,
    usesSize: true,
    absolutePx: 0,
    percentFraction: 0,
    sizeFraction: factor,
  };
}

function ownedAffineAtom(atom, allowSize, allowUnitless) {
  if (atom.endsWith('px')) {
    const absolutePx = ownedFixtureNumber(atom.slice(0, -2));
    if (absolutePx !== undefined) {
      return { valid: true, usesSize: false, absolutePx, percentFraction: 0, sizeFraction: 0 };
    }
  }
  if (atom.endsWith('%')) {
    const percent = ownedFixtureNumber(atom.slice(0, -1));
    if (percent !== undefined) {
      const percentFraction = Math.fround(percent / 100);
      if (Number.isFinite(percentFraction)) {
        return { valid: true, usesSize: false, absolutePx: 0, percentFraction, sizeFraction: 0 };
      }
    }
  }
  if (allowSize) {
    if (atom === 'size') {
      return { valid: true, usesSize: true, absolutePx: 0, percentFraction: 0, sizeFraction: 1 };
    }
    const factorText = atom.endsWith('*size')
      ? atom.slice(0, -5)
      : atom.startsWith('size*')
        ? atom.slice(5)
        : undefined;
    if (factorText !== undefined) {
      const sizeFraction = ownedFixtureNumber(factorText);
      if (sizeFraction !== undefined) {
        return { valid: true, usesSize: true, absolutePx: 0, percentFraction: 0, sizeFraction };
      }
    }
  }
  if (allowUnitless) {
    const absolutePx = ownedFixtureNumber(atom);
    if (absolutePx !== undefined) {
      return { valid: true, usesSize: false, absolutePx, percentFraction: 0, sizeFraction: 0 };
    }
  }
  return { valid: false, usesSize: false };
}

function parseTypedOmDimension(value) {
  if (!value) return undefined;
  if (value.unit === "percent") return { unit: "percent", value: value.value / 100 };
  if (value.unit === "px") return { unit: "px", value: value.value };
  if (value.unit === "fr") return { unit: "fraction", value: value.value };
  return parseCalcDimension(value.toString ? value.toString() : "");
}

function parseCalcDimension(input) {
  const value = normalizeCalcString(input);
  return value ? { unit: "calc", value } : undefined;
}

function normalizeCalcString(input) {
  if (typeof input !== "string") return "";
  const value = input.trim();
  if (!value.startsWith("calc(") || !value.endsWith(")")) return "";
  return value;
}

function containsCalcFunction(input) {
  return typeof input === "string" && input.includes("calc(");
}

function parseResolvedDimension(input, computedInput) {
  return parseSizingDimension(input) || (input ? parseSizingDimension(computedInput) : undefined);
}

// Try-parsers retain their grammar/absence contract. The actual descriptor
// consumer must not publish an omitted field for a present sizing declaration.
function requireSizingProjection(input, projected) {
  if (input && projected === undefined) {
    throw new Error('Root context: UnresolvedSizingDeclaration');
  }
  return projected;
}

function representedEdges(edges) {
  const projected = parseEdges(edges);
  for (const edge of ['left', 'right', 'top', 'bottom']) {
    requireSizingProjection(edges[edge], projected?.[edge]);
  }
  return projected;
}

function representedInlineEdges(edges, inlineStart, inlineEnd, computedStyle) {
  for (const [edge, input] of [
    [inlineStartEdge(computedStyle), inlineStart],
    [inlineEndEdge(computedStyle), inlineEnd],
  ]) {
    if (!input) continue;
    // Physical and logical declarations on the same side need cascade authority
    // that this closed descriptor does not provide. Opposite sides remain valid.
    if (edges[edge]) throw new Error('Root context: ConflictingEdgeDeclarations');
    edges[edge] = input;
  }
  return representedEdges(edges);
}

function representedGaps(styleValue) {
  const values = { gap: styleValue('gap'), rowGap: styleValue('rowGap'), columnGap: styleValue('columnGap') };
  const projected = parseGaps(property => values[property]);
  requireSizingProjection(values.gap || values.rowGap, projected?.row);
  requireSizingProjection(values.gap || values.columnGap, projected?.column);
  return projected;
}

function parseNumber(input) {
  if (input === '' || isNaN(input)) return undefined;
  return Number(input);
}

function parseCssPx(value) {
  if (!value.endsWith("px")) {
    throw new Error(`expected computed px value, got ${value}`);
  }
  return Number(value.slice(0, -2));
}

function resolveLineHeightPx(lineHeight, fontSize) {
  if (lineHeight === "normal") {
    return fontSize * 1.2;
  }
  return parseCssPx(lineHeight);
}

function estimateInlineBaselinePx(fontSize, lineHeight) {
  const fontBaseline = fontSize * 0.8;
  const leading = Math.max(0, lineHeight - fontSize);
  return leading / 2 + fontBaseline;
}

function measureInlineBaselinePx(computedStyle, lineHeight) {
  const writingMode = computedStyle.writingMode;
  const direction = computedStyle.direction;
  if (!['horizontal-tb', 'vertical-rl', 'vertical-lr', 'sideways-rl', 'sideways-lr'].includes(writingMode) ||
      !['ltr', 'rtl'].includes(direction) ||
      typeof computedStyle.font !== 'string' || computedStyle.font.length === 0) {
    throw new Error('inline BR metrics require a complete computed line context');
  }

  const probe = document.createElement('span');
  Object.assign(probe.style, {
    position: 'absolute',
    visibility: 'hidden',
    pointerEvents: 'none',
    left: '0',
    top: '0',
    width: '0',
    height: '0',
    margin: '0',
    padding: '0',
    border: '0',
    whiteSpace: 'nowrap',
    font: computedStyle.font,
    lineHeight: `${lineHeight}px`,
    writingMode,
    direction,
  });

  const lineOverMarker = document.createElement('span');
  const baselineMarker = document.createElement('span');
  for (const marker of [lineOverMarker, baselineMarker]) {
    Object.assign(marker.style, {
      display: 'inline-block',
      width: '0',
      height: '0',
      margin: '0',
      padding: '0',
      border: '0',
    });
  }
  lineOverMarker.style.verticalAlign = 'top';
  baselineMarker.style.verticalAlign = 'baseline';
  probe.append(lineOverMarker, baselineMarker);

  let lineOverRect;
  let baselineRect;
  document.body.appendChild(probe);
  try {
    lineOverRect = lineOverMarker.getBoundingClientRect();
    baselineRect = baselineMarker.getBoundingClientRect();
  } finally {
    probe.remove();
  }

  let distance;
  if (writingMode === 'horizontal-tb') {
    distance = baselineRect.top - lineOverRect.top;
  } else {
    distance = Math.abs(lineOverRect.left - baselineRect.left);
  }
  if (!Number.isFinite(distance) || distance < 0) {
    throw new Error('inline BR baseline probe requires a finite logical block distance');
  }
  return Math.min(lineHeight, distance);
}

function parseRatio(input) {
  if (!input) return undefined;

  if (input.includes('/')) {
    let [width, height] = input.split("/").map(part => parseFloat(part.trim()));
    if (!width || width < 0 || !height || height <= 0) return undefined;
    return width / height;
  }

  let ratio = parseFloat(input);
  if (!ratio || ratio < 0) return undefined;
  return ratio;
}

function parseEnum(input) {
  if (input) return input;
  return undefined;
}

function parseEdges(edges) {
  const left = parseDimension(edges.left);
  const right = parseDimension(edges.right);
  const top = parseDimension(edges.top);
  const bottom = parseDimension(edges.bottom);

  if (!left && !right && !top && !bottom) return undefined;
  return { left, right, top, bottom };
}

function parseEffectiveMargin(e, computedStyle) {
  const authoredEdges = authoredMarginEdges(e, computedStyle);
  if (!hasAuthoredMarginDeclaration(e, computedStyle) &&
      !Object.values(authoredEdges).includes("auto")) return undefined;

  return representedEdges({
    left: effectiveMarginValue(authoredEdges.left, computedStyle.marginLeft),
    right: effectiveMarginValue(authoredEdges.right, computedStyle.marginRight),
    top: effectiveMarginValue(authoredEdges.top, computedStyle.marginTop),
    bottom: effectiveMarginValue(authoredEdges.bottom, computedStyle.marginBottom),
  });
}

function effectiveMarginValue(authoredValue, computedValue) {
  if (marginValueIsNonInitial(authoredValue)) return authoredValue;
  return marginValueIsNonInitial(computedValue) ? computedValue : "";
}

function parseSize(size) {
  const width = parseSizeDimension(size.width);
  const height = parseSizeDimension(size.height);

  if (!width && !height) return undefined;
  return { width, height };
}

function parseSizeDimension(input) {
  if (!input) return undefined;
  if (typeof input === 'object') return input;
  return parseSizingDimension(input);
}

function px(value) {
  return { unit: 'px', value };
}

// The getter supplies one preferred/minimum/maximum group's actual values.
// All three roles share this one node-writing-mode axis/alias projection.
function parseElementSize(styleValue, computedStyle) {
  const width = styleValue("width");
  const height = styleValue("height");
  const inlineSize = styleValue("inlineSize");
  const blockSize = styleValue("blockSize");

  const vertical = isVerticalWritingMode(computedStyle.writingMode);
  const logicalWidth = vertical ? blockSize : inlineSize;
  const logicalHeight = vertical ? inlineSize : blockSize;
  // Specified logical/physical declarations remain distinct but cascade into
  // one computed axis. This source profile does not interpret their priority.
  if ((width && logicalWidth) || (height && logicalHeight)) {
    throw new Error('Root context: ConflictingSizingDeclarations');
  }
  return parseSize({ width: width || logicalWidth, height: height || logicalHeight });
}

function isVerticalWritingMode(writingMode) {
  return writingMode && (writingMode.startsWith("vertical-") || writingMode.startsWith("sideways-"));
}

function parseGaps(styleValue) {
  const gap = styleValue("gap");
  const rowGap = styleValue("rowGap");
  const columnGap = styleValue("columnGap");
  if (gap) {
    if (typeof gap === 'object') {
      const parsedGap = parseDimension(gap);
      return { row: parsedGap, column: parsedGap };
    }
    const components = splitCssComponentValues(gap);
    const gaps = components.map(part => parseDimension(part));
    // Only one authored component repeats. A present second component whose
    // projection failed must reach representedGaps as absent, not inherit row.
    return { row: gaps[0], column: components.length === 1 ? gaps[0] : gaps[1] };
  }
  if (rowGap || columnGap) {
    return { row: parseDimension(rowGap), column: parseDimension(columnGap) };
  }
  return undefined;
}

function splitCssComponentValues(input) {
  const values = [];
  let current = "";
  let depth = 0;
  for (const char of input.trim()) {
    if (/\s/.test(char) && depth === 0) {
      if (current) {
        values.push(current);
        current = "";
      }
      continue;
    }
    if (char === "(") depth++;
    if (char === ")") depth = Math.max(0, depth - 1);
    current += char;
  }
  if (current) values.push(current);
  return values;
}

function cssPropertyName(property) {
  return property.replace(/[A-Z]/g, match => `-${match.toLowerCase()}`);
}

function typedOmStyleValue(e, property) {
  if (!e.computedStyleMap) return undefined;
  const styleMap = e.computedStyleMap();
  if (!styleMap || !styleMap.get) return undefined;
  return parseTypedOmDimension(styleMap.get(cssPropertyName(property)));
}

function inlineAuthoredCalcValue(e, property) {
  return containsCalcFunction(e.style[property]) ? e.style[property] : "";
}


function parseGridTrackDefinitions(input) {
  if (input === '') return undefined;
  return new TrackSizingParser(input).parseList();
}

function parseGridAutoFlow(input) {
  if (!/column/.test(input) && !/row/.test(input) && !/dense/.test(input)) return undefined;
  const direction = /column/.test(input) ? 'column' : 'row';
  const algorithm = /dense/.test(input) ? 'dense' : 'sparse';
  return { direction, algorithm };
}

function parseGridPosition(input) {
  if (input === '') return undefined;
  if (input === 'auto') return { kind: 'auto' };
  if (/^span +\d+$/.test(input)) return { kind: 'span', value: parseInt(input.replace(/[^\d]/g, ''), 10) };
  if (/^-?\d+$/.test(input)) return { kind: 'line', value: parseInt(input, 10) };
  const parts = input.trim().split(/ +/);
  if (parts[0] === 'span') {
    const number = parts.find(part => /^-?\d+$/.test(part));
    const name = parts.find(part => !/^-?\d+$/.test(part) && part !== 'span');
    if (name) return { kind: 'named-span', name, ...(number === undefined ? {} : { occurrence: parseInt(number, 10) }) };
  }
  const name = parts.find(part => !/^-?\d+$/.test(part));
  const number = parts.find(part => /^-?\d+$/.test(part));
  if (name) return { kind: 'named-line', name, ...(number === undefined ? {} : { occurrence: parseInt(number, 10) }) };
  throw new Error(`Unsupported grid placement ${input}`);
}

function brInlineMetricsForElement(e, computedStyle) {
  if (e.tagName === 'BR') {
    const lineHeight = parseCssPx(computedStyle.lineHeight);
    if (!Number.isFinite(lineHeight) || lineHeight < 0) {
      throw new Error('inline BR metrics require a finite non-negative line height');
    }
    const baseline = lineHeight === 0 ? 0 : measureInlineBaselinePx(computedStyle, lineHeight);
    return {
      baseline,
      lineHeight,
    };
  }
  return undefined;
}

function layoutReadyShapeBands(e) {
  const raw = e.getAttribute('data-surgeist-shape-bands');
  if (raw === null) return undefined;

  let parsed;
  try {
    parsed = JSON.parse(raw);
  } catch (_) {
    throw new Error('data-surgeist-shape-bands must be valid JSON');
  }
  if (!Array.isArray(parsed) || parsed.length === 0) {
    throw new Error('data-surgeist-shape-bands must be a nonempty finite table');
  }

  const seen = new Set();
  return parsed.map((band, index) => {
    const allowed = new Set(['bandMinimum', 'bandMaximum', 'intervalMinimum', 'intervalMaximum']);
    if (!band || typeof band !== 'object' || Array.isArray(band) ||
        Object.keys(band).some((key) => !allowed.has(key))) {
      throw new Error(`shape band ${index} has an unsupported field`);
    }
    const { bandMinimum, bandMaximum, intervalMinimum, intervalMaximum } = band;
    if (!Number.isFinite(bandMinimum) || !Number.isFinite(bandMaximum) || bandMinimum > bandMaximum) {
      throw new Error(`shape band ${index} requires finite ordered query endpoints`);
    }
    const key = `${bandMinimum}:${bandMaximum}`;
    if (seen.has(key)) throw new Error(`shape band ${index} duplicates query ${key}`);
    seen.add(key);

    const hasIntervalMinimum = intervalMinimum !== undefined;
    const hasIntervalMaximum = intervalMaximum !== undefined;
    if (hasIntervalMinimum !== hasIntervalMaximum) {
      throw new Error(`shape band ${index} requires both interval endpoints`);
    }
    if (hasIntervalMinimum &&
        (!Number.isFinite(intervalMinimum) || !Number.isFinite(intervalMaximum) ||
         intervalMinimum > intervalMaximum)) {
      throw new Error(`shape band ${index} requires finite ordered interval endpoints`);
    }

    return hasIntervalMinimum
      ? { bandMinimum, bandMaximum, interval: { minimum: intervalMinimum, maximum: intervalMaximum } }
      : { bandMinimum, bandMaximum };
  });
}

const GRID_TEMPLATE_AREA_IDENT = /^-?(?:[A-Za-z_]|-[A-Za-z_])[A-Za-z0-9_-]*$/;
const GRID_TEMPLATE_AREA_RESERVED_IDENTS = new Set([
  'auto',
  'default',
  'inherit',
  'initial',
  'none',
  'revert',
  'revert-layer',
  'span',
  'unset',
]);

function parseGridTemplateAreas(input) {
  if (input === undefined || input === null) return undefined;
  if (typeof input !== 'string') throw new Error('grid-template-areas must be a string');
  const value = input.trim();
  if (value === '' || value === 'none') return undefined;

  const rows = [];
  const rowPattern = /"([^"\\]*)"/g;
  let cursor = 0;
  for (const match of value.matchAll(rowPattern)) {
    if (value.slice(cursor, match.index).trim() !== '') {
      throw new Error(`Unsupported grid-template-areas syntax ${input}`);
    }
    const row = match[1].trim();
    if (row === '') throw new Error('grid-template-areas rows must not be empty');
    const cells = row.split(/\s+/).map((cell) => {
      if (/^\.+$/.test(cell)) return null;
      if (!GRID_TEMPLATE_AREA_IDENT.test(cell) || GRID_TEMPLATE_AREA_RESERVED_IDENTS.has(cell)) {
        throw new Error(`Unsupported grid-template-areas cell ${cell}`);
      }
      return cell;
    });
    rows.push(cells);
    cursor = match.index + match[0].length;
  }
  if (rows.length === 0 || value.slice(cursor).trim() !== '') {
    throw new Error(`Unsupported grid-template-areas syntax ${input}`);
  }

  const columnCount = rows[0].length;
  if (rows.some((row) => row.length !== columnCount)) {
    throw new Error('grid-template-areas rows must have equal lengths');
  }

  const areas = new Map();
  rows.forEach((row, rowIndex) => row.forEach((cell, columnIndex) => {
    if (cell === null) return;
    const area = areas.get(cell) ?? {
      rowStart: rowIndex,
      rowEnd: rowIndex,
      columnStart: columnIndex,
      columnEnd: columnIndex,
      count: 0,
    };
    area.rowStart = Math.min(area.rowStart, rowIndex);
    area.rowEnd = Math.max(area.rowEnd, rowIndex);
    area.columnStart = Math.min(area.columnStart, columnIndex);
    area.columnEnd = Math.max(area.columnEnd, columnIndex);
    area.count++;
    areas.set(cell, area);
  }));
  for (const [name, area] of areas) {
    const rectangleCells = (area.rowEnd - area.rowStart + 1) *
      (area.columnEnd - area.columnStart + 1);
    if (rectangleCells !== area.count) {
      throw new Error(`grid-template-areas area ${name} must form one rectangle`);
    }
  }

  return rows;
}

function describeElement(e, expectedElement = null) {

  // Get precise, unrounded dimensions for the current element and it's parent
  let boundingRect = e.getBoundingClientRect();
  let parentBoundingRect = e.parentNode.getBoundingClientRect();
  const observation = rootLocalObservation;
  const isLocalRoot = observation?.root === e;
  // Retain the prior world-corner observation as diagnostics. Its x/y have
  // always been parent-local, not document positions.
  const worldRoundedLayout = {
    width: Math.round(boundingRect.right) - Math.round(boundingRect.left),
    height: Math.round(boundingRect.bottom) - Math.round(boundingRect.top),
    x: Math.round(boundingRect.x - parentBoundingRect.x),
    y: Math.round(boundingRect.y - parentBoundingRect.y),
    scrollWidth: e.scrollWidth, scrollHeight: e.scrollHeight,
    clientWidth: e.clientWidth, clientHeight: e.clientHeight,
  };
  const smartRoundedLayout = observation ? {
    ...worldRoundedLayout,
    width: Math.round(boundingRect.right - observation.x) - Math.round(boundingRect.left - observation.x),
    height: Math.round(boundingRect.bottom - observation.y) - Math.round(boundingRect.top - observation.y),
    x: isLocalRoot ? 0 : worldRoundedLayout.x,
    y: isLocalRoot ? 0 : worldRoundedLayout.y,
  } : worldRoundedLayout;

  const computedStyle = getComputedStyle(e);
  const useAuthoredCssRules = expectedElement !== null;
  const styleValue = (property) => useAuthoredCssRules ? authoredStyleValue(e, property, computedStyle) : e.style[property];
  const lengthStyleValue = (property) => {
    const authored = styleValue(property);
    const inlineCalc = inlineAuthoredCalcValue(e, property);
    if (!inlineCalc) return containsCalcFunction(authored) ? "" : authored;
    const typed = typedOmStyleValue(e, property);
    if (typed?.unit === "calc") return typed;
    return inlineCalc;
  };
  const preferredSizeValue = property => {
    const input = lengthStyleValue(property);
    return requireSizingProjection(input, parseSizeDimension(input));
  };
  const resolvedSizeValue = property => {
    const input = lengthStyleValue(property);
    return requireSizingProjection(input, parseResolvedDimension(input, computedStyle[property]));
  };
  const boundSize = prefix => parseElementSize(
    property => resolvedSizeValue(prefix + property[0].toUpperCase() + property.slice(1)),
    computedStyle,
  );
  const basisInput = lengthStyleValue('flexBasis');
  const flexBasis = requireSizingProjection(basisInput, parseSizingDimension(basisInput));
  const layoutReadyInlineRoot = e.getAttribute?.('data-surgeist-layout-ready-inline') === 'true';
  if (layoutReadyInlineRoot) resetLayoutReadyRangeLineRegistry(e);
  const preparedCapture = e.getAttribute?.('data-surgeist-prepared-text') === 'ahem-ttf-v1'
    ? capturePreparedChildren(e) : undefined;
  const children = preparedCapture ? preparedCapture.children : describeChildNodes(e, expectedElement);
  const brInlineMetrics = brInlineMetricsForElement(e, computedStyle);
  const lineControlParticipation = layoutReadyLineControlParticipation(e, computedStyle);
  const hasTypedInlineText = children.some((child) => ['inline-text', 'prepared-text'].includes(child.layoutInput));

  const described = {
    layoutInput: 'box',
    preparedParagraph: preparedCapture?.paragraph,
    gridAlignmentReference: gridAlignmentReferenceForElement(e),
    atomicParagraphMetrics: !preparedCapture ? atomicParagraphMetricsForElement(e, children) : undefined,
    tagName: e.tagName.toLowerCase(),
    layoutReadyInlineRoot: layoutReadyInlineRoot || undefined,
    layoutReadyAnonymousGridTextWrapper: layoutReadyAnonymousGridTextWrapper(e, computedStyle, children),
    lineControlParticipation,
    browserBrObservation: browserBrObservationForElement(e, computedStyle),
    unsupportedReason: preparedCapture?.unsupportedReason || unsupportedElementReason(e, computedStyle) ||
      (!preparedCapture && hasTypedInlineText ? 'Historical measured-width inline input lacks original source and pinned font capture' : unsupportedChildNodesReason(e)),
    style: {
      display: parseEnum(computedStyle.display),
      boxSizing: parseEnum(computedStyle.boxSizing),

      position: parseEnum(styleValue("position")),
      direction: parseEnum(computedStyle.direction),

      writingMode: parseEnum(computedStyle.writingMode),
      order: computedStyle.order,
      flexItemCollapse: normalizedFlexItemCollapse(e, computedStyle),

      cssFloat: parseEnum(styleValue("cssFloat")),
      clear: parseEnum(styleValue("clear")),

      textAlign: parseEnum(styleValue("textAlign")),
      verticalAlign: parseEnum(styleValue("verticalAlign")),
      fontFamily: parseEnum(computedStyle.fontFamily),
      fontSize: parseDimension(computedStyle.fontSize),
      lineHeight: parseDimension(computedStyle.lineHeight),
      inlineMetrics: brInlineMetrics,

      flexDirection: parseEnum(styleValue("flexDirection")),
      flexWrap: parseEnum(styleValue("flexWrap")),
      overflowX: parseEnum(computedStyle.overflowX),
      overflowY: parseEnum(computedStyle.overflowY),
      overflowClipMargin: computedStyle.overflowClipMargin,
      scrollbarGutter: computedStyle.scrollbarGutter,
      scrollPaddingTop: computedStyle.scrollPaddingTop,
      scrollPaddingRight: computedStyle.scrollPaddingRight,
      scrollPaddingBottom: computedStyle.scrollPaddingBottom,
      scrollPaddingLeft: computedStyle.scrollPaddingLeft,
      scrollMarginTop: computedStyle.scrollMarginTop,
      scrollMarginRight: computedStyle.scrollMarginRight,
      scrollMarginBottom: computedStyle.scrollMarginBottom,
      scrollMarginLeft: computedStyle.scrollMarginLeft,
      scrollSnapType: computedStyle.scrollSnapType,
      scrollSnapAlign: computedStyle.scrollSnapAlign,
      scrollSnapStop: computedStyle.scrollSnapStop,
      scrollbarWidth: getScrollBarWidth(),

      alignItems: parseEnum(styleValue("alignItems")),
      alignSelf: parseEnum(styleValue("alignSelf")),
      justifyItems: parseEnum(styleValue("justifyItems")),
      justifySelf: parseEnum(styleValue("justifySelf")),

      alignContent: parseEnum(styleValue("alignContent")),
      justifyContent: parseEnum(styleValue("justifyContent")),

      flexGrow: parseNumber(styleValue("flexGrow")),
      flexShrink: parseNumber(styleValue("flexShrink")),
      flexBasis,

      gridTemplateRows: parseGridTrackDefinitions(lengthStyleValue("gridTemplateRows")),
      gridTemplateColumns: parseGridTrackDefinitions(lengthStyleValue("gridTemplateColumns")),
      gridTemplateAreas: parseGridTemplateAreas(
        styleValue("gridTemplateAreas") || computedStyle.gridTemplateAreas
      ),
      gridAutoRows: parseGridTrackDefinitions(lengthStyleValue("gridAutoRows")),
      gridAutoColumns: parseGridTrackDefinitions(lengthStyleValue("gridAutoColumns")),
      gridAutoFlow: parseGridAutoFlow(styleValue("gridAutoFlow")),

      gridRowStart: parseGridPosition(styleValue("gridRowStart")),
      gridRowEnd: parseGridPosition(styleValue("gridRowEnd")),
      gridColumnStart: parseGridPosition(styleValue("gridColumnStart")),
      gridColumnEnd: parseGridPosition(styleValue("gridColumnEnd")),

      gap: representedGaps(lengthStyleValue),

      size: parseElementSize(preferredSizeValue, computedStyle),
      minSize: boundSize('min'),
      maxSize: boundSize('max'),
      aspectRatio: parseRatio(styleValue("aspectRatio")),

      margin: parseEffectiveMargin(e, computedStyle),

      padding: representedInlineEdges({
        left: lengthStyleValue("paddingLeft"),
        right: lengthStyleValue("paddingRight"),
        top: lengthStyleValue("paddingTop"),
        bottom: lengthStyleValue("paddingBottom"),
      }, lengthStyleValue("paddingInlineStart"), lengthStyleValue("paddingInlineEnd"), computedStyle),

      border: representedEdges({
        left: lengthStyleValue("borderLeftWidth"),
        right: lengthStyleValue("borderRightWidth"),
        top: lengthStyleValue("borderTopWidth"),
        bottom: lengthStyleValue("borderBottomWidth"),
      }),

      inset: representedInlineEdges({
        left: lengthStyleValue("left"),
        right: lengthStyleValue("right"),
        top: lengthStyleValue("top"),
        bottom: lengthStyleValue("bottom"),
      }, lengthStyleValue("insetInlineStart"), lengthStyleValue("insetInlineEnd"), computedStyle),
    },

    // The textContent is used for generating intrinsic sizing measure funcs
    // So we're only interested in the text content of leaf nodes
    textContent: !hasTypedInlineText && e.childElementCount === 0 && e.textContent.length && e.textContent !== "\n"
      ? e.textContent
      : undefined,

    // The layout of the node in full precision (floating-point)
    unroundedLayout: {
      width: boundingRect.width,
      height: boundingRect.height,
      x: isLocalRoot ? 0 : boundingRect.x - parentBoundingRect.x,
      y: isLocalRoot ? 0 : boundingRect.y - parentBoundingRect.y,
      scrollWidth: e.scrollWidth,
      scrollHeight: e.scrollHeight,
      clientWidth: e.clientWidth,
      clientHeight: e.clientHeight,
    },

    // Original offset/offset-parent diagnostics. These do not define the
    // selected absolute-local observation or its cumulative corner phase.
    naivelyRoundedLayout: {
      width: e.offsetWidth,
      height: e.offsetHeight,
      x: e.offsetLeft + e.parentNode.clientLeft,
      y: e.offsetTop + e.parentNode.clientTop,
      scrollWidth: e.scrollWidth,
      scrollHeight: e.scrollHeight,
      clientWidth: e.clientWidth,
      clientHeight: e.clientHeight,
    },

    // Absolute-local observations normalize continuous corners BEFORE Q.
    // Child locations retain Native's Q(parent-local delta) policy; only the
    // root location is zero. BlockFlow keeps its actual world-corner phase.
    smartRoundedLayout,
    worldRoundedLayout,

    // Whether the test should enable rounding
    useRounding: e.getAttribute("data-test-rounding") !== "false",

    viewport: undefined,

    shapeBands: layoutReadyShapeBands(e),

    children,
  };
  if (e.id === 'test-root') described.gridAlignmentProfile = completeGridAlignmentProfile(described);
  return described;
}

window.__surgeistGridTemplateAreaCaptureInstalled = true;

function normalizedFlexItemCollapse(e, computedStyle) {
  if (computedStyle.visibility !== 'collapse' ||
      computedStyle.display === 'none' ||
      computedStyle.position === 'absolute' ||
      computedStyle.position === 'fixed') {
    return undefined;
  }
  const parent = e.parentElement;
  if (!parent || getComputedStyle(parent).display !== 'flex') return undefined;
  return 'collapsed';
}

function authoredStyleValue(e, property, computedStyle) {
  if (e.style[property]) return e.style[property];

  let value = "";
  let hadOpaqueSheet = false;
  for (const sheet of Array.from(document.styleSheets)) {
    let rules;
    try {
      rules = sheet.cssRules;
    } catch (_) {
      hadOpaqueSheet = true;
      continue;
    }
    for (const rule of Array.from(rules)) {
      if (rule.type !== CSSRule.STYLE_RULE) continue;
      if (!rule.style[property]) continue;
      try {
        if (e.matches(rule.selectorText)) value = rule.style[property];
      } catch (_) {
        continue;
      }
    }
  }
  if (!value && hadOpaqueSheet) return nonInitialComputedStyleValue(property, computedStyle);
  return value;
}

function nonInitialComputedStyleValue(property, computedStyle) {
  const initial = {
    alignContent: "normal",
    alignItems: "normal",
    alignSelf: "auto",
    clear: "none",
    cssFloat: "none",
    flexBasis: "auto",
    flexDirection: "row",
    flexGrow: "0",
    flexShrink: "1",
    flexWrap: "nowrap",
    gridAutoColumns: "auto",
    gridAutoFlow: "row",
    gridAutoRows: "auto",
    gridColumnEnd: "auto",
    gridColumnStart: "auto",
    gridRowEnd: "auto",
    gridRowStart: "auto",
    gridTemplateColumns: "none",
    gridTemplateRows: "none",
    justifyContent: "normal",
    justifyItems: "normal",
    justifySelf: "auto",
    overflowX: "visible",
    overflowY: "visible",
    position: "static",
    textAlign: "start",
    verticalAlign: "baseline",
    writingMode: "horizontal-tb",
  }[property];
  if (initial === undefined) return "";
  const value = computedStyle[property];
  return value && value !== initial ? value : "";
}

function hasAuthoredMarginDeclaration(e, computedStyle) {
  if (styleDeclarationHasMargin(e.style)) return true;

  let hadOpaqueSheet = false;
  for (const sheet of Array.from(document.styleSheets)) {
    let rules;
    try {
      rules = sheet.cssRules;
    } catch (_) {
      hadOpaqueSheet = true;
      continue;
    }
    for (const rule of Array.from(rules)) {
      if (rule.type !== CSSRule.STYLE_RULE) continue;
      if (!styleDeclarationHasMargin(rule.style)) continue;
      try {
        if (e.matches(rule.selectorText)) return true;
      } catch (_) {
        continue;
      }
    }
  }

  return hadOpaqueSheet && computedMarginIsNonInitial(computedStyle);
}

function authoredMarginEdges(e, computedStyle) {
  const typedOmEdges = typedOmMarginEdges(e, inlineAuthoredMarginHasCalc(e.style));
  if (typedOmEdges) return typedOmEdges;

  const edges = { left: "", right: "", top: "", bottom: "" };
  applyInlineAuthoredMarginDeclarations(edges, e.style, computedStyle);
  return edges;
}

function typedOmMarginEdges(e, allowCalc = false) {
  if (!e.computedStyleMap) return undefined;
  const styleMap = e.computedStyleMap();
  if (!styleMap || !styleMap.get) return undefined;
  return {
    left: typedOmMarginValue(styleMap.get("margin-left"), allowCalc),
    right: typedOmMarginValue(styleMap.get("margin-right"), allowCalc),
    top: typedOmMarginValue(styleMap.get("margin-top"), allowCalc),
    bottom: typedOmMarginValue(styleMap.get("margin-bottom"), allowCalc),
  };
}

function typedOmMarginValue(value, allowCalc = false) {
  if (value?.value === "auto") return "auto";
  const dimension = parseTypedOmDimension(value);
  if (!dimension) return "";
  if (dimension.unit === "percent") return `${dimension.value * 100}%`;
  if (dimension.unit === "px") return `${dimension.value}px`;
  if (dimension.unit === "calc" && allowCalc) return dimension.value;
  return "";
}

function inlineAuthoredMarginHasCalc(style) {
  return styleDeclarationProperties(style).some((property) => {
    return property.startsWith("margin") && containsCalcFunction(styleDeclarationValue(style, property));
  });
}

function applyInlineAuthoredMarginDeclarations(edges, style, computedStyle) {
  const importantEdges = new Set();
  for (const property of styleDeclarationProperties(style)) {
    const important = style.getPropertyPriority?.(property) === "important";
    const assign = (edge, value) => {
      if (!important && importantEdges.has(edge)) return;
      edges[edge] = value;
      if (important) importantEdges.add(edge);
    };
    applyInlineAuthoredMarginDeclaration(assign, property, styleDeclarationValue(style, property), computedStyle);
  }
}

function applyInlineAuthoredMarginDeclaration(assign, property, value, computedStyle) {
  if (!value.trim()) return;

  switch (property) {
    case "margin-top":
      assign("top", value);
      return;
    case "margin-right":
      assign("right", value);
      return;
    case "margin-bottom":
      assign("bottom", value);
      return;
    case "margin-left":
      assign("left", value);
      return;
    case "margin-block-start":
      assign(blockStartEdge(computedStyle), value);
      return;
    case "margin-block-end":
      assign(blockEndEdge(computedStyle), value);
      return;
    case "margin-block": {
      const [start, end = start] = splitCssComponentValues(value);
      assign(blockStartEdge(computedStyle), start);
      assign(blockEndEdge(computedStyle), end);
      return;
    }
    case "margin-inline-start":
      assign(inlineStartEdge(computedStyle), value);
      return;
    case "margin-inline-end":
      assign(inlineEndEdge(computedStyle), value);
      return;
    case "margin-inline": {
      const [start, end = start] = splitCssComponentValues(value);
      assign(inlineStartEdge(computedStyle), start);
      assign(inlineEndEdge(computedStyle), end);
      return;
    }
    case "margin": {
      const parts = splitCssComponentValues(value);
      const [top, right = top, bottom = top, left = right] = parts;
      assign("top", top);
      assign("right", right);
      assign("bottom", bottom);
      assign("left", left);
      return;
    }
  }
}

function styleDeclarationHasMargin(style) {
  return styleDeclarationProperties(style).some((property) => {
    return property.startsWith("margin") && marginValueIsNonInitial(styleDeclarationValue(style, property));
  });
}

function marginValueIsNonInitial(value) {
  if (!value) return false;
  const parts = splitCssComponentValues(value);
  return parts.some((part) => part !== "0" && part !== "0px");
}

function computedMarginIsNonInitial(computedStyle) {
  return ["marginLeft", "marginRight", "marginTop", "marginBottom"].some((property) => {
    return marginValueIsNonInitial(computedStyle[property]);
  });
}

function styleDeclarationProperties(style) {
  if (!style || !style.length) return [];
  return Array.from({ length: style.length }, (_, index) => style[index]).filter(Boolean);
}

function styleDeclarationValue(style, property) {
  if (!style) return "";
  return style.getPropertyValue ? style.getPropertyValue(property) : "";
}

function blockStartEdge(computedStyle) {
  switch (computedStyle.writingMode) {
    case "vertical-rl":
    case "sideways-rl":
      return "right";
    case "vertical-lr":
    case "sideways-lr":
      return "left";
    default:
      return "top";
  }
}

function blockEndEdge(computedStyle) {
  return oppositeEdge(blockStartEdge(computedStyle));
}

function inlineStartEdge(computedStyle) {
  const rtl = computedStyle.direction === "rtl";
  switch (computedStyle.writingMode) {
    case "vertical-rl":
    case "vertical-lr":
    case "sideways-rl":
      return rtl ? "bottom" : "top";
    case "sideways-lr":
      return rtl ? "top" : "bottom";
    default:
      return rtl ? "right" : "left";
  }
}

function inlineEndEdge(computedStyle) {
  return oppositeEdge(inlineStartEdge(computedStyle));
}

function oppositeEdge(edge) {
  return { left: "right", right: "left", top: "bottom", bottom: "top" }[edge];
}

function explicitTrueMarker(e, name) {
  const raw = e.getAttribute?.(name);
  if (raw === null || raw === undefined) return false;
  if (raw !== 'true') throw new Error(`${name} must be exactly true`);
  return true;
}

function inlineBoundaryData(kind, metrics = undefined) {
  return {
    layoutInput: 'inline-boundary',
    inlineBoundary: metrics ? { kind, ...metrics } : { kind },
    children: [],
  };
}

function layoutReadyAnonymousGridTextWrapper(e, computedStyle, children) {
  if (!explicitTrueMarker(e, 'data-surgeist-anonymous-grid-text-wrapper')) return undefined;
  if (!['grid', 'inline-grid', 'grid-lanes', 'inline-grid-lanes'].includes(computedStyle.display)) {
    throw new Error('data-surgeist-anonymous-grid-text-wrapper requires a grid formatting role');
  }
  if (children.length === 0 || children.some((child) => {
    return child.layoutInput !== 'inline-text' || child.children?.length !== 0;
  })) {
    throw new Error('data-surgeist-anonymous-grid-text-wrapper requires only direct typed text');
  }
  const unsupportedDirectChild = Array.from(e.childNodes).some((child) => {
    return child.nodeType === Node.ELEMENT_NODE ||
      child.nodeType === Node.TEXT_NODE && !/^\s*$/.test(child.textContent) &&
        !children.some((candidate) => candidate.inlineSegments?.[0]?.id === Array.from(e.childNodes).indexOf(child));
  });
  if (unsupportedDirectChild) {
    throw new Error('data-surgeist-anonymous-grid-text-wrapper rejects mixed fallback content');
  }
  return true;
}

function layoutReadyTransparentInlineProjection(e) {
  if (!explicitTrueMarker(e, 'data-surgeist-transparent-inline-container')) return undefined;
  if (e.tagName !== 'BDO' || getComputedStyle(e).display !== 'inline') {
    throw new Error('data-surgeist-transparent-inline-container requires an inline bdo');
  }
  const childNodes = Array.from(e.childNodes);
  if (childNodes.length !== 1 || childNodes[0].nodeType !== Node.TEXT_NODE ||
      !shouldSerializeLayoutReadyText(childNodes[0], childNodes, 0, e)) {
    throw new Error('data-surgeist-transparent-inline-container requires one direct shaped-text child');
  }
  const bidiLevels = layoutReadyInlineBidiLevels(e, childNodes);
  const text = layoutReadyTextNodeData(
    childNodes[0], e, 0, undefined, consumeLayoutReadyInlineBidiLevel(bidiLevels, 0)
  );
  if (!text) {
    throw new Error('data-surgeist-transparent-inline-container requires one complete shaped-text child');
  }
  rejectUnusedLayoutReadyInlineBidiLevels(bidiLevels);
  return [inlineBoundaryData('start'), text, inlineBoundaryData('end')];
}

function layoutReadyInlineBidiLevels(parent, childNodes) {
  const raw = parent.getAttribute?.('data-surgeist-inline-bidi-levels');
  if (raw === null || raw === undefined) return new Map();

  let parsed;
  try {
    parsed = JSON.parse(raw);
  } catch (_) {
    throw new Error('data-surgeist-inline-bidi-levels must be valid JSON');
  }
  if (!Array.isArray(parsed) || parsed.length === 0) {
    throw new Error('data-surgeist-inline-bidi-levels must be a nonempty finite table');
  }

  const bidiLevels = new Map();
  for (const [recordIndex, record] of parsed.entries()) {
    const fields = new Set(['sourceIndex', 'bidiLevel', 'whenDirection']);
    const hasDirection = record && Object.hasOwn(record, 'whenDirection');
    const expectedFieldCount = hasDirection ? 3 : 2;
    if (!record || typeof record !== 'object' || Array.isArray(record) ||
        Object.keys(record).length !== expectedFieldCount ||
        Object.keys(record).some((key) => !fields.has(key))) {
      throw new Error(`inline bidi level ${recordIndex} must contain exactly the closed fields`);
    }
    const { sourceIndex, bidiLevel, whenDirection } = record;
    if (!Number.isInteger(sourceIndex) || sourceIndex < 0 || sourceIndex >= childNodes.length) {
      throw new Error(`inline bidi level ${recordIndex} requires an existing non-negative sourceIndex`);
    }
    if (bidiLevels.has(sourceIndex)) {
      throw new Error(`inline bidi level ${recordIndex} duplicates sourceIndex ${sourceIndex}`);
    }
    if (!Number.isInteger(bidiLevel) || bidiLevel < 1 || bidiLevel > 126) {
      throw new Error(`inline bidi level ${recordIndex} must be an integer in 1..=126`);
    }
    if (hasDirection && whenDirection !== 'ltr' && whenDirection !== 'rtl') {
      throw new Error(`inline bidi level ${recordIndex} requires direction ltr or rtl`);
    }
    const child = childNodes[sourceIndex];
    const shapedText = child.nodeType === Node.TEXT_NODE &&
      shouldSerializeLayoutReadyText(child, childNodes, sourceIndex, parent);
    const atomic = child.nodeType === Node.ELEMENT_NODE && child.tagName !== 'BR' &&
      isLoweredAtomicInline(child);
    if (!shapedText && !atomic) {
      throw new Error(`inline bidi level ${recordIndex} must target shaped text or an atomic inline`);
    }
    const applicable = !hasDirection || getComputedStyle(parent).direction === whenDirection;
    bidiLevels.set(sourceIndex, { bidiLevel, applicable });
  }
  return bidiLevels;
}

function consumeLayoutReadyInlineBidiLevel(bidiLevels, sourceIndex) {
  const record = bidiLevels.get(sourceIndex);
  bidiLevels.delete(sourceIndex);
  return record?.applicable ? record.bidiLevel : 0;
}

function rejectUnusedLayoutReadyInlineBidiLevels(bidiLevels) {
  for (const [sourceIndex, record] of bidiLevels) {
    if (record.applicable) {
      throw new Error(
        `data-surgeist-inline-bidi-levels contains an unused sourceIndex ${sourceIndex}`
      );
    }
  }
}

function layoutReadyInlineStruts(parent, childNodes) {
  const raw = parent.getAttribute?.('data-surgeist-inline-struts');
  if (raw === null || raw === undefined) return new Map();
  if (parent.getAttribute?.('data-surgeist-layout-ready-inline') !== 'true') {
    throw new Error('data-surgeist-inline-struts requires a layout-ready containing root');
  }

  let parsed;
  try {
    parsed = JSON.parse(raw);
  } catch (_) {
    throw new Error('data-surgeist-inline-struts must be valid JSON');
  }
  if (!Array.isArray(parsed) || parsed.length === 0) {
    throw new Error('data-surgeist-inline-struts must be a nonempty finite table');
  }

  const struts = new Map();
  for (const [recordIndex, record] of parsed.entries()) {
    const fields = new Set(['beforeSourceIndex', 'baseline', 'lineHeight']);
    if (!record || typeof record !== 'object' || Array.isArray(record) ||
        Object.keys(record).length !== fields.size ||
        Object.keys(record).some((key) => !fields.has(key))) {
      throw new Error(`inline strut ${recordIndex} must contain exactly the closed fields`);
    }
    const { beforeSourceIndex, baseline, lineHeight } = record;
    if (!Number.isInteger(beforeSourceIndex) || beforeSourceIndex < 0 || beforeSourceIndex >= childNodes.length) {
      throw new Error(`inline strut ${recordIndex} requires an existing non-negative beforeSourceIndex`);
    }
    if (struts.has(beforeSourceIndex)) {
      throw new Error(`inline strut ${recordIndex} duplicates beforeSourceIndex ${beforeSourceIndex}`);
    }
    if (!Number.isFinite(baseline) || !Number.isFinite(lineHeight) ||
        lineHeight <= 0 || baseline < 0 || baseline > lineHeight) {
      throw new Error(`inline strut ${recordIndex} requires complete finite metrics`);
    }
    const target = childNodes[beforeSourceIndex];
    if (target.nodeType !== Node.ELEMENT_NODE || target.tagName === 'BR' || !isLoweredAtomicInline(target)) {
      throw new Error(`inline strut ${recordIndex} must target a typed atomic child`);
    }
    struts.set(beforeSourceIndex, inlineBoundaryData('start', { baseline, lineHeight }));
  }
  return struts;
}

function layoutReadyInlineBreaks(parent, childNodes) {
  const raw = parent.getAttribute('data-surgeist-inline-breaks');
  if (raw === null) return new Map();

  let parsed;
  try {
    parsed = JSON.parse(raw);
  } catch (_) {
    throw new Error('data-surgeist-inline-breaks must be valid JSON');
  }
  if (!Array.isArray(parsed) || parsed.length === 0) {
    throw new Error('data-surgeist-inline-breaks must be a nonempty finite table');
  }

  const breaks = new Map();
  const breakKinds = new Set(['prohibited', 'allowed', 'allowed-with-replacement', 'mandatory']);
  for (const [recordIndex, record] of parsed.entries()) {
    const fields = new Set(['sourceIndex', 'followingBreak', 'replacementInlineExtent']);
    if (!record || typeof record !== 'object' || Array.isArray(record) ||
        Object.keys(record).some((key) => !fields.has(key))) {
      throw new Error(`inline break ${recordIndex} has an unsupported field`);
    }
    const { sourceIndex, followingBreak, replacementInlineExtent } = record;
    if (!Number.isInteger(sourceIndex) || sourceIndex < 0 || sourceIndex >= childNodes.length) {
      throw new Error(`inline break ${recordIndex} requires an existing non-negative sourceIndex`);
    }
    if (breaks.has(sourceIndex)) {
      throw new Error(`inline break ${recordIndex} duplicates sourceIndex ${sourceIndex}`);
    }
    if (!breakKinds.has(followingBreak)) {
      throw new Error(`inline break ${recordIndex} has invalid followingBreak ${followingBreak}`);
    }
    const child = childNodes[sourceIndex];
    const text = child.nodeType === Node.TEXT_NODE;
    const atomic = child.nodeType === Node.ELEMENT_NODE && child.tagName !== 'BR' && isLoweredAtomicInline(child);
    if (!text && !atomic) {
      throw new Error(`inline break ${recordIndex} must target shaped text or an atomic inline`);
    }
    if (followingBreak === 'allowed-with-replacement') {
      if (!text || !Number.isFinite(replacementInlineExtent) || replacementInlineExtent < 0) {
        throw new Error(`inline break ${recordIndex} replacement requires shaped text and a finite non-negative extent`);
      }
    } else if (replacementInlineExtent !== undefined) {
      throw new Error(`inline break ${recordIndex} replacement requires allowed-with-replacement`);
    }
    breaks.set(sourceIndex, replacementInlineExtent === undefined
      ? { followingBreak }
      : { followingBreak, replacementInlineExtent });
  }
  return breaks;
}

function describeChildNodes(e, expectedElement = null) {
  let children = [];
  let childNodes = Array.from(e.childNodes);
  const inlineBreaks = layoutReadyInlineBreaks(e, childNodes);
  const inlineStruts = layoutReadyInlineStruts(e, childNodes);
  const inlineBidiLevels = layoutReadyInlineBidiLevels(e, childNodes);
  const atomicMetricSource = atomicParagraphSource(e);
  const layoutReadyInlineRun = hasLayoutReadyInlineFixture(e) && childNodes.some((child, index) => {
    return child.nodeType === Node.ELEMENT_NODE && child.tagName === 'BR' ||
      child.nodeType === Node.TEXT_NODE && shouldSerializeLayoutReadyText(child, childNodes, index, e);
  });
  for (let i = 0; i < childNodes.length; i++) {
    let child = childNodes[i];
    if (inlineStruts.has(i)) {
      children.push(inlineStruts.get(i));
      inlineStruts.delete(i);
    }
    if (child.nodeType === Node.ELEMENT_NODE) {
      const transparent = layoutReadyTransparentInlineProjection(child);
      if (transparent) {
        children.push(...transparent);
        continue;
      }
      const described = describeElement(child, expectedElement);
      if (layoutReadyInlineRun && child.tagName !== 'BR' && isLoweredAtomicInline(child)) {
        described.atomicInlineParticipation = {
          bidiLevel: atomicMetricSource ? (getComputedStyle(e).direction === 'rtl' ? 1 : 0) : consumeLayoutReadyInlineBidiLevel(inlineBidiLevels, i),
          ...(inlineBreaks.get(i) || { followingBreak: 'prohibited' }),
        };
        inlineBreaks.delete(i);
      }
      children.push(described);
    } else if (
      layoutReadyInlineRun &&
      child.nodeType === Node.TEXT_NODE &&
      shouldSerializeLayoutReadyText(child, childNodes, i, e)
    ) {
      const described = layoutReadyTextNodeData(
        child, e, i, inlineBreaks.get(i), consumeLayoutReadyInlineBidiLevel(inlineBidiLevels, i)
      );
      if (described) {
        children.push(described);
        inlineBreaks.delete(i);
      }
    }
  }
  if (inlineBreaks.size !== 0) {
    throw new Error(`data-surgeist-inline-breaks contains an unused sourceIndex ${inlineBreaks.keys().next().value}`);
  }
  if (inlineStruts.size !== 0) {
    throw new Error(`data-surgeist-inline-struts contains an unused beforeSourceIndex ${inlineStruts.keys().next().value}`);
  }
  rejectUnusedLayoutReadyInlineBidiLevels(inlineBidiLevels);
  return children;
}

function shouldSerializeLayoutReadyText(node, siblings, index, parent) {
  if (!/^\s*$/.test(node.textContent)) return true;
  return isSignificantInlineWhitespace(node, siblings, index, parent);
}

function layoutReadyTextNodeData(
  node, parent, segmentId, reviewedBreak = undefined, reviewedBidiLevel = 0
) {
  const range = document.createRange();
  range.selectNodeContents(node);
  const rect = range.getBoundingClientRect();
  const fragmentRects = Array.from(range.getClientRects());
  range.detach();
  const completeZeroSizeBoundingTuple = [
    rect.x, rect.y, rect.left, rect.top, rect.right, rect.bottom, rect.width, rect.height,
  ].every(Number.isFinite) &&
    rect.width === 0 && rect.height === 0 &&
    rect.left === rect.x && rect.right === rect.x &&
    rect.top === rect.y && rect.bottom === rect.y;
  const zeroFragmentWhitespaceAnchor =
    fragmentRects.length === 0 &&
    /^\s+$/.test(node.textContent) &&
    isSignificantInlineWhitespace(node, Array.from(parent.childNodes || []), segmentId, parent) &&
    completeZeroSizeBoundingTuple;
  if (fragmentRects.length !== 1 && !zeroFragmentWhitespaceAnchor) {
    throw new Error(`layout-ready text segment ${segmentId} must have exactly one fragment`);
  }

  const containingRoot = layoutReadyInlineContainingRoot(parent);
  if (!containingRoot) {
    throw new Error(`layout-ready text segment ${segmentId} requires an explicit layout-ready inline root`);
  }
  const containingRootRect = containingRoot.getBoundingClientRect();
  const computedStyle = getComputedStyle(parent);
  const containingStyle = getComputedStyle(containingRoot);
  const fontSize = parseCssPx(computedStyle.fontSize);
  const lineHeight = resolveLineHeightPx(computedStyle.lineHeight, fontSize);
  const baseline = Math.min(lineHeight, estimateInlineBaselinePx(fontSize, lineHeight));
  const vertical = isVerticalWritingMode(computedStyle.writingMode);
  const inlineExtent = vertical ? rect.height : rect.width;
  const whitespace = /^\s+$/.test(node.textContent);
  const finite = [
    rect.x, rect.y, rect.width, rect.height,
    containingRootRect.x, containingRootRect.y,
    inlineExtent, baseline, lineHeight,
  ];
  if (!finite.every(Number.isFinite) || inlineExtent < 0 || baseline < 0 || lineHeight < baseline) {
    throw new Error(`layout-ready text segment ${segmentId} requires a complete finite tuple`);
  }

  const rangeInks = fragmentRects.map((fragment) => {
    const physicalStartEdge = inlineStartEdge(containingStyle);
    const horizontal = physicalStartEdge === 'left' || physicalStartEdge === 'right';
    const start = horizontal
      ? fragment[physicalStartEdge] - containingRootRect.left
      : fragment[physicalStartEdge] - containingRootRect.top;
    const advance = horizontal ? fragment.width : fragment.height;
    if (![start, advance].every(Number.isFinite) || advance < 0) {
      throw new Error(`layout-ready text Range rectangle ${segmentId} requires a complete finite inline tuple`);
    }
    return {
      sourceSegmentId: segmentId,
      lineIndex: layoutReadyRangeLineIndex(containingRoot, fragment),
      physicalStartEdge,
      start,
      advance,
    };
  });

  return {
    layoutInput: 'inline-text',
    inlineSegments: [{
      id: segmentId,
      inlineExtent,
      inlineBaseline: baseline,
      inlineLineHeight: lineHeight,
      bidiLevel: reviewedBidiLevel,
      whitespaceEdge: whitespace ? 'discard-at-both' : 'preserve',
      ...(reviewedBreak || { followingBreak: whitespace ? 'allowed' : 'prohibited' }),
    }],
    rangeInks,
    children: [],
  };
}

const layoutReadyRangeLineRegistries = new WeakMap();
const layoutReadyRangeLineTolerance = 0.1;

function resetLayoutReadyRangeLineRegistry(root) {
  layoutReadyRangeLineRegistries.set(root, { anchors: [], nextLineIndex: 0 });
}

function preparedRangeRootRect(root) {
  return rootLocalObservation?.root === root ? rootLocalObservation.stableRect : undefined;
}

function layoutReadyRangeLineIndex(root, fragment) {
  if (root?.getAttribute?.('data-surgeist-layout-ready-inline') !== 'true') {
    throw new Error('Range line identity requires an explicit layout-ready inline root');
  }
  let registry = layoutReadyRangeLineRegistries.get(root);
  if (!registry) {
    resetLayoutReadyRangeLineRegistry(root);
    registry = layoutReadyRangeLineRegistries.get(root);
  }

  const rootRect = preparedRangeRootRect(root) ?? root.getBoundingClientRect();
  const writingMode = getComputedStyle(root).writingMode;
  let coordinate;
  switch (writingMode) {
    case 'horizontal-tb':
      coordinate = fragment.top - rootRect.top;
      break;
    case 'vertical-rl':
    case 'sideways-rl':
      coordinate = rootRect.right - fragment.right;
      break;
    case 'vertical-lr':
    case 'sideways-lr':
      coordinate = fragment.left - rootRect.left;
      break;
    default:
      throw new Error(`Range line identity has unknown writing mode ${writingMode}`);
  }
  if (!Number.isFinite(coordinate)) {
    throw new Error('Range line identity requires a finite block-progress coordinate');
  }

  const matches = registry.anchors.filter((anchor) => {
    return Math.abs(anchor.coordinate - coordinate) <= layoutReadyRangeLineTolerance;
  });
  if (matches.length > 1) {
    throw new Error(`ambiguous Range line identity at block-progress ${coordinate}`);
  }
  if (matches.length === 1) return matches[0].lineIndex;

  const lineIndex = registry.nextLineIndex++;
  registry.anchors.push({ coordinate, lineIndex });
  return lineIndex;
}

function unsupportedElementReason(e, computedStyle) {
  if (
    e.tagName === 'BR' &&
    isVerticalWritingMode(computedStyle.writingMode) &&
    !hasLayoutReadyVerticalBrFixture(e) &&
    !hasLayoutReadyInlineFixture(e)
  ) {
    return "Unsupported vertical <br> line-break semantics";
  }
  if (
    e.tagName === 'BR' &&
    !hasSupportedBrLineBreakParent(e) &&
    !hasLayoutReadyInlineFixture(e)
  ) {
    return "Unsupported <br> outside block inline-run semantics";
  }
  return undefined;
}

function hasSupportedBrLineBreakParent(e) {
  const parent = e.parentElement;
  if (!parent) return false;
  return getComputedStyle(parent).display === "block";
}

function hasLayoutReadyVerticalBrFixture(e) {
  return e.parentElement?.getAttribute?.('data-surgeist-layout-ready-vertical-br') === 'true';
}

function hasLayoutReadyInlineFixture(e) {
  return layoutReadyInlineContainingRoot(e) !== undefined;
}

function layoutReadyInlineContainingRoot(e) {
  for (let current = e; current; current = current.parentElement) {
    if (current.getAttribute?.('data-surgeist-layout-ready-inline') === 'true') return current;
  }
  return undefined;
}

// Observation intent belongs to the actual fixture, independently of which
// native input profile represents its break. Only an inline BR has this band.
function browserBrObservationForElement(e, computedStyle) {
  for (let containing = e; containing; containing = containing.parentElement) {
    if (containing.getAttribute?.('id') !== 'test-root') continue;
    const request = containing.getAttribute('data-surgeist-br-observation');
    if (request === null) return undefined;
    if (request !== 'font-band') throw new Error('Unsupported explicit BR observation request');
    return e.tagName === 'BR' && computedStyle.display === 'inline' ? request : undefined;
  }
  return undefined;
}

function layoutReadyLineControlParticipation(e, computedStyle) {
  if (e.tagName !== 'BR' || computedStyle.display !== 'inline' ||
      !layoutReadyInlineContainingRoot(e.parentElement)) {
    return undefined;
  }
  return { kind: 'forced-break' };
}

function unsupportedChildNodesReason(e) {
  let childNodes = Array.from(e.childNodes);
  let hasElementChild = childNodes.some(child => child.nodeType === Node.ELEMENT_NODE);
  if (!hasElementChild) return undefined;

  for (let i = 0; i < childNodes.length; i++) {
    let child = childNodes[i];
    if (child.nodeType !== Node.TEXT_NODE) continue;
    if (!/^\s*$/.test(child.textContent) && !hasLayoutReadyInlineFixture(e)) {
      return "Unsupported mixed text/element content";
    }
    if (
      isSignificantInlineWhitespace(child, childNodes, i, e) &&
      !hasLayoutReadyInlineFixture(e)
    ) {
      return "Unsupported mixed text/element content";
    }
  }

  return undefined;
}

function isSignificantInlineWhitespace(node, siblings, index, parent = node.parentElement) {
  if (!/^\s+$/.test(node.textContent)) return false;
  const parentDisplay = parent ? getComputedStyle(parent).display : '';
  if (['grid', 'inline-grid', 'grid-lanes', 'inline-grid-lanes'].includes(parentDisplay)) {
    return false;
  }

  let previous = nearestElementSibling(siblings, index, -1);
  let next = nearestElementSibling(siblings, index, 1);
  return previous && next && isInlineLevel(previous) && isInlineLevel(next);
}

function nearestElementSibling(siblings, index, step) {
  for (let i = index + step; i >= 0 && i < siblings.length; i += step) {
    if (siblings[i].nodeType === Node.ELEMENT_NODE) return siblings[i];
    if (siblings[i].nodeType === Node.TEXT_NODE && !/^\s*$/.test(siblings[i].textContent)) return undefined;
  }
  return undefined;
}

function isInlineLevel(e) {
  let authored = e.style.display;
  let computed = getComputedStyle(e).display;
  return authored.startsWith("inline") || computed.startsWith("inline");
}

function isLoweredAtomicInline(e) {
  return ['inline-block', 'inline-grid', 'inline-grid-lanes'].includes(getComputedStyle(e).display);
}

// Source/font capture is independent from the Range rectangle oracle. Only an
// explicit fixture opt-in selects this finite profile; no old width is decoded
// as text, a glyph, a font size, an original range, or a resolved bidi level.
const PINNED_TEXT_FONT = Object.freeze({
  id: 'ahem-ttf-f0a92cd0',
  sha256: 'f0a92cd0cc45735591c9b5b1fa8aecd5194e8dc518895ca22af94a46c23550dc',
  faceIndex: 0, family: 'SurgeistParityAhem', style: 'normal', weight: 400,
  stretch: 'normal', features: 'normal', variations: 'normal', synthesis: 'none',
});
let pinnedTextFontState;

async function preparePinnedTextFont(supplied) {
  if (typeof document === 'undefined' || !document.querySelector?.('[data-surgeist-prepared-text="ahem-ttf-v1"]')) return;
  if (!Array.isArray(supplied) || supplied.length !== 22572 || supplied.some(byte => !Number.isInteger(byte) || byte < 0 || byte > 255)) {
    throw new Error('Pinned text profile requires the exact generator-owned font bytes');
  }
  const bytes = new Uint8Array(supplied);
  // The owning adapter SHA256-authenticates these captured bytes before JSON
  // transport. This exact Uint8Array also backs the loaded FontFace and cmap.
  for (const existing of document.fonts) { if (existing.family.replace(/^['"]|['"]$/g, '') === PINNED_TEXT_FONT.family) throw new Error('Pinned fixture family must not have another FontFace resource'); }
  const face = new FontFace(PINNED_TEXT_FONT.family, bytes.buffer, {
    style: 'normal', weight: '400', stretch: 'normal',
    featureSettings: 'normal', variationSettings: 'normal',
  });
  await face.load();
  document.fonts.add(face);
  await document.fonts.ready;
  if (face.status !== 'loaded') throw new Error('Pinned text FontFace did not load');
  pinnedTextFontState = { face, bytes };
}

function qualifyLoadedPreparedRangeFace() {
  const faces = Array.from(document.fonts);
  const face = pinnedTextFontState?.face;
  const stylesheetFace = faces.find(candidate => referenceFontFamily(candidate.family) === 'ahem');
  if (faces.length !== 2 || !stylesheetFace || !faces.includes(face) || face.status !== 'loaded' ||
      referenceFontFamily(face.family) !== referenceFontFamily(PINNED_TEXT_FONT.family) ||
      face.style !== 'normal' || face.weight !== '400' || !['normal', '100%'].includes(face.stretch) ||
      !hasDefaultFontMetricDescriptors(face) ||
      face.featureSettings !== 'normal' || face.variationSettings !== 'normal') {
    throw new Error('Root context: PreparedRangeFontAuthority');
  }
}

function pinnedTextNominalGlyph(codePoint) {
  if (!pinnedTextFontState || codePoint > 0xffff) return 0;
  const bytes = pinnedTextFontState.bytes;
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const count = view.getUint16(4);
  let cmap;
  for (let index = 0; index < count; index++) {
    const row = 12 + index * 16;
    if (String.fromCharCode(...bytes.slice(row, row + 4)) === 'cmap') cmap = view.getUint32(row + 8);
  }
  if (cmap === undefined) throw new Error('Pinned font lacks cmap');
  for (let index = 0; index < view.getUint16(cmap + 2); index++) {
    const row = cmap + 4 + index * 8;
    if (view.getUint16(row) !== 3 || view.getUint16(row + 2) !== 1) continue;
    const table = cmap + view.getUint32(row + 4);
    if (view.getUint16(table) !== 4) throw new Error('Pinned font cmap profile changed');
    const segments = view.getUint16(table + 6) / 2;
    for (let segment = 0; segment < segments; segment++) {
      const end = view.getUint16(table + 14 + segment * 2);
      const start = view.getUint16(table + 16 + segments * 2 + segment * 2);
      if (codePoint < start || codePoint > end) continue;
      const delta = view.getUint16(table + 16 + segments * 4 + segment * 2);
      const position = table + 16 + segments * 6 + segment * 2;
      const offset = view.getUint16(position);
      const glyph = offset === 0 ? codePoint : view.getUint16(position + offset + (codePoint - start) * 2);
      return glyph === 0 ? 0 : (glyph + delta) & 0xffff;
    }
  }
  return 0;
}

function glyphFreeSourceControl(codePoint) {
  return [0x0a, 0x0d, 0x061c, 0x200e, 0x200f, 0x2028, 0x2029].includes(codePoint) ||
    (codePoint >= 0x202a && codePoint <= 0x202e) || (codePoint >= 0x2066 && codePoint <= 0x2069);
}

function capturedTextStyle(parent) {
  const style = getComputedStyle(parent);
  const family = style.fontFamily.replace(/^['"]|['"]$/g, '');
  if (family !== PINNED_TEXT_FONT.family || style.fontStyle !== 'normal' || style.fontWeight !== '400' ||
      !['normal', '100%'].includes(style.fontStretch) || style.fontFeatureSettings !== 'normal' ||
      style.fontVariationSettings !== 'normal' || style.fontSynthesis !== 'none' ||
      !['horizontal-tb', 'vertical-lr', 'vertical-rl'].includes(style.writingMode) ||
      (style.writingMode !== 'horizontal-tb' &&
        (style.textOrientation !== 'mixed' || style.textCombineUpright !== 'none')) ||
      !['normal', ''].includes(style.unicodeBidi)) {
    throw new Error('Prepared text requires the exact unsynthesized pinned face and supported normal-bidi frame');
  }
  if (style.writingMode !== 'horizontal-tb' && style.dominantBaseline !== 'auto') {
    throw new Error('Vertical prepared text requires the default dominant baseline');
  }
  const fontSize = parseCssPx(style.fontSize);
  const lineHeight = parseCssPx(style.lineHeight);
  const whitespace = { normal: 'collapse', nowrap: 'collapse', pre: 'preserve', 'pre-wrap': 'preserve', 'break-spaces': 'preserve-spaces' }[style.whiteSpace];
  if (!whitespace || (style.writingMode !== 'horizontal-tb' && whitespace !== 'collapse') || !Number.isFinite(fontSize) || Math.fround(fontSize) !== fontSize ||
      !Number.isFinite(lineHeight) || fontSize <= 0 || lineHeight < 0 ||
      style.wordBreak !== 'normal' || style.overflowWrap !== 'normal') {
    throw new Error('Prepared text style is outside the closed finite source profile');
  }
  const spacing = value => value === 'normal' ? 0 : parseCssPx(value);
  const letterSpacing = spacing(style.letterSpacing), wordSpacing = spacing(style.wordSpacing);
  if (![letterSpacing, wordSpacing].every(Number.isFinite)) throw new Error('Prepared text spacing must be finite pixels');
  if (style.textTransform !== 'none' || style.textDecorationLine !== 'none' ||
      !['normal', 'none'].includes(style.fontVariant) || style.hyphens !== 'manual') throw new Error('Prepared text transforms/decorations need their own explicit source profile');
  const match = /^rgba?\(\s*([0-9.]+)[, ]+([0-9.]+)[, ]+([0-9.]+)(?:\s*[,/]\s*([0-9.]+))?\s*\)$/.exec(style.color);
  if (!match) throw new Error('Prepared text requires resolved sRGB color components');
  const color = [Number(match[1])/255, Number(match[2])/255, Number(match[3])/255, match[4] === undefined ? 1 : Number(match[4])].map(Math.fround);
  if (!color.every(component => Number.isFinite(component) && component >= 0 && component <= 1)) throw new Error('Prepared color is outside canonical RGBA');
  const locale = parent.closest('[lang]')?.getAttribute('lang') ?? null;
  return { fontSize, lineHeight, color, locale, whitespace, wrap: ['nowrap', 'pre'].includes(style.whiteSpace) ? 'none' : 'word', letterSpacing, wordSpacing };
}

function qualifyTransparentPreparedInline(node, containingElement) {
  // A metricless boundary can represent only an actually transparent inline.
  // Inspect the element itself even when it contains no text: its own strut,
  // box edges or positioning context cannot be recovered from child text rows.
  const own = capturedTextStyle(node);
  const containing = capturedTextStyle(containingElement);
  const style = getComputedStyle(node);
  if (own.fontSize !== containing.fontSize || own.lineHeight !== containing.lineHeight) {
    throw new Error('Transparent inline requires the containing font size and line height; an independent strut is unsupported');
  }
  const edges = ['paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft',
    'marginTop', 'marginRight', 'marginBottom', 'marginLeft',
    'borderTopWidth', 'borderRightWidth', 'borderBottomWidth', 'borderLeftWidth'];
  if (edges.some(edge => parseCssPx(style[edge]) !== 0)) {
    throw new Error('Transparent inline requires zero actual margin, padding and border edges');
  }
  if (style.verticalAlign !== 'baseline' || style.cssFloat !== 'none' || style.transform !== 'none' ||
      !isUnshiftedInlinePosition(style)) {
    throw new Error('Transparent inline positioning/alignment context is unsupported');
  }
}

// The prepared wire carries one containing writing frame. In the vertical
// extension only opaque empty boxes and collapsed original formatting source
// are admitted; visible glyph observations remain a separate profile.
function qualifyPreparedFrame(node, containingFrame) {
  const style = getComputedStyle(node);
  if (style.writingMode !== containingFrame.writingMode ||
      (containingFrame.writingMode !== 'horizontal-tb' &&
        (style.direction !== containingFrame.direction || style.textOrientation !== 'mixed' ||
          style.textCombineUpright !== 'none' || !['normal', ''].includes(style.unicodeBidi)))) {
    throw new Error('Prepared participant requires the actual containing writing frame');
  }
  return style;
}

function qualifyPreparedBreak(node, containingFrame, containingText) {
  const style = qualifyPreparedFrame(node, containingFrame);
  const own = capturedTextStyle(node);
  if (own.fontSize !== containingText.fontSize || own.lineHeight !== containingText.lineHeight ||
      style.display !== 'inline' || style.verticalAlign !== 'baseline' ||
      style.cssFloat !== 'none' || style.clear !== 'none' || style.transform !== 'none' ||
      !isUnshiftedInlinePosition(style)) {
    throw new Error('Prepared BR requires the containing font strut and undisplaced baseline control context');
  }
  const edges = ['paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft',
    'marginTop', 'marginRight', 'marginBottom', 'marginLeft',
    'borderTopWidth', 'borderRightWidth', 'borderBottomWidth', 'borderLeftWidth'];
  if (edges.some(edge => parseCssPx(style[edge]) !== 0)) {
    throw new Error('Prepared BR cannot discard independently authored box edges');
  }
}

function capturePreparedChildren(root) {
  const frame = getComputedStyle(root);
  const vertical = frame.writingMode !== 'horizontal-tb';
  const paragraph = { font: { ...PINNED_TEXT_FONT, loaded: pinnedTextFontState?.face.status === 'loaded' },
    direction: frame.direction, writingMode: frame.writingMode, baseStyle: undefined,
    origins: [], connections: [], atomics: [], controls: [], order: [] };
  const children = [];
  let sourceEnd = 0;
  let precedingTextNode;
  try {
    if (!paragraph.font.loaded) throw new Error('Pinned FontFace must load before capture');
    paragraph.baseStyle = capturedTextStyle(root);
    if (!['ltr', 'rtl'].includes(frame.direction)) throw new Error('Prepared text requires the actual resolved computed direction');
    resetLayoutReadyRangeLineRegistry(root);
    const capture = (element) => {
      for (const node of element.childNodes) {
        if (node.nodeType === Node.COMMENT_NODE) continue;
        const child = children.length;
        if (node.nodeType === Node.TEXT_NODE) {
          const text = node.textContent;
          qualifyPreparedFrame(node.parentElement, frame);
          if (vertical && !/^[\n\r ]*$/.test(text)) {
            throw new Error('Vertical prepared text admits only original LF, CR and SPACE formatting source');
          }
          const units = Array.from({ length: text.length }, (_, index) => text.charCodeAt(index));
          for (const character of text) {
            const cp = character.codePointAt(0);
            if (!glyphFreeSourceControl(cp) && !pinnedTextNominalGlyph(cp)) throw new Error(`Pinned font lacks original U+${cp.toString(16)}`);
          }
          const index = paragraph.origins.length;
          const style = capturedTextStyle(node.parentElement);
          paragraph.origins.push({ child, units, start: sourceEnd, end: sourceEnd + units.length, style });
          if (index !== 0) {
            const ancestors = new Set(); for (let ancestor = precedingTextNode.parentElement; ancestor; ancestor = ancestor.parentElement) ancestors.add(ancestor);
            let common = node.parentElement; while (common && !ancestors.has(common)) common = common.parentElement;
            if (!common) throw new Error('Original text occurrences lack their actual common containing element');
            paragraph.connections.push({ preceding: index - 1, following: index, kind: 'continuous',
              precedingWrap: paragraph.origins[index - 1].style.wrap, followingWrap: style.wrap,
              ancestorWrap: capturedTextStyle(common).wrap });
          }
          precedingTextNode = node;
          paragraph.order.push({ kind: 'origin', index, affinity: 'before' });
          sourceEnd += units.length;
          const range = document.createRange(); range.selectNodeContents(node);
          const rootRect = preparedRangeRootRect(root) ?? root.getBoundingClientRect();
          const rangeRectangles = Array.from(range.getClientRects()).map(rect => ({
            origin: index, line: layoutReadyRangeLineIndex(root, rect),
            physicalStartEdge: vertical ? 'top' : 'left',
            start: vertical ? rect.top - rootRect.top : rect.left - rootRect.left,
            advance: vertical ? rect.height : rect.width,
          }));
          range.detach();
          children.push({ layoutInput: 'prepared-text', preparedOccurrence: index, rangeFragments: rangeRectangles, children: [] });
        } else if (node.nodeType === Node.ELEMENT_NODE) {
          const style = getComputedStyle(node);
          if (node.tagName === 'BR') {
            qualifyPreparedBreak(node, frame, paragraph.baseStyle);
            const index = paragraph.controls.length;
            paragraph.controls.push({ child, at: sourceEnd, kind: 'forced-break' });
            paragraph.order.push({ kind: 'control', index, affinity: 'before' });
            const measured = describeElement(node);
            if (measured.style) delete measured.style.inlineMetrics;
            children.push(measured);
          } else if (['inline-block', 'inline-grid', 'inline-grid-lanes'].includes(style.display)) {
            if (vertical) qualifyPreparedFrame(node, frame);
            if (vertical && (style.display !== 'inline-block' || style.cssFloat !== 'none' ||
                ['absolute', 'fixed'].includes(style.position) ||
                Array.from(node.childNodes).some(child => child.nodeType !== Node.COMMENT_NODE))) {
              throw new Error('Vertical prepared atomics require same-frame empty in-flow inline-blocks');
            }
            const width = /^\d+(?:\.\d+)?px$/.test(node.style.width) ? parseCssPx(node.style.width) : NaN;
            const height = /^\d+(?:\.\d+)?px$/.test(node.style.height) ? parseCssPx(node.style.height) : NaN;
            if (![width, height].every(Number.isFinite) || node.textContent.length !== 0) throw new Error('Prepared atomic profile needs explicit finite pixel input dimensions and no unbound text subtree');
            const index = paragraph.atomics.length;
            const wrap = capturedTextStyle(element).wrap;
            paragraph.atomics.push({ child, at: sourceEnd, kind: style.position === 'absolute' ? 'out-of-flow' : 'in-flow', width, height,
              precedingWrap: wrap, followingWrap: wrap, ancestorWrap: wrap });
            paragraph.order.push({ kind: 'atomic', index, affinity: 'before' });
            const measured = describeElement(node); delete measured.atomicInlineParticipation;
            children.push(measured);
          } else if (style.display === 'inline' && node.getAttribute('data-surgeist-transparent-inline-container') === 'true') {
            if (vertical) throw new Error('Vertical prepared profile does not admit independent inline boundary controls');
            qualifyPreparedFrame(node, frame);
            qualifyTransparentPreparedInline(node, element);
            // A qualified transparent boundary has no independent strut. Source
            // continuity and style are captured from its actual text children.
            for (const kind of ['inline-boundary-start', 'inline-boundary-end']) {
              if (kind.endsWith('end')) capture(node);
              const index = paragraph.controls.length;
              paragraph.controls.push({ child: children.length, at: sourceEnd, kind });
              paragraph.order.push({ kind: 'control', index, affinity: 'before' });
              children.push({ layoutInput: 'inline-boundary', inlineBoundary: {kind: kind.endsWith('start') ? 'start' : 'end'}, children: [] });
            }
          } else throw new Error('Prepared text encountered an unsupported actual inline participant');
        } else throw new Error('Prepared text encountered an unsupported DOM node kind');
      }
    };
    capture(root);
    // The initial atomic profile captures one uniform containing wrap policy.
    // Mixed inline styles remain explicit profile refusals, never guessed NCA facts.
    if (paragraph.atomics.length && paragraph.origins.some(origin => origin.style.wrap !== paragraph.baseStyle.wrap)) {
      throw new Error('Atomic capture requires the actual uniform containing wrap profile');
    }
    return { paragraph, children };
  } catch (error) {
    return { children: [], unsupportedReason: `Prepared text profile: ${error.message}` };
  }
}

function unsupportedTestData(reason) {
  return { layoutInput: 'unsupported', unsupportedReason: reason };
}

function getTestData() {
  const root = document.getElementById('test-root');
  if (!root) {
    const reason = "Unsupported missing #test-root fixture root";
    return JSON.stringify({
      schemaVersion: 4,
      borderBoxLtrData: unsupportedTestData(reason),
      contentBoxLtrData: unsupportedTestData(reason),
      borderBoxRtlData: unsupportedTestData(reason),
      contentBoxRtlData: unsupportedTestData(reason),
    });
  }

  const borderBoxLtrData = captureRootVariant(root, "border-box ltr");
  const contentBoxLtrData = captureRootVariant(root, "content-box ltr");
  const borderBoxRtlData = captureRootVariant(root, "border-box rtl");
  const contentBoxRtlData = captureRootVariant(root, "content-box rtl");

  return JSON.stringify({ schemaVersion: 4, borderBoxLtrData, contentBoxLtrData, borderBoxRtlData, contentBoxRtlData });
}

// Useful when developing this script. Logs the parsed style to the console when any test fixture is opened in a browser.
window.onload = function () {
  try {
    console.log(describeElement(document.getElementById('test-root')));
  } catch (e) {
    console.error(e);
  }
};
