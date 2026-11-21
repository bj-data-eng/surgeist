use surgeist_html::{
    attributes::HtmlInputType::{Checkbox, Hidden, Text},
    bounded::attributes::{
        HtmlAutocompleteControl::{Input, Select, Textarea},
        control_autocomplete_idl_with,
    },
};

#[test]
fn autocomplete_idl_preserves_the_pinned_token_casing_and_mantle() {
    for (raw, control, expected) in [
        (None, Select, ""),
        (Some(""), Select, ""),
        (Some("  OfF  "), Select, "off"),
        (Some("  OfF  "), Input(Text), "off"),
        (Some("  OfF  "), Input(Checkbox), "off"),
        (Some("  OfF  "), Input(Hidden), ""),
        (
            Some("  SECTION-Home\tSHIPPING HOME TeL WEBAUTHN "),
            Select,
            "section-home shipping home TeL WEBAUTHN",
        ),
        (
            Some("USERNAME WEBAUTHN"),
            Input(Hidden),
            "USERNAME WEBAUTHN",
        ),
        (Some("WEBAUTHN"), Textarea, "WEBAUTHN"),
        (Some("HOME SHIPPING TEL"), Select, ""),
        (Some("on webauthn"), Select, ""),
        (
            Some("extra section-home shipping home tel webauthn"),
            Select,
            "",
        ),
    ] {
        let raw = raw.map(|raw| raw.encode_utf16().collect::<Vec<_>>());
        let expected = expected.encode_utf16().collect::<Vec<_>>();
        let plan =
            control_autocomplete_idl_with(raw.as_deref(), control, |_| Ok::<_, ()>(())).unwrap();
        assert_eq!(plan.code_units_len(), Some(expected.len()));
        let mut output = Vec::new();
        plan.visit_code_units_with(
            || Ok::<_, ()>(()),
            |unit| {
                output.push(unit);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(output, expected);
    }
}

#[test]
fn autocomplete_idl_admits_reached_work_and_each_exact_output_unit() {
    let mut raw = "SeCtIoN-ABC".encode_utf16().collect::<Vec<_>>();
    raw.push(0xd800);
    raw.extend(" NAME".encode_utf16());
    let mut expected = "section-abc".encode_utf16().collect::<Vec<_>>();
    expected.push(0xd800);
    expected.extend(" NAME".encode_utf16());
    let mut required = 0;
    let plan = control_autocomplete_idl_with(Some(&raw), Select, |work| {
        assert!(work > 0);
        required += work;
        Ok::<_, ()>(())
    })
    .unwrap();
    assert!(required > raw.len());

    let mut remaining = required - 1;
    let refused = control_autocomplete_idl_with(Some(&raw), Select, |work| {
        remaining = remaining
            .checked_sub(work)
            .ok_or("first admission refusal")?;
        Ok(())
    });
    assert!(matches!(refused, Err("first admission refusal")));
    assert_eq!(plan.code_units_len(), Some(expected.len()));
    for cutoff in 0..=expected.len() {
        let mut reached = 0;
        let mut output = Vec::new();
        let result = plan.visit_code_units_with(
            || {
                if reached == cutoff {
                    return Err(reached);
                }
                reached += 1;
                Ok(())
            },
            |unit| {
                output.push(unit);
                Ok(())
            },
        );
        let outcome = if cutoff == expected.len() {
            Ok(())
        } else {
            Err(cutoff)
        };
        assert_eq!(result, outcome);
        assert_eq!(output, expected[..cutoff]);
    }
}
