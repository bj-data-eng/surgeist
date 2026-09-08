use surgeist_style::{SubgridLineNameComponent, SubgridLineNameRepeatCount, SubgridTrack};

fn main() {
    let auto_fill = SubgridLineNameComponent::repeat(
        SubgridLineNameRepeatCount::AutoFill,
        [["line-name"]],
    )
    .unwrap();

    let _subgrid = SubgridTrack {
        name_components: vec![auto_fill.clone(), auto_fill],
    };
}
