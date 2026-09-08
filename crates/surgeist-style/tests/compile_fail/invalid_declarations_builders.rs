use surgeist_style::{Declarations, GridTrackList, Length};

fn main() {
    let _ = Declarations::new().width(Length::px(-1.0));
    let _ = Declarations::new().opacity(2.0);
    let _ = Declarations::new().transition_duration(-0.25);
    let _ = Declarations::new().grid_template_rows(GridTrackList::new(Vec::new()));
}
