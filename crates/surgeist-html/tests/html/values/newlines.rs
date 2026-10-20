#![forbid(unsafe_code)]

use surgeist_html::HtmlNewlineNormalizer;

fn normalize(input: &[u16]) -> Vec<u16> {
    let mut normalizer = HtmlNewlineNormalizer::new();
    input
        .iter()
        .filter_map(|&unit| normalizer.push(unit))
        .collect()
}

// Independent literal expectations from Infra #normalize-newlines: replace
// CR LF pairs first, then the remaining CR units. No other unit is rewritten.
#[test]
fn cr_lf_pairs_and_remaining_cr_have_literal_normalized_outputs() {
    let cases: &[(&[u16], &[u16])] = &[
        (&[], &[]),
        (&[0x000a], &[0x000a]),
        (&[0x000d], &[0x000a]),
        (&[0x000d, 0x000a], &[0x000a]),
        (&[0x000d, 0x000d], &[0x000a, 0x000a]),
        (&[0x000d, 0x000d, 0x000a], &[0x000a, 0x000a]),
        (&[0x000d, 0x000a, 0x000a], &[0x000a, 0x000a]),
        (&[0x000a, 0x000a], &[0x000a, 0x000a]),
        (&[0x000a, 0x000d], &[0x000a, 0x000a]),
        (
            &[0x0041, 0x000d, 0x000a, 0x0042, 0x000d, 0x0043, 0x000a],
            &[0x0041, 0x000a, 0x0042, 0x000a, 0x0043, 0x000a],
        ),
    ];
    for &(input, expected) in cases {
        assert_eq!(normalize(input), expected, "raw input: {input:x?}");
    }
}

#[test]
fn each_push_emits_immediately_or_consumes_only_the_paired_lf() {
    let mut normalizer = HtmlNewlineNormalizer::new();
    assert_eq!(normalizer.push(0x000d), Some(0x000a));
    assert_eq!(normalizer.push(0x000d), Some(0x000a));
    assert_eq!(normalizer.push(0x000a), None);
    assert_eq!(normalizer.push(0x000a), Some(0x000a));
    assert_eq!(normalizer.push(0x0000), Some(0x0000));
    assert_eq!(normalizer.push(0x000d), Some(0x000a));
    assert_eq!(normalizer.push(0xd800), Some(0xd800));
    assert_eq!(normalizer.push(0x000a), Some(0x000a));
    // A final CR already produced its output; no finish step is required.
    assert_eq!(normalizer.push(0x000d), Some(0x000a));
}

#[test]
fn nul_surrogates_and_non_cr_lf_separators_remain_exact_units() {
    assert_eq!(
        normalize(&[
            0x0000, 0xd800, 0x000d, 0xdc00, 0x000d, 0x000a, 0xdbff, 0x2028, 0xdfff, 0x2029, 0x0085,
            0xffff,
        ]),
        [
            0x0000, 0xd800, 0x000a, 0xdc00, 0x000a, 0xdbff, 0x2028, 0xdfff, 0x2029, 0x0085, 0xffff,
        ]
    );
}

#[test]
fn every_chunk_boundary_preserves_raw_cr_lf_adjacency() {
    let input = [0x0041, 0x000d, 0x000a, 0x000a, 0x000d, 0x000d, 0x000a];
    for split in 0..=input.len() {
        let mut normalizer = HtmlNewlineNormalizer::new();
        let mut output = Vec::new();
        for chunk in [&input[..split], &input[split..]] {
            output.extend(chunk.iter().filter_map(|&unit| normalizer.push(unit)));
        }
        assert_eq!(output, [0x0041, 0x000a, 0x000a, 0x000a, 0x000a]);
    }
}

#[test]
fn separate_strings_do_not_inherit_a_preceding_cr() {
    let mut first = HtmlNewlineNormalizer::new();
    assert_eq!(first.push(0x000d), Some(0x000a));
    let mut second = HtmlNewlineNormalizer::new();
    assert_eq!(second.push(0x000a), Some(0x000a));
    let mut default = HtmlNewlineNormalizer::default();
    assert_eq!(default.push(0x000a), Some(0x000a));
    assert_eq!(first.push(0x000a), None);
}
