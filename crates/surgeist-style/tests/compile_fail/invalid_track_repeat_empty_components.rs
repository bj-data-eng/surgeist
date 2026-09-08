use surgeist_style::{GridTrackComponent, TrackRepeat, TrackRepeatCount};

fn main() {
    let _repeat = TrackRepeat {
        count: TrackRepeatCount::AutoFit,
        components: Vec::<GridTrackComponent>::new(),
    };
}
