//! Document selection, timing rounds and the shared report table.

use std::{hint::black_box, time::Instant};

/// Measured rounds per document, after one unmeasured warm-up computation.
pub const ROUNDS: usize = 11;

/// A document family the bench can build on demand.
pub struct Family {
    pub name: &'static str,
    pub build: fn() -> Document,
}

/// A built document: its size, and one full layout computation.
pub struct Document {
    pub nodes: usize,
    pub depth: usize,
    pub round: Box<dyn Fn()>,
}

/// Node count and maximum depth, with the root at depth one, of a tree given
/// as per-node child lists whose first node is the root.
pub fn shape(children: &[Vec<u32>]) -> (usize, usize) {
    let mut deepest = 0;
    let mut stack = vec![(0_u32, 1_usize)];
    while let Some((node, depth)) = stack.pop() {
        deepest = deepest.max(depth);
        stack.extend(
            children[node as usize]
                .iter()
                .map(|&child| (child, depth + 1)),
        );
    }
    (children.len(), deepest)
}

/// Prints the report for the selected families.
pub fn run(rounds_header: &'static str, families: &'static [Family]) {
    let filter = std::env::args()
        .skip(1)
        .find(|argument| !argument.starts_with('-'));
    if let Some(wanted) = &filter
        && !families
            .iter()
            .any(|family| family.name.contains(wanted.as_str()))
    {
        let names: Vec<&str> = families.iter().map(|family| family.name).collect();
        eprintln!(
            "no benchmark document name contains {wanted:?}; known documents: {}",
            names.join(", ")
        );
        std::process::exit(2);
    }
    report(rounds_header, families, filter.as_deref());
}

fn report(rounds_header: &str, families: &[Family], filter: Option<&str>) {
    println!("{rounds_header}: {ROUNDS} rounds per document, after one warm-up layout");
    println!();
    println!(
        "{:<42} {:>8} {:>6} {:>10} {:>10} {:>10}",
        "document", "nodes", "depth", "median ms", "min ms", "max ms"
    );
    for family in families {
        if filter.is_some_and(|wanted| !family.name.contains(wanted)) {
            continue;
        }
        let document = (family.build)();
        (document.round)();
        let mut rounds: Vec<f64> = (0..ROUNDS)
            .map(|_| {
                let start = Instant::now();
                black_box(&document.round)();
                start.elapsed().as_secs_f64() * 1_000.0
            })
            .collect();
        rounds.sort_by(f64::total_cmp);
        println!(
            "{:<42} {:>8} {:>6} {:>10.2} {:>10.2} {:>10.2}",
            family.name,
            document.nodes,
            document.depth,
            rounds[ROUNDS / 2],
            rounds[0],
            rounds[ROUNDS - 1]
        );
    }
}
