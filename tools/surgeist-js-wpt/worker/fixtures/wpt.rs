//! Source identity for the first upstream Window variants.

use std::io;

use surgeist_js_wpt::FixtureId;

use super::matches_git_blob;

const REVISION: &str = "69f2bba61ce7e9469ccbce53de4644a89f6b7dff";

struct PinnedSource {
    path: &'static str,
    blob: &'static str,
    bytes: &'static [u8],
}

impl PinnedSource {
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

    fn utf8(&self) -> io::Result<&'static str> {
        std::str::from_utf8(self.bytes).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("retained WPT source is not UTF-8 ({}): {error}", self.path),
            )
        })
    }
}

struct WindowCase {
    url: &'static str,
    script: PinnedSource,
}

const HARNESS: PinnedSource = PinnedSource {
    path: "resources/testharness.js",
    blob: "dd841a084662637b32b7f064b1816ab0b35f89d1",
    bytes: include_bytes!("../../fixtures/wpt/resources/testharness.js"),
};

const REPORT: PinnedSource = PinnedSource {
    path: "resources/testharnessreport.js",
    blob: "405a2d8b06f00fe8292e0e9d5b917e193cfd416c",
    bytes: include_bytes!("../../fixtures/wpt/resources/testharnessreport.js"),
};

const HTML_COLLECTION: PinnedSource = PinnedSource {
    path: "dom/collections/HTMLCollection-as-prototype.html",
    blob: "d572d35c041496904901a9eb900e84f2f5b63704",
    bytes: include_bytes!("../../fixtures/wpt/dom/collections/HTMLCollection-as-prototype.html"),
};

pub(crate) struct HtmlFixture {
    pub(crate) document: &'static str,
    harness: &'static str,
    report: &'static str,
}

pub(crate) enum LocalScript {
    Harness(&'static str),
    HarnessReport(&'static str),
}

impl HtmlFixture {
    /// Only the embedded, verified local support URLs may be resolved.
    pub(crate) fn external_script(&self, url: &[u16]) -> Option<LocalScript> {
        if url
            .iter()
            .copied()
            .eq("/resources/testharness.js".encode_utf16())
        {
            Some(LocalScript::Harness(self.harness))
        } else if url
            .iter()
            .copied()
            .eq("/resources/testharnessreport.js".encode_utf16())
        {
            Some(LocalScript::HarnessReport(self.report))
        } else {
            None
        }
    }
}

pub(crate) fn resolve_html(identity: &FixtureId) -> io::Result<Option<HtmlFixture>> {
    if identity.revision().as_str() != REVISION
        || identity.path().as_str() != HTML_COLLECTION.path
        || identity.url() != "/dom/collections/HTMLCollection-as-prototype.html"
    {
        return Ok(None);
    }
    HTML_COLLECTION.verify()?;
    HARNESS.verify()?;
    REPORT.verify()?;
    Ok(Some(HtmlFixture {
        document: HTML_COLLECTION.utf8()?,
        harness: HARNESS.utf8()?,
        report: REPORT.utf8()?,
    }))
}

const CASES: &[WindowCase] = &[
    WindowCase {
        url: "/dom/events/EventTarget-addEventListener.any.html",
        script: PinnedSource {
            path: "dom/events/EventTarget-addEventListener.any.js",
            blob: "e22da4aff850a2f0042c5ecda4c844cc1852f2ec",
            bytes: include_bytes!(
                "../../fixtures/wpt/dom/events/EventTarget-addEventListener.any.js"
            ),
        },
    },
    WindowCase {
        url: "/dom/events/EventTarget-add-remove-listener.any.html",
        script: PinnedSource {
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
        let changed = PinnedSource {
            bytes: &HARNESS.bytes[..HARNESS.bytes.len() - 1],
            ..HARNESS
        };
        let error = changed.verify().unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("resources/testharness.js"));
    }
}
