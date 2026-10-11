#![forbid(unsafe_code)]

fn main() {
    if let Err(error) = surgeist_cssom_reflections::run(std::env::args().skip(1)) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
