//! Owner-local authoring example: strict bounded codecs and one evaluator used
//! by workers, property testing, coverage and replay. This is a toy fixture.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use surgeist_fixture as subject;
use surgeist_fuzz::{
    generation::{Budget, BudgetCharge, FixtureId, OrderedForest, SequenceBuilder},
    registry::FuzzManifest,
    *,
};

fn error(detail: &'static str) -> FuzzError {
    FuzzError::new(FuzzErrorKind::InvalidCase, "fixture codec", detail)
}
fn descriptor(name: &str) -> TargetDescriptor {
    let manifest: FuzzManifest =
        toml::from_str(include_str!("../fuzz.toml")).expect("checked fixture descriptor");
    manifest
        .targets
        .into_iter()
        .find(|target| target.id().as_str() == name)
        .expect("declared fixture target")
}

pub struct BytesTarget {
    descriptor: TargetDescriptor,
}
impl Default for BytesTarget {
    fn default() -> Self {
        Self {
            descriptor: descriptor("surgeist-fixture::bytes"),
        }
    }
}
impl CaseTarget for BytesTarget {
    type Case = Vec<u8>;
    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }
    fn decode(&self, input: &[u8]) -> Result<Self::Case> {
        surgeist_fuzz::generation::Entropy::new(input)
            .bytes(input.len(), &mut Budget::new(self.descriptor.limits()))
    }
    fn encode(&self, case: &Self::Case) -> Result<Vec<u8>> {
        self.decode(case)
    }
    fn generate(&self, input: &[u8]) -> Result<Generation<Self::Case>> {
        self.decode(input).map(Generation::Generated)
    }
    fn evaluate(&self, case: &Self::Case) -> Result<CaseOutcome> {
        if case.starts_with(b"PANIC!") {
            panic!("controlled fixture panic");
        }
        if case.starts_with(b"HANG!") {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
        let observed = subject::reported_bytes(case);
        if observed != case.len() {
            let discriminator = if observed == case.len() + 1 {
                "length-one"
            } else {
                "length-two"
            };
            return CaseOutcome::failure(
                FailureKey::new(
                    self.descriptor.properties()[0].clone(),
                    FailureClass::PropertyViolation,
                    Some(discriminator.into()),
                )?,
                "controlled byte-count violation",
            );
        }
        CaseOutcome::evaluated(self.descriptor.properties().to_vec())
    }
    fn reduction_candidates(
        &self,
        case: &Self::Case,
        emit: &mut dyn FnMut(Self::Case) -> bool,
    ) -> Result<()> {
        // These fixtures are byte subsequences, not guessed product repairs.
        for size in [0, 4, case.len() / 2] {
            if size < case.len() && !emit(case[..size].to_vec()) {
                return Ok(());
            }
        }
        for marker in [b"BUG!".as_slice(), b"ALT!".as_slice()] {
            if case.windows(marker.len()).any(|part| part == marker) && !emit(marker.to_vec()) {
                return Ok(());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Operation {
    #[default]
    Noop,
    Create(u8),
    Use(u8),
    RejectUse(u8),
}

// Fixed arrays put a hard allocation boundary ahead of deserialization. Private
// raw fields are checked through StructuredTarget::decode before use.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCase {
    node_count: u8,
    parents: [Option<u8>; 8],
    text_count: u8,
    text: [u16; 8],
    operation_count: u8,
    operations: [Operation; 8],
}

#[derive(Clone, Debug)]
pub struct StructuredCase {
    raw: RawCase,
    forest: OrderedForest,
}
pub struct StructuredTarget {
    descriptor: TargetDescriptor,
}
impl Default for StructuredTarget {
    fn default() -> Self {
        Self {
            descriptor: descriptor("surgeist-fixture::structured"),
        }
    }
}

impl StructuredTarget {
    fn validate(&self, raw: RawCase) -> Result<StructuredCase> {
        let nodes = usize::from(raw.node_count);
        let text = usize::from(raw.text_count);
        let operations = usize::from(raw.operation_count);
        if nodes > 8
            || text > 8
            || operations > 8
            || raw.parents[nodes..].iter().any(Option::is_some)
            || raw.text[text..].iter().any(|unit| *unit != 0)
            || raw.operations[operations..]
                .iter()
                .any(|op| *op != Operation::Noop)
        {
            return Err(error("invalid count or noncanonical unused slots"));
        }
        let mut parents = [None; 8];
        for (index, parent) in raw.parents[..nodes].iter().enumerate() {
            parents[index] = parent.map(|parent| FixtureId::new(usize::from(parent)));
        }
        let mut budget = Budget::new(self.descriptor.limits());
        let forest = OrderedForest::from_parents(&parents[..nodes], &mut budget)?;
        budget.credit(BudgetCharge {
            bytes: text as u64 * 2,
            text_units: text as u64,
            ..BudgetCharge::default()
        })?;
        let mut sequence = SequenceBuilder::new();
        let mut created = [false; 8];
        for operation in raw.operations[..operations].iter().copied() {
            let admitted = match operation {
                Operation::RejectUse(_) => {
                    sequence
                        .append_invalid(&mut budget, BudgetCharge::default(), || Ok(operation))?;
                    true
                }
                _ => sequence.append_valid(
                    &mut budget,
                    BudgetCharge::default(),
                    || Ok(operation),
                    |_, operation| match operation {
                        Operation::Noop => true,
                        Operation::Create(id) => {
                            usize::from(*id) < nodes && !created[usize::from(*id)]
                        }
                        Operation::Use(id) => {
                            created.get(usize::from(*id)).copied().unwrap_or(false)
                        }
                        Operation::RejectUse(_) => false,
                    },
                )?,
            };
            if !admitted {
                return Err(error("operation precondition rejected without repair"));
            }
            if let Operation::Create(id) = operation {
                created[usize::from(id)] = true;
            }
        }
        Ok(StructuredCase { raw, forest })
    }
}

impl CaseTarget for StructuredTarget {
    type Case = StructuredCase;
    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }
    fn decode(&self, input: &[u8]) -> Result<Self::Case> {
        if input.len() as u64 > self.descriptor.limits().bytes() {
            return Err(error("input byte limit exceeded"));
        }
        let raw = serde_json::from_slice(input).map_err(|cause| {
            FuzzError::with_source(
                FuzzErrorKind::Codec,
                "fixture codec",
                "strict structured JSON",
                cause,
            )
        })?;
        self.validate(raw)
    }
    fn encode(&self, case: &Self::Case) -> Result<Vec<u8>> {
        let output = serde_json::to_vec(&case.raw).map_err(|cause| {
            FuzzError::with_source(
                FuzzErrorKind::Codec,
                "fixture codec",
                "encode structured case",
                cause,
            )
        })?;
        if output.len() as u64 > self.descriptor.limits().bytes() {
            return Err(error("canonical byte limit exceeded"));
        }
        Ok(output)
    }
    fn generate(&self, input: &[u8]) -> Result<Generation<Self::Case>> {
        if input.len() as u64 > self.descriptor.limits().bytes() {
            return Err(error("entropy byte limit exceeded"));
        }
        let nodes = input.first().copied().unwrap_or(0) % 9;
        let text_count = (input.len().saturating_sub(1) / 2).min(8) as u8;
        let mut raw = RawCase {
            node_count: nodes,
            parents: [None; 8],
            text_count,
            text: [0; 8],
            operation_count: u8::from(nodes != 0) * 3,
            operations: [Operation::Noop; 8],
        };
        for index in 1..usize::from(nodes) {
            raw.parents[index] = Some(index as u8 - 1);
        }
        for (slot, pair) in raw
            .text
            .iter_mut()
            .zip(input.get(1..).unwrap_or(&[]).chunks_exact(2).take(8))
        {
            *slot = u16::from_le_bytes([pair[0], pair[1]]);
        }
        if nodes != 0 {
            raw.operations[..3].copy_from_slice(&[
                Operation::Create(0),
                Operation::Use(0),
                Operation::RejectUse(255),
            ]);
        }
        self.validate(raw).map(Generation::Generated)
    }
    fn evaluate(&self, case: &Self::Case) -> Result<CaseOutcome> {
        let units = &case.raw.text[..usize::from(case.raw.text_count)];
        if subject::reported_units(units) != units.len() {
            return CaseOutcome::failure(
                FailureKey::new(
                    self.descriptor.properties()[0].clone(),
                    FailureClass::PropertyViolation,
                    Some("surrogate-count".into()),
                )?,
                "controlled code-unit count violation",
            );
        }
        let mut created = [false; 8];
        for operation in &case.raw.operations[..usize::from(case.raw.operation_count)] {
            match *operation {
                Operation::Noop => {}
                Operation::Create(id) => created[usize::from(id)] = true,
                Operation::Use(id) => {
                    if !subject::use_node(&mut created, id) {
                        return Err(error("admitted operation rejected"));
                    }
                }
                Operation::RejectUse(id) => {
                    let before = created;
                    if subject::use_node(&mut created, id) || before != created {
                        return CaseOutcome::failure(
                            FailureKey::new(
                                self.descriptor.properties()[1].clone(),
                                FailureClass::PropertyViolation,
                                Some("invalid-reference".into()),
                            )?,
                            "invalid operation must reject atomically",
                        );
                    }
                }
            }
        }
        assert_eq!(
            case.forest.preorder().count(),
            usize::from(case.raw.node_count)
        );
        CaseOutcome::evaluated(self.descriptor.properties().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surgeist_fuzz::engine::{GeneratedEvaluation, evaluate_generated, replay};
    #[test]
    fn controlled_failures_retain_distinct_keys_and_replay_equivalently() {
        let target = BytesTarget::default();
        let bug = target.evaluate(&target.decode(b"BUG!").unwrap()).unwrap();
        let alternative = target.evaluate(&target.decode(b"ALT!").unwrap()).unwrap();
        assert_ne!(bug.failure_key(), alternative.failure_key());
        for input in [
            b"ordinary".as_slice(),
            b"BUG!".as_slice(),
            b"ALT!".as_slice(),
        ] {
            let generated = evaluate_generated(&target, input, &mut |_| Ok(())).unwrap();
            let GeneratedEvaluation::Evaluated(generated) = generated else {
                panic!("raw generator rejected");
            };
            assert_eq!(
                generated.outcome,
                replay(&target, &generated.canonical, &mut |_| Ok(()))
                    .unwrap()
                    .outcome
            );
        }
    }
    #[test]
    fn structured_generation_materializes_lossless_units_and_checked_operations() {
        let target = StructuredTarget::default();
        for input in [&[3, 0x61, 0][..], &[8, 0, 0xd8][..], &[][..]] {
            let Generation::Generated(case) = target.generate(input).unwrap() else {
                panic!("finite generator rejected");
            };
            let bytes = target.encode(&case).unwrap();
            assert_eq!(
                target.evaluate(&case).unwrap(),
                target.evaluate(&target.decode(&bytes).unwrap()).unwrap()
            );
        }
        let Generation::Generated(case) = target.generate(&[8, 0, 0xd8]).unwrap() else {
            panic!("finite generator rejected");
        };
        assert_eq!(case.raw.text[0], 0xd800);
        assert_eq!(case.forest.depth(FixtureId::new(7)).unwrap(), 8);
        assert!(target.evaluate(&case).unwrap().failure_key().is_some());
        let mut raw = case.raw.clone();
        raw.parents[0] = Some(0);
        assert!(target.validate(raw).is_err());
        let mut raw = case.raw.clone();
        raw.operations[1] = Operation::Use(7);
        assert!(target.validate(raw).is_err());
        let mut raw = case.raw.clone();
        raw.text[7] = 1;
        assert!(target.validate(raw).is_err());
        let mut json = serde_json::to_value(&case.raw).unwrap();
        json["unknown"] = serde_json::json!(true);
        assert!(target.decode(&serde_json::to_vec(&json).unwrap()).is_err());
    }
    #[cfg(feature = "property")]
    #[test]
    fn real_property_engine_detects_the_controlled_subject_failure() {
        use surgeist_fuzz::engine::{PropertyConfig, run_property};
        let report = run_property(
            &BytesTarget::default(),
            &proptest::strategy::Just(b"BUG!".to_vec()),
            PropertyConfig {
                cases: 1,
                max_rejections: 0,
                max_shrink_iterations: 0,
                seed: 1,
            },
            &mut |_| Ok(()),
        )
        .unwrap();
        assert!(report.failure.is_some());
        assert!(report.counts.evaluated() >= 1);
    }

    #[cfg(feature = "coverage")]
    #[test]
    fn coverage_and_replay_use_the_same_materialized_owner_case() {
        let target = StructuredTarget::default();
        let mut driver = surgeist_fuzz::engine::CoverageDriver::new(&target, None).unwrap();
        let GeneratedEvaluation::Evaluated(evaluation) =
            driver.test_one(&[8, 0, 0xd8], &mut |_| Ok(())).unwrap()
        else {
            panic!("finite generator rejected");
        };
        assert_eq!(
            evaluation.outcome,
            replay(&target, &evaluation.canonical, &mut |_| Ok(()))
                .unwrap()
                .outcome
        );
        assert!(evaluation.outcome.failure_key().is_some());
        assert!(driver.counts().evaluated() >= 1);
    }
}
