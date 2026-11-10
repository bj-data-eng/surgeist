use super::TestFetcher;
use crate::test::{TestAction, run_test_actions};

fn register(ctx: &mut surgeist_js::Context) {
    crate::fetch::register(TestFetcher::default(), None, ctx).expect("failed to register fetch");
}

#[test]
fn headers_registration_preserves_global_and_explicit_realm_constructor_identity() {
    use crate::fetch::headers::JsHeaders;
    use surgeist_js::{Context, JsObject, JsValue, Source, js_string};

    let mut context = Context::builder().build().unwrap();
    let other = context.create_realm().unwrap();
    assert!(context.get_global_class::<JsHeaders>().unwrap().is_none());
    assert!(other.get_class::<JsHeaders>().unwrap().is_none());
    register(&mut context);
    let global = context.get_global_class::<JsHeaders>().unwrap().unwrap();
    assert!(other.get_class::<JsHeaders>().unwrap().is_none());

    crate::fetch::register(TestFetcher::default(), Some(other.clone()), &mut context).unwrap();
    let local = other.get_class::<JsHeaders>().unwrap().unwrap();
    assert!(!JsObject::equals(
        &global.constructor(),
        &local.constructor()
    ));
    assert!(!JsObject::equals(&global.prototype(), &local.prototype()));
    assert!(JsObject::equals(
        &context
            .get_global_class::<JsHeaders>()
            .unwrap()
            .unwrap()
            .constructor(),
        &global.constructor(),
    ));

    let previous = context.enter_realm(other).unwrap();
    let input = context.eval(Source::from_bytes("[['x', 'y']]")).unwrap();
    let instance = local
        .constructor()
        .construct(&[input], None, context.access())
        .unwrap();
    assert!(JsObject::equals(
        &instance.prototype().unwrap().unwrap(),
        &local.prototype(),
    ));
    context
        .register_global_property(
            js_string!("testProjectedHeadersInstance"),
            instance,
            surgeist_js::property::Attribute::all(),
        )
        .unwrap();
    assert_eq!(
        context
            .eval(Source::from_bytes(
                "[...testProjectedHeadersInstance][0].join(':')"
            ))
            .unwrap(),
        JsValue::from(js_string!("x:y"))
    );
    context.enter_realm(previous).unwrap();
    assert_eq!(
        context
            .eval(Source::from_bytes(
                "[...new Headers([['a', 'b']])][0].join(':')"
            ))
            .unwrap(),
        JsValue::from(js_string!("a:b"))
    );
}

#[test]
fn headers_iterator_missing_registration_preserves_its_existing_guest_error() {
    use crate::fetch::headers_iterator::HeadersIterator;
    use surgeist_js::{Context, JsNativeErrorKind, JsObject, Source};

    let mut context = Context::builder().build().unwrap();
    register(&mut context);
    let installed = context
        .realm()
        .get_class::<HeadersIterator>()
        .unwrap()
        .unwrap();
    let removed = context
        .realm()
        .unregister_class::<HeadersIterator>()
        .unwrap()
        .unwrap();
    assert!(JsObject::equals(
        &installed.constructor(),
        &removed.constructor()
    ));
    assert!(
        context
            .realm()
            .get_class::<HeadersIterator>()
            .unwrap()
            .is_none()
    );

    let error = context
        .eval(Source::from_bytes("[...new Headers([['x', 'y']])]"))
        .unwrap_err();
    let error = error.as_native().unwrap();
    assert_eq!(error.kind(), &JsNativeErrorKind::Error);
    assert_eq!(error.message(), "Headers Iterator not registered");
}

#[test]
fn headers_are_iterable() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                const headers = new Headers([["x", "y"]]);
                const entries = [...headers];
                assertEq(entries.length, 1);
                assertEq(entries[0][0], "x");
                assertEq(entries[0][1], "y");

                const map = new Map(headers);
                assertEq(map.get("x"), "y");
            "#,
        ),
    ]);
}

#[test]
fn headers_get_combines_duplicate_values_with_comma_space() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                const headers = new Headers([
                    ["x-test", "1"],
                    ["x-test", "2"],
                ]);

                assertEq(headers.get("x-test"), "1, 2");
            "#,
        ),
    ]);
}

#[test]
fn headers_normalize_values() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                const expectations = {
                    name1: [" space ", "space"],
                    name2: ["\ttab\t", "tab"],
                    name3: [" spaceAndTab\t", "spaceAndTab"],
                    name4: ["\r\n newLine", "newLine"],
                    name5: ["newLine\r\n ", "newLine"],
                    name6: ["\r\n\tnewLine", "newLine"],
                };

                const fromObject = new Headers(
                    Object.fromEntries(
                        Object.entries(expectations).map(([name, [value]]) => [name, value]),
                    ),
                );

                for (const [name, [, expected]] of Object.entries(expectations)) {
                    assertEq(fromObject.get(name), expected, `constructor should normalize ${name}`);
                }

                const appended = new Headers();
                for (const [name, [value, expected]] of Object.entries(expectations)) {
                    appended.append(name, value);
                    assertEq(appended.get(name), expected, `append should normalize ${name}`);
                }

                const setHeaders = new Headers();
                for (const [name, [value, expected]] of Object.entries(expectations)) {
                    setHeaders.set(name, value);
                    assertEq(setHeaders.get(name), expected, `set should normalize ${name}`);
                }
            "#,
        ),
    ]);
}

#[test]
fn headers_invalid_inputs_throw_type_error_objects() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                const cases = [
                    () => new Headers([["a\n", "b"]]),
                    () => new Headers([["x-test", "a\u0000b"]]),
                    () => {
                        const h = new Headers();
                        h.append("a\n", "b");
                    },
                    () => {
                        const h = new Headers();
                        h.set("x-test", "a\u0000b");
                    },
                ];

                for (const make of cases) {
                    try {
                        make();
                        throw Error("expected the call above to throw");
                    } catch (e) {
                        assert(e instanceof TypeError, "should throw TypeError instance");
                        assert(typeof e.message === "string" && e.message.length > 0, "error message should be non-empty");
                    }

                }
            "#,
        ),
    ]);
}

#[test]
fn headers_iterator_throws_on_invalid_this() {
    run_test_actions([
        TestAction::harness(),
        TestAction::inspect_context(register),
        TestAction::run(
            r#"
                try {
                    const iterator = Headers.prototype[Symbol.iterator].call({});
                    throw Error("expected the call above to throw");
                } catch (e) {
                    assert(e instanceof TypeError);
                    assertEq(e.message, "`Headers.prototype[Symbol.iterator]` requires a `Headers` object");
                }

                try {
                    const iterator = Headers.prototype[Symbol.iterator].call(1);
                    throw Error("expected the call above to throw");
                } catch (e) {
                    assert(e instanceof TypeError);
                    assertEq(e.message, "`Headers.prototype[Symbol.iterator]` requires a `Headers` object");
                }
            "#,
        ),
    ]);
}
