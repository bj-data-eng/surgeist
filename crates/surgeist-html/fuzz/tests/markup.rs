#![forbid(unsafe_code)]
use surgeist_fuzz::{engine, *};
use surgeist_html_fuzz::{MarkupCase, MarkupTarget};

/// The documented generous product envelope every default case carries.
const ENVELOPE: [u64; 19] = [
    100_000,
    100_000,
    10_000,
    1_000_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100,
    100_000,
    100_000_000,
    100_000_000,
    100_000_000,
    100_000_000,
];
const SEEDS: u8 = 12;
const TREE: &str = "surgeist-html::exact-markup-tree";

/// Handwritten HMK1 layout (see the README): header, then the body.
fn header(mode: u8, utf16: u8, a: u8, b: u8, body: u8, flags: u8, caps: [u64; 19]) -> Vec<u8> {
    let mut v = b"HMK1".to_vec();
    v.extend_from_slice(&[mode, utf16, a, b, body, flags]);
    for cap in caps {
        v.extend_from_slice(&cap.to_le_bytes());
    }
    assert_eq!(v.len(), 162);
    v
}
/// A fragment in a div context holding one text item of the given pieces.
fn text_case(pieces: &[&[u8]]) -> Vec<u8> {
    let mut v = header(1, 1, 0, 0, 0, 0, ENVELOPE);
    v.extend_from_slice(&[1, 0]); // one item, text
    v.extend_from_slice(&(pieces.len() as u16).to_le_bytes());
    for p in pieces {
        v.extend_from_slice(p);
    }
    v
}
const UNIT_D800: &[u8] = &[0, 0x00, 0xD8];
const UNIT_A: &[u8] = &[0, 0x41, 0x00];

fn evaluate(t: &MarkupTarget, c: &MarkupCase) -> CaseOutcome {
    t.evaluate(c).unwrap()
}
fn checked(o: &CaseOutcome, property: &str) -> bool {
    o.checked_properties()
        .iter()
        .any(|p| p.as_str() == property)
}

#[test]
fn strict_codec_roundtrips_isolated_surrogates_and_rejects_noncanonical_input() {
    let t = MarkupTarget::parse();
    let bytes = text_case(&[UNIT_A, UNIT_D800, &[3], &[4], &[1, 0]]);
    let c = t.decode(&bytes).unwrap();
    assert_eq!(t.encode(&c).unwrap(), bytes);
    for n in 0..bytes.len() {
        assert!(t.decode(&bytes[..n]).is_err(), "truncated at {n}");
    }
    for (at, value) in [(0, 0), (4, 2), (5, 2), (7, 2), (8, 2), (9, 2), (162, 3)] {
        let mut b = bytes.clone();
        b[at] = value;
        assert!(t.decode(&b).is_err(), "tag {at}");
    }
    let mut b = bytes.clone();
    b.push(0);
    assert!(t.decode(&b).is_err(), "trailing byte");
    let mut b = bytes.clone();
    b[10..18].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(t.decode(&b).is_err(), "cap over the envelope");
    // A NUL, CR, `<`, `&` or quote may not be a plain unit; each has a piece.
    for unit in [0u16, 13, 0x3C, 0x26, 0x22, 0x27] {
        let mut b = text_case(&[UNIT_A]);
        let n = b.len();
        b[n - 2..].copy_from_slice(&unit.to_le_bytes());
        assert!(t.decode(&b).is_err(), "unit {unit}");
    }
    // A CR piece followed by a spelled LF, and a text without a parsed value.
    assert!(t.decode(&text_case(&[&[3], &[0, 10, 0]])).is_err());
    assert!(t.decode(&text_case(&[&[5]])).is_err());
    // Unknown named reference, tag, attribute and piece.
    assert!(t.decode(&text_case(&[&[1, 99]])).is_err());
    assert!(t.decode(&text_case(&[&[9]])).is_err());
}

#[test]
fn isolated_surrogates_require_a_utf16_source() {
    let t = MarkupTarget::parse();
    let mut b = text_case(&[UNIT_A, UNIT_D800]);
    assert!(t.decode(&b).is_ok());
    b[5] = 0; // UTF-8 source
    assert!(t.decode(&b).is_err());
}

#[test]
fn handwritten_recipes_are_real_successes_with_the_independent_tree_checked() {
    let t = MarkupTarget::parse();
    for id in 0..SEEDS {
        let c = t.seed(id).unwrap();
        let r = engine::evaluate_case(&t, &c, &mut |_| Ok(())).unwrap();
        assert_eq!(
            r.outcome.kind(),
            OutcomeKind::Evaluated,
            "{id}: {:?}",
            r.outcome
        );
        assert!(checked(&r.outcome, TREE), "{id}");
        assert_eq!(
            engine::replay(&t, &r.canonical, &mut |_| Ok(()))
                .unwrap()
                .outcome,
            r.outcome
        );
    }
    assert!(t.seed(SEEDS).is_err());
}

#[test]
fn seed_files_are_the_canonical_encoding_of_the_handwritten_recipes() {
    let t = MarkupTarget::parse();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("seeds/markup");
    let mut files = std::fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    files.sort();
    let mut wanted = (0..SEEDS).map(|i| format!("{i}.case")).collect::<Vec<_>>();
    wanted.sort();
    assert_eq!(files, wanted);
    for id in 0..SEEDS {
        let b = std::fs::read(root.join(format!("{id}.case"))).unwrap();
        assert_eq!(b, t.encode(&t.seed(id).unwrap()).unwrap(), "{id}");
        assert_eq!(&b[..4], b"HMK1");
    }
}

#[test]
fn generated_recipes_round_trip_replay_and_never_fail_a_property() {
    let t = MarkupTarget::parse();
    let mut state = 0xA5A5_1234_9E37_79B9u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let (mut exact, mut free_form, mut refused) = (0, 0, 0);
    for _ in 0..400 {
        let len = 4 + (next() % 160) as usize;
        let entropy: Vec<u8> = (0..len).map(|_| next() as u8).collect();
        let r = engine::evaluate_generated(&t, &entropy, &mut |_| Ok(())).unwrap();
        let engine::GeneratedEvaluation::Evaluated(r) = r else {
            continue;
        };
        match r.outcome.kind() {
            OutcomeKind::Evaluated if checked(&r.outcome, TREE) => exact += 1,
            OutcomeKind::Evaluated => free_form += 1,
            OutcomeKind::ExpectedRejection => refused += 1,
            kind => panic!("{kind:?}: {:?} for {entropy:?}", r.outcome),
        }
        assert_eq!(
            engine::replay(&t, &r.canonical, &mut |_| Ok(()))
                .unwrap()
                .outcome,
            r.outcome
        );
    }
    assert!(
        exact > 100 && free_form > 20,
        "{exact}/{free_form}/{refused}"
    );
}

#[test]
fn every_fragment_context_builds_and_parses_free_form_markup_without_a_harness_error() {
    let t = MarkupTarget::parse();
    // Fragment, UTF-8, free-form body of the single token `<p>` then `x`.
    for context in 0..24u8 {
        let mut b = header(1, 0, context, 0, 1, 0, ENVELOPE);
        b.extend_from_slice(&2u16.to_le_bytes());
        b.extend_from_slice(&[0, 0, 0, 64]); // tokens `<p>` and `x`
        let c = t.decode(&b).unwrap();
        let o = evaluate(&t, &c);
        assert!(
            matches!(
                o.kind(),
                OutcomeKind::Evaluated | OutcomeKind::Unsupported | OutcomeKind::ExpectedRejection
            ),
            "context {context}: {o:?}"
        );
    }
}

#[test]
fn cancellation_and_lowered_caps_are_expected_rejections_with_terminal_ownership() {
    let t = MarkupTarget::parse();
    let c = t.seed(0).unwrap();
    let mut b = t.encode(&c).unwrap();
    b[9] = 1;
    let o = evaluate(&t, &t.decode(&b).unwrap());
    assert_eq!(o.kind(), OutcomeKind::ExpectedRejection, "{o:?}");
    assert!(checked(&o, "surgeist-html::terminal-ownership"));
    // Zero work credit, zero source units and one-short managed nodes each stop
    // the parse with a typed, owned refusal rather than a property failure.
    for (axis, value) in [(18usize, 0u64), (0, 0), (4, 1)] {
        let mut caps = ENVELOPE;
        caps[axis] = value;
        let o = evaluate(&t, &c.clone().with_caps(caps).unwrap());
        assert_eq!(
            o.kind(),
            OutcomeKind::ExpectedRejection,
            "axis {axis}: {o:?}"
        );
        assert!(checked(&o, "surgeist-html::terminal-ownership"));
    }
    let mut over = ENVELOPE;
    over[0] += 1;
    assert!(c.with_caps(over).is_err());
}

#[test]
fn reduction_proposes_checked_smaller_recipes_with_the_same_caps_and_stops_at_once() {
    let t = MarkupTarget::parse();
    let c = t.seed(2).unwrap();
    let before = t.encode(&c).unwrap().len();
    let mut proposals = Vec::new();
    t.reduction_candidates(&c, &mut |r| {
        proposals.push(r);
        true
    })
    .unwrap();
    assert!(proposals.len() >= 4, "{}", proposals.len());
    for r in &proposals {
        assert_eq!(r.caps(), c.caps());
        let bytes = t.encode(r).unwrap();
        assert!(bytes.len() <= before);
        assert_eq!(t.decode(&bytes).unwrap(), *r);
        assert_ne!(evaluate(&t, r).kind(), OutcomeKind::Failure);
    }
    let mut calls = 0;
    t.reduction_candidates(&c, &mut |_| {
        calls += 1;
        false
    })
    .unwrap();
    assert_eq!(calls, 1);
}

#[test]
fn shared_worker_protocol_retains_the_markup_canonical_before_real_evaluation() {
    use surgeist_fuzz::worker::*;
    let t = MarkupTarget::parse();
    let canonical = t.encode(&t.seed(3).unwrap()).unwrap();
    let mut input = Vec::new();
    write_frame(
        &mut input,
        &WorkerRequest::Evaluate {
            canonical: canonical.clone(),
        },
        262144,
    )
    .unwrap();
    let mut output = Vec::new();
    serve_io(
        &t,
        &mut std::io::Cursor::new(input),
        &mut output,
        WorkerLimits::new(262144, 262144, 1_048_576, 4).unwrap(),
    )
    .unwrap();
    let mut r = std::io::Cursor::new(output);
    assert!(
        matches!(read_frame::<_,WorkerResponse>(&mut r,262144).unwrap(),Some(WorkerResponse::Captured{canonical:b}) if b==canonical)
    );
    assert!(
        matches!(read_frame::<_,WorkerResponse>(&mut r,262144).unwrap(),Some(WorkerResponse::Evaluated{outcome,..}) if outcome.kind()==OutcomeKind::Evaluated)
    );
    assert!(
        read_frame::<_, WorkerResponse>(&mut r, 262144)
            .unwrap()
            .is_none()
    );
}

#[cfg(not(feature = "coverage"))]
#[test]
fn actual_markup_worker_binary_serves_shared_frames() {
    use std::io::Write;
    use surgeist_fuzz::worker::*;
    let t = MarkupTarget::parse();
    let canonical = t.encode(&t.seed(8).unwrap()).unwrap();
    let mut b = Vec::new();
    write_frame(
        &mut b,
        &WorkerRequest::Evaluate {
            canonical: canonical.clone(),
        },
        262144,
    )
    .unwrap();
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_html-markup-worker"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&b).unwrap();
    let o = child.wait_with_output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let mut r = std::io::Cursor::new(o.stdout);
    assert!(
        matches!(read_frame::<_,WorkerResponse>(&mut r,262144).unwrap(),Some(WorkerResponse::Captured{canonical:b}) if b==canonical)
    );
    assert!(
        matches!(read_frame::<_,WorkerResponse>(&mut r,262144).unwrap(),Some(WorkerResponse::Evaluated{outcome,..}) if outcome.kind()==OutcomeKind::Evaluated)
    );
    assert!(
        read_frame::<_, WorkerResponse>(&mut r, 262144)
            .unwrap()
            .is_none()
    );
}

#[test]
fn registry_names_the_markup_worker_and_its_source_references_exist() {
    use surgeist_fuzz::registry::TargetRegistry;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let output = std::process::Command::new(env!("CARGO"))
        .current_dir(&root)
        .args([
            "metadata",
            "--offline",
            "--locked",
            "--no-deps",
            "--format-version",
            "1",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let registry = TargetRegistry::from_metadata_json(&root, &output.stdout).unwrap();
    let target = registry
        .get(&TargetId::new("surgeist-html::markup-parse").unwrap())
        .unwrap();
    assert_eq!(target.package(), "surgeist-html-fuzz");
    assert_eq!(target.descriptor().binary(), "html-markup-worker");
    assert_eq!(target.descriptor(), MarkupTarget::parse().descriptor());
    assert!(
        target
            .fixtures()
            .iter()
            .any(|p| p.ends_with("seeds/markup") && p.is_dir())
    );
}

#[cfg(feature = "property")]
#[test]
fn finite_property_backend_credits_nonvacuous_exact_markup_success() {
    use proptest::prelude::*;
    let t = MarkupTarget::parse();
    let r = engine::run_property(
        &t,
        &Just(vec![0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
        engine::PropertyConfig {
            cases: 4,
            max_rejections: 0,
            max_shrink_iterations: 8,
            seed: 1172,
        },
        &mut |_| Ok(()),
    )
    .unwrap();
    assert_eq!(r.counts.evaluated(), 4);
    assert!(r.failure.is_none());
}
#[cfg(feature = "coverage")]
#[test]
fn finite_coverage_backend_and_canonical_replay_share_the_markup_consumer() {
    let t = MarkupTarget::parse();
    let mut driver = engine::CoverageDriver::new(&t, None).unwrap();
    let mut bytes = Vec::new();
    let r = driver
        .test_one(&[0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9], &mut |b| {
            bytes = b.to_vec();
            Ok(())
        })
        .unwrap();
    let engine::GeneratedEvaluation::Evaluated(r) = r else {
        panic!("generation rejected")
    };
    assert_eq!(r, engine::replay(&t, &bytes, &mut |_| Ok(())).unwrap());
}
