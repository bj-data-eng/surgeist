use std::path::Path;

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("generator lives under surgeist/api/generator");

    if let Err(error) = surgeist_api_generator::run(root, std::env::args()) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
