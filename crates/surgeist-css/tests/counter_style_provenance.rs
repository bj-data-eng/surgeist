#![forbid(unsafe_code)]
//! Counter Styles 3 §§3 and 3.8 retain decoded identity separately from genuine
//! lexical origins. Parsing, normalization and projection never fabricate a
//! stylesheet host or resolve counter names/images.
use surgeist_css::*;

fn counter(source: &str) -> CssCounterStyleRule {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("one counter rule");
    };
    rule.clone()
}

fn text(origin: &CssParsedOrigin) -> &str {
    &origin.source().as_str()
        [origin.span().start().byte_offset().value()..origin.span().end().byte_offset().value()]
}

fn assert_coordinates(origin: &CssParsedOrigin, source: &str) {
    assert_eq!(origin.source().as_str(), source);
    let start = origin.span().start();
    let offset = start.byte_offset().value();
    let line_start = source[..offset]
        .rfind('\n')
        .map_or(0, |position| position + 1);
    assert_eq!(
        start.line().value() as usize,
        source[..offset]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
    );
    assert_eq!(
        start.column().value() as usize,
        source[line_start..offset].encode_utf16().count()
    );
}

fn assert_descriptor<T>(
    occurrence: &CssDescriptorOccurrence<T>,
    rule: &CssCounterStyleRule,
    source: &str,
    name: &str,
    value: &str,
) {
    let parsed_name = occurrence.parsed_name().unwrap();
    let parsed_value = occurrence.parsed_value().unwrap();
    assert_eq!(text(parsed_name), name);
    assert_eq!(text(parsed_value).trim(), value);
    assert_eq!(occurrence.position(), parsed_name.span().start());
    assert!(parsed_name.source().same_snapshot(parsed_value.source()));
    assert!(
        parsed_name
            .source()
            .same_snapshot(rule.parsed_name().unwrap().source())
    );
    assert_coordinates(parsed_name, source);
    assert_coordinates(parsed_value, source);
    assert_original_components(
        occurrence.value_components().unwrap(),
        parsed_value.source(),
    );
}

fn assert_original_components(values: &CssComponentValues, snapshot: &CssSourceSnapshot) {
    for component in values.items() {
        let CssValueOrigin::Parsed(origin) = component.origin() else {
            panic!("every authored component opener/token retains a parsed origin");
        };
        assert!(origin.source().same_snapshot(snapshot));
        assert_coordinates(origin, snapshot.as_str());
        match component.view() {
            CssComponentValueRef::Function(function) => {
                let CssValueOrigin::Parsed(closing) = function.closing_origin() else {
                    panic!("explicit function closing origin");
                };
                assert_eq!(text(closing), ")");
                assert!(closing.source().same_snapshot(snapshot));
                assert_original_components(function.values(), snapshot);
            }
            CssComponentValueRef::Block(block) => {
                let CssValueOrigin::Parsed(closing) = block.closing_origin() else {
                    panic!("explicit block closing origin");
                };
                assert!(closing.source().same_snapshot(snapshot));
                assert_original_components(block.values(), snapshot);
            }
            _ => {}
        }
    }
}

fn identifiers(occurrence: &CssDescriptorOccurrence<impl Sized>) -> Vec<(&str, &CssParsedOrigin)> {
    occurrence
        .value_components()
        .unwrap()
        .items()
        .iter()
        .filter_map(|component| {
            let CssComponentValueRef::Token(CssValueTokenRef::Ident(value)) = component.view()
            else {
                return None;
            };
            let CssValueOrigin::Parsed(origin) = component.origin() else {
                panic!("parsed identifier");
            };
            Some((value, origin))
        })
        .collect()
}

#[test]
fn rule_name_origin_excludes_trivia_and_preserves_escaped_decoded_identity() {
    let source = "/*😀*/\n@counter-style /* name */ \\31 Name { symbols: a; }";
    let rule = counter(source);
    assert_eq!(rule.name().as_str(), "1Name");
    let origin = rule.parsed_name().unwrap();
    assert_eq!(text(origin), r"\31 Name");
    assert_eq!(
        origin.span().start().byte_offset().value(),
        source.find(r"\31 Name").unwrap()
    );
    assert_coordinates(origin, source);
    assert_eq!(rule.name_specified_css().unwrap(), r"\31 Name");
    assert_eq!(text(rule.parsed_name().unwrap()), r"\31 Name");
}

#[test]
fn predefined_normalization_does_not_replace_original_name_or_reference_tokens() {
    let source = r"@counter-style UPPER-ROMAN { system: extends \44 ISC; fallback: \55 PPER-ROMAN; speak-as: \4c OWER-GREEK; }";
    let rule = counter(source);
    assert_eq!(rule.name().as_str(), "upper-roman");
    assert_eq!(text(rule.parsed_name().unwrap()), "UPPER-ROMAN");
    let system = rule.descriptors().system().unwrap();
    assert!(
        matches!(system.value(), CssCounterStyleSystem::Extends(name) if name.as_str() == "disc")
    );
    assert_descriptor(system, &rule, source, "system", r"extends \44 ISC");
    let names = identifiers(system);
    assert_eq!(names[1].0, "DISC");
    assert_eq!(text(names[1].1), r"\44 ISC");
    let fallback = rule.descriptors().fallback().unwrap();
    assert_eq!(fallback.as_str(), "upper-roman");
    assert_descriptor(fallback, &rule, source, "fallback", r"\55 PPER-ROMAN");
    assert_eq!(identifiers(fallback)[0].0, "UPPER-ROMAN");
    let spoken = rule.descriptors().speak_as().unwrap();
    assert!(
        matches!(spoken.value(), CssCounterStyleSpeakAs::CounterStyle(name) if name.as_str() == "lower-greek")
    );
    assert_descriptor(spoken, &rule, source, "speak-as", r"\4c OWER-GREEK");
    assert_eq!(identifiers(spoken)[0].0, "LOWER-GREEK");
}

#[test]
fn all_descriptor_kinds_retain_original_regions_and_component_origins() {
    let source = concat!(
        "/*😀*/\n@counter-style Rich { ",
        r"S\59 STEM: CYCLIC; negative: '(' ')'; prefix: linear-gradient(red, blue); suffix: '. '; ",
        "range: infinite -0001, +0001 infinite; pad: '_' +0003; fallback: Missing; ",
        r"symbols: 'text' \31 Name url(mark.svg); additive-symbols: 10 X, 0 N; speak-as: Voice; }"
    );
    let rule = counter(source);
    let descriptors = rule.descriptors();
    assert_descriptor(
        descriptors.system().unwrap(),
        &rule,
        source,
        r"S\59 STEM",
        "CYCLIC",
    );
    assert_descriptor(
        descriptors.negative().unwrap(),
        &rule,
        source,
        "negative",
        "'(' ')'",
    );
    assert_descriptor(
        descriptors.prefix().unwrap(),
        &rule,
        source,
        "prefix",
        "linear-gradient(red, blue)",
    );
    assert_descriptor(
        descriptors.suffix().unwrap(),
        &rule,
        source,
        "suffix",
        "'. '",
    );
    assert_descriptor(
        descriptors.range().unwrap(),
        &rule,
        source,
        "range",
        "infinite -0001, +0001 infinite",
    );
    assert_descriptor(
        descriptors.pad().unwrap(),
        &rule,
        source,
        "pad",
        "'_' +0003",
    );
    assert_descriptor(
        descriptors.fallback().unwrap(),
        &rule,
        source,
        "fallback",
        "Missing",
    );
    assert_descriptor(
        descriptors.symbols().unwrap(),
        &rule,
        source,
        "symbols",
        r"'text' \31 Name url(mark.svg)",
    );
    assert_descriptor(
        descriptors.additive_symbols().unwrap(),
        &rule,
        source,
        "additive-symbols",
        "10 X, 0 N",
    );
    assert_descriptor(
        descriptors.speak_as().unwrap(),
        &rule,
        source,
        "speak-as",
        "Voice",
    );
    let CssCounterStyleRange::Ranges(ranges) = descriptors.range().unwrap().value() else {
        panic!("ranges");
    };
    let CssCounterStyleRangeBound::Integer(integer) = ranges.ranges()[0].upper() else {
        panic!("integer bound");
    };
    let CssValueOrigin::Parsed(origin) = integer.origin() else {
        panic!("integer parsed origin");
    };
    assert_eq!(text(origin), "-0001");
    assert!(
        origin
            .source()
            .same_snapshot(rule.parsed_name().unwrap().source())
    );
}

#[test]
fn complete_nested_image_components_and_closings_keep_the_original_snapshot() {
    let source = r##"/*😀*/
@counter-style Images { symbols: "x" \31 Name light-dark(src("#mark" Hint mode([a])), linear-gradient(currentcolor calc(1px + 5%), blue)); }"##;
    let rule = counter(source);
    let symbols = rule.descriptors().symbols().unwrap();
    assert_descriptor(
        symbols,
        &rule,
        source,
        "symbols",
        r##""x" \31 Name light-dark(src("#mark" Hint mode([a])), linear-gradient(currentcolor calc(1px + 5%), blue))"##,
    );
    let components = symbols.value_components().unwrap();
    let image = components
        .items()
        .iter()
        .find_map(|component| match component.view() {
            CssComponentValueRef::Function(function) if function.name() == "light-dark" => {
                Some((component, function))
            }
            _ => None,
        })
        .unwrap();
    let CssValueOrigin::Parsed(opening) = image.0.origin() else {
        panic!("image opener");
    };
    assert_eq!(text(opening), "light-dark(");
    let CssValueOrigin::Parsed(closing) = image.1.closing_origin() else {
        panic!("image closing");
    };
    assert_eq!(text(closing), ")");
    let retained = rule.clone();
    assert!(rule.to_specified_css().unwrap().contains("calc(5% + 1px)"));
    assert_eq!(rule, retained);
    assert_original_components(components, rule.parsed_name().unwrap().source());
}

#[test]
fn suppressed_valid_duplicates_retain_origins_while_invalid_occurrences_are_dropped() {
    let source =
        r"@counter-style Duplicates { symbols:a; prefix:'first'; prefix:inherit; prefix:\4c ast; }";
    let report = parse_sheet(source);
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("retained rule");
    };
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid descriptor");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
    let prefixes: Vec<_> = rule
        .descriptors()
        .occurrences()
        .filter_map(|descriptor| match descriptor {
            CssCounterStyleDescriptorRef::Prefix(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_descriptor(prefixes[0], rule, source, "prefix", "'first'");
    assert_descriptor(prefixes[1], rule, source, "prefix", r"\4c ast");
    assert_eq!(
        rule.descriptor_specified_css(CssCounterStyleDescriptorKind::Prefix)
            .unwrap(),
        "Last"
    );
    assert_eq!(text(prefixes[0].parsed_value().unwrap()), "'first'");
    assert_eq!(text(prefixes[1].parsed_value().unwrap()), r"\4c ast");
    assert_eq!(rule.descriptors().prefix().unwrap(), prefixes[1]);
}

#[test]
fn eof_image_closure_is_implicit_provenance_and_never_an_authored_delimiter() {
    let source = "@counter-style Eof { symbols: linear-gradient(red, blue";
    let report = parse_sheet(source);
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("retained EOF rule");
    };
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure)
    );
    let symbols = rule.descriptors().symbols().unwrap();
    let function = symbols
        .value_components()
        .unwrap()
        .items()
        .iter()
        .find_map(|component| match component.view() {
            CssComponentValueRef::Function(function) => Some(function),
            _ => None,
        })
        .unwrap();
    let CssValueOrigin::ImplicitClosure { opening, at } = function.closing_origin() else {
        panic!("EOF closing provenance");
    };
    assert_eq!(text(opening), "linear-gradient(");
    assert_eq!(at.span().start().byte_offset().value(), source.len());
    assert_eq!(at.span().end().byte_offset().value(), source.len());
    assert!(opening.source().same_snapshot(at.source()));
    assert!(
        opening
            .source()
            .same_snapshot(rule.parsed_name().unwrap().source())
    );
    assert_eq!(
        text(symbols.parsed_value().unwrap()).trim(),
        "linear-gradient(red, blue"
    );
    assert_eq!(
        rule.descriptor_specified_css(CssCounterStyleDescriptorKind::Symbols)
            .unwrap(),
        "linear-gradient(red, blue)"
    );
    assert_eq!(
        validate_sheet(source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
}

#[test]
fn normalization_retains_counter_name_reference_and_image_origin_identity() {
    let source = r"/*😀*/ @media print { @counter-style \31 Name { system:extends Target; prefix:linear-gradient(red, blue); } } @scope { @counter-style Scoped { symbols:x; fallback:\4f ther; } }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let retained = report.syntax().clone();
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let counters: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(context) => match context.kind() {
                CssRuleContextKindRef::CounterStyle(rule) => Some(rule),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(text(counters[0].parsed_name().unwrap()), r"\31 Name");
    assert_descriptor(
        counters[0].descriptors().system().unwrap(),
        counters[0],
        source,
        "system",
        "extends Target",
    );
    assert_descriptor(
        counters[0].descriptors().prefix().unwrap(),
        counters[0],
        source,
        "prefix",
        "linear-gradient(red, blue)",
    );
    assert_descriptor(
        counters[1].descriptors().fallback().unwrap(),
        counters[1],
        source,
        "fallback",
        r"\4f ther",
    );
    assert!(
        counters[0]
            .parsed_name()
            .unwrap()
            .source()
            .same_snapshot(counters[1].parsed_name().unwrap().source())
    );
    assert_eq!(report.syntax(), &retained);
}

#[test]
fn direct_image_symbol_construction_excludes_none_and_keeps_url_identity() {
    assert_eq!(
        CssImage::try_new(CssImageValue::None).unwrap_err(),
        CssImageConstructionError::NotImage
    );
    let image = CssImage::try_new(CssImageValue::Url(CssUrl::from_parts(
        CssUrlFunction::Src,
        "#symbol",
        vec![CssUrlModifier::Ident(CssIdent::try_new("Hint").unwrap())],
    )))
    .unwrap();
    let symbol = CssCounterSymbol::Image(image);
    let CssCounterSymbol::Image(image) = symbol else {
        panic!("checked image symbol");
    };
    let CssImageValue::Url(url) = image.value() else {
        panic!("symbol URL payload");
    };
    assert_eq!(url.function(), CssUrlFunction::Src);
    assert_eq!(url.as_str(), "#symbol");
    assert!(url.is_local_url());
    assert_eq!(
        image.serialize_specified().unwrap(),
        "src(\"#symbol\" Hint)"
    );
}
