//! Native punctuation through actual capture, fitting, finalization and placement.
use super::*;
use crate::value::SizingOperation;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");

fn paragraph(source: &str, style: text::style::Style) -> PreparedParagraph {
    PreparedParagraph::plain(
        1,
        Source::identified(source, text::source::Id::from_u64(71072), 4),
        style,
    )
}

fn compute<S: LayoutScalar>(
    p: &PreparedParagraph,
    width: f64,
    zone: f64,
) -> CompletedLayoutBatchOf<u32, S> {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(width)),
                    PreferredSizeOf::AUTO,
                ),
                hyphenate_limit_zone: LengthPercentageOf::px(S::from_f64(zone)).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .input(1, p.text_input(1));
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![p.binding(0)],
        crate::inline_wrapping::WrappingInput {
            native_inline_boxes: Vec::new(),
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    compute_layout_with_flow(&capture, 0, {
        let containing_flow_axes = {
            let root_input = crate::tree::LayoutTree::node_input((capture).tree(), 0);
            crate::geometry::FlowAxes::new(root_input.writing_mode, root_input.direction)
        };
        let requested_axes = Size::new(
            crate::value::AxisSizingRequestOf::new(
                crate::value::AvailableSpaceOf::Definite(
                    crate::value::NonNegativeFiniteOf::new(S::from_f64(width)).unwrap(),
                ),
                SizingOperation::Normal,
            ),
            crate::value::AxisSizingRequestOf::new(
                crate::value::AvailableSpaceOf::Indefinite,
                SizingOperation::MaxContent,
            ),
        );
        let mut logical_requests = containing_flow_axes.logical_size(requested_axes);
        // These fixtures perform ordinary block-size layout independently of
        // the explicitly requested inline contribution operation.
        logical_requests.block = logical_requests
            .block
            .with_operation(SizingOperation::Normal);
        logical_requests.inline = logical_requests
            .inline
            .with_operation(SizingOperation::Normal);
        let axes = containing_flow_axes.physical_size(logical_requests);
        // The old Native NoParent fixture explicitly supplied these percentage
        // bases. It supplied no independent ICB dimensions or ancestor scrollport.
        let percentage_parent = requested_axes.map(|axis| axis.space().definite_size());
        let fallback = crate::value::OrthogonalFallbackOf::new(
            crate::geometry::Size::NONE,
            crate::value::NearestScrollportOf::Absent,
            crate::geometry::Size::splat(crate::value::OrthogonalFallbackLimitOf::Unbounded),
        );
        crate::output::LayoutRootRequestOf::standalone(
            crate::value::SizingRequestOf::new(axes, fallback),
            containing_flow_axes,
            percentage_parent,
        )
    })
    .unwrap()
}

fn lines<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    p: &PreparedParagraph,
) -> Vec<text::finalized::FinalizedLine> {
    let mut lines = vec![None; batch.placed_text()[0].layout().lines().len()];
    assert_eq!(
        batch.placed_text()[0].layout().source(),
        p.prepared.analyzed_source().source()
    );
    assert_eq!(
        p.origin(1).view().source().id(),
        Some(text::source::Id::from_u64(71072))
    );
    assert_eq!(p.origin(1).view().source().revision(), 4);
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let line = fragment.finalized_fragment().finalized_line();
        assert_eq!(line.prepared_text(), p.prepared);
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), AHEM);
            let instance = run.effective_instance().unwrap();
            assert_eq!(instance.font_size(), 10.0);
            assert_eq!(
                instance
                    .metrics(run.face().catalogue_identity())
                    .unwrap()
                    .units_per_em(),
                1000
            );
            for glyph in run.glyphs().filter(|glyph| glyph.advance() != 0.0) {
                assert_eq!(glyph.advance(), 10.0);
            }
        }
        lines[fragment.line_index()] = Some(line);
    }
    lines
        .into_iter()
        .map(|line| line.expect("each authored line keeps real source custody"))
        .collect()
}

fn end_geometry<S: LayoutScalar>() {
    for (mode, fit, minimum, maximum) in [
        (None, 20.0, 20.0, 20.0),
        (
            Some(text::style::HangingPunctuationEnd::ForceEnd),
            10.0,
            10.0,
            10.0,
        ),
        (
            Some(text::style::HangingPunctuationEnd::AllowEnd),
            10.0,
            10.0,
            20.0,
        ),
    ] {
        let style = text::style::Style {
            hanging_punctuation: text::style::HangingPunctuation::new(false, mode, false),
            wrap: Wrap::Word,
            ..ahem_style(10.0, 10.0)
        };
        let p = paragraph("A.", style);
        for width in [25.0, 15.0] {
            let batch = compute::<S>(&p, width, 0.0);
            let rows = lines(&batch, &p);
            assert_eq!(rows.len(), 1);
            let line = &rows[0];
            assert_eq!(line.feedback().advance, 20.0);
            assert_eq!(line.measurements().fit().advance(), fit);
            assert_eq!(
                line.measurements().min_content_contribution().advance(),
                minimum
            );
            assert_eq!(
                line.measurements().max_content_contribution().advance(),
                maximum
            );
            let alignment = match mode {
                Some(text::style::HangingPunctuationEnd::AllowEnd) => width.min(20.0),
                _ => fit,
            };
            assert_eq!(line.measurements().alignment().advance(), alignment);
            let mut positions = Vec::new();
            for run in batch.placed_text()[0].layout().run_views() {
                let origin = run.origin().x;
                positions.extend(
                    run.glyphs()
                        .map(|glyph| (glyph.id(), origin + glyph.pen() + f64::from(glyph.x()))),
                );
            }
            // Independent original Ahem cmap: A35, period16, both one em.
            assert_eq!(positions, [(35, 0.0), (16, 10.0)]);
        }
    }
}

#[test]
fn end_hanging_changes_issued_measures_and_keeps_original_glyph_placement() {
    end_geometry::<f32>();
    end_geometry::<f64>();
}

fn receiving_partition<S: LayoutScalar>() {
    for placement in [
        text::hyphenation::HyphenationPlacement::LogicalAfter,
        text::hyphenation::HyphenationPlacement::BothSides,
    ] {
        for first in [false, true] {
            let style = text::style::Style {
                hanging_punctuation: text::style::HangingPunctuation::new(first, None, false),
                wrap: Wrap::Word,
                hyphens: text::hyphenation::Hyphens::Auto,
                locale: Some("en-US".into()),
                hyphenate_character: text::hyphenation::HyphenateCharacter::Auto,
                native_hyphenation_convention: Some(
                    text::hyphenation::NativeHyphenationConvention::language_script(
                        "en-US",
                        "Latn",
                        "(".into(),
                        placement,
                    )
                    .unwrap(),
                ),
                ..ahem_style(10.0, 10.0)
            };
            let p = paragraph("as\u{ad}sociate", style);
            for (width, cut, automatic) in [(75.0, 5, true), (85.0, 3, false)] {
                let batch = compute::<S>(&p, width, 0.0);
                let rows = lines(&batch, &p);
                assert_eq!(rows.len(), 2);
                assert_eq!(
                    batch.placed_text()[0].layout().lines()[0].range(),
                    text::source::Range::new(0, cut)
                );
                let selected = rows[0].selected_line();
                let alternative = selected.request().alternative.as_ref().unwrap();
                assert_eq!(
                    matches!(
                        alternative.view().definition(),
                        text::prepared::PreparedAlternativeDefinition::Automatic { .. }
                    ),
                    automatic
                );
                assert_eq!(alternative.view().text(), &text::source::Text::from("("));
                let incoming = rows[0]
                    .hyphenation_continuation()
                    .expect("the real After/Both ending issues its receiver");
                assert_eq!(rows[1].selected_line().request().incoming, Some(incoming));
                assert_eq!(
                    rows[1]
                        .selected_line()
                        .request()
                        .context
                        .formatted_line
                        .first(),
                    first.then_some(false)
                );
                // The SHY residual is seven10px letters plus the received10px
                // opening mark. It is a receiving hypothetical portion, so
                // First cannot omit that mark to turn80 into70 at width75.
                // At85 it really fits and manual priority wins. None and First
                // keep the same complete target/body/indication and decisions.
                if !automatic {
                    assert_eq!(rows[1].feedback().advance, 80.0);
                }
            }
        }
    }
}

#[test]
fn receiving_manual_partition_does_not_inherit_current_first_or_change_neutral_target() {
    receiving_partition::<f32>();
    receiving_partition::<f64>();
}

fn zone_natural<S: LayoutScalar>() {
    for last in [false, true] {
        let style = text::style::Style {
            hanging_punctuation: text::style::HangingPunctuation::new(false, None, last),
            wrap: Wrap::Word,
            hyphens: text::hyphenation::Hyphens::Auto,
            locale: Some("en-US".into()),
            hyphenate_character: text::hyphenation::HyphenateCharacter::String(
                text::source::Text::from("A"),
            ),
            ..ahem_style(10.0, 10.0)
        };
        let p = paragraph("A) associate", style);
        let batch = compute::<S>(&p, 80.0, 10.0);
        let rows = lines(&batch, &p);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            batch.placed_text()[0].layout().lines()[0].range(),
            text::source::Range::new(0, 7)
        );
        assert_eq!(rows[0].feedback().advance, 80.0);
        assert_eq!(rows[1].feedback().advance, 50.0);
        assert!(matches!(
            rows[0]
                .selected_line()
                .request()
                .alternative
                .as_ref()
                .unwrap()
                .view()
                .definition(),
            text::prepared::PreparedAlternativeDefinition::Automatic { .. }
        ));
        // The natural-only zone reference ends after A) SPACE, whose closing
        // mark consumes false Last. Actual selected hyphen/terminal lines end
        // in letters; only the zone path reaches this punctuation role.
    }
}

#[test]
fn natural_only_zone_reference_resolves_its_own_formatted_role() {
    zone_natural::<f32>();
    zone_natural::<f64>();
}
