use std::collections::BTreeMap;

use surgeist_style::LayerRegistry;

fn main() {
    let _registry = LayerRegistry {
        named: BTreeMap::new(),
        next_order: 0,
    };
}
