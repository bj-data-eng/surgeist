//! Stable's local quality choice, real source revisions and eligible leaf reuse.
use super::*;

fn assert_rows<S: l::LayoutScalar>(
    fixture: &Advanced<S>,
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    expected: &[(usize, usize, f64)],
) {
    let layout = batch.placed_text()[0].layout();
    assert_eq!(layout.lines().len(), expected.len());
    let receipts = fixture.receipts(batch);
    for (i, ((line, receipt), &(start, end, advance))) in layout
        .lines()
        .iter()
        .zip(receipts)
        .zip(expected)
        .enumerate()
    {
        assert_eq!(line.range(), t::Range::new(start, end));
        assert_eq!(
            line.bounds(),
            t::Rect::new(0.0, i as f64 * 10.0, 60.0, 10.0)
        );
        assert_eq!(line.baseline(), i as f64 * 10.0 + 8.0);
        let selected = receipt.selected_line();
        assert_eq!(selected.request().context.available_inline, Some(60.0));
        assert_eq!(selected.request().context.indent, 0.0);
        assert_eq!(
            selected.request().context.end,
            if i + 1 == expected.len() {
                t::LineEnd::Paragraph
            } else {
                t::LineEnd::Soft
            }
        );
        assert!(selected.request().alternative.is_none());
        assert_eq!(selected.request().end.view().source_after().get(), end);
        assert_eq!(
            selected
                .finalize(surgeist_text::LineAllocations {
                    spacing: t::SpacingAllocations::None,
                    scopes: surgeist_text::ScopeAllocations::None
                })
                .unwrap()
                .measurements()
                .alignment()
                .advance(),
            advance
        );
        assert!(receipt.run_views().all(|run| run.font_size() == 10.0));
    }
    let root = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    assert_eq!(
        root.size,
        l::Size::new(S::from_f64(60.0), S::from_f64(20.0))
    );
}

fn suffix<S: l::LayoutScalar>() {
    let mut base = Advanced::<S>::with_revision(&["AAA BB C"], 60.0, None, false, 3);
    base.root(|root| root.text_wrap_style = l::TextWrapStyle::Stable);
    let old = base.compute(&[]);
    // Collapsed trailing SPACE belongs to the first issued source range, while
    // its visible natural advance is AAA(30)+SPACE(10)+BB(20)=60.
    assert_rows(&base, &old, &[(0, 7, 60.0), (7, 8, 10.0)]);
    let old_source = old.placed_text()[0].layout().source().clone();
    let old_prepared = base.prepared.clone();
    let old_receipts = base.receipts(&old);
    old.apply_to(&mut base.host).unwrap();

    base.root(|root| root.text_wrap_style = l::TextWrapStyle::Balance);
    let balanced = base.compute(&[0]);
    assert_rows(&base, &balanced, &[(0, 4, 30.0), (4, 8, 40.0)]);
    // The frame is unchanged; only the genuine selected legal cuts differ.
    for (ordinary, balance) in old_receipts.iter().zip(base.receipts(&balanced)) {
        assert_eq!(
            ordinary.selected_line().request().context.tabs,
            balance.selected_line().request().context.tabs
        );
    }
    balanced.apply_to(&mut base.host).unwrap();
    base.root(|root| root.text_wrap_style = l::TextWrapStyle::Stable);
    let restored = base.compute(&[0]);
    assert_rows(&base, &restored, &[(0, 7, 60.0), (7, 8, 10.0)]);
    restored.apply_to(&mut base.host).unwrap();

    // Same document identity, coherent next revision; offers/ends are issued by
    // the new preparation rather than borrowed from the old paragraph.
    let mut revised = Advanced::<S>::with_revision(&["AAA BB CCCC"], 60.0, None, false, 4);
    revised.root(|root| root.text_wrap_style = l::TextWrapStyle::Stable);
    let changed = revised.compute(&[]);
    assert_rows(&revised, &changed, &[(0, 7, 60.0), (7, 11, 40.0)]);

    // Keep the genuinely revised tree/capture/preparation, but carry the old
    // host's actual published state into this same node graph. Root invalidation
    // must not replay revision3's paragraph or cuts into revision4.
    revised.host.published = base.host.published.clone();
    let current = revised.compute(&[0]);
    assert!(current.invalidated_nodes().contains(&0));
    assert_rows(&revised, &current, &[(0, 7, 60.0), (7, 11, 40.0)]);
    assert_eq!(
        revised.cuts_and_natural(&current),
        revised.cuts_and_natural(&changed)
    );
    current.apply_to(&mut revised.host).unwrap();
    let repeated = revised.compute(&[]);
    assert_rows(&revised, &repeated, &[(0, 7, 60.0), (7, 11, 40.0)]);
    assert_eq!(
        revised.cuts_and_natural(&repeated),
        revised.cuts_and_natural(&changed)
    );
    // This unchanged Text computation is reexecution equivalence; positive
    // eligible cache reuse is established separately by the real float leaf.
    assert_rows(&base, &restored, &[(0, 7, 60.0), (7, 8, 10.0)]);
    assert_rows(&base, &balanced, &[(0, 4, 30.0), (4, 8, 40.0)]);
    let old_units = old_source.text().code_units();
    assert_eq!(
        old_units,
        &revised.capture.source().text().code_units()[..8]
    );
    assert_ne!(old_prepared, revised.prepared);
    assert_eq!(old.placed_text()[0].layout().source(), &old_source);
    assert_eq!(base.receipts(&old), old_receipts);
    assert_rows(&base, &old, &[(0, 7, 60.0), (7, 8, 10.0)]);
    revised.root(|root| root.text_wrap_style = l::TextWrapStyle::Balance);
    let changed_balance = revised.compute(&[0]);
    // The alternative 30/70 no longer fits60, so both modes retain60/40.
    assert_rows(&revised, &changed_balance, &[(0, 7, 60.0), (7, 11, 40.0)]);
}
#[test]
fn stable_text_uses_local_cuts_across_a_coherent_suffix_revision_f32() {
    suffix::<f32>();
}
#[test]
fn stable_text_uses_local_cuts_across_a_coherent_suffix_revision_f64() {
    suffix::<f64>();
}

fn warm_float<S: l::LayoutScalar>() {
    let mut fixture = Advanced::<S>::new(&["AAA BB C"], 80.0, Some(t::Range::new(4, 6)), true);
    let first = fixture.compute(&[]);
    assert_float(&fixture, &first, false);
    first.apply_to(&mut fixture.host).unwrap();
    let mut retained = vec![(first, false)];
    for style in [
        l::TextWrapStyle::Balance,
        l::TextWrapStyle::Stable,
        l::TextWrapStyle::Auto,
    ] {
        let balance = style == l::TextWrapStyle::Balance;
        fixture.root(|root| root.text_wrap_style = style);
        let switched = fixture.compute(&[0]);
        assert!(switched.invalidated_nodes().contains(&0));
        assert_float(&fixture, &switched, balance);
        switched.apply_to(&mut fixture.host).unwrap();
        let float = fixture.float.unwrap() as usize;
        let before = fixture.host.hits[float].get();
        let warm = fixture.compute(&[]);
        assert_float(&fixture, &warm, balance);
        assert!(
            fixture.host.hits[float].get() > before,
            "warm evidence requires a genuine eligible float leaf CacheOf hit"
        );
        warm.apply_to(&mut fixture.host).unwrap();
        let mut fresh = Advanced::<S>::new(&["AAA BB C"], 80.0, Some(t::Range::new(4, 6)), true);
        fresh.root(|root| root.text_wrap_style = style);
        let cold = fresh.compute(&[]);
        assert_float(&fresh, &cold, balance);
        assert_eq!(
            fixture.cuts_and_natural(&warm),
            fresh.cuts_and_natural(&cold)
        );
        retained.push((switched, balance));
        for (old, old_balance) in &retained {
            assert_float(&fixture, old, *old_balance);
        }
    }
    // This proves the real float leaf is reusable; Text paragraph finalization
    // is intentionally reexecuted, not presented as a root cache hit.
}
#[test]
fn stable_text_style_switches_reuse_real_float_leaf_without_stale_rows_f32() {
    warm_float::<f32>();
}
#[test]
fn stable_text_style_switches_reuse_real_float_leaf_without_stale_rows_f64() {
    warm_float::<f64>();
}
