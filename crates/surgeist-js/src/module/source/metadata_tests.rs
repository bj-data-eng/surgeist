// Intended as a child module of the source-module owner.
use super::*;
use crate::execution::{ExecutionOptions, ExecutionReport};

fn parsed(text: &str) -> (Context, Module, ExecutionReport) {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    let outcome = context
        .with_execution(ExecutionOptions::new(100_000), |access| {
            Module::parse(crate::Source::from_bytes(text), None, access)
        })
        .unwrap();
    assert!(!outcome.report.effects_started());
    (context, outcome.result.unwrap(), outcome.report)
}

fn code(module: &Module) -> &ModuleCode {
    &module.kind().as_source_text().unwrap().code
}

fn request(actual: &ModuleRequest, specifier: &str, attributes: &[(&str, &str)]) {
    assert_eq!(actual.specifier(), &JsString::from(specifier));
    let actual: Vec<_> = actual
        .attributes()
        .iter()
        .map(|attribute| (attribute.key().clone(), attribute.value().clone()))
        .collect();
    let expected: Vec<_> = attributes
        .iter()
        .map(|(key, value)| (JsString::from(*key), JsString::from(*value)))
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn full_request_identity_deduplicates_canonical_attributes_and_preserves_first_occurrence() {
    let (_context, module, report) = parsed(
        "import 'same' with { z: 'last', a: 'first' };
         import 'other';
         export * from 'same' with { a: 'first', z: 'last' };
         import 'same' with { a: 'changed', z: 'last' };",
    );
    let code = code(&module);
    let requests: Vec<_> = code.requested_modules.iter().collect();
    assert_eq!(requests.len(), 3);
    request(requests[0], "same", &[("a", "first"), ("z", "last")]);
    request(requests[1], "other", &[]);
    request(requests[2], "same", &[("a", "changed"), ("z", "last")]);
    assert_eq!(code.star_export_entries.len(), 1);
    request(
        &code.star_export_entries[0],
        "same",
        &[("a", "first"), ("z", "last")],
    );
    assert!(code.import_entries.is_empty());
    assert!(report.module_metadata_work().unwrap() > 0);
}

#[test]
fn imported_local_forward_exports_and_destructuring_retain_their_exact_entry_roles() {
    let (_context, module, _) = parsed(
        "export { local as forwarded };
         import { remote as local } from 'dep';
         export const { a: first, b: [second] } = sourceObject;
         export * as namespace from 'star';
         export * from 'all';",
    );
    let code = code(&module);
    assert_eq!(code.import_entries.len(), 1);
    let imported = &code.import_entries[0];
    request(&imported.module_request, "dep", &[]);
    assert_eq!(imported.local_name, js_string!("local"));
    assert!(
        matches!(&imported.import_name, ImportedName::Name(name) if name == &js_string!("remote"))
    );
    let local: Vec<_> = code
        .local_export_entries
        .iter()
        .map(|entry| (entry.local_name.clone(), entry.export_name.clone()))
        .collect();
    assert_eq!(
        local,
        vec![
            (js_string!("first"), js_string!("first")),
            (js_string!("second"), js_string!("second"))
        ]
    );
    assert_eq!(code.indirect_export_entries.len(), 2);
    let forwarded = &code.indirect_export_entries[0];
    request(&forwarded.module_request, "dep", &[]);
    assert_eq!(forwarded.export_name, js_string!("forwarded"));
    assert!(
        matches!(&forwarded.import_name, ImportedName::Name(name) if name == &js_string!("remote"))
    );
    let namespace = &code.indirect_export_entries[1];
    request(&namespace.module_request, "star", &[]);
    assert_eq!(namespace.export_name, js_string!("namespace"));
    assert!(matches!(namespace.import_name, ImportedName::Namespace));
    assert_eq!(code.star_export_entries.len(), 1);
    request(&code.star_export_entries[0], "all", &[]);
    assert!(!code.has_tla);
}

#[test]
fn request_spelling_preserves_long_shared_prefixes_exact_utf16_and_unpaired_surrogates() {
    let prefix = "x".repeat(512);
    let text = format!(
        "import '{prefix}🙂a'; import '{prefix}🙂b'; import '\\uD800x'; import 'é'; import '\\u00E9';"
    );
    let (_context, module, _) = parsed(&text);
    let requests: Vec<_> = code(&module).requested_modules.iter().collect();
    assert_eq!(requests.len(), 4);
    assert_eq!(
        requests[0].specifier(),
        &JsString::from(format!("{prefix}🙂a"))
    );
    assert_eq!(
        requests[1].specifier(),
        &JsString::from(format!("{prefix}🙂b"))
    );
    assert_eq!(
        requests[2].specifier(),
        &JsString::from(&[0xD800u16, 0x0078][..])
    );
    assert_eq!(requests[3].specifier(), &js_string!("é"));
}

#[test]
fn has_tla_prunes_nested_function_bodies_and_keeps_top_level_await_even_when_unreachable() {
    for (text, expected) in [
        ("async function hidden() { await 0; }", false),
        ("const hidden = async () => { await 0; };", false),
        ("class Holder { async method() { await 0; } }", false),
        ("if (false) { await 0; }", true),
        ("await 0; async function hidden() { await 1; }", true),
    ] {
        let (_context, module, _) = parsed(text);
        assert_eq!(code(&module).has_tla, expected, "{text}");
        assert_eq!(code(&module).requested_modules.len(), 0);
        assert!(code(&module).import_entries.is_empty());
        assert!(code(&module).local_export_entries.is_empty());
        assert!(code(&module).indirect_export_entries.is_empty());
    }
}

#[test]
fn duplicate_request_attempts_cost_additional_metadata_work_without_new_owned_requests() {
    let (_single_context, single, single_report) = parsed("import 'same';");
    let (_duplicate_context, duplicate, duplicate_report) = parsed("import 'same'; import 'same';");
    assert_eq!(code(&single).requested_modules.len(), 1);
    assert_eq!(code(&duplicate).requested_modules.len(), 1);
    request(
        code(&single).requested_modules.iter().next().unwrap(),
        "same",
        &[],
    );
    request(
        code(&duplicate).requested_modules.iter().next().unwrap(),
        "same",
        &[],
    );
    // The unique output and lifecycle-slot count are unchanged. The second raw
    // request still requires at least its positive Visit credit; no measured
    // total from either run is used as a later refusal threshold.
    assert!(
        duplicate_report.module_metadata_work().unwrap()
            > single_report.module_metadata_work().unwrap()
    );
}

use surgeist_js_ast::{
    ModuleEntryRef, ModuleEntryWork, ModuleItemList, ModuleMetadataSink, operations::NameWork,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Refusal {
    Visit,
    NameRecord,
}
struct Sink<'ast> {
    visits: u64,
    visit_cap: u64,
    name_records: u64,
    name_record_cap: u64,
    names: Vec<NameWork>,
    entries: Vec<ModuleEntryRef<'ast>>,
}
impl<'ast> Sink<'ast> {
    fn new(visit_cap: u64, name_record_cap: u64) -> Self {
        Self {
            visits: 0,
            visit_cap,
            name_records: 0,
            name_record_cap,
            names: Vec::new(),
            entries: Vec::new(),
        }
    }
}
impl<'ast> ModuleMetadataSink<'ast> for Sink<'ast> {
    type Error = Refusal;
    fn admit(&mut self, work: ModuleEntryWork) -> Result<(), Refusal> {
        match work {
            ModuleEntryWork::Visit => {
                if self.visits == self.visit_cap {
                    return Err(Refusal::Visit);
                }
                self.visits += 1;
            }
            ModuleEntryWork::Name(work) => {
                if work == NameWork::Record {
                    if self.name_records == self.name_record_cap {
                        return Err(Refusal::NameRecord);
                    }
                    self.name_records += 1;
                }
                self.names.push(work);
            }
        }
        Ok(())
    }
    fn emit(&mut self, entry: ModuleEntryRef<'ast>) -> Result<(), Refusal> {
        self.entries.push(entry);
        Ok(())
    }
}

#[test]
fn empty_metadata_passes_each_admit_exactly_one_list_visit() {
    let items = ModuleItemList::default();
    for cap in [0, 1] {
        for pass in 0..3 {
            let mut sink = Sink::new(cap, u64::MAX);
            let result = match pass {
                0 => items.try_emit_requests(&mut sink),
                1 => items.try_emit_import_entries(&mut sink),
                _ => items.try_emit_export_entries(&mut sink),
            };
            assert_eq!(
                result,
                if cap == 0 {
                    Err(Refusal::Visit)
                } else {
                    Ok(())
                }
            );
            assert_eq!(sink.visits, cap);
            assert!(sink.names.is_empty());
            assert!(sink.entries.is_empty());
        }
    }
}

#[test]
fn named_import_emission_has_list_item_and_two_specifier_visits_with_exact_refusal_prefixes() {
    let (_context, module, _) = parsed("import { a as first, b as second } from 'dep';");
    let source_module = module.kind().as_source_text().unwrap();
    let status = source_module.status.borrow();
    let ModuleStatus::Unlinked { source } = &*status else {
        unreachable!()
    };
    for cap in 0..=4 {
        let mut sink = Sink::new(cap, u64::MAX);
        let result = source.ast.items().try_emit_import_entries(&mut sink);
        assert_eq!(result, if cap < 4 { Err(Refusal::Visit) } else { Ok(()) });
        assert_eq!(sink.visits, cap);
        assert!(sink.names.is_empty());
        let expected = if cap < 3 {
            0
        } else if cap == 3 {
            1
        } else {
            2
        };
        assert_eq!(sink.entries.len(), expected);
        for (entry, expected) in sink.entries.iter().zip(["first", "second"]) {
            let ModuleEntryRef::Import { local_name, .. } = entry else {
                unreachable!()
            };
            assert_eq!(
                spelling(local_name.sym(), &source.interner),
                JsString::from(expected)
            );
        }
    }
}

#[test]
fn exported_bound_names_keep_name_records_outside_metadata_visits_and_preserve_first_refusal() {
    let (_context, module, _) = parsed("export const { a: first, b: [second] } = sourceObject;");
    let source_module = module.kind().as_source_text().unwrap();
    let status = source_module.status.borrow();
    let ModuleStatus::Unlinked { source } = &*status else {
        unreachable!()
    };
    let mut complete = Sink::new(2, 2);
    source
        .ast
        .items()
        .try_emit_export_entries(&mut complete)
        .unwrap();
    assert_eq!(
        complete.visits, 2,
        "only the list and its one export item are metadata visits"
    );
    assert_eq!(
        complete.name_records, 2,
        "the two binding identifiers belong to the existing NameWork stream"
    );
    assert_eq!(complete.entries.len(), 2);
    assert!(complete.names.contains(&NameWork::Visit));
    let mut refused_name = Sink::new(2, 1);
    assert_eq!(
        source
            .ast
            .items()
            .try_emit_export_entries(&mut refused_name),
        Err(Refusal::NameRecord)
    );
    assert_eq!(refused_name.visits, 2);
    assert_eq!(refused_name.name_records, 1);
    assert!(refused_name.entries.len() <= 1);
    assert!(refused_name.entries.iter().all(|entry| matches!(entry, ModuleEntryRef::LocalExport { local_name, export_name } if spelling(*local_name, &source.interner) == js_string!("first") && spelling(*export_name, &source.interner) == js_string!("first"))));
    let mut refused_metadata = Sink::new(1, 0);
    assert_eq!(
        source
            .ast
            .items()
            .try_emit_export_entries(&mut refused_metadata),
        Err(Refusal::Visit)
    );
    assert_eq!(refused_metadata.visits, 1);
    assert_eq!(refused_metadata.name_records, 0);
}

fn spelling(symbol: surgeist_js_interner::Sym, interner: &Interner) -> JsString {
    JsString::from(interner.resolve_expect(symbol).utf16())
}
