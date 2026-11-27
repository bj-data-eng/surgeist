use std::{fs, path::Path};

use super::*;

const OUTPUT: &str = "xml/block/fri06_atomic_inline_percentage_block_size__border_box_ltr.xml";

fn artifacts() -> (String, String, Vec<u8>) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/layout/browser_parity/captured/prepared-atomic-percentage-qualification");
    (
        fs::read_to_string(root.join(OUTPUT)).unwrap(),
        fs::read_to_string(root.join("xml/generation-reports/all.json")).unwrap(),
        fs::read(root.join(profile::SOURCE)).unwrap(),
    )
}

fn parse(xml: &str, report: &str, source: &[u8]) -> Result<Golden, Error> {
    Golden::parse_with_prepared_atomic_percentage(xml, report, profile::SOURCE, source, OUTPUT)
}

#[test]
fn complete_capture_keeps_all_exact_box_and_range_differences() {
    let (xml, report, source) = artifacts();
    let mut golden = parse(&xml, &report, &source).unwrap();
    let historical = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(capture_authority::SpecializedProfile::PreparedAtomicPercentage.artifact_root())
        .join(OUTPUT.replacen("xml/", "historical/", 1));
    let mut historical = Golden::parse_file(historical).unwrap();
    verify_historical(&golden, &historical).unwrap();
    historical.expectations.children[1].height = Some(41.0);
    assert!(verify_historical(&golden, &historical).is_err());

    let (fields, verdict) = compare(&golden, |_| Ok(())).unwrap();
    assert!(fields.is_empty(), "{fields:?}");
    assert_eq!(verdict.unwrap().raw, RawVerdict::Matches);

    golden.expectations.children[1].x = Some(97.0);
    golden.expectations.children[2]
        .range_rectangles
        .as_mut()
        .unwrap()[0]
        .advance = 127.0;
    let (fields, verdict) = compare(&golden, |_| Ok(())).unwrap();
    assert_eq!(fields.len(), 2, "{fields:?}");
    assert!(fields[0].contains("browser: 97.0") && fields[0].contains("native: 96.0"));
    assert!(
        fields[1].contains("browser_advance: 127.0") && fields[1].contains("native_advance: 128.0")
    );
    assert!(matches!(
        verdict,
        Err(QualificationError::DifferenceSetChanged { extra, .. }) if extra == fields
    ));
}

#[test]
fn capture_refuses_missing_authority_stale_basis_and_numeric_substitution() {
    let (xml, report, source) = artifacts();
    assert!(
        Golden::parse(&xml)
            .unwrap_err()
            .to_string()
            .contains("actual containing frame")
    );

    let mut golden = parse(&xml, &report, &source).unwrap();
    golden
        .root
        .style
        .attrs
        .insert("height".into(), "60px".into());
    assert!(
        compare(&golden, |_| Ok(()))
            .unwrap_err()
            .to_string()
            .contains("sizing frame changed")
    );

    let substitute = xml.replace(
        "&quot;authoredSize&quot;:{&quot;width&quot;:&quot;20px&quot;,&quot;height&quot;:&quot;50%&quot;}",
        "&quot;width&quot;:20,&quot;height&quot;:40",
    );
    assert_ne!(xml, substitute);
    assert!(
        parse(&substitute, &report, &source)
            .unwrap_err()
            .to_string()
            .contains("output binding changed")
    );
    let changed_report = report.replace(&sha256(xml.as_bytes()), &sha256(substitute.as_bytes()));
    assert_ne!(report, changed_report);
    assert!(
        parse(&substitute, &changed_report, &source)
            .unwrap_err()
            .to_string()
            .contains("authored source or font metrics changed")
    );
}

#[test]
fn rehashed_capture_cannot_change_effective_variant_or_containing_frame() {
    let (xml, report, source) = artifacts();
    let changed_modes = [
        xml.replace(
            "source-tag=\"span\"",
            "source-tag=\"span\" box-sizing=\"content-box\"",
        ),
        xml.replace(" direction=\"ltr\"", " direction=\"rtl\"")
            .replace(
                "&quot;direction&quot;:&quot;ltr&quot;",
                "&quot;direction&quot;:&quot;rtl&quot;",
            ),
        xml.replace("800px", "801px"),
    ];
    for changed in changed_modes {
        assert_ne!(xml, changed);
        let changed_report = report.replace(&sha256(xml.as_bytes()), &sha256(changed.as_bytes()));
        let verdict = parse(&changed, &changed_report, &source);
        assert!(
            verdict.as_ref().err().is_some_and(|error| {
                let reason = error.to_string();
                reason.contains("variant") || reason.contains("containing frame")
            }),
            "rehashed changed box mode, direction or frame was not specifically refused"
        );
    }
}

#[test]
fn changed_native_atomic_custody_preserves_raw_box_and_range_differences() {
    let (xml, report, source) = artifacts();
    let mut golden = parse(&xml, &report, &source).unwrap();
    golden
        .root
        .style
        .attrs
        .insert("height".into(), "60px".into());
    // Exercise the comparison after Native succeeds with different actual
    // geometry. The public reader rejects this changed preparation input.
    golden.prepared_atomic_basis = Some(Basis::new(&golden.root, &golden.viewport.root_context));
    golden.expectations.children[2]
        .range_rectangles
        .as_mut()
        .unwrap()[0]
        .advance = 127.0;
    let (raw, qualified) = compare(&golden, |_| Ok(())).unwrap();
    assert!(
        raw.iter()
            .any(|field| field.contains("browser: 40.0") && field.contains("native: 30.0")),
        "{raw:?}"
    );
    assert!(
        raw.iter()
            .any(|field| field.contains("browser_advance: 127.0")
                && field.contains("native_advance: 128.0")),
        "{raw:?}"
    );
    assert!(qualified.unwrap_err().to_string().contains("custody"));
}

#[test]
fn script_custody_admission_atomic_input_map_accepts_closed_producers_and_preserves_other_pins() {
    let (xml, report, source) = artifacts();
    parse(&xml, &report, &source).expect("the retained atomic producer remains admissible");
    let mut report = capture_authority::unique_json(&report).unwrap();
    report["metadata"]["inputs"][capture_authority::ROOT_HELPER_INPUT] =
        serde_json::json!(crate::capture_helper_identity::SCRIPT_CUSTODY_HELPER_SHA256);
    parse(&xml, &report.to_string(), &source)
        .expect("the script-custody atomic producer remains admissible");
    for helper in [
        "0".repeat(64),
        "af0f458c290f0261106810fa3e42442afe7a5599872774130ac3244548cb74ee".to_owned(),
        "bd1b351e488f846b2e73947746ced8d8b290f49b108e220c228b2c4d94c069c0".to_owned(),
    ] {
        let mut invalid = report.clone();
        invalid["metadata"]["inputs"][capture_authority::ROOT_HELPER_INPUT] =
            serde_json::json!(helper);
        assert!(parse(&xml, &invalid.to_string(), &source).is_err());
    }
    let mut missing = report.clone();
    missing["metadata"]["inputs"]
        .as_object_mut()
        .unwrap()
        .remove(capture_authority::ROOT_HELPER_INPUT);
    assert!(parse(&xml, &missing.to_string(), &source).is_err());
    report["metadata"]["inputs"]["fonts/Ahem.ttf"] = serde_json::json!("0".repeat(64));
    assert!(parse(&xml, &report.to_string(), &source).is_err());
}
