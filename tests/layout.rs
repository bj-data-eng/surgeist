use surgeist::layout::{
    Available, Dimension, Display, Edges, GridAutoFlow, GridPlacement, Length, NodeInput, Position,
    Size, TrackComponent,
};

#[test]
fn root_facade_reexports_layout_front_door_types() {
    let input = NodeInput {
        display: Display::Block,
        size: Size::new(Dimension::px(10.0), Dimension::AUTO),
        margin: Edges::all(Length::px(2.0).into()),
        position: Position::Relative,
        grid_auto_flow: GridAutoFlow::Row,
        grid_column: GridPlacement::AUTO,
        grid_row: GridPlacement::AUTO,
        ..NodeInput::DEFAULT
    };

    let available = Size::splat(Available::definite(100.0));
    let track = TrackComponent::px(10.0);

    assert_eq!(input.display, Display::Block);
    assert_eq!(available.width.into_option(), Some(100.0));
    assert_eq!(track, TrackComponent::px(10.0));
}
