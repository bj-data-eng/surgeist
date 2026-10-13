#![forbid(unsafe_code)]

use std::error::Error;
use surgeist_fuzz::{
    BuildRequirements, Capability, CaseFormat, CaseOutcome, CaseTarget, CoverageAssessment,
    ExecutionCounts, FailureClass, FailureKey, FuzzError, FuzzErrorKind, Generation,
    GenerationRejection, GeneratorIdentity, InputEncoding, InputLimits, InputUsage, OutcomeKind,
    PropertyId, Result, TargetDescriptor, TargetDescriptorSpec, TargetId,
};

fn property() -> PropertyId {
    PropertyId::new("surgeist-test::sum-is-even").unwrap()
}

fn spec() -> TargetDescriptorSpec {
    TargetDescriptorSpec {
        id: TargetId::new("surgeist-test::even-sum").unwrap(),
        owner: "surgeist-test".into(),
        binary: "even-sum".into(),
        properties: vec![property()],
        source_references: vec!["https://example.test/requirements/even".into()],
        format: CaseFormat::new("surgeist-test::two-bytes", 1).unwrap(),
        generator_identity: GeneratorIdentity::new("surgeist-test::byte-pair", 1).unwrap(),
        encoding: InputEncoding::Structured,
        limits: InputLimits::new(2, 0, 0, 0, 0).unwrap(),
        requirements: BuildRequirements::new(vec![], vec![Capability::Replay]).unwrap(),
        min_evaluated: 2,
        max_generation_rejections: 1,
    }
}

#[test]
fn qualified_ids_and_codec_revisions_reject_noncanonical_construction_and_serde() {
    for value in [
        "",
        "test::property",
        "surgeist-::x",
        "surgeist-test::",
        "surgeist-test::A",
        "surgeist-test::a--b",
        "surgeist-test::a::b",
        " surgeist-test::a",
    ] {
        assert!(TargetId::new(value).is_err(), "accepted {value:?}");
        assert!(PropertyId::new(value).is_err(), "accepted {value:?}");
        assert!(
            serde_json::from_str::<PropertyId>(&serde_json::to_string(value).unwrap()).is_err()
        );
    }
    let id = TargetId::new("surgeist-test::byte-round-trip").unwrap();
    assert_eq!(id.owner(), "surgeist-test");
    assert_eq!(
        serde_json::to_string(&id).unwrap(),
        "\"surgeist-test::byte-round-trip\""
    );
    assert_eq!(
        serde_json::from_str::<TargetId>(&serde_json::to_string(&id).unwrap()).unwrap(),
        id
    );
    assert!(CaseFormat::new("surgeist-test::pair", 0).is_err());
    assert!(GeneratorIdentity::new("surgeist-test::pair", 0).is_err());
    let first = CaseFormat::new("surgeist-test::pair", 1).unwrap();
    assert_eq!(
        first
            .require(&CaseFormat::new("surgeist-test::pair", 2).unwrap())
            .unwrap_err()
            .kind(),
        FuzzErrorKind::UnsupportedFormat
    );
    for json in [
        r#"{"identity":"surgeist-test::pair","revision":1,"extra":true}"#,
        r#"{"identity":"surgeist-test::pair","revision":1,"revision":2}"#,
        r#"{"identity":"surgeist-test::pair"}"#,
        r#"{"identity":"surgeist-test::pair","revision":0}"#,
    ] {
        assert!(serde_json::from_str::<CaseFormat>(json).is_err());
        assert!(serde_json::from_str::<GeneratorIdentity>(json).is_err());
    }
}

#[test]
fn admission_checks_all_exact_hard_limits_and_rejects_inconsistent_depth() {
    let limits = InputLimits::new(10, 4, 3, 5, 2).unwrap();
    limits
        .check(InputUsage::new(10, 4, 3, 5, 2).unwrap())
        .unwrap();
    for usage in [
        (11, 4, 3, 5, 2),
        (10, 5, 3, 5, 2),
        (10, 4, 4, 5, 2),
        (10, 4, 3, 6, 2),
        (10, 4, 3, 5, 3),
    ] {
        assert_eq!(
            limits
                .check(InputUsage::new(usage.0, usage.1, usage.2, usage.3, usage.4).unwrap())
                .unwrap_err()
                .kind(),
            FuzzErrorKind::InvalidLimits
        );
    }
    for (nodes, depth) in [(0, 1), (2, 0), (1, 2)] {
        assert!(InputLimits::new(10, nodes, depth, 0, 0).is_err());
        assert!(InputUsage::new(0, nodes, depth, 0, 0).is_err());
    }
    assert!(InputLimits::new(0, 0, 0, 0, 0).is_err());
    InputLimits::new(1, 0, 0, 0, 0)
        .unwrap()
        .check(InputUsage::new(0, 0, 0, 0, 0).unwrap())
        .unwrap();
    let encoded = serde_json::to_string(&limits).unwrap();
    assert_eq!(
        serde_json::from_str::<InputLimits>(&encoded).unwrap(),
        limits
    );
    assert!(
        serde_json::from_str::<InputLimits>(
            r#"{"bytes":10,"nodes":0,"depth":1,"text_units":0,"operations":0}"#
        )
        .is_err()
    );
}

#[test]
fn descriptor_construction_rejects_owner_collisions_ambiguous_metadata_and_empty_assertions() {
    let mut bad = spec();
    bad.owner = "surgeist-css".into();
    assert!(TargetDescriptor::new(bad).is_err());
    let mut bad = spec();
    bad.binary = "../escape".into();
    assert!(TargetDescriptor::new(bad).is_err());
    let mut bad = spec();
    bad.properties.push(property());
    assert!(TargetDescriptor::new(bad).is_err());
    let mut bad = spec();
    bad.properties.clear();
    assert!(TargetDescriptor::new(bad).is_err());
    let mut bad = spec();
    bad.properties = vec![PropertyId::new("surgeist-css::sum-is-even").unwrap()];
    assert!(TargetDescriptor::new(bad).is_err());
    let mut bad = spec();
    bad.format = CaseFormat::new("surgeist-css::pair", 1).unwrap();
    assert!(TargetDescriptor::new(bad).is_err());
    let mut bad = spec();
    bad.generator_identity = GeneratorIdentity::new("surgeist-css::pair", 1).unwrap();
    assert!(TargetDescriptor::new(bad).is_err());
    let mut bad = spec();
    bad.source_references = vec![" ".into()];
    assert!(TargetDescriptor::new(bad).is_err());
    let mut bad = spec();
    bad.min_evaluated = 0;
    assert!(TargetDescriptor::new(bad).is_err());
    assert!(BuildRequirements::new(vec!["../bad".into()], vec![]).is_err());
    assert!(BuildRequirements::new(vec!["test".into(), "test".into()], vec![]).is_err());
    assert!(BuildRequirements::new(vec![], vec![Capability::Replay, Capability::Replay]).is_err());
    let requirements =
        BuildRequirements::new(vec![], vec![Capability::Replay, Capability::CaseCapture]).unwrap();
    assert_eq!(
        requirements
            .require_capabilities(&[Capability::Replay])
            .unwrap_err()
            .kind(),
        FuzzErrorKind::MissingCapability
    );
    requirements
        .require_capabilities(&[Capability::Replay, Capability::CaseCapture])
        .unwrap();
}

#[test]
fn descriptor_json_and_toml_round_trips_preserve_strict_schema() {
    let descriptor = TargetDescriptor::new(spec()).unwrap();
    let json = serde_json::to_string(&descriptor).unwrap();
    assert_eq!(
        serde_json::from_str::<TargetDescriptor>(&json).unwrap(),
        descriptor
    );
    let toml = toml::to_string(&descriptor).unwrap();
    assert_eq!(
        toml::from_str::<TargetDescriptor>(&toml).unwrap(),
        descriptor
    );
    let mut object = serde_json::to_value(&descriptor).unwrap();
    object["unknown"] = true.into();
    assert!(serde_json::from_value::<TargetDescriptor>(object).is_err());
    let mut object = serde_json::to_value(&descriptor).unwrap();
    object.as_object_mut().unwrap().remove("min_evaluated");
    assert!(serde_json::from_value::<TargetDescriptor>(object).is_err());
    let duplicated = json.replacen(
        "\"min_evaluated\":2",
        "\"min_evaluated\":2,\"min_evaluated\":3",
        1,
    );
    assert!(serde_json::from_str::<TargetDescriptor>(&duplicated).is_err());
    let mut object = serde_json::to_value(&descriptor).unwrap();
    object["encoding"] = "unknown-format".into();
    assert!(serde_json::from_value::<TargetDescriptor>(object).is_err());
}

#[test]
fn observations_reject_schema_bypasses_and_keep_failure_identity_independent_of_diagnostics() {
    let key = FailureKey::new(
        property(),
        FailureClass::PropertyViolation,
        Some("odd-sum".into()),
    )
    .unwrap();
    let first = CaseOutcome::failure(key.clone(), "sum 3 is odd").unwrap();
    let second = CaseOutcome::failure(key, "sum 5 is odd").unwrap();
    assert_eq!(first.failure_key(), second.failure_key());
    assert_ne!(first, second);
    assert_eq!(
        serde_json::from_str::<CaseOutcome>(&serde_json::to_string(&first).unwrap()).unwrap(),
        first
    );
    assert!(
        FailureKey::new(
            property(),
            FailureClass::Panic,
            Some("panic at 0xabc".into())
        )
        .is_err()
    );
    assert!(CaseOutcome::evaluated(vec![]).is_err());
    assert!(CaseOutcome::evaluated(vec![property(), property()]).is_err());
    assert!(GenerationRejection::new(" ").is_err());
    let descriptor = TargetDescriptor::new(spec()).unwrap();
    descriptor.validate_outcome(&first).unwrap();
    assert!(
        descriptor
            .validate_outcome(
                &CaseOutcome::evaluated(vec![
                    PropertyId::new("surgeist-test::undeclared").unwrap()
                ])
                .unwrap()
            )
            .is_err()
    );
    for json in [
        r#"{"kind":"evaluated","checked_properties":[],"failure_key":null,"detail":null}"#,
        r#"{"kind":"evaluated","checked_properties":["surgeist-test::sum-is-even"],"failure_key":null}"#,
        r#"{"kind":"unsupported","checked_properties":[],"failure_key":null,"detail":"missing","unknown":1}"#,
        r#"{"kind":"unsupported","checked_properties":[],"failure_key":null,"detail":"missing","detail":"again"}"#,
    ] {
        assert!(
            serde_json::from_str::<CaseOutcome>(json).is_err(),
            "accepted {json}"
        );
    }
    assert!(
        serde_json::from_str::<FailureKey>(
            r#"{"property":"surgeist-test::sum-is-even","class":"panic"}"#
        )
        .is_err()
    );
}

struct PairTarget {
    descriptor: TargetDescriptor,
}
impl CaseTarget for PairTarget {
    type Case = [u8; 2];
    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }
    fn decode(&self, input: &[u8]) -> Result<Self::Case> {
        self.descriptor
            .limits()
            .check(InputUsage::new(input.len() as u64, 0, 0, 0, 0)?)?;
        input.try_into().map_err(|_| {
            FuzzError::new(
                FuzzErrorKind::InvalidCase,
                "decode pair",
                "requires exactly two bytes",
            )
        })
    }
    fn encode(&self, case: &Self::Case) -> Result<Vec<u8>> {
        Ok(case.to_vec())
    }
    fn generate(&self, input: &[u8]) -> Result<Generation<Self::Case>> {
        if input.len() != 2 {
            return Ok(Generation::Rejected(GenerationRejection::new(
                "requires two generation bytes",
            )?));
        }
        Ok(Generation::Generated(self.decode(input)?))
    }
    fn evaluate(&self, case: &Self::Case) -> Result<CaseOutcome> {
        if (u16::from(case[0]) + u16::from(case[1])) % 2 != 0 {
            return CaseOutcome::failure(
                FailureKey::new(
                    property(),
                    FailureClass::PropertyViolation,
                    Some("odd-sum".into()),
                )?,
                "sum is odd",
            );
        }
        CaseOutcome::evaluated(vec![property()])
    }
    fn reduction_candidates(
        &self,
        case: &Self::Case,
        emit: &mut dyn FnMut(Self::Case) -> bool,
    ) -> Result<()> {
        for candidate in [[0, case[1]], [case[0], 0], [0, 0]] {
            if !emit(candidate) {
                break;
            }
        }
        Ok(())
    }
}

#[test]
fn one_owner_evaluator_detects_controlled_failure_through_generated_and_replayed_cases() {
    let target = PairTarget {
        descriptor: TargetDescriptor::new(spec()).unwrap(),
    };
    let Generation::Generated(case) = target.generate(&[1, 2]).unwrap() else {
        panic!("valid materialization rejected")
    };
    let generated = target.evaluate(&case).unwrap();
    assert_eq!(generated.kind(), OutcomeKind::Failure);
    assert_eq!(generated.failure_key().unwrap().property(), &property());
    assert_eq!(
        generated.failure_key().unwrap().class(),
        FailureClass::PropertyViolation
    );
    let bytes = target.encode(&case).unwrap();
    assert_eq!(bytes, vec![1, 2]);
    let replayed = target.evaluate(&target.decode(&bytes).unwrap()).unwrap();
    assert_eq!(replayed, generated);
    target.descriptor().validate_outcome(&replayed).unwrap();
    assert_eq!(
        target.evaluate(&[1, 1]).unwrap().kind(),
        OutcomeKind::Evaluated
    );
    assert!(target.decode(&[1]).is_err());
    assert!(target.decode(&[1, 2, 3]).is_err());
    assert!(matches!(
        target.generate(&[]).unwrap(),
        Generation::Rejected(_)
    ));
    let mut candidates = vec![];
    target
        .reduction_candidates(&case, &mut |candidate| {
            candidates.push(candidate);
            false
        })
        .unwrap();
    assert_eq!(candidates, vec![[0, 2]]);
}

#[test]
fn raw_byte_targets_preserve_binary_input_and_use_the_same_owner_evaluator() {
    let mut raw = spec();
    raw.encoding = InputEncoding::RawBytes;
    raw.format = CaseFormat::new("surgeist-test::raw-bytes", 1).unwrap();
    let target = PairTarget {
        descriptor: TargetDescriptor::new(raw).unwrap(),
    };
    assert_eq!(target.descriptor().encoding(), InputEncoding::RawBytes);
    let case = target.decode(&[0, 255]).unwrap();
    assert_eq!(target.encode(&case).unwrap(), vec![0, 255]);
    assert_eq!(target.evaluate(&case).unwrap().kind(), OutcomeKind::Failure);
    assert_eq!(
        target.evaluate(&[255, 255]).unwrap().kind(),
        OutcomeKind::Evaluated
    );
}

#[test]
fn coverage_counts_generation_discards_and_unchecked_rejections_without_vacuous_success() {
    let descriptor = TargetDescriptor::new(spec()).unwrap();
    let mut counts = ExecutionCounts::default();
    assert_eq!(
        descriptor.assess_generated(counts),
        CoverageAssessment::Insufficient
    );
    assert_eq!(counts.assess_regressions(), CoverageAssessment::Empty);
    counts.record_generation_rejection().unwrap();
    counts
        .record_outcome(
            &CaseOutcome::expected_rejection(vec![], "product rejected invalid input").unwrap(),
        )
        .unwrap();
    counts
        .record_outcome(&CaseOutcome::unsupported("missing environment").unwrap())
        .unwrap();
    assert_eq!(counts.evaluated(), 0);
    counts
        .record_outcome(&CaseOutcome::evaluated(vec![property()]).unwrap())
        .unwrap();
    assert_eq!(
        descriptor.assess_generated(counts),
        CoverageAssessment::Insufficient
    );
    counts
        .record_outcome(
            &CaseOutcome::expected_rejection(vec![property()], "rejection atomicity verified")
                .unwrap(),
        )
        .unwrap();
    assert_eq!(counts.evaluated(), 2);
    assert_eq!(
        descriptor.assess_generated(counts),
        CoverageAssessment::Sufficient
    );
    counts.record_generation_rejection().unwrap();
    assert_eq!(
        descriptor.assess_generated(counts),
        CoverageAssessment::Insufficient
    );
    assert_eq!(counts.generation_rejected(), 2);
    let mut overflow = ExecutionCounts::new(u64::MAX, 0);
    assert_eq!(
        overflow
            .record_outcome(&CaseOutcome::evaluated(vec![property()]).unwrap())
            .unwrap_err()
            .kind(),
        FuzzErrorKind::Harness
    );
    assert_eq!(overflow.evaluated(), u64::MAX);
}

#[test]
fn semantic_errors_preserve_safe_sources() {
    let error = FuzzError::with_source(
        FuzzErrorKind::Io,
        "read seed",
        "read failed",
        std::io::Error::other("disk unavailable"),
    );
    assert_eq!(error.kind(), FuzzErrorKind::Io);
    assert_eq!(error.source().unwrap().to_string(), "disk unavailable");
    assert_eq!(error.operation(), "read seed");
}

fn assert_closed_schema<T>(value: T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let encoded = serde_json::to_string(&value).unwrap();
    assert_eq!(serde_json::from_str::<T>(&encoded).unwrap(), value);
    let object = serde_json::to_value(&value).unwrap();
    let fields = object.as_object().unwrap();
    for key in fields.keys() {
        let mut missing = object.clone();
        missing.as_object_mut().unwrap().remove(key);
        assert!(
            serde_json::from_value::<T>(missing).is_err(),
            "accepted missing {key}"
        );
        let duplicate = format!(
            "{{{},{}:{}}}",
            &encoded[1..encoded.len() - 1],
            serde_json::to_string(key).unwrap(),
            fields[key]
        );
        assert!(
            serde_json::from_str::<T>(&duplicate).is_err(),
            "accepted duplicate {key}"
        );
    }
    let mut unknown = object;
    unknown["unexpected-field"] = true.into();
    assert!(serde_json::from_value::<T>(unknown).is_err());
}

#[test]
fn every_persisted_contract_rejects_missing_duplicate_and_unknown_fields() {
    assert_closed_schema(CaseFormat::new("surgeist-test::pair", 1).unwrap());
    assert_closed_schema(GeneratorIdentity::new("surgeist-test::pair", 1).unwrap());
    assert_closed_schema(BuildRequirements::new(vec![], vec![Capability::Replay]).unwrap());
    assert_closed_schema(InputLimits::new(2, 0, 0, 0, 0).unwrap());
    assert_closed_schema(InputUsage::new(1, 0, 0, 0, 0).unwrap());
    assert_closed_schema(TargetDescriptor::new(spec()).unwrap());
    assert_closed_schema(FailureKey::new(property(), FailureClass::Panic, None).unwrap());
    assert_closed_schema(CaseOutcome::evaluated(vec![property()]).unwrap());
    assert_closed_schema(GenerationRejection::new("bounded input exhausted").unwrap());
    assert_closed_schema(ExecutionCounts::default());
}
