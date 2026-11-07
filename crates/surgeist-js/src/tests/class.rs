use crate::{TestAction, run_test_actions};
use indoc::indoc;
use surgeist_js_macros::js_str;

#[test]
fn class_field_initializer_name_static() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            class C {
                static a = function() {};
                static ["b"] = function() {};
                static #c = function() {};
                static c = this.#c
            }
        "#}),
        TestAction::assert_eq("C.a.name", js_str!("a")),
        TestAction::assert_eq("C.b.name", js_str!("b")),
        TestAction::assert_eq("C.c.name", js_str!("#c")),
    ]);
}

#[test]
fn class_field_initializer_name() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            class C {
                a = function() {};
                ["b"] = function() {};
                #c = function() {};
                c = this.#c
            }
            let c = new C();
        "#}),
        TestAction::assert_eq("c.a.name", js_str!("a")),
        TestAction::assert_eq("c.b.name", js_str!("b")),
        TestAction::assert_eq("c.c.name", js_str!("#c")),
    ]);
}

#[test]
fn class_superclass_from_regex_error() {
    run_test_actions([TestAction::assert_native_error(
        "class A extends /=/ {}",
        crate::JsNativeErrorKind::Type,
        "superclass must be a constructor",
    )]);
}

// https://github.com/boa-dev/boa/issues/3055
#[test]
fn class_can_access_super_from_static_initializer() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            class a {
                static field = "super field";
            }

            class b extends a {
                static #field = super.field;
                static get field() {
                    return this.#field;
                }
            }

            class c extends a {
                static field = super.field;
            }

        "#}),
        TestAction::assert_eq("a.field", js_str!("super field")),
        TestAction::assert_eq("b.field", js_str!("super field")),
        TestAction::assert_eq("c.field", js_str!("super field")),
    ]);
}

// https://github.com/boa-dev/boa/issues/4400
#[test]
fn class_in_constructor() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            class C {
                constructor() {
                    class D {}
                    this.v = D.name.toString()
                }
            }
            let c = new C()

        "#}),
        TestAction::assert_eq("c.v", js_str!("D")),
    ]);
}

// https://github.com/boa-dev/boa/issues/4555
#[test]
fn nested_class_in_class_expression_constructor() {
    run_test_actions([TestAction::run(
        "new (class { constructor() { class D {} } })();",
    )]);
}

// https://github.com/boa-dev/boa/issues/4555
#[test]
fn nested_class_in_static_block() {
    run_test_actions([TestAction::run("(class { static { class D {} } });")]);
}

#[test]
fn property_initializer_reference_escaped_variable() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            function run() {
                const x = "D";
                class C {
                    a = x;
                    static b = x;
                    #c = x;
                    static #d = x;

                    getC() { return this.#c }
                    static getD() { return C.#d }
                }
                return C
            }
            var Z = run();
            var z = new Z();
        "#}),
        TestAction::assert_eq("z.a", js_str!("D")),
        TestAction::assert_eq("Z.b", js_str!("D")),
        TestAction::assert_eq("z.getC()", js_str!("D")),
        TestAction::assert_eq("Z.getD()", js_str!("D")),
    ]);
}

#[test]
fn private_field_initializer_reference_non_escaped_variable() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            function outer() {
                let x = 1;
                class C {
                    #p = x;
                    m() { return this.#p; }
                }
                return new C().m();
            }
        "#}),
        TestAction::assert_eq("outer()", 1),
    ]);
}

// https://github.com/boa-dev/boa/issues/4605
#[test]
fn class_boolean_literal_method_names() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            class A {
                true() { return 1; }
                false() { return 2; }
                null() { return 3; }
            }
            var a = new A();
        "#}),
        TestAction::assert_eq("a.true()", 1),
        TestAction::assert_eq("a.false()", 2),
        TestAction::assert_eq("a.null()", 3),
    ]);
}

// https://github.com/boa-dev/boa/issues/4605
#[test]
fn class_boolean_literal_static_method_names() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            class B {
                static true() { return 10; }
                static false() { return 20; }
            }
        "#}),
        TestAction::assert_eq("B.true()", 10),
        TestAction::assert_eq("B.false()", 20),
    ]);
}

// https://github.com/boa-dev/boa/issues/4605
#[test]
fn class_boolean_literal_getter_setter_names() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            class C {
                get true() { return this._true; }
                set true(v) { this._true = v; }
                get false() { return this._false; }
                set false(v) { this._false = v; }
            }
            var c = new C();
            c.true = 42;
            c.false = 84;
        "#}),
        TestAction::assert_eq("c.true", 42),
        TestAction::assert_eq("c.false", 84),
    ]);
}

#[test]
fn computed_class_method_names_analyze_callable_keys() {
    for callable in ["function () {}", "() => {}", "async () => {}"] {
        for declaration in ["class C", "const C = class"] {
            let methods = format!(
                r#"
                {declaration} {{
                    [{callable}]() {{ return 7; }}
                    static [{callable}]() {{ return 9; }}
                }};
                const key = String({callable});
                const instance = new C();
                instance[key]() === 7 && C[key]() === 9;
                "#,
            );
            run_test_actions([TestAction::assert(methods)]);

            let accessors = format!(
                r#"
                {declaration} {{
                    get [{callable}]() {{ return this.value; }}
                    set [{callable}](v) {{ this.value = v; }}
                    static get [{callable}]() {{ return this.value; }}
                    static set [{callable}](v) {{ this.value = v; }}
                }};
                const key = String({callable});
                const instance = new C();
                instance[key] = 7;
                C[key] = 9;
                instance[key] === 7 && C[key] === 9;
                "#,
            );
            run_test_actions([TestAction::assert(accessors)]);
        }
    }
}

#[test]
fn computed_class_method_name_closures_capture_class_binding() {
    run_test_actions([
        TestAction::assert(indoc! {r#"
            (() => {
                let capture;
                class C {
                    [(capture = () => C, "method")]() { return 7; }
                }
                return new C().method() === 7 && capture() === C;
            })()
        "#}),
        TestAction::assert(indoc! {r#"
            (() => {
                let capture;
                const C = class Inner {
                    [(capture = () => Inner, "method")]() { return 9; }
                };
                return new C().method() === 9 && capture() === C;
            })()
        "#}),
    ]);
}
