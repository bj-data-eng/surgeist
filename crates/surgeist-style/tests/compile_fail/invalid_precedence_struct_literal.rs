use surgeist_style::{LayerOrder, RulePrecedence, SourceOrder};

fn main() {
    let _precedence = RulePrecedence {
        layer_order: LayerOrder::new(1),
        source_order: SourceOrder::new(2),
    };
}
