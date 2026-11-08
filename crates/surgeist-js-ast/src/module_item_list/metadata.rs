//! Borrowed module metadata emission shared by ordinary and admitted consumers.

use super::{ModuleItem, ModuleItemList};
use crate::{
    declaration::{
        ExportDeclaration, ExportEntry, ImportAttribute, ImportEntry, ImportKind, ImportName,
        IndirectExportEntry, LocalExportEntry, ReExportImportName, ReExportKind,
    },
    expression::Identifier,
    operations::{NameWork, try_bound_names},
};
use std::convert::Infallible;
use surgeist_js_interner::Sym;

/// Work performed before a borrowed module metadata event is emitted.
#[derive(Clone, Copy, Debug)]
pub enum ModuleEntryWork {
    /// One module-list, item, or explicit specifier slot inspection.
    Visit,
    /// Work delegated to the existing name-operation producer.
    Name(NameWork),
}

/// One borrowed result of module request, import, or export static semantics.
#[derive(Clone, Copy, Debug)]
pub enum ModuleEntryRef<'ast> {
    /// A source-ordered request, including its complete authored attributes.
    Request {
        /// Interned module specifier.
        specifier: Sym,
        /// Authored import attributes.
        attributes: &'ast [ImportAttribute],
    },
    /// One import binding.
    Import {
        /// Interned module specifier.
        specifier: Sym,
        /// Authored import attributes.
        attributes: &'ast [ImportAttribute],
        /// Imported binding or namespace.
        import_name: ImportName,
        /// Local binding, retaining its source identity.
        local_name: Identifier,
    },
    /// One local export binding.
    LocalExport {
        /// Local binding name.
        local_name: Sym,
        /// Public export name.
        export_name: Sym,
    },
    /// One star export without an alias.
    StarExport {
        /// Interned module specifier.
        specifier: Sym,
        /// Authored import attributes.
        attributes: &'ast [ImportAttribute],
    },
    /// One indirect export binding or namespace alias.
    IndirectExport {
        /// Interned module specifier.
        specifier: Sym,
        /// Authored import attributes.
        attributes: &'ast [ImportAttribute],
        /// Imported binding or namespace.
        import_name: ReExportImportName,
        /// Public export name.
        export_name: Sym,
    },
}

/// A single mutable consumer of borrowed metadata and producer admission.
///
/// No owned attribute arrays or spellings are constructed by emission. The
/// consumer controls their preparation; delegated name work retains its own
/// events. Returning an error stops before later inspections or emissions.
pub trait ModuleMetadataSink<'ast> {
    /// Failure returned by admission or output preparation.
    type Error;
    /// Admits the producer's next inspection.
    fn admit(&mut self, work: ModuleEntryWork) -> Result<(), Self::Error>;
    /// Consumes one borrowed semantic result.
    fn emit(&mut self, entry: ModuleEntryRef<'ast>) -> Result<(), Self::Error>;
}

impl ModuleItemList {
    /// Emits requests in source order, retaining attributes and duplicates.
    ///
    /// # Errors
    /// Returns the first consumer refusal before inspecting later items.
    pub fn try_emit_requests<'ast, S: ModuleMetadataSink<'ast>>(
        &'ast self,
        sink: &mut S,
    ) -> Result<(), S::Error> {
        sink.admit(ModuleEntryWork::Visit)?;
        for item in self.items() {
            sink.admit(ModuleEntryWork::Visit)?;
            let (specifier, attributes) = match item {
                ModuleItem::ImportDeclaration(import) => (import.specifier(), import.attributes()),
                ModuleItem::ExportDeclaration(export) => match &**export {
                    ExportDeclaration::ReExport {
                        specifier,
                        attributes,
                        ..
                    } => (*specifier, attributes.as_ref()),
                    _ => continue,
                },
                ModuleItem::StatementListItem(_) => continue,
            };
            sink.emit(ModuleEntryRef::Request {
                specifier: specifier.sym(),
                attributes,
            })?;
        }
        Ok(())
    }

    /// Emits import bindings without copying their attributes.
    ///
    /// # Errors
    /// Returns the first consumer refusal before inspecting later slots.
    pub fn try_emit_import_entries<'ast, S: ModuleMetadataSink<'ast>>(
        &'ast self,
        sink: &mut S,
    ) -> Result<(), S::Error> {
        sink.admit(ModuleEntryWork::Visit)?;
        for item in self.items() {
            sink.admit(ModuleEntryWork::Visit)?;
            let ModuleItem::ImportDeclaration(import) = item else {
                continue;
            };
            let specifier = import.specifier().sym();
            let attributes = import.attributes();
            if let Some(local_name) = import.default() {
                sink.emit(ModuleEntryRef::Import {
                    specifier,
                    attributes,
                    import_name: ImportName::Name(Sym::DEFAULT),
                    local_name,
                })?;
            }
            match import.kind() {
                ImportKind::DefaultOrUnnamed => {}
                ImportKind::Namespaced { binding } => {
                    sink.emit(ModuleEntryRef::Import {
                        specifier,
                        attributes,
                        import_name: ImportName::Namespace,
                        local_name: *binding,
                    })?;
                }
                ImportKind::Named { names } => {
                    for name in &**names {
                        sink.admit(ModuleEntryWork::Visit)?;
                        sink.emit(ModuleEntryRef::Import {
                            specifier,
                            attributes,
                            import_name: ImportName::Name(name.export_name()),
                            local_name: name.binding(),
                        })?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Emits export entries using the same bound-name producer as ordinary collection.
    ///
    /// # Errors
    /// Returns the first inspection, name-operation, or output refusal.
    pub fn try_emit_export_entries<'ast, S: ModuleMetadataSink<'ast>>(
        &'ast self,
        sink: &mut S,
    ) -> Result<(), S::Error> {
        sink.admit(ModuleEntryWork::Visit)?;
        for item in self.items() {
            sink.admit(ModuleEntryWork::Visit)?;
            let ModuleItem::ExportDeclaration(export) = item else {
                continue;
            };
            let local_name = match &**export {
                ExportDeclaration::ReExport {
                    kind,
                    specifier,
                    attributes,
                } => {
                    let specifier = specifier.sym();
                    let attributes = attributes.as_ref();
                    match kind {
                        ReExportKind::Namespaced {
                            name: Some(export_name),
                        } => sink.emit(ModuleEntryRef::IndirectExport {
                            specifier,
                            attributes,
                            import_name: ReExportImportName::Star,
                            export_name: *export_name,
                        })?,
                        ReExportKind::Namespaced { name: None } => {
                            sink.emit(ModuleEntryRef::StarExport {
                                specifier,
                                attributes,
                            })?
                        }
                        ReExportKind::Named { names } => {
                            for name in &**names {
                                sink.admit(ModuleEntryWork::Visit)?;
                                sink.emit(ModuleEntryRef::IndirectExport {
                                    specifier,
                                    attributes,
                                    import_name: ReExportImportName::Name(name.private_name()),
                                    export_name: name.alias(),
                                })?;
                            }
                        }
                    }
                    continue;
                }
                ExportDeclaration::List(names) => {
                    for name in &**names {
                        sink.admit(ModuleEntryWork::Visit)?;
                        sink.emit(ModuleEntryRef::LocalExport {
                            local_name: name.private_name(),
                            export_name: name.alias(),
                        })?;
                    }
                    continue;
                }
                ExportDeclaration::VarStatement(var) => {
                    let names = try_bound_names(
                        var,
                        Some(&mut |work| sink.admit(ModuleEntryWork::Name(work))),
                    )?;
                    for name in names {
                        sink.emit(ModuleEntryRef::LocalExport {
                            local_name: name,
                            export_name: name,
                        })?;
                    }
                    continue;
                }
                ExportDeclaration::Declaration(decl) => {
                    let names = try_bound_names(
                        decl,
                        Some(&mut |work| sink.admit(ModuleEntryWork::Name(work))),
                    )?;
                    for name in names {
                        sink.emit(ModuleEntryRef::LocalExport {
                            local_name: name,
                            export_name: name,
                        })?;
                    }
                    continue;
                }
                ExportDeclaration::DefaultFunctionDeclaration(f) => f.name().sym(),
                ExportDeclaration::DefaultGeneratorDeclaration(f) => f.name().sym(),
                ExportDeclaration::DefaultAsyncFunctionDeclaration(f) => f.name().sym(),
                ExportDeclaration::DefaultAsyncGeneratorDeclaration(f) => f.name().sym(),
                ExportDeclaration::DefaultClassDeclaration(c) => c.name().sym(),
                ExportDeclaration::DefaultAssignmentExpression(_) => Sym::DEFAULT_EXPORT,
            };
            sink.emit(ModuleEntryRef::LocalExport {
                local_name,
                export_name: Sym::DEFAULT,
            })?;
        }
        Ok(())
    }
}

#[derive(Default)]
struct OwnedEntries {
    requests: indexmap::IndexSet<Sym, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>,
    imports: Vec<ImportEntry>,
    exports: Vec<ExportEntry>,
}
impl<'ast> ModuleMetadataSink<'ast> for OwnedEntries {
    type Error = Infallible;
    fn admit(&mut self, _: ModuleEntryWork) -> Result<(), Infallible> {
        Ok(())
    }
    fn emit(&mut self, entry: ModuleEntryRef<'ast>) -> Result<(), Infallible> {
        match entry {
            ModuleEntryRef::Import {
                specifier,
                attributes,
                import_name,
                local_name,
            } => self.imports.push(ImportEntry::new(
                specifier,
                import_name,
                local_name,
                attributes.into(),
            )),
            ModuleEntryRef::LocalExport {
                local_name,
                export_name,
            } => self
                .exports
                .push(LocalExportEntry::new(local_name, export_name).into()),
            ModuleEntryRef::StarExport {
                specifier,
                attributes,
            } => self.exports.push(ExportEntry::StarReExport {
                module_request: specifier,
                attributes: attributes.into(),
            }),
            ModuleEntryRef::IndirectExport {
                specifier,
                attributes,
                import_name,
                export_name,
            } => self.exports.push(
                IndirectExportEntry::new(specifier, import_name, export_name, attributes.into())
                    .into(),
            ),
            ModuleEntryRef::Request { specifier, .. } => {
                self.requests.insert(specifier);
            }
        }
        Ok(())
    }
}
pub(super) fn imports(items: &ModuleItemList) -> Vec<ImportEntry> {
    let mut sink = OwnedEntries::default();
    super::unwrap_infallible(items.try_emit_import_entries(&mut sink));
    sink.imports
}
pub(super) fn exports(items: &ModuleItemList) -> Vec<ExportEntry> {
    let mut sink = OwnedEntries::default();
    super::unwrap_infallible(items.try_emit_export_entries(&mut sink));
    sink.exports
}

pub(super) fn requests(
    items: &ModuleItemList,
) -> indexmap::IndexSet<Sym, std::hash::BuildHasherDefault<rustc_hash::FxHasher>> {
    let mut sink = OwnedEntries::default();
    super::unwrap_infallible(items.try_emit_requests(&mut sink));
    sink.requests
}
