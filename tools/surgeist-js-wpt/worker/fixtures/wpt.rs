//! Source identity for the first upstream Window variants.

use std::io;

use surgeist_js_wpt::FixtureId;

use super::matches_git_blob;

const REVISION: &str = "69f2bba61ce7e9469ccbce53de4644a89f6b7dff";

struct PinnedScript {
    path: &'static str,
    blob: &'static str,
    bytes: &'static [u8],
}

impl PinnedScript {
    fn verify(&self) -> io::Result<()> {
        if matches_git_blob(self.bytes, self.blob) {
            return Ok(());
        }
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "retained WPT source failed its Git blob check ({})",
                self.path
            ),
        ))
    }
}

struct WindowCase {
    url: &'static str,
    script: PinnedScript,
}

const HARNESS: PinnedScript = PinnedScript {
    path: "resources/testharness.js",
    blob: "dd841a084662637b32b7f064b1816ab0b35f89d1",
    bytes: include_bytes!("../../fixtures/wpt/resources/testharness.js"),
};

const CASES: &[WindowCase] = &[
    WindowCase {
        url: "/dom/events/EventTarget-addEventListener.any.html",
        script: PinnedScript {
            path: "dom/events/EventTarget-addEventListener.any.js",
            blob: "e22da4aff850a2f0042c5ecda4c844cc1852f2ec",
            bytes: include_bytes!(
                "../../fixtures/wpt/dom/events/EventTarget-addEventListener.any.js"
            ),
        },
    },
    WindowCase {
        url: "/dom/events/EventTarget-add-remove-listener.any.html",
        script: PinnedScript {
            path: "dom/events/EventTarget-add-remove-listener.any.js",
            blob: "b1d7ffb3e07fbe2038f0e33a8e63173edf12234c",
            bytes: include_bytes!(
                "../../fixtures/wpt/dom/events/EventTarget-add-remove-listener.any.js"
            ),
        },
    },
];

pub(crate) fn is_pinned_window(identity: &FixtureId) -> io::Result<bool> {
    if identity.revision().as_str() != REVISION {
        return Ok(false);
    }
    let Some(case) = CASES
        .iter()
        .find(|case| case.url == identity.url() && case.script.path == identity.path().as_str())
    else {
        return Ok(false);
    };
    HARNESS.verify()?;
    case.script.verify()?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_source_verification_rejects_changed_bytes() {
        HARNESS.verify().unwrap();
        let changed = PinnedScript {
            bytes: &HARNESS.bytes[..HARNESS.bytes.len() - 1],
            ..HARNESS
        };
        let error = changed.verify().unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("resources/testharness.js"));
    }
}
