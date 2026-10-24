#![forbid(unsafe_code)]

//! Native default-ignorable behavior through the public Text protocol.
//! Oracles: unsupported DI has no paint or independent advance/spacing;
//! actual font operations retain surviving normal descendants and joint output.
//! Font metrics and table actions are independently authored and qualified.

#[path = "default_ignorable/font.rs"]
mod font;

use font::{A, Action, B, D, Q, R, SPACE, fixture, native_glyphs};
use surgeist_text as text;

const DI: char = '\u{2065}';
const LTR: text::ParagraphBaseDirection = text::ParagraphBaseDirection::LeftToRight;
const RTL: text::ParagraphBaseDirection = text::ParagraphBaseDirection::RightToLeft;

#[derive(Debug)]
struct Glyph {
    id: u32,
    advance: f32,
    pen: f64,
    origin_x: f64,
    x: f32,
    y: f32,
    range: text::Range,
    ink: bool,
}

#[derive(Debug)]
struct Observation {
    glyphs: Vec<Glyph>,
    units: Vec<(text::Range, f64)>,
    advance: f64,
    ink: text::LineOutlineInk,
    line_height: f64,
}

fn observe(
    bytes: &[u8],
    value: &str,
    spacing: f64,
    direction: text::ParagraphBaseDirection,
) -> Observation {
    let source = text::Source::new(value);
    let end = source.text().len();
    let analyzed = text::AnalyzedSource::from_paragraph(text::ParagraphInput {
        source: source.clone(),
        base_style: text::Style {
            font: text::Font::new().family("Ahem"),
            size: 10.0,
            line_height: text::LineHeight::Absolute(10.0),
            white_space: text::WhiteSpace::Preserve,
            wrap: text::Wrap::None,
            letter_spacing: spacing,
            ..text::Style::default()
        },
        segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: direction,
        paragraph_strut: text::ParagraphStrut::DefaultFont,
        terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![text::ParagraphOrigin {
            text_node: None,
            source,
            local: text::Range::new(0, end),
            aggregate: text::Range::new(0, end),
            kind: text::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    })
    .unwrap();
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let units = prepared
        .unit_views()
        .map(|unit| (unit.source_range(), unit.advance()))
        .collect();
    let end_offer = prepared
        .opportunity_views()
        .find(|offer| offer.kind() == text::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let line = prepared
        .select(text::LineSelectionRequest {
            incoming: None,
            start: prepared.boundary_views().next().unwrap().handle(),
            end: end_offer.view().boundary(),
            termination: text::PreparedTermination::Ordinary(end_offer),
            alternative: None,
            participants: prepared.participants().collect(),
            atomics: vec![],
            controls: vec![],
            line_strut: None,
            context: text::LineContext {
                base_direction: direction,
                orientation: text::LineOrientation::Horizontal,
                start: text::LineStart::Paragraph,
                end: text::LineEnd::Paragraph,
                available_inline: Some(1000.0),
                indent: 0.0,
                alignment: text::Alignment::Start,
                tabs: text::TabContext::NoTabs,
                spacing: text::SpacingPolicy::Spaces,
            },
        })
        .unwrap()
        .finalize(text::SpacingAllocations::None)
        .unwrap();
    let feedback = line.feedback();
    let mut glyphs = Vec::new();
    for run in line.run_views() {
        assert_eq!(
            run.face().bytes(),
            bytes,
            "must use the actual registered derivative"
        );
        assert_eq!(run.face().index(), 0);
        let instance = run.effective_instance().unwrap();
        for glyph in run.glyphs() {
            let outline = instance
                .glyph_outline(glyph.id(), run.face().catalogue_identity())
                .unwrap();
            glyphs.push(Glyph {
                id: glyph.id(),
                advance: glyph.advance(),
                pen: glyph.pen(),
                origin_x: run.origin().x,
                x: glyph.x(),
                y: glyph.y(),
                range: glyph.range(),
                ink: outline
                    .bounds()
                    .is_some_and(|bounds| bounds.size.width > 0.0 && bounds.size.height > 0.0),
            });
        }
    }
    let ink = line.outline_ink().unwrap();
    let layout = text::Layout::from_finalized_lines(text::PlacementRequest {
        prepared,
        coverage: text::PlacementCoverage::WholeParagraph,
        lines: vec![text::LinePlacement {
            line,
            anchor: text::LineAnchor {
                inline_origin: 0.0,
                baseline: 8.0,
            },
            orientation: text::LineOrientation::Horizontal,
            atomics: vec![],
        }],
    })
    .unwrap();
    let utf16: Vec<_> = value.encode_utf16().collect();
    assert_eq!(layout.source().text().code_units(), utf16);
    let positions: Vec<_> = layout.positions().collect();
    let before = positions
        .iter()
        .find(|position| position.source_cursor() == text::Cursor::new(0, text::Affinity::After))
        .unwrap();
    let after = positions
        .iter()
        .find(|position| position.source_cursor() == text::Cursor::new(end, text::Affinity::Before))
        .unwrap();
    let selection = layout.position_selection(before, after).unwrap();
    assert_eq!(selection.source_text().code_units(), utf16);
    assert_eq!(selection.plaintext().code_units(), utf16);
    Observation {
        glyphs,
        units,
        advance: feedback.advance,
        ink,
        line_height: feedback.line_height,
    }
}

fn ids_and_advances(value: &Observation) -> Vec<(u32, f32)> {
    value
        .glyphs
        .iter()
        .map(|glyph| (glyph.id, glyph.advance))
        .collect()
}

fn qualify_literals() {
    let bytes = fixture(Some(DI), Action::None);
    for (literal, id, advance) in [
        ("A", A, 10.0),
        ("B", B, 20.0),
        ("C", D, 7.0),
        ("R", R, 15.0),
        ("Q", Q, 25.0),
        (" ", SPACE, 10.0),
    ] {
        let value = observe(&bytes, literal, 0.0, LTR);
        assert_eq!(ids_and_advances(&value), [(u32::from(id), advance)]);
        assert!(
            value.glyphs[0].ink,
            "literal {literal:?} must have real ink: {value:?}"
        );
        assert_eq!(
            (value.glyphs[0].pen, value.glyphs[0].x, value.glyphs[0].y),
            (0.0, 0.0, 0.0)
        );
        assert_eq!(value.glyphs[0].range, text::Range::new(0, 1));
        assert_eq!(value.advance, f64::from(advance));
    }
}

fn qualified(action: Action, mapped: Option<char>) -> Vec<u8> {
    qualify_literals();
    let bytes = fixture(mapped, action);
    // Every table has actual positive-output controls in addition to deletion.
    let (ordinary, native_expected): (&str, &[(u32, i32)]) = match action {
        Action::None => (
            "ABRQ ",
            &[(35, 1000), (36, 2000), (58, 1500), (277, 2500), (274, 1000)],
        ),
        Action::Delete | Action::ReplaceThenDelete | Action::DeleteBoth => {
            ("ACB", &[(35, 1000), (36, 2000)])
        }
        Action::PartialDelete => ("C", &[(58, 1500)]),
        Action::Consume => ("ACB", &[(58, 1500), (36, 2000)]),
        Action::Skip => ("AB", &[(58, 1500)]),
    };
    assert_eq!(
        native_glyphs(&bytes, ordinary),
        native_expected,
        "real {action:?} GSUB control"
    );
    let value = observe(&bytes, ordinary, 0.0, LTR);
    let expected: Vec<_> = native_expected
        .iter()
        .map(|&(id, advance)| (id, advance as f32 / 100.0))
        .collect();
    assert_eq!(
        ids_and_advances(&value),
        expected,
        "public literal table control {action:?}"
    );
    assert!(value.glyphs.iter().all(|glyph| glyph.ink));
    bytes
}

fn check_empty(value: &Observation, label: &str, failures: &mut Vec<String>) {
    if !value.glyphs.is_empty()
        || value.ink != text::LineOutlineInk::Empty
        || value.advance != 0.0
        || value.units.iter().any(|unit| unit.1 != 0.0)
        || value.line_height != 10.0
    {
        failures.push(format!(
            "{label}: expected no paint glyph/ink, zero input allocation and spacing, real strut10; got {value:?}"
        ));
    }
}

fn check_ab(
    value: &Observation,
    units: &[(text::Range, f64)],
    label: &str,
    failures: &mut Vec<String>,
) {
    // Pens are run-local; script/direction can split runs. Compare actual line
    // positions after composing the retained run origins, relative to A.
    let origin = value
        .glyphs
        .first()
        .map_or(0.0, |glyph| glyph.origin_x + glyph.pen);
    if ids_and_advances(value) != [(35, 10.0), (36, 20.0)]
        || value.advance != 36.0
        || value.units != units
        || value
            .glyphs
            .iter()
            .map(|glyph| (glyph.origin_x + glyph.pen - origin, glyph.x, glyph.y))
            .collect::<Vec<_>>()
            != [(0.0, 0.0, 0.0), (13.0, 0.0, 0.0)]
    {
        failures.push(format!(
            "{label}: expected only A10/B20, final36, units {units:?}; got {value:?}"
        ));
    }
}

#[test]
fn literal_outlines_metrics_and_real_ordered_tables_are_qualified() {
    for action in [
        Action::None,
        Action::Delete,
        Action::ReplaceThenDelete,
        Action::PartialDelete,
        Action::DeleteBoth,
        Action::Consume,
        Action::Skip,
    ] {
        qualified(action, Some(DI));
    }
}

fn residual(scalar: char, mapped: bool) {
    let bytes = qualified(Action::None, mapped.then_some(scalar));
    let mut failures = Vec::new();
    for direction in [LTR, RTL] {
        let label = format!(
            "U+{:04X}, mapped={mapped}, {direction:?}",
            u32::from(scalar)
        );
        check_empty(
            &observe(&bytes, &scalar.to_string(), 3.0, direction),
            &format!("lone {label}"),
            &mut failures,
        );
        let source = format!("A{scalar}B");
        let value = observe(&bytes, &source, 3.0, direction);
        let end = 1 + scalar.len_utf16();
        // PreparedUnit retains each original presentation atom, independently
        // of semantic grapheme/caret boundaries. These are source allocations;
        // no caret or selection boundary inside base+selector is required.
        let units = vec![
            (text::Range::new(0, 1), 13.0),
            (text::Range::new(1, end), 0.0),
            (text::Range::new(end, end + 1), 23.0),
        ];
        check_ab(&value, &units, &format!("middle {label}"), &mut failures);
        if scalar == DI {
            for (source, units) in [
                (
                    format!("{DI}AB"),
                    vec![
                        (text::Range::new(0, 1), 0.0),
                        (text::Range::new(1, 2), 13.0),
                        (text::Range::new(2, 3), 23.0),
                    ],
                ),
                (
                    format!("AB{DI}"),
                    vec![
                        (text::Range::new(0, 1), 13.0),
                        (text::Range::new(1, 2), 23.0),
                        (text::Range::new(2, 3), 0.0),
                    ],
                ),
            ] {
                check_ab(
                    &observe(&bytes, &source, 3.0, direction),
                    &units,
                    &format!("{source:?} {label}"),
                    &mut failures,
                );
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn absent_reserved_di_cannot_paint_visible_space_or_receive_spacing() {
    residual(DI, false);
}

#[test]
fn nominal_visible_reserved_di_is_still_residual() {
    residual(DI, true);
}

#[test]
fn unsupported_mongolian_fvs4_has_no_independent_paint_or_spacing() {
    residual('\u{180f}', false);
}

#[test]
fn unsupported_supplementary_selector_keeps_exact_utf16_without_paint_or_spacing() {
    residual('\u{e0100}', false);
}

#[test]
fn absent_shorthand_format_control_is_residual_without_native_action() {
    residual('\u{1bca0}', false);
}

#[test]
fn nominal_shorthand_format_control_is_residual_without_native_action() {
    residual('\u{1bca0}', true);
}

fn deletion(action: Action) {
    let bytes = qualified(action, Some(DI));
    // Preserve-mode native control proves physical disappearance, including later
    // deletion of transformed descendants; Text assertions use normal profile.
    assert!(native_glyphs(&bytes, &DI.to_string()).is_empty());
    let mut failures = Vec::new();
    for direction in [LTR, RTL] {
        check_empty(
            &observe(&bytes, &DI.to_string(), 3.0, direction),
            &format!("lone {action:?} {direction:?}"),
            &mut failures,
        );
        for (source, units) in [
            (
                format!("A{DI}B"),
                vec![
                    (text::Range::new(0, 1), 13.0),
                    (text::Range::new(1, 2), 0.0),
                    (text::Range::new(2, 3), 23.0),
                ],
            ),
            (
                format!("{DI}AB"),
                vec![
                    (text::Range::new(0, 1), 0.0),
                    (text::Range::new(1, 2), 13.0),
                    (text::Range::new(2, 3), 23.0),
                ],
            ),
            (
                format!("AB{DI}"),
                vec![
                    (text::Range::new(0, 1), 13.0),
                    (text::Range::new(1, 2), 23.0),
                    (text::Range::new(2, 3), 0.0),
                ],
            ),
        ] {
            let value = observe(&bytes, &source, 3.0, direction);
            check_ab(
                &value,
                &units,
                &format!("{source:?} {action:?} {direction:?}"),
                &mut failures,
            );
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn gsub_zero_output_di_has_zero_allocation_despite_merged_clusters() {
    deletion(Action::Delete);
}

#[test]
fn gsub_normal_replacement_then_deletion_retires_original_di() {
    deletion(Action::ReplaceThenDelete);
}

#[test]
fn deleting_both_gsub_descendants_retires_original_di() {
    deletion(Action::DeleteBoth);
}

#[test]
fn partial_descendant_deletion_preserves_surviving_normal_output() {
    let bytes = qualified(Action::PartialDelete, Some(DI));
    assert_eq!(native_glyphs(&bytes, &DI.to_string()), [(58, 1500)]);
    let value = observe(&bytes, &DI.to_string(), 3.0, LTR);
    assert_eq!(ids_and_advances(&value), [(58, 15.0)]);
    assert_eq!(value.advance, 18.0);
    assert_eq!(value.units, [(text::Range::new(0, 1), 18.0)]);
    assert!(value.glyphs[0].ink);
}

#[test]
fn actual_di_component_consumption_preserves_joint_output_and_metrics() {
    let bytes = qualified(Action::Consume, Some(DI));
    let source = format!("A{DI}B");
    assert_eq!(native_glyphs(&bytes, &source), [(58, 1500), (36, 2000)]);
    let value = observe(&bytes, &source, 0.0, LTR);
    assert_eq!(ids_and_advances(&value), [(58, 15.0), (36, 20.0)]);
    assert_eq!(value.advance, 35.0);
    assert_eq!(value.glyphs[0].range, text::Range::new(0, 2));
    assert!(value.glyphs.iter().all(|glyph| glyph.ink));
}

#[test]
fn skipped_residual_di_cannot_acquire_joint_output_advance_or_spacing() {
    let bytes = qualified(Action::Skip, Some(DI));
    let source = format!("A{DI}B");
    assert_eq!(native_glyphs(&bytes, &source), [(58, 1500), (100, 700)]);
    let value = observe(&bytes, &source, 3.0, LTR);
    assert_eq!(
        ids_and_advances(&value),
        [(58, 15.0)],
        "skip differs from consuming D: {value:?}"
    );
    assert_eq!(
        value.advance, 21.0,
        "ordinary A and B own normal joint allocation: {value:?}"
    );
    assert_eq!(
        value
            .units
            .iter()
            .find(|unit| unit.0 == text::Range::new(1, 2))
            .map(|unit| unit.1),
        Some(0.0)
    );
    assert_eq!(value.glyphs[0].range, text::Range::new(0, 3));
    assert!(value.glyphs[0].ink);
}
