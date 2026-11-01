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


// These are exact original occurrence/style facts and controlled DOM query
// replies. They qualify the bundled producer, not browser layout, font loading
// or the native font-strut implementation. Font coverage reads the real bytes.
function atomicBreakCapture(spec) {
  const base = {
    fontFamily: 'SurgeistParityAhem', fontStyle: 'normal', fontWeight: '400',
    fontStretch: '100%', fontFeatureSettings: 'normal', fontVariationSettings: 'normal',
    fontSynthesis: 'none', fontKerning: 'auto', writingMode: spec.mode,
    textOrientation: 'mixed', textCombineUpright: 'none', unicodeBidi: 'normal',
    fontSize: '20px', lineHeight: '30px', whiteSpace: 'normal', wordBreak: 'normal',
    overflowWrap: 'normal', letterSpacing: 'normal', wordSpacing: 'normal',
    textTransform: 'none', textDecorationLine: 'none', fontVariant: 'normal',
    hyphens: 'manual', color: 'rgb(0, 128, 0)', direction: spec.direction,
    paddingTop: '0px', paddingRight: '0px', paddingBottom: '0px', paddingLeft: '0px',
    marginTop: '0px', marginRight: '0px', marginBottom: '0px', marginLeft: '0px',
    borderTopWidth: '0px', borderRightWidth: '0px', borderBottomWidth: '0px', borderLeftWidth: '0px',
    boxSizing: 'border-box', position: 'relative', top: 'auto', right: 'auto', bottom: 'auto', left: 'auto',
    verticalAlign: 'baseline', cssFloat: 'none', clear: 'none', transform: 'none',
    overflowX: 'visible', overflowY: 'visible', gridTemplateAreas: 'none',
    minWidth: '0px', minHeight: '0px', maxWidth: 'none', maxHeight: 'none',
  };
  const rect = (left, top, width, height) => ({ x:left,y:top,left,top,width,height,right:left+width,bottom:top+height });
  const declaration = values => new Proxy({ length:0,...values }, {
    get(target,key) { return Reflect.has(target,key) ? Reflect.get(target,key) : ''; },
  });
  const language = { getAttribute:name => name==='lang' ? 'en' : null };
  const reads = [];
  function element(tag, authored={}, changes={}, geometry=rect(0,0,0,0)) {
    const node = {
      nodeType:1,tagName:tag,style:declaration(authored),childNodes:[],attributes:{},
      computed:{...base,display:'inline',...changes},geometry,
      classList:{ contains:()=>false },clientLeft:0,clientTop:0,
      closest:selector => selector==='[lang]' ? language : null,
      getAttribute(name) { return this.attributes[name] ?? null; },
      matches:()=>false,
      append(...children) { for(const child of children) this.appendChild(child); },
      appendChild(child) { child.parentElement=this;child.parentNode=this;this.childNodes.push(child);return child; },
      remove() { const parent=this.parentNode;if(parent) parent.childNodes=parent.childNodes.filter(child=>child!==this); },
      getBoundingClientRect() {
        // Legacy BR measurement still probes markers before discarding numeric
        // metrics in a prepared capture. These replies merely satisfy that DOM
        // dependency; no baseline probe becomes a captured Source input.
        if(this.style.verticalAlign==='top') return rect(0,0,0,0);
        if(this.style.verticalAlign==='baseline') {
          const vertical=this.parentNode?.style.writingMode!=='horizontal-tb';
          return vertical ? rect(15,0,0,0) : rect(0,21,0,0);
        }
        return this.geometry;
      },
    };
    Object.defineProperties(node,{
      textContent:{get(){return this.childNodes.map(child=>child.textContent).join('');}},
      childElementCount:{get(){return this.childNodes.filter(child=>child.nodeType===1).length;}},
      offsetWidth:{get(){return this.geometry.width;}},offsetHeight:{get(){return this.geometry.height;}},
      clientWidth:{get(){return this.geometry.width;}},clientHeight:{get(){return this.geometry.height;}},
      scrollWidth:{get(){return this.geometry.width;}},scrollHeight:{get(){return this.geometry.height;}},
      offsetLeft:{get(){return this.geometry.left-(this.parentNode?.geometry.left??0);}},offsetTop:{get(){return this.geometry.top-(this.parentNode?.geometry.top??0);}},
    });
    node.computed.font = 'normal 400 '+node.computed.fontSize+' / '+node.computed.lineHeight+' '+node.computed.fontFamily;
    return node;
  }
  const body=element('BODY');body.className=spec.bodyClass ?? 'border-box ltr';
  const vertical=spec.mode!=='horizontal-tb';
  const rootRect=rect(100,200,vertical?80:120,vertical?30:60);
  const root=element('DIV', {display:'block',width:vertical?'80px':'120px'},
    {display:'block',...spec.rootChanges},rootRect);
  root.attributes['data-surgeist-layout-ready-inline']='true';
  root.attributes['data-surgeist-prepared-text']='ahem-ttf-v1';
  body.append(root);
  const size = vertical ? [[30,30],[30,16]] : [[10,10],[10,10]];
  const atomic = index => {
    const display=index===0?(spec.atomicDisplay ?? 'inline-block'):'inline-block';
    const width=size[index][0]+'px',height=size[index][1]+'px';
    return element('SPAN',{display,width,height},
      {display,width,height,...(index===0?spec.atomicChanges:{})});
  };
  const before=atomic(0),after=atomic(1);
  if(spec.atomicText) before.append(text(spec.atomicText,rect(100,200,10,20)));
  const br=element('BR',{}, {position:'static',...spec.brChanges});
  // Consistent empty-box DOM observations; only authored dimensions enter the
  // Source atomics below. These replies are not a native layout oracle.
  if(vertical) {
    const rightward=spec.mode==='vertical-lr';
    before.geometry=rect(100+(rightward?0:50),200,30,30);
    after.geometry=rect(100+(rightward?30:20),200+(spec.direction==='rtl'?14:0),30,16);
    br.geometry=rect(100+(rightward?5:55),200+(spec.direction==='rtl'?0:30),20,0);
  } else {
    const start=spec.direction==='rtl'?110:0;
    before.geometry=rect(100+start,211,10,10);
    after.geometry=rect(100+start,241,10,10);
    br.geometry=rect(100+(spec.direction==='rtl'?110:10),205,0,20);
  }
  function text(value,observation) {
    return {nodeType:3,textContent:value,observation};
  }
  // Literal source from the actual vertical HTML, including indentation. The
  // asymmetric zero-inline Range replies test axis transport even if a real
  // browser later supplies an empty list for these collapsed source nodes.
  const prefix=spec.visible ? 'X' : (spec.prefix ?? '\n  ');
  const suffix=spec.suffix ?? '\n';
  if(spec.whitespace || spec.visible) root.append(text(prefix,rect(103,207,20,0)));
  root.append(before,br,after);
  if(spec.whitespace || spec.visible) root.append(text(suffix,rect(143,219,20,0)));
  const document={
    body,styleSheets:[],createElement:tag=>element(tag.toUpperCase()),
    createRange() {
      let selected;
      return {
        selectNodeContents(node) { selected=node; },
        getClientRects() {
          reads.push(Array.from(selected.textContent,character=>character.charCodeAt(0)));
          return [selected.observation];
        },
        detach() {},
      };
    },
  };
  const context=vm.createContext({
    window:{innerWidth:800},Node:{COMMENT_NODE:8,TEXT_NODE:3,ELEMENT_NODE:1},
    CSSRule:{STYLE_RULE:1},document,root,fontBytes:input.fontBytes,
    getComputedStyle:node=>node.computed,
  });
  vm.runInContext(input.helperSource,context);
  // Admission precondition only. Actual FontFace loading is qualified by the
  // existing adapter/browser boundary, never by this focused DOM recorder.
  vm.runInContext("pinnedTextFontState={face:{status:'loaded'},bytes:Uint8Array.from(fontBytes)}",context);
  const captured=JSON.parse(JSON.stringify(vm.runInContext('capturePreparedChildren(root)',context)));
  return {captured,reads};
}

function atomicBreakProfileTests() {
  const accepted=[
    {name:'horizontal original LTR has genuinely zero origins',mode:'horizontal-tb',direction:'ltr'},
    {name:'horizontal original RTL has genuinely zero origins',mode:'horizontal-tb',direction:'rtl'},
    {name:'vertical-lr actual LTR survives RTL variant class',mode:'vertical-lr',direction:'ltr',whitespace:true,bodyClass:'border-box rtl'},
    {name:'vertical-rl actual LTR survives RTL variant class',mode:'vertical-rl',direction:'ltr',whitespace:true,bodyClass:'border-box rtl'},
    {name:'vertical-rl actual RTL survives LTR variant class',mode:'vertical-rl',direction:'rtl',whitespace:true,bodyClass:'border-box ltr'},
    {name:'vertical original CR and SPACE are retained source units',mode:'vertical-lr',direction:'ltr',whitespace:true,prefix:'\r ',suffix:'\r'},
  ];
  const refused=[
    {name:'whole-upright is not the Mixed source profile',mode:'vertical-rl',direction:'rtl',whitespace:true,rootChanges:{textOrientation:'upright'}},
    {name:'sideways is outside the three captured modes',mode:'sideways-rl',direction:'ltr',whitespace:true},
    {name:'visible vertical glyph text is not qualified by whitespace support',mode:'vertical-rl',direction:'ltr',visible:true},
    {name:'out-of-flow blockified atomic is outside the vertical profile',mode:'vertical-rl',direction:'ltr',whitespace:true,atomicChanges:{position:'absolute',display:'block'}},
    {name:'floated blockified atomic is outside the vertical profile',mode:'vertical-rl',direction:'ltr',whitespace:true,atomicChanges:{cssFloat:'left',display:'block'}},
    {name:'vertical inline-grid is outside the empty inline-block profile',mode:'vertical-rl',direction:'ltr',whitespace:true,atomicDisplay:'inline-grid'},
    {name:'vertical atomic content is not an empty original box',mode:'vertical-rl',direction:'ltr',whitespace:true,atomicText:'X'},
    {name:'atomic child has a foreign orientation',mode:'vertical-rl',direction:'ltr',whitespace:true,atomicChanges:{textOrientation:'upright'}},
    {name:'atomic child has a foreign writing mode',mode:'vertical-rl',direction:'ltr',whitespace:true,atomicChanges:{writingMode:'horizontal-tb'}},
    {name:'BR has a foreign writing mode',mode:'vertical-rl',direction:'ltr',whitespace:true,brChanges:{writingMode:'horizontal-tb'}},
    {name:'BR has a foreign orientation',mode:'vertical-rl',direction:'ltr',whitespace:true,brChanges:{textOrientation:'upright'}},
    {name:'BR has an independent font size',mode:'horizontal-tb',direction:'ltr',brChanges:{fontSize:'40px'}},
    {name:'BR has an independent line height',mode:'horizontal-tb',direction:'ltr',brChanges:{lineHeight:'45px'}},
    {name:'BR has a different face',mode:'horizontal-tb',direction:'ltr',brChanges:{fontFamily:'UnregisteredOtherFont'}},
    {name:'BR alignment is not baseline',mode:'vertical-rl',direction:'ltr',whitespace:true,brChanges:{verticalAlign:'top'}},
    {name:'BR has actual clearance',mode:'vertical-lr',direction:'ltr',whitespace:true,brChanges:{clear:'both'}},
    {name:'BR has actual relative displacement',mode:'vertical-rl',direction:'ltr',whitespace:true,brChanges:{position:'relative',left:'2px'}},
    {name:'BR has an actual transform',mode:'vertical-rl',direction:'ltr',whitespace:true,brChanges:{transform:'translateX(2px)'}},
  ];
  const rows=[...accepted.map(spec=>({spec,expected:true})),...refused.map(spec=>({spec,expected:false}))]
    .map(row=>({...row,...atomicBreakCapture(row.spec)}));
  process.stdout.write(JSON.stringify(rows.map(row=>({
    name:row.spec.name,expected:row.expected,admitted:row.captured.unsupportedReason===undefined,
    writingMode:row.captured.paragraph?.writingMode,direction:row.captured.paragraph?.direction,
    unsupportedReason:row.captured.unsupportedReason,
  })),null,2)+'\n');
  for(const {spec,expected,captured,reads} of rows) {
    assert.equal(captured.unsupportedReason===undefined,expected,spec.name+': '+captured.unsupportedReason);
    if(!expected) {
      assert.match(captured.unsupportedReason,/^Prepared text profile:/,spec.name);
      assert.equal(captured.paragraph,undefined,spec.name);
      assert.deepEqual(captured.children,[],spec.name);
      continue;
    }
    const p=captured.paragraph;
    assert.equal(p.writingMode,spec.mode,spec.name);
    assert.equal(p.direction,spec.direction,spec.name);
    assert.equal(p.font.sha256,'f0a92cd0cc45735591c9b5b1fa8aecd5194e8dc518895ca22af94a46c23550dc');
    assert.equal(p.font.faceIndex,0);
    assert.equal(p.baseStyle.fontSize,20);
    assert.equal(p.baseStyle.lineHeight,30);
    assert.equal(p.baseStyle.whitespace,'collapse');
    assert.equal(p.baseStyle.wrap,'word');
    const expectedPrefix=spec.prefix===undefined?[10,32,32]:[13,32];
    const expectedSuffix=spec.suffix===undefined?[10]:[13];
    const start=spec.whitespace?expectedPrefix.length:0;
    const shift=spec.whitespace?1:0;
    assert.deepEqual(p.atomics,[
      {child:shift,at:start,kind:'in-flow',width:spec.whitespace?30:10,height:spec.whitespace?30:10,precedingWrap:'word',followingWrap:'word',ancestorWrap:'word'},
      {child:shift+2,at:start,kind:'in-flow',width:spec.whitespace?30:10,height:spec.whitespace?16:10,precedingWrap:'word',followingWrap:'word',ancestorWrap:'word'},
    ],spec.name);
    assert.deepEqual(p.controls,[{child:shift+1,at:start,kind:'forced-break'}],spec.name);
    assert.deepEqual(p.order,spec.whitespace?[
      {kind:'origin',index:0,affinity:'before'},{kind:'atomic',index:0,affinity:'before'},
      {kind:'control',index:0,affinity:'before'},{kind:'atomic',index:1,affinity:'before'},
      {kind:'origin',index:1,affinity:'before'},
    ]:[
      {kind:'atomic',index:0,affinity:'before'},{kind:'control',index:0,affinity:'before'},
      {kind:'atomic',index:1,affinity:'before'},
    ],spec.name);
    const control=captured.children[shift+1];
    assert.equal(control.tagName,'br');
    assert.deepEqual(control.lineControlParticipation,{kind:'forced-break'});
    assert.equal(control.style.inlineMetrics,undefined,'a browser baseline must not become a prepared BR input');
    for(const index of [shift,shift+2]) assert.equal(captured.children[index].atomicInlineParticipation,undefined);
    if(spec.whitespace) {
      assert.deepEqual(p.origins.map(origin=>({child:origin.child,units:origin.units,start:origin.start,end:origin.end})),[
        {child:0,units:expectedPrefix,start:0,end:start},{child:4,units:expectedSuffix,start,end:start+expectedSuffix.length},
      ],spec.name);
      assert.deepEqual(p.connections,[{preceding:0,following:1,kind:'continuous',precedingWrap:'word',followingWrap:'word',ancestorWrap:'word'}]);
      assert.deepEqual(reads,[expectedPrefix,expectedSuffix],spec.name);
      assert.deepEqual(captured.children[0].rangeFragments,[{origin:0,line:0,physicalStartEdge:'top',start:7,advance:0}],spec.name);
      assert.deepEqual(captured.children[4].rangeFragments,[{origin:1,line:1,physicalStartEdge:'top',start:19,advance:0}],spec.name);
    } else {
      assert.deepEqual(p.origins,[],spec.name);
      assert.deepEqual(p.connections,[],spec.name);
      assert.deepEqual(reads,[],spec.name);
    }
  }
}

if(input.suite==='atomic-br') {
  atomicBreakProfileTests();
} else {
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

}
