use sha1::{Digest, Sha1};
use surgeist_js_wpt::FixtureId;

pub(super) struct Fixture {
    pub(super) source: &'static str,
    pub(super) admission: Admission,
}

#[derive(Clone, Copy)]
pub(super) enum Admission {
    Standard,
    ScriptingDisabled,
    NoSourceWork,
}

struct Asset {
    path: &'static str,
    url: &'static str,
    source: &'static str,
    admission: Admission,
}

impl Asset {
    fn resolve(&self, identity: &FixtureId) -> Option<Fixture> {
        if identity.path().as_str() != self.path || identity.url() != self.url {
            return None;
        }

        // The authored source revision is the Git blob ID of the embedded
        // file. The request carries no file path for the worker to open.
        let mut hash = Sha1::new();
        hash.update(format!("blob {}\0", self.source.len()));
        hash.update(self.source.as_bytes());
        if identity.revision().as_str() != format!("{:x}", hash.finalize()) {
            return None;
        }

        Some(Fixture {
            source: self.source,
            admission: self.admission,
        })
    }
}

const NATIVE: Asset = Asset {
    path: "qualification/native-window.js",
    url: "/qualification/native-window.html",
    source: include_str!("../fixtures/qualification/native-window.js"),
    admission: Admission::Standard,
};

const ASSETS: &[Asset] = &[
    NATIVE,
    Asset {
        url: "/qualification/native-window.html?scripting=disabled",
        admission: Admission::ScriptingDisabled,
        ..NATIVE
    },
    Asset {
        path: "qualification/assertion-failure.js",
        url: "/qualification/assertion-failure.html",
        source: include_str!("../fixtures/qualification/assertion-failure.js"),
        admission: Admission::Standard,
    },
    Asset {
        path: "qualification/compile-failure.js",
        url: "/qualification/compile-failure.html",
        source: include_str!("../fixtures/qualification/compile-failure.js"),
        admission: Admission::Standard,
    },
    Asset {
        path: "qualification/evaluation-failure.js",
        url: "/qualification/evaluation-failure.html",
        source: include_str!("../fixtures/qualification/evaluation-failure.js"),
        admission: Admission::Standard,
    },
    Asset {
        path: "qualification/diagnostic-refusal.js",
        url: "/qualification/diagnostic-refusal.html",
        source: include_str!("../fixtures/qualification/diagnostic-refusal.js"),
        admission: Admission::Standard,
    },
    Asset {
        path: "qualification/source-refusal.js",
        url: "/qualification/source-refusal.html",
        source: include_str!("../fixtures/qualification/source-refusal.js"),
        admission: Admission::NoSourceWork,
    },
    Asset {
        path: "qualification/deadline.js",
        url: "/qualification/deadline.html",
        source: include_str!("../fixtures/qualification/deadline.js"),
        admission: Admission::Standard,
    },
];

pub(super) fn resolve(identity: &FixtureId) -> Option<Fixture> {
    ASSETS.iter().find_map(|asset| asset.resolve(identity))
}
