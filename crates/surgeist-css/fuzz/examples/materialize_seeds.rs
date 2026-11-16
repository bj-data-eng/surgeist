#![forbid(unsafe_code)]
//! Materialize the fixed source-qualified cases through their canonical codecs.
use std::{error::Error, fs, path::Path};
use surgeist_css_fuzz::{ByteTarget, SourceTarget};
use surgeist_fuzz::CaseTarget;

fn write<T: CaseTarget>(
    root: &Path,
    target: &T,
    count: u8,
    literal: impl Fn(u8) -> surgeist_fuzz::Result<T::Case>,
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(root)?;
    for selector in 0..count {
        let encoded = target.encode(&literal(selector)?)?;
        // The same owner decoder must accept and canonically preserve every seed.
        let decoded = target.decode(&encoded)?;
        if target.encode(&decoded)? != encoded {
            return Err("noncanonical literal seed".into());
        }
        fs::write(root.join(format!("{selector:02}.case")), encoded)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);
    let destination = args.next().ok_or("usage: materialize_seeds DESTINATION")?;
    if args.next().is_some() {
        return Err("usage: materialize_seeds DESTINATION".into());
    }
    let destination = Path::new(&destination);
    let bytes = ByteTarget::new();
    write(
        &destination.join("bytes"),
        &bytes,
        ByteTarget::LITERALS,
        |id| bytes.literal(id),
    )?;
    let source = SourceTarget::new();
    write(
        &destination.join("source"),
        &source,
        SourceTarget::LITERALS,
        |id| source.literal(id),
    )?;
    Ok(())
}
