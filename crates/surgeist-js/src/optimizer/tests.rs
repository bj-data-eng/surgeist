use crate::{Context, JsNativeErrorKind, JsValue, Source, js_string, optimizer::OptimizerOptions};

const MODES: [OptimizerOptions; 3] = [
    OptimizerOptions::empty(),
    OptimizerOptions::STRENGTH_REDUCTION,
    OptimizerOptions::OPTIMIZE_ALL,
];

#[test]
fn exponentiation_reads_an_observable_identifier_once() {
    for options in MODES {
        let mut context = Context::builder().build().unwrap();
        context.set_optimizer_options(options);
        let result = context
            .eval(Source::from_bytes(
                "let reads = 0;
                 const holder = { get value() { reads++; return reads; } };
                 let result;
                 with (holder) { result = value ** 2; }
                 String(result) + ',' + reads",
            ))
            .unwrap();
        assert_eq!(result, JsValue::from(js_string!("1,1")), "{options:?}");
    }
}

#[test]
fn exponentiation_coerces_an_object_base_once() {
    for options in MODES {
        let mut context = Context::builder().build().unwrap();
        context.set_optimizer_options(options);
        let result = context
            .eval(Source::from_bytes(
                "let coercions = 0;
                 const value = { valueOf() { coercions++; return coercions; } };
                 const result = value ** 2;
                 String(result) + ',' + coercions",
            ))
            .unwrap();
        assert_eq!(result, JsValue::from(js_string!("1,1")), "{options:?}");
    }
}

#[test]
fn exponentiation_rejects_a_bigint_base_with_a_number_exponent() {
    for options in MODES {
        let mut context = Context::builder().build().unwrap();
        context.set_optimizer_options(options);
        let error = context.eval(Source::from_bytes("2n ** 2")).unwrap_err();
        assert_eq!(error.as_native().unwrap().kind(), &JsNativeErrorKind::Type);
    }
}

#[test]
fn numeric_exponentiation_retains_special_values_and_bigint_results() {
    for options in MODES {
        let mut context = Context::builder().build().unwrap();
        context.set_optimizer_options(options);
        let result = context
            .eval(Source::from_bytes(
                "[3 ** 2, (-0) ** 2, (-2n) ** 2n, 0n ** 0n, NaN ** 2,
                  Object.is((-0) ** 2, 0)].map(String).join(',')",
            ))
            .unwrap();
        assert_eq!(result, JsValue::from(js_string!("9,0,4,1,NaN,true")));
    }
}

#[test]
fn division_reduction_retains_values_and_one_object_coercion() {
    for options in MODES {
        let mut context = Context::builder().build().unwrap();
        context.set_optimizer_options(options);
        let result = context
            .eval(Source::from_bytes(
                "let coercions = 0;
                 const value = { valueOf() { coercions++; return 6; } };
                 [value / 2, coercions, 1 / 2, Infinity / 2, -Infinity / 2,
                  NaN / 2, 8n / 2n, Object.is(-0 / 2, -0)].map(String).join(',')",
            ))
            .unwrap();
        assert_eq!(
            result,
            JsValue::from(js_string!("3,1,0.5,Infinity,-Infinity,NaN,4,true"))
        );
        let error = context.eval(Source::from_bytes("2n / 2")).unwrap_err();
        assert_eq!(error.as_native().unwrap().kind(), &JsNativeErrorKind::Type);
    }
}

#[test]
fn contextual_numeric_conversion_preserves_floating_edge_results() {
    for options in MODES {
        let mut context = Context::builder().build().unwrap();
        context.set_optimizer_options(options);
        let value = context
            .eval(Source::from_bytes(
                "const zero = { valueOf() { return 0; } };\n\
             const minimum = { valueOf() { return -2147483648; } };\n\
             Object.is(zero / -1, -0) && Object.is(minimum % -1, -0)",
            ))
            .unwrap();
        assert_eq!(value, JsValue::new(true));
    }
}
