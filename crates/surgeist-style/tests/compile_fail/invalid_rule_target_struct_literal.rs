use surgeist_style::{RuleTarget, Selector, StyleBucket};

fn main() -> surgeist_style::Result<()> {
    let _target = RuleTarget {
        selector: Selector::tag("button")?,
        bucket: StyleBucket::Before,
    };
    Ok(())
}
