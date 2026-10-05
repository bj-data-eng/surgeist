#![forbid(unsafe_code)]
//! Functional new-API coverage alongside the authored Grid implementation.
//! Grid 2 §7.2/§7.4/§7.8 owns subgrid/name-repeat, area defaults and six-member
//! projections. Values 4 owns symbolic Integer math and deferred range/rounding.
//! The adopted intrinsic enumeration and work accounting are public product
//! contracts; these tests do not impose used repetition or layout behavior.

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

const EXPLICIT: [P; 3] = [
    P::GridTemplateRows,
    P::GridTemplateColumns,
    P::GridTemplateAreas,
];
const GRID: [P; 6] = [
    P::GridTemplateRows,
    P::GridTemplateColumns,
    P::GridTemplateAreas,
    P::GridAutoRows,
    P::GridAutoColumns,
    P::GridAutoFlow,
];

fn parsed(property: P, value: &str) -> CssDeclaration {
    let css = format!("/*😀*/{}:{value}!important", property.canonical_name());
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert!(validate_style_attribute(&css).is_ok());
    let [source] = report.syntax().as_slice() else {
        panic!("one occurrence");
    };
    source.clone()
}

fn checked(property: P, value: &str, grammar: bool) -> CssDeclaration {
    let components = parse_component_values(value).unwrap();
    let before = components.clone();
    let source = if grammar {
        parse_property_value_for_grammar(
            property.grammar(),
            components.clone(),
            CssImportance::Important,
        )
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(property),
            components.clone(),
            CssImportance::Important,
        )
    }
    .unwrap_or_else(|error| panic!("{}:{value}: {error:?}", property.canonical_name()));
    assert_eq!(components, before);
    assert_eq!(source.value_components(), &components);
    assert!(source.position().is_none());
    source
}

fn fronts(property: P, value: &str) -> [CssDeclaration; 3] {
    [
        parsed(property, value),
        checked(property, value, false),
        checked(property, value, true),
    ]
}

fn axis(source: &CssDeclaration) -> &CssGridTrackList {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::GridTemplateRows(v) => v.value(),
        CssKnownPropertyValueRef::GridTemplateColumns(v) => v.value(),
        _ => panic!("axis property"),
    }
}

fn template(source: &CssDeclaration) -> &CssGridTemplate {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::GridTemplate(v) => v.value(),
        CssKnownPropertyValueRef::Grid(v) => v.value().template_value().unwrap(),
        _ => panic!("template alternative"),
    }
}

fn complete(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("complete terminals");
    };
    for item in values.items() {
        context(item, source, None);
    }
    values
}

fn context(
    item: &CssLonghandContribution,
    source: &CssDeclaration,
    replacement: Option<&CssComponentValues>,
) {
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    assert_eq!(item.source().position(), source.position());
    assert_eq!(
        item.source().known().unwrap().grammar(),
        source.known().unwrap().grammar()
    );
    assert_eq!(item.source().value_components(), source.value_components());
    assert_eq!(item.replacement_components(), replacement);
}

fn output(
    value: CssLonghandValueRef<'_>,
    limits: Limits,
) -> Result<String, CssSpecifiedValueSerializationError> {
    match value {
        CssLonghandValueRef::GridTemplateRows(v) | CssLonghandValueRef::GridTemplateColumns(v) => {
            v.serialize_specified_with_limits(limits)
        }
        CssLonghandValueRef::GridTemplateAreas(v) => v.serialize_specified_with_limits(limits),
        CssLonghandValueRef::GridAutoRows(v) | CssLonghandValueRef::GridAutoColumns(v) => {
            v.serialize_specified_with_limits(limits)
        }
        CssLonghandValueRef::GridAutoFlow(v) => v.serialize_specified_with_limits(limits),
        _ => panic!("Grid terminal"),
    }
}

fn projected_axis(values: &CssLonghandContributions, index: usize) -> &CssGridTrackList {
    match values.items()[index].value() {
        CssContributionValueRef::Ordinary(
            CssLonghandValueRef::GridTemplateRows(v) | CssLonghandValueRef::GridTemplateColumns(v),
        ) => v,
        _ => panic!("borrowed explicit axis"),
    }
}

fn assert_values(values: &CssLonghandContributions, expected: &[&str]) {
    assert_eq!(values.items().len(), expected.len());
    let members: &[P] = if expected.len() == 3 {
        &EXPLICIT
    } else {
        &GRID
    };
    for ((item, property), expected) in values.items().iter().zip(members).zip(expected) {
        assert_eq!(item.property(), *property);
        let typed = item.ordinary_value().unwrap();
        assert_eq!(typed.property().known_property(), *property);
        assert_eq!(output(typed.view(), Limits::default()).unwrap(), *expected);
    }
}

fn names(items: &[&str]) -> CssGridLineNames {
    CssGridLineNames::new(
        items
            .iter()
            .map(|name| CssGridLineName::try_new(CssIdent::try_new(*name).unwrap()).unwrap())
            .collect(),
    )
}
fn auto() -> CssGridTrackSize {
    CssGridTrackSize::from_breadth(CssGridTrackBreadth::auto())
}
fn size_px(text: &str) -> CssGridTrackSize {
    CssGridTrackSize::from_breadth(CssGridTrackBreadth::from_length_percentage(
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(
            CssComponentValue::try_dimension(text, "px").unwrap(),
        )
        .unwrap(),
    ))
}
fn track_content() -> CssGridTrackRepeatContent {
    CssGridTrackRepeatContent::try_new(vec![CssGridTrackRepeatComponent::TrackSize(auto())])
        .unwrap()
}
fn fixed_content() -> CssGridFixedRepeatContent {
    CssGridFixedRepeatContent::try_new(vec![CssGridFixedRepeatComponent::FixedSize(
        CssGridFixedSize::try_new(size_px("1")).unwrap(),
    )])
    .unwrap()
}
fn calculation(text: &str) -> CssPositiveIntegerValue {
    CssPositiveIntegerValue::Calculation(
        CssIntegerCalculation::try_from_components(parse_component_values(text).unwrap()).unwrap(),
    )
}
fn row(cells: &[&str]) -> CssGridTemplateAreaRow {
    CssGridTemplateAreaRow::try_new(
        cells
            .iter()
            .map(|cell| {
                if *cell == "." {
                    CssGridTemplateAreaCell::Empty
                } else {
                    CssGridTemplateAreaCell::Named(CssGridTemplateAreaName::try_new(*cell).unwrap())
                }
            })
            .collect(),
    )
    .unwrap()
}

fn find_function<'a>(components: &'a CssComponentValues, name: &str) -> &'a CssComponentValue {
    for component in components.items() {
        if let CssComponentValueRef::Function(value) = component.view() {
            if value.name().eq_ignore_ascii_case(name) {
                return component;
            }
            if let Some(found) = find_function_optional(value.values(), name) {
                return found;
            }
        }
    }
    panic!("original {name} function");
}
fn find_function_optional<'a>(
    components: &'a CssComponentValues,
    name: &str,
) -> Option<&'a CssComponentValue> {
    for component in components.items() {
        if let CssComponentValueRef::Function(value) = component.view() {
            if value.name().eq_ignore_ascii_case(name) {
                return Some(component);
            }
            if let Some(found) = find_function_optional(value.values(), name) {
                return Some(found);
            }
        }
    }
    None
}
fn original_math(value: &CssPositiveIntegerValue, components: &CssComponentValues) {
    let CssPositiveIntegerValue::Calculation(value) = value else {
        panic!("symbolic function count");
    };
    let root = find_function(components, "calc");
    let actual = find_function(value.components(), "calc");
    assert_eq!(actual, root);
    assert_eq!(actual.origin(), root.origin());
    let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(expected)) =
        (actual.origin(), root.origin())
    else {
        panic!("original parsed math");
    };
    assert!(actual.source().same_snapshot(expected.source()));
    assert_eq!(actual.span(), expected.span());
}

#[test]
fn axis_initials_and_shorthand_members_expose_complete_typed_contracts() {
    let none = CssGridTrackList::none();
    assert!(none.is_none());
    assert!(none.general_list().is_none());
    assert!(none.auto_list().is_none());
    assert!(none.subgrid_components().is_none());
    assert_eq!(none.serialize_specified().unwrap(), "none");
    for property in [P::GridTemplateRows, P::GridTemplateColumns] {
        let CssPropertyKindRef::Longhand(metadata) = property.metadata().unwrap().kind() else {
            panic!("longhand");
        };
        assert!(!metadata.inherited_by_default());
        let initial = metadata.initial_value();
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("typed initial");
        };
        match value.view() {
            CssLonghandValueRef::GridTemplateRows(v)
            | CssLonghandValueRef::GridTemplateColumns(v) => assert_eq!(v, &none),
            _ => panic!("owning borrowed initial variant"),
        }
        for source in fronts(property, "none") {
            let values = complete(&source);
            assert_eq!(values.items().len(), 1);
            assert_eq!(values.items()[0].property(), property);
            assert_eq!(projected_axis(&values, 0), axis(&source));
        }
    }
    for (property, expected) in [
        (P::GridTemplate, EXPLICIT.as_slice()),
        (P::Grid, GRID.as_slice()),
    ] {
        let CssPropertyKindRef::Shorthand(metadata) = property.metadata().unwrap().kind() else {
            panic!("shorthand");
        };
        assert_eq!(
            metadata
                .members()
                .iter()
                .map(|p| p.known_property())
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(metadata.settable_members(), metadata.members());
        assert!(metadata.reset_only_members().is_empty());
    }
    assert!(CssGridTemplate::none().is_none());
    let template = CssGridTemplate::rows_columns(none.clone(), none);
    assert!(template.rows().unwrap().is_none());
    assert!(template.columns().unwrap().is_none());
    assert!(template.area_rows().is_none());
    assert!(template.area_columns().is_none());
    assert_eq!(
        CssGrid::template(template).serialize_specified().unwrap(),
        "none / none"
    );
}

#[test]
fn counted_repeat_constructors_normalize_bare_roots_without_evaluating_functions() {
    for text in ["0", "-0", "-1", "-999999999999999999999999999999"] {
        let count = calculation(text);
        assert!(CssGridIntegerTrackRepeat::try_new(count.clone(), track_content()).is_none());
        assert!(CssGridIntegerFixedRepeat::try_new(count.clone(), fixed_content()).is_none());
        assert!(CssGridNameRepeat::try_new(count, vec![names(&[])]).is_none());
    }
    for text in ["+0007", "2147483648", "999999999999999999999999999999"] {
        let count = calculation(text);
        let CssPositiveIntegerValue::Calculation(original) = &count else {
            unreachable!();
        };
        let component = original.components().items()[0].clone();
        let track = CssGridIntegerTrackRepeat::try_new(count.clone(), track_content()).unwrap();
        let fixed = CssGridIntegerFixedRepeat::try_new(count.clone(), fixed_content()).unwrap();
        let name = CssGridNameRepeat::try_new(count, vec![names(&[])]).unwrap();
        for value in [track.count(), fixed.count(), name.count().unwrap()] {
            let CssPositiveIntegerValue::Literal(value) = value else {
                panic!("normalized ordinary positive token");
            };
            // The source of the constructor's calculation is retained, rather
            // than a new numeric token being fabricated from a machine count.
            assert_eq!(value.integer().component(), &component);
            let CssValueOrigin::Parsed(origin) = value.integer().origin() else {
                panic!("parsed count origin");
            };
            assert_eq!(origin.source().as_str(), text);
            assert_eq!(origin.span().start().byte_offset().value(), 0);
            assert_eq!(origin.span().end().byte_offset().value(), text.len());
            assert_eq!(value.integer().numeric().representation(), text);
        }
    }
    for text in ["calc(2)", "calc(1.5)", "calc(0)", "calc(-1)"] {
        let count = calculation(text);
        let CssPositiveIntegerValue::Calculation(original) = &count else {
            unreachable!();
        };
        for value in [
            CssGridIntegerTrackRepeat::try_new(count.clone(), track_content())
                .unwrap()
                .count()
                .clone(),
            CssGridIntegerFixedRepeat::try_new(count.clone(), fixed_content())
                .unwrap()
                .count()
                .clone(),
            CssGridNameRepeat::try_new(count.clone(), vec![names(&[])])
                .unwrap()
                .count()
                .unwrap()
                .clone(),
        ] {
            assert_eq!(value.serialize_specified().unwrap(), text);
            let CssPositiveIntegerValue::Calculation(retained) = value else {
                panic!("function graph remains symbolic");
            };
            assert_eq!(retained.components(), original.components());
            assert_eq!(retained.origin(), original.origin());
        }
    }
    for text in ["1.5", "calc(1px)", "calc(1 +)", "1 2"] {
        assert!(
            CssIntegerCalculation::try_from_components(parse_component_values(text).unwrap())
                .is_err(),
            "{text}"
        );
    }
    for value in [
        CssIntegerCalculation::literal(0),
        CssIntegerCalculation::literal(-2),
    ] {
        assert!(
            CssGridIntegerTrackRepeat::try_new(
                CssPositiveIntegerValue::Calculation(value),
                track_content()
            )
            .is_none()
        );
    }
    let programmatic = CssPositiveIntegerValue::Calculation(CssIntegerCalculation::literal(2));
    let track = CssGridIntegerTrackRepeat::try_new(programmatic.clone(), track_content()).unwrap();
    let fixed = CssGridIntegerFixedRepeat::try_new(programmatic.clone(), fixed_content()).unwrap();
    let name = CssGridNameRepeat::try_new(programmatic, vec![names(&[])]).unwrap();
    for count in [track.count(), fixed.count(), name.count().unwrap()] {
        let CssPositiveIntegerValue::Literal(count) = count else {
            panic!("programmatic positive bare root");
        };
        assert_eq!(count.integer().numeric().representation(), "2");
        assert_eq!(count.integer().origin(), &CssValueOrigin::Programmatic);
    }
}

#[test]
fn every_repeat_role_retains_parsed_function_children_and_exact_count_origins() {
    for property in [P::GridTemplateRows, P::GridTemplateColumns] {
        for text in ["calc(2)", "calc(1.5)", "calc(0)", "calc(-1)"] {
            for (css, mode) in [
                (format!("repeat({text}, auto)"), 0),
                (format!("repeat({text}, 1px) repeat(auto-fill, auto)"), 1),
                (format!("subgrid repeat({text}, [] [a a])"), 2),
            ] {
                for source in fronts(property, &css) {
                    let value = axis(&source);
                    let count = match mode {
                        0 => {
                            let [CssGridGeneralTrackComponent::Repeat(v)] =
                                value.general_list().unwrap().components()
                            else {
                                panic!("track repeat");
                            };
                            v.count()
                        }
                        1 => {
                            let CssGridAutoTrackComponent::Repeat(v) =
                                &value.auto_list().unwrap().components()[0]
                            else {
                                panic!("fixed repeat");
                            };
                            v.count()
                        }
                        _ => {
                            let [CssGridSubgridComponent::Repeat(v)] =
                                value.subgrid_components().unwrap()
                            else {
                                panic!("name repeat");
                            };
                            assert_eq!(v.groups().len(), 2);
                            v.count().unwrap()
                        }
                    };
                    original_math(count, source.value_components());
                    assert_eq!(value.serialize_specified().unwrap(), css);
                    let projected = complete(&source);
                    assert_eq!(projected_axis(&projected, 0), value);
                    let projected_count = match mode {
                        0 => {
                            let [CssGridGeneralTrackComponent::Repeat(v)] =
                                projected_axis(&projected, 0)
                                    .general_list()
                                    .unwrap()
                                    .components()
                            else {
                                unreachable!();
                            };
                            v.count()
                        }
                        1 => {
                            let CssGridAutoTrackComponent::Repeat(v) =
                                &projected_axis(&projected, 0)
                                    .auto_list()
                                    .unwrap()
                                    .components()[0]
                            else {
                                unreachable!();
                            };
                            v.count()
                        }
                        _ => {
                            let [CssGridSubgridComponent::Repeat(v)] =
                                projected_axis(&projected, 0).subgrid_components().unwrap()
                            else {
                                unreachable!();
                            };
                            v.count().unwrap()
                        }
                    };
                    original_math(projected_count, source.value_components());
                }
            }
        }
        for source in fronts(property, "repeat(+0007, auto)") {
            let [CssGridGeneralTrackComponent::Repeat(value)] =
                axis(&source).general_list().unwrap().components()
            else {
                panic!("ordinary repeat");
            };
            let CssPositiveIntegerValue::Literal(count) = value.count() else {
                panic!("ordinary count");
            };
            let CssValueOrigin::Parsed(origin) = count.integer().origin() else {
                panic!("parsed exact token");
            };
            let text = origin.source().as_str();
            let start = text.find("+0007").unwrap();
            assert_eq!(origin.span().start().byte_offset().value(), start);
            assert_eq!(origin.span().end().byte_offset().value(), start + 5);
            assert_eq!(count.integer().numeric().representation(), "+0007");
        }
    }
}

#[test]
fn subgrid_views_keep_long_ordered_lists_empty_groups_and_one_automatic_repeat() {
    let mut components = Vec::new();
    let mut spelling = Vec::new();
    for index in 0usize..20 {
        if index.is_multiple_of(2) {
            components.push(CssGridSubgridComponent::LineNames(names(&[])));
            spelling.push("[]".to_owned());
        } else {
            let name = format!("N{index}");
            components.push(CssGridSubgridComponent::LineNames(names(&[&name, &name])));
            spelling.push(format!("[{name} {name}]"));
        }
    }
    components.insert(
        8,
        CssGridSubgridComponent::Repeat(
            CssGridNameRepeat::try_auto_fill(vec![names(&[]), names(&["a b"])]).unwrap(),
        ),
    );
    spelling.insert(8, r"repeat(auto-fill, [] [a\ b])".to_owned());
    let expected = format!("subgrid {}", spelling.join(" "));
    let programmatic = CssGridTrackList::try_subgrid(components.clone()).unwrap();
    assert_eq!(programmatic.serialize_specified().unwrap(), expected);
    for property in [P::GridTemplateRows, P::GridTemplateColumns] {
        for source in fronts(property, &expected) {
            let value = axis(&source);
            assert!(!value.is_none());
            assert!(value.general_list().is_none());
            assert!(value.auto_list().is_none());
            assert_eq!(value.subgrid_components().unwrap(), components);
            assert_eq!(value.subgrid_components().unwrap().len(), 21);
            for (actual, expected) in value.subgrid_components().unwrap().iter().zip(&components) {
                assert_eq!(actual, expected);
            }
            let values = complete(&source);
            assert_eq!(
                projected_axis(&values, 0).subgrid_components(),
                value.subgrid_components()
            );
        }
        for source in fronts(property, "subgrid") {
            assert_eq!(axis(&source).subgrid_components(), Some([].as_slice()));
        }
    }
    assert!(CssGridNameRepeat::try_auto_fill(vec![]).is_none());
    assert!(CssGridNameRepeat::try_new(calculation("calc(2)"), vec![]).is_none());
    let automatic = CssGridNameRepeat::try_auto_fill(vec![names(&[])]).unwrap();
    assert!(automatic.is_auto_fill());
    assert!(automatic.count().is_none());
    assert!(automatic.groups()[0].names().is_empty());
    assert!(
        CssGridTrackList::try_subgrid(vec![
            CssGridSubgridComponent::Repeat(automatic.clone()),
            CssGridSubgridComponent::Repeat(automatic)
        ])
        .is_none()
    );
}

#[test]
fn area_constructors_validate_the_matrix_and_preserve_authored_optional_fields() {
    let first = CssGridTemplateAreaTrack::new(
        row(&["a", "a"]),
        None,
        Some(names(&[])),
        Some(names(&["end", "end"])),
    );
    let second = CssGridTemplateAreaTrack::new(
        row(&["b", "b"]),
        Some(auto()),
        Some(names(&["start"])),
        None,
    );
    assert!(first.size().is_none());
    assert!(first.before().unwrap().names().is_empty());
    assert_eq!(first.after().unwrap().names().len(), 2);
    assert_eq!(
        second.size().unwrap().breadth().unwrap().kind(),
        CssGridTrackBreadthKind::Auto
    );
    let columns = CssGridTrackRepeatContent::try_new(vec![CssGridTrackRepeatComponent::TrackSize(
        size_px("1"),
    )])
    .unwrap();
    let value =
        CssGridTemplate::try_areas(vec![first.clone(), second.clone()], Some(columns.clone()))
            .unwrap();
    assert!(!value.is_none());
    assert!(value.rows().is_none());
    assert!(value.columns().is_none());
    assert_eq!(value.area_rows().unwrap(), &[first, second]);
    assert_eq!(value.area_columns(), Some(&columns));
    // Two cell columns with one authored explicit track are valid: matrix
    // validity does not impose a used track-count equality.
    assert_eq!(
        value.serialize_specified().unwrap(),
        r#"[] "a a" [end end] [start] "b b" auto / 1px"#
    );
    assert_eq!(
        CssGrid::template(value.clone()).template_value(),
        Some(&value)
    );
    assert_eq!(
        CssGridTemplate::try_areas(vec![], None).unwrap_err(),
        CssGridTemplateAreaError::MissingRows
    );
    let inconsistent = vec![
        CssGridTemplateAreaTrack::new(row(&["a", "a"]), None, None, None),
        CssGridTemplateAreaTrack::new(row(&["b"]), None, None, None),
    ];
    assert_eq!(
        CssGridTemplate::try_areas(inconsistent, None).unwrap_err(),
        CssGridTemplateAreaError::InconsistentWidths
    );
    let diagonal = vec![
        CssGridTemplateAreaTrack::new(row(&["a", "."]), None, None, None),
        CssGridTemplateAreaTrack::new(row(&[".", "a"]), None, None, None),
    ];
    assert_eq!(
        CssGridTemplate::try_areas(diagonal, None).unwrap_err(),
        CssGridTemplateAreaError::NonRectangular("a".to_owned())
    );
}

#[test]
fn area_branch_views_and_effective_boundaries_keep_defaults_order_and_numeric_children() {
    let css = r#"[top] "a a" [end end] [start] "b b" calc(1px + 2%) [bottom] / 10px"#;
    for property in [P::GridTemplate, P::Grid] {
        for source in fronts(property, css) {
            let value = template(&source);
            let rows = value.area_rows().unwrap();
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].area(), &row(&["a", "a"]));
            assert_eq!(rows[1].area(), &row(&["b", "b"]));
            assert!(rows[0].size().is_none());
            assert_eq!(rows[0].before(), Some(&names(&["top"])));
            assert_eq!(rows[0].after(), Some(&names(&["end", "end"])));
            assert_eq!(rows[1].before(), Some(&names(&["start"])));
            let original = find_function(source.value_components(), "calc");
            let size = rows[1]
                .size()
                .unwrap()
                .breadth()
                .unwrap()
                .length_percentage()
                .unwrap();
            assert_eq!(
                find_function(size.calculation().unwrap().components(), "calc"),
                original
            );
            let values = complete(&source);
            let expected = [
                "[top] auto [end end start] calc(2% + 1px) [bottom]",
                "10px",
                r#""a a" "b b""#,
                "auto",
                "auto",
                "normal",
            ];
            assert_values(
                &values,
                &expected[..if property == P::Grid { 6 } else { 3 }],
            );
            let components = projected_axis(&values, 0)
                .general_list()
                .unwrap()
                .components();
            let CssGridGeneralTrackComponent::TrackSize(default) = &components[1] else {
                panic!("omitted auto row");
            };
            assert_eq!(default, &auto());
            let CssGridGeneralTrackComponent::TrackSize(explicit) = &components[3] else {
                panic!("explicit numeric row");
            };
            assert_eq!(
                find_function(
                    explicit
                        .breadth()
                        .unwrap()
                        .length_percentage()
                        .unwrap()
                        .calculation()
                        .unwrap()
                        .components(),
                    "calc"
                ),
                original
            );
            // Values 4 specified math sorts percentages before dimensions;
            // the graph above still retains the authored operand order.
            assert_eq!(
                value.serialize_specified().unwrap(),
                r#"[top] "a a" [end end] [start] "b b" calc(2% + 1px) [bottom] / 10px"#
            );
        }
        for (css, expected, has_empty) in [
            (r#"[] "a" [] "b" []"#, "[] auto [] auto []", true),
            (r#""a" [shared] "b""#, "auto [shared] auto", false),
        ] {
            for source in fronts(property, css) {
                let value = template(&source);
                assert!(value.area_rows().unwrap()[1].before().is_none());
                let values = complete(&source);
                assert_eq!(
                    projected_axis(&values, 0).serialize_specified().unwrap(),
                    expected
                );
                assert!(projected_axis(&values, 1).is_none());
                if has_empty {
                    let CssGridGeneralTrackComponent::LineNames(group) =
                        &projected_axis(&values, 0)
                            .general_list()
                            .unwrap()
                            .components()[2]
                    else {
                        panic!("empty boundary retained");
                    };
                    assert!(group.names().is_empty());
                }
            }
        }
        let source = checked(property, r#""auto span" "1 inherit""#, true);
        let values = complete(&source);
        assert_eq!(
            output(
                values.items()[2].ordinary_value().unwrap().view(),
                Limits::default()
            )
            .unwrap(),
            r#""auto span" "1 inherit""#
        );
    }
}

#[test]
fn every_grid_alternative_projects_source_defined_axes_and_implicit_defaults() {
    for (css, expected) in [
        ("none", ["none", "none", "none", "auto", "auto", "normal"]),
        (
            "none / subgrid []",
            ["none", "subgrid []", "none", "auto", "auto", "normal"],
        ),
        (
            "10px / repeat(calc(2), auto)",
            [
                "10px",
                "repeat(calc(2), auto)",
                "none",
                "auto",
                "auto",
                "normal",
            ],
        ),
        (
            r#""a""#,
            ["auto", "none", r#""a""#, "auto", "auto", "normal"],
        ),
        (
            "auto-flow / subgrid [a]",
            ["none", "subgrid [a]", "none", "auto", "auto", "row"],
        ),
        (
            "dense auto-flow 1px 2px / none",
            ["none", "none", "none", "1px 2px", "auto", "row dense"],
        ),
        (
            "subgrid [] / auto-flow",
            ["subgrid []", "none", "none", "auto", "auto", "column"],
        ),
        (
            "none / auto-flow dense 1px 2px",
            ["none", "none", "none", "auto", "1px 2px", "column dense"],
        ),
    ] {
        for source in fronts(P::Grid, css) {
            assert_values(&complete(&source), &expected);
        }
    }
    let flow = CssGridAutoFlowMode::new(CssGridAutoFlowAxis::Column, true);
    let explicit =
        CssGridTrackList::try_subgrid(vec![CssGridSubgridComponent::LineNames(names(&[]))])
            .unwrap();
    let implicit = CssGridTrackSizeList::try_new(vec![size_px("3"), auto()]).unwrap();
    let value = CssGrid::from_auto_flow(flow, Some(implicit.clone()), explicit.clone());
    assert!(value.template_value().is_none());
    assert_eq!(value.auto_flow(), Some(flow));
    assert_eq!(value.auto_tracks(), Some(&implicit));
    assert_eq!(value.explicit_tracks(), Some(&explicit));
    assert_eq!(
        value.serialize_specified().unwrap(),
        "subgrid [] / auto-flow dense 3px auto"
    );
}

#[test]
fn recovered_numeric_children_remain_recovered_through_projection_and_typed_composition() {
    let css = r#".a{grid:"a" "b" calc(2px"#;
    let report = parse_sheet(css);
    assert!(!report.is_clean());
    assert!(validate_sheet(css).is_err());
    let CssRule::Style(rule) = &report.syntax().rules()[0] else {
        panic!("retained style rule");
    };
    let source = &rule.declarations()[0];
    let original = find_function(source.value_components(), "calc");
    let CssComponentValueRef::Function(function) = original.view() else {
        unreachable!();
    };
    let origin = function.closing_origin();
    let CssValueOrigin::ImplicitClosure { at, .. } = origin else {
        panic!("original zero-width EOF");
    };
    assert_eq!(at.source().as_str(), css);
    assert_eq!(at.span().start().byte_offset().value(), css.len());
    assert_eq!(at.span().end().byte_offset().value(), css.len());
    let values = complete(source);
    let CssGridGeneralTrackComponent::TrackSize(size) = &projected_axis(&values, 0)
        .general_list()
        .unwrap()
        .components()[1]
    else {
        panic!("late explicit size");
    };
    assert_eq!(
        find_function(
            size.breadth()
                .unwrap()
                .length_percentage()
                .unwrap()
                .calculation()
                .unwrap()
                .components(),
            "calc"
        ),
        original
    );
    assert_eq!(
        source.to_specified_css().unwrap(),
        r#"grid: "a" "b" calc(2px);"#
    );
    assert!(
        parse_property_value_for_grammar(
            P::Grid.grammar(),
            source.value_components().clone(),
            CssImportance::Normal
        )
        .is_err()
    );
    let before = report.clone();
    let normalized = normalize_report(&report).unwrap();
    assert!(!normalized.is_clean());
    assert_eq!(normalized.diagnostics(), report.diagnostics());
    assert_eq!(report, before);

    // Reusing a parser-admitted Integer child is typed composition. It retains
    // recovery; it cannot turn that child's original components into clean CSS.
    let count_report = parse_style_attribute("column-count:calc(2");
    assert!(!count_report.is_clean());
    let count_source = &count_report.syntax()[0];
    let CssKnownPropertyValueRef::ColumnCount(value) =
        count_source.known().unwrap().property_value().unwrap()
    else {
        panic!("integer provider");
    };
    let CssColumnCount::Count(count) = value.count() else {
        panic!("count");
    };
    original_math(count, count_source.value_components());
    let repeat = CssGridIntegerTrackRepeat::try_new(count.clone(), track_content()).unwrap();
    original_math(repeat.count(), count_source.value_components());
    let repeated = CssGridTrackList::general(
        CssGridGeneralTrackList::try_new(vec![CssGridGeneralTrackComponent::Repeat(repeat)])
            .unwrap(),
    );
    assert_eq!(
        repeated.serialize_specified().unwrap(),
        "repeat(calc(2), auto)"
    );
    let name = CssGridNameRepeat::try_new(count.clone(), vec![names(&[])]).unwrap();
    original_math(name.count().unwrap(), count_source.value_components());
    let fixed = CssGridIntegerFixedRepeat::try_new(count.clone(), fixed_content()).unwrap();
    original_math(fixed.count(), count_source.value_components());
    let CssPositiveIntegerValue::Calculation(count) = count else {
        panic!("calculation");
    };
    assert!(CssIntegerCalculation::try_from_components(count.components().clone()).is_err());
}

#[test]
fn pending_retries_keep_new_borrowed_payloads_and_original_replacement_identity() {
    for property in [
        P::GridTemplateRows,
        P::GridTemplateColumns,
        P::GridTemplate,
        P::Grid,
    ] {
        for source in fronts(property, "var(--grid)") {
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending");
            };
            for _ in 0..2 {
                for invalid in ["junk", "initial/*", "var(--residual)"] {
                    let components = parse_component_values(invalid).unwrap();
                    let before = components.clone();
                    let error = handle.reenter(components.clone()).unwrap_err();
                    if invalid.starts_with("var") {
                        assert!(matches!(
                            error.kind(),
                            CssExpansionErrorKind::ResidualSubstitution
                        ));
                    } else {
                        assert!(matches!(
                            error.kind(),
                            CssExpansionErrorKind::InvalidReplacement(_)
                        ));
                    }
                    assert_eq!(components, before);
                    assert!(handle.source().same_occurrence(&source));
                }
            }
            let text = if matches!(property, P::GridTemplateRows | P::GridTemplateColumns) {
                "subgrid repeat(calc(2), [a] [])"
            } else {
                r#""a" calc(2px) / 1px"#
            };
            let replacement = parse_component_values(text).unwrap();
            let before = replacement.clone();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("complete retry");
            };
            for item in values.items() {
                context(item, &source, Some(&replacement));
            }
            let rows = projected_axis(&values, 0);
            if matches!(property, P::GridTemplateRows | P::GridTemplateColumns) {
                let [CssGridSubgridComponent::Repeat(repeat)] = rows.subgrid_components().unwrap()
                else {
                    panic!("name repeat replacement");
                };
                original_math(repeat.count().unwrap(), &replacement);
            } else {
                let [CssGridGeneralTrackComponent::TrackSize(size)] =
                    rows.general_list().unwrap().components()
                else {
                    panic!("explicit replacement row size");
                };
                assert_eq!(
                    find_function(
                        size.breadth()
                            .unwrap()
                            .length_percentage()
                            .unwrap()
                            .calculation()
                            .unwrap()
                            .components(),
                        "calc"
                    ),
                    find_function(&replacement, "calc")
                );
            }
            assert_eq!(replacement, before);
            let ordinary = values.items()[0].ordinary_value().unwrap();
            let expected = if matches!(property, P::GridTemplateRows | P::GridTemplateColumns) {
                text
            } else {
                "calc(2px)"
            };
            for _ in 0..2 {
                assert_eq!(
                    output(
                        ordinary.view(),
                        Limits::new(usize::MAX, usize::MAX, expected.len() - 1)
                    )
                    .unwrap_err()
                    .kind(),
                    Kind::ByteLimit
                );
            }
            assert_eq!(
                output(
                    ordinary.view(),
                    Limits::new(usize::MAX, usize::MAX, expected.len())
                )
                .unwrap(),
                expected
            );
            let CssContributions::Longhands(retry) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("reusable handle after provider failure");
            };
            for item in retry.items() {
                context(item, &source, Some(&replacement));
            }
        }
    }
}

#[test]
fn new_branch_provider_limits_are_exact_atomic_and_cumulative_at_declaration_boundaries() {
    for (property, text, nodes) in [
        (P::GridTemplateRows, "none", 1),
        (P::GridTemplateColumns, "subgrid [] [a]", 4),
        (P::GridTemplateRows, "subgrid repeat(2, [a] [])", 6),
        (P::GridTemplate, r#""a""#, 3),
        (P::Grid, r#""a" auto / 1px"#, 6),
    ] {
        for source in fronts(property, text) {
            let before = source.clone();
            let serialize = |limits| match source.known().unwrap().property_value().unwrap() {
                CssKnownPropertyValueRef::GridTemplateRows(v) => {
                    v.value().serialize_specified_with_limits(limits)
                }
                CssKnownPropertyValueRef::GridTemplateColumns(v) => {
                    v.value().serialize_specified_with_limits(limits)
                }
                CssKnownPropertyValueRef::GridTemplate(v) => {
                    v.value().serialize_specified_with_limits(limits)
                }
                CssKnownPropertyValueRef::Grid(v) => {
                    v.value().serialize_specified_with_limits(limits)
                }
                _ => unreachable!(),
            };
            assert_eq!(
                serialize(Limits::new(nodes, nodes, text.len())).unwrap(),
                text
            );
            let declaration = format!("{}: {text} !important;", property.canonical_name());
            assert_eq!(
                source
                    .to_specified_css_with_limits(Limits::new(
                        nodes + 2,
                        nodes + 2,
                        declaration.len()
                    ))
                    .unwrap(),
                declaration
            );
            for (limits, declaration_limits, kind) in [
                (
                    Limits::new(nodes - 1, nodes, text.len()),
                    Limits::new(nodes + 1, nodes + 2, declaration.len()),
                    Kind::InputNodeLimit,
                ),
                (
                    Limits::new(nodes, nodes - 1, text.len()),
                    Limits::new(nodes + 2, nodes + 1, declaration.len()),
                    Kind::ProjectionNodeLimit,
                ),
                (
                    Limits::new(nodes, nodes, text.len() - 1),
                    Limits::new(nodes + 2, nodes + 2, declaration.len() - 1),
                    Kind::ByteLimit,
                ),
            ] {
                for _ in 0..2 {
                    assert_eq!(serialize(limits).unwrap_err().kind(), kind);
                    assert_eq!(
                        source
                            .to_specified_css_with_limits(declaration_limits)
                            .unwrap_err()
                            .kind(),
                        kind
                    );
                    assert_eq!(source, before);
                }
            }
            assert_eq!(source.to_specified_css().unwrap(), declaration);
        }
    }
}

#[test]
fn area_and_subgrid_siblings_share_sheet_byte_budget_and_normalization_terminals() {
    let css = r#".a{grid:"a"!important;grid-template:subgrid [] / none;grid-template-columns:none;grid:var(--g)}.b{grid:"b" / 1px}"#;
    let report = parse_sheet(css);
    assert!(report.is_clean());
    let before = report.clone();
    let expected = ".a { grid: \"a\" !important; grid-template: subgrid [] / none; grid-template-columns: none; grid: var(--g); }\n.b { grid: \"b\" / 1px; }";
    let exact = Limits::new(usize::MAX, usize::MAX, expected.len());
    assert_eq!(
        report.syntax().to_specified_css_with_limits(exact).unwrap(),
        expected
    );
    for _ in 0..2 {
        let error = report
            .syntax()
            .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, expected.len() - 1))
            .unwrap_err();
        assert_eq!(
            error.kind(),
            CssSpecifiedRuleSerializationErrorKind::Resource(Kind::ByteLimit)
        );
        assert_eq!(error.rule_index(), Some(1));
    }
    // 6+3+1+one whole pending+6 = 17 terminal work units, five declarations.
    let limits = CssNormalizationLimits::try_new(1, 2, 5, 17).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), limits).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| {
            if let CssNormalizedItem::Declaration(value) = item {
                Some(value)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(declarations.len(), 5);
    for (index, item) in declarations.iter().enumerate() {
        assert_eq!(item.order(), index);
        if index < 4 {
            assert!(
                item.selector_context()
                    .same_context(declarations[0].selector_context())
            );
        } else {
            assert!(
                !item
                    .selector_context()
                    .same_context(declarations[0].selector_context())
            );
        }
        match item.expansion() {
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                for terminal in values.items() {
                    context(terminal, item.source(), None);
                }
            }
            CssExpansion::Pending(handle) => {
                assert_eq!(index, 3);
                assert!(handle.source().same_occurrence(item.source()));
            }
            _ => panic!("Grid occurrence"),
        }
    }
    for _ in 0..2 {
        let error = normalize_sheet_with_limits(
            report.syntax(),
            CssNormalizationLimits::try_new(1, 2, 5, 16).unwrap(),
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded {
                resource: CssNormalizationResource::Contributions,
                limit: 16
            }
        );
        assert_eq!(error.declaration_order(), Some(4));
        assert!(
            error
                .declaration()
                .unwrap()
                .same_occurrence(declarations[4].source())
        );
    }
    assert_eq!(report, before);
    assert!(normalize_sheet_with_limits(report.syntax(), limits).is_ok());
    assert_eq!(
        report.syntax().to_specified_css_with_limits(exact).unwrap(),
        expected
    );
}
