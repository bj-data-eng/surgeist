#![forbid(unsafe_code)]
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use surgeist_fuzz::{
    registry::{FuzzManifest, TargetRegistry},
    *,
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "surgeist-registry-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("owner/fuzz/seeds")).unwrap();
        fs::write(root.join("owner/Cargo.toml"), b"owner").unwrap();
        fs::write(root.join("owner/fuzz/Cargo.toml"), b"fuzz").unwrap();
        Self { root }
    }
    fn descriptor(&self, manifest: FuzzManifest) {
        fs::write(
            self.root.join("owner/fuzz/fuzz.toml"),
            toml::to_string(&manifest).unwrap(),
        )
        .unwrap();
    }
    fn metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "workspace_root": self.root, "workspace_members":["owner","fuzz"],
            "packages":[
                {"id":"owner","name":"surgeist-example","manifest_path":self.root.join("owner/Cargo.toml"),"targets":[],"features":{},"metadata":{}},
                {"id":"fuzz","name":"surgeist-example-fuzz","manifest_path":self.root.join("owner/fuzz/Cargo.toml"),"targets":[{"name":"example-target","kind":["bin"],"required-features":["property"]}],"features":{"property":[]},"metadata":{"surgeist-fuzz":{"descriptor":"fuzz.toml"}}}
            ], "irrelevant_cargo_field":"tolerated"
        })
    }
    fn discover(&self, metadata: &serde_json::Value) -> Result<TargetRegistry> {
        TargetRegistry::from_metadata_json(&self.root, &serde_json::to_vec(metadata).unwrap())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn target() -> TargetDescriptor {
    TargetDescriptor::new(TargetDescriptorSpec {
        id: TargetId::new("surgeist-example::bytes").unwrap(),
        owner: "surgeist-example".into(),
        binary: "example-target".into(),
        properties: vec![PropertyId::new("surgeist-example::length").unwrap()],
        source_references: vec!["https://example.org/requirements".into()],
        format: CaseFormat::new("surgeist-example::bytes", 1).unwrap(),
        generator_identity: GeneratorIdentity::new("surgeist-example::bytes", 1).unwrap(),
        encoding: InputEncoding::RawBytes,
        limits: InputLimits::new(64, 0, 0, 0, 0).unwrap(),
        requirements: BuildRequirements::new(vec!["property".into()], vec![]).unwrap(),
        min_evaluated: 1,
        max_generation_rejections: 0,
    })
    .unwrap()
}
fn declaration() -> FuzzManifest {
    FuzzManifest {
        schema: 1,
        targets: vec![target()],
        fixtures: vec![RelativePath::new("seeds").unwrap()],
    }
}

#[test]
fn discovers_owner_local_declared_target_without_building() {
    let f = Fixture::new();
    f.descriptor(declaration());
    let registry = f.discover(&f.metadata()).unwrap();
    let entry = registry.get(target().id()).unwrap();
    assert_eq!(entry.package(), "surgeist-example-fuzz");
    assert_eq!(entry.fixtures(), [f.root.join("owner/fuzz/seeds").canonicalize().unwrap()]);
    assert!(!f.root.join("target").exists());
}
#[test]
fn missing_binary_feature_or_owner_rejected() {
    let f = Fixture::new();
    f.descriptor(declaration());
    for field in ["targets", "features"] {
        let mut m = f.metadata();
        m["packages"][1][field] = if field == "targets" {
            serde_json::json!([])
        } else {
            serde_json::json!({})
        };
        assert!(f.discover(&m).is_err());
    }
    let mut m = f.metadata();
    m["packages"][1]["name"] = serde_json::json!("surgeist-missing-fuzz");
    assert!(f.discover(&m).is_err());
    let mut m = f.metadata();
    m["packages"][1]["targets"][0]["required-features"] = serde_json::json!(["extra"]);
    assert!(f.discover(&m).is_err());
}
#[test]
fn descriptor_schema_collisions_and_unknown_fields_rejected() {
    let f = Fixture::new();
    let mut d = declaration();
    d.targets.push(target());
    f.descriptor(d);
    assert!(f.discover(&f.metadata()).is_err());
    let mut d = declaration();
    d.schema = 2;
    f.descriptor(d);
    assert!(f.discover(&f.metadata()).is_err());
    f.descriptor(declaration());
    let p = f.root.join("owner/fuzz/fuzz.toml");
    let text = fs::read_to_string(&p).unwrap();
    fs::write(&p, format!("unknown=true\n{text}")).unwrap();
    assert!(f.discover(&f.metadata()).is_err());
    let mut m = f.metadata();
    m["packages"][1]["metadata"]["surgeist-fuzz"]["unknown"] = serde_json::json!(true);
    assert!(f.discover(&m).is_err());
}
#[test]
fn path_escape_missing_fixture_and_wrong_location_rejected() {
    let f = Fixture::new();
    let mut d = declaration();
    d.fixtures = vec![RelativePath::new("missing").unwrap()];
    f.descriptor(d);
    assert!(f.discover(&f.metadata()).is_err());
    f.descriptor(declaration());
    let mut m = f.metadata();
    m["packages"][1]["manifest_path"] = serde_json::json!(f.root.join("owner/Cargo.toml"));
    assert!(f.discover(&m).is_err());
    let mut m = f.metadata();
    m["packages"][1]["metadata"]["surgeist-fuzz"]["descriptor"] = serde_json::json!("../fuzz.toml");
    assert!(f.discover(&m).is_err());
}
#[cfg(unix)]
#[test]
fn symlinked_fixture_cannot_escape_package() {
    let f = Fixture::new();
    std::os::unix::fs::symlink(&f.root, f.root.join("owner/fuzz/escape")).unwrap();
    let mut d = declaration();
    d.fixtures = vec![RelativePath::new("escape").unwrap()];
    f.descriptor(d);
    assert!(f.discover(&f.metadata()).is_err());
}
#[test]
fn unregistered_workspace_is_valid_empty_discovery() {
    let f = Fixture::new();
    let mut m = f.metadata();
    m["packages"][1]["metadata"] = serde_json::json!({});
    assert_eq!(f.discover(&m).unwrap().targets().count(), 0);
    assert!(TargetRegistry::from_metadata_json(Path::new("/missing"), b"{}").is_err());
}
