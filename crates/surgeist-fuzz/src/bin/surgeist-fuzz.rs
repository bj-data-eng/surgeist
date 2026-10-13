#![forbid(unsafe_code)]

fn main() {
    std::process::exit(surgeist_fuzz::campaign::main_entry(
        std::env::args_os().skip(1),
    ));
}
