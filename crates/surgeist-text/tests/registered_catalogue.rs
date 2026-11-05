use surgeist_text::{ErrorCode, Options, Source, Style, System};

#[test]
fn default_text_system_requires_an_application_supplied_font() {
    let mut system = System::default();
    let result = system.layout(
        Source::new("A font must be supplied by the application"),
        Style::default(),
        Options::default(),
    );
    match result {
        Err(error) => assert_eq!(error.code, ErrorCode::NoUsableFontFace),
        Ok(_) => panic!("an empty application catalogue cannot shape ordinary text"),
    }
}
