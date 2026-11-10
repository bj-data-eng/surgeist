// Intended as a child of the source-module owner. The lifecycle test harness
// exposes expected_lifecycle_debits only under cfg(test).
use super::*;
use crate::{
    ContextAccess,
    execution::{ExecutionOptions, ProfileMode},
    job::{Job, JobAcceptance, JobExecutor, JobPublicationError},
    module::{ImportAttribute, ModuleRepr, metadata::Entry},
};
use std::{convert::Infallible, mem::size_of};
use surgeist_js_ast::scope::{PreparedScopes, ScopeAnalysisError};

#[derive(Default)]
struct RecordingExecutor(RefCell<Vec<Job>>);
impl JobExecutor for RecordingExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        self.0.borrow_mut().push(job);
        Ok(JobAcceptance::Accepted)
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("constructor fixtures must not dispatch jobs")
    }
}
fn context(executor: Rc<RecordingExecutor>) -> Context {
    let builder = Context::builder().job_executor(executor);
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    builder.build().unwrap()
}
fn next_scope_identity(context: &Context) -> u32 {
    // A real private proposal exposes the next canonical identity. Dropping
    // that proposal publishes no scope or binding and spends no identity.
    let mut scopes = PreparedScopes::new(context.realm().scope());
    scopes
        .with_access::<_, Infallible, ScopeAnalysisError<Infallible>>(None, |access| {
            let root = access.root_handle()?;
            let (_, names) = access.create_module_scope(&root, &[js_string!("identityProbe")])?;
            Ok(names[0].unique_scope_id())
        })
        .unwrap()
}

fn header<T>(debits: &mut Vec<u64>) {
    debits.extend([1, u64::try_from(size_of::<T>()).unwrap()]);
}
fn empty_request(debits: &mut Vec<u64>) {
    // No attributes, sorting, or nonempty string payloads. The retained
    // attribute vector header and request record still receive credits.
    header::<Vec<ImportAttribute>>(debits);
    debits.push(1);
}
fn first_request_index(debits: &mut Vec<u64>) {
    // Attempt Record; empty-specifier hash, attribute-count Probe; no initial
    // search slots; eight slots; four-entry geometric append capacity; final
    // vacant Probe; mapping Write and entry Write. All relocated lengths are 0.
    debits.extend([1, 1, u64::try_from(8 * size_of::<Option<usize>>()).unwrap()]);
    debits.extend([1; 8]);
    debits.extend([
        u64::try_from(4 * size_of::<Entry<ModuleRequest, ()>>()).unwrap(),
        1,
        1,
        1,
    ]);
}
fn append<T>(debits: &mut Vec<u64>) {
    // First unknown-stream append reserves four logical entries. Its empty
    // buffer has no relocation debit; record and write each cost one.
    debits.extend([u64::try_from(4 * size_of::<T>()).unwrap(), 1, 1]);
}
#[derive(Clone, Copy)]
enum Fixture {
    Empty,
    Import,
    Star,
    Indirect,
}
impl Fixture {
    fn source(self) -> &'static str {
        match self {
            Self::Empty => "",
            Self::Import => "import '';",
            Self::Star => "export * from '';",
            Self::Indirect => "export { '' as '' } from '';",
        }
    }
    fn requested(self) -> usize {
        usize::from(!matches!(self, Self::Empty))
    }
    fn debits(self) -> Vec<u64> {
        let mut debits = Vec::new();
        // No source path: neither before-parser nor after-parser path copies.
        header::<surgeist_js_ast::SourceText>(&mut debits);
        header::<SourceTextModule>(&mut debits);
        // Request pass: list, optional item, then its emitted request/index.
        debits.push(1);
        if self.requested() != 0 {
            debits.push(1);
            empty_request(&mut debits);
            first_request_index(&mut debits);
        }
        // Import pass has no emitted bindings in these fixtures.
        debits.push(1);
        if self.requested() != 0 {
            debits.push(1);
        }
        // Export pass: list and optional item, then explicit-specifier Visit
        // where present and the independent export request/entry construction.
        debits.push(1);
        if self.requested() != 0 {
            debits.push(1);
        }
        match self {
            Self::Star => {
                empty_request(&mut debits);
                append::<ModuleRequest>(&mut debits);
            }
            Self::Indirect => {
                debits.push(1);
                empty_request(&mut debits);
                append::<IndirectExportEntry>(&mut debits);
            }
            Self::Empty | Self::Import => {}
        }
        // HasTLA delegates to NameWork, excluded from these metadata credits.
        debits.extend(
            crate::module::lifecycle::metadata_cost_tests::expected_lifecycle_debits(
                self.requested(),
            ),
        );
        header::<ModuleRepr>(&mut debits);
        debits.push(1); // Publication precedes the actual module allocation.
        debits
    }
    fn assert_module(self, module: &Module) {
        assert!(module.path().is_none());
        let source = module.kind().as_source_text().unwrap();
        assert!(!source.code.has_tla);
        assert_eq!(source.code.requested_modules.len(), self.requested());
        for request in &source.code.requested_modules {
            assert!(request.specifier().is_empty());
            assert!(request.attributes().is_empty());
        }
        assert!(source.code.import_entries.is_empty());
        assert!(source.code.local_export_entries.is_empty());
        assert_eq!(
            source.code.star_export_entries.len(),
            usize::from(matches!(self, Self::Star))
        );
        assert_eq!(
            source.code.indirect_export_entries.len(),
            usize::from(matches!(self, Self::Indirect))
        );
        if let Some(entry) = source.code.indirect_export_entries.first() {
            let ImportedName::Name(name) = &entry.import_name else {
                panic!("authored named re-export")
            };
            assert!(name.is_empty());
            assert!(entry.export_name.is_empty());
            assert!(entry.module_request.specifier().is_empty());
        }
        assert!(matches!(
            *source.status.borrow(),
            ModuleStatus::Unlinked { .. }
        ));
    }
}

fn exact_constructor_and_final_publication_refusal(fixture: Fixture) {
    let total: u64 = fixture.debits().into_iter().sum();
    let executor = Rc::new(RecordingExecutor::default());
    let mut context = context(executor.clone());
    let next_scope = next_scope_identity(&context);
    let keys = context
        .global_object()
        .unwrap()
        .own_property_keys(context.access())
        .unwrap();
    let mut direct = None;
    let outcome = context
        .with_execution(
            ExecutionOptions::new(100_000).with_module_metadata_work_limit(total - 1),
            |access| {
                let result =
                    Module::parse(crate::Source::from_bytes(fixture.source()), None, access);
                direct = result.as_ref().err().cloned();
                result
            },
        )
        .unwrap();
    let error = direct.expect("the final Publication credit must refuse before returning a Module");
    assert_eq!(
        error.as_engine(),
        Some(&crate::error::EngineError::RuntimeLimit(
            crate::error::RuntimeLimitError::ModuleMetadataWork { limit: total - 1 },
        ))
    );
    assert_eq!(outcome.result.unwrap_err(), error);
    assert_eq!(outcome.report.module_metadata_work().unwrap(), total - 1);
    assert_eq!(outcome.report.instructions().unwrap(), 0);
    assert_eq!(next_scope_identity(&context), next_scope);
    assert!(!outcome.report.effects_started());
    assert!(executor.0.borrow().is_empty());
    assert_eq!(
        context
            .global_object()
            .unwrap()
            .own_property_keys(context.access())
            .unwrap(),
        keys
    );

    // The same context gets a fresh budget. Refusal did not publish executable
    // work or bindings; an independently parsed replacement can be returned.
    let accepted = context
        .with_execution(
            ExecutionOptions::new(100_000)
                .with_module_metadata_work_limit(total)
                .with_profile(ProfileMode::Summary { timing: false }),
            |access| Module::parse(crate::Source::from_bytes(fixture.source()), None, access),
        )
        .unwrap();
    fixture.assert_module(&accepted.result.unwrap());
    assert_eq!(accepted.report.module_metadata_work().unwrap(), total);
    assert_eq!(accepted.report.instructions().unwrap(), 0);
    assert_eq!(next_scope_identity(&context), next_scope);
    assert!(!accepted.report.effects_started());
    assert!(executor.0.borrow().is_empty());
    assert_eq!(
        context
            .global_object()
            .unwrap()
            .own_property_keys(context.access())
            .unwrap(),
        keys
    );
}

#[test]
fn empty_source_constructor_has_exact_headers_lifecycle_and_publication_cost() {
    exact_constructor_and_final_publication_refusal(Fixture::Empty);
}
#[test]
fn empty_specifier_import_constructor_has_exact_request_index_and_parent_slot_cost() {
    exact_constructor_and_final_publication_refusal(Fixture::Import);
}
#[test]
fn empty_specifier_star_export_constructor_adds_its_own_ordered_export_record() {
    exact_constructor_and_final_publication_refusal(Fixture::Star);
}
#[test]
fn empty_named_reexport_constructor_prices_specifier_visit_and_indirect_entry() {
    exact_constructor_and_final_publication_refusal(Fixture::Indirect);
}
