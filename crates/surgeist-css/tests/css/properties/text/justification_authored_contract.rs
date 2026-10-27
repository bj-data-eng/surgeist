//! Independent authored contract: Text4 WD20260814 #propdef-text-justify,
//! #valdef-text-justify-distribute and #propdef-text-group-align; Values3/4
//! §2.2 and Text4 §1.2 supply grammar, CSS-wide exclusivity and metadata.
//! Base-before-modifier ordering and transparent keyword prices are adopted
//! Surgeist policies; the text-justify canonical-order table says n/a.
//! Existing public front doors only. New typed constructors/borrowed views,
//! direct primitive providers and private suppression tests accompany functional
//! implementation; computed distribute equivalence remains downstream.

use crate::css::support::authored_checked_declaration as checked;
use crate::css::support::authored_parsed_declaration as parsed;
use crate::css::support::authored_property::assert_source;
use crate::css::support::check_named_components as checked_components;
use crate::css::support::known_property as property;
use crate::css::support::named_property::invalid_named_value as invalid;
use crate::css::support::named_property::named_fronts as fronts;
use crate::css::support::serialization_limits::LimitLadder;
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValue, CssComponentValues, CssValueOrigin};
use surgeist_css::declaration::{CssDeclaration, CssGlobalKeyword, CssImportance};
use surgeist_css::normalize::{
    CssNormalizationLimits, CssNormalizationResource, CssNormalizedItem,
};
use surgeist_css::property::CssKnownProperty as P;
use surgeist_css::property::{
    CssContributionValueRef, CssContributions, CssExpansion, CssInitialValueRef,
    CssLonghandContributions, CssPropertyKindRef,
};

fn contributions(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) = source.expand().unwrap()
    else {
        panic!("completed intrinsic contribution")
    };
    let name = source.known().unwrap().property().canonical_name();
    assert_eq!(values.items().len(), terminals(name).len());
    for (item, name) in values.items().iter().zip(terminals(name)) {
        assert_eq!(item.property(), property(name));
        assert_source(item, source);
    }
    values
}
fn canonical(source: &CssDeclaration, name: &str, text: &str) {
    assert_eq!(
        source.serialize_specified(Default::default()).unwrap(),
        format!("{name}: {text} !important;")
    );
}
fn accepted(name: &str, input: &str, expected: &str) {
    for source in fronts(name, input) {
        let before = source.clone();
        canonical(&source, name, expected);
        assert!(source.known().unwrap().property_value().is_some());
        assert!(source.known().unwrap().global().is_none());
        assert!(source.known().unwrap().substitution_dependent().is_none());
        for item in contributions(&source).items() {
            assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
            assert_eq!(
                item.ordinary_value().unwrap().property().known_property(),
                item.property()
            );
            assert!(item.replacement_components().is_none());
        }
        assert_eq!(source, before);
    }
    canonical(&checked(name, expected, true), name, expected);
}
fn permutations(words: &[&str]) -> Vec<String> {
    fn walk(words: &[&str], used: &mut Vec<bool>, prefix: &mut Vec<String>, out: &mut Vec<String>) {
        if prefix.len() == words.len() {
            out.push(prefix.join(" "));
            return;
        }
        for i in 0..words.len() {
            if !used[i] {
                used[i] = true;
                prefix.push(words[i].to_owned());
                walk(words, used, prefix, out);
                prefix.pop();
                used[i] = false;
            }
        }
    }
    let mut out = Vec::new();
    walk(
        words,
        &mut vec![false; words.len()],
        &mut Vec::new(),
        &mut out,
    );
    out
}
#[test]
fn programmatic_components_preserve_origin_identity_and_importance_at_both_checked_fronts() {
    for name in FAMILY {
        let text = ordinary_cases(name)[0];
        let value = if name == "line-padding" {
            CssComponentValue::try_number("0").unwrap()
        } else {
            CssComponentValue::try_ident(text).unwrap()
        };
        let components = CssComponentValues::try_new(vec![value], Default::default()).unwrap();
        for grammar in [false, true] {
            let source = checked_components(name, components.clone(), grammar).unwrap();
            assert_eq!(source.value_components(), &components);
            assert_eq!(
                source.value_components().items()[0].origin(),
                &CssValueOrigin::Programmatic
            );
            assert!(source.parsed_value().is_none());
            assert_eq!(source.importance(), CssImportance::Important);
            canonical(&source, name, text);
            contributions(&source);
        }
    }
}
#[test]
fn intrinsic_initials_and_inheritance_equal_authored_terminal_values_without_fabricated_sources() {
    for (name, text, inherited) in INITIALS {
        let p = property(name);
        assert_eq!(P::from_name(&name.to_ascii_uppercase()), Some(p));
        let CssPropertyKindRef::Longhand(meta) = p.metadata().unwrap().kind() else {
            panic!("intrinsic longhand")
        };
        assert_eq!(meta.property().known_property(), p);
        assert_eq!(meta.inherited_by_default(), inherited);
        let initial = meta.initial_value();
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("ordinary initial")
        };
        assert_eq!(value.property().known_property(), p);
        let declaration = checked(name, text, true);
        let values = contributions(&declaration);
        assert_eq!(values.items()[0].ordinary_value(), Some(value));
    }
}
#[test]
fn normalization_counts_complete_terminals_and_pending_occurrences_cumulatively() {
    for name in FAMILY {
        let css = format!(
            "@media screen{{.a{{{name}:{}!important;{name}:var(--text);{name}:unset!important;color:red}}}}",
            ordinary_cases(name)[0]
        );
        let report = CssParser::default().parse_sheet(&css);
        assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
        let before = report.clone();
        let count = terminals(name).len() * 2 + 2;
        let exact = CssNormalizationLimits::try_new(1, 2, 4, count).unwrap();
        let normalized = report.syntax().normalize(exact).unwrap();
        let declarations: Vec<_> = normalized
            .items()
            .iter()
            .filter_map(|v| match v {
                CssNormalizedItem::Declaration(v) => Some(v),
                _ => None,
            })
            .collect();
        assert_eq!(declarations.len(), 4);
        for (index, item) in declarations.iter().enumerate() {
            assert_eq!(item.order(), index);
            assert_eq!(
                item.source().known().unwrap().property(),
                if index == 3 { P::Color } else { property(name) }
            );
            assert_eq!(
                item.source().importance(),
                if index == 0 || index == 2 {
                    CssImportance::Important
                } else {
                    CssImportance::Normal
                }
            );
            assert!(
                item.selector_context()
                    .same_context(declarations[0].selector_context())
            );
            assert!(
                item.rule_context()
                    .same_context(declarations[0].rule_context())
            );
            if index > 0 {
                assert!(
                    !item
                        .source()
                        .same_occurrence(declarations[index - 1].source())
                );
            }
            match item.expansion() {
                CssExpansion::Pending(handle) => {
                    assert_eq!(index, 1);
                    assert!(handle.source().same_occurrence(item.source()));
                }
                CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                    assert_eq!(
                        values.items().len(),
                        if index == 3 { 1 } else { terminals(name).len() }
                    );
                    for value in values.items() {
                        assert_source(value, item.source());
                        if index == 2 {
                            assert!(matches!(
                                value.value(),
                                CssContributionValueRef::Global(CssGlobalKeyword::Unset)
                            ));
                        } else {
                            assert!(matches!(
                                value.value(),
                                CssContributionValueRef::Ordinary(_)
                            ));
                        }
                    }
                }
                _ => panic!("intrinsic authored lifecycle"),
            }
        }
        for (limits, resource, limit) in [
            (
                CssNormalizationLimits::try_new(1, 2, 4, count - 1).unwrap(),
                CssNormalizationResource::Contributions,
                count - 1,
            ),
            (
                CssNormalizationLimits::try_new(1, 2, 3, count).unwrap(),
                CssNormalizationResource::Declarations,
                3,
            ),
        ] {
            for _ in 0..2 {
                let error = report.syntax().normalize(limits).unwrap_err();
                assert_eq!(error.exhausted_limit(), Some((resource, limit)));
                assert_eq!(error.declaration_order(), Some(3));
                assert!(
                    error
                        .declaration()
                        .unwrap()
                        .same_occurrence(declarations[3].source())
                );
            }
        }
        assert_eq!(report, before);
        assert!(report.syntax().normalize(exact).is_ok());
    }
}
fn declaration_limits(name: &str, text: &str, nodes: usize) {
    for source in [parsed(name, text), checked(name, text, true)] {
        let expected = format!("{name}: {text} !important;");
        LimitLadder::new(&expected, nodes + 2, nodes + 2)
            .assert_unchanged(&source, CssDeclaration::serialize_specified);
    }
}
#[test]
fn every_emitted_keyword_and_literal_charges_exact_declaration_work_and_utf8_bytes_atomically() {
    for name in FAMILY {
        for text in budget_cases(name) {
            declaration_limits(name, &text, text.split_whitespace().count());
        }
    }
}
fn sheet_limits(input: &str, expected: &str, nodes: usize) {
    let report = CssParser::default().parse_sheet(input);
    assert!(report.is_clean(), "{input}: {:?}", report.diagnostics());
    // Each rule fits separately; exhaustion only comes from cumulative use.
    LimitLadder::new(expected, nodes, nodes).assert_cumulative_sheet(report.syntax(), 1);
}

const FAMILY: [&str; 2] = ["text-justify", "text-group-align"];
const BASES: [&str; 6] = [
    "auto",
    "none",
    "inter-word",
    "inter-character",
    "ruby",
    "distribute",
];
const GROUP: [&str; 6] = ["none", "start", "end", "left", "right", "center"];
const INITIALS: [(&str, &str, bool); 2] = [
    ("text-justify", "auto", true),
    ("text-group-align", "none", false),
];
fn terminals(name: &str) -> &'static [&'static str] {
    match name {
        "text-justify" => &["text-justify"],
        "text-group-align" => &["text-group-align"],
        _ => panic!("family"),
    }
}
fn ordinary_cases(name: &str) -> &'static [&'static str] {
    match name {
        "text-justify" => &[
            "auto",
            "none",
            "inter-word",
            "inter-character",
            "ruby",
            "distribute",
            "no-compress",
            "ruby no-compress",
            "distribute no-compress",
        ],
        "text-group-align" => &GROUP,
        _ => panic!("family"),
    }
}
fn budget_cases(name: &str) -> Vec<String> {
    match name {
        "text-justify" => BASES
            .into_iter()
            .flat_map(|base| [base.to_owned(), format!("{base} no-compress")])
            .chain(["no-compress".to_owned()])
            .collect(),
        "text-group-align" => GROUP.into_iter().map(str::to_owned).collect(),
        _ => panic!("family"),
    }
}
#[test]
fn all_justify_bases_and_modifier_orders_preserve_base_omission_and_authored_distribute() {
    accepted("text-justify", "no-compress", "no-compress");
    for base in BASES {
        accepted("text-justify", base, base);
        let canonical = format!("{base} no-compress");
        for text in permutations(&[base, "no-compress"]) {
            accepted("text-justify", &text, &canonical);
        }
    }
    // The chosen specified phase retains distribute; computed inter-character
    // equivalence belongs to the downstream computation owner.
    let distribute = contributions(&checked("text-justify", "distribute", true));
    let inter_character = contributions(&checked("text-justify", "inter-character", true));
    assert_ne!(
        distribute.items()[0].ordinary_value(),
        inter_character.items()[0].ordinary_value()
    );
}
#[test]
fn group_alignment_admits_all_six_complete_keywords() {
    for text in GROUP {
        accepted("text-group-align", text, text);
    }
}
#[test]
fn duplicate_bases_modifiers_and_group_keywords_reject_with_atomic_sibling_recovery() {
    for name in FAMILY {
        for text in [
            "",
            "normal",
            "1",
            "-1px",
            "25%",
            "calc(1 + 2)",
            "min(1,2)",
            "[auto]",
            "\"auto\"",
            "auto,none",
        ] {
            invalid(name, text);
        }
    }
    for left in BASES {
        for right in BASES {
            invalid("text-justify", &format!("{left} {right}"));
            invalid("text-justify", &format!("{left} no-compress {right}"));
        }
        invalid("text-justify", &format!("no-compress {left} no-compress"));
    }
    invalid("text-justify", "no-compress no-compress");
    for left in GROUP {
        for right in GROUP {
            invalid("text-group-align", &format!("{left} {right}"));
        }
    }
    for text in ["auto", "ruby", "no-compress", "distribute"] {
        invalid("text-group-align", text);
    }
}
#[test]
fn escaped_case_and_comment_keywords_keep_original_sources_and_canonical_roles() {
    for (name, input, expected) in [
        (
            "text-justify",
            "NO-COMPRESS/**/INTER-WORD",
            "inter-word no-compress",
        ),
        (
            "text-justify",
            r"\64 istribute \6e o-compress",
            "distribute no-compress",
        ),
        ("text-justify", r"\6e o-compress", "no-compress"),
        ("text-group-align", r"\63 enter", "center"),
        ("text-group-align", "LEFT/**/", "left"),
    ] {
        accepted(name, input, expected);
    }
}
#[test]
fn each_single_terminal_isolated_from_group_alignment_and_spacing_siblings() {
    let report = CssParser::default().parse_block_contents("text-justify:no-compress!important;text-group-align:center;letter-spacing:2px;word-spacing:3px;color:red");
    assert!(report.is_clean());
    let names = [
        "text-justify",
        "text-group-align",
        "letter-spacing",
        "word-spacing",
        "color",
    ];
    for (source, name) in report.syntax().iter().zip(names) {
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            source.expand().unwrap()
        else {
            panic!("intrinsic single terminal")
        };
        assert_eq!(values.items().len(), 1);
        assert_eq!(values.items()[0].property(), property(name));
        assert_source(&values.items()[0], source);
    }
}
#[test]
fn sheet_siblings_share_exact_work_and_non_ascii_final_bytes() {
    // Published rule tariff: sheet 1; each class rule 4 structural nodes;
    // declaration/name 2; two and one explicit keyword leaves respectively.
    sheet_limits(
        ".é{text-justify:no-compress ruby!important}.b{text-group-align:center}",
        ".é { text-justify: ruby no-compress !important; }\n.b { text-group-align: center; }",
        16,
    );
}
