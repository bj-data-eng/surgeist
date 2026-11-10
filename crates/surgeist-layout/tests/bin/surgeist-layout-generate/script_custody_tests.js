// Execute the actual script-profile validators with controlled DOM inventory
// facts. This does not qualify browser parsing, execution, layout or recovery.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const helperSource = process.argv[2]
  ? fs.readFileSync(process.argv[2], 'utf8')
  : JSON.parse(fs.readFileSync(0, 'utf8')).helperSource;
const htmlNamespace = 'http://www.w3.org/1999/xhtml';
let scriptNodes = [];
const document = {
  baseURI: 'file:///corpus/html/flex/fixture.html', head: {},
  get scripts() { return scriptNodes.filter(node => node.namespaceURI === htmlNamespace); },
  querySelectorAll(selector) {
    assert.equal(selector, 'script');
    return scriptNodes.slice();
  },
};
const context = vm.createContext({ window: {}, document, URL });
vm.runInContext(helperSource, context);
const { rootContextSourceScriptProfile: sourceProfile, rootContextScriptInventory: inventory } = context;
const helperUrl = 'file:///corpus/scripts/gentest/test_helper.js';
const reference = '../../scripts/gentest/test_helper.js';
const sourceHead = {};
function script(parent = sourceHead, namespaceURI = htmlNamespace) {
  return {
    namespaceURI,
    parentElement: parent,
    isConnected: true,
    attributes: [{ name: 'src', value: reference }],
    textContent: '',
    getAttribute(name) { return this.attributes.find(attribute => attribute.name === name)?.value ?? null; },
    get src() { return new URL(this.getAttribute('src'), document.baseURI).href; },
  };
}
function source(scripts) {
  return { head: sourceHead, querySelectorAll(selector) {
    assert.equal(selector, 'script');
    return scripts;
  } };
}
const absent = sourceProfile(source([]), helperUrl);
assert.equal(absent.kind, 'absent');
assert.equal(inventory(absent, null), true);
const original = script();
const helper = sourceProfile(source([original]), helperUrl);
assert.equal(helper.kind, 'helper');
assert.equal(sourceProfile(source([original, script()]), helperUrl), null);
for (const name of ['type', 'async', 'defer', 'nomodule', 'crossorigin', 'integrity', 'onload']) {
  const changed = script();
  changed.attributes.push({ name, value: name === 'type' ? 'application/json' : '' });
  assert.equal(sourceProfile(source([changed]), helperUrl), null, name);
}
for (const reference of ['', '../../foreign.js', 'https://example.invalid/helper.js', 'http://[']) {
  const changed = script();
  changed.attributes[0].value = reference;
  assert.equal(sourceProfile(source([changed]), helperUrl), null, reference);
}
const inline = script();
inline.textContent = 'globalThis.changed = true;';
assert.equal(sourceProfile(source([inline]), helperUrl), null);
assert.equal(sourceProfile(source([script({})]), helperUrl), null, 'non-head source owner');

const owner = script(document.head);
scriptNodes = [owner];
assert.equal(inventory(helper, owner), true);
assert.equal(inventory(absent, null), false, 'absent source cannot gain the exact helper');
const svg = script(document.head, 'http://www.w3.org/2000/svg');
scriptNodes = [svg];
assert.equal(document.scripts.length, 0);
assert.equal(document.querySelectorAll('script').length, 1);
assert.equal(inventory(absent, null), false, 'SVG script violates source script absence');
scriptNodes = [owner, svg];
assert.deepEqual(document.scripts, [owner]);
assert.equal(document.querySelectorAll('script').length, 2);
assert.equal(inventory(helper, owner), false, 'SVG script violates exact helper census');
const clone = script(document.head);
assert.equal(clone.src, owner.src);
assert.equal(clone.textContent, owner.textContent);
assert.deepEqual(clone.attributes, owner.attributes);
scriptNodes = [clone];
assert.equal(inventory(helper, owner), false, 'identical clone is a different owner');
scriptNodes = [owner, clone];
assert.equal(inventory(helper, owner), false, 'extra helper');
scriptNodes = [];
assert.equal(inventory(helper, owner), false, 'missing helper');
scriptNodes = [owner];
owner.attributes.push({ name: 'type', value: 'application/json' });
assert.equal(inventory(helper, owner), false, 'changed live attributes');
owner.attributes.pop();
owner.textContent = ' ';
assert.equal(inventory(helper, owner), false, 'changed whitespace is changed source text');
owner.textContent = '';
owner.attributes[0].value = helperUrl;
assert.equal(owner.src, helperUrl);
assert.equal(inventory(helper, owner), false, 'same URL does not preserve the source attribute');
owner.attributes[0].value = reference;
document.baseURI = 'file:///different/html/flex/fixture.html';
assert.equal(inventory(helper, owner), false, 'changed resolved URL');
document.baseURI = 'file:///corpus/html/flex/fixture.html';
owner.parentElement = {};
assert.equal(inventory(helper, owner), false, 'changed head ownership');
owner.parentElement = document.head;
owner.isConnected = false;
assert.equal(inventory(helper, owner), false, 'detached owner');
owner.isConnected = true;
assert.equal(inventory(helper, owner), true, 'original inventory restored');
console.log(JSON.stringify({ sourceProfiles: ['absent', 'helper'], restored: true }));
