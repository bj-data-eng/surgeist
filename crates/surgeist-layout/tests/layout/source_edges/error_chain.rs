//! A refused source prints each level of its error chain.
use super::*;

/// The `Display` of an error and of every level of its `source()` chain.
fn chain(error: &(dyn std::error::Error + 'static)) -> Vec<String> {
    let mut out = vec![error.to_string()];
    let mut next = error.source();
    while let Some(level) = next {
        out.push(level.to_string());
        next = level.source();
    }
    out
}

#[test]
fn a_contrary_generated_order_prints_the_wrapping_scope_chain() {
    // The source canonically contains A in S, while the borrowed producer
    // claims A followed by S containing B, so the source association is
    // refused after Text has processed the independently valid source.
    for direction in [
        layout::geometry::Direction::Ltr,
        layout::geometry::Direction::Rtl,
    ] {
        let host = Host::<f32>::new(true, layout::inline_wrapping::WrapEdge::Line, direction);
        let error = match layout::inline_wrapping::process_wrapping_source(
            &Early(&host),
            0,
            source(false, direction),
            host.occurrences(),
        ) {
            Ok(_) => panic!("a contrary generated order cannot issue a resolved receipt"),
            Err(error) => error,
        };
        assert_eq!(
            chain(&error),
            [
                "inline wrapping failed at node 0: layout input is invalid",
                "the wrapping scopes are invalid",
                "a source association is inconsistent with the processed source",
            ]
        );
    }
}
