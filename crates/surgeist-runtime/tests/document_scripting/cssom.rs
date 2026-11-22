use super::*;

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
