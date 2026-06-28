use surgeist::app::{ResourceStatus, TaskStatus, testing::ThumbnailImportExample};

fn main() {
    let mut example = ThumbnailImportExample::new();

    example.choose_folder("/tmp/photos");
    example.drain_once();
    assert_eq!(example.thumbnail_status(0), ResourceStatus::Starting);

    example.finish_thumbnail(0);
    example.drain_all();
    assert_eq!(example.thumbnail_status(0), ResourceStatus::Ready);

    example.refresh_thumbnail(0);
    example.drain_once();
    assert_eq!(example.thumbnail_status(0), ResourceStatus::Refreshing);

    example.finish_thumbnail(0);
    example.navigate_away();
    example.drain_all();
    assert_eq!(example.thumbnail_status(0), ResourceStatus::Ready);
    assert_eq!(example.import_task_status(), TaskStatus::Running);
    assert!(
        example
            .redrawn_surfaces()
            .contains(&example.gallery_surface())
    );

    println!("initial_tiles={}", example.initial_tile_count());
    println!("thumbnail_0=ready");
    println!("import=running");
}
