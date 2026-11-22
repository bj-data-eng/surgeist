//! Independent guest oracles over the attached Window and its CSSOM owner.

use super::*;
use surgeist_runtime::style::DomConstructedSheetOwnerInit;

pub(super) fn run() {
    prototype_own_key_order();
    prototype_cycles_are_rejected_by_ordinary_receivers();
    reuse_native_owner();
    indexed_properties_and_rule_custody();
    indexed_list_reflection_and_receivers();
    indexed_definition_converts_without_native_admission();
    constructor_and_mutation_order();
    media_mutations_and_branched_copy();
    constructor_admission_rolls_back_before_publication();
    constructor_conversion_charges_original_host();
}

pub(super) fn prototype_own_key_order() {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    let keys = host
        .in_document(&permit, |scope| {
            let source = r#"(() => {
                return [StyleSheet, CSSStyleSheet, CSSRuleList, MediaList, CSSRule].map(ctor => {
                    const prototype = ctor.prototype;
                    const keys = Reflect.ownKeys(prototype);
                    const strings = keys.filter(key => typeof key === 'string'
                        && (ctor !== MediaList || key !== 'toString'));
                    const symbols = keys.filter(key => typeof key === 'symbol').map(key =>
                        key === Symbol.toStringTag ? 'toStringTag' :
                        key === Symbol.iterator ? 'iterator' : 'unexpected');
                    const stringifier = ctor === MediaList && Object.hasOwn(prototype, 'toString');
                    return ctor.name + ': ' + strings.join(',') + ' / ' + symbols.join(',')
                        + (stringifier ? ' / own toString' : '');
                }).join('\n');
            })()"#;
            let value = scope.evaluate(&source.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    // Web IDL installs the regular attributes, operations and iteration
    // members before constructor. Its stringifier property is specified
    // declaratively, so only its own presence is compared here.
    let expected = [
        "StyleSheet: type,href,media,disabled,constructor / toStringTag",
        "CSSStyleSheet: cssRules,rules,insertRule,deleteRule,constructor / toStringTag",
        "CSSRuleList: length,item,constructor / toStringTag,iterator",
        "MediaList: mediaText,length,item,appendMedium,deleteMedium,constructor / toStringTag,iterator / own toString",
        "CSSRule: cssText,type,constructor / toStringTag",
    ];
    assert_eq!(
        String::from_utf16(&keys)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        expected,
        "installed stylesheet prototype own-key order"
    );
}

pub(super) fn prototype_cycles_are_rejected_by_ordinary_receivers() {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    let outcomes = host
        .in_document(&permit, |scope| {
            let source = r#"
                (() => {
                    const sheet = new CSSStyleSheet();
                    return [["CSSRuleList", sheet.cssRules], ["MediaList", sheet.media]].map(([name, list]) => {
                        const original = Reflect.getPrototypeOf(list);
                        const bridge = Object.create(null);
                        const first = Reflect.setPrototypeOf(list, bridge);
                        const rejected = Reflect.setPrototypeOf(bridge, list);
                        const bridgeUnchanged = Reflect.getPrototypeOf(bridge) === null;

                        // Break a cycle on an affected backend before reading any properties.
                        Reflect.setPrototypeOf(bridge, null);
                        Reflect.setPrototypeOf(list, original);

                        const reverse = Object.create(null);
                        const reverseFirst = Reflect.setPrototypeOf(reverse, list);
                        const reverseRejected = Reflect.setPrototypeOf(list, reverse);
                        const listUnchanged = Reflect.getPrototypeOf(list) === original;
                        Reflect.setPrototypeOf(reverse, null);
                        Reflect.setPrototypeOf(list, original);

                        return name + ":" + [first, rejected, bridgeUnchanged, reverseFirst,
                            reverseRejected, listUnchanged].join(",");
                    }).join(";");
                })()
            "#;
            let value = scope.evaluate(&source.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(
        String::from_utf16(&outcomes).unwrap(),
        concat!(
            "CSSRuleList:true,false,true,true,false,true;",
            "MediaList:true,false,true,true,false,true"
        ),
        "indexed CSSOM lists must reject prototype cycles in either direction"
    );
}

fn reuse_native_owner() {
    let (mut session, top) = setup();
    let style = session.style_execution();
    let hosted = session.hosted_execution();
    let original = style
        .construct_document_sheet(
            &mut session.dom(),
            &hosted,
            top,
            Default::default(),
            DomConstructedSheetOwnerInit {
                label: "native-receipt-before-guest",
                limits: Default::default(),
            },
            &cancel(),
        )
        .unwrap();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    eval(
        &mut host,
        &permit,
        "Number((new CSSStyleSheet()).insertRule('a {}') === 0)",
        1.0,
    );
    let snapshot = original.owner().snapshot().unwrap();
    assert_eq!(snapshot.revision().value(), 3);
    assert!(snapshot.sheets().is_empty());
    assert!(snapshot.sheet_rules(original.sheet()).unwrap().is_empty());
}

pub(super) fn indexed_properties_and_rule_custody() {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    assert_contract(
        &mut host,
        &permit,
        r#"Number((() => {
            const sheet = new CSSStyleSheet();
            sheet.insertRule('a {}');
            sheet.insertRule('b {}', 1);
            const rules = sheet.cssRules;
            const a = rules[0], b = rules[1];
            if (Object.prototype.toString.call(rules) !== '[object CSSRuleList]') return 21;
            if (Object.getPrototypeOf(rules) !== CSSRuleList.prototype) return 22;
            if (rules[Symbol.iterator] !== Array.prototype.values) return 23;
            const spread = [...rules];
            if (rules.length !== 2) return 241;
            if (spread.length !== 2) return 242;
            if (a === undefined) return 2430;
            if (spread[0] === undefined) return 2431;
            if (spread[0] !== a && a !== rules[0]) return 2432;
            if (spread[0] !== a) return 2433;
            if (spread[1] !== b) return 244;
            if (!(a instanceof CSSRule)) return 25;
            if (a.type !== 1) return 26;
            let ruleTextWrites = 0;
            a.cssText = {toString() { ruleTextWrites++; return 'article {}'; }};
            if (ruleTextWrites !== 1 || a.cssText !== 'a { }') return 27;

            const symbols = [Symbol('left'), Symbol('right')];
            for (const key of ['x', '01', '-0', '4294967295', ...symbols]) {
                if (!Reflect.defineProperty(rules, key, {
                    value: key, writable: true, configurable: true, enumerable: true
                })) return 3;
            }
            const keys = Reflect.ownKeys(rules);
            const expected = ['0', '1', 'x', '01', '-0', '4294967295', ...symbols];
            if (keys.length !== expected.length || keys.some((key, i) => key !== expected[i])
                || !('0' in rules) || '2' in rules
                || Reflect.defineProperty(rules, '0', {value: 'override'})
                || Reflect.defineProperty(rules, '2', {value: 'unsupported'})
                || Reflect.set(rules, '0', 'override')
                || Reflect.set(rules, '2', 'unsupported')
                || Reflect.deleteProperty(rules, '0') || !Reflect.deleteProperty(rules, '2')
                || rules[0] !== a || Object.getOwnPropertyDescriptor(rules, '2') !== undefined
                || Reflect.setPrototypeOf(rules, rules)) return 4;

            const inherited = Object.create(CSSRuleList.prototype);
            let receiver;
            Object.defineProperty(inherited, '2', {
                get() { return this; },
                set() { receiver = this; },
            });
            if (!Reflect.setPrototypeOf(rules, inherited) || rules[2] !== rules
                || !Reflect.set(rules, '2', 4) || receiver !== rules
                || rules.item(2) !== null) return 5;

            let getterCalled = false;
            Object.defineProperty(Object.prototype, 'writable', {
                configurable: true, get() { getterCalled = true; return 6; }
            });
            try {
                if (!Reflect.defineProperty(rules, 'safe', {__proto__: null, value: 4})
                    || getterCalled || rules.safe !== 4) return 7;
            } finally { delete Object.prototype.writable; }
            for (const operation of [Object.preventExtensions, Object.freeze, Object.seal]) {
                try { operation(rules); return 8; }
                catch (error) { if (!(error instanceof TypeError)) return 9; }
            }
            try { rules.item(); return 10; }
            catch (error) { if (!(error instanceof TypeError)) return 11; }
            try { CSSRuleList.prototype.item.call(new Proxy(rules, {}), 0); return 12; }
            catch (error) { if (!(error instanceof TypeError)) return 13; }
            sheet.deleteRule(0);
            if (rules[0] !== b || rules[1] !== undefined || rules.item(1) !== null
                || a.cssText !== 'a { }') return 14;
            globalThis.heldList = rules;
            globalThis.heldRule = a;
            return true;
        })())"#,
    );
    // A new SDK entry uses only the retained guest list/rule. The original
    // constructor result has no explicit guest or Rust root in this fixture.
    eval(
        &mut host,
        &permit,
        "Number(heldList.length === 1 && heldList[0].cssText === 'b { }' && heldRule.cssText === 'a { }')",
        1.0,
    );
}

pub(super) fn indexed_list_reflection_and_receivers() {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    assert_contract(
        &mut host,
        &permit,
        r#"(() => {
            const sheet = new CSSStyleSheet({media: 'screen, print'});
            sheet.insertRule('a {}');
            sheet.insertRule('b {}', 1);
            for (const list of [sheet.cssRules, sheet.media]) {
                const proto = Object.getPrototypeOf(list);
                const value = list.item(0);
                const descriptor = Object.getOwnPropertyDescriptor(list, '0');
                if (!descriptor || descriptor.value !== value || descriptor.writable
                    || !descriptor.enumerable || !descriptor.configurable
                    || Object.getOwnPropertyDescriptor(list, '2') !== undefined
                    || Reflect.preventExtensions(list) || !Object.isExtensible(list)) return 2;

                if (Reflect.set({}, '0', 42, list) || Reflect.set({}, '2', 42, list)
                    || Reflect.set(list, '0', 42, {}) || Reflect.set(list, '2', 42)
                    || Reflect.deleteProperty(list, '0') || !Reflect.deleteProperty(list, '2')
                    || !('0' in list) || '2' in list || list[0] !== value) return 3;
                const ordinary = Object.create(list);
                if (!Reflect.set(list, '2', 42, ordinary)
                    || !Object.hasOwn(ordinary, '2') || ordinary[2] !== 42
                    || Reflect.has(list, '2')) return 4;

                const layer = Object.create(proto);
                let received;
                Object.defineProperty(layer, '2', {
                    set(value) { received = [this, value]; }
                });
                const receiver = {};
                if (!Reflect.setPrototypeOf(list, layer) || Object.getPrototypeOf(list) !== layer
                    || !Reflect.set(list, '2', 17, receiver) || received?.[0] !== receiver
                    || received[1] !== 17 || Object.hasOwn(receiver, '2')
                    || !Reflect.setPrototypeOf(list, proto)) return 5;

                const symbol = Symbol('lossless');
                for (const key of [symbol, 'expand', '02', '4294967295']) {
                    if (!Reflect.defineProperty(list, key, {value: key, configurable: true})) return 6;
                }
                const expected = ['0', '1', 'expand', '02', '4294967295', symbol];
                const keys = Reflect.ownKeys(list);
                if (keys.length !== expected.length || keys.some((key, i) => key !== expected[i])) return 7;
            }
            return 1;
        })()"#,
    );
}

pub(super) fn indexed_definition_converts_without_native_admission() {
    let upper = usize::try_from(limits().hosted.work).unwrap();
    let exact_work = exact_limit(upper, |work| indexed_probe(work as u64, "plain"));
    for list in ["rules", "media"] {
        assert_eq!(indexed_probe(exact_work as u64, list), Admission::Accepted);
        assert_eq!(
            indexed_probe(exact_work as u64, &format!("read-{list}")),
            Admission::Work
        );
        assert_eq!(
            indexed_probe(exact_work as u64, &format!("ordinary-{list}")),
            Admission::Accepted
        );
    }
}

fn indexed_probe(work: u64, probe: &str) -> Admission {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    eval(
        &mut host,
        &permit,
        r#"(() => {
            const sheet = new CSSStyleSheet({media: 'screen'});
            sheet.insertRule('a {}');
            const rules = sheet.cssRules;
            const media = sheet.media;
            const plain = Object.preventExtensions({});
            function refusedDefinition(object) {
                for (const key of ['0', '1']) {
                    const reads = [];
                    const descriptor = {
                        get enumerable() { reads.push('enumerable'); return true; },
                        get configurable() { reads.push('configurable'); return true; },
                        get value() { reads.push('value'); return 9; },
                        get writable() { reads.push('writable'); return true; },
                    };
                    if (Reflect.defineProperty(object, key, descriptor)
                        || reads.join(',') !== 'enumerable,configurable,value,writable') return false;
                    const sentinel = {};
                    try {
                        Reflect.defineProperty(object, key, {
                            get configurable() { throw sentinel; },
                            get value() { throw new Error('continued after refusal'); },
                        });
                        return false;
                    } catch (error) {
                        if (error !== sentinel) return false;
                    }
                }
                return true;
            }
            function ordinary(object) {
                return Object.getOwnPropertyDescriptor(object, 'plainName') === undefined
                    && Reflect.defineProperty(object, 'plainName', {value: 17})
                    && Object.getOwnPropertyDescriptor(object, 'plainName').value === 17;
            }
            function read(object) {
                return Object.getOwnPropertyDescriptor(object, '0') !== undefined;
            }
            globalThis.probes = {
                plain: () => refusedDefinition(plain),
                rules: () => refusedDefinition(rules),
                media: () => refusedDefinition(media),
                'read-rules': () => read(rules),
                'read-media': () => read(media),
                'ordinary-rules': () => ordinary(rules),
                'ordinary-media': () => ordinary(media),
            };
            return 1;
        })()"#,
        1.0,
    );
    let document = top.document().node();
    let before = host
        .with_native(|dom, _, _| {
            dom.style_cssom_root(document)
                .unwrap()
                .unwrap()
                .snapshot()
                .unwrap()
                .revision()
                .value()
        })
        .unwrap();
    let mut policy = limits().hosted;
    policy.work = work;
    if host
        .with_native(|dom, mut hosted, _| hosted.set_limits(dom, policy, &cancel()))
        .unwrap()
        .is_err()
    {
        return Admission::BeforeOperation;
    }
    let script = format!("Number(probes['{probe}']())");
    let operation = host.in_document(&permit, |scope| assert_script_number(scope, &script, 1.0));
    let after = host
        .with_native(|dom, _, _| {
            dom.style_cssom_root(document)
                .unwrap()
                .unwrap()
                .snapshot()
                .unwrap()
                .revision()
                .value()
        })
        .unwrap();
    assert_eq!(after, before);
    match operation {
        Ok(()) => Admission::Accepted,
        Err(DocumentScriptError::Script(Error::HostFailure(failure))) => {
            let mut source = std::error::Error::source(&failure);
            while let Some(error) = source {
                if matches!(
                    error.downcast_ref::<DomHostedRuntimeError>(),
                    Some(DomHostedRuntimeError::Admission)
                ) {
                    return Admission::Work;
                }
                source = error.source();
            }
            panic!("indexed probe {probe} did not preserve the admission cause: {failure:?}");
        }
        Err(DocumentScriptError::Native(DocumentScriptNativeError::Hosted(
            DomHostedRuntimeError::Admission,
        ))) => Admission::BeforeOperation,
        other => panic!("unexpected indexed probe result for {probe}: {other:?}"),
    }
}

fn constructor_and_mutation_order() {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            function throws(operation, check) {
                try { operation(); return false; }
                catch (value) { return check(value); }
            }
            const intrinsicError = error => error instanceof TypeError;
            const domError = name => error => Object.getPrototypeOf(error) === DOMException.prototype
                && error.name === name;
            const marker = {};
            const log = [];
            const mediaOwner = new CSSStyleSheet({media: 'screen'});
            const source = mediaOwner.media;
            source[Symbol.toPrimitive] = () => { throw marker; };
            const targetPrototype = Object.create(CSSStyleSheet.prototype);
            const target = new Proxy(function Target() {}, {
                get(object, key, receiver) {
                    if (key !== 'prototype') return Reflect.get(object, key, receiver);
                    log.push('prototype');
                    const inner = new CSSStyleSheet({media: 'print'});
                    if (inner.media.mediaText !== 'print') throw marker;
                    return targetPrototype;
                },
            });
            const sheet = Reflect.construct(CSSStyleSheet, [{
                get baseURL() { log.push('baseURL'); return {toString() {
                    log.push('baseURL string'); return '\ud800';
                }}; },
                get disabled() { log.push('disabled'); return true; },
                get media() { log.push('media'); return source; },
            }, marker], target);
            const copied = sheet.media;
            if (log.join(',') !== 'baseURL,baseURL string,disabled,media,prototype'
                || Object.getPrototypeOf(sheet) !== targetPrototype || !sheet.disabled || sheet.href !== 'about:blank'
                || copied === source || copied.mediaText !== 'screen' || sheet.media !== copied
                || copied[0] !== 'screen' || copied.item(1) !== null
                || copied.toString() !== 'screen') return false;

            const ctor = Object.getOwnPropertyDescriptor(globalThis, 'CSSStyleSheet');
            const proto = Object.getOwnPropertyDescriptor(CSSStyleSheet, 'prototype');
            if (CSSStyleSheet.name !== 'CSSStyleSheet' || CSSStyleSheet.length !== 0
                || ctor.value !== CSSStyleSheet || ctor.enumerable || !ctor.writable || !ctor.configurable
                || proto.value !== CSSStyleSheet.prototype || proto.writable || proto.configurable || proto.enumerable
                || Object.getPrototypeOf(CSSStyleSheet) !== StyleSheet
                || CSSStyleSheet.prototype.insertRule.length !== 1
                || CSSStyleSheet.prototype.deleteRule.length !== 1
                || !throws(() => CSSStyleSheet(), intrinsicError)
                || !throws(() => new MediaList(), intrinsicError)
                || !throws(() => new CSSStyleSheet(42), intrinsicError)
                || !throws(() => Reflect.construct(CSSStyleSheet, [{
                    get baseURL() { throw marker; },
                    get disabled() { throw new Error('unreached'); }
                }], target), value => value === marker)
                || log.at(-1) !== 'prototype') return false;

            const insert = CSSStyleSheet.prototype.insertRule;
            const details = [];
            if (!throws(() => insert.call({}, {toString() { throw marker; }}), intrinsicError)
                || !throws(() => insert.call(sheet), intrinsicError)
                || !throws(() => insert.call(sheet,
                    {toString() { details.push('rule'); return 'invalid'; }},
                    {valueOf() { details.push('index'); throw marker; }}), value => value === marker)
                || details.join(',') !== 'rule,index'
                || !throws(() => insert.call(sheet, 'invalid', 99), domError('SyntaxError'))
                || !throws(() => insert.call(sheet, 'a {}', 99), domError('IndexSizeError'))
                || !throws(() => insert.call(sheet, '@import "a";'), domError('SyntaxError'))
                || !throws(() => sheet.deleteRule(0), domError('IndexSizeError'))
                || sheet.cssRules.length !== 0) return false;
            return true;
        })())"#,
        1.0,
    );
    host.with_native(|dom, _, _| {
        let owner = dom
            .style_cssom_root(top.document().node())
            .unwrap()
            .unwrap();
        assert_eq!(owner.snapshot().unwrap().revision().value(), 3);
    })
    .unwrap();
}

fn media_mutations_and_branched_copy() {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    assert_contract(
        &mut host,
        &permit,
        r#"Number((() => {
        const sheet = new CSSStyleSheet({media: 'screen'});
        const media = sheet.media;
        const text = Object.getOwnPropertyDescriptor(MediaList.prototype, 'mediaText');
        const forwarded = Object.getOwnPropertyDescriptor(StyleSheet.prototype, 'media');
        if (text.get.name !== 'get mediaText' || text.set.name !== 'set mediaText'
            || text.set.length !== 1 || !text.enumerable || !text.configurable
            || forwarded.set.name !== 'set media' || forwarded.set.length !== 1
            || !forwarded.enumerable || !forwarded.configurable
            || MediaList.prototype.appendMedium.length !== 1
            || MediaList.prototype.deleteMedium.length !== 1) return 2;
        let coerced = 0;
        sheet.media = {toString() { coerced++; return 'print, screen'; }};
        if (coerced !== 1 || media !== sheet.media || media.mediaText !== 'print, screen'
            || [...media].join(',') !== 'print,screen' || media.item(1) !== 'screen') return 3;
        media.appendMedium('speech');
        media.deleteMedium('print');
        if (media.mediaText !== 'screen, speech' || media[1] !== 'speech') return 4;
        try { media.deleteMedium('print'); return 5; }
        catch (error) {
            if (Object.getPrototypeOf(error) !== DOMException.prototype || error.name !== 'NotFoundError') return 6;
        }
        media.appendMedium('print, screen');
        if (media.mediaText !== 'screen, speech') return 7;
        sheet.media = null;
        if (media.mediaText !== '' || media.length !== 0 || media[0] !== undefined || media.item(0) !== null) return 8;

        media[Symbol.toPrimitive] = () => { throw new Error('the branded MediaList was coerced'); };
        const proto = CSSStyleSheet.prototype;
        const target = new Proxy(function Target() {}, {
            get(object, key, receiver) {
                if (key !== 'prototype') return Reflect.get(object, key, receiver);
                media.mediaText = 'print';
                return proto;
            }
        });
        const copy = Reflect.construct(CSSStyleSheet, [{media}], target).media;
        media.mediaText = 'screen';
        if (copy === media || copy.mediaText !== 'print' || media.mediaText !== 'screen') return 9;

        let looked = 0;
        const marker = {};
        Object.defineProperty(sheet, 'media', {configurable: true, get() { looked++; throw marker; }});
        try { forwarded.set.call(sheet, 'print'); return 10; }
        catch (error) { if (error !== marker || looked !== 1) return 11; }
        const rejected = Object.freeze({mediaText: 'fixed'});
        Object.defineProperty(sheet, 'media', {value: rejected, configurable: true});
        forwarded.set.call(sheet, 'print');
        if (rejected.mediaText !== 'fixed') return 12;
        return true;
    })())"#,
    );
    host.with_native(|dom, _, _| {
        let owner = dom
            .style_cssom_root(top.document().node())
            .unwrap()
            .unwrap();
        assert_eq!(owner.snapshot().unwrap().revision().value(), 8);
    })
    .unwrap();
}

#[derive(Debug, PartialEq, Eq)]
enum Admission {
    BeforeOperation,
    Storage,
    Work,
    Accepted,
}

pub(super) fn constructor_admission_rolls_back_before_publication() {
    let exact_storage = exact_limit(STORAGE, |bytes| {
        constructor_admission(bytes, limits().hosted.work)
    });
    assert_eq!(
        constructor_admission(exact_storage - 1, limits().hosted.work),
        Admission::Storage
    );

    let upper = usize::try_from(limits().hosted.work).unwrap();
    let exact_work = exact_limit(upper, |work| constructor_admission(STORAGE, work as u64));
    assert_eq!(
        constructor_admission(STORAGE, (exact_work - 1) as u64),
        Admission::Work
    );
}

pub(super) fn constructor_conversion_charges_original_host() {
    let upper = usize::try_from(limits().hosted.work).unwrap();
    let script = |options| {
        format!(
            r#"(() => {{
                const sentinel = {{}};
                try {{
                    new CSSStyleSheet({{{options}}});
                    return 0;
                }} catch (error) {{
                    if (error !== sentinel) throw error;
                    return 1;
                }}
            }})()"#,
        )
    };
    // Both controls enter the native constructor with one object and throw in
    // a getter before prototype access. The second reaches one more member.
    let first = script("get baseURL() { throw sentinel; }");
    let second = script("get baseURL() { return null; }, get disabled() { throw sentinel; }");
    let first_getter = exact_limit(upper, |work| {
        constructor_trial(STORAGE, work as u64, &first, None)
    });
    assert_eq!(
        constructor_trial(STORAGE, upper as u64, &second, None),
        Admission::Accepted
    );
    assert_eq!(
        constructor_trial(STORAGE, first_getter as u64, &second, None),
        Admission::Work
    );
}

fn exact_limit(upper: usize, trial: impl Fn(usize) -> Admission) -> usize {
    assert_eq!(trial(upper), Admission::Accepted);
    let mut success = upper;
    let mut refused = 0;
    while success - refused > 1 {
        let middle = refused + (success - refused) / 2;
        if trial(middle) == Admission::Accepted {
            success = middle;
        } else {
            refused = middle;
        }
    }
    assert_eq!(trial(success), Admission::Accepted);
    success
}

fn constructor_admission(storage: usize, work: u64) -> Admission {
    // Make the converted input larger than initial attachment backings.
    let script =
        "new CSSStyleSheet({baseURL:'https://wpt.test/' + 'x'.repeat(16384), media:'screen'}); 1";
    constructor_trial(storage, work, script, Some(1))
}

fn constructor_trial(
    storage: usize,
    work: u64,
    script: &str,
    expected_revision: Option<u64>,
) -> Admission {
    use std::error::Error as _;

    let (session, top) = setup();
    let Ok(prepared) = PreparedDocumentScriptHost::prepare(session, storage, Arc::new(cancel()))
    else {
        return Admission::BeforeOperation;
    };
    let Ok(mut host) = prepared.enable(top, ALLOW, hooks(&hook())) else {
        return Admission::BeforeOperation;
    };
    let mut policy = limits().hosted;
    policy.work = work;
    if host
        .with_native(|dom, mut hosted, _| hosted.set_limits(dom, policy, &cancel()))
        .unwrap()
        .is_err()
    {
        return Admission::BeforeOperation;
    }
    let permit = host.permit().unwrap();
    let result = host.in_document(&permit, |scope| {
        let result = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
        if expected_revision.is_none() {
            assert_eq!(scope.unrestricted_double(&result)?, 1.0);
        }
        Ok(())
    });
    let revision = host
        .with_native(|dom, _, _| {
            dom.style_cssom_root(top.document().node())
                .unwrap()
                .map(|owner| owner.snapshot().unwrap().revision().value())
        })
        .unwrap();
    match result {
        Ok(()) => {
            assert_eq!(revision, expected_revision);
            Admission::Accepted
        }
        Err(DocumentScriptError::Script(Error::HostFailure(failure))) => {
            assert_eq!(revision, None);
            let mut source = failure.source();
            while let Some(error) = source {
                if matches!(
                    error.downcast_ref::<DocumentScriptNativeError>(),
                    Some(DocumentScriptNativeError::StorageLimit)
                ) {
                    return Admission::Storage;
                }
                if matches!(
                    error.downcast_ref::<DomHostedRuntimeError>(),
                    Some(DomHostedRuntimeError::Admission)
                ) {
                    return Admission::Work;
                }
                source = error.source();
            }
            panic!("unexpected construction refusal {:?}", failure.source());
        }
        Err(DocumentScriptError::Native(DocumentScriptNativeError::Hosted(
            DomHostedRuntimeError::Admission,
        ))) => {
            assert_eq!(revision, None);
            Admission::BeforeOperation
        }
        other => panic!("unexpected construction result {other:?}"),
    }
}

fn assert_contract(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit, script: &str) {
    let checkpoint = host
        .in_document(permit, |scope| {
            let result = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.unrestricted_double(&result)
        })
        .unwrap();
    assert_eq!(
        checkpoint, 1.0,
        "guest CSSOM contract failed at checkpoint {checkpoint}"
    );
}

pub(super) fn native_window_constructs_a_stylesheet_with_a_live_rule_list() {
    let (session, top) = setup();
    let mut host = prepare(session, Arc::new(cancel()))
        .enable(top, ALLOW, hooks(&hook()))
        .unwrap();
    let permit = host.permit().unwrap();
    let document = top.document().node();
    host.with_native(|dom, _, _| {
        assert!(dom.style_cssom_root(document).unwrap().is_none());
    })
    .unwrap();
    let constructor_type = host
        .in_document(&permit, |scope| {
            let source = "typeof globalThis.CSSStyleSheet"
                .encode_utf16()
                .collect::<Vec<_>>();
            let value = scope.evaluate(&source)?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(
        String::from_utf16(&constructor_type).unwrap(),
        "function",
        "native Window CSSStyleSheet constructor is missing"
    );
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            const reads = [];
            const sheet = new CSSStyleSheet({
                get media() { reads.push("media"); return "screen"; },
                get disabled() { reads.push("disabled"); return true; },
                get baseURL() { reads.push("baseURL"); return "\ud800"; },
            });
            const rules = sheet.cssRules;
            if (reads.join(",") !== "baseURL,disabled,media"
                || !(sheet instanceof CSSStyleSheet) || !(sheet instanceof StyleSheet)
                || Object.getPrototypeOf(CSSStyleSheet.prototype) !== StyleSheet.prototype
                || sheet.disabled !== true || sheet.media.mediaText !== "screen"
                || rules !== sheet.rules || rules.length !== 0
                || rules.item(0) !== null || rules[0] !== undefined) return false;
            if (sheet.insertRule("div {}", 0) !== 0) return false;
            const original = rules[0];
            const descriptor = Object.getOwnPropertyDescriptor(rules, "0");
            if (original !== rules.item(0) || descriptor.value !== original
                || descriptor.writable || !descriptor.enumerable || !descriptor.configurable
                || Reflect.preventExtensions(rules) || !Object.isExtensible(rules)) return false;
            if (sheet.insertRule("span {}", 0) !== 0 || rules.length !== 2
                || rules[1] !== original || rules[0] === original) return false;
            sheet.deleteRule(1);
            return sheet.cssRules === rules && rules.length === 1
                && rules.item(1) === null && rules[1] === undefined;
        })())"#,
        1.0,
    );
    host.with_native(|dom, _, _| {
        let root = dom.style_cssom_root(document).unwrap().unwrap();
        let snapshot = root.snapshot().unwrap();
        assert!(snapshot.sheets().is_empty());
        assert_eq!(snapshot.revision().value(), 4);
    })
    .unwrap();
}
