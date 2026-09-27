#![forbid(unsafe_code)]

//! The outer test executes before the normalization public API exists. Its
//! assertion requires a real external consumer to compile and complete the
//! independent behavioral cases. A missing API produces nested compilation RED;
//! it does not establish that the consumer's behavioral assertions ran.
//! The nested offline Cargo process inherits the outer bounded process group.

#[path = "support/external_consumer.rs"]
mod external_consumer;

#[test]
fn public_normalization_consumer_preserves_order_contexts_payloads_and_failures() {
    let output = external_consumer::run_example("normalization_consumer");
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 consumer output"),
        concat!(
            "ordered grouped contributions: ok\n",
            "pending shorthand and reentry: ok\n",
            "custom values and construction: ok\n",
            "nested declaration context identity: ok\n",
            "pseudo-element context preservation: ok\n",
            "nested selector binding classification: ok\n",
            "linear selector representation: ok\n",
            "symbolic conditional and layer contexts: ok\n",
            "ordered terminal payloads: ok\n",
            "atomic unsupported declaration: ok\n",
            "normalization resource boundaries: ok\n",
            "unchanged recovery diagnostics: ok\n",
            "complete scoped rule traversal: ok\n",
        )
    );
}
