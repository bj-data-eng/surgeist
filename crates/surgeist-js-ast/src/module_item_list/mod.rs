//! Module item list AST nodes.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-modules

use crate::{
    StatementListItem,
    declaration::{
        ExportDeclaration, ExportEntry, ExportSpecifier, ImportDeclaration, ImportEntry,
        ReExportKind,
    },
    operations::{BoundNamesVisitor, NameAdmission, NameCollection, NameWork, unwrap_infallible},
    visitor::{VisitWith, Visitor, VisitorMut},
};
use indexmap::IndexSet;
use rustc_hash::{FxHashSet, FxHasher};
use std::{hash::BuildHasherDefault, ops::ControlFlow};
use surgeist_js_interner::Sym;

mod metadata;
pub use metadata::{ModuleEntryRef, ModuleEntryWork, ModuleMetadataSink};

/// Module item list AST node.
///
/// It contains a list of module items.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ModuleItemList
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModuleItemList {
    items: Box<[ModuleItem]>,
}

impl ModuleItemList {
    /// Gets the list of module items.
    #[inline]
    #[must_use]
    pub const fn items(&self) -> &[ModuleItem] {
        &self.items
    }

    /// Abstract operation [`ExportedNames`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-exportednames
    #[inline]
    #[must_use]
    pub fn exported_names(&self) -> Vec<Sym> {
        unwrap_infallible(self.collect_exported_names(|_| Ok(())))
    }

    /// Collects exported names in declaration order with optional work admission.
    ///
    /// # Errors
    ///
    /// Returns the first error from the admission callback.
    pub fn try_exported_names<E>(&self, admission: NameAdmission<'_, E>) -> Result<Vec<Sym>, E> {
        match admission {
            Some(admit) => self.collect_exported_names(admit),
            None => Ok(self.exported_names()),
        }
    }

    fn collect_exported_names<E>(
        &self,
        admit: impl FnMut(NameWork) -> Result<(), E>,
    ) -> Result<Vec<Sym>, E> {
        struct ExportedNamesVisitor<'a, F, E>(&'a mut NameCollection<Vec<Sym>, F, E>)
        where
            F: FnMut(NameWork) -> Result<(), E>;

        impl<'ast, F, E> Visitor<'ast> for ExportedNamesVisitor<'_, F, E>
        where
            F: FnMut(NameWork) -> Result<(), E>,
        {
            type BreakTy = ();

            fn visit_module_item_list(
                &mut self,
                node: &'ast ModuleItemList,
            ) -> ControlFlow<Self::BreakTy> {
                self.0.visit()?;
                node.visit_with(self)
            }

            fn visit_module_item(&mut self, node: &'ast ModuleItem) -> ControlFlow<Self::BreakTy> {
                self.0.visit()?;
                node.visit_with(self)
            }

            fn visit_import_declaration(
                &mut self,
                _: &'ast ImportDeclaration,
            ) -> ControlFlow<Self::BreakTy> {
                self.0.visit()
            }
            fn visit_statement_list_item(
                &mut self,
                _: &'ast StatementListItem,
            ) -> ControlFlow<Self::BreakTy> {
                self.0.visit()
            }
            fn visit_export_specifier(
                &mut self,
                node: &'ast ExportSpecifier,
            ) -> ControlFlow<Self::BreakTy> {
                self.0.visit()?;
                self.0.add(node.alias(), false)
            }
            fn visit_export_declaration(
                &mut self,
                node: &'ast ExportDeclaration,
            ) -> ControlFlow<Self::BreakTy> {
                self.0.visit()?;
                match node {
                    ExportDeclaration::ReExport { kind, .. } => {
                        match kind {
                            ReExportKind::Namespaced { name: Some(name) } => {
                                self.0.add(*name, false)?;
                            }
                            ReExportKind::Namespaced { name: None } => {}
                            ReExportKind::Named { names } => {
                                for specifier in &**names {
                                    self.visit_export_specifier(specifier)?;
                                }
                            }
                        }
                        ControlFlow::Continue(())
                    }
                    ExportDeclaration::List(list) => {
                        for specifier in &**list {
                            self.visit_export_specifier(specifier)?;
                        }
                        ControlFlow::Continue(())
                    }
                    ExportDeclaration::VarStatement(var) => {
                        BoundNamesVisitor(self.0).visit_var_declaration(var)
                    }
                    ExportDeclaration::Declaration(decl) => {
                        BoundNamesVisitor(self.0).visit_declaration(decl)
                    }
                    ExportDeclaration::DefaultFunctionDeclaration(_)
                    | ExportDeclaration::DefaultGeneratorDeclaration(_)
                    | ExportDeclaration::DefaultAsyncFunctionDeclaration(_)
                    | ExportDeclaration::DefaultAsyncGeneratorDeclaration(_)
                    | ExportDeclaration::DefaultClassDeclaration(_)
                    | ExportDeclaration::DefaultAssignmentExpression(_) => {
                        self.0.add(Sym::DEFAULT, false)
                    }
                }
            }
        }

        let mut collection = NameCollection::new(Vec::new(), admit);
        let _ = ExportedNamesVisitor(&mut collection).visit_module_item_list(self);
        collection.finish()
    }

    /// Abstract operation [`ExportedBindings`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-exportedbindings
    #[inline]
    #[must_use]
    pub fn exported_bindings(&self) -> FxHashSet<Sym> {
        unwrap_infallible(self.collect_exported_bindings(|_| Ok(())))
    }

    /// Collects local exported bindings with optional work admission.
    ///
    /// # Errors
    ///
    /// Returns the first error from the admission callback.
    pub fn try_exported_bindings<E>(
        &self,
        admission: NameAdmission<'_, E>,
    ) -> Result<FxHashSet<Sym>, E> {
        match admission {
            Some(admit) => self.collect_exported_bindings(admit),
            None => Ok(self.exported_bindings()),
        }
    }

    fn collect_exported_bindings<E>(
        &self,
        admit: impl FnMut(NameWork) -> Result<(), E>,
    ) -> Result<FxHashSet<Sym>, E> {
        struct ExportedBindingsVisitor<'a, F, E>(&'a mut NameCollection<FxHashSet<Sym>, F, E>)
        where
            F: FnMut(NameWork) -> Result<(), E>;

        impl<'ast, F, E> Visitor<'ast> for ExportedBindingsVisitor<'_, F, E>
        where
            F: FnMut(NameWork) -> Result<(), E>,
        {
            type BreakTy = ();

            fn visit_module_item_list(
                &mut self,
                node: &'ast ModuleItemList,
            ) -> ControlFlow<Self::BreakTy> {
                self.0.visit()?;
                node.visit_with(self)
            }

            fn visit_module_item(&mut self, node: &'ast ModuleItem) -> ControlFlow<Self::BreakTy> {
                self.0.visit()?;
                node.visit_with(self)
            }

            fn visit_import_declaration(
                &mut self,
                _: &'ast ImportDeclaration,
            ) -> ControlFlow<Self::BreakTy> {
                self.0.visit()
            }
            fn visit_statement_list_item(
                &mut self,
                _: &'ast StatementListItem,
            ) -> ControlFlow<Self::BreakTy> {
                self.0.visit()
            }
            fn visit_export_specifier(
                &mut self,
                node: &'ast ExportSpecifier,
            ) -> ControlFlow<Self::BreakTy> {
                self.0.visit()?;
                self.0.add(node.private_name(), false)
            }
            fn visit_export_declaration(
                &mut self,
                node: &'ast ExportDeclaration,
            ) -> ControlFlow<Self::BreakTy> {
                self.0.visit()?;
                let name = match node {
                    ExportDeclaration::ReExport { .. } => return ControlFlow::Continue(()),
                    ExportDeclaration::List(list) => {
                        for specifier in &**list {
                            self.visit_export_specifier(specifier)?;
                        }
                        return ControlFlow::Continue(());
                    }
                    ExportDeclaration::DefaultAssignmentExpression(expr) => {
                        return BoundNamesVisitor(self.0).visit_expression(expr);
                    }
                    ExportDeclaration::VarStatement(var) => {
                        return BoundNamesVisitor(self.0).visit_var_declaration(var);
                    }
                    ExportDeclaration::Declaration(decl) => {
                        return BoundNamesVisitor(self.0).visit_declaration(decl);
                    }
                    ExportDeclaration::DefaultFunctionDeclaration(f) => f.name(),
                    ExportDeclaration::DefaultGeneratorDeclaration(g) => g.name(),
                    ExportDeclaration::DefaultAsyncFunctionDeclaration(af) => af.name(),
                    ExportDeclaration::DefaultAsyncGeneratorDeclaration(ag) => ag.name(),
                    ExportDeclaration::DefaultClassDeclaration(cl) => cl.name(),
                };

                self.0.add(name.sym(), false)
            }
        }

        let mut collection = NameCollection::new(FxHashSet::default(), admit);
        let _ = ExportedBindingsVisitor(&mut collection).visit_module_item_list(self);
        collection.finish()
    }

    /// Operation [`ModuleRequests`][spec].
    ///
    /// Gets the list of modules that need to be fetched by the module resolver to link this module.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-modulerequests
    #[inline]
    #[must_use]
    pub fn requests(&self) -> IndexSet<Sym, BuildHasherDefault<FxHasher>> {
        metadata::requests(self)
    }

    /// Operation [`ImportEntries`][spec].
    ///
    /// Gets the list of import entries of this module.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-importentries
    #[inline]
    #[must_use]
    pub fn import_entries(&self) -> Vec<ImportEntry> {
        metadata::imports(self)
    }

    /// Operation [`ExportEntries`][spec].
    ///
    /// Gets the list of export entries of this module.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-exportentries
    #[inline]
    #[must_use]
    pub fn export_entries(&self) -> Vec<ExportEntry> {
        metadata::exports(self)
    }
}

impl<T> From<T> for ModuleItemList
where
    T: Into<Box<[ModuleItem]>>,
{
    #[inline]
    fn from(items: T) -> Self {
        Self {
            items: items.into(),
        }
    }
}

impl VisitWith for ModuleItemList {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        for item in &*self.items {
            visitor.visit_module_item(item)?;
        }

        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        for item in &mut *self.items {
            visitor.visit_module_item_mut(item)?;
        }

        ControlFlow::Continue(())
    }
}

/// Module item AST node.
///
/// This is an extension over a [`StatementList`](crate::StatementList), which can also include
/// multiple [`ImportDeclaration`] and [`ExportDeclaration`] nodes, along with
/// [`StatementListItem`] nodes.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ModuleItem
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum ModuleItem {
    /// See [`ImportDeclaration`].
    ImportDeclaration(ImportDeclaration),
    /// See [`ExportDeclaration`].
    ExportDeclaration(Box<ExportDeclaration>),
    /// See [`StatementListItem`].
    StatementListItem(StatementListItem),
}

impl VisitWith for ModuleItem {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        match self {
            Self::ImportDeclaration(i) => visitor.visit_import_declaration(i),
            Self::ExportDeclaration(e) => visitor.visit_export_declaration(e),
            Self::StatementListItem(s) => visitor.visit_statement_list_item(s),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        match self {
            Self::ImportDeclaration(i) => visitor.visit_import_declaration_mut(i),
            Self::ExportDeclaration(e) => visitor.visit_export_declaration_mut(e),
            Self::StatementListItem(s) => visitor.visit_statement_list_item_mut(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declaration::ModuleSpecifier;
    use std::convert::Infallible;

    #[test]
    fn export_name_collection_distinguishes_aliases_from_local_bindings() {
        let module = ModuleItemList::from(vec![
            ModuleItem::ExportDeclaration(Box::new(ExportDeclaration::List(Box::new([
                ExportSpecifier::new(Sym::TRUE, Sym::NULL, false),
                ExportSpecifier::new(Sym::FALSE, Sym::NULL, false),
            ])))),
            ModuleItem::ExportDeclaration(Box::new(ExportDeclaration::ReExport {
                kind: ReExportKind::Namespaced {
                    name: Some(Sym::DEFAULT),
                },
                specifier: ModuleSpecifier::new(Sym::NULL),
                attributes: Box::new([]),
            })),
        ]);

        assert_eq!(
            module.exported_names(),
            [Sym::TRUE, Sym::FALSE, Sym::DEFAULT]
        );
        assert_eq!(
            module.exported_bindings(),
            FxHashSet::from_iter([Sym::NULL])
        );

        let mut records = 0;
        let mut count_records = |work| {
            if matches!(work, NameWork::Record) {
                records += 1;
            }
            Ok::<(), Infallible>(())
        };
        let bindings = module.try_exported_bindings(Some(&mut count_records));
        assert_eq!(bindings, Ok(FxHashSet::from_iter([Sym::NULL])));
        assert_eq!(records, 2);

        let mut refused = false;
        let mut refuse_record = |work| {
            assert!(!refused, "work was requested after refusal");
            if matches!(work, NameWork::Record) {
                refused = true;
                return Err("export record refused");
            }
            Ok(())
        };
        assert_eq!(
            module.try_exported_names(Some(&mut refuse_record)),
            Err("export record refused")
        );
    }
}
