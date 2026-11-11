//! Independent guest oracles over the attached Window and its CSSOM owner.

use super::*;

pub(super) fn run() {
    prototype_own_key_order();
    reuse_native_owner();
    indexed_properties_and_rule_custody();
    constructor_and_mutation_order();
    media_mutations_and_branched_copy();
    constructor_admission_rolls_back_before_publication();
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

fn indexed_properties_and_rule_custody() {
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
    BeforeAttachment,
    Storage,
    Work,
    Published,
}

fn constructor_admission_rolls_back_before_publication() {
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

fn exact_limit(upper: usize, trial: impl Fn(usize) -> Admission) -> usize {
    assert_eq!(trial(upper), Admission::Published);
    let mut success = upper;
    let mut refused = 0;
    while success - refused > 1 {
        let middle = refused + (success - refused) / 2;
        if trial(middle) == Admission::Published {
            success = middle;
        } else {
            refused = middle;
        }
    }
    assert_eq!(trial(success), Admission::Published);
    success
}

fn constructor_admission(storage: usize, work: u64) -> Admission {
    use std::error::Error as _;

    let (mut session, top) = setup();
    let mut policy = limits().hosted;
    policy.work = work;
    if session
        .hosted_execution()
        .set_limits(&mut session.dom(), policy, &cancel())
        .is_err()
    {
        return Admission::BeforeAttachment;
    }
    let Ok(prepared) = PreparedDocumentScriptHost::prepare(session, storage, Arc::new(cancel()))
    else {
        return Admission::BeforeAttachment;
    };
    let Ok(mut host) = prepared.enable(top, ALLOW, hooks(&hook())) else {
        return Admission::BeforeAttachment;
    };
    let permit = host.permit().unwrap();
    let result = host.in_document(&permit, |scope| {
        let script = "void new CSSStyleSheet({media:'screen'})"
            .encode_utf16()
            .collect::<Vec<_>>();
        scope.evaluate(&script)?;
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
            assert_eq!(revision, Some(1));
            Admission::Published
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
            Admission::BeforeAttachment
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
