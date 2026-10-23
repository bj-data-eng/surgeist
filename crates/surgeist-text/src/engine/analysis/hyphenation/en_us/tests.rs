use super::*;

// These immutable qualified inputs are linked, not copied into another software
// capture. Expected exceptions come from the actual TeX section, not EXCEPTIONS.
const TEX: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/licenses/hypher/hyph-en-us.tex"
));
const RAW_ORACLE: &str = include_str!("oracle.tsv");

fn source_body() -> String {
    TEX.lines()
        .map(|line| line.split('%').next().unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n")
}

fn source_exceptions() -> Vec<(String, Vec<usize>)> {
    let text = source_body();
    let section = text
        .split_once("\\hyphenation{")
        .unwrap()
        .1
        .split_once('}')
        .unwrap()
        .0;
    section
        .split_whitespace()
        .map(|entry| {
            let mut word = String::new();
            let mut cuts = Vec::new();
            for cp in entry.chars() {
                if cp == '-' {
                    cuts.push(word.len());
                } else {
                    word.push(cp);
                }
            }
            (word, cuts)
        })
        .collect()
}

fn check_slices<'word>(query: &'word str, observation: &Observation<'word>) -> Vec<usize> {
    assert!(std::ptr::eq(query, observation.word()));
    let mut previous = 0;
    for cut in observation.cuts() {
        let byte = cut.byte();
        assert!(previous < byte && byte < query.len());
        assert!(query.is_char_boundary(byte));
        assert!(std::ptr::eq(&query[..byte], cut.before()));
        assert!(std::ptr::eq(&query[byte..], cut.after()));
        assert_eq!(cut.before().len(), byte);
        assert_eq!(cut.before().len() + cut.after().len(), query.len());
        assert_eq!(format!("{}{}", cut.before(), cut.after()), query);
        previous = byte;
    }
    observation.cuts().iter().map(|cut| cut.byte()).collect()
}

#[test]
fn every_source_exception_replaces_patterns_and_retains_original_case_slices() {
    let rows = source_exceptions();
    assert_eq!(rows.len(), 14);
    for (word, expected) in rows {
        let mixed = word
            .chars()
            .enumerate()
            .map(|(i, cp)| {
                if i % 2 == 0 {
                    cp.to_ascii_uppercase()
                } else {
                    cp
                }
            })
            .collect::<String>();
        for spelling in [word.clone(), word.to_ascii_uppercase(), mixed] {
            let observation = observe(&spelling).unwrap();
            assert_eq!(
                check_slices(&spelling, &observation),
                expected,
                "{spelling}"
            );
            let Provenance::SourceException { entry } = observation.provenance() else {
                panic!("actual source exception lost its provenance")
            };
            assert_eq!(entry, word);
        }
    }
}

#[test]
fn zero_cut_exceptions_are_distinct_from_absence_and_override_real_pattern_cuts() {
    let empty = source_exceptions()
        .into_iter()
        .filter(|(_, cuts)| cuts.is_empty())
        .map(|(word, _)| word)
        .collect::<Vec<_>>();
    assert_eq!(empty, ["present", "presents", "project", "projects"]);
    let patterns = ReferencePatterns::new();
    for word in empty {
        assert!(
            !patterns.cuts(&word).is_empty(),
            "independent raw pattern control for {word}"
        );
        let observation = observe(&word).unwrap();
        assert!(check_slices(&word, &observation).is_empty());
        assert!(matches!(
            observation.provenance(),
            Provenance::SourceException { .. }
        ));
    }
    let absent = observe("hi").unwrap();
    assert!(absent.cuts().is_empty());
    assert_eq!(absent.provenance(), Provenance::Pattern);
    let added = observe("table").unwrap();
    assert!(patterns.cuts("table").is_empty());
    assert_eq!(check_slices("table", &added), [2]);
}

#[test]
fn exact_democrat_correction_has_separate_provenance_without_derivative_expansion() {
    assert_eq!(ReferencePatterns::new().cuts("democrat"), [2, 4, 5, 7]);
    for word in ["democrat", "DEMOCRAT", "DeMoCrAt"] {
        let observation = observe(word).unwrap();
        assert_eq!(check_slices(word, &observation), [3, 4]);
        assert_eq!(
            observation.provenance(),
            Provenance::TextCorrection { entry: "democrat" }
        );
    }
    let patterns = ReferencePatterns::new();
    for word in [
        "democrats",
        "democratic",
        "democratization",
        "xdemocrat",
        "democratx",
        "democrat\u{301}",
        "presently",
        "tableness",
        "ｔable",
    ] {
        let observation = observe(word).unwrap();
        assert_eq!(observation.provenance(), Provenance::Pattern, "{word}");
        assert_eq!(
            check_slices(word, &observation),
            patterns.cuts(word),
            "{word}"
        );
    }
}

#[test]
fn raw_one_one_pattern_observations_match_the_qualified_independent_oracle() {
    let exceptions = source_exceptions();
    let mut checked = 0;
    for line in RAW_ORACLE.lines() {
        let (word, values) = line.split_once('\t').unwrap();
        if word.eq_ignore_ascii_case("democrat")
            || exceptions
                .iter()
                .any(|(entry, _)| word.eq_ignore_ascii_case(entry))
        {
            continue;
        }
        let expected = if values.is_empty() {
            Vec::new()
        } else {
            values
                .split(',')
                .map(|value| value.parse::<usize>().unwrap())
                .collect()
        };
        let observation = observe(word).unwrap();
        assert_eq!(observation.provenance(), Provenance::Pattern);
        assert_eq!(check_slices(word, &observation), expected, "{word}");
        checked += 1;
    }
    assert_eq!(checked, 19);
    // 1/1 permits this final two-character portion; the default English2/3
    // bounds would suppress it. CSS character-count admission belongs elsewhere.
    assert_eq!(
        check_slices("probability", &observe("probability").unwrap()),
        [4, 5, 8, 9]
    );
}

#[test]
fn allocated_long_multibyte_queries_remain_exact_utf8_observations() {
    let patterns = ReferencePatterns::new();
    for word in [
        "é".repeat(48) + "extensive",
        "😀".repeat(16) + "hyphenation",
        "e\u{301}".repeat(24) + "extensive",
        "é".repeat(2048) + "extensive",
    ] {
        assert!(word.len() > 45);
        let before = word.clone();
        let observation = observe(&word).unwrap();
        assert_eq!(observation.provenance(), Provenance::Pattern);
        let actual = check_slices(&word, &observation);
        assert!(
            !actual.is_empty(),
            "long-input test must exercise actual cut observations"
        );
        assert_eq!(actual, patterns.cuts(&word));
        assert_eq!(word, before);
    }
    for word in [
        "",
        "a",
        "İSTANBUL",
        "A\u{301}A",
        "\0extensive",
        "ßextensive",
    ] {
        let observation = observe(word).unwrap();
        check_slices(word, &observation);
    }
}

#[test]
fn checked_cut_construction_refuses_endpoints_nonboundaries_and_reversal() {
    for (word, offsets) in [
        ("abc", vec![0]),
        ("abc", vec![3]),
        ("abc", vec![2, 1]),
        ("abc", vec![1, 1]),
        ("éx", vec![1]),
    ] {
        assert!(matches!(
            from_offsets(word, Provenance::Pattern, &offsets),
            Err(ObservationError::InvalidCut { .. })
        ));
    }
}

// Independent uncompressed Liang max/odd scoring of the exact source patterns.
// No provider output or adapter constant constructs these expected cuts.
struct ReferencePatterns(Vec<(Vec<u8>, Vec<u8>)>);
impl ReferencePatterns {
    fn new() -> Self {
        let text = source_body();
        let section = text
            .split_once("\\patterns{")
            .unwrap()
            .1
            .split_once('}')
            .unwrap()
            .0;
        let patterns = section
            .split_whitespace()
            .map(|token| {
                assert!(token.is_ascii());
                let mut letters = Vec::new();
                let mut weights = vec![0];
                for byte in token.bytes() {
                    if byte.is_ascii_digit() {
                        *weights.last_mut().unwrap() = byte - b'0';
                    } else {
                        letters.push(byte);
                        weights.push(0);
                    }
                }
                (letters, weights)
            })
            .collect::<Vec<_>>();
        assert_eq!(patterns.len(), 4938);
        Self(patterns)
    }
    fn cuts(&self, word: &str) -> Vec<usize> {
        let mut dotted = vec![b'.'];
        dotted.extend(word.bytes().map(|byte| byte.to_ascii_lowercase()));
        dotted.push(b'.');
        let mut levels = vec![0; dotted.len() + 1];
        for (letters, weights) in &self.0 {
            if letters.len() > dotted.len() {
                continue;
            }
            for start in 0..=dotted.len() - letters.len() {
                if dotted[start..].starts_with(letters) {
                    for (relative, weight) in weights.iter().copied().enumerate() {
                        levels[start + relative] = levels[start + relative].max(weight);
                    }
                }
            }
        }
        (1..word.len())
            .filter(|at| word.is_char_boundary(*at) && levels[at + 1] % 2 == 1)
            .collect()
    }
}
