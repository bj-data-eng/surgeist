//! Real native endpoint controls retain their independent paired-box authority.
use super::*;

fn unscoped_native_pair_input(
    direction: layout::geometry::Direction,
) -> text::capture::ContextualSourceInput {
    use text::capture::SourceProcessingEvent as E;
    let mut raw = paired_source(direction);
    raw.scopes.truncate(1); // The Block domain remains; no InlineScope.
    raw.events
        .retain(|event| !matches!(event, E::EnterScope(1) | E::ExitScope(1)));
    raw.computed_line_boundaries.clear();
    raw
}

fn actual_early_native_pair_without_inline_scope<S: layout::scalar::LayoutScalar>() {
    for direction in [
        layout::geometry::Direction::Ltr,
        layout::geometry::Direction::Rtl,
    ] {
        let mut host = PairedEarly::<S>::new(direction, false);
        host.declarations.retain(|row| {
            matches!(
                &row.box_,
                layout::inline_wrapping::InlineBoxRef::NativePair { .. }
            )
        });
        host.scopes.clear();
        host.base.orders.clear(); // Complete actual native inventory proves it.
        let raw = unscoped_native_pair_input(direction);
        assert_eq!(raw.scopes.len(), 1);
        assert!(host.scopes.is_empty());
        assert_eq!(host.declarations.len(), 1);

        // Independently exercise the current Text front door with the exact
        // semantic Endpoint request the adapter is required to supply.
        let mut direct = raw.clone();
        direct.computed_line_boundaries =
            vec![text::capture::ComputedLineBoundaryInput::Endpoint { control: 0 }];
        let direct = text::capture::process_contextual_source(direct)
            .expect("real endpoint input does not require a canonical InlineScope");
        assert!(direct.captured_scope(1).is_none());
        assert_eq!(
            direct.captured_computed_line_boundary(0).unwrap().target(),
            text::capture::CapturedComputedLineBoundaryTarget::Endpoint(
                direct.captured_control(0).unwrap()
            )
        );

        // The host implements only Traverse + WrapEdgeSource. It has real
        // boundary controls and source origins, and no prepared handles.
        let (processed, _receipt) =
            layout::inline_wrapping::process_wrapping_source(&host, 0, raw, host.occurrences())
                .expect(
                    "an explicit authentic native pair must not require a fabricated InlineScope",
                )
                .into_parts();
        assert!(processed.captured_scope(1).is_none());
        assert_eq!(processed.source().text(), &text::source::Text::from("AB"));
        assert_eq!(processed.processed_text().code_units(), [0x41, 0x42]);
        assert_eq!(
            processed
                .captured_computed_line_boundary(0)
                .unwrap()
                .target(),
            text::capture::CapturedComputedLineBoundaryTarget::Endpoint(
                processed.captured_control(0).unwrap()
            )
        );
        assert!(processed.captured_computed_line_boundary(1).is_none());
        let processed = Arc::new(processed);
        let prepared = normal_prepare(&processed);
        assert_eq!(prepared.control_views().count(), 2);
        assert_eq!(
            prepared
                .opportunity_views()
                .map(|offer| (offer.kind(), offer.boundary().view().source_before().get()))
                .collect::<Vec<_>>(),
            [
                (text::prepared::PreparedBreakKind::Forced, 1),
                (text::prepared::PreparedBreakKind::ParagraphEnd, 2),
            ]
        );
    }
}

#[test]
fn real_early_native_pair_with_unassociated_endpoint_controls_keeps_endpoint_force() {
    actual_early_native_pair_without_inline_scope::<f32>();
    actual_early_native_pair_without_inline_scope::<f64>();
}

mod final_layout;
