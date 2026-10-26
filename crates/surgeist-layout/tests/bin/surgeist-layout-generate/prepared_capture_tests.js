// Actual bundled producer test. DOM style facts are the focused test double;
// this is schema/profile qualification, not a browser geometry or font-load test.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const path = require('node:path');
const input = process.argv[2]
  ? { helperSource: fs.readFileSync(process.argv[2], 'utf8'), fontBytes: Array.from(fs.readFileSync(path.join(path.dirname(process.argv[2]), '../../fonts/Ahem.ttf'))) }
  : JSON.parse(fs.readFileSync(0, 'utf8'));

function capture(changes, nonempty) {
  const style = {
    fontFamily: 'SurgeistParityAhem', fontStyle: 'normal', fontWeight: '400',
    fontStretch: '100%', fontFeatureSettings: 'normal', fontVariationSettings: 'normal',
    fontSynthesis: 'none', writingMode: 'horizontal-tb', unicodeBidi: 'normal',
    fontSize: '16px', lineHeight: '20px', whiteSpace: 'normal', wordBreak: 'normal',
    overflowWrap: 'normal', letterSpacing: 'normal', wordSpacing: 'normal',
    textTransform: 'none', textDecorationLine: 'none', fontVariant: 'normal',
    hyphens: 'manual', color: 'rgb(0, 128, 0)', direction: 'ltr',
    paddingTop: '0px', paddingRight: '0px', paddingBottom: '0px', paddingLeft: '0px',
    marginTop: '0px', marginRight: '0px', marginBottom: '0px', marginLeft: '0px',
    borderTopWidth: '0px', borderRightWidth: '0px', borderBottomWidth: '0px', borderLeftWidth: '0px',
    position: 'relative', top: 'auto', right: 'auto', bottom: 'auto', left: 'auto',
    verticalAlign: 'baseline', cssFloat: 'none', transform: 'none',
  };
  const root = { childNodes: [], computed: { ...style, display: 'block' }, closest: () => null,
    getAttribute: name => name === 'data-surgeist-layout-ready-inline' ? 'true' : null,
    getBoundingClientRect: () => ({ left: 0, top: 0 }) };
  const span = {
    nodeType: 1, tagName: 'SPAN', childNodes: [], style: {}, parentElement: root,
    getAttribute: name => name === 'data-surgeist-transparent-inline-container' ? 'true' : null,
    closest: () => null, computed: { ...style, display: 'inline', ...changes },
  };
  if (nonempty) span.childNodes.push({ nodeType: 3, textContent: 'X', parentElement: span });
  root.childNodes.push(span);
  const context = vm.createContext({
    window: {}, Node: { COMMENT_NODE: 8, TEXT_NODE: 3, ELEMENT_NODE: 1 },
    getComputedStyle: element => element.computed, root, fontBytes: input.fontBytes,
    document: { createRange: () => ({ selectNodeContents() {}, getClientRects: () => [{ left: 15, top: 0, width: 16 }], detach() {} }) },
  });
  vm.runInContext(input.helperSource, context);
  vm.runInContext("pinnedTextFontState = { face: { status: 'loaded' }, bytes: Uint8Array.from(fontBytes) };", context);
  return JSON.parse(JSON.stringify(vm.runInContext('capturePreparedChildren(root)', context)));
}

const cases = [
  ['plain inherited metrics and unshifted relative position', {}, true],
  ['own independent strut', { fontSize: '32px', lineHeight: '40px' }, false],
  ['padding', { paddingLeft: '10px', paddingRight: '10px' }, false],
  ['margin', { marginLeft: '3px' }, false],
  ['border', { borderLeftWidth: '2px' }, false],
  ['vertical alignment', { verticalAlign: 'top' }, false],
  ['relative displacement', { left: '2px' }, false],
  ['bidi context', { unicodeBidi: 'bidi-override' }, false],
  ['unrepresented font features', { fontFeatureSettings: '"kern" 0' }, false],
];
const observed = [false,true].flatMap(nonempty => cases.map(([name, changes, allowed]) => {
  const result = capture(changes, nonempty);
  return { name: `${nonempty ? 'text' : 'empty'}: ${name}`, nonempty, expected: allowed, admitted: result.unsupportedReason === undefined, result };
}));
process.stdout.write(JSON.stringify(observed, null, 2) + '\n');
for (const row of observed) {
  assert.equal(row.admitted, row.expected, row.name);
  if (!row.expected) {
    assert.match(row.result.unsupportedReason, /^Prepared text profile:/);
    assert.equal(row.result.children.length, 0);
    assert.equal(row.result.paragraph, undefined);
  } else {
    assert.equal(row.result.paragraph.origins.length, row.nonempty ? 1 : 0);
    if (row.nonempty) {
      assert.deepEqual(row.result.paragraph.origins[0].units, [88]);
      assert.equal(row.result.paragraph.origins[0].child, 1);
      assert.equal(row.result.children[1].rangeFragments[0].start, 15);
    }
    assert.deepEqual(row.result.paragraph.controls.map(control => control.kind),
      ['inline-boundary-start', 'inline-boundary-end']);
    assert.equal(row.result.children.length, row.nonempty ? 3 : 2);
  }
}
