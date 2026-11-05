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
    let (fields, verdict) = compare(&golden).unwrap();
    assert!(fields.is_empty(), "{fields:?}");
    assert_eq!(verdict.unwrap().raw, RawVerdict::Matches);

    golden.expectations.children[1].x = Some(97.0);
    golden.expectations.children[2]
        .range_rectangles
        .as_mut()
        .unwrap()[0]
        .advance = 127.0;
    let (fields, verdict) = compare(&golden).unwrap();
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
        compare(&golden)
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
