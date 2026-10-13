//! Admission of the resolved coverage engine before recording its provenance.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::{FuzzError, FuzzErrorKind, Result};

const ADMITTED_VERSION: &str = "0.4.13";

fn malformed(detail: impl Into<String>) -> FuzzError {
    FuzzError::new(
        FuzzErrorKind::Registry,
        "validate coverage engine metadata",
        detail,
    )
}

fn not_admitted(detail: impl Into<String>) -> FuzzError {
    FuzzError::new(
        FuzzErrorKind::MissingCapability,
        "admit coverage engine",
        detail,
    )
}

fn text<'a>(value: &'a Value, field: &str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| malformed(format!("missing or invalid {field}")))
}

fn array<'a>(value: &'a Value, field: &str) -> Result<&'a [Value]> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| malformed(format!("missing or invalid {field}")))
}

fn indexed<'a>(values: &'a [Value], label: &str) -> Result<BTreeMap<&'a str, &'a Value>> {
    let mut entries = BTreeMap::new();
    for value in values {
        let id = text(value, "id")?;
        if entries.insert(id, value).is_some() {
            return Err(malformed(format!("duplicate {label} package ID {id}")));
        }
    }
    Ok(entries)
}

/// The caller must obtain full format-version 1 metadata for the prepared
/// coverage source, with the build's features and `--filter-platform` target.
/// Cargo's resolve edges already exclude disabled optional dependencies and
/// nonmatching platforms. Declared manifest dependencies do not establish this.
pub(crate) fn validate_engine_identity(metadata: &Value, owner_package_id: &str) -> Result<String> {
    if metadata.get("version").and_then(Value::as_u64) != Some(1) {
        return Err(malformed("Cargo metadata format version must be 1"));
    }
    let packages = indexed(array(metadata, "packages")?, "metadata")?;
    let resolve = metadata
        .get("resolve")
        .filter(|resolve| resolve.is_object())
        .ok_or_else(|| malformed("missing Cargo resolve graph; full metadata is required"))?;
    let nodes = indexed(array(resolve, "nodes")?, "resolve")?;
    if !packages.contains_key(owner_package_id) || !nodes.contains_key(owner_package_id) {
        return Err(malformed(format!(
            "selected owner package ID {owner_package_id} is absent from metadata or resolve graph"
        )));
    }

    // Each package ID enters the frontier once, so cycles and shared paths are
    // bounded by the supplied graph rather than growing the traversal.
    let mut visited = BTreeSet::from([owner_package_id]);
    let mut frontier = vec![owner_package_id];
    let mut engines = BTreeMap::new();
    while let Some(id) = frontier.pop() {
        let package = packages
            .get(id)
            .ok_or_else(|| malformed(format!("resolve package ID {id} has no package record")))?;
        let name = text(package, "name")?;
        let version = text(package, "version")?;
        if name == "libfuzzer-sys" {
            engines.insert(id, version);
        }
        let node = nodes
            .get(id)
            .ok_or_else(|| malformed(format!("package ID {id} has no resolve node")))?;
        for dependency in array(node, "deps")? {
            let dependency_id = text(dependency, "pkg")?;
            let kinds = array(dependency, "dep_kinds")?;
            if kinds.is_empty() {
                return Err(malformed(format!(
                    "dependency {dependency_id} has no dependency kinds"
                )));
            }
            let mut normal = false;
            for kind in kinds {
                match kind.get("kind") {
                    Some(Value::Null) => normal = true,
                    Some(Value::String(kind)) if matches!(kind.as_str(), "dev" | "build") => {}
                    _ => {
                        return Err(malformed(format!(
                            "dependency {dependency_id} has an invalid kind"
                        )));
                    }
                }
                match kind.get("target") {
                    Some(Value::Null | Value::String(_)) => {}
                    _ => {
                        return Err(malformed(format!(
                            "dependency {dependency_id} has an invalid target"
                        )));
                    }
                }
            }
            if normal && visited.insert(dependency_id) {
                frontier.push(dependency_id);
            }
        }
    }
    if engines.is_empty() {
        return Err(not_admitted(
            "selected owner has no enabled normal libfuzzer-sys dependency",
        ));
    }
    if engines.len() != 1 {
        return Err(not_admitted(format!(
            "selected owner has ambiguous libfuzzer-sys engines: {}",
            engines.keys().copied().collect::<Vec<_>>().join(", ")
        )));
    }
    let (id, version) = engines.into_iter().next().expect("one engine checked");
    if version != ADMITTED_VERSION {
        return Err(not_admitted(format!(
            "resolved libfuzzer-sys {version} ({id}) is unsupported; admitted version is {ADMITTED_VERSION}"
        )));
    }
    Ok(format!("libfuzzer-sys {version}"))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn package(id: &str, name: &str, version: &str) -> Value {
        json!({"id": id, "name": name, "version": version})
    }

    fn node(id: &str, dependencies: Vec<Value>) -> Value {
        json!({"id": id, "deps": dependencies, "features": []})
    }

    fn dependency(id: &str, kind: Value) -> Value {
        json!({"name": "renamed_dependency", "pkg": id, "dep_kinds": [{"kind": kind, "target": null}]})
    }

    fn metadata(owner_dependencies: Vec<Value>) -> Value {
        json!({
            "version": 1,
            "packages": [package("owner", "owner-fuzz", "0.1.0"), package("engine", "libfuzzer-sys", "0.4.13")],
            "resolve": {"nodes": [node("owner", owner_dependencies), node("engine", vec![])], "root": "owner"}
        })
    }

    fn assert_rejected(metadata: &Value, kind: FuzzErrorKind, detail: &str) {
        let error = validate_engine_identity(metadata, "owner").unwrap_err();
        assert_eq!(error.kind(), kind);
        assert!(error.detail().contains(detail), "{error}");
    }

    #[test]
    fn direct_normal_engine_has_resolved_provenance() {
        assert_eq!(
            validate_engine_identity(&metadata(vec![dependency("engine", Value::Null)]), "owner")
                .unwrap(),
            "libfuzzer-sys 0.4.13"
        );
    }

    #[test]
    fn unrelated_workspace_engine_does_not_establish_owner_capability() {
        let mut metadata = metadata(vec![]);
        metadata["packages"]
            .as_array_mut()
            .unwrap()
            .push(package("other", "other-fuzz", "0.1.0"));
        metadata["resolve"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(node("other", vec![dependency("engine", Value::Null)]));
        assert_rejected(
            &metadata,
            FuzzErrorKind::MissingCapability,
            "no enabled normal",
        );
    }

    #[test]
    fn disabled_optional_and_non_normal_engines_do_not_establish_capability() {
        for kind in [json!("dev"), json!("build")] {
            assert_rejected(
                &metadata(vec![dependency("engine", kind)]),
                FuzzErrorKind::MissingCapability,
                "no enabled normal",
            );
        }
        let mut disabled = metadata(vec![]);
        disabled["packages"][0]["dependencies"] =
            json!([{"name": "libfuzzer-sys", "optional": true, "kind": null}]);
        assert_rejected(
            &disabled,
            FuzzErrorKind::MissingCapability,
            "no enabled normal",
        );
    }

    #[test]
    fn transitive_normal_engine_is_admitted_despite_unrelated_wrong_version() {
        let mut metadata = metadata(vec![dependency("adapter", Value::Null)]);
        metadata["packages"].as_array_mut().unwrap().extend([
            package("adapter", "engine-adapter", "0.1.0"),
            package("other-engine", "libfuzzer-sys", "0.4.12"),
        ]);
        metadata["resolve"]["nodes"]
            .as_array_mut()
            .unwrap()
            .extend([
                node("adapter", vec![dependency("engine", Value::Null)]),
                node("other-engine", vec![]),
            ]);
        assert_eq!(
            validate_engine_identity(&metadata, "owner").unwrap(),
            "libfuzzer-sys 0.4.13"
        );
    }

    #[test]
    fn unsupported_resolved_engine_version_is_explicit() {
        let mut metadata = metadata(vec![dependency("engine", Value::Null)]);
        metadata["packages"][1]["version"] = json!("0.4.12");
        assert_rejected(&metadata, FuzzErrorKind::MissingCapability, "0.4.12");
    }

    #[test]
    fn two_engine_package_ids_are_ambiguous_even_at_the_same_version() {
        let mut metadata = metadata(vec![
            dependency("engine", Value::Null),
            dependency("other-engine", Value::Null),
        ]);
        metadata["packages"].as_array_mut().unwrap().push(package(
            "other-engine",
            "libfuzzer-sys",
            "0.4.13",
        ));
        metadata["resolve"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(node("other-engine", vec![]));
        assert_rejected(&metadata, FuzzErrorKind::MissingCapability, "ambiguous");
    }

    #[test]
    fn missing_or_malformed_graph_is_rejected() {
        for (resolve, detail) in [
            (Value::Null, "resolve"),
            (json!({}), "nodes"),
            (json!({"nodes": "invalid"}), "nodes"),
        ] {
            let mut metadata = metadata(vec![]);
            metadata["resolve"] = resolve;
            assert_rejected(&metadata, FuzzErrorKind::Registry, detail);
        }
        let mut missing = metadata(vec![]);
        missing.as_object_mut().unwrap().remove("resolve");
        assert_rejected(&missing, FuzzErrorKind::Registry, "resolve");
    }

    #[test]
    fn reachable_missing_packages_nodes_and_kinds_are_rejected() {
        let mut missing_package = metadata(vec![dependency("absent", Value::Null)]);
        assert_rejected(
            &missing_package,
            FuzzErrorKind::Registry,
            "no package record",
        );
        missing_package["resolve"]["nodes"] =
            json!([node("owner", vec![dependency("engine", Value::Null)])]);
        assert_rejected(&missing_package, FuzzErrorKind::Registry, "no resolve node");
        let mut missing_kind = metadata(vec![dependency("engine", Value::Null)]);
        missing_kind["resolve"]["nodes"][0]["deps"][0]["dep_kinds"][0]
            .as_object_mut()
            .unwrap()
            .remove("kind");
        assert_rejected(&missing_kind, FuzzErrorKind::Registry, "invalid kind");
    }

    #[test]
    fn duplicate_package_ids_and_wrong_format_are_rejected() {
        let mut metadata = metadata(vec![dependency("engine", Value::Null)]);
        metadata["packages"].as_array_mut().unwrap().push(package(
            "engine",
            "libfuzzer-sys",
            "0.4.13",
        ));
        assert_rejected(&metadata, FuzzErrorKind::Registry, "duplicate");
        metadata["version"] = json!(2);
        assert_rejected(&metadata, FuzzErrorKind::Registry, "format version");
    }

    #[test]
    fn owner_selection_requires_the_exact_package_id() {
        let metadata = metadata(vec![dependency("engine", Value::Null)]);
        let error = validate_engine_identity(&metadata, "owner-fuzz").unwrap_err();
        assert_eq!(error.kind(), FuzzErrorKind::Registry);
        assert!(
            error
                .detail()
                .contains("selected owner package ID owner-fuzz")
        );
    }

    #[test]
    fn mixed_dependency_kinds_preserve_the_normal_edge() {
        let mut metadata = metadata(vec![dependency("engine", json!("dev"))]);
        metadata["resolve"]["nodes"][0]["deps"][0]["dep_kinds"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind": null, "target": "cfg(unix)"}));
        assert_eq!(
            validate_engine_identity(&metadata, "owner").unwrap(),
            "libfuzzer-sys 0.4.13"
        );
    }

    #[test]
    fn cycles_and_repeated_normal_paths_are_visited_once() {
        let mut metadata = metadata(vec![
            dependency("engine", Value::Null),
            dependency("adapter", Value::Null),
        ]);
        metadata["packages"].as_array_mut().unwrap().push(package(
            "adapter",
            "engine-adapter",
            "0.1.0",
        ));
        metadata["resolve"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(node(
                "adapter",
                vec![
                    dependency("owner", Value::Null),
                    dependency("engine", Value::Null),
                ],
            ));
        assert_eq!(
            validate_engine_identity(&metadata, "owner").unwrap(),
            "libfuzzer-sys 0.4.13"
        );
    }
}
