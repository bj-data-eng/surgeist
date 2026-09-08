use surgeist_style::{SubgridLineNameComponent, SubgridLineNameRepeatCount};

fn main() {
    let _subgrid = SubgridLineNameComponent::Repeat {
        count: SubgridLineNameRepeatCount::AutoFill,
        line_name_sets: vec![],
    };
}
