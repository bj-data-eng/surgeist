use super::*;
use surgeist_test::checkpoint::Identity as _;

#[test]
fn native_record_preserves_single_stage_wire_bytes_and_sample_selection() {
    let identity = Identity::Native {
        population: "detached-text",
        managed: 1000,
        sample: Sample::First,
    };
    assert_eq!(
        identity.record("unit-token", 42, "edited", None),
        "token=unit-token\npid=42\npopulation=detached-text\nmanaged=1000\nsample=0\nstage=edited\n"
    );
    assert_eq!(
        identity.record("unit-token", 42, "edited", Some(1)),
        "token=unit-token\npid=42\npopulation=detached-text\nmanaged=1000\nsample=0\nstage=edited\ncheckpoint=1\n"
    );
    for (literal, sample) in [
        ("warmup", Sample::Warmup),
        ("0", Sample::First),
        ("1", Sample::Second),
        ("2", Sample::Third),
    ] {
        assert_eq!(Sample::parse(literal).unwrap(), sample);
        assert_eq!(sample.name(), literal);
    }
    assert!(Sample::parse("3").is_err());
}

#[test]
fn parsed_record_binds_complete_geometry_and_ordinal() {
    let identity = Identity::Parsed {
        rows: 4,
        columns: 8,
        cells: 32,
        parsed_nodes: 75,
        managed: 76,
        sample: Sample::First,
    };
    let expected = "token=unit-token\npid=42\nfixture=parsed-table\nrows=4\ncolumns=8\ncells=32\nparsed_nodes=75\nmanaged=76\nsample=0\nstage=edited\ncheckpoint=0\n";
    assert_eq!(identity.record("unit-token", 42, "edited", None), expected);
    assert_eq!(
        identity.record("unit-token", 42, "edited", Some(0)),
        expected
    );
    assert_eq!(
        identity.record("second-token", 91, "released", Some(13)),
        "token=second-token\npid=91\nfixture=parsed-table\nrows=4\ncolumns=8\ncells=32\nparsed_nodes=75\nmanaged=76\nsample=0\nstage=released\ncheckpoint=13\n"
    );
}

#[test]
fn layered_record_binds_mode_id_pattern_geometry_and_ordinal() {
    let identity = Identity::Layered {
        mode: "combined",
        ids: "row-ids",
        rows: 2,
        columns: 3,
        cells: 6,
        managed: 16,
        sample: Sample::First,
    };
    assert_eq!(
        identity.record("unit-token", 42, "edited", Some(1)),
        "token=unit-token\npid=42\nfixture=native-layered\nmode=combined\nids=row-ids\nrows=2\ncolumns=3\ncells=6\nmanaged=16\nsample=0\nstage=edited\ncheckpoint=1\n"
    );
    assert_eq!(
        identity.record("unit-token", 42, "released", None),
        "token=unit-token\npid=42\nfixture=native-layered\nmode=combined\nids=row-ids\nrows=2\ncolumns=3\ncells=6\nmanaged=16\nsample=0\nstage=released\ncheckpoint=0\n"
    );
}
