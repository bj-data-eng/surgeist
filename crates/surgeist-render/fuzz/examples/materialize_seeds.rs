#![forbid(unsafe_code)]
//! Materialize the fixed source-qualified cases through their canonical codec.
use std::{error::Error, fs, path::Path};
use surgeist_fuzz::CaseTarget;
use surgeist_render_fuzz::{ResourceTarget, SceneTarget};

fn write<T: CaseTarget>(
    root: &Path,
    target: &T,
    count: u8,
    literal: impl Fn(u8) -> surgeist_fuzz::Result<T::Case>,
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(root)?;
    for selector in 0..count {
        let case = literal(selector)?;
        let encoded = target.encode(&case)?;
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
    let resources = ResourceTarget::new();
    write(&destination.join("resources"), &resources, 24, |selector| {
        resources.literal(selector)
    })?;
    let scenes = SceneTarget::new();
    write(&destination.join("scenes"), &scenes, 12, |selector| {
        scenes.literal(selector)
    })?;
    Ok(())
}
