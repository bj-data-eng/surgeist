use super::{Locale, Mapper, Operation};

fn mapped(input: &str, operation: Operation, locale: Locale) -> String {
    let mapper = Mapper::new().unwrap();
    let input: Vec<_> = input.chars().map(u32::from).collect();
    (0..input.len())
        .flat_map(|at| mapper.map(&input, at, operation, locale).unwrap())
        .map(|cp| char::from_u32(cp).unwrap())
        .collect()
}

#[test]
fn full_contextual_rows_execute_independent_unicode_and_language_literals() {
    // Literal expectations from Unicode17 SpecialCasing and the qualified
    // Greek-uppercase operation, not another mapper's output or a diff.
    for (input, operation, locale, expected) in [
        ("ABc", Operation::Lower, Locale::Root, "abc"),
        ("Straße", Operation::Upper, Locale::Root, "STRASSE"),
        ("ΐ", Operation::Upper, Locale::Root, "Ι\u{308}\u{301}"),
        ("İ", Operation::Lower, Locale::Root, "i\u{307}"),
        ("ΟΣ", Operation::Lower, Locale::Root, "ος"),
        ("ΟΣΑ", Operation::Lower, Locale::Root, "οσα"),
        ("ΟΣ\u{301}Α", Operation::Lower, Locale::Root, "οσ\u{301}α"),
        ("ΟΣ\u{301}", Operation::Lower, Locale::Root, "ος\u{301}"),
        ("iı", Operation::Upper, Locale::Turkic, "İI"),
        ("Iİ", Operation::Lower, Locale::Turkic, "ıi"),
        ("I\u{307}", Operation::Lower, Locale::Turkic, "i"),
        (
            "I\u{301}",
            Operation::Lower,
            Locale::Lithuanian,
            "i\u{307}\u{301}",
        ),
        (
            "i\u{307}\u{301}",
            Operation::Upper,
            Locale::Lithuanian,
            "I\u{301}",
        ),
        ("άδικος", Operation::Upper, Locale::Greek, "ΑΔΙΚΟΣ"),
        ("άι", Operation::Upper, Locale::Greek, "ΑΪ"),
        ("ή", Operation::Upper, Locale::Greek, "Ή"),
        ("η\u{301}", Operation::Upper, Locale::Greek, "Η\u{301}"),
        ("և", Operation::Upper, Locale::Armenian, "ԵՎ"),
        ("և", Operation::Upper, Locale::Root, "ԵՒ"),
    ] {
        assert_eq!(
            mapped(input, operation, locale),
            expected,
            "{input:?}/{operation:?}/{locale:?}"
        );
    }
}

#[test]
fn context_is_original_and_each_emission_or_removal_has_its_actual_row() {
    let mapper = Mapper::new().unwrap();
    let input = [0x39f, 0x3a3, 0x301, 0x391];
    // Identity neighbors remain context, not contributors to sigma's output.
    assert_eq!(
        mapper
            .map(&input, 1, Operation::Lower, Locale::Root)
            .unwrap(),
        [0x3c3]
    );
    let input = [0x49, 0x307];
    assert_eq!(
        mapper
            .map(&input, 0, Operation::Lower, Locale::Turkic)
            .unwrap(),
        [0x69]
    );
    assert!(
        mapper
            .map(&input, 1, Operation::Lower, Locale::Turkic)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        mapper
            .map(&input, 1, Operation::Lower, Locale::Root)
            .unwrap(),
        [0x307]
    );
    let input = [0xdf];
    assert_eq!(
        mapper
            .map(&input, 0, Operation::Title, Locale::Root)
            .unwrap(),
        [0x53, 0x73]
    );
    assert_eq!(
        mapper
            .map(&input, 0, Operation::Upper, Locale::Root)
            .unwrap(),
        [0x53, 0x53]
    );
    // Opaque input never borrows U+FFFD properties or fuses into a scalar.
    let input = [0xd800, 0xfffd, 0xdc00];
    for at in 0..3 {
        assert_eq!(
            mapper
                .map(&input, at, Operation::Upper, Locale::Root)
                .unwrap(),
            [input[at]]
        );
    }
}

#[test]
fn title_initial_and_context_are_distinct_operations_on_original_components() {
    let mapper = Mapper::new().unwrap();
    let input = [0x3b1, 0x313, 0x345];
    assert_eq!(
        mapper
            .map(&input, 0, Operation::Title, Locale::Root)
            .unwrap(),
        [0x391]
    );
    for at in 1..3 {
        assert_eq!(
            mapper
                .map(&input, at, Operation::TitleContext, Locale::Root)
                .unwrap(),
            [input[at]]
        );
    }
    assert_eq!(
        mapper
            .map(&[0x1f80], 0, Operation::Title, Locale::Root)
            .unwrap(),
        [0x1f88]
    );
    let input = [0x69, 0x307];
    assert_eq!(
        mapper
            .map(&input, 0, Operation::Title, Locale::Lithuanian)
            .unwrap(),
        [0x49]
    );
    assert!(
        mapper
            .map(&input, 1, Operation::TitleContext, Locale::Lithuanian)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        mapper
            .map(&input, 1, Operation::TitleContext, Locale::Root)
            .unwrap(),
        [0x307]
    );
    let input = [0x69, 0x6a, 0x53, 0x53, 0x45, 0x4c];
    assert_eq!(
        mapper
            .map(&input, 0, Operation::Title, Locale::Dutch)
            .unwrap(),
        [0x49]
    );
    assert_eq!(
        mapper
            .map(&input, 1, Operation::TitleContext, Locale::Dutch)
            .unwrap(),
        [0x4a]
    );
    assert_eq!(
        mapper
            .map(&input, 1, Operation::TitleContext, Locale::Root)
            .unwrap(),
        [0x6a]
    );
    assert!(!mapper.dutch_j(&[0x69, 0x301], &[]).unwrap());
    assert!(mapper.dutch_j(&[0x69, 0x301], &[0x301, 0x53]).unwrap());
}

#[test]
fn actual_language_script_and_region_choose_only_applicable_conventions() {
    for (tag, expected) in [
        (None, Locale::Root),
        (Some("und"), Locale::Root),
        (Some("und-Grek"), Locale::Root),
        (Some("zz"), Locale::Root),
        (Some("tr"), Locale::Turkic),
        (Some("az"), Locale::Turkic),
        (Some("tr-Arab"), Locale::Root),
        (Some("az-IR"), Locale::Root),
        (Some("az-RU"), Locale::Root),
        (Some("az-Latn-IR"), Locale::Turkic),
        (Some("ba"), Locale::Root),
        (Some("tt"), Locale::Root),
        (Some("crh"), Locale::Root),
        (Some("ba-Latn"), Locale::Turkic),
        (Some("tt-Latn"), Locale::Turkic),
        (Some("crh-Latn"), Locale::Turkic),
        (Some("ga"), Locale::Irish),
        (Some("ga-Latn"), Locale::Irish),
        (Some("ga-Cyrl"), Locale::Root),
        (Some("lt"), Locale::Lithuanian),
        (Some("nl"), Locale::Dutch),
        (Some("hy"), Locale::Armenian),
        (Some("el"), Locale::Greek),
        (Some("en-Grek"), Locale::Greek),
        (Some("en-Armn"), Locale::Armenian),
    ] {
        assert_eq!(Locale::from_tag(tag).unwrap(), expected, "{tag:?}");
    }
}

#[test]
fn reordered_emissions_retain_rows_and_do_not_normalize_away_a_real_removal() {
    let mapper = Mapper::new().unwrap();
    // Precomposed alpha+psili+ypogegrammeni emits ALPHA, psili, IOTA.
    // The separate acute remains root-uppercase; the later psili is genuinely
    // removed by its own Greek-uppercase rule. Row0's IOTA follows the whole
    // original combining sequence, so its output is noncontiguous.
    let input = [0x1f80, 0x301, 0x313];
    let operations = [Operation::Upper; 3];
    let mapped: Vec<_> = [Locale::Root, Locale::Root, Locale::Greek]
        .into_iter()
        .enumerate()
        .map(|(at, locale)| mapper.map(&input, at, operations[at], locale).unwrap())
        .collect();
    assert_eq!(mapped[0], [0x1f08, 0x399]);
    assert_eq!(mapped[1], [0x301]);
    assert!(mapped[2].is_empty());
    let order = super::emission_order(&input, &operations, &mapped).unwrap();
    assert_eq!(order, [(0, 0), (1, 0), (0, 1)]);
    let actual: Vec<_> = order
        .into_iter()
        .map(|(row, sub)| mapped[row][sub])
        .collect();
    assert_eq!(actual, [0x1f08, 0x301, 0x399]);
    // The decomposed spelling requires the same movement, without NFC/NFD
    // changing the input or making the accent a contributor to row1's IOTA.
    let input = [0x3b1, 0x345, 0x301, 0x42];
    let operations = [Operation::Upper; 4];
    let mapped: Vec<_> = (0..4)
        .map(|at| {
            mapper
                .map(&input, at, operations[at], Locale::Root)
                .unwrap()
        })
        .collect();
    let order = super::emission_order(&input, &operations, &mapped).unwrap();
    assert_eq!(order, [(0, 0), (2, 0), (1, 0), (3, 0)]);
    assert_eq!(
        order
            .into_iter()
            .map(|(row, sub)| mapped[row][sub])
            .collect::<Vec<_>>(),
        [0x391, 0x301, 0x399, 0x42]
    );
}
