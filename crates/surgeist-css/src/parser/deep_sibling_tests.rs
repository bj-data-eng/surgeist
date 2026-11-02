//! Recovering many independently deep rules does work in proportion to the
//! sheet and runs on an ordinary thread (#1295).
//!
//! Each fixture is valid CSS: `count` sibling style rules, each nested `depth`
//! levels deep. The work measures are counters, not time: the nodes of the token
//! trees built for the parse, and the nodes the structural queries read of them.
//! A sheet of `count` such rules has `2 * depth * count` nodes (an identifier
//! and a block per level), and each deep rule reached through a copy of the
//! source or a walk from its start would add the whole sheet to the count.

use super::bounded_execution::work;
use crate::rule::CssRule;

fn sheet(depth: usize, count: usize) -> String {
    let rule = format!("{}{}", "a{".repeat(depth), "}".repeat(depth));
    rule.repeat(count)
}

struct Work {
    built: usize,
    scanned: usize,
    nodes: usize,
}

impl Work {
    fn per_node(&self) -> f64 {
        (self.built + self.scanned) as f64 / self.nodes as f64
    }
}

fn measure(depth: usize, count: usize) -> Work {
    let source = sheet(depth, count);
    let (report, built, scanned) =
        work::measure(|| crate::CssParser::default().parse_sheet(&source));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().rules().len(), count);
    let mut rule = &report.syntax().rules()[count - 1];
    for _ in 1..depth {
        let CssRule::Style(style) = rule else {
            panic!("nested style chain")
        };
        let [child] = style.rules() else {
            panic!("one nested child")
        };
        rule = child;
    }
    Work {
        built,
        scanned,
        nodes: 2 * depth * count,
    }
}

#[test]
fn sibling_deep_rules_are_recovered_by_reading_each_node_a_bounded_number_of_times() {
    for depth in [32, 64] {
        let few = measure(depth, 25);
        assert!(
            few.built >= few.nodes,
            "depth {depth}: the sheet's tree is built"
        );
        for count in [50, 100] {
            let many = measure(depth, count);
            println!(
                "depth {depth}: 25 rules {:.2} reads per node; {count} rules {:.2} ({} built, {} scanned of {} nodes)",
                few.per_node(),
                many.per_node(),
                many.built,
                many.scanned,
                many.nodes
            );
            // A copy of the source or a walk from its start per deep rule
            // would make the reads per node grow with the rule count.
            assert!(
                many.per_node() <= few.per_node() * 1.1 + 0.05,
                "depth {depth}: {count} rules read {:.2} per node, 25 read {:.2}",
                many.per_node(),
                few.per_node()
            );
            // One tree is built for the sheet, not one per deep rule.
            assert!(
                many.built <= 2 * many.nodes,
                "depth {depth}: {count} rules build {} nodes for a sheet of {}",
                many.built,
                many.nodes
            );
        }
    }
}

/// The depth that sheets and rule fragments trust the caller's stack with parses
/// on an ordinary 2 MiB thread for every kind of nesting rule.
#[test]
fn every_kind_of_rule_parses_at_the_trusted_depth_on_a_two_mib_thread() {
    let depth = super::bounded_execution::STRUCTURAL_DIRECT_DEPTH as usize;
    for (open, leaf, close, top) in [
        ("a{", "b:c", "}", 1),
        ("@media x{", "a{b:c}", "}", 1),
        ("@layer l{", "a{b:c}", "}", 1),
        ("@scope(a){", "a{b:c}", "}", 1),
        ("@supports(a:b){", "a{b:c}", "}", 1),
        ("@container n (a:b){", "a{b:c}", "}", 1),
        ("@when media(all){", "a{b:c}", "}", 1),
        ("@when media(not all){}@else{", "a{b:c}", "}", 2),
        ("@media x{a{", "b:c", "}}", 1),
    ] {
        // Each unit opens one or two blocks.
        let levels = depth / close.len();
        let source = format!("{}{leaf}{}", open.repeat(levels), close.repeat(levels));
        std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(move || {
                let report = crate::CssParser::default().parse_sheet(&source);
                assert_eq!(report.syntax().rules().len(), top);
            })
            .unwrap()
            .join()
            .unwrap();
    }
}

#[test]
fn many_deep_siblings_parse_on_an_ordinary_test_thread() {
    for (depth, count) in [(32, 400), (64, 200)] {
        let source = sheet(depth, count);
        let report = crate::CssParser::default().parse_sheet(&source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        assert_eq!(report.syntax().rules().len(), count);
    }
}
