
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

function parseViewportConstraint(e, boundingRect) {
  if (e.parentNode.classList.contains('viewport')) {
    const parentStyle = getComputedStyle(e.parentElement);
    const hostInlineSize = parentStyle.writingMode === 'horizontal-tb'
      ? boundingRect.width
      : boundingRect.height;
    return {
      width: parseDimension(e.parentNode.style.width || 'max-content'),
      height: parseDimension(e.parentNode.style.height || 'max-content'),
      rootContext: 'flex-item',
      parentWritingMode: parentStyle.writingMode,
      parentDirection: parentStyle.direction,
      hostInlineSize,
    }
  } else {
    return {
      width: rootFillsBrowserViewport(e, boundingRect) ? px(window.innerWidth) : { unit: 'max-content' },
      height: { unit: 'max-content' },
      rootContext: 'root',
    }
  }
}

function rootFillsBrowserViewport(e, boundingRect) {
  if (e.style.width) return false;
  if (e.style.display === 'inline-grid' || e.style.display === 'inline-flex' || e.style.display === 'inline-block') return false;
  return Math.round(boundingRect.width) === window.innerWidth;
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
  const autoEdges = inlineAutoMarginEdges(e, computedStyle);
  const authoredEdges = authoredMarginEdges(e, computedStyle);
  if (!hasAuthoredMarginDeclaration(e, computedStyle) && !Object.values(autoEdges).some(Boolean)) return undefined;

  return parseEdges({
    left: effectiveMarginValue(authoredEdges.left, computedStyle.marginLeft, autoEdges.left),
    right: effectiveMarginValue(authoredEdges.right, computedStyle.marginRight, autoEdges.right),
    top: effectiveMarginValue(authoredEdges.top, computedStyle.marginTop, autoEdges.top),
    bottom: effectiveMarginValue(authoredEdges.bottom, computedStyle.marginBottom, autoEdges.bottom),
  });
}

function effectiveMarginValue(authoredValue, computedValue, isAuto) {
  if (isAuto) return "auto";
  if (authoredValue.trim() !== "auto" && marginValueIsNonInitial(authoredValue)) return authoredValue;
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

function parseElementSize(styleValue, computedStyle) {
  const width = styleValue("width");
  const height = styleValue("height");
  const inlineSize = styleValue("inlineSize");
  const blockSize = styleValue("blockSize");

  if (isVerticalWritingMode(computedStyle.writingMode)) {
    return parseSize({ width: width || blockSize, height: height || inlineSize });
  }
  return parseSize({ width: width || inlineSize, height: height || blockSize });
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
    const gaps = splitCssComponentValues(gap).map(part => parseDimension(part));
    return { row: gaps[0], column: gaps[1] ?? gaps[0] };
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
  const layoutReadyInlineRoot = e.getAttribute?.('data-surgeist-layout-ready-inline') === 'true';
  if (layoutReadyInlineRoot) resetLayoutReadyRangeLineRegistry(e);
  const preparedCapture = e.getAttribute?.('data-surgeist-prepared-text') === 'ahem-ttf-v1'
    ? capturePreparedChildren(e) : undefined;
  const children = preparedCapture ? preparedCapture.children : describeChildNodes(e, expectedElement);
  const brInlineMetrics = brInlineMetricsForElement(e, computedStyle);
  const lineControlParticipation = layoutReadyLineControlParticipation(e, computedStyle);
  const hasTypedInlineText = children.some((child) => ['inline-text', 'prepared-text'].includes(child.layoutInput));

  return {
    layoutInput: 'box',
    preparedParagraph: preparedCapture?.paragraph,
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
      flexBasis: parseSizingDimension(lengthStyleValue("flexBasis")),

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

      gap: parseGaps(lengthStyleValue),

      size: parseElementSize(lengthStyleValue, computedStyle),
      minSize: parseSize({
        width: parseResolvedDimension(lengthStyleValue("minWidth"), computedStyle.minWidth),
        height: parseResolvedDimension(lengthStyleValue("minHeight"), computedStyle.minHeight),
      }),
      maxSize: parseSize({
        width: parseResolvedDimension(lengthStyleValue("maxWidth"), computedStyle.maxWidth),
        height: parseResolvedDimension(lengthStyleValue("maxHeight"), computedStyle.maxHeight),
      }),
      aspectRatio: parseRatio(styleValue("aspectRatio")),

      margin: parseEffectiveMargin(e, computedStyle),

      padding: parseEdges({
        left: lengthStyleValue("paddingLeft"),
        right: lengthStyleValue("paddingRight"),
        top: lengthStyleValue("paddingTop"),
        bottom: lengthStyleValue("paddingBottom"),
      }),

      border: parseEdges({
        left: lengthStyleValue("borderLeftWidth"),
        right: lengthStyleValue("borderRightWidth"),
        top: lengthStyleValue("borderTopWidth"),
        bottom: lengthStyleValue("borderBottomWidth"),
      }),

      inset: parseEdges({
        left: lengthStyleValue("left"),
        right: lengthStyleValue("right"),
        top: lengthStyleValue("top"),
        bottom: lengthStyleValue("bottom"),
      }),
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
      x: boundingRect.x - parentBoundingRect.x,
      y: boundingRect.y - parentBoundingRect.y,
      scrollWidth: e.scrollWidth,
      scrollHeight: e.scrollHeight,
      clientWidth: e.clientWidth,
      clientHeight: e.clientHeight,
    },

    // The naively rounded layout of the node. This is equivalent to calling Math.round() on
    // each value in the unrounded layout individually
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

    // The naive rounding can result in 1px gaps in the layout. Chrome also uses
    // a smarter algorithm, but it doesn't expose the output of that rounding.
    // So we emulate the cumulative edge computation here.
    smartRoundedLayout: {
      width: Math.round(boundingRect.right) - Math.round(boundingRect.left),
      height: Math.round(boundingRect.bottom) - Math.round(boundingRect.top),
      x: Math.round(boundingRect.x - parentBoundingRect.x),
      y: Math.round(boundingRect.y - parentBoundingRect.y),
      scrollWidth: e.scrollWidth,
      scrollHeight: e.scrollHeight,
      clientWidth: e.clientWidth,
      clientHeight: e.clientHeight,
    },

    // Whether the test should enable rounding
    useRounding: e.getAttribute("data-test-rounding") !== "false",

    viewport: parseViewportConstraint(e, boundingRect),

    shapeBands: layoutReadyShapeBands(e),

    children,
  };
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
  for (const property of styleDeclarationProperties(style)) {
    applyInlineAuthoredMarginDeclaration(edges, property, styleDeclarationValue(style, property), computedStyle);
  }
}

function applyInlineAuthoredMarginDeclaration(edges, property, value, computedStyle) {
  if (!marginValueIsNonInitial(value)) return;

  switch (property) {
    case "margin-top":
      edges.top = value;
      return;
    case "margin-right":
      edges.right = value;
      return;
    case "margin-bottom":
      edges.bottom = value;
      return;
    case "margin-left":
      edges.left = value;
      return;
    case "margin-inline-start":
      edges[inlineStartEdge(computedStyle)] = value;
      return;
    case "margin-inline-end":
      edges[inlineEndEdge(computedStyle)] = value;
      return;
    case "margin-inline": {
      const [start, end = start] = splitCssComponentValues(value);
      edges[inlineStartEdge(computedStyle)] = start;
      edges[inlineEndEdge(computedStyle)] = end;
      return;
    }
    case "margin": {
      const parts = splitCssComponentValues(value);
      const [top, right = top, bottom = top, left = right] = parts;
      edges.top = top;
      edges.right = right;
      edges.bottom = bottom;
      edges.left = left;
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

function inlineAutoMarginEdges(e, computedStyle) {
  const edges = { left: false, right: false, top: false, bottom: false };
  applyAutoMarginDeclarations(edges, e.style, computedStyle);
  return edges;
}

function applyAutoMarginDeclarations(edges, style, computedStyle) {
  for (const property of styleDeclarationProperties(style)) {
    applyAutoMarginDeclaration(edges, property, styleDeclarationValue(style, property), computedStyle);
  }
}

function styleDeclarationProperties(style) {
  if (!style || !style.length) return [];
  return Array.from({ length: style.length }, (_, index) => style[index]).filter(Boolean);
}

function styleDeclarationValue(style, property) {
  if (!style) return "";
  return style.getPropertyValue ? style.getPropertyValue(property) : "";
}

function applyAutoMarginDeclaration(edges, property, value, computedStyle) {
  const isAuto = value.trim() === "auto";
  switch (property) {
    case "margin-top":
      edges.top = isAuto;
      return;
    case "margin-right":
      edges.right = isAuto;
      return;
    case "margin-bottom":
      edges.bottom = isAuto;
      return;
    case "margin-left":
      edges.left = isAuto;
      return;
    case "margin-inline-start":
      edges[inlineStartEdge(computedStyle)] = isAuto;
      return;
    case "margin-inline-end":
      edges[inlineEndEdge(computedStyle)] = isAuto;
      return;
    case "margin-inline": {
      const [start, end = start] = splitCssComponentValues(value);
      edges[inlineStartEdge(computedStyle)] = start === "auto";
      edges[inlineEndEdge(computedStyle)] = end === "auto";
      return;
    }
    case "margin": {
      const parts = splitCssComponentValues(value);
      const [top, right = top, bottom = top, left = right] = parts;
      edges.top = top === "auto";
      edges.right = right === "auto";
      edges.bottom = bottom === "auto";
      edges.left = left === "auto";
      return;
    }
  }
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
  const start = inlineStartEdge(computedStyle);
  return { left: "right", right: "left", top: "bottom", bottom: "top" }[start];
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
          bidiLevel: consumeLayoutReadyInlineBidiLevel(inlineBidiLevels, i),
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

function layoutReadyRangeLineIndex(root, fragment) {
  if (root?.getAttribute?.('data-surgeist-layout-ready-inline') !== 'true') {
    throw new Error('Range line identity requires an explicit layout-ready inline root');
  }
  let registry = layoutReadyRangeLineRegistries.get(root);
  if (!registry) {
    resetLayoutReadyRangeLineRegistry(root);
    registry = layoutReadyRangeLineRegistries.get(root);
  }

  const rootRect = root.getBoundingClientRect();
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
      !['static', 'relative'].includes(style.position) ||
      (style.position === 'relative' && ['top', 'right', 'bottom', 'left'].some(edge =>
        style[edge] !== 'auto' && parseCssPx(style[edge]) !== 0))) {
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
      !['static', 'relative'].includes(style.position) ||
      (style.position === 'relative' && ['top', 'right', 'bottom', 'left'].some(edge =>
        style[edge] !== 'auto' && parseCssPx(style[edge]) !== 0))) {
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
          const rootRect = root.getBoundingClientRect();
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
      schemaVersion: 2,
      borderBoxLtrData: unsupportedTestData(reason),
      contentBoxLtrData: unsupportedTestData(reason),
      borderBoxRtlData: unsupportedTestData(reason),
      contentBoxRtlData: unsupportedTestData(reason),
    });
  }

  document.body.className = "border-box ltr";
  const borderBoxLtrData = describeElement(root);
  document.body.className = "content-box ltr";
  const contentBoxLtrData = describeElement(root);
  document.body.className = "border-box rtl";
  const borderBoxRtlData = describeElement(root);
  document.body.className = "content-box rtl";
  const contentBoxRtlData = describeElement(root);

  return JSON.stringify({ schemaVersion: 2, borderBoxLtrData, contentBoxLtrData, borderBoxRtlData, contentBoxRtlData });
}

// Useful when developing this script. Logs the parsed style to the console when any test fixture is opened in a browser.
window.onload = function () {
  try {
    console.log(describeElement(document.getElementById('test-root')));
  } catch (e) {
    console.error(e);
  }
};
