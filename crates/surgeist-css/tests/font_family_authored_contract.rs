#![forbid(unsafe_code)]

//! This outer test executes against the current public API. Its assertion
//! requires a separate consumer to compile and exercise the checked authored
//! font-family contract, including independent token and generic expectations.
//! System spellings remain literal family names outside the whole-font branch,
//! as explicitly defined by Fonts4 section 2.7:
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-prop
//! The nested offline Cargo process inherits the outer bounded process group
//! and owns its target directory, following font_source_compatibility_contract.rs.

#[path = "support/external_consumer.rs"]
mod external_consumer;

#[test]
fn public_font_family_consumer_compiles_and_preserves_tokens_and_generic_meaning() {
    let output = external_consumer::run_example("font_family_authored_consumer");
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 consumer output"),
        concat!(
            "decoded identifier boundaries: ok\n",
            "token exclusions and literal strings: ok\n",
            "typed generic families: ok\n",
            "system spellings retain contextual meaning: ok\n",
            "current property and font values: ok\n",
        )
    );
}
